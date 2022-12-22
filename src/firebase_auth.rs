use actix_web::error::ErrorUnauthorized;
use actix_web::{dev, http::header::Header, web, Error, FromRequest, HttpRequest};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use futures::future::{err, ok, Ready};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use std::{
    sync::{
        mpsc::{self, TryRecvError},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use tracing::*;

use crate::structs::{FirebaseUser, JwkConfiguration, JwkKeys, KeyResponse, PublicKeysError};

const FALLBACK_TIMEOUT: Duration = Duration::from_secs(60);
const JWK_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

pub fn get_configuration(project_id: &str) -> JwkConfiguration {
    JwkConfiguration {
        jwk_url: JWK_URL.to_owned(),
        audience: project_id.to_owned(),
        issuer: format!("https://securetoken.google.com/{}", project_id),
    }
}

fn parse_max_age_value(cache_control_value: &str) -> Result<Duration, PublicKeysError> {
    let tokens: Vec<(&str, &str)> = cache_control_value
        .split(',')
        .map(|s| s.split('=').map(|ss| ss.trim()).collect::<Vec<&str>>())
        .map(|ss| {
            let key = ss.first().unwrap_or(&"");
            let val = ss.get(1).unwrap_or(&"");
            (*key, *val)
        })
        .collect();
    match tokens
        .iter()
        .find(|(key, _)| key.to_lowercase() == *"max-age")
    {
        None => Err(PublicKeysError::NoMaxAgeSpecified),
        Some((_, str_val)) => Ok(Duration::from_secs(
            str_val
                .parse()
                .map_err(|_| PublicKeysError::NonNumericMaxAge)?,
        )),
    }
}

fn get_public_keys() -> Result<JwkKeys, PublicKeysError> {
    let response =
        reqwest::blocking::get(JWK_URL).map_err(|_| PublicKeysError::NoCacheControlHeader)?;

    let cache_control = match response.headers().get("Cache-Control") {
        Some(header_value) => header_value.to_str(),
        None => return Err(PublicKeysError::NoCacheControlHeader),
    };

    let max_age = match cache_control {
        Ok(v) => parse_max_age_value(v),
        Err(_) => return Err(PublicKeysError::MaxAgeValueEmpty),
    };

    let public_keys = response
        .json::<KeyResponse>()
        .map_err(|_| PublicKeysError::CannotParsePublicKey)?;

    Ok(JwkKeys {
        keys: public_keys.keys,
        max_age: max_age.unwrap_or(FALLBACK_TIMEOUT),
    })
}

#[derive(Debug)]
pub enum VerificationError {
    InvalidSignature,
    UnkownKeyAlgorithm,
    NoKidHeader,
    NotfoundMatchKid,
    CannotDecodePublicKeys,
}

fn verify_id_token_with_project_id(
    config: &JwkConfiguration,
    public_keys: &JwkKeys,
    token: &str,
) -> Result<FirebaseUser, VerificationError> {
    let header = decode_header(token).map_err(|_| VerificationError::UnkownKeyAlgorithm)?;

    if header.alg != Algorithm::RS256 {
        return Err(VerificationError::UnkownKeyAlgorithm);
    }

    let kid = match header.kid {
        Some(v) => v,
        None => return Err(VerificationError::NoKidHeader),
    };

    let public_key = match public_keys.keys.iter().find(|v| v.kid == kid) {
        Some(v) => v,
        None => return Err(VerificationError::NotfoundMatchKid),
    };
    let decoding_key = DecodingKey::from_rsa_components(&public_key.n, &public_key.e)
        .map_err(|_| VerificationError::CannotDecodePublicKeys)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.audience.to_owned()]);
    validation.set_issuer(&[config.issuer.to_owned()]);

    let user = decode::<FirebaseUser>(token, &decoding_key, &validation)
        .map_err(|_| VerificationError::InvalidSignature)?
        .claims;
    Ok(user)
}

#[derive(Debug)]
struct JwkVerifier {
    keys: JwkKeys,
    config: JwkConfiguration,
}

