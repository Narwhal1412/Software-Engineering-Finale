//! Trait truy cập bảng `users`.

use async_trait::async_trait;

use super::RepoError;
use crate::models::user::{NewUser, ProfileUpdate, User};

/// Truy cập dữ liệu tài khoản. Email được so sánh không phân biệt hoa thường.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Tạo tài khoản mới. Lỗi `Duplicate { field: "email" }` nếu email đã tồn tại.
    async fn create(&self, new_user: NewUser) -> Result<User, RepoError>;

    /// Tìm theo id; `Ok(None)` nếu không có.
    async fn find_by_id(&self, id: i64) -> Result<Option<User>, RepoError>;

    /// Tìm theo email (không phân biệt hoa thường); `Ok(None)` nếu không có.
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError>;

    /// Liệt kê mọi tài khoản, id tăng dần.
    async fn list(&self) -> Result<Vec<User>, RepoError>;

    /// Cập nhật hồ sơ. `NotFound` nếu id không tồn tại.
    async fn update_profile(&self, id: i64, update: ProfileUpdate) -> Result<User, RepoError>;

    /// Xóa tài khoản cùng giỏ hàng của nó. `NotFound` nếu id không tồn tại;
    /// `InUse { entity: "user", referenced_by: "order" }` nếu tài khoản đã có đơn hàng.
    /// Việc cấm xóa admin thuộc về service, không phải repository.
    async fn delete(&self, id: i64) -> Result<(), RepoError>;

    /// Tổng số tài khoản.
    async fn count(&self) -> Result<u64, RepoError>;
}
