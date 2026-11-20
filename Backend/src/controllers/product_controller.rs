//! Endpoint sản phẩm: `/products`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, patch, put};
use axum::{Json, Router};

use super::api_json::ApiJson;
use crate::app_state::AppState;
use crate::dtos::product_dto::{PinRequest, ProductQueryParams, ProductRequest, ProductResponse};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;

/// Router của feature sản phẩm.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/products", get(list).post(create))
        .route("/products/:id", put(update).delete(remove))
        .route("/products/:id/pin", patch(pin))
}

/// `GET /products?q=&categoryId=&sort=` (công khai) -> 200 `[product]`.
async fn list(
    State(state): State<AppState>,
    Query(params): Query<ProductQueryParams>,
) -> Result<Json<Vec<ProductResponse>>, AppError> {
    Ok(Json(state.services.products.search(params).await?))
}

/// `POST /products` -> 201 product.
async fn create(
    State(state): State<AppState>,
    actor: AuthUser,
    ApiJson(body): ApiJson<ProductRequest>,
) -> Result<(StatusCode, Json<ProductResponse>), AppError> {
    let product = state.services.products.create(actor, body).await?;
    Ok((StatusCode::CREATED, Json(product)))
}

/// `PUT /products/:id` -> 200 product.
async fn update(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(id): Path<i64>,
    ApiJson(body): ApiJson<ProductRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    Ok(Json(state.services.products.update(actor, id, body).await?))
}

/// `DELETE /products/:id` -> 204.
async fn remove(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    state.services.products.delete(actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `PATCH /products/:id/pin` `{pinned}` -> 200 product.
async fn pin(
    State(state): State<AppState>,
    actor: AuthUser,
    Path(id): Path<i64>,
    ApiJson(body): ApiJson<PinRequest>,
) -> Result<Json<ProductResponse>, AppError> {
    let product = state
        .services
        .products
        .set_pinned(actor, id, body.pinned)
        .await?;
    Ok(Json(product))
}
