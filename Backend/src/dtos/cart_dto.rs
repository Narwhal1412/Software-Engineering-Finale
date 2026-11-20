//! DTO của giỏ hàng: `[{productId, quantity, product}]`.

use serde::{Deserialize, Serialize};

use super::product_dto::ProductResponse;
use crate::models::cart::CartLine;

/// Một dòng giỏ hàng kèm sản phẩm.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CartItemResponse {
    /// Id sản phẩm.
    pub product_id: i64,
    /// Số lượng.
    pub quantity: i32,
    /// Thông tin sản phẩm.
    pub product: ProductResponse,
}

impl From<CartLine> for CartItemResponse {
    fn from(line: CartLine) -> Self {
        Self {
            product_id: line.product_id,
            quantity: line.quantity,
            product: line.product.into(),
        }
    }
}

/// Body của `POST /api/cart/items`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCartItemRequest {
    /// Id sản phẩm cần thêm.
    #[serde(default)]
    pub product_id: i64,
    /// Số lượng; thiếu thì mặc định 1 (giống frontend mock).
    #[serde(default)]
    pub quantity: Option<i32>,
}

/// Body của `PATCH /api/cart/items/:productId`.
#[derive(Debug, Deserialize)]
pub struct SetQuantityRequest {
    /// Số lượng mới (>= 1); thiếu được coi là 0 để bị từ chối.
    #[serde(default)]
    pub quantity: i32,
}
