//! Entity `User` (bảng `users`).

use chrono::{DateTime, Utc};

use super::enums::Role;

/// Tài khoản người dùng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Khóa chính, tự tăng.
    pub id: i64,
    /// Email đăng nhập, duy nhất không phân biệt hoa thường.
    pub email: String,
    /// Tên hiển thị.
    pub display_name: String,
    /// Mật khẩu đã băm bằng Argon2 (chuỗi PHC). Không bao giờ đưa vào response.
    pub password_hash: String,
    /// Số điện thoại; chuỗi rỗng nếu chưa có.
    pub phone: String,
    /// Địa chỉ; chuỗi rỗng nếu chưa có.
    pub address: String,
    /// Vai trò.
    pub role: Role,
    /// Thời điểm tạo (UTC).
    pub created_at: DateTime<Utc>,
}

/// Dữ liệu để tạo tài khoản mới; `id` và `created_at` do repository sinh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewUser {
    /// Email đăng nhập.
    pub email: String,
    /// Tên hiển thị.
    pub display_name: String,
    /// Mật khẩu đã băm.
    pub password_hash: String,
    /// Số điện thoại; rỗng nếu chưa có.
    pub phone: String,
    /// Địa chỉ; rỗng nếu chưa có.
    pub address: String,
    /// Vai trò.
    pub role: Role,
}

/// Các trường người dùng được tự sửa qua `PUT /api/me`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileUpdate {
    /// Tên hiển thị mới.
    pub display_name: String,
    /// Số điện thoại mới; rỗng để xóa.
    pub phone: String,
    /// Địa chỉ mới; rỗng để xóa.
    pub address: String,
}
