//! Entity `Product` (bảng `products`) và các kiểu phục vụ truy vấn.

use chrono::{DateTime, Utc};

/// Sản phẩm bán trong cửa hàng. Giá tính bằng đồng (số nguyên, không có phần lẻ).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Product {
    /// Khóa chính, tự tăng.
    pub id: i64,
    /// Tên sản phẩm.
    pub name: String,
    /// Mô tả thông số.
    pub specs: String,
    /// Đơn giá (VND), không âm.
    pub price: i64,
    /// Số lượng tồn kho, không âm. Giảm khi đặt hàng, hoàn lại khi đơn bị hủy.
    pub stock: i32,
    /// Danh mục; `None` khi chưa phân loại hoặc danh mục đã bị xóa.
    pub category_id: Option<i64>,
    /// URL ảnh; `None` nếu chưa có ảnh.
    pub image_url: Option<String>,
    /// Thời điểm ghim; `None` nếu không ghim.
    pub pinned_at: Option<DateTime<Utc>>,
    /// Thời điểm tạo (UTC).
    pub created_at: DateTime<Utc>,
}

/// Dữ liệu tạo mới hoặc cập nhật sản phẩm (tương ứng body của POST/PUT).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductInput {
    /// Tên sản phẩm.
    pub name: String,
    /// Mô tả thông số.
    pub specs: String,
    /// Đơn giá (VND).
    pub price: i64,
    /// Số lượng tồn kho (không âm).
    pub stock: i32,
    /// Danh mục (phải tồn tại nếu có giá trị).
    pub category_id: Option<i64>,
    /// URL ảnh.
    pub image_url: Option<String>,
}

/// Cách sắp xếp danh sách sản phẩm (sau khi đã đưa sản phẩm ghim lên đầu).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProductSort {
    /// Mới nhất trước (`sort=new`).
    #[default]
    Newest,
    /// Giá tăng dần (`sort=priceAsc`).
    PriceAsc,
    /// Giá giảm dần (`sort=priceDesc`).
    PriceDesc,
}

impl ProductSort {
    /// Chuyển giá trị `sort` của query string; giá trị lạ hoặc thiếu coi như `new`.
    pub fn from_query(raw: &str) -> Self {
        match raw {
            "priceAsc" => Self::PriceAsc,
            "priceDesc" => Self::PriceDesc,
            _ => Self::Newest,
        }
    }
}

/// Điều kiện lọc cho `GET /api/products`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProductQuery {
    /// Từ khóa tìm trong tên và thông số; `None` hoặc rỗng = không lọc.
    pub q: Option<String>,
    /// Chỉ lấy sản phẩm thuộc danh mục này.
    pub category_id: Option<i64>,
    /// Cách sắp xếp.
    pub sort: ProductSort,
}
