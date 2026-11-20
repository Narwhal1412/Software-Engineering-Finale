//! Nghiệp vụ danh mục.

use std::sync::Arc;

use crate::dtos::category_dto::{CategoryRequest, CategoryResponse};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::limits::CATEGORY_NAME_MAX_LEN;
use crate::repositories::CategoryRepository;
use crate::services::authz::require_admin;
use crate::services::validation::required_text;

/// Nghiệp vụ danh mục: ai cũng xem được, chỉ admin được ghi.
pub struct CategoryService {
    categories: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    /// Tạo service từ repository danh mục.
    pub fn new(categories: Arc<dyn CategoryRepository>) -> Self {
        Self { categories }
    }

    /// Danh sách mọi danh mục (công khai).
    pub async fn list(&self) -> Result<Vec<CategoryResponse>, AppError> {
        let categories = self.categories.list().await?;
        Ok(categories.into_iter().map(Into::into).collect())
    }

    /// Tạo danh mục mới (admin).
    pub async fn create(
        &self,
        actor: AuthUser,
        request: CategoryRequest,
    ) -> Result<CategoryResponse, AppError> {
        require_admin(actor)?;
        let name = category_name(&request)?;
        Ok(self.categories.create(&name).await?.into())
    }

    /// Đổi tên danh mục (admin).
    pub async fn rename(
        &self,
        actor: AuthUser,
        id: i64,
        request: CategoryRequest,
    ) -> Result<CategoryResponse, AppError> {
        require_admin(actor)?;
        let name = category_name(&request)?;
        Ok(self.categories.rename(id, &name).await?.into())
    }

    /// Xóa danh mục (admin); sản phẩm thuộc danh mục được đặt về chưa phân loại.
    pub async fn delete(&self, actor: AuthUser, id: i64) -> Result<(), AppError> {
        require_admin(actor)?;
        self.categories.delete(id).await?;
        Ok(())
    }
}

/// Validate tên danh mục từ request.
fn category_name(request: &CategoryRequest) -> Result<String, AppError> {
    required_text(&request.name, "tên danh mục", CATEGORY_NAME_MAX_LEN)
}
