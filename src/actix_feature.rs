use actix_web::error::ErrorUnauthorized;
use actix_web::{dev, http::header::Header, web, Error, FromRequest, HttpRequest};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use futures::future::{err, ok, Ready};
use tracing::debug;

use crate::{FirebaseAuth, FirebaseUser};

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
            Ok(v) => get_bearer_token(&v.to_string()).unwrap_or_default(),
        };

        debug!("Got bearer token {}", bearer);

        match firebase_auth.verify(&bearer) {
            Err(e) => err(ErrorUnauthorized(format!("Failed to verify Token {}", e))),
            Ok(user) => ok(user),
        }
    }
}
