//! Phát hành và kiểm tra JWT (HS256) cho header `Authorization: Bearer <token>`.

use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::AppError;
use crate::models::enums::Role;

/// Thông báo cho mọi token không dùng được; frontend sẽ tự đăng xuất khi nhận 401.
pub const SESSION_EXPIRED_MESSAGE: &str = "Phiên đăng nhập đã hết hạn, hãy đăng nhập lại";

/// Nội dung (claims) nằm trong token.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Id người dùng, dạng chuỗi theo chuẩn JWT.
    pub sub: String,
    /// Vai trò lúc phát hành. Chỉ để tham khảo: quyền thật luôn lấy từ database.
    pub role: Role,
    /// Thời điểm phát hành (giây Unix).
    pub iat: u64,
    /// Thời điểm hết hạn (giây Unix).
    pub exp: u64,
}

impl Claims {
    /// Id người dùng trong token; token có `sub` không phải số bị coi là không hợp lệ.
    pub fn user_id(&self) -> Result<i64, AppError> {
        self.sub
            .parse::<i64>()
            .map_err(|_| AppError::Unauthorized(SESSION_EXPIRED_MESSAGE.to_owned()))
    }
}

/// Phát hành token cho người dùng, hết hạn sau `config.jwt_ttl_hours` giờ.
pub fn issue_token(config: &Config, user_id: i64, role: Role) -> Result<String, AppError> {
    let issued_at = u64::try_from(Utc::now().timestamp()).unwrap_or(0);
    let claims = Claims {
        sub: user_id.to_string(),
        role,
        iat: issued_at,
        exp: issued_at.saturating_add(config.jwt_ttl_hours.saturating_mul(3600)),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|err| AppError::Internal(format!("không ký được token: {err}")))
}

/// Kiểm tra chữ ký và hạn của token; mọi lỗi đều trả về 401 với cùng một thông báo.
pub fn verify_token(config: &Config, token: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized(SESSION_EXPIRED_MESSAGE.to_owned()))
}
