//! Endpoint upload ảnh: `/uploads`.

use axum::body::Bytes;
use axum::extract::multipart::MultipartError;
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

use crate::app_state::AppState;
use crate::dtos::upload_dto::UploadResponse;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::services::upload_service::MAX_IMAGE_BYTES;

/// Tên trường multipart chứa ảnh (theo API.md).
const FILE_FIELD_NAME: &str = "file";
/// Dư cho phần đầu multipart để tệp đúng 2 MB không bị axum chặn trước khi service báo lỗi rõ ràng.
const MULTIPART_OVERHEAD_BYTES: usize = 64 * 1024;

/// Router của feature upload. Giới hạn body riêng vì mặc định của axum (2 MB) quá sát.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/uploads", post(upload_image))
        .layer(DefaultBodyLimit::max(MAX_IMAGE_BYTES + MULTIPART_OVERHEAD_BYTES))
}

/// `POST /uploads` (multipart, trường `file`) -> 201 `{url}`.
async fn upload_image(
    State(state): State<AppState>,
    actor: AuthUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), AppError> {
    let bytes = read_file_field(&mut multipart).await?;
    let response = state.services.uploads.save_image(actor, &bytes).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// Đọc nội dung trường `file`; các trường khác bị bỏ qua.
async fn read_file_field(multipart: &mut Multipart) -> Result<Bytes, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(upload_error)? {
        if field.name() == Some(FILE_FIELD_NAME) {
            return field.bytes().await.map_err(upload_error);
        }
    }
    Err(AppError::BadRequest("Chưa chọn ảnh".to_owned()))
}

/// Mọi lỗi đọc multipart (sai định dạng, vượt giới hạn body) đều quy về 400 có thông báo.
fn upload_error(_err: MultipartError) -> AppError {
    AppError::BadRequest("Tệp tải lên không hợp lệ hoặc quá lớn (tối đa 2 MB)".to_owned())
}
