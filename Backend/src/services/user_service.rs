//! Hồ sơ cá nhân và quản lý thành viên.

use std::sync::Arc;

use crate::dtos::user_dto::{UpdateProfileRequest, UserResponse};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::enums::Role;
use crate::models::limits::{ADDRESS_MAX_LEN, DISPLAY_NAME_MAX_LEN};
use crate::models::user::ProfileUpdate;
use crate::repositories::UserRepository;
use crate::services::authz::require_admin;
use crate::services::validation::{optional_text, required_text, validate_phone};

/// Nghiệp vụ người dùng.
pub struct UserService {
    users: Arc<dyn UserRepository>,
}

impl UserService {
    /// Tạo service từ repository người dùng.
    pub fn new(users: Arc<dyn UserRepository>) -> Self {
        Self { users }
    }

    /// Hồ sơ của chính người đang đăng nhập.
    pub async fn get_me(&self, actor: AuthUser) -> Result<UserResponse, AppError> {
        let user = self
            .users
            .find_by_id(actor.id)
            .await?
            .ok_or_else(user_not_found)?;
        Ok(user.into())
    }

    /// Cập nhật tên hiển thị, số điện thoại, địa chỉ của chính mình.
    pub async fn update_me(
        &self,
        actor: AuthUser,
        request: UpdateProfileRequest,
    ) -> Result<UserResponse, AppError> {
        let display_name = required_text(
            request.display_name.as_deref().unwrap_or(""),
            "tên hiển thị",
            DISPLAY_NAME_MAX_LEN,
        )?;
        let phone = request.phone.as_deref().unwrap_or("").trim().to_owned();
        if !phone.is_empty() {
            validate_phone(&phone)?;
        }
        let address = optional_text(
            request.address.as_deref().unwrap_or(""),
            "địa chỉ",
            ADDRESS_MAX_LEN,
        )?;

        let user = self
            .users
            .update_profile(
                actor.id,
                ProfileUpdate {
                    display_name,
                    phone,
                    address,
                },
            )
            .await?;
        Ok(user.into())
    }

    /// Danh sách mọi thành viên (admin).
    pub async fn list_users(&self, actor: AuthUser) -> Result<Vec<UserResponse>, AppError> {
        require_admin(actor)?;
        let users = self.users.list().await?;
        Ok(users.into_iter().map(Into::into).collect())
    }

    /// Xóa thành viên (admin). Không xóa được tài khoản admin.
    pub async fn delete_user(&self, actor: AuthUser, id: i64) -> Result<(), AppError> {
        require_admin(actor)?;
        let target = self.users.find_by_id(id).await?.ok_or_else(user_not_found)?;
        if target.role == Role::Admin {
            return Err(AppError::Forbidden(
                "Không thể xóa tài khoản admin".to_owned(),
            ));
        }
        self.users.delete(id).await?;
        Ok(())
    }
}

/// Thông báo khi không có thành viên (khớp frontend mock).
fn user_not_found() -> AppError {
    AppError::NotFound("Không tìm thấy thành viên".to_owned())
}
