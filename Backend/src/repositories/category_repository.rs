//! Trait truy cập bảng `categories`.

use async_trait::async_trait;

use super::RepoError;
use crate::models::category::Category;

/// Truy cập dữ liệu danh mục. Tên danh mục là duy nhất, không phân biệt hoa thường.
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    /// Liệt kê mọi danh mục, id tăng dần.
    async fn list(&self) -> Result<Vec<Category>, RepoError>;

    /// Tìm theo id; `Ok(None)` nếu không có.
    async fn find_by_id(&self, id: i64) -> Result<Option<Category>, RepoError>;

    /// Tạo danh mục. `Duplicate { field: "name" }` nếu trùng tên.
    async fn create(&self, name: &str) -> Result<Category, RepoError>;

    /// Đổi tên. `NotFound` nếu không có; `Duplicate { field: "name" }` nếu trùng tên danh mục khác.
    async fn rename(&self, id: i64, name: &str) -> Result<Category, RepoError>;

    /// Xóa danh mục; mọi sản phẩm thuộc danh mục này được đặt `category_id = NULL`
    /// (không xóa sản phẩm). `NotFound` nếu không có.
    async fn delete(&self, id: i64) -> Result<(), RepoError>;
}
