//! DTO của chính sách:
//! `policy = {id, icon, title, content, categoryId, sortOrder}`.

use serde::{Deserialize, Serialize};

use crate::models::policy::Policy;

/// Chính sách trả về cho client.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyResponse {
    /// Id chính sách.
    pub id: i64,
    /// Tên biểu tượng.
    pub icon: String,
    /// Tiêu đề.
    pub title: String,
    /// Nội dung mô tả.
    pub content: String,
    /// Danh mục áp dụng, `null` nếu áp dụng cho mọi sản phẩm.
    pub category_id: Option<i64>,
    /// Thứ tự hiển thị.
    pub sort_order: i32,
}

impl From<Policy> for PolicyResponse {
    fn from(policy: Policy) -> Self {
        Self {
            id: policy.id,
            icon: policy.icon,
            title: policy.title,
            content: policy.content,
            category_id: policy.category_id,
            sort_order: policy.sort_order,
        }
    }
}

/// Body của `POST /api/policies` và `PUT /api/policies/:id`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyRequest {
    /// Tên biểu tượng; thiếu được coi là rỗng để service từ chối với thông báo rõ ràng.
    #[serde(default)]
    pub icon: String,
    /// Tiêu đề.
    #[serde(default)]
    pub title: String,
    /// Nội dung mô tả.
    #[serde(default)]
    pub content: String,
    /// Danh mục áp dụng (`null` = mọi sản phẩm).
    #[serde(default)]
    pub category_id: Option<i64>,
    /// Thứ tự hiển thị; thiếu được coi là 0.
    #[serde(default)]
    pub sort_order: i32,
}
