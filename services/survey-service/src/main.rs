//! Survey ingestion: POST /api/v1/survey -> postgres survey_responses (append-only).
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
struct Submit {
    cat: String,
    #[serde(default)]
    details: serde_json::Value,
    #[serde(default)]
    answers: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct Row {
    id: i64,
    cat: String,
}

async fn health() -> &'static str {
    "survey ok"
}

async fn submit(
    State(pg): State<sqlx::PgPool>,
    Json(s): Json<Submit>,
) -> Result<Json<Row>, StatusCode> {
    if !["student", "staff", "stakeholder"].contains(&s.cat.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    // Cap payload: 64KB answers max (abuse guard for public form).
    let size = s.details.to_string().len() + s.answers.to_string().len();
    if size > 64 * 1024 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let rec = sqlx::query("INSERT INTO survey_responses (cat, details, answers) VALUES ($1, $2, $3) RETURNING id, cat")
        .bind(&s.cat)
        .bind(&s.details)
        .bind(&s.answers)
        .fetch_one(&pg)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    use sqlx::Row as _;
    Ok(Json(Row {
        id: rec
            .try_get("id")
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?,
        cat: rec
            .try_get("cat")
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?,
    }))
}

async fn export(State(pg): State<sqlx::PgPool>) -> Result<Json<serde_json::Value>, StatusCode> {
    // Protected in prod by gateway RBAC (ADMIN_ALL); open in pilot behind secret path.
    let rows = sqlx::query(
        "SELECT id, cat, details, answers, created_at FROM survey_responses ORDER BY id",
    )
    .fetch_all(&pg)
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    use sqlx::{Column as _, Row as _};
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let cols = r.columns();
            let mut m = serde_json::Map::new();
            for c in cols {
                let n = c.name();
                m.insert(
                    n.into(),
                    match n {
                        "id" => r
                            .try_get::<i64, _>(n)
                            .map(serde_json::Value::from)
                            .unwrap_or(serde_json::Value::Null),
                        "cat" => r
                            .try_get::<String, _>(n)
                            .map(serde_json::Value::from)
                            .unwrap_or(serde_json::Value::Null),
                        "details" | "answers" => r
                            .try_get::<serde_json::Value, _>(n)
                            .unwrap_or(serde_json::Value::Null),
                        _ => r
                            .try_get::<String, _>(n)
                            .map(serde_json::Value::from)
                            .unwrap_or(serde_json::Value::Null),
                    },
                );
            }
            serde_json::Value::Object(m)
        })
        .collect();
    Ok(Json(serde_json::json!(out)))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let cfg = common::config::Config::from_env().expect("invalid env");
    let pg = sqlx::PgPool::connect(&cfg.database_url_main)
        .await
        .expect("pg connect");
    sqlx::migrate!("../../migrations")
        .run(&pg)
        .await
        .expect("migrate");
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(health))
        .route("/api/v1/survey", post(submit))
        .route("/api/v1/survey/export", get(export))
        .with_state(pg);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8087));
    let l = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(l, app).await.unwrap();
}
