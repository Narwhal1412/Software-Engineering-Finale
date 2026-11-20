//! Giỏ hàng: entity `CartItem` (bảng `cart_items`) và dạng đọc kèm sản phẩm.

use super::product::Product;

/// Một dòng trong giỏ hàng của một người dùng. Khóa chính là `(user_id, product_id)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartItem {
    /// Chủ giỏ hàng.
    pub user_id: i64,
    /// Sản phẩm trong giỏ.
    pub product_id: i64,
    /// Số lượng, tối thiểu 1.
    pub quantity: i32,
}

/// Dòng giỏ hàng đã ghép thông tin sản phẩm (kết quả của `GET /api/cart`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartLine {
    /// Mã sản phẩm.
    pub product_id: i64,
    /// Số lượng.
    pub quantity: i32,
    /// Thông tin sản phẩm hiện tại.
    pub product: Product,
}
