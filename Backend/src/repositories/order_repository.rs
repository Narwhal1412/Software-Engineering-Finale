//! Trait truy cập bảng `orders` và `order_items`.

use async_trait::async_trait;

use super::RepoError;
use crate::models::enums::OrderStatus;
use crate::models::order::{Order, OrderDraft, OrderWithEmail};

/// Truy cập đơn hàng. Danh sách luôn sắp mới nhất trước (`created_at` giảm dần, rồi `id` giảm dần).
#[async_trait]
pub trait OrderRepository: Send + Sync {
    /// Tạo đơn từ giỏ hàng hiện tại của `user_id` trong MỘT giao dịch nguyên tử:
    /// lấy giá và tên sản phẩm hiện hành làm snapshot, tính `total`, tạo đơn với
    /// trạng thái `pending`, rồi xóa giỏ hàng. Thất bại thì không thay đổi gì.
    /// `Constraint` nếu giỏ trống; `InvalidReference { field: "user_id" }` nếu người dùng không tồn tại.
    async fn create_from_cart(&self, user_id: i64, draft: OrderDraft) -> Result<Order, RepoError>;

    /// Đơn hàng của một người dùng.
    async fn list_by_user(&self, user_id: i64) -> Result<Vec<Order>, RepoError>;

    /// Mọi đơn hàng kèm email người đặt (cho trang quản trị).
    async fn list_all_with_user_email(&self) -> Result<Vec<OrderWithEmail>, RepoError>;

    /// Tìm theo id; `Ok(None)` nếu không có.
    async fn find_by_id(&self, id: i64) -> Result<Option<Order>, RepoError>;

    /// Đổi trạng thái đơn. `NotFound` nếu không có. Quy tắc chuyển trạng thái do service quyết định.
    async fn update_status(&self, id: i64, status: OrderStatus) -> Result<Order, RepoError>;

    /// Tổng số đơn (gồm cả đơn đã hủy).
    async fn count(&self) -> Result<u64, RepoError>;

    /// Tổng `total` của các đơn có trạng thái khác `cancelled`.
    async fn revenue_excluding_cancelled(&self) -> Result<i64, RepoError>;
}
