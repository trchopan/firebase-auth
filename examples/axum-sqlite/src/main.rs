use std::env;

use axum::{
    extract::{self, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_macros::FromRef;
use chrono::Utc;
use env_logger::Env;
use firebase_auth::{FirebaseAuth, FirebaseAuthState, FirebaseUser};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(FromRef, Clone)]
struct AppState {
    pool: SqlitePool,
    firebase_auth: FirebaseAuthState,
}

async fn public() -> &'static str {
    "ok"
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
struct Visit {
    id: i64,
    user_id: String,
    timestamp: String,
    detail: String,
}

const DB_URL: &str = "sqlite::memory:";

async fn init_database() -> SqlitePool {
    let db = SqlitePool::connect(DB_URL).await.unwrap();
    let result = sqlx::query(
        "CREATE TABLE IF NOT EXISTS visit (id INTEGER PRIMARY KEY NOT NULL, user_id VARCHAR(32), timestamp VARCHAR(25), detail VARCHAR(255) NOT NULL);"
    ).execute(&db).await.unwrap();
    println!("Create visit table result: {:?}", result);

    db
}

#[derive(Deserialize)]
struct NewVisit {
    detail: String,
}

async fn new_visit(
    user: FirebaseUser,
    State(pool): State<SqlitePool>,
    extract::Json(new_visit): extract::Json<NewVisit>,
) -> impl IntoResponse {
    let user_id = user.user_id;
    let timestamp = Utc::now().to_string();

    let sql = "INSERT INTO visit (user_id, timestamp, detail) values ($1, $2, $3) RETURNING *";

    sqlx::query_as(sql)
        .bind(user_id.clone())
        .bind(timestamp.clone())
        .bind(new_visit.detail.clone())
        .fetch_one(&pool)
        .await
        .map(|taskwithid: Visit| (StatusCode::CREATED, Json(taskwithid)))
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

async fn my_visits(user: FirebaseUser, State(pool): State<SqlitePool>) -> impl IntoResponse {
    let user_id = user.user_id;

    let sql = "SELECT id, user_id, timestamp, detail FROM visit WHERE user_id=$1".to_string();

    sqlx::query_as::<_, Visit>(&sql)
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .map(|visits: Vec<Visit>| (StatusCode::OK, Json(visits)))
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let pool = init_database().await;

    let project_id = env::var("PROJECT_ID").unwrap_or_else(|_| panic!("must set PROJECT_ID"));
    let firebase_auth = FirebaseAuth::new(&project_id).await;

    let state = AppState {
        pool,
        firebase_auth: FirebaseAuthState { firebase_auth },
    };

    let app = Router::new()
        .route("/", get(public))
        .route("/visit", post(new_visit))
        .route("/visit", get(my_visits))
        .with_state(state);

    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
