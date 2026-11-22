//! Trait truy cập bảng `policies`.

use async_trait::async_trait;

use super::RepoError;
use crate::models::policy::{Policy, PolicyInput};

/// Truy cập dữ liệu chính sách và ưu đãi.
#[async_trait]
pub trait PolicyRepository: Send + Sync {
    /// Liệt kê mọi chính sách, theo `sort_order` tăng dần rồi `id` tăng dần.
    async fn list(&self) -> Result<Vec<Policy>, RepoError>;

    /// Tạo chính sách. `InvalidReference { field: "category_id" }` nếu danh mục không tồn tại.
    async fn create(&self, input: PolicyInput) -> Result<Policy, RepoError>;

    /// Cập nhật chính sách. `NotFound` nếu không có; `InvalidReference` nếu danh mục không tồn tại.
    async fn update(&self, id: i64, input: PolicyInput) -> Result<Policy, RepoError>;

    /// Xóa chính sách. `NotFound` nếu không có.
    async fn delete(&self, id: i64) -> Result<(), RepoError>;
}
