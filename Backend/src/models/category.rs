//! Entity `Category` (bảng `categories`).

/// Danh mục sản phẩm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    /// Khóa chính, tự tăng.
    pub id: i64,
    /// Tên danh mục, duy nhất không phân biệt hoa thường.
    pub name: String,
}
