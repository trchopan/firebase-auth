use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{self, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use firebase_auth::{FirebaseAuth, FirebaseAuthState, FirebaseProvider};
use serde::{Deserialize, Serialize};
use tracing::debug;

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

#[async_trait]
impl<S> FromRequestParts<S> for FirebaseUser
where
    FirebaseAuthState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = UnauthorizedResponse;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let store = FirebaseAuthState::from_ref(state);

        let auth_header = parts
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");

        let bearer = get_bearer_token(auth_header).map_or(
            Err(UnauthorizedResponse {
                msg: "Missing Bearer Token".to_string(),
            }),
            Ok,
        )?;

        debug!("Got bearer token {}", bearer);

        match store.firebase_auth.verify(&bearer) {
            Err(e) => Err(UnauthorizedResponse {
                msg: format!("Failed to verify Token: {}", e),
            }),
            Ok(current_user) => Ok(current_user),
        }
    }
}

pub struct UnauthorizedResponse {
    msg: String,
}

impl IntoResponse for UnauthorizedResponse {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, self.msg).into_response()
    }
}

async fn greet(user: FirebaseUser) -> String {
    let email = user.email.unwrap_or("empty email".to_string());
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

    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
