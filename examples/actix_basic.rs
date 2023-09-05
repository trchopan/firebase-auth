use actix_web::{get, middleware::Logger, web::Data, App, HttpServer, Responder};
use env_logger::Env;
use firebase_auth::{FirebaseAuth, FirebaseUser};

#[get("/hello")]
async fn greet(user: FirebaseUser) -> impl Responder {
    let email = user.email.unwrap_or("empty email".to_string());
    format!("Hello {}!", email)
}

#[get("/public")]
async fn public() -> impl Responder {
    "ok"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let firebase_auth = FirebaseAuth::new("my-project-id").await;

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
