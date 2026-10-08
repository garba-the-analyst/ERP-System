//! FR-4.x raw-body HMAC-SHA512 webhook (Paystack/Remita) + idempotent tx_ref.
use axum::{
    body::Bytes,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Router,
};
use common::config::Config;
use std::{net::SocketAddr, sync::Arc};

async fn health() -> &'static str {
    "financial ok"
}
async fn ready() -> &'static str {
    "ready (deep pg check lands with ledger repo W9-12)"
}

/// MUST use raw bytes + header — never JSON-wrapped body (breaks HMAC).
async fn webhook(
    axum::extract::State(secret): axum::extract::State<Arc<String>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<&'static str, StatusCode> {
    let sig = headers
        .get("x-paystack-signature")
        .or_else(|| headers.get("x-remita-signature"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if common::finance::verify_hmac_sha512(secret.as_bytes(), &body, sig) {
        Ok("ack — apply idempotently on tx_ref UNIQUE (W9-12 ledger)")
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let cfg = Config::from_env().expect("invalid env");
    let secret = Arc::new(cfg.paystack_secret.clone());
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/webhook", post(webhook))
        .with_state(secret);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8083));
    let l = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(l, app).await.unwrap();
}
