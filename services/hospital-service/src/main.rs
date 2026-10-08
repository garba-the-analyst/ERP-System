//! Hospital service: isolated PG + AES-256-GCM field encryption (NFR-3).
//! Real crypto lands Week 13; this skeleton exposes the boundary.
use axum::{routing::get, Router};
use std::net::SocketAddr;

async fn health() -> &'static str {
    "hospital ok (isolated pg + AES-256-GCM boundary)"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let app = Router::new().route("/health", get(health));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8084));
    let l = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(l, app).await.unwrap();
}
