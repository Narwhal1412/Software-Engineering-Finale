//! Implementation MariaDB của 5 trait repository (đặc tả: docs/DATABASE_SPEC.md mục 5 và 7.4).
//!
//! Mỗi struct giữ một `MySqlPool` (handle dùng chung, rẻ khi clone). Lỗi sqlx được đổi sang
//! `RepoError` ở `error.rs`. Services và controllers không biết tới file này.

mod cart_mysql_repository;
mod category_mysql_repository;
mod error;
mod order_mysql_repository;
mod product_mysql_repository;
mod user_mysql_repository;

pub use cart_mysql_repository::MySqlCartRepository;
pub use category_mysql_repository::MySqlCategoryRepository;
pub use order_mysql_repository::MySqlOrderRepository;
pub use product_mysql_repository::MySqlProductRepository;
pub use user_mysql_repository::MySqlUserRepository;
