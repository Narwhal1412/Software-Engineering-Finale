//! Logging bằng `tracing`.

use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

/// Mức log mặc định khi không đặt `RUST_LOG`.
const DEFAULT_LOG_FILTER: &str = "info,tower_http=debug";

/// Khởi tạo bộ ghi log toàn cục, đọc mức log từ `RUST_LOG`.
pub fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));
    // `try_init` để gọi lại (ví dụ trong test) không làm chương trình panic.
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

/// Layer ghi log mỗi request HTTP (method, đường dẫn, status, thời gian xử lý).
pub fn http_trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}
