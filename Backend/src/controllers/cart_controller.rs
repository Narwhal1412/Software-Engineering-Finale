//! Endpoint giỏ hàng: `/cart`. Mọi lệnh đều trả về giỏ hàng mới.

use axum::extract::{Path, State};
use axum::routing::{get, patch, post};
use axum::{Json, Router};

use super::api_json::ApiJson;
use crate::app_state::AppState;
use crate::dtos::cart_dto::{AddCartItemRequest, CartItemResponse, SetQuantityRequest};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Router của feature giỏ hàng.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cart", get(get_cart))
        .route("/cart/items", post(add_item))
        .route(
            "/cart/items/:product_id",
            patch(set_quantity).delete(remove_item),
        )
}

/// `GET /cart` -> 200 `[{productId, quantity, product}]`.
async fn get_cart(
    State(state): State<AppState>,
    actor: AuthUser,
) -> Result<Json<Vec<CartItemResponse>>, AppError> {
    Ok(Json(state.services.carts.get(actor).await?))
}

/// `POST /cart/items` -> 200 giỏ hàng mới.
async fn add_item(
    State(state): State<AppState>,
    actor: AuthUser,
    ApiJson(body): ApiJson<AddCartItemRequest>,
) -> Result<Json<Vec<CartItemResponse>>, AppError> {
    Ok(Json(state.services.carts.add_item(actor, body).await?))
}

/// `PATCH /cart/items/:productId` -> 200 giỏ hàng mới.
async fn set_quantity(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(product_id): Path<i64>,
    ApiJson(body): ApiJson<SetQuantityRequest>,
) -> Result<Json<Vec<CartItemResponse>>, AppError> {
    Ok(Json(
        state
            .services
            .carts
            .set_quantity(actor, product_id, body)
            .await?,
    ))
}

/// `DELETE /cart/items/:productId` -> 200 giỏ hàng mới.
async fn remove_item(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(product_id): Path<i64>,
) -> Result<Json<Vec<CartItemResponse>>, AppError> {
    Ok(Json(
        state.services.carts.remove_item(actor, product_id).await?,
    ))
}
