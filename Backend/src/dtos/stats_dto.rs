//! DTO của thống kê: `{products, users, orders, revenue}`.

use serde::Serialize;

/// Số liệu tổng quan cho trang quản trị.
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    /// Tổng số sản phẩm.
    pub products: u64,
    /// Tổng số tài khoản.
    pub users: u64,
    /// Tổng số đơn (gồm đơn đã hủy).
    pub orders: u64,
    /// Doanh thu, không tính đơn đã hủy.
    pub revenue: i64,
}
