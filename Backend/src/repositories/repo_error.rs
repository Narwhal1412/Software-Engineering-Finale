//! Lỗi chung của tầng truy cập dữ liệu.

use thiserror::Error;

/// Lỗi do repository trả về. Các biến thể mô tả vi phạm ràng buộc theo cách
/// độc lập với loại database, để mock và implementation thật hành xử giống nhau.
#[derive(Debug, Error)]
pub enum RepoError {
    /// Không có bản ghi với khóa được yêu cầu.
    #[error("{entity} not found")]
    NotFound {
        /// Tên entity: `user`, `product`, `category`, `order` hoặc `cart item`.
        entity: &'static str,
    },
    /// Vi phạm ràng buộc UNIQUE.
    #[error("duplicate value for {field}")]
    Duplicate {
        /// Tên trường bị trùng (ví dụ `email`, `name`).
        field: &'static str,
    },
    /// Khóa ngoại trỏ tới bản ghi không tồn tại.
    #[error("{field} refers to a row that does not exist")]
    InvalidReference {
        /// Tên trường khóa ngoại (ví dụ `category_id`).
        field: &'static str,
    },
    /// Bản ghi đang bị bản ghi khác tham chiếu (ON DELETE RESTRICT).
    #[error("{entity} is still referenced by {referenced_by}")]
    InUse {
        /// Entity định xóa.
        entity: &'static str,
        /// Entity đang tham chiếu tới nó.
        referenced_by: &'static str,
    },
    /// Số lượng yêu cầu vượt quá tồn kho của sản phẩm.
    #[error("{product} is out of stock (available: {available})")]
    OutOfStock {
        /// Tên sản phẩm (dùng cho thông báo).
        product: String,
        /// Số lượng còn trong kho tại thời điểm kiểm tra.
        available: i32,
    },
    /// Vi phạm ràng buộc CHECK / NOT NULL / độ dài cột.
    #[error("constraint violated: {0}")]
    Constraint(String),
    /// Lỗi hạ tầng (mất kết nối, lỗi SQL...). Chi tiết chỉ dùng để log.
    #[error("internal repository error: {0}")]
    Internal(String),
}
