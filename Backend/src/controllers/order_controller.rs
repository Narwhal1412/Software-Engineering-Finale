//! Endpoint đơn hàng: `/orders` và `/admin/orders`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch};
use axum::{Json, Router};

use super::api_json::ApiJson;
use crate::app_state::AppState;
use crate::dtos::order_dto::{
    AdminOrderResponse, CreateOrderRequest, OrderResponse, UpdateOrderStatusRequest,
};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Router của feature đơn hàng.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/orders", get(list_mine).post(place_order))
        .route("/orders/:id/cancel", patch(cancel_order))
        .route("/admin/orders", get(list_all))
        .route("/admin/orders/:id", patch(update_status))
}

/// `POST /orders` -> 201 order.
async fn place_order(
    State(state): State<AppState>,
    actor: AuthUser,
    ApiJson(body): ApiJson<CreateOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), AppError> {
    let order = state.services.orders.place_order(actor, body).await?;
    Ok((StatusCode::CREATED, Json(order)))
}

/// `GET /orders` -> 200 `[order]` của người dùng hiện tại.
async fn list_mine(
    State(state): State<AppState>,
    actor: AuthUser,
) -> Result<Json<Vec<OrderResponse>>, AppError> {
    Ok(Json(state.services.orders.list_mine(actor).await?))
}

/// `PATCH /orders/:id/cancel` -> 200 order (khách tự hủy đơn `pending` của mình).
async fn cancel_order(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<OrderResponse>, AppError> {
    Ok(Json(state.services.orders.cancel_order(actor, id).await?))
}

/// `GET /admin/orders` -> 200 `[order + userEmail]`.
async fn list_all(
    State(state): State<AppState>,
    actor: AuthUser,
) -> Result<Json<Vec<AdminOrderResponse>>, AppError> {
    Ok(Json(state.services.orders.list_all(actor).await?))
}

/// `PATCH /admin/orders/:id` `{status}` -> 200 order.
async fn update_status(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(id): Path<i64>,
    ApiJson(body): ApiJson<UpdateOrderStatusRequest>,
) -> Result<Json<OrderResponse>, AppError> {
    Ok(Json(
        state.services.orders.update_status(actor, id, body).await?,
    ))
}
