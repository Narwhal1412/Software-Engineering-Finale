//! Các kiểm tra mô phỏng ràng buộc cột mà database thật sẽ áp dụng (độ dài, CHECK).

use crate::repositories::RepoError;

/// Mô phỏng giới hạn `VARCHAR(n)`: MariaDB ở strict mode từ chối chuỗi quá dài.
pub(super) fn check_len(field: &'static str, value: &str, max: usize) -> Result<(), RepoError> {
    if value.chars().count() > max {
        Err(RepoError::Constraint(format!(
            "{field} exceeds {max} characters"
        )))
    } else {
        Ok(())
    }
}

/// Mô phỏng `CHECK (price >= 0)`.
pub(super) fn check_price(price: i64) -> Result<(), RepoError> {
    if price < 0 {
        Err(RepoError::Constraint("price must be >= 0".to_owned()))
    } else {
        Ok(())
    }
}

/// Mô phỏng `CHECK (quantity >= 1)`.
pub(super) fn check_quantity(quantity: i32) -> Result<(), RepoError> {
    if quantity < 1 {
        Err(RepoError::Constraint("quantity must be >= 1".to_owned()))
    } else {
        Ok(())
    }
}
