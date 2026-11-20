//! Nghiệp vụ sản phẩm.

use std::sync::Arc;

use crate::dtos::product_dto::{ProductQueryParams, ProductRequest, ProductResponse};
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::limits::{IMAGE_URL_MAX_LEN, PRODUCT_NAME_MAX_LEN};
use crate::models::product::{ProductInput, ProductQuery, ProductSort};
use crate::repositories::ProductRepository;
use crate::services::authz::require_admin;
use crate::services::validation::{optional_text, required_text};

/// Độ dài tối đa của phần thông số (cột `TEXT`, giới hạn thực tế cho dễ hiển thị).
const SPECS_MAX_LEN: usize = 5000;
/// Giá tối đa chấp nhận (100 tỷ đồng), chặn nhập nhầm và tràn số khi nhân với số lượng.
const MAX_PRICE: i64 = 100_000_000_000;
/// Các tiền tố URL ảnh được phép: ảnh upload nội bộ hoặc ảnh ngoài qua HTTP(S).
const ALLOWED_IMAGE_PREFIXES: [&str; 3] = ["/uploads/", "http://", "https://"];

/// Nghiệp vụ sản phẩm: ai cũng xem được, chỉ admin được ghi.
pub struct ProductService {
    products: Arc<dyn ProductRepository>,
}

impl ProductService {
    /// Tạo service từ repository sản phẩm.
    pub fn new(products: Arc<dyn ProductRepository>) -> Self {
        Self { products }
    }

    /// Tìm kiếm, lọc, sắp xếp sản phẩm (công khai). Sản phẩm ghim luôn ở đầu.
    pub async fn search(&self, params: ProductQueryParams) -> Result<Vec<ProductResponse>, AppError> {
        let query = ProductQuery {
            q: params.q,
            category_id: parse_category_filter(params.category_id.as_deref())?,
            sort: ProductSort::from_query(params.sort.as_deref().unwrap_or("")),
        };
        let products = self.products.search(&query).await?;
        Ok(products.into_iter().map(Into::into).collect())
    }

    /// Tạo sản phẩm (admin).
    pub async fn create(
        &self,
        actor: AuthUser,
        request: ProductRequest,
    ) -> Result<ProductResponse, AppError> {
        require_admin(actor)?;
        let input = to_input(request)?;
        Ok(self.products.create(input).await?.into())
    }

    /// Cập nhật sản phẩm (admin).
    pub async fn update(
        &self,
        actor: AuthUser,
        id: i64,
        request: ProductRequest,
    ) -> Result<ProductResponse, AppError> {
        require_admin(actor)?;
        let input = to_input(request)?;
        Ok(self.products.update(id, input).await?.into())
    }

    /// Xóa sản phẩm (admin). Sản phẩm đã nằm trong đơn hàng bị từ chối với 409.
    pub async fn delete(&self, actor: AuthUser, id: i64) -> Result<(), AppError> {
        require_admin(actor)?;
        self.products.delete(id).await?;
        Ok(())
    }

    /// Ghim hoặc bỏ ghim sản phẩm (admin).
    pub async fn set_pinned(
        &self,
        actor: AuthUser,
        id: i64,
        pinned: bool,
    ) -> Result<ProductResponse, AppError> {
        require_admin(actor)?;
        Ok(self.products.set_pinned(id, pinned).await?.into())
    }
}

/// Chuyển và validate body POST/PUT thành dữ liệu cho repository.
fn to_input(request: ProductRequest) -> Result<ProductInput, AppError> {
    let name = required_text(&request.name, "tên sản phẩm", PRODUCT_NAME_MAX_LEN)?;
    let specs = optional_text(&request.specs, "thông số", SPECS_MAX_LEN)?;
    validate_price(request.price)?;
    Ok(ProductInput {
        name,
        specs,
        price: request.price,
        category_id: request.category_id,
        image_url: clean_image_url(request.image_url.as_deref())?,
    })
}

/// Giá phải dương và không vượt trần.
fn validate_price(price: i64) -> Result<(), AppError> {
    if price <= 0 {
        Err(AppError::BadRequest("Giá phải lớn hơn 0".to_owned()))
    } else if price > MAX_PRICE {
        Err(AppError::BadRequest("Giá quá lớn".to_owned()))
    } else {
        Ok(())
    }
}

/// Rỗng -> `None`; còn lại phải là đường dẫn ảnh hợp lệ. Chặn các scheme lạ (như `javascript:`)
/// vì URL này sẽ được frontend đưa vào thẻ `<img>`.
fn clean_image_url(raw: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(url) = raw.map(str::trim).filter(|url| !url.is_empty()) else {
        return Ok(None);
    };
    if url.chars().count() > IMAGE_URL_MAX_LEN {
        return Err(AppError::BadRequest("Đường dẫn ảnh quá dài".to_owned()));
    }
    if ALLOWED_IMAGE_PREFIXES
        .into_iter()
        .any(|prefix| url.starts_with(prefix))
    {
        Ok(Some(url.to_owned()))
    } else {
        Err(AppError::BadRequest("Đường dẫn ảnh không hợp lệ".to_owned()))
    }
}

/// Frontend luôn gửi `categoryId=` (rỗng) khi không lọc, nên rỗng nghĩa là "tất cả".
fn parse_category_filter(raw: Option<&str>) -> Result<Option<i64>, AppError> {
    match raw.map(str::trim) {
        None | Some("") => Ok(None),
        Some(text) => text
            .parse::<i64>()
            .map(Some)
            .map_err(|_| AppError::BadRequest("Danh mục không hợp lệ".to_owned())),
    }
}
