//! FR-2.x Academic Engine HTTP surface (Week 6-8 full DB wiring; pure logic already in common::academic).
use axum::{
    routing::{get, post},
    Json, Router,
};
use common::academic::{
    carryovers_locked, evaluate_standing, validate_cart, Cohort, CourseEnrollment,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

async fn health() -> &'static str {
    "academic ok"
}

#[derive(Debug, Deserialize)]
struct RegisterReq {
    #[allow(dead_code)]
    cohort: String,
    #[allow(dead_code)]
    program_years: u8,
    history: Vec<CourseEnrollment>,
    cart: Vec<String>,
    prereqs: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize)]
struct RegisterResp {
    locked_carryovers: Vec<String>,
    valid: bool,
    error: Option<String>,
}

async fn validate_registration(Json(r): Json<RegisterReq>) -> Json<RegisterResp> {
    let locked = carryovers_locked(&r.history);
    match validate_cart(&r.history, &r.cart, &r.prereqs) {
        Ok(()) => Json(RegisterResp {
            locked_carryovers: locked,
            valid: true,
            error: None,
        }),
        Err(e) => Json(RegisterResp {
            locked_carryovers: locked,
            valid: false,
            error: Some(e),
        }),
    }
}

#[derive(Debug, Deserialize)]
struct StandingReq {
    cohort: String,
    program_years: u8,
    current_year: u16,
    enrollments: Vec<CourseEnrollment>,
    all_required_cleared: bool,
}

async fn standing(Json(r): Json<StandingReq>) -> Json<serde_json::Value> {
    let cohort = Cohort::parse(&r.cohort, r.program_years).unwrap();
    let s = evaluate_standing(
        cohort,
        r.current_year,
        &r.enrollments,
        r.all_required_cleared,
    );
    Json(serde_json::to_value(&s).unwrap())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let app = Router::new()
        .route("/health", get(health))
        .route("/validate-registration", post(validate_registration))
        .route("/standing", post(standing));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    tracing::info!("academic-service on {}", addr);
    let l = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(l, app).await.unwrap();
}

#[allow(dead_code)]
fn _use_hashset() {
    let _: HashSet<String> = HashSet::new();
}
