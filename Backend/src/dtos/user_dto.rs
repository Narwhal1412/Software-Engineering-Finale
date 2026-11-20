//! DTO của người dùng: `user = {id, email, displayName, phone, address, role}`.

use serde::{Deserialize, Serialize};

use crate::models::enums::Role;
use crate::models::user::User;

/// Người dùng trả về cho client (không có mật khẩu).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    /// Id người dùng.
    pub id: i64,
    /// Email đăng nhập.
    pub email: String,
    /// Tên hiển thị.
    pub display_name: String,
    /// Số điện thoại (rỗng nếu chưa có).
    pub phone: String,
    /// Địa chỉ (rỗng nếu chưa có).
    pub address: String,
    /// Vai trò: `user` hoặc `admin`.
    pub role: Role,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            phone: user.phone,
            address: user.address,
            role: user.role,
        }
    }
}

/// Body của `PUT /api/me`. Trường thiếu hoặc `null` được coi là rỗng để service báo lỗi tiếng Việt thay vì lỗi parse.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    /// Tên hiển thị mới (bắt buộc).
    #[serde(default)]
    pub display_name: Option<String>,
    /// Số điện thoại mới.
    #[serde(default)]
    pub phone: Option<String>,
    /// Địa chỉ mới.
    #[serde(default)]
    pub address: Option<String>,
}
