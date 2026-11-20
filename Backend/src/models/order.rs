//! Đơn hàng: entity `Order` (bảng `orders`) và `OrderItem` (bảng `order_items`).

use chrono::{DateTime, Utc};

use super::enums::{OrderStatus, PaymentMethod};

/// Một dòng hàng của đơn; tên và giá là bản chụp (snapshot) tại thời điểm đặt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderItem {
    /// Khóa chính, tự tăng.
    pub id: i64,
    /// Đơn hàng chứa dòng này.
    pub order_id: i64,
    /// Sản phẩm đã mua.
    pub product_id: i64,
    /// Tên sản phẩm lúc đặt.
    pub name: String,
    /// Đơn giá lúc đặt (VND).
    pub price: i64,
    /// Số lượng, tối thiểu 1.
    pub quantity: i32,
}

/// Đơn hàng cùng các dòng hàng. Repository luôn nạp sẵn `items` (hai bảng gộp thành một aggregate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    /// Khóa chính, tự tăng.
    pub id: i64,
    /// Người đặt hàng.
    pub user_id: i64,
    /// Các dòng hàng.
    pub items: Vec<OrderItem>,
    /// Tổng tiền (VND) = tổng `price * quantity` của `items`, được lưu cố định.
    pub total: i64,
    /// Trạng thái hiện tại.
    pub status: OrderStatus,
    /// Phương thức thanh toán.
    pub payment_method: PaymentMethod,
    /// Họ tên người nhận.
    pub full_name: String,
    /// Số điện thoại người nhận.
    pub phone: String,
    /// Địa chỉ giao hàng.
    pub address: String,
    /// Thời điểm tạo (UTC).
    pub created_at: DateTime<Utc>,
}

/// Thông tin giao hàng do người dùng nhập khi đặt hàng; phần còn lại lấy từ giỏ hàng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderDraft {
    /// Họ tên người nhận.
    pub full_name: String,
    /// Số điện thoại người nhận.
    pub phone: String,
    /// Địa chỉ giao hàng.
    pub address: String,
    /// Phương thức thanh toán.
    pub payment_method: PaymentMethod,
}

/// Đơn hàng kèm email người đặt, dùng cho trang quản trị.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderWithEmail {
    /// Đơn hàng.
    pub order: Order,
    /// Email người đặt; `None` nếu không tìm thấy tài khoản.
    pub user_email: Option<String>,
}
