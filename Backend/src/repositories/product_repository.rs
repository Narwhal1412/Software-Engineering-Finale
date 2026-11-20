//! Trait truy cập bảng `products`.

use async_trait::async_trait;

use super::RepoError;
use crate::models::product::{Product, ProductInput, ProductQuery};

/// Truy cập dữ liệu sản phẩm.
#[async_trait]
pub trait ProductRepository: Send + Sync {
    /// Tìm kiếm và sắp xếp sản phẩm. Thứ tự kết quả: sản phẩm ghim đứng đầu
    /// (`pinned_at` mới nhất trước), sau đó theo `query.sort`. Từ khóa `q` được
    /// tìm (không phân biệt hoa thường) trong tên và thông số.
    async fn search(&self, query: &ProductQuery) -> Result<Vec<Product>, RepoError>;

    /// Tìm theo id; `Ok(None)` nếu không có.
    async fn find_by_id(&self, id: i64) -> Result<Option<Product>, RepoError>;

    /// Tạo sản phẩm (không ghim). `InvalidReference { field: "category_id" }` nếu danh mục không tồn tại.
    async fn create(&self, input: ProductInput) -> Result<Product, RepoError>;

    /// Cập nhật toàn bộ trường nhập. `NotFound` nếu không có;
    /// `InvalidReference { field: "category_id" }` nếu danh mục không tồn tại.
    async fn update(&self, id: i64, input: ProductInput) -> Result<Product, RepoError>;

    /// Xóa sản phẩm và mọi dòng giỏ hàng chứa nó. `NotFound` nếu không có;
    /// `InUse { entity: "product", referenced_by: "order" }` nếu đã nằm trong đơn hàng.
    async fn delete(&self, id: i64) -> Result<(), RepoError>;

    /// Ghim (`pinned_at = bây giờ`) hoặc bỏ ghim (`pinned_at = NULL`). `NotFound` nếu không có.
    async fn set_pinned(&self, id: i64, pinned: bool) -> Result<Product, RepoError>;

    /// Tổng số sản phẩm.
    async fn count(&self) -> Result<u64, RepoError>;
}
