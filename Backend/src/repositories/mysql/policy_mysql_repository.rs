//! Repository MariaDB cho bảng `policies`.

use async_trait::async_trait;
use sqlx::mysql::MySqlRow;
use sqlx::{MySqlPool, Row};

use super::error::{last_insert_id, lost_row, map_sqlx_error};
use crate::models::policy::{Policy, PolicyInput};
use crate::repositories::{PolicyRepository, RepoError};

/// Danh sách cột của `policies`.
const POLICY_COLUMNS: &str = "id, icon, title, content, category_id, sort_order";

/// Đọc một dòng `policies`.
fn policy_from_row(row: &MySqlRow) -> Result<Policy, RepoError> {
    Ok(Policy {
        id: row.try_get("id").map_err(map_sqlx_error)?,
        icon: row.try_get("icon").map_err(map_sqlx_error)?,
        title: row.try_get("title").map_err(map_sqlx_error)?,
        content: row.try_get("content").map_err(map_sqlx_error)?,
        category_id: row.try_get("category_id").map_err(map_sqlx_error)?,
        sort_order: row.try_get("sort_order").map_err(map_sqlx_error)?,
    })
}

/// Implementation MariaDB của [`PolicyRepository`].
pub struct MySqlPolicyRepository {
    pool: MySqlPool,
}

impl MySqlPolicyRepository {
    /// Tạo repository dùng chung pool với các repository khác.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Tìm theo id (dùng cả để đọc lại sau khi ghi).
    async fn fetch_by_id(&self, id: i64) -> Result<Option<Policy>, RepoError> {
        let sql = format!("SELECT {POLICY_COLUMNS} FROM policies WHERE id = ?");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        row.as_ref().map(policy_from_row).transpose()
    }
}

#[async_trait]
impl PolicyRepository for MySqlPolicyRepository {
    async fn list(&self) -> Result<Vec<Policy>, RepoError> {
        let sql = format!("SELECT {POLICY_COLUMNS} FROM policies ORDER BY sort_order ASC, id ASC");
        let rows = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        rows.iter().map(policy_from_row).collect()
    }

    async fn create(&self, input: PolicyInput) -> Result<Policy, RepoError> {
        let result = sqlx::query(
            "INSERT INTO policies (icon, title, content, category_id, sort_order) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(input.icon)
        .bind(input.title)
        .bind(input.content)
        .bind(input.category_id)
        .bind(input.sort_order)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?; // category_id không tồn tại -> InvalidReference

        let id = last_insert_id(&result)?;
        self.fetch_by_id(id)
            .await?
            .ok_or_else(|| lost_row("chính sách"))
    }

    async fn update(&self, id: i64, input: PolicyInput) -> Result<Policy, RepoError> {
        // Kiểm tra tồn tại trước để khớp thứ tự lỗi của mock (NotFound trước InvalidReference).
        if self.fetch_by_id(id).await?.is_none() {
            return Err(RepoError::NotFound { entity: "policy" });
        }
        sqlx::query(
            "UPDATE policies SET icon = ?, title = ?, content = ?, category_id = ?, sort_order = ? \
             WHERE id = ?",
        )
        .bind(input.icon)
        .bind(input.title)
        .bind(input.content)
        .bind(input.category_id)
        .bind(input.sort_order)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        self.fetch_by_id(id)
            .await?
            .ok_or(RepoError::NotFound { entity: "policy" })
    }

    async fn delete(&self, id: i64) -> Result<(), RepoError> {
        let result = sqlx::query("DELETE FROM policies WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound { entity: "policy" });
        }
        Ok(())
    }
}
