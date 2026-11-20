//! Đăng ký và đăng nhập.

use std::sync::Arc;

use crate::config::Config;
use crate::dtos::auth_dto::{AuthResponse, LoginRequest, RegisterRequest};
use crate::error::AppError;
use crate::middleware::password::{hash_password, verify_password};
use crate::middleware::token::issue_token;
use crate::models::enums::Role;
use crate::models::limits::DISPLAY_NAME_MAX_LEN;
use crate::models::user::{NewUser, User};
use crate::repositories::UserRepository;
use crate::services::validation::{required_text, validate_email};

/// Độ dài mật khẩu tối thiểu (khớp `minLength={6}` của form frontend).
const MIN_PASSWORD_LEN: usize = 6;
/// Độ dài mật khẩu tối đa, để không tốn CPU băm chuỗi khổng lồ.
const MAX_PASSWORD_LEN: usize = 128;

/// Nghiệp vụ xác thực.
pub struct AuthService {
    users: Arc<dyn UserRepository>,
    config: Arc<Config>,
}

impl AuthService {
    /// Tạo service từ repository người dùng và cấu hình (để ký token).
    pub fn new(users: Arc<dyn UserRepository>, config: Arc<Config>) -> Self {
        Self { users, config }
    }

    /// Đăng ký tài khoản khách hàng mới và đăng nhập luôn.
    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AppError> {
        let email = request.email.trim();
        if email.is_empty() || request.display_name.trim().is_empty() || request.password.is_empty()
        {
            return Err(AppError::BadRequest(
                "Nhập đủ email, tên hiển thị và mật khẩu".to_owned(),
            ));
        }
        validate_email(email)?;
        let display_name = required_text(&request.display_name, "tên hiển thị", DISPLAY_NAME_MAX_LEN)?;
        validate_password(&request.password)?;
        if request.password != request.confirm_password {
            return Err(AppError::BadRequest(
                "Mật khẩu nhập lại không khớp".to_owned(),
            ));
        }

        let password_hash = hash_in_background(request.password).await?;
        let user = self
            .users
            .create(NewUser {
                email: email.to_owned(),
                display_name,
                password_hash,
                phone: String::new(),
                address: String::new(),
                // Đăng ký công khai không bao giờ được tạo admin.
                role: Role::User,
            })
            .await?;
        self.start_session(user)
    }

    /// Đăng nhập bằng email và mật khẩu.
    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AppError> {
        let user = self
            .users
            .find_by_email(request.email.trim())
            .await?
            .ok_or_else(invalid_credentials)?;
        let matches = verify_in_background(request.password, user.password_hash.clone()).await?;
        if !matches {
            return Err(invalid_credentials());
        }
        self.start_session(user)
    }

    /// Phát hành token và đóng gói thành phản hồi.
    fn start_session(&self, user: User) -> Result<AuthResponse, AppError> {
        let token = issue_token(&self.config, user.id, user.role)?;
        Ok(AuthResponse {
            user: user.into(),
            token,
        })
    }
}

/// Dùng cùng một thông báo cho "sai email" và "sai mật khẩu" để không lộ email nào đã đăng ký.
/// Là 400 chứ không phải 401: frontend coi 401 khi đang có token là hết phiên.
fn invalid_credentials() -> AppError {
    AppError::BadRequest("Sai email hoặc mật khẩu".to_owned())
}

/// Mật khẩu phải đủ dài nhưng không quá dài.
fn validate_password(password: &str) -> Result<(), AppError> {
    let len = password.chars().count();
    if len < MIN_PASSWORD_LEN {
        return Err(AppError::BadRequest(format!(
            "Mật khẩu tối thiểu {MIN_PASSWORD_LEN} ký tự"
        )));
    }
    if len > MAX_PASSWORD_LEN {
        return Err(AppError::BadRequest(format!(
            "Mật khẩu tối đa {MAX_PASSWORD_LEN} ký tự"
        )));
    }
    Ok(())
}

/// Argon2 tốn CPU nên chạy ngoài luồng async để không làm nghẽn các request khác.
async fn hash_in_background(plain: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || hash_password(&plain))
        .await
        .map_err(|err| AppError::Internal(format!("tác vụ băm mật khẩu bị gián đoạn: {err}")))?
}

/// Kiểm tra mật khẩu ngoài luồng async, cùng lý do với [`hash_in_background`].
async fn verify_in_background(plain: String, stored_hash: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || verify_password(&plain, &stored_hash))
        .await
        .map_err(|err| AppError::Internal(format!("tác vụ kiểm tra mật khẩu bị gián đoạn: {err}")))
}
