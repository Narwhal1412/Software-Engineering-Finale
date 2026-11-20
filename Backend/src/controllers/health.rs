use axum::{Json, extract::State};
use crate::{
        db::AppState,
        services::health::{self, HealthResponse}
};

// Endpoint Function: Receive HTTP request + shared AppState, return real JSON
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let report = health::get_system_health(&state).await;
    Json(report)
}
