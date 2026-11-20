//! Extractor xác thực và phân quyền. Controller chỉ cần khai báo tham số
//! `AuthUser` (cần đăng nhập) hoặc `AdminUser` (cần quyền admin).
//!
//! Quy ước mã lỗi, vì frontend tự đăng xuất khi nhận 401 lúc đang có token:
//! - thiếu token, token sai/hết hạn, hoặc tài khoản đã bị xóa -> 401;
//! - đã đăng nhập nhưng không phải admin -> 403 (KHÔNG dùng 401).

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use super::token::{verify_token, SESSION_EXPIRED_MESSAGE};
use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::enums::Role;

/// Người dùng đã đăng nhập, lấy từ token và đối chiếu lại với database.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthUser {
    /// Id người dùng.
    pub id: i64,
    /// Vai trò hiện tại trong database (không phải vai trò ghi trong token).
    pub role: Role,
}

/// Người dùng đã đăng nhập VÀ có vai trò admin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdminUser(pub AuthUser);

/// Lấy chuỗi token sau tiền tố `Bearer ` của header `Authorization`.
fn bearer_token(parts: &Parts) -> Option<&str> {
    let value = parts.headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ")?.trim();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_id = {
            let token = bearer_token(parts)
                .ok_or_else(|| AppError::Unauthorized("Cần đăng nhập".to_owned()))?;
            verify_token(&state.config, token)?.user_id()?
        };

        // Tra lại database để token của tài khoản đã bị admin xóa lập tức mất hiệu lực,
        // và để vai trò luôn là vai trò mới nhất.
        let user = state
            .repos
            .users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized(SESSION_EXPIRED_MESSAGE.to_owned()))?;

        Ok(Self {
            id: user.id,
            role: user.role,
        })
    }
}

#[async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await?;
        if user.role == Role::Admin {
            Ok(Self(user))
        } else {
            Err(AppError::Forbidden("Cần quyền quản trị".to_owned()))
        }
    }
}
