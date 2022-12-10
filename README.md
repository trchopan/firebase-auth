<div align="center">
  <h1>Firebase Auth</h1>
  <p>A simple and small Rust Actix web framework Extractor for verifing JWT token from Firebase Authentication.</p>
</div>

## Example

Dependencies:
```toml
[dependencies]
actix-web = "4"
```

Code:

[Basic](https://github.com/trchopan/firebase-auth/tree/main/examples/basic.rs)

```rust
use actix_web::{get, middleware::Logger, web::Data, App, HttpServer, Responder};
use env_logger::Env;
use firebase_auth::{FirebaseAuth, FirebaseUser};

// Use `FirebaseUser` extractor to verify the user token and decode the claims
#[get("/hello")]
async fn greet(user: FirebaseUser) -> impl Responder {
    format!("Hello {}!", user.email)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    // Create Application State for the `FirebaseAuth` it will refresh the public keys
    // automatically.
    // Change project_id to your Firebase Project ID
    // We put this in blocking because the first time it run, it will try to get the public keys
    // from Google endpoint, if it failed it will panic.
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
```

# Request the endpoint

## Obtain the Bearer token

Use firebase sdk to get the User Token.

For example: [getIdToken()](https://firebase.google.com/docs/reference/js/v8/firebase.User#getidtoken)

## Request the endpoint with Authorization Bearer

Make the request using the User's token. Note that it will expire so you will need to get it again if expired.

```
TOKEN="<paste your token here>"

curl --header 'Authorization: Bearer $TOKEN' http://127.0.0.1:8080/hello
```

## License

[MIT](https://opensource.org/licenses/MIT)

Copyright (c) 2022-, Quang Tran.
