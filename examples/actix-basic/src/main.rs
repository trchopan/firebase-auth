use std::env;

use actix_web::{get, middleware::Logger, web::Data, App, HttpServer, Responder};
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
