//! Trạng thái dùng chung của ứng dụng, được axum clone cho mỗi request.

use std::sync::Arc;

use crate::config::Config;
use crate::db;
use crate::repositories::mock::{
    InMemoryCartRepository, InMemoryCategoryRepository, InMemoryOrderRepository,
    InMemoryPolicyRepository, InMemoryProductRepository, InMemoryUserRepository, MockStore,
};
use crate::repositories::mysql::{
    MySqlCartRepository, MySqlCategoryRepository, MySqlOrderRepository, MySqlPolicyRepository,
    MySqlProductRepository, MySqlUserRepository,
};
use crate::repositories::{
    CartRepository, CategoryRepository, OrderRepository, PolicyRepository, ProductRepository,
    RepoError, UserRepository,
};
use crate::services::Services;

/// Tập hợp mọi repository mà service cần. Service chỉ biết các trait, nên đổi
/// implementation không ảnh hưởng tầng trên.
#[derive(Clone)]
pub struct Repositories {
    /// Truy cập tài khoản.
    pub users: Arc<dyn UserRepository>,
    /// Truy cập danh mục.
    pub categories: Arc<dyn CategoryRepository>,
    /// Truy cập sản phẩm.
    pub products: Arc<dyn ProductRepository>,
    /// Truy cập giỏ hàng.
    pub carts: Arc<dyn CartRepository>,
    /// Truy cập đơn hàng.
    pub orders: Arc<dyn OrderRepository>,
    /// Truy cập chính sách và ưu đãi.
    pub policies: Arc<dyn PolicyRepository>,
}

/// Trạng thái toàn ứng dụng.
#[derive(Clone)]
pub struct AppState {
    /// Cấu hình đã nạp.
    pub config: Arc<Config>,
    /// Các repository.
    pub repos: Repositories,
    /// Các service nghiệp vụ, dựng từ `repos`.
    pub services: Services,
}

impl AppState {
    /// Ghép cấu hình và repository thành trạng thái ứng dụng.
    pub fn new(config: Config, repos: Repositories) -> Self {
        let config = Arc::new(config);
        let services = Services::new(&config, &repos);
        Self {
            config,
            repos,
            services,
        }
    }
}

/// Dựng repository MariaDB thật. ĐÂY LÀ CHỖ DUY NHẤT chọn implementation của tầng DAL
/// khi chạy chương trình. Mở pool (có thử lại, vì DB có thể khởi động chậm hơn backend)
/// rồi dùng chung một pool cho sáu repository.
pub async fn build_repositories(config: &Config) -> Result<Repositories, RepoError> {
    let pool = db::connect_with_retry(config).await?;
    Ok(Repositories {
        users: Arc::new(MySqlUserRepository::new(pool.clone())),
        categories: Arc::new(MySqlCategoryRepository::new(pool.clone())),
        products: Arc::new(MySqlProductRepository::new(pool.clone())),
        carts: Arc::new(MySqlCartRepository::new(pool.clone())),
        orders: Arc::new(MySqlOrderRepository::new(pool.clone())),
        policies: Arc::new(MySqlPolicyRepository::new(pool)),
    })
}

/// Dựng repository mock in-memory đã có dữ liệu mẫu. Chỉ dùng cho test tích hợp
/// (`tests/api_tests.rs`), không cần database. Dữ liệu mất khi tiến trình kết thúc.
pub fn build_mock_repositories() -> Result<Repositories, RepoError> {
    let store = MockStore::seeded()?;
    Ok(Repositories {
        users: Arc::new(InMemoryUserRepository::new(store.clone())),
        categories: Arc::new(InMemoryCategoryRepository::new(store.clone())),
        products: Arc::new(InMemoryProductRepository::new(store.clone())),
        carts: Arc::new(InMemoryCartRepository::new(store.clone())),
        orders: Arc::new(InMemoryOrderRepository::new(store.clone())),
        policies: Arc::new(InMemoryPolicyRepository::new(store)),
    })
}
