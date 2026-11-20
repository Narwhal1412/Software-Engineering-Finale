//! Quy tắc phân quyền dùng chung cho các service.

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::enums::Role;

/// Chỉ cho phép admin. Kiểm tra ở service (không chỉ ở route) để quy tắc nằm một chỗ
/// và test được mà không cần dựng HTTP. Trả 403 chứ không phải 401 vì frontend
/// tự đăng xuất khi nhận 401.
pub fn require_admin(actor: AuthUser) -> Result<(), AppError> {
    if actor.role == Role::Admin {
        Ok(())
    } else {
        Err(AppError::Forbidden("Cần quyền quản trị".to_owned()))
    }
}
