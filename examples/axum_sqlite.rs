use axum::{
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use env_logger::Env;
use firebase_auth::{FirebaseAuth, FirebaseAuthState, FirebaseUser};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct Visit {
    pub id: i64,
    pub user_id: String,
    pub timestamp: String,
}

async fn public() -> &'static str {
    "ok"
}

const DB_URL: &str = "sqlite://sqlite.db";

async fn init_database() -> SqlitePool {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let db = SqlitePool::connect(DB_URL).await.unwrap();
    let result = sqlx::query(
        "CREATE TABLE IF NOT EXISTS visit (id INTEGER PRIMARY KEY NOT NULL, user_id VARCHAR(32), timestamp VARCHAR(250) NOT NULL);"
    ).execute(&db).await.unwrap();
    println!("Create visit table result: {:?}", result);

    db
}

pub async fn new_visit(
    user: FirebaseUser,
    Extension(pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    let user_id = user.user_id;
    let timestamp = Utc::now().to_string();

    let sql = "INSERT INTO visit (user_id, timestamp) values ($1, $2) RETURNING *";

    let result: Result<Visit, sqlx::Error> = sqlx::query_as(sql)
        .bind(user_id.clone())
        .bind(timestamp.clone())
        .fetch_one(&pool)
        .await;

    match result {
        Ok(taskwithid) => (StatusCode::CREATED, Json(taskwithid)),
        Err(err) => {
            tracing::error!("could not create task. error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Visit {
                    id: 0,
                    user_id,
                    timestamp,
                }),
            )
        }
    }
}

pub async fn my_visits(
    user: FirebaseUser,
    Extension(pool): Extension<SqlitePool>,
) -> impl IntoResponse {
    let user_id = user.user_id;

    let sql = "SELECT id, user_id, timestamp FROM visit WHERE user_id=$1".to_string();

    let result: Result<Vec<Visit>, sqlx::Error> = sqlx::query_as::<_, Visit>(&sql)
        .bind(user_id)
        .fetch_all(&pool)
        .await;

    match result {
        Ok(tasks) => (StatusCode::OK, Json(tasks)),
        Err(err) => {
            tracing::error!("error retrieving tasks: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::<Visit>::new()))
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let pool = init_database().await;

    let firebase_auth = FirebaseAuth::new("my-project-id").await;

    let app = Router::new()
        .route("/", get(public))
        .route("/visit", post(new_visit))
        .route("/visit", get(my_visits))
        .layer(Extension(pool))
        .with_state(FirebaseAuthState { firebase_auth });

    let addr = &"127.0.0.1:8080".parse().expect("Cannot parse the addr");
    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
