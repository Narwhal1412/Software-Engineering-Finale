//! DTO của upload ảnh: `{url}`.

use serde::Serialize;

/// Kết quả upload.
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    /// Đường dẫn công khai của ảnh, dạng `/uploads/<tên>`.
    pub url: String,
}
