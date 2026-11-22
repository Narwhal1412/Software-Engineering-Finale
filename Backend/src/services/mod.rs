//! TẦNG BLL (Business Logic Layer): validation, phân quyền, quy tắc nghiệp vụ,
//! chuyển model <-> DTO. Chỉ phụ thuộc vào các trait repository.

pub mod auth_service;
pub mod authz;
pub mod cart_service;
pub mod category_service;
pub mod order_service;
pub mod policy_service;
pub mod product_service;
pub mod stats_service;
pub mod upload_service;
pub mod user_service;
pub mod validation;

#[cfg(test)]
mod service_tests;

use std::sync::Arc;

use crate::app_state::Repositories;
use crate::config::Config;

use auth_service::AuthService;
use cart_service::CartService;
use category_service::CategoryService;
use order_service::OrderService;
use policy_service::PolicyService;
use product_service::ProductService;
use stats_service::StatsService;
use upload_service::UploadService;
use user_service::UserService;

/// Tập hợp mọi service, được clone cho mỗi request (chỉ clone con trỏ `Arc`).
#[derive(Clone)]
pub struct Services {
    /// Đăng ký, đăng nhập.
    pub auth: Arc<AuthService>,
    /// Hồ sơ cá nhân và quản lý thành viên.
    pub users: Arc<UserService>,
    /// Danh mục.
    pub categories: Arc<CategoryService>,
    /// Chính sách và ưu đãi.
    pub policies: Arc<PolicyService>,
    /// Sản phẩm.
    pub products: Arc<ProductService>,
    /// Upload ảnh.
    pub uploads: Arc<UploadService>,
    /// Giỏ hàng.
    pub carts: Arc<CartService>,
    /// Đơn hàng.
    pub orders: Arc<OrderService>,
    /// Thống kê quản trị.
    pub stats: Arc<StatsService>,
}

impl Services {
    /// Dựng toàn bộ service từ cấu hình và repository.
    pub fn new(config: &Arc<Config>, repos: &Repositories) -> Self {
        Self {
            auth: Arc::new(AuthService::new(repos.users.clone(), config.clone())),
            users: Arc::new(UserService::new(repos.users.clone())),
            categories: Arc::new(CategoryService::new(repos.categories.clone())),
            policies: Arc::new(PolicyService::new(repos.policies.clone())),
            products: Arc::new(ProductService::new(repos.products.clone())),
            uploads: Arc::new(UploadService::new(&config.upload_dir)),
            carts: Arc::new(CartService::new(
                repos.carts.clone(),
                repos.products.clone(),
            )),
            orders: Arc::new(OrderService::new(repos.orders.clone(), repos.carts.clone())),
            stats: Arc::new(StatsService::new(
                repos.products.clone(),
                repos.users.clone(),
                repos.orders.clone(),
            )),
        }
    }
}
