//! Implementation mock in-memory của tầng DAL, dùng khi chưa có database.
//!
//! Tất cả repository mock chia sẻ một [`MockStore`] (một `RwLock` duy nhất cho
//! mọi "bảng") để các ràng buộc xuyên bảng như khóa ngoại, cascade và
//! "không xóa bản ghi đang bị tham chiếu" được kiểm tra nguyên tử, giống database thật.

mod cart_mock_repository;
mod category_mock_repository;
mod constraints;
mod order_mock_repository;
mod policy_mock_repository;
mod product_mock_repository;
mod seed;
mod store;
#[cfg(test)]
mod tests;
mod user_mock_repository;

pub use cart_mock_repository::InMemoryCartRepository;
pub use category_mock_repository::InMemoryCategoryRepository;
pub use order_mock_repository::InMemoryOrderRepository;
pub use policy_mock_repository::InMemoryPolicyRepository;
pub use product_mock_repository::InMemoryProductRepository;
pub use seed::{SEED_ADMIN_EMAIL, SEED_CUSTOMER_EMAILS, SEED_PASSWORD};
pub use store::MockStore;
pub use user_mock_repository::InMemoryUserRepository;
