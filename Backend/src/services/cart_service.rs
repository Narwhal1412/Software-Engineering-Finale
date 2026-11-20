//! Nghiệp vụ giỏ hàng.

use std::sync::Arc;

use crate::dtos::cart_dto::{AddCartItemRequest, CartItemResponse, SetQuantityRequest};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::repositories::{CartRepository, ProductRepository};

/// Số lượng tối đa của một dòng giỏ hàng (chặn nhập nhầm; API.md không nêu giới hạn).
const MAX_CART_QUANTITY: i32 = 99;

/// Nghiệp vụ giỏ hàng. Mọi thao tác đều trả về giỏ hàng mới như API.md quy định.
pub struct CartService {
    carts: Arc<dyn CartRepository>,
    products: Arc<dyn ProductRepository>,
}

impl CartService {
    /// Tạo service từ repository giỏ hàng và sản phẩm.
    pub fn new(carts: Arc<dyn CartRepository>, products: Arc<dyn ProductRepository>) -> Self {
        Self { carts, products }
    }

    /// Giỏ hàng hiện tại của người dùng.
    pub async fn get(&self, actor: AuthUser) -> Result<Vec<CartItemResponse>, AppError> {
        let lines = self.carts.list_with_products(actor.id).await?;
        Ok(lines.into_iter().map(Into::into).collect())
    }

    /// Thêm sản phẩm vào giỏ (cộng dồn nếu đã có), mặc định số lượng 1.
    pub async fn add_item(
        &self,
        actor: AuthUser,
        request: AddCartItemRequest,
    ) -> Result<Vec<CartItemResponse>, AppError> {
        let quantity = request.quantity.unwrap_or(1);
        validate_quantity(quantity)?;
        if self.products.find_by_id(request.product_id).await?.is_none() {
            return Err(AppError::NotFound("Không tìm thấy sản phẩm".to_owned()));
        }

        let item = self
            .carts
            .add_item(actor.id, request.product_id, quantity)
            .await?;
        // Cộng dồn có thể vượt trần; kẹp lại thay vì báo lỗi vì thao tác đã hợp lệ với người dùng.
        if item.quantity > MAX_CART_QUANTITY {
            self.carts
                .set_quantity(actor.id, request.product_id, MAX_CART_QUANTITY)
                .await?;
        }
        self.get(actor).await
    }

    /// Đặt số lượng tuyệt đối của một dòng.
    pub async fn set_quantity(
        &self,
        actor: AuthUser,
        product_id: i64,
        request: SetQuantityRequest,
    ) -> Result<Vec<CartItemResponse>, AppError> {
        validate_quantity(request.quantity)?;
        self.carts
            .set_quantity(actor.id, product_id, request.quantity)
            .await?;
        self.get(actor).await
    }

    /// Xóa một dòng khỏi giỏ.
    pub async fn remove_item(
        &self,
        actor: AuthUser,
        product_id: i64,
    ) -> Result<Vec<CartItemResponse>, AppError> {
        self.carts.remove_item(actor.id, product_id).await?;
        self.get(actor).await
    }
}

/// Số lượng phải trong khoảng 1..=`MAX_CART_QUANTITY`.
fn validate_quantity(quantity: i32) -> Result<(), AppError> {
    if (1..=MAX_CART_QUANTITY).contains(&quantity) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "Số lượng phải từ 1 đến {MAX_CART_QUANTITY}"
        )))
    }
}
