//! Thống kê cho trang quản trị.

use std::sync::Arc;

use crate::dtos::stats_dto::StatsResponse;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::repositories::{OrderRepository, ProductRepository, UserRepository};
use crate::services::authz::require_admin;

/// Nghiệp vụ thống kê.
pub struct StatsService {
    products: Arc<dyn ProductRepository>,
    users: Arc<dyn UserRepository>,
    orders: Arc<dyn OrderRepository>,
}

impl StatsService {
    /// Tạo service từ các repository cần đếm.
    pub fn new(
        products: Arc<dyn ProductRepository>,
        users: Arc<dyn UserRepository>,
        orders: Arc<dyn OrderRepository>,
    ) -> Self {
        Self {
            products,
            users,
            orders,
        }
    }

    /// Số sản phẩm, thành viên, đơn hàng và doanh thu (bỏ đơn đã hủy). Chỉ admin.
    pub async fn overview(&self, actor: AuthUser) -> Result<StatsResponse, AppError> {
        require_admin(actor)?;
        Ok(StatsResponse {
            products: self.products.count().await?,
            users: self.users.count().await?,
            orders: self.orders.count().await?,
            revenue: self.orders.revenue_excluding_cancelled().await?,
        })
    }
}
