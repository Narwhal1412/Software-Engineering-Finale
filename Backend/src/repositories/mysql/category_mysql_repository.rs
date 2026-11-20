//! Repository MariaDB cho bảng `categories` (đặc tả mục 5.2).

use async_trait::async_trait;
use sqlx::mysql::MySqlRow;
use sqlx::{MySqlPool, Row};

use super::error::{last_insert_id, lost_row, map_sqlx_error};
use crate::models::category::Category;
use crate::repositories::{CategoryRepository, RepoError};

/// Implementation MariaDB của [`CategoryRepository`].
pub struct MySqlCategoryRepository {
    pool: MySqlPool,
}

impl MySqlCategoryRepository {
    /// Tạo repository dùng chung pool với các repository khác.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Tìm theo id (dùng cả để đọc lại sau khi ghi).
    async fn fetch_by_id(&self, id: i64) -> Result<Option<Category>, RepoError> {
        let row = sqlx::query("SELECT id, name FROM categories WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        row.as_ref().map(category_from_row).transpose()
    }
}

fn category_from_row(row: &MySqlRow) -> Result<Category, RepoError> {
    Ok(Category {
        id: row.try_get("id").map_err(map_sqlx_error)?,
        name: row.try_get("name").map_err(map_sqlx_error)?,
    })
}

#[async_trait]
impl CategoryRepository for MySqlCategoryRepository {
    async fn list(&self) -> Result<Vec<Category>, RepoError> {
        let rows = sqlx::query("SELECT id, name FROM categories ORDER BY id ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        rows.iter().map(category_from_row).collect()
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Category>, RepoError> {
        self.fetch_by_id(id).await
    }

    async fn create(&self, name: &str) -> Result<Category, RepoError> {
        let result = sqlx::query("INSERT INTO categories (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?; // trùng tên -> Duplicate { field: "name" }

        let id = last_insert_id(&result)?;
        self.fetch_by_id(id)
            .await?
            .ok_or_else(|| lost_row("danh mục"))
    }

    async fn rename(&self, id: i64, name: &str) -> Result<Category, RepoError> {
        // Kiểm tra tồn tại trước để khớp thứ tự lỗi của mock (NotFound trước Duplicate).
        if self.fetch_by_id(id).await?.is_none() {
            return Err(RepoError::NotFound { entity: "category" });
        }
        sqlx::query("UPDATE categories SET name = ? WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?; // trùng tên -> Duplicate { field: "name" }

        self.fetch_by_id(id)
            .await?
            .ok_or(RepoError::NotFound { entity: "category" })
    }

    async fn delete(&self, id: i64) -> Result<(), RepoError> {
        // ON DELETE SET NULL: sản phẩm thuộc danh mục vẫn còn, chỉ mất phân loại.
        let result = sqlx::query("DELETE FROM categories WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound { entity: "category" });
        }
        Ok(())
    }
}
