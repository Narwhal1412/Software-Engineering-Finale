//! Trạng thái dùng chung của ứng dụng, được axum clone cho mỗi request.

use std::sync::Arc;

use crate::config::Config;
use crate::repositories::mock::{
    InMemoryCartRepository, InMemoryCategoryRepository, InMemoryOrderRepository,
    InMemoryProductRepository, InMemoryUserRepository, MockStore, SEED_ADMIN_EMAIL,
    SEED_CUSTOMER_EMAILS, SEED_PASSWORD,
};
use crate::repositories::{
    CartRepository, CategoryRepository, OrderRepository, ProductRepository, RepoError,
    UserRepository,
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

/// Dựng toàn bộ repository. ĐÂY LÀ CHỖ DUY NHẤT chọn implementation của tầng DAL.
///
/// Hiện trả về bản mock in-memory có dữ liệu mẫu. Để chuyển sang database thật,
/// thay thân hàm này bằng việc tạo connection pool từ `config` và trả về các
/// repository thật (xem docs/DATABASE_SPEC.md mục 7).
///
/// Dòng log tài khoản mẫu bên dưới thuộc về nhánh mock: khi thay thân hàm bằng DB
/// thật thì nó biến mất cùng mock, không còn lộ mật khẩu demo ra log.
pub async fn build_repositories(_config: &Config) -> Result<Repositories, RepoError> {
    let store = MockStore::seeded()?;
    tracing::info!(
        admin = SEED_ADMIN_EMAIL,
        customers = ?SEED_CUSTOMER_EMAILS,
        password = SEED_PASSWORD,
        "đang chạy với dữ liệu mock in-memory; dữ liệu mất khi tắt server"
    );
    Ok(Repositories {
        users: Arc::new(InMemoryUserRepository::new(store.clone())),
        categories: Arc::new(InMemoryCategoryRepository::new(store.clone())),
        products: Arc::new(InMemoryProductRepository::new(store.clone())),
        carts: Arc::new(InMemoryCartRepository::new(store.clone())),
        orders: Arc::new(InMemoryOrderRepository::new(store)),
    })
}
