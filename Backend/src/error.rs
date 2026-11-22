//! Kiểu lỗi của tầng ứng dụng và cách chuyển thành HTTP response.
//!
//! API.md quy định lỗi là HTTP 4xx kèm `{ "message": "..." }`, nên đó là
//! định dạng duy nhất được trả về.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

use crate::repositories::RepoError;

/// Thông báo chung cho lỗi 5xx; chi tiết thật chỉ ghi vào log.
const INTERNAL_ERROR_MESSAGE: &str = "Lỗi hệ thống, vui lòng thử lại sau";

/// Lỗi mà service và controller trả về. Chuỗi bên trong là thông báo hiển thị cho người dùng.
#[derive(Debug, Error)]
pub enum AppError {
    /// 400 - dữ liệu gửi lên không hợp lệ.
    #[error("{0}")]
    BadRequest(String),
    /// 401 - thiếu hoặc sai token. Lưu ý: frontend tự đăng xuất khi nhận 401 lúc đang có token.
    #[error("{0}")]
    Unauthorized(String),
    /// 403 - đã đăng nhập nhưng không đủ quyền.
    #[error("{0}")]
    Forbidden(String),
    /// 404 - không tìm thấy tài nguyên.
    #[error("{0}")]
    NotFound(String),
    /// 409 - xung đột với dữ liệu hiện có (trùng, đang được tham chiếu).
    #[error("{0}")]
    Conflict(String),
    /// 413 - tệp tải lên quá lớn.
    #[error("{0}")]
    PayloadTooLarge(String),
    /// 415 - định dạng tệp không được hỗ trợ.
    #[error("{0}")]
    UnsupportedMediaType(String),
    /// 500 - lỗi nội bộ; nội dung chỉ để log, không gửi cho client.
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    /// Mã HTTP tương ứng với lỗi.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Thân response lỗi theo API.md.
#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = match &self {
            Self::Internal(detail) => {
                // Không lộ chi tiết nội bộ ra ngoài nhưng vẫn phải truy vết được.
                tracing::error!(detail = %detail, "internal server error");
                INTERNAL_ERROR_MESSAGE.to_owned()
            }
            other => other.to_string(),
        };
        (status, Json(ErrorBody { message })).into_response()
    }
}

impl From<RepoError> for AppError {
    /// Cho phép service dùng `?` trực tiếp trên kết quả repository.
    /// Service nên `map_err` khi cần thông báo cụ thể hơn.
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::NotFound { entity } => {
                Self::NotFound(format!("Không tìm thấy {}", entity_label(entity)))
            }
            RepoError::Duplicate { field } => Self::Conflict(duplicate_message(field).to_owned()),
            RepoError::InvalidReference { field } => {
                Self::BadRequest(invalid_reference_message(field).to_owned())
            }
            RepoError::InUse {
                entity,
                referenced_by,
            } => Self::Conflict(in_use_message(entity, referenced_by).to_owned()),
            RepoError::OutOfStock { product, available } => {
                Self::Conflict(out_of_stock_message(&product, available))
            }
            RepoError::Constraint(detail) => {
                Self::BadRequest(format!("Dữ liệu không hợp lệ: {detail}"))
            }
            RepoError::Internal(detail) => Self::Internal(detail),
        }
    }
}

/// Thông báo khi số lượng đặt vượt quá tồn kho (dùng chung cho giỏ hàng và đặt hàng).
pub(crate) fn out_of_stock_message(product: &str, available: i32) -> String {
    if available <= 0 {
        format!("Sản phẩm \"{product}\" đã hết hàng")
    } else {
        format!("Sản phẩm \"{product}\" chỉ còn {available} trong kho")
    }
}

/// Tên tiếng Việt của entity, khớp thông báo mà frontend mock đang dùng.
fn entity_label(entity: &str) -> &str {
    match entity {
        "user" => "thành viên",
        "product" => "sản phẩm",
        "category" => "danh mục",
        "order" => "đơn hàng",
        "policy" => "chính sách",
        "cart item" => "sản phẩm trong giỏ hàng",
        other => other,
    }
}

/// Thông báo khi vi phạm ràng buộc UNIQUE.
fn duplicate_message(field: &str) -> &'static str {
    match field {
        "email" => "Email đã được đăng ký",
        "name" => "Tên danh mục đã tồn tại",
        _ => "Dữ liệu đã tồn tại",
    }
}

/// Thông báo khi khóa ngoại trỏ tới bản ghi không tồn tại.
fn invalid_reference_message(field: &str) -> &'static str {
    match field {
        "category_id" => "Danh mục không tồn tại",
        "product_id" => "Không tìm thấy sản phẩm",
        "user_id" => "Không tìm thấy thành viên",
        _ => "Tham chiếu không hợp lệ",
    }
}

/// Thông báo khi bản ghi đang bị bản ghi khác tham chiếu nên không xóa được.
fn in_use_message(entity: &str, referenced_by: &str) -> &'static str {
    match (entity, referenced_by) {
        ("product", "order") => "Sản phẩm đã có trong đơn hàng nên không thể xóa",
        ("user", "order") => "Thành viên đã có đơn hàng nên không thể xóa",
        _ => "Dữ liệu đang được sử dụng nên không thể xóa",
    }
}
