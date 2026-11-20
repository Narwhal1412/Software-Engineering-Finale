//! Repository mock cho `cart_items`.

use async_trait::async_trait;

use super::constraints::check_quantity;
use super::store::{user_cart_range, MockStore};
use crate::models::cart::{CartItem, CartLine};
use crate::repositories::{CartRepository, RepoError};

/// Implementation in-memory của [`CartRepository`].
pub struct InMemoryCartRepository {
    store: MockStore,
}

impl InMemoryCartRepository {
    /// Tạo repository dùng chung `store` với các repository mock khác.
    pub fn new(store: MockStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl CartRepository for InMemoryCartRepository {
    async fn list_with_products(&self, user_id: i64) -> Result<Vec<CartLine>, RepoError> {
        let tables = self.store.read().await;
        let lines = tables
            .cart_items
            .range(user_cart_range(user_id))
            .filter_map(|(_, item)| {
                tables
                    .products
                    .get(&item.product_id)
                    .map(|product| CartLine {
                        product_id: item.product_id,
                        quantity: item.quantity,
                        product: product.clone(),
                    })
            })
            .collect();
        Ok(lines)
    }

    async fn add_item(
        &self,
        user_id: i64,
        product_id: i64,
        quantity: i32,
    ) -> Result<CartItem, RepoError> {
        check_quantity(quantity)?;

        let mut guard = self.store.write().await;
        let tables = &mut *guard;
        if !tables.users.contains_key(&user_id) {
            return Err(RepoError::InvalidReference { field: "user_id" });
        }
        if !tables.products.contains_key(&product_id) {
            return Err(RepoError::InvalidReference {
                field: "product_id",
            });
        }

        let item = tables
            .cart_items
            .entry((user_id, product_id))
            .and_modify(|existing| {
                existing.quantity = existing.quantity.saturating_add(quantity);
            })
            .or_insert(CartItem {
                user_id,
                product_id,
                quantity,
            });
        Ok(item.clone())
    }

    async fn set_quantity(
        &self,
        user_id: i64,
        product_id: i64,
        quantity: i32,
    ) -> Result<CartItem, RepoError> {
        check_quantity(quantity)?;

        let mut tables = self.store.write().await;
        let item = tables
            .cart_items
            .get_mut(&(user_id, product_id))
            .ok_or(RepoError::NotFound {
                entity: "cart item",
            })?;
        item.quantity = quantity;
        Ok(item.clone())
    }

    async fn remove_item(&self, user_id: i64, product_id: i64) -> Result<(), RepoError> {
        self.store
            .write()
            .await
            .cart_items
            .remove(&(user_id, product_id));
        Ok(())
    }

    async fn clear(&self, user_id: i64) -> Result<(), RepoError> {
        self.store
            .write()
            .await
            .cart_items
            .retain(|&(owner, _), _| owner != user_id);
        Ok(())
    }
}
