use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug)]
pub struct JwkConfiguration {
    pub jwk_url: String,
    pub audience: String,
    pub issuer: String,
}

#[derive(Debug, Deserialize)]
pub struct KeyResponse {
    pub keys: Vec<JwkKey>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JwkKey {
    pub e: String,
    pub alg: String,
    pub kty: String,
    pub kid: String,
    pub n: String,
}

/// The Jwt claims decoded from the user token. Can also be viewed as the Firebase User
/// information.
#[derive(Serialize, Deserialize)]
pub struct FirebaseUser {
    pub name: Option<String>,
    pub picture: Option<String>,
    pub iss: String,
    pub aud: String,
    pub auth_time: u64,
    pub user_id: String,
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
    pub email: String,
    pub email_verified: bool,
    pub firebase: FirebaseProvider,
}

#[derive(Serialize, Deserialize)]
pub struct FirebaseProvider {
    sign_in_provider: String,
    identities: Map<String, Value>,
}

#[derive(Debug, Clone)]
pub struct JwkKeys {
    pub keys: Vec<JwkKey>,
    pub max_age: Duration,
}

#[derive(Debug)]
pub enum PublicKeysError {
    NoCacheControlHeader,
    MaxAgeValueEmpty,
    NonNumericMaxAge,
    NoMaxAgeSpecified,
    CannotParsePublicKey,
}


