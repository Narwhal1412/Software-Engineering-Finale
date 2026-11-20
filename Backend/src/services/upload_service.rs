//! Lưu ảnh sản phẩm do admin tải lên.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::Utc;

use crate::dtos::upload_dto::UploadResponse;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::services::authz::require_admin;

/// Kích thước ảnh tối đa: 2 MB (theo API.md).
pub const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;

/// Bộ đếm để hai ảnh lưu cùng nanosecond vẫn có tên khác nhau.
static FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Nghiệp vụ upload ảnh. Ảnh nằm trên đĩa chứ không phải trong database nên không dùng repository.
pub struct UploadService {
    upload_dir: PathBuf,
}

impl UploadService {
    /// Tạo service ghi ảnh vào thư mục `upload_dir`.
    pub fn new(upload_dir: &str) -> Self {
        Self {
            upload_dir: PathBuf::from(upload_dir),
        }
    }

    /// Kiểm tra rồi lưu ảnh (admin), trả về URL công khai.
    ///
    /// Định dạng được nhận diện bằng chữ ký byte đầu tệp chứ không tin `Content-Type`
    /// do client gửi, vì header đó giả mạo được.
    pub async fn save_image(&self, actor: AuthUser, bytes: &[u8]) -> Result<UploadResponse, AppError> {
        require_admin(actor)?;
        if bytes.is_empty() {
            return Err(AppError::BadRequest("Chưa chọn ảnh".to_owned()));
        }
        if bytes.len() > MAX_IMAGE_BYTES {
            return Err(AppError::PayloadTooLarge("Ảnh tối đa 2 MB".to_owned()));
        }
        let extension = detect_extension(bytes).ok_or_else(|| {
            AppError::UnsupportedMediaType("Chỉ nhận ảnh JPG, PNG hoặc WebP".to_owned())
        })?;

        let file_name = unique_file_name(extension);
        tokio::fs::create_dir_all(&self.upload_dir)
            .await
            .map_err(|err| AppError::Internal(format!("không tạo được thư mục upload: {err}")))?;
        tokio::fs::write(self.upload_dir.join(&file_name), bytes)
            .await
            .map_err(|err| AppError::Internal(format!("không ghi được ảnh: {err}")))?;

        Ok(UploadResponse {
            url: format!("/uploads/{file_name}"),
        })
    }
}

/// Đuôi tệp tương ứng với chữ ký JPEG, PNG hoặc WebP; `None` nếu không phải ảnh được hỗ trợ.
fn detect_extension(bytes: &[u8]) -> Option<&'static str> {
    const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("jpg")
    } else if bytes.starts_with(&PNG_SIGNATURE) {
        Some("png")
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP".as_slice()) {
        Some("webp")
    } else {
        None
    }
}

/// Tên tệp do server tự sinh; không bao giờ dùng tên client gửi để tránh path traversal.
fn unique_file_name(extension: &str) -> String {
    let nanos = Utc::now().timestamp_nanos_opt().unwrap_or_default();
    let sequence = FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{nanos:x}-{sequence:x}.{extension}")
}
