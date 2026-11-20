//! Extractor JSON trả lỗi theo định dạng `{message}` của API.md.

use async_trait::async_trait;
use axum::extract::{FromRequest, Request};
use axum::Json;
use serde::de::DeserializeOwned;

use crate::error::AppError;

/// Giống `axum::Json` khi đọc request, nhưng body sai cú pháp hay sai kiểu trả
/// `400 {message}` thay vì văn bản thuần của axum, để frontend hiển thị được.
pub struct ApiJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(_) => Err(AppError::BadRequest(
                "Dữ liệu gửi lên không hợp lệ".to_owned(),
            )),
        }
    }
}
