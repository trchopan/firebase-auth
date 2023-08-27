//! [Firebase](https://firebase.google.com) authentication layer for popular frameworks.
//!
//! Support:
//!
//! - [Axum](https://github.com/tokio-rs/axum)
//! - [Actix](https://github.com/actix/actix-web)
//!
//! ## Example:
//!
//! ### Actix
//!
//! ```rust
//! use actix_web::{get, middleware::Logger, web::Data, App, HttpServer, Responder};
//! use firebase_auth::{FirebaseAuth, FirebaseUser};
//!
//! #[get("/hello")]
//! async fn greet(user: FirebaseUser) -> impl Responder {
//!     let email = user.email.unwrap_or("empty email".to_string());
//!     format!("Hello {}!", email)
//! }
//!
//! #[get("/public")]
//! async fn public() -> impl Responder {
//!     "ok"
//! }
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     let firebase_auth = FirebaseAuth::new("my-project-id").await;
//!
//!     let app_data = Data::new(firebase_auth);
//!
//!     HttpServer::new(move || {
//!         App::new()
//!             .wrap(Logger::default())
//!             .app_data(app_data.clone())
//!             .service(greet)
//!             .service(public)
//!     })
//!     .bind(("127.0.0.1", 8080))?
//!     .run()
//!     .await
//! }
//! ```
//!
//! ### Axum
//!
//! ```rust
//! use axum::{routing::get, Router};
//! use firebase_auth::{FirebaseAuth, FirebaseAuthState, FirebaseUser};
//!
//! async fn greeting(user: FirebaseUser) -> String {
//!     let email = user.email.unwrap_or("empty email".to_string());
//!     format!("hello {}", email)
//! }
//!
//! async fn public() -> &'static str {
//!     "ok"
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let firebase_auth = FirebaseAuth::new("my-project-id").await;
//!
//!     let app = Router::new()
//!         .route("/hello", get(greeting))
//!         .route("/", get(public))
//!         .with_state(FirebaseAuthState { firebase_auth });
//!
//!     let addr = &"127.0.0.1:8080".parse().expect("Cannot parse the addr");
//!     axum::Server::bind(addr)
//!         .serve(app.into_make_service())
//!         .await
//!         .unwrap()
//! }
//! ```
//!
//!Visit [README.md](https://github.com/trchopan/firebase-auth/) for more details.

mod firebase_auth;
mod structs;

#[cfg(feature = "actix-web")]
mod actix_feature;

#[cfg(feature = "axum")]
mod axum_feature;

#[cfg(feature = "axum")]
pub use axum_feature::FirebaseAuthState;

pub use firebase_auth::FirebaseAuth;
pub use structs::{FirebaseUser, PublicKeysError};
