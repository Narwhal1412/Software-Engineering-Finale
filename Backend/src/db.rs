//! Kết nối MariaDB bằng sqlx (đặc tả: docs/DATABASE_SPEC.md mục 7.3).
//!
//! Dựng pool bằng `MySqlConnectOptions` (không ghép chuỗi URL, vì mật khẩu có thể chứa ký tự đặc biệt).
//! `connect_with_retry` thử lại vài lần vì khi `docker compose up`, MariaDB khởi động chậm hơn backend.

use std::time::Duration;

use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};
use sqlx::MySqlPool;

use crate::config::Config;
use crate::repositories::RepoError;

/// Số lần thử kết nối tối đa khi khởi động.
const MAX_ATTEMPTS: u32 = 10;
/// Thời gian chờ giữa hai lần thử.
const RETRY_DELAY: Duration = Duration::from_secs(3);
/// Số kết nối tối đa trong pool.
const MAX_CONNECTIONS: u32 = 10;
/// Thời gian chờ tối đa để lấy một kết nối từ pool.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Mở pool kết nối một lần, không thử lại.
pub async fn connect(config: &Config) -> Result<MySqlPool, RepoError> {
    let options = MySqlConnectOptions::new()
        .host(&config.db_host)
        .port(config.db_port)
        .username(&config.db_user)
        .password(&config.db_password)
        .database(&config.db_name)
        .charset("utf8mb4")
        // Backend và DB nói chuyện trong mạng nội bộ của Docker nên không dùng TLS.
        .ssl_mode(MySqlSslMode::Disabled);

    MySqlPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_with(options)
        .await
        .map_err(|err| RepoError::Internal(format!("không kết nối được database: {err}")))
}

/// Mở pool, thử lại tối đa `MAX_ATTEMPTS` lần. Được gọi từ `build_repositories()`.
pub async fn connect_with_retry(config: &Config) -> Result<MySqlPool, RepoError> {
    let mut attempt: u32 = 1;
    loop {
        match connect(config).await {
            Ok(pool) => {
                tracing::info!(attempt, "đã kết nối database");
                return Ok(pool);
            }
            Err(err) if attempt < MAX_ATTEMPTS => {
                tracing::warn!(attempt, max = MAX_ATTEMPTS, %err, "database chưa sẵn sàng, sẽ thử lại");
                tokio::time::sleep(RETRY_DELAY).await;
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}
