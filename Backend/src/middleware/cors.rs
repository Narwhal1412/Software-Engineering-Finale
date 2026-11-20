//! Cấu hình CORS.

use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::config::Config;

/// Dựng CORS layer: danh sách origin rỗng nghĩa là cho phép mọi origin
/// (an toàn vì xác thực dùng header Bearer chứ không dùng cookie).
pub fn cors_layer(config: &Config) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    if config.cors_allowed_origins.is_empty() {
        return base.allow_origin(Any);
    }

    // Origin không phải header hợp lệ bị bỏ qua thay vì làm sập server khi khởi động.
    let origins: Vec<HeaderValue> = config
        .cors_allowed_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();
    base.allow_origin(AllowOrigin::list(origins))
}
