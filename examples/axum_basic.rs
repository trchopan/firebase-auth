use axum::{routing::get, Router};
use firebase_auth::{FirebaseAuth, FirebaseAuthState, FirebaseUser};

async fn greeting(user: FirebaseUser) -> String {
    let email = user.email.unwrap_or("empty email".to_string());
    format!("hello {}", email)
}

async fn public() -> &'static str{
    "ok"
}

#[tokio::main]
async fn main() {
    let firebase_auth = tokio::task::spawn_blocking(|| FirebaseAuth::new("my-project-id"))
        .await
        .expect("panic init FirebaseAuth");

    let app = Router::new()
        .route("/hello", get(greeting))
        .route("/", get(public))
        .with_state(FirebaseAuthState { firebase_auth });

    let addr = &"127.0.0.1:8080".parse().expect("Cannot parse the addr");
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
