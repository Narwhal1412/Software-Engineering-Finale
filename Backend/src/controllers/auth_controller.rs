//! Endpoint xác thực: `/auth/register`, `/auth/login`.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

use super::api_json::ApiJson;
use crate::app_state::AppState;
use crate::dtos::auth_dto::{AuthResponse, LoginRequest, RegisterRequest};
use crate::error::AppError;

/// Router của feature xác thực.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
}

/// `POST /auth/register` -> 201 `{user, token}`.
async fn register(
    State(state): State<AppState>,
    ApiJson(body): ApiJson<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AppError> {
    let response = state.services.auth.register(body).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// `POST /auth/login` -> 200 `{user, token}`.
async fn login(
    State(state): State<AppState>,
    ApiJson(body): ApiJson<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    Ok(Json(state.services.auth.login(body).await?))
}
