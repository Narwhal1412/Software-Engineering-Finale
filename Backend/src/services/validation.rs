//! Hàm validate đầu vào dùng chung. Thông báo lỗi bằng tiếng Việt vì frontend hiển thị nguyên văn.

use crate::error::AppError;
use crate::models::limits::EMAIL_MAX_LEN;

/// Số chữ số tối thiểu của số điện thoại (theo pattern `[0-9]{9,11}` của frontend).
const PHONE_MIN_DIGITS: usize = 9;
/// Số chữ số tối đa của số điện thoại.
const PHONE_MAX_DIGITS: usize = 11;

/// Cắt khoảng trắng hai đầu, bắt buộc không rỗng và không dài quá `max_len` ký tự.
pub fn required_text(value: &str, label: &str, max_len: usize) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(format!("Nhập {label}")));
    }
    ensure_max_len(trimmed, label, max_len)?;
    Ok(trimmed.to_owned())
}

/// Như [`required_text`] nhưng cho phép rỗng.
pub fn optional_text(value: &str, label: &str, max_len: usize) -> Result<String, AppError> {
    let trimmed = value.trim();
    ensure_max_len(trimmed, label, max_len)?;
    Ok(trimmed.to_owned())
}

/// Số điện thoại phải gồm 9 đến 11 chữ số.
pub fn validate_phone(phone: &str) -> Result<(), AppError> {
    let digits_only = phone.chars().all(|c| c.is_ascii_digit());
    if digits_only && (PHONE_MIN_DIGITS..=PHONE_MAX_DIGITS).contains(&phone.len()) {
        Ok(())
    } else {
        Err(AppError::BadRequest(
            "Số điện thoại phải có 9 đến 11 chữ số".to_owned(),
        ))
    }
}

/// Kiểm tra email ở mức hợp lý (một `@`, có tên miền chứa dấu chấm, không có khoảng trắng).
/// Cố ý không dùng regex đầy đủ vì chuẩn email quá rộng; xác minh thật là gửi thư.
pub fn validate_email(email: &str) -> Result<(), AppError> {
    let well_formed = match email.split_once('@') {
        Some((local, domain)) => {
            !local.is_empty()
                && !domain.contains('@')
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && !email.chars().any(char::is_whitespace)
        }
        None => false,
    };
    if !well_formed {
        return Err(AppError::BadRequest("Email không hợp lệ".to_owned()));
    }
    ensure_max_len(email, "email", EMAIL_MAX_LEN)
}

/// Từ chối chuỗi dài hơn `max_len` ký tự (đếm theo ký tự Unicode, giống `VARCHAR(n)`).
fn ensure_max_len(value: &str, label: &str, max_len: usize) -> Result<(), AppError> {
    if value.chars().count() > max_len {
        Err(AppError::BadRequest(format!(
            "Độ dài {label} tối đa {max_len} ký tự"
        )))
    } else {
        Ok(())
    }
}
