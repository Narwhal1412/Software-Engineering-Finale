//! Đổi lỗi sqlx/MariaDB sang `RepoError` và các hàm đọc dữ liệu dùng chung (đặc tả mục 4).
//!
//! Ánh xạ lỗi dựa vào TÊN ràng buộc (`uq_...`, `fk_...`): không đổi tên ràng buộc trong
//! `Database/init scripts/init.sql` mà không sửa file này.

use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::mysql::{MySqlDatabaseError, MySqlQueryResult, MySqlRow};
use sqlx::Row;

use crate::models::enums::ParseEnumError;
use crate::repositories::RepoError;

/// Đổi một lỗi sqlx thành `RepoError`. Trường hợp "không thấy dòng" (`NotFound`)
/// do từng repository tự quyết, vì chỉ repository biết entity nào.
pub(super) fn map_sqlx_error(err: sqlx::Error) -> RepoError {
    if let sqlx::Error::Database(db) = &err {
        let message = db.message();
        let code = db
            .try_downcast_ref::<MySqlDatabaseError>()
            .map(MySqlDatabaseError::number);
        match code {
            Some(1062) => {
                if message.contains("uq_users_email") {
                    return RepoError::Duplicate { field: "email" };
                }
                if message.contains("uq_categories_name") {
                    return RepoError::Duplicate { field: "name" };
                }
            }
            Some(1452) => {
                return RepoError::InvalidReference {
                    field: reference_field(message),
                };
            }
            Some(1451) => {
                if message.contains("fk_order_items_product") {
                    return RepoError::InUse {
                        entity: "product",
                        referenced_by: "order",
                    };
                }
                if message.contains("fk_orders_user") {
                    return RepoError::InUse {
                        entity: "user",
                        referenced_by: "order",
                    };
                }
            }
            // 4025: CHECK của MariaDB; 3819: CHECK của MySQL; 1406: chuỗi quá dài.
            Some(4025) | Some(3819) | Some(1406) => {
                return RepoError::Constraint(message.to_owned());
            }
            _ => {}
        }
    }
    RepoError::Internal(err.to_string())
}

/// Tên trường khóa ngoại, suy ra từ tên ràng buộc.
fn reference_field(message: &str) -> &'static str {
    if message.contains("fk_products_category") {
        "category_id"
    } else if message.contains("fk_cart_items_user") || message.contains("fk_orders_user") {
        "user_id"
    } else if message.contains("fk_cart_items_product") || message.contains("fk_order_items_product")
    {
        "product_id"
    } else {
        "reference"
    }
}

/// Lấy id vừa tạo bằng `AUTO_INCREMENT`.
pub(super) fn last_insert_id(result: &MySqlQueryResult) -> Result<i64, RepoError> {
    i64::try_from(result.last_insert_id()).map_err(|err| RepoError::Internal(err.to_string()))
}

/// Đọc cột enum (đã `CAST ... AS CHAR`) thành kiểu Rust.
pub(super) fn decode_enum<T>(raw: &str) -> Result<T, RepoError>
where
    T: FromStr<Err = ParseEnumError>,
{
    raw.parse::<T>()
        .map_err(|err| RepoError::Internal(err.to_string()))
}

/// Đọc cột `DATETIME(6)` không NULL (lưu UTC) thành `DateTime<Utc>`.
pub(super) fn read_utc(row: &MySqlRow, column: &str) -> Result<DateTime<Utc>, RepoError> {
    let naive: NaiveDateTime = row.try_get(column).map_err(map_sqlx_error)?;
    Ok(naive.and_utc())
}

/// Đọc cột `DATETIME(6)` có thể NULL.
pub(super) fn read_opt_utc(
    row: &MySqlRow,
    column: &str,
) -> Result<Option<DateTime<Utc>>, RepoError> {
    let naive: Option<NaiveDateTime> = row.try_get(column).map_err(map_sqlx_error)?;
    Ok(naive.map(|time| time.and_utc()))
}

/// Lỗi khi vừa ghi xong mà không đọc lại được dòng (không nên xảy ra).
pub(super) fn lost_row(what: &str) -> RepoError {
    RepoError::Internal(format!("{what} vừa được ghi nhưng không đọc lại được"))
}
