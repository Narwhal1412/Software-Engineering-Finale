//! Endpoint thống kê: `/admin/stats`.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::dtos::stats_dto::StatsResponse;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Router của feature thống kê.
pub fn router() -> Router<AppState> {
    Router::new().route("/admin/stats", get(overview))
}

/// `GET /admin/stats` -> 200 `{products, users, orders, revenue}`.
async fn overview(
    State(state): State<AppState>,
    actor: AuthUser,
) -> Result<Json<StatsResponse>, AppError> {
    Ok(Json(state.services.stats.overview(actor).await?))
}
