//! Đọc cấu hình ứng dụng từ biến môi trường.

use std::env;
use std::fmt::Display;
use std::str::FromStr;

use thiserror::Error;

/// Cổng mặc định; khớp cấu hình proxy của frontend (vite và nginx).
const DEFAULT_PORT: u16 = 3000;
/// Số giờ token còn hiệu lực nếu không cấu hình.
const DEFAULT_JWT_TTL_HOURS: u64 = 24;
/// Thư mục lưu ảnh upload mặc định (tương đối so với thư mục chạy).
const DEFAULT_UPLOAD_DIR: &str = "uploads";
/// Cổng mặc định của MariaDB/MySQL.
const DEFAULT_DB_PORT: u16 = 3306;
/// Độ dài tối thiểu của khóa JWT để chống khóa quá yếu.
const MIN_JWT_SECRET_LEN: usize = 16;

/// Khóa JWT dùng khi chưa cấu hình `JWT_SECRET`; chỉ phù hợp cho môi trường dev.
pub const DEV_JWT_SECRET: &str = "dev-only-insecure-secret-change-me";

/// Lỗi khi đọc cấu hình.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Một biến môi trường có giá trị không hợp lệ.
    #[error("biến môi trường {name} không hợp lệ: {reason}")]
    Invalid {
        /// Tên biến môi trường.
        name: &'static str,
        /// Lý do giá trị bị từ chối.
        reason: String,
    },
}

/// Toàn bộ cấu hình của backend. Không derive `Debug` để tránh vô tình log bí mật.
#[derive(Clone)]
pub struct Config {
    /// Cổng HTTP lắng nghe (`APP_PORT`).
    pub port: u16,
    /// Khóa ký JWT (`JWT_SECRET`).
    pub jwt_secret: String,
    /// Thời hạn token tính bằng giờ (`JWT_TTL_HOURS`).
    pub jwt_ttl_hours: u64,
    /// Thư mục lưu ảnh upload (`UPLOAD_DIR`).
    pub upload_dir: String,
    /// Origin được phép gọi API; rỗng nghĩa là cho phép tất cả (`CORS_ALLOWED_ORIGINS`).
    pub cors_allowed_origins: Vec<String>,
    /// Máy chủ database (`DB_HOST`).
    pub db_host: String,
    /// Cổng database (`DB_PORT`).
    pub db_port: u16,
    /// Tài khoản database (`DB_USER`).
    pub db_user: String,
    /// Mật khẩu database (`DB_PASSWORD`).
    pub db_password: String,
    /// Tên database (`DB_NAME`).
    pub db_name: String,
}

impl Config {
    /// Dựng cấu hình từ biến môi trường, dùng giá trị mặc định khi biến vắng mặt.
    pub fn from_env() -> Result<Self, ConfigError> {
        let jwt_secret = string_env("JWT_SECRET", DEV_JWT_SECRET);
        if jwt_secret.len() < MIN_JWT_SECRET_LEN {
            return Err(ConfigError::Invalid {
                name: "JWT_SECRET",
                reason: format!("cần ít nhất {MIN_JWT_SECRET_LEN} ký tự"),
            });
        }

        Ok(Self {
            port: parse_env("APP_PORT", DEFAULT_PORT)?,
            jwt_secret,
            jwt_ttl_hours: parse_env("JWT_TTL_HOURS", DEFAULT_JWT_TTL_HOURS)?,
            upload_dir: string_env("UPLOAD_DIR", DEFAULT_UPLOAD_DIR),
            cors_allowed_origins: origins_env("CORS_ALLOWED_ORIGINS"),
            db_host: string_env("DB_HOST", "localhost"),
            db_port: parse_env("DB_PORT", DEFAULT_DB_PORT)?,
            db_user: string_env("DB_USER", "app"),
            db_password: string_env("DB_PASSWORD", ""),
            db_name: string_env("DB_NAME", "appdb"),
        })
    }

    /// Cho biết đang dùng khóa JWT mặc định (không an toàn cho production).
    pub fn uses_default_jwt_secret(&self) -> bool {
        self.jwt_secret == DEV_JWT_SECRET
    }
}

/// Đọc biến dạng chuỗi; biến rỗng được coi như chưa đặt.
fn string_env(name: &str, default: &str) -> String {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_owned())
}

/// Đọc và parse một biến; biến vắng mặt hoặc rỗng dùng giá trị mặc định.
fn parse_env<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: Display,
{
    match env::var(name) {
        Ok(raw) if !raw.trim().is_empty() => {
            raw.trim()
                .parse::<T>()
                .map_err(|err| ConfigError::Invalid {
                    name,
                    reason: err.to_string(),
                })
        }
        _ => Ok(default),
    }
}

/// Đọc danh sách origin phân tách bằng dấu phẩy; `*` được quy về "cho tất cả".
fn origins_env(name: &str) -> Vec<String> {
    let origins: Vec<String> = env::var(name)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(str::to_owned)
        .collect();

    if origins.iter().any(|origin| origin == "*") {
        Vec::new()
    } else {
        origins
    }
}
