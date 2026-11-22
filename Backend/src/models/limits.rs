//! Giới hạn độ dài cột trong database. Service phải validate theo các hằng số
//! này để lỗi được báo sớm và thống nhất giữa mock và database thật.

/// Độ dài tối đa của email (`users.email`).
pub const EMAIL_MAX_LEN: usize = 254;
/// Độ dài tối đa của tên hiển thị (`users.display_name`).
pub const DISPLAY_NAME_MAX_LEN: usize = 100;
/// Độ dài tối đa của số điện thoại (`users.phone`, `orders.phone`).
pub const PHONE_MAX_LEN: usize = 20;
/// Độ dài tối đa của địa chỉ (`users.address`, `orders.address`).
pub const ADDRESS_MAX_LEN: usize = 500;
/// Độ dài tối đa của tên danh mục (`categories.name`).
pub const CATEGORY_NAME_MAX_LEN: usize = 100;
/// Độ dài tối đa của tên sản phẩm (`products.name`, `order_items.name`).
pub const PRODUCT_NAME_MAX_LEN: usize = 200;
/// Độ dài tối đa của URL ảnh (`products.image_url`).
pub const IMAGE_URL_MAX_LEN: usize = 512;
/// Độ dài tối đa của họ tên người nhận (`orders.full_name`).
pub const FULL_NAME_MAX_LEN: usize = 100;
/// Độ dài tối đa của tên biểu tượng chính sách (`policies.icon`).
pub const POLICY_ICON_MAX_LEN: usize = 30;
/// Độ dài tối đa của tiêu đề chính sách (`policies.title`).
pub const POLICY_TITLE_MAX_LEN: usize = 150;
/// Độ dài tối đa của nội dung chính sách (`policies.content`).
pub const POLICY_CONTENT_MAX_LEN: usize = 500;
