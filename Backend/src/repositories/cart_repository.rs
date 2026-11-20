//! Trait truy cập bảng `cart_items`.

use async_trait::async_trait;

use super::RepoError;
use crate::models::cart::{CartItem, CartLine};

/// Truy cập giỏ hàng. Mỗi người dùng có tối đa một dòng cho mỗi sản phẩm.
#[async_trait]
pub trait CartRepository: Send + Sync {
    /// Giỏ hàng của người dùng kèm thông tin sản phẩm, sắp theo `product_id` tăng dần.
    async fn list_with_products(&self, user_id: i64) -> Result<Vec<CartLine>, RepoError>;

    /// Thêm sản phẩm; nếu đã có thì CỘNG DỒN số lượng. `Constraint` nếu `quantity < 1`;
    /// `InvalidReference` (`user_id` hoặc `product_id`) nếu không tồn tại.
    async fn add_item(
        &self,
        user_id: i64,
        product_id: i64,
        quantity: i32,
    ) -> Result<CartItem, RepoError>;

    /// Đặt số lượng tuyệt đối. `Constraint` nếu `quantity < 1`;
    /// `NotFound { entity: "cart item" }` nếu sản phẩm chưa có trong giỏ.
    async fn set_quantity(
        &self,
        user_id: i64,
        product_id: i64,
        quantity: i32,
    ) -> Result<CartItem, RepoError>;

    /// Xóa một dòng khỏi giỏ. Idempotent: không có dòng đó vẫn trả `Ok(())`.
    async fn remove_item(&self, user_id: i64, product_id: i64) -> Result<(), RepoError>;

    /// Xóa toàn bộ giỏ của người dùng.
    async fn clear(&self, user_id: i64) -> Result<(), RepoError>;
}
