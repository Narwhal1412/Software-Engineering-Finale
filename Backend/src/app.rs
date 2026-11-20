//! Lắp ráp router axum hoàn chỉnh (route, file tĩnh, layer).

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::services::ServeDir;

use crate::app_state::AppState;
use crate::controllers::build_router;
use crate::middleware::cors::cors_layer;
use crate::middleware::logging::http_trace_layer;

/// Dựng ứng dụng. Tách khỏi `main` để test tích hợp gọi trực tiếp được.
pub fn build_app(state: AppState) -> Router {
    // Frontend gọi `/uploads/...` tới backend (xem vite.config.js và nginx.conf).
    let uploads = ServeDir::new(state.config.upload_dir.clone());

    Router::new()
        .route("/health", get(health))
        .merge(build_router())
        .nest_service("/uploads", uploads)
        .layer(http_trace_layer())
        .layer(cors_layer(&state.config))
        .with_state(state)
}

/// Kiểm tra sống: luôn trả 200 khi tiến trình còn phục vụ được.
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
