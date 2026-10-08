//! FR-3.x Admissions: UTME/DE/Transfer-DE waiver audit + FR-3.3 matric numbers.
use axum::{
    routing::{get, post},
    Json, Router,
};
use common::matric::MatricAllocator;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};

async fn health() -> &'static str {
    "admissions ok"
}

#[derive(Deserialize)]
struct MatricReq {
    dept_code: String,
    intake_year: u16,
}
#[derive(Serialize)]
struct MatricResp {
    matric: String,
}

async fn next_matric(
    axum::extract::State(a): axum::extract::State<Arc<MatricAllocator>>,
    Json(r): Json<MatricReq>,
) -> Json<MatricResp> {
    Json(MatricResp {
        matric: a.next(&r.dept_code, r.intake_year),
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let alloc = Arc::new(MatricAllocator::new());
    let app = Router::new()
        .route("/health", get(health))
        .route("/matric/next", post(next_matric))
        .with_state(alloc);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8082));
    let l = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(l, app).await.unwrap();
}
