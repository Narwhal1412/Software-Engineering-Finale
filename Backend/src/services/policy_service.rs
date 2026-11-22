//! Nghiệp vụ chính sách và ưu đãi.

use std::sync::Arc;

use crate::dtos::policy_dto::{PolicyRequest, PolicyResponse};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::limits::{POLICY_CONTENT_MAX_LEN, POLICY_TITLE_MAX_LEN};
use crate::models::policy::PolicyInput;
use crate::repositories::PolicyRepository;
use crate::services::authz::require_admin;
use crate::services::validation::required_text;

/// Các biểu tượng được phép dùng. Phải khớp với `ICONS` trong `Frontend/src/ui.jsx`.
pub const POLICY_ICONS: [&str; 8] = [
    "shield", "refresh", "truck", "check", "card", "gift", "phone", "box",
];
/// Thứ tự hiển thị lớn nhất chấp nhận, chặn nhập nhầm.
const MAX_SORT_ORDER: i32 = 9999;

/// Nghiệp vụ chính sách: ai cũng xem được, chỉ admin được ghi.
pub struct PolicyService {
    policies: Arc<dyn PolicyRepository>,
}

impl PolicyService {
    /// Tạo service từ repository chính sách.
    pub fn new(policies: Arc<dyn PolicyRepository>) -> Self {
        Self { policies }
    }

    /// Danh sách mọi chính sách (công khai), đã sắp theo thứ tự hiển thị.
    pub async fn list(&self) -> Result<Vec<PolicyResponse>, AppError> {
        let policies = self.policies.list().await?;
        Ok(policies.into_iter().map(Into::into).collect())
    }

    /// Tạo chính sách (admin).
    pub async fn create(
        &self,
        actor: AuthUser,
        request: PolicyRequest,
    ) -> Result<PolicyResponse, AppError> {
        require_admin(actor)?;
        let input = to_input(request)?;
        Ok(self.policies.create(input).await?.into())
    }

    /// Cập nhật chính sách (admin).
    pub async fn update(
        &self,
        actor: AuthUser,
        id: i64,
        request: PolicyRequest,
    ) -> Result<PolicyResponse, AppError> {
        require_admin(actor)?;
        let input = to_input(request)?;
        Ok(self.policies.update(id, input).await?.into())
    }

    /// Xóa chính sách (admin).
    pub async fn delete(&self, actor: AuthUser, id: i64) -> Result<(), AppError> {
        require_admin(actor)?;
        self.policies.delete(id).await?;
        Ok(())
    }
}

/// Validate request và đổi sang dữ liệu ghi của repository.
fn to_input(request: PolicyRequest) -> Result<PolicyInput, AppError> {
    let icon = request.icon.trim().to_owned();
    if !POLICY_ICONS.iter().any(|allowed| *allowed == icon) {
        return Err(AppError::BadRequest("Biểu tượng không hợp lệ".to_owned()));
    }
    let title = required_text(&request.title, "tiêu đề chính sách", POLICY_TITLE_MAX_LEN)?;
    let content = required_text(
        &request.content,
        "nội dung chính sách",
        POLICY_CONTENT_MAX_LEN,
    )?;
    if !(0..=MAX_SORT_ORDER).contains(&request.sort_order) {
        return Err(AppError::BadRequest(format!(
            "Thứ tự hiển thị phải từ 0 đến {MAX_SORT_ORDER}"
        )));
    }
    Ok(PolicyInput {
        icon,
        title,
        content,
        category_id: request.category_id,
        sort_order: request.sort_order,
    })
}
