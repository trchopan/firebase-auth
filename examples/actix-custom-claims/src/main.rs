use std::env;

use actix_web::error::ErrorUnauthorized;
use actix_web::{
    dev, get, http::header::Header, middleware::Logger, web, web::Data, App, Error, FromRequest,
    HttpRequest, HttpServer, Responder,
};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use firebase_auth::{FirebaseAuth, FirebaseProvider};
use futures::future::{err, ok, Ready};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct FirebaseUser {
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
    pub auth_time: u64,
    pub user_id: String,
    pub provider_id: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub firebase: FirebaseProvider,

    #[serde(rename = "https://hasura.io/jwt/claims")]
    pub hasura: HasuraClaims,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HasuraClaims {
    pub x_hasura_default_role: String,
    pub x_hasura_allowed_roles: Vec<String>,
    pub x_hasura_user_id: String,
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
            Ok(v) => get_bearer_token(&v.to_string()).unwrap_or_default(),
        };

        match firebase_auth.verify(&bearer) {
            Err(e) => err(ErrorUnauthorized(format!("Failed to verify Token {}", e))),
            Ok(user) => ok(user),
        }
    }
}

#[get("/hello")]
async fn greet(user: FirebaseUser) -> impl Responder {
    let hasura_user_id = user.hasura.x_hasura_user_id;
    format!("Hello user id {}!", hasura_user_id)
}

#[get("/public")]
async fn public() -> impl Responder {
    "ok"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let project_id = env::var("PROJECT_ID").unwrap_or_else(|_| panic!("must set PROJECT_ID"));
    let firebase_auth = FirebaseAuth::new(&project_id).await;

    let app_data = Data::new(firebase_auth);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_data.clone())
            .service(greet)
            .service(public)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