impl JwkVerifier {
    fn new(project_id: &str, keys: JwkKeys) -> JwkVerifier {
        JwkVerifier {
            keys,
            config: get_configuration(project_id),
        }
    }

    fn verify(&self, token: &str) -> Option<FirebaseUser> {
        match verify_id_token_with_project_id(&self.config, &self.keys, token) {
            Ok(token_data) => Some(token_data),
            _ => None,
        }
    }

    fn set_keys(&mut self, keys: JwkKeys) {
        self.keys = keys;
    }
}

type CleanupFn = Box<dyn Fn() + Send>;

/// Provide a service to automatically pull the new google public key based on the Cache-Control
/// header.
/// If there is an error during refreshing, automatically retry indefinitely every 10 seconds.
pub struct FirebaseAuth {
    verifier: Arc<Mutex<JwkVerifier>>,
    cleanup: Mutex<CleanupFn>,
}

impl Drop for FirebaseAuth {
    fn drop(&mut self) {
        // Stop the update thread when the updater is destructed
        let cleanup_fn = self.cleanup.lock().unwrap();
        cleanup_fn();
    }
}

impl FirebaseAuth {
    pub fn new(project_id: &str) -> FirebaseAuth {
        let jwk_keys: JwkKeys = match get_public_keys() {
            Ok(keys) => keys,
            Err(_) => {
                panic!("Unable to get public jwk keys! Cannot verify user tokens! Shutting down...")
            }
        };
        let verifier = Arc::new(Mutex::new(JwkVerifier::new(project_id, jwk_keys)));

        let mut instance = FirebaseAuth {
            verifier,
            cleanup: Mutex::new(Box::new(|| {})),
        };

        instance.start_key_update();
        instance
    }

    pub fn verify(&self, token: &str) -> Option<FirebaseUser> {
        let verifier = self.verifier.lock().unwrap();
        verifier.verify(token)
    }

    fn start_key_update(&mut self) {
        let verifier_ref = Arc::clone(&self.verifier);

        let stop = use_repeating_job(move || match get_public_keys() {
            Ok(jwk_keys) => {
                let mut verifier = verifier_ref.lock().unwrap();
                verifier.set_keys(jwk_keys.clone());
                debug!(
                    "Updated JWK keys. Next refresh will be in {:?}",
                    jwk_keys.max_age
                );
                jwk_keys.max_age
            }
            Err(err) => {
                warn!("Error getting public jwk keys {:?}", err);
                warn!("Re-try getting public keys in 10 seconds");
                Duration::from_secs(10)
            }
        });

        let mut cleanup = self.cleanup.lock().unwrap();
        *cleanup = stop;
    }
}

type Delay = Duration;
type Cancel = Box<dyn Fn() + Send>;

// Runs a given closure as a repeating job until the cancel callback is invoked.
// The jobs are run with a delay returned by the closure execution.
fn use_repeating_job<F>(job: F) -> Cancel
where
    F: Fn() -> Delay,
    F: Send + 'static,
{
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    thread::spawn(move || loop {
        let delay = job();
        thread::sleep(delay);

        if let Ok(_) | Err(TryRecvError::Disconnected) = shutdown_rx.try_recv() {
            break;
        }
    });

    Box::new(move || {
        info!("Stop pulling google public keys...");
        let _ = shutdown_tx.send("stop");
    })
}

fn get_bearer_token(header: &str) -> Option<String> {
    let prefix_len = "Bearer ".len();

    match header.len() {
        l if l < prefix_len => None,
        _ => Some(header[prefix_len..].to_string()),
    }
}

impl FromRequest for FirebaseUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let firebase_auth = req
            .app_data::<web::Data<FirebaseAuth>>()
            .expect("must init FirebaseAuth in Application Data. see description in https://crates.io/crates/firebase-auth");

        let bearer = match Authorization::<Bearer>::parse(req) {
            Err(e) => return err(e.into()),
            Ok(v) => get_bearer_token(&v.to_string()).unwrap_or_else(|| "".to_string()),
        };

        match firebase_auth.verify(&bearer) {
            None => err(ErrorUnauthorized("Please provide valid Authorization Bearer token")),
            Some(user) => ok(user),
        }
    }
}
