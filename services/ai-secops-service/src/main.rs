//! FR-5.x AI SecOps: llama3.2:3b copilot/tutor + qwen2.5:1.5b JSON risk scoring.
//! Ollama forced to `format: json` (risk mitigation); Rust validates schema.
use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

async fn health() -> &'static str {
    "ai-secops ok"
}

#[derive(Debug, Deserialize)]
struct LogBatch {
    logs: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RiskScore {
    risk_score: u8,
    threat_type: String,
    affected_endpoint: String,
    recommended_action: String,
}

async fn score(Json(b): Json<LogBatch>) -> Json<Vec<RiskScore>> {
    // Heuristic stub until Ollama wiring (Week 13-16). Real path: POST OLLAMA_URL/api/generate qwen2.5:1.5b format=json.
    let out = b
        .logs
        .iter()
        .map(|l| {
            let brute = l.contains("401") || l.to_lowercase().contains("failed login");
            RiskScore {
                risk_score: if brute { 88 } else { 5 },
                threat_type: if brute {
                    "Brute Force Authentication Spike".into()
                } else {
                    "Benign".into()
                },
                affected_endpoint: "/api/v1/auth/login".into(),
                recommended_action: if brute {
                    "ENFORCE_IP_THROTTLE".into()
                } else {
                    "NONE".into()
                },
            }
        })
        .collect();
    Json(out)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let app = Router::new()
        .route("/health", get(health))
        .route("/score", post(score));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8085));
    let l = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(l, app).await.unwrap();
}
