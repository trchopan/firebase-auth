<div align="center">
  <h1>Firebase Auth</h1>
    <p>A simple and small Rust library for handling Firebase Authorization.</p>
    <p>Supports the two most popular frameworks: Tokio's Axum and Actix-web.</p>
</div>

[![Build badge]][Build] [![crates.io badge]][crates.io]

[Build]: https://github.com/trchopan/firebase-auth/actions/workflows/rust.yml
[Build badge]: https://github.com/trchopan/firebase-auth/actions/workflows/rust.yml/badge.svg
[crates.io]: https://crates.io/crates/firebase-auth
[crates.io badge]: https://img.shields.io/crates/v/firebase-auth.svg?color=%23B48723

## Notice

Version 0.4.x supports Axum 0.7.

Version 0.3.x will continue to provide support and fix bugs for Axum 0.6.

## Setup

_Actix_

```toml
[dependencies]
firebase-auth = { version = "<version>", features = ["actix-web"] }
actix-web = "4"
```

_Axum_

```toml
[dependencies]
firebase-auth = { version = "<version>", features = ["axum"] }
axum = "0.7"
```

# Examples

## Actix

[https://github.com/trchopan/firebase-auth/tree/main/examples/actix_basic.rs](https://github.com/trchopan/firebase-auth/tree/main/examples/actix_basic.rs)

```rust
use actix_web::{get, middleware::Logger, web::Data, App, HttpServer, Responder};
use firebase_auth::{FirebaseAuth, FirebaseUser};

// Use `FirebaseUser` extractor to verify the user token and decode the claims
#[get("/hello")]
async fn greet(user: FirebaseUser) -> impl Responder {
    let email = user.email.unwrap_or("empty email".to_string());
    format!("Hello {}!", email)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let firebase_auth = FirebaseAuth::new("my-project-id").await;

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
```

## Axum

[https://github.com/trchopan/firebase-auth/tree/main/examples/axum_basic.rs](https://github.com/trchopan/firebase-auth/tree/main/examples/axum_basic.rs)

```rust
use axum::{routing::get, Router};
use firebase_auth::{FirebaseAuth, FirebaseAuthState, FirebaseUser};

async fn greet(current_user: FirebaseUser) -> String {
    let email = current_user.email.unwrap_or("empty email".to_string());
    format!("hello {}", email)
}

async fn public() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let firebase_auth = FirebaseAuth::new("my-project-id").await;

    let app = Router::new()
        .route("/hello", get(greet))
        .route("/", get(public))
        .with_state(FirebaseAuthState { firebase_auth });

    let addr = &"127.0.0.1:8080".parse().expect("Cannot parse the addr");
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
```

## More complete example with Axum, SQLite and slqx

[examples/axum_sqlite.rs](https://github.com/trchopan/firebase-auth/tree/main/examples/axum_sqlite.rs)

This is more real world application with Firebase Authentication and SQLite as database.

# How to call the endpoint with Bearer Token

## Obtain the Bearer token

Use firebase sdk to get the User Token.

For example: [getIdToken()](https://firebase.google.com/docs/reference/js/v8/firebase.User#getidtoken)

## Request the endpoint with Authorization Bearer

Make the request using the User's token. Note that it will expire so you will need to get it again if expired.

```
TOKEN="<paste your token here>"

curl --header "Authorization: Bearer $TOKEN" http://127.0.0.1:8080/hello
```

## License

[MIT](https://opensource.org/licenses/MIT)

Copyright (c) 2022-, Quang Tran.
