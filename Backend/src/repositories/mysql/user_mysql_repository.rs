//! Repository MariaDB cho bảng `users` (đặc tả mục 5.1).

use async_trait::async_trait;
use sqlx::mysql::MySqlRow;
use sqlx::{MySql, MySqlPool, Row};

use super::error::{decode_enum, last_insert_id, lost_row, map_sqlx_error, read_utc};
use crate::models::enums::Role;
use crate::models::user::{NewUser, ProfileUpdate, User};
use crate::repositories::{RepoError, UserRepository};

/// Cột của `users`. `role` đọc qua CAST thành chuỗi để decode ổn định.
const USER_SELECT: &str = "SELECT id, email, display_name, password_hash, phone, address, \
     CAST(role AS CHAR) AS role, created_at FROM users";

/// Implementation MariaDB của [`UserRepository`].
pub struct MySqlUserRepository {
    pool: MySqlPool,
}

impl MySqlUserRepository {
    /// Tạo repository dùng chung pool với các repository khác.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Tìm theo id (dùng cả để đọc lại sau khi ghi).
    async fn fetch_by_id(&self, id: i64) -> Result<Option<User>, RepoError> {
        let sql = format!("{USER_SELECT} WHERE id = ?");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        row.as_ref().map(user_from_row).transpose()
    }
}

fn user_from_row(row: &MySqlRow) -> Result<User, RepoError> {
    let role: String = row.try_get("role").map_err(map_sqlx_error)?;
    Ok(User {
        id: row.try_get("id").map_err(map_sqlx_error)?,
        email: row.try_get("email").map_err(map_sqlx_error)?,
        display_name: row.try_get("display_name").map_err(map_sqlx_error)?,
        password_hash: row.try_get("password_hash").map_err(map_sqlx_error)?,
        phone: row.try_get("phone").map_err(map_sqlx_error)?,
        address: row.try_get("address").map_err(map_sqlx_error)?,
        role: decode_enum::<Role>(&role)?,
        created_at: read_utc(row, "created_at")?,
    })
}

#[async_trait]
impl UserRepository for MySqlUserRepository {
    async fn create(&self, new_user: NewUser) -> Result<User, RepoError> {
        let result = sqlx::query(
            "INSERT INTO users (email, display_name, password_hash, phone, address, role, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, UTC_TIMESTAMP(6))",
        )
        .bind(new_user.email)
        .bind(new_user.display_name)
        .bind(new_user.password_hash)
        .bind(new_user.phone)
        .bind(new_user.address)
        .bind(new_user.role.as_str())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?; // email trùng -> Duplicate { field: "email" }

        let id = last_insert_id(&result)?;
        self.fetch_by_id(id)
            .await?
            .ok_or_else(|| lost_row("tài khoản"))
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, RepoError> {
        self.fetch_by_id(id).await
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError> {
        // Collation utf8mb4_unicode_ci nên so sánh không phân biệt hoa thường.
        let sql = format!("{USER_SELECT} WHERE email = ?");
        let row = sqlx::query(&sql)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        row.as_ref().map(user_from_row).transpose()
    }

    async fn list(&self) -> Result<Vec<User>, RepoError> {
        let sql = format!("{USER_SELECT} ORDER BY id ASC");
        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        rows.iter().map(user_from_row).collect()
    }

    async fn update_profile(&self, id: i64, update: ProfileUpdate) -> Result<User, RepoError> {
        sqlx::query("UPDATE users SET display_name = ?, phone = ?, address = ? WHERE id = ?")
            .bind(update.display_name)
            .bind(update.phone)
            .bind(update.address)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        // Không dùng rows_affected: MariaDB trả 0 khi giá trị không đổi. Đọc lại để biết có dòng không.
        self.fetch_by_id(id)
            .await?
            .ok_or(RepoError::NotFound { entity: "user" })
    }

    async fn delete(&self, id: i64) -> Result<(), RepoError> {
        // Giỏ hàng tự xóa nhờ ON DELETE CASCADE; đơn hàng còn thì 1451 -> InUse.
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound { entity: "user" });
        }
        Ok(())
    }

    async fn count(&self) -> Result<u64, RepoError> {
        let total = sqlx::query_scalar::<MySql, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(u64::try_from(total).unwrap_or(0))
    }
}
