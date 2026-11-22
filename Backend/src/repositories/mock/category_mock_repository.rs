//! Repository mock cho `categories`.

use async_trait::async_trait;

use super::constraints::check_len;
use super::store::{MockStore, Tables};
use crate::models::category::Category;
use crate::models::limits::CATEGORY_NAME_MAX_LEN;
use crate::repositories::{CategoryRepository, RepoError};

/// Implementation in-memory của [`CategoryRepository`].
pub struct InMemoryCategoryRepository {
    store: MockStore,
}

impl InMemoryCategoryRepository {
    /// Tạo repository dùng chung `store` với các repository mock khác.
    pub fn new(store: MockStore) -> Self {
        Self { store }
    }
}

/// Kiểm tra tên chưa bị danh mục khác (khác `except_id`) dùng, không phân biệt hoa thường.
fn ensure_name_available(
    tables: &Tables,
    name: &str,
    except_id: Option<i64>,
) -> Result<(), RepoError> {
    let name_key = name.to_lowercase();
    let taken = tables.categories.values().any(|category| {
        Some(category.id) != except_id && category.name.to_lowercase() == name_key
    });
    if taken {
        Err(RepoError::Duplicate { field: "name" })
    } else {
        Ok(())
    }
}

#[async_trait]
impl CategoryRepository for InMemoryCategoryRepository {
    async fn list(&self) -> Result<Vec<Category>, RepoError> {
        Ok(self
            .store
            .read()
            .await
            .categories
            .values()
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Category>, RepoError> {
        Ok(self.store.read().await.categories.get(&id).cloned())
    }

    async fn create(&self, name: &str) -> Result<Category, RepoError> {
        check_len("name", name, CATEGORY_NAME_MAX_LEN)?;

        let mut guard = self.store.write().await;
        let tables = &mut *guard;
        ensure_name_available(tables, name, None)?;

        let category = Category {
            id: tables.sequences.category.next(),
            name: name.to_owned(),
        };
        tables.categories.insert(category.id, category.clone());
        Ok(category)
    }

    async fn rename(&self, id: i64, name: &str) -> Result<Category, RepoError> {
        check_len("name", name, CATEGORY_NAME_MAX_LEN)?;

        let mut guard = self.store.write().await;
        let tables = &mut *guard;
        if !tables.categories.contains_key(&id) {
            return Err(RepoError::NotFound { entity: "category" });
        }
        ensure_name_available(tables, name, Some(id))?;

        let category = tables
            .categories
            .get_mut(&id)
            .ok_or(RepoError::NotFound { entity: "category" })?;
        category.name = name.to_owned();
        Ok(category.clone())
    }

    async fn delete(&self, id: i64) -> Result<(), RepoError> {
        let mut guard = self.store.write().await;
        let tables = &mut *guard;

        if tables.categories.remove(&id).is_none() {
            return Err(RepoError::NotFound { entity: "category" });
        }
        // ON DELETE SET NULL: sản phẩm vẫn còn, chỉ mất phân loại.
        for product in tables.products.values_mut() {
            if product.category_id == Some(id) {
                product.category_id = None;
            }
        }
        // ON DELETE CASCADE: chính sách riêng của danh mục bị xóa theo.
        tables
            .policies
            .retain(|_, policy| policy.category_id != Some(id));
        Ok(())
    }
}
