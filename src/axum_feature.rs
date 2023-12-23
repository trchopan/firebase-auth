#[cfg(all(feature = "axum", feature = "axum07"))]
compile_error!("feature \"axum\" and feature \"axum07\" cannot be enabled at the same time");

#[cfg(feature = "axum07")]
use axum07 as axum;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{self, request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use tracing::debug;

use crate::{FirebaseAuth, FirebaseUser};

#[derive(Clone)]
pub struct FirebaseAuthState {
    pub firebase_auth: FirebaseAuth,
}

impl FromRef<FirebaseAuthState> for FirebaseAuth {
    fn from_ref(state: &FirebaseAuthState) -> Self {
        state.firebase_auth.clone()
    }
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

        let bearer = get_bearer_token(auth_header);
        let bearer = if let Some(bearer) = bearer {
            bearer
        } else {
            return Err(UnauthorizedResponse {
                msg: "Missing Bearer Token".to_string(),
            });
        };

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
