//! Repository mock cho `policies`.

use async_trait::async_trait;

use super::constraints::check_len;
use super::store::{MockStore, Tables};
use crate::models::limits::{POLICY_CONTENT_MAX_LEN, POLICY_ICON_MAX_LEN, POLICY_TITLE_MAX_LEN};
use crate::models::policy::{Policy, PolicyInput};
use crate::repositories::{PolicyRepository, RepoError};

/// Implementation in-memory của [`PolicyRepository`].
pub struct InMemoryPolicyRepository {
    store: MockStore,
}

impl InMemoryPolicyRepository {
    /// Tạo repository dùng chung `store` với các repository mock khác.
    pub fn new(store: MockStore) -> Self {
        Self { store }
    }
}

/// Mô phỏng độ dài cột và khóa ngoại `category_id` của database thật.
fn check_input(tables: &Tables, input: &PolicyInput) -> Result<(), RepoError> {
    check_len("icon", &input.icon, POLICY_ICON_MAX_LEN)?;
    check_len("title", &input.title, POLICY_TITLE_MAX_LEN)?;
    check_len("content", &input.content, POLICY_CONTENT_MAX_LEN)?;
    if let Some(category_id) = input.category_id {
        if !tables.categories.contains_key(&category_id) {
            return Err(RepoError::InvalidReference {
                field: "category_id",
            });
        }
    }
    Ok(())
}

#[async_trait]
impl PolicyRepository for InMemoryPolicyRepository {
    async fn list(&self) -> Result<Vec<Policy>, RepoError> {
        let mut policies: Vec<Policy> = self
            .store
            .read()
            .await
            .policies
            .values()
            .cloned()
            .collect();
        policies.sort_by_key(|policy| (policy.sort_order, policy.id));
        Ok(policies)
    }

    async fn create(&self, input: PolicyInput) -> Result<Policy, RepoError> {
        let mut guard = self.store.write().await;
        let tables = &mut *guard;
        check_input(tables, &input)?;

        let policy = Policy {
            id: tables.sequences.policy.next(),
            icon: input.icon,
            title: input.title,
            content: input.content,
            category_id: input.category_id,
            sort_order: input.sort_order,
        };
        tables.policies.insert(policy.id, policy.clone());
        Ok(policy)
    }

    async fn update(&self, id: i64, input: PolicyInput) -> Result<Policy, RepoError> {
        let mut guard = self.store.write().await;
        let tables = &mut *guard;
        if !tables.policies.contains_key(&id) {
            return Err(RepoError::NotFound { entity: "policy" });
        }
        check_input(tables, &input)?;

        let policy = tables
            .policies
            .get_mut(&id)
            .ok_or(RepoError::NotFound { entity: "policy" })?;
        policy.icon = input.icon;
        policy.title = input.title;
        policy.content = input.content;
        policy.category_id = input.category_id;
        policy.sort_order = input.sort_order;
        Ok(policy.clone())
    }

    async fn delete(&self, id: i64) -> Result<(), RepoError> {
        let mut guard = self.store.write().await;
        if guard.policies.remove(&id).is_none() {
            return Err(RepoError::NotFound { entity: "policy" });
        }
        Ok(())
    }
}
