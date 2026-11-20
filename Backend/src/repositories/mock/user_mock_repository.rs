//! Repository mock cho `users`.

use async_trait::async_trait;
use chrono::Utc;

use super::constraints::check_len;
use super::store::{to_count, MockStore};
use crate::models::limits::{ADDRESS_MAX_LEN, DISPLAY_NAME_MAX_LEN, EMAIL_MAX_LEN, PHONE_MAX_LEN};
use crate::models::user::{NewUser, ProfileUpdate, User};
use crate::repositories::{RepoError, UserRepository};

/// Implementation in-memory của [`UserRepository`].
pub struct InMemoryUserRepository {
    store: MockStore,
}

impl InMemoryUserRepository {
    /// Tạo repository dùng chung `store` với các repository mock khác.
    pub fn new(store: MockStore) -> Self {
        Self { store }
    }
}

/// Kiểm tra độ dài các cột dạng chuỗi của `users`.
fn check_user_lengths(
    display_name: &str,
    phone: &str,
    address: &str,
) -> Result<(), RepoError> {
    check_len("display_name", display_name, DISPLAY_NAME_MAX_LEN)?;
    check_len("phone", phone, PHONE_MAX_LEN)?;
    check_len("address", address, ADDRESS_MAX_LEN)
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, new_user: NewUser) -> Result<User, RepoError> {
        check_len("email", &new_user.email, EMAIL_MAX_LEN)?;
        check_user_lengths(&new_user.display_name, &new_user.phone, &new_user.address)?;

        let mut guard = self.store.write().await;
        let tables = &mut *guard;

        // Unique không phân biệt hoa thường, giống collation utf8mb4_unicode_ci của DB.
        let email_key = new_user.email.to_lowercase();
        if tables
            .users
            .values()
            .any(|user| user.email.to_lowercase() == email_key)
        {
            return Err(RepoError::Duplicate { field: "email" });
        }

        let user = User {
            id: tables.sequences.user.next(),
            email: new_user.email,
            display_name: new_user.display_name,
            password_hash: new_user.password_hash,
            phone: new_user.phone,
            address: new_user.address,
            role: new_user.role,
            created_at: Utc::now(),
        };
        tables.users.insert(user.id, user.clone());
        Ok(user)
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<User>, RepoError> {
        Ok(self.store.read().await.users.get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepoError> {
        let email_key = email.to_lowercase();
        let tables = self.store.read().await;
        Ok(tables
            .users
            .values()
            .find(|user| user.email.to_lowercase() == email_key)
            .cloned())
    }

    async fn list(&self) -> Result<Vec<User>, RepoError> {
        Ok(self.store.read().await.users.values().cloned().collect())
    }

    async fn update_profile(&self, id: i64, update: ProfileUpdate) -> Result<User, RepoError> {
        check_user_lengths(&update.display_name, &update.phone, &update.address)?;

        let mut tables = self.store.write().await;
        let user = tables
            .users
            .get_mut(&id)
            .ok_or(RepoError::NotFound { entity: "user" })?;
        user.display_name = update.display_name;
        user.phone = update.phone;
        user.address = update.address;
        Ok(user.clone())
    }

    async fn delete(&self, id: i64) -> Result<(), RepoError> {
        let mut guard = self.store.write().await;
        let tables = &mut *guard;

        if !tables.users.contains_key(&id) {
            return Err(RepoError::NotFound { entity: "user" });
        }
        // ON DELETE RESTRICT: giữ đơn hàng để không làm sai lệch doanh thu.
        if tables.orders.values().any(|order| order.user_id == id) {
            return Err(RepoError::InUse {
                entity: "user",
                referenced_by: "order",
            });
        }

        tables.users.remove(&id);
        // ON DELETE CASCADE: giỏ hàng chỉ có ý nghĩa khi còn chủ.
        tables.cart_items.retain(|&(owner, _), _| owner != id);
        Ok(())
    }

    async fn count(&self) -> Result<u64, RepoError> {
        Ok(to_count(self.store.read().await.users.len()))
    }
}
