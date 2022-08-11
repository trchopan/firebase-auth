use actix_web::{get, middleware::Logger, web::Data, App, HttpServer, Responder};
use env_logger::Env;
use firebase_auth::firebase_auth::{FirebaseAuth, FirebaseUser};

// Use `FirebaseUser` extractor to verify the user token and decode the claims
#[get("/hello")]
async fn greet(user: FirebaseUser) -> impl Responder {
    format!("Hello {}!", user.email)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    // create Application State for the `FirebaseAuth` it will refresh the public keys
    // automatically.
    // Change project_id to your Firebase [Project ID](https://firebase.google.com/docs/projects/learn-more#project-id)
    let firebase_auth = tokio::task::spawn_blocking(|| FirebaseAuth::new("my-project-id"))
        .await
        .expect("panic init FirebaseAuth");

    let app_data = Data::new(firebase_auth);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_data.clone())
            .service(greet)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
