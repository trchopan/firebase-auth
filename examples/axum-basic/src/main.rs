use axum::{routing::get, Router};
use firebase_auth::{FirebaseAuth, FirebaseAuthState, FirebaseUser};
use std::env;

async fn greet(user: FirebaseUser) -> String {
    let email = user.email.unwrap_or("empty email".to_string());
    format!("hello {}", email)
}

async fn public() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let project_id = env::var("PROJECT_ID").unwrap_or_else(|_| panic!("must set PROJECT_ID"));
    let firebase_auth = FirebaseAuth::new(&project_id).await;

    let app = Router::new()
        .route("/hello", get(greet))
        .route("/", get(public))
        .with_state(FirebaseAuthState::new(firebase_auth));

    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
