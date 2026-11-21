//! Nghiệp vụ đơn hàng.

use std::str::FromStr;
use std::sync::Arc;

use crate::dtos::order_dto::{
    AdminOrderResponse, CreateOrderRequest, OrderResponse, UpdateOrderStatusRequest,
};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::enums::{OrderStatus, PaymentMethod};
use crate::models::limits::{ADDRESS_MAX_LEN, FULL_NAME_MAX_LEN};
use crate::models::order::OrderDraft;
use crate::repositories::{CartRepository, OrderRepository};
use crate::services::authz::require_admin;
use crate::services::validation::{required_text, validate_phone};

/// Nghiệp vụ đơn hàng.
pub struct OrderService {
    orders: Arc<dyn OrderRepository>,
    carts: Arc<dyn CartRepository>,
}

impl OrderService {
    /// Tạo service từ repository đơn hàng và giỏ hàng.
    pub fn new(orders: Arc<dyn OrderRepository>, carts: Arc<dyn CartRepository>) -> Self {
        Self { orders, carts }
    }

    /// Đặt hàng từ giỏ hàng hiện tại. Giá và tên lấy từ sản phẩm lúc đặt (snapshot),
    /// giỏ được xóa và đơn có trạng thái `pending` — repository làm nguyên tử cả chuỗi này.
    pub async fn place_order(
        &self,
        actor: AuthUser,
        request: CreateOrderRequest,
    ) -> Result<OrderResponse, AppError> {
        let draft = to_draft(request)?;
        // Kiểm tra trước để trả thông báo rõ ràng; repository vẫn chặn giỏ trống nếu có race.
        if self.carts.list_with_products(actor.id).await?.is_empty() {
            return Err(AppError::BadRequest("Giỏ hàng đang trống".to_owned()));
        }
        Ok(self.orders.create_from_cart(actor.id, draft).await?.into())
    }

    /// Đơn hàng của người dùng hiện tại, mới nhất trước.
    pub async fn list_mine(&self, actor: AuthUser) -> Result<Vec<OrderResponse>, AppError> {
        let orders = self.orders.list_by_user(actor.id).await?;
        Ok(orders.into_iter().map(Into::into).collect())
    }

    /// Khách tự hủy đơn của chính mình, chỉ khi đơn còn `pending` (chưa được admin xác nhận).
    /// Đơn của người khác trả 404 (không tiết lộ đơn có tồn tại); đơn đã qua `pending` trả 409.
    pub async fn cancel_order(&self, actor: AuthUser, id: i64) -> Result<OrderResponse, AppError> {
        let order = self
            .orders
            .find_by_id(id)
            .await?
            .filter(|order| order.user_id == actor.id)
            .ok_or_else(|| AppError::NotFound("Không tìm thấy đơn hàng".to_owned()))?;
        ensure_cancellable(order.status)?;
        Ok(self
            .orders
            .update_status(id, OrderStatus::Cancelled)
            .await?
            .into())
    }

    /// Mọi đơn hàng kèm email người đặt (admin).
    pub async fn list_all(&self, actor: AuthUser) -> Result<Vec<AdminOrderResponse>, AppError> {
        require_admin(actor)?;
        let rows = self.orders.list_all_with_user_email().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Đổi trạng thái đơn (admin), tuân theo luật chuyển trạng thái.
    pub async fn update_status(
        &self,
        actor: AuthUser,
        id: i64,
        request: UpdateOrderStatusRequest,
    ) -> Result<OrderResponse, AppError> {
        require_admin(actor)?;
        let next = OrderStatus::from_str(request.status.trim())
            .map_err(|_| AppError::BadRequest("Trạng thái không hợp lệ".to_owned()))?;
        let current = self
            .orders
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("Không tìm thấy đơn hàng".to_owned()))?
            .status;
        ensure_transition_allowed(current, next)?;
        Ok(self.orders.update_status(id, next).await?.into())
    }
}

/// Validate thông tin giao hàng và phương thức thanh toán.
fn to_draft(request: CreateOrderRequest) -> Result<OrderDraft, AppError> {
    let full_name = required_text(&request.full_name, "họ tên", FULL_NAME_MAX_LEN)?;
    let phone = request.phone.trim().to_owned();
    validate_phone(&phone)?;
    let address = required_text(&request.address, "địa chỉ", ADDRESS_MAX_LEN)?;
    let payment_method = PaymentMethod::from_str(request.payment_method.trim())
        .map_err(|_| AppError::BadRequest("Chọn phương thức thanh toán".to_owned()))?;
    Ok(OrderDraft {
        full_name,
        phone,
        address,
        payment_method,
    })
}

/// Khách chỉ được hủy khi đơn còn chờ xác nhận; sau đó việc hủy phải do admin xử lý.
fn ensure_cancellable(status: OrderStatus) -> Result<(), AppError> {
    match status {
        OrderStatus::Pending => Ok(()),
        OrderStatus::Cancelled => Err(AppError::Conflict("Đơn hàng đã được hủy trước đó".to_owned())),
        _ => Err(AppError::Conflict(
            "Đơn hàng đã được xác nhận, không thể tự hủy. Vui lòng liên hệ cửa hàng".to_owned(),
        )),
    }
}

/// Đơn đã hoàn tất hoặc đã hủy là trạng thái cuối: không mở lại hay chuyển sang trạng thái khác,
/// vì doanh thu và tồn kho thực tế đã được tính theo kết quả đó. Đặt lại đúng trạng thái hiện tại thì được.
fn ensure_transition_allowed(current: OrderStatus, next: OrderStatus) -> Result<(), AppError> {
    let is_final = matches!(current, OrderStatus::Completed | OrderStatus::Cancelled);
    if is_final && current != next {
        Err(AppError::Conflict(
            "Đơn hàng đã hoàn tất hoặc đã hủy, không thể đổi trạng thái".to_owned(),
        ))
    } else {
        Ok(())
    }
}
