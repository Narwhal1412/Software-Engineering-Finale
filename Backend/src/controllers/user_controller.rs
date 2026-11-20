//! Endpoint người dùng: `/me` và `/admin/users`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};

use super::api_json::ApiJson;
use crate::app_state::AppState;
use crate::dtos::user_dto::{UpdateProfileRequest, UserResponse};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Router của feature người dùng.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me).put(update_me))
        .route("/admin/users", get(list_users))
        .route("/admin/users/:id", delete(delete_user))
}

/// `GET /me` -> 200 user.
async fn get_me(
    State(state): State<AppState>,
    actor: AuthUser,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(state.services.users.get_me(actor).await?))
}

/// `PUT /me` -> 200 user.
async fn update_me(
    State(state): State<AppState>,
    actor: AuthUser,
    ApiJson(body): ApiJson<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(state.services.users.update_me(actor, body).await?))
}

/// `GET /admin/users` -> 200 `[user]`.
async fn list_users(
    State(state): State<AppState>,
    actor: AuthUser,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    Ok(Json(state.services.users.list_users(actor).await?))
}

/// `DELETE /admin/users/:id` -> 204.
async fn delete_user(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    state.services.users.delete_user(actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
