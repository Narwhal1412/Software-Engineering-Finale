//! DTO của danh mục: `{id, name}`.

use serde::{Deserialize, Serialize};

use crate::models::category::Category;

/// Danh mục trả về cho client.
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    /// Id danh mục.
    pub id: i64,
    /// Tên danh mục.
    pub name: String,
}

impl From<Category> for CategoryResponse {
    fn from(category: Category) -> Self {
        Self {
            id: category.id,
            name: category.name,
        }
    }
}

/// Body của `POST /api/categories` và `PUT /api/categories/:id`.
#[derive(Debug, Deserialize)]
pub struct CategoryRequest {
    /// Tên danh mục; thiếu được coi là rỗng.
    #[serde(default)]
    pub name: String,
}
