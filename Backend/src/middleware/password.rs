//! Băm và kiểm tra mật khẩu bằng Argon2id.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

use crate::error::AppError;

/// Băm mật khẩu với muối ngẫu nhiên; trả về chuỗi PHC (`$argon2id$...`) để lưu vào `users.password_hash`.
pub fn hash_password(plain: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plain.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| AppError::Internal(format!("không băm được mật khẩu: {err}")))
}

/// Kiểm tra mật khẩu với chuỗi PHC đã lưu. Hash hỏng cũng coi là không khớp
/// (thay vì lỗi 500) để không lộ thông tin về dữ liệu lưu trữ.
pub fn verify_password(plain: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}
