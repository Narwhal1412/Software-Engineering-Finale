//! Endpoint danh mục: `/categories`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};

use super::api_json::ApiJson;
use crate::app_state::AppState;
use crate::dtos::category_dto::{CategoryRequest, CategoryResponse};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Router của feature danh mục.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/categories", get(list).post(create))
        .route("/categories/:id", put(rename).delete(remove))
}

/// `GET /categories` (công khai) -> 200 `[{id, name}]`.
async fn list(State(state): State<AppState>) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    Ok(Json(state.services.categories.list().await?))
}

/// `POST /categories` -> 201 `{id, name}`.
async fn create(
    State(state): State<AppState>,
    actor: AuthUser,
    ApiJson(body): ApiJson<CategoryRequest>,
) -> Result<(StatusCode, Json<CategoryResponse>), AppError> {
    let category = state.services.categories.create(actor, body).await?;
    Ok((StatusCode::CREATED, Json(category)))
}

/// `PUT /categories/:id` -> 200 `{id, name}`.
async fn rename(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(id): Path<i64>,
    ApiJson(body): ApiJson<CategoryRequest>,
) -> Result<Json<CategoryResponse>, AppError> {
    Ok(Json(state.services.categories.rename(actor, id, body).await?))
}

/// `DELETE /categories/:id` -> 204.
async fn remove(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    state.services.categories.delete(actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
