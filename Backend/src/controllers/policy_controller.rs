//! Endpoint chính sách và ưu đãi: `/policies`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};

use super::api_json::ApiJson;
use crate::app_state::AppState;
use crate::dtos::policy_dto::{PolicyRequest, PolicyResponse};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Router của feature chính sách.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/policies", get(list).post(create))
        .route("/policies/:id", put(update).delete(remove))
}

/// `GET /policies` (công khai) -> 200 `[policy]`, đã sắp theo thứ tự hiển thị.
async fn list(State(state): State<AppState>) -> Result<Json<Vec<PolicyResponse>>, AppError> {
    Ok(Json(state.services.policies.list().await?))
}

/// `POST /policies` (admin) -> 201 `policy`.
async fn create(
    State(state): State<AppState>,
    actor: AuthUser,
    ApiJson(body): ApiJson<PolicyRequest>,
) -> Result<(StatusCode, Json<PolicyResponse>), AppError> {
    let policy = state.services.policies.create(actor, body).await?;
    Ok((StatusCode::CREATED, Json(policy)))
}

/// `PUT /policies/:id` (admin) -> 200 `policy`.
async fn update(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(id): Path<i64>,
    ApiJson(body): ApiJson<PolicyRequest>,
) -> Result<Json<PolicyResponse>, AppError> {
    Ok(Json(state.services.policies.update(actor, id, body).await?))
}

/// `DELETE /policies/:id` (admin) -> 204.
async fn remove(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    state.services.policies.delete(actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
