//! DTO của sản phẩm:
//! `product = {id, name, specs, price, stock, categoryId, imageUrl, pinnedAt, createdAt}`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::product::Product;

/// Sản phẩm trả về cho client.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductResponse {
    /// Id sản phẩm.
    pub id: i64,
    /// Tên sản phẩm.
    pub name: String,
    /// Thông số.
    pub specs: String,
    /// Đơn giá (VND).
    pub price: i64,
    /// Số lượng còn trong kho.
    pub stock: i32,
    /// Danh mục, `null` nếu chưa phân loại.
    pub category_id: Option<i64>,
    /// URL ảnh, `null` nếu chưa có.
    pub image_url: Option<String>,
    /// Thời điểm ghim (ISO 8601), `null` nếu không ghim.
    pub pinned_at: Option<DateTime<Utc>>,
    /// Thời điểm tạo (ISO 8601).
    pub created_at: DateTime<Utc>,
}

impl From<Product> for ProductResponse {
    fn from(product: Product) -> Self {
        Self {
            id: product.id,
            name: product.name,
            specs: product.specs,
            price: product.price,
            stock: product.stock,
            category_id: product.category_id,
            image_url: product.image_url,
            pinned_at: product.pinned_at,
            created_at: product.created_at,
        }
    }
}

/// Query string của `GET /api/products?q=&categoryId=&sort=`.
/// Mọi tham số là chuỗi vì frontend luôn gửi `categoryId=` (rỗng) khi không lọc,
/// và serde_urlencoded không parse chuỗi rỗng thành số.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductQueryParams {
    /// Từ khóa tìm trong tên và thông số.
    pub q: Option<String>,
    /// Id danh mục cần lọc.
    pub category_id: Option<String>,
    /// `new` | `priceAsc` | `priceDesc`.
    pub sort: Option<String>,
}

/// Body của `POST /api/products` và `PUT /api/products/:id`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductRequest {
    /// Tên sản phẩm.
    #[serde(default)]
    pub name: String,
    /// Thông số.
    #[serde(default)]
    pub specs: String,
    /// Đơn giá (VND); thiếu được coi là 0 để bị service từ chối với thông báo rõ ràng.
    #[serde(default)]
    pub price: i64,
    /// Số lượng tồn kho; thiếu được coi là 0 (hết hàng).
    #[serde(default)]
    pub stock: i32,
    /// Danh mục (`null` = chưa phân loại).
    #[serde(default)]
    pub category_id: Option<i64>,
    /// URL ảnh (`null` = không có ảnh).
    #[serde(default)]
    pub image_url: Option<String>,
}

/// Body của `PATCH /api/products/:id/pin`.
#[derive(Debug, Deserialize)]
pub struct PinRequest {
    /// `true` để ghim, `false` để bỏ ghim. Bắt buộc, tránh vô tình bỏ ghim khi quên gửi.
    pub pinned: bool,
}
