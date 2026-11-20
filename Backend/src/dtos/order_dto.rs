//! DTO của đơn hàng:
//! `order = {id, items:[{productId,name,price,quantity}], total, status, paymentMethod, fullName, phone, address, createdAt}`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::enums::{OrderStatus, PaymentMethod};
use crate::models::order::{Order, OrderItem, OrderWithEmail};

/// Một dòng hàng của đơn.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderItemResponse {
    /// Id sản phẩm.
    pub product_id: i64,
    /// Tên sản phẩm lúc đặt.
    pub name: String,
    /// Đơn giá lúc đặt.
    pub price: i64,
    /// Số lượng.
    pub quantity: i32,
}

impl From<OrderItem> for OrderItemResponse {
    fn from(item: OrderItem) -> Self {
        Self {
            product_id: item.product_id,
            name: item.name,
            price: item.price,
            quantity: item.quantity,
        }
    }
}

/// Đơn hàng trả về cho client.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    /// Id đơn.
    pub id: i64,
    /// Các dòng hàng.
    pub items: Vec<OrderItemResponse>,
    /// Tổng tiền (VND).
    pub total: i64,
    /// Trạng thái.
    pub status: OrderStatus,
    /// Phương thức thanh toán.
    pub payment_method: PaymentMethod,
    /// Họ tên người nhận.
    pub full_name: String,
    /// Số điện thoại người nhận.
    pub phone: String,
    /// Địa chỉ giao hàng.
    pub address: String,
    /// Thời điểm tạo (ISO 8601).
    pub created_at: DateTime<Utc>,
}

impl From<Order> for OrderResponse {
    fn from(order: Order) -> Self {
        Self {
            id: order.id,
            items: order.items.into_iter().map(Into::into).collect(),
            total: order.total,
            status: order.status,
            payment_method: order.payment_method,
            full_name: order.full_name,
            phone: order.phone,
            address: order.address,
            created_at: order.created_at,
        }
    }
}

/// Đơn hàng cho trang quản trị: `order + userEmail`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminOrderResponse {
    /// Các trường của đơn, trải phẳng cùng cấp với `userEmail`.
    #[serde(flatten)]
    pub order: OrderResponse,
    /// Email người đặt.
    pub user_email: Option<String>,
}

impl From<OrderWithEmail> for AdminOrderResponse {
    fn from(row: OrderWithEmail) -> Self {
        Self {
            order: row.order.into(),
            user_email: row.user_email,
        }
    }
}

/// Body của `POST /api/orders`. `payment_method` là chuỗi để service tự báo lỗi tiếng Việt khi giá trị lạ.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    /// Họ tên người nhận.
    #[serde(default)]
    pub full_name: String,
    /// Số điện thoại người nhận.
    #[serde(default)]
    pub phone: String,
    /// Địa chỉ giao hàng.
    #[serde(default)]
    pub address: String,
    /// `cod` hoặc `bank`.
    #[serde(default)]
    pub payment_method: String,
}

/// Body của `PATCH /api/admin/orders/:id`.
#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    /// Một trong `pending|confirmed|shipped|completed|cancelled`.
    #[serde(default)]
    pub status: String,
}
