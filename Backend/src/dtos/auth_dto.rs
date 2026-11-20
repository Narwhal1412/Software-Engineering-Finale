//! DTO của đăng ký / đăng nhập.

use serde::{Deserialize, Serialize};

use super::user_dto::UserResponse;

/// Body của `POST /api/auth/register`. Trường thiếu được coi là rỗng để service báo lỗi tiếng Việt.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    /// Email đăng nhập.
    #[serde(default)]
    pub email: String,
    /// Tên hiển thị.
    #[serde(default)]
    pub display_name: String,
    /// Mật khẩu.
    #[serde(default)]
    pub password: String,
    /// Mật khẩu nhập lại, phải trùng `password`.
    #[serde(default)]
    pub confirm_password: String,
}

/// Body của `POST /api/auth/login`.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// Email đăng nhập.
    #[serde(default)]
    pub email: String,
    /// Mật khẩu.
    #[serde(default)]
    pub password: String,
}

/// Kết quả đăng ký/đăng nhập: `{user, token}`.
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    /// Người dùng vừa xác thực.
    pub user: UserResponse,
    /// JWT gửi lại qua header `Authorization: Bearer <token>`.
    pub token: String,
}
