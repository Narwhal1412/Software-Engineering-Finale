//! Repository mock cho `products`.

use std::cmp::Ordering;

use async_trait::async_trait;
use chrono::Utc;

use super::constraints::{check_len, check_price, check_stock};
use super::store::{to_count, MockStore, Tables};
use crate::models::limits::{IMAGE_URL_MAX_LEN, PRODUCT_NAME_MAX_LEN};
use crate::models::product::{Product, ProductInput, ProductQuery, ProductSort};
use crate::repositories::{ProductRepository, RepoError};

/// Implementation in-memory của [`ProductRepository`].
pub struct InMemoryProductRepository {
    store: MockStore,
}

impl InMemoryProductRepository {
    /// Tạo repository dùng chung `store` với các repository mock khác.
    pub fn new(store: MockStore) -> Self {
        Self { store }
    }
}

/// Kiểm tra các ràng buộc cột của dữ liệu nhập (độ dài, giá không âm).
fn validate_input(input: &ProductInput) -> Result<(), RepoError> {
    check_len("name", &input.name, PRODUCT_NAME_MAX_LEN)?;
    check_price(input.price)?;
    check_stock(input.stock)?;
    if let Some(url) = &input.image_url {
        check_len("image_url", url, IMAGE_URL_MAX_LEN)?;
    }
    Ok(())
}

/// Mô phỏng khóa ngoại `products.category_id -> categories.id`.
fn ensure_category_exists(tables: &Tables, category_id: Option<i64>) -> Result<(), RepoError> {
    match category_id {
        Some(id) if !tables.categories.contains_key(&id) => {
            Err(RepoError::InvalidReference {
                field: "category_id",
            })
        }
        _ => Ok(()),
    }
}

/// Từ khóa (đã viết thường) có nằm trong tên hoặc thông số không.
fn matches_keyword(product: &Product, keyword: &str) -> bool {
    product.name.to_lowercase().contains(keyword) || product.specs.to_lowercase().contains(keyword)
}

/// Thứ tự hiển thị: sản phẩm ghim trước (ghim mới nhất trước), rồi theo `sort`.
fn display_order(a: &Product, b: &Product, sort: ProductSort) -> Ordering {
    let pinned_first = b.pinned_at.is_some().cmp(&a.pinned_at.is_some());
    let newest_pin_first = match (a.pinned_at, b.pinned_at) {
        (Some(a_pin), Some(b_pin)) => b_pin.cmp(&a_pin),
        _ => Ordering::Equal,
    };
    let by_sort = match sort {
        ProductSort::Newest => b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)),
        ProductSort::PriceAsc => a.price.cmp(&b.price).then(a.id.cmp(&b.id)),
        ProductSort::PriceDesc => b.price.cmp(&a.price).then(a.id.cmp(&b.id)),
    };
    pinned_first.then(newest_pin_first).then(by_sort)
}

#[async_trait]
impl ProductRepository for InMemoryProductRepository {
    async fn search(&self, query: &ProductQuery) -> Result<Vec<Product>, RepoError> {
        let keyword = query
            .q
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_lowercase);

        let tables = self.store.read().await;
        let mut rows: Vec<Product> = tables
            .products
            .values()
            .filter(|product| match query.category_id {
                Some(category_id) => product.category_id == Some(category_id),
                None => true,
            })
            .filter(|product| match &keyword {
                Some(text) => matches_keyword(product, text),
                None => true,
            })
            .cloned()
            .collect();
        rows.sort_by(|a, b| display_order(a, b, query.sort));
        Ok(rows)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Product>, RepoError> {
        Ok(self.store.read().await.products.get(&id).cloned())
    }

    async fn create(&self, input: ProductInput) -> Result<Product, RepoError> {
        validate_input(&input)?;

        let mut guard = self.store.write().await;
        let tables = &mut *guard;
        ensure_category_exists(tables, input.category_id)?;

        let product = Product {
            id: tables.sequences.product.next(),
            name: input.name,
            specs: input.specs,
            price: input.price,
            stock: input.stock,
            category_id: input.category_id,
            image_url: input.image_url,
            pinned_at: None,
            created_at: Utc::now(),
        };
        tables.products.insert(product.id, product.clone());
        Ok(product)
    }

    async fn update(&self, id: i64, input: ProductInput) -> Result<Product, RepoError> {
        validate_input(&input)?;

        let mut guard = self.store.write().await;
        let tables = &mut *guard;
        ensure_category_exists(tables, input.category_id)?;

        let product = tables
            .products
            .get_mut(&id)
            .ok_or(RepoError::NotFound { entity: "product" })?;
        product.name = input.name;
        product.specs = input.specs;
        product.price = input.price;
        product.stock = input.stock;
        product.category_id = input.category_id;
        product.image_url = input.image_url;
        Ok(product.clone())
    }

    async fn delete(&self, id: i64) -> Result<(), RepoError> {
        let mut guard = self.store.write().await;
        let tables = &mut *guard;

        if !tables.products.contains_key(&id) {
            return Err(RepoError::NotFound { entity: "product" });
        }
        // ON DELETE RESTRICT từ order_items: đơn hàng cũ phải còn tham chiếu được.
        let in_order = tables
            .orders
            .values()
            .any(|order| order.items.iter().any(|item| item.product_id == id));
        if in_order {
            return Err(RepoError::InUse {
                entity: "product",
                referenced_by: "order",
            });
        }

        tables.products.remove(&id);
        // ON DELETE CASCADE từ cart_items.
        tables.cart_items.retain(|&(_, product_id), _| product_id != id);
        Ok(())
    }

    async fn set_pinned(&self, id: i64, pinned: bool) -> Result<Product, RepoError> {
        let mut tables = self.store.write().await;
        let product = tables
            .products
            .get_mut(&id)
            .ok_or(RepoError::NotFound { entity: "product" })?;
        product.pinned_at = if pinned { Some(Utc::now()) } else { None };
        Ok(product.clone())
    }

    async fn count(&self) -> Result<u64, RepoError> {
        Ok(to_count(self.store.read().await.products.len()))
    }
}
