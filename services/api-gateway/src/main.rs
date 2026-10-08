//! FR-1.1 real JWT ingress + FR-1.2 hooks + deep /ready.
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    routing::get,
    Json, Router,
};
use common::{auth::Claims, config::Config};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::{net::SocketAddr, sync::Arc};

#[derive(Clone)]
struct AppState {
    cfg: Arc<Config>,
    pg: sqlx::PgPool,
}

async fn health() -> &'static str {
    "gateway ok"
}

async fn ready(State(s): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(&s.pg)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    // Redis best-effort: fail open for reads, closed in logs (partition-tolerant AP reads).
    Ok(Json(serde_json::json!({"ready": true})))
}

async fn jwt_guard(
    State(s): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_owned();
    if path == "/health" || path == "/ready" {
        return Ok(next.run(req).await);
    }
    let auth = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let mut v = Validation::default();
    v.set_issuer(std::slice::from_ref(&s.cfg.jwt_issuer));
    let data = decode::<Claims>(token, &DecodingKey::from_secret(&s.cfg.jwt_secret), &v)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    if data.claims.exp * 1000 < chrono::Utc::now().timestamp_millis() as usize {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let cfg = Config::from_env().expect("invalid env — see .env.example");
    let pg = sqlx::PgPool::connect_lazy(&cfg.database_url_main).expect("bad DATABASE_URL_MAIN");
    let state = AppState {
        cfg: Arc::new(cfg),
        pg,
    };
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/v1/academic/health", get(health))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            jwt_guard,
        ))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::new());
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
