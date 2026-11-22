//! Entity `Policy` (bảng `policies`): chính sách và ưu đãi hiển thị trong chi tiết sản phẩm.

/// Một chính sách hoặc ưu đãi (bảo hành, đổi trả, giao hàng...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// Khóa chính, tự tăng.
    pub id: i64,
    /// Tên biểu tượng (một trong `POLICY_ICONS` của service).
    pub icon: String,
    /// Tiêu đề ngắn, ví dụ "Bảo hành chính hãng 36 tháng".
    pub title: String,
    /// Nội dung mô tả.
    pub content: String,
    /// Danh mục áp dụng; `None` nghĩa là áp dụng cho mọi sản phẩm.
    pub category_id: Option<i64>,
    /// Thứ tự hiển thị, nhỏ hơn hiển thị trước.
    pub sort_order: i32,
}

/// Dữ liệu tạo mới hoặc cập nhật chính sách (tương ứng body của POST/PUT).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyInput {
    /// Tên biểu tượng.
    pub icon: String,
    /// Tiêu đề.
    pub title: String,
    /// Nội dung mô tả.
    pub content: String,
    /// Danh mục áp dụng (phải tồn tại nếu có giá trị); `None` = mọi sản phẩm.
    pub category_id: Option<i64>,
    /// Thứ tự hiển thị.
    pub sort_order: i32,
}
