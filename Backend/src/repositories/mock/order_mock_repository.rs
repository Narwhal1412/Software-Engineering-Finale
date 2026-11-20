//! Repository mock cho `orders` và `order_items`.

use async_trait::async_trait;
use chrono::Utc;

use super::constraints::check_len;
use super::store::{to_count, user_cart_range, MockStore};
use crate::models::cart::CartItem;
use crate::models::enums::OrderStatus;
use crate::models::limits::{ADDRESS_MAX_LEN, FULL_NAME_MAX_LEN, PHONE_MAX_LEN};
use crate::models::order::{Order, OrderDraft, OrderItem, OrderWithEmail};
use crate::repositories::{OrderRepository, RepoError};

/// Implementation in-memory của [`OrderRepository`].
pub struct InMemoryOrderRepository {
    store: MockStore,
}

impl InMemoryOrderRepository {
    /// Tạo repository dùng chung `store` với các repository mock khác.
    pub fn new(store: MockStore) -> Self {
        Self { store }
    }
}

/// Sắp xếp mới nhất trước, `id` giảm dần để ổn định khi cùng thời điểm.
fn sort_newest_first(orders: &mut [Order]) {
    orders.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
}

#[async_trait]
impl OrderRepository for InMemoryOrderRepository {
    async fn create_from_cart(&self, user_id: i64, draft: OrderDraft) -> Result<Order, RepoError> {
        check_len("full_name", &draft.full_name, FULL_NAME_MAX_LEN)?;
        check_len("phone", &draft.phone, PHONE_MAX_LEN)?;
        check_len("address", &draft.address, ADDRESS_MAX_LEN)?;

        // Giữ khóa ghi suốt hàm để việc đọc giỏ, tạo đơn và xóa giỏ là một giao dịch.
        let mut guard = self.store.write().await;
        let tables = &mut *guard;

        if !tables.users.contains_key(&user_id) {
            return Err(RepoError::InvalidReference { field: "user_id" });
        }
        let lines: Vec<CartItem> = tables
            .cart_items
            .range(user_cart_range(user_id))
            .map(|(_, item)| item.clone())
            .collect();
        if lines.is_empty() {
            return Err(RepoError::Constraint("cart is empty".to_owned()));
        }

        let order_id = tables.sequences.order.next();
        let mut items = Vec::with_capacity(lines.len());
        let mut total: i64 = 0;
        for line in &lines {
            let product = tables
                .products
                .get(&line.product_id)
                .ok_or(RepoError::InvalidReference {
                    field: "product_id",
                })?;
            total = total.saturating_add(product.price.saturating_mul(i64::from(line.quantity)));
            items.push(OrderItem {
                id: tables.sequences.order_item.next(),
                order_id,
                product_id: product.id,
                name: product.name.clone(),
                price: product.price,
                quantity: line.quantity,
            });
        }

        let order = Order {
            id: order_id,
            user_id,
            items,
            total,
            status: OrderStatus::Pending,
            payment_method: draft.payment_method,
            full_name: draft.full_name,
            phone: draft.phone,
            address: draft.address,
            created_at: Utc::now(),
        };
        tables.cart_items.retain(|&(owner, _), _| owner != user_id);
        tables.orders.insert(order.id, order.clone());
        Ok(order)
    }

    async fn list_by_user(&self, user_id: i64) -> Result<Vec<Order>, RepoError> {
        let tables = self.store.read().await;
        let mut orders: Vec<Order> = tables
            .orders
            .values()
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect();
        sort_newest_first(&mut orders);
        Ok(orders)
    }

    async fn list_all_with_user_email(&self) -> Result<Vec<OrderWithEmail>, RepoError> {
        let tables = self.store.read().await;
        let mut orders: Vec<Order> = tables.orders.values().cloned().collect();
        sort_newest_first(&mut orders);
        let rows = orders
            .into_iter()
            .map(|order| {
                let user_email = tables
                    .users
                    .get(&order.user_id)
                    .map(|user| user.email.clone());
                OrderWithEmail { order, user_email }
            })
            .collect();
        Ok(rows)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Order>, RepoError> {
        Ok(self.store.read().await.orders.get(&id).cloned())
    }

    async fn update_status(&self, id: i64, status: OrderStatus) -> Result<Order, RepoError> {
        let mut tables = self.store.write().await;
        let order = tables
            .orders
            .get_mut(&id)
            .ok_or(RepoError::NotFound { entity: "order" })?;
        order.status = status;
        Ok(order.clone())
    }

    async fn count(&self) -> Result<u64, RepoError> {
        Ok(to_count(self.store.read().await.orders.len()))
    }

    async fn revenue_excluding_cancelled(&self) -> Result<i64, RepoError> {
        let tables = self.store.read().await;
        let revenue = tables
            .orders
            .values()
            .filter(|order| order.status != OrderStatus::Cancelled)
            .fold(0_i64, |sum, order| sum.saturating_add(order.total));
        Ok(revenue)
    }
}
