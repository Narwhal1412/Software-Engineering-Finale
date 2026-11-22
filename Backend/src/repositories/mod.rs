//! TẦNG DAL (Data Access Layer): trait repository và các implementation.
//!
//! Services chỉ phụ thuộc vào các trait ở đây. Có hai implementation:
//! - `mysql/`: MariaDB thật, dùng khi chạy chương trình (`app_state::build_repositories`);
//! - `mock/`: in-memory có dữ liệu mẫu, dùng cho test (`app_state::build_mock_repositories`).

pub mod cart_repository;
pub mod category_repository;
pub mod mock;
pub mod mysql;
pub mod order_repository;
pub mod policy_repository;
pub mod product_repository;
pub mod repo_error;
pub mod user_repository;

pub use cart_repository::CartRepository;
pub use category_repository::CategoryRepository;
pub use order_repository::OrderRepository;
pub use policy_repository::PolicyRepository;
pub use product_repository::ProductRepository;
pub use repo_error::RepoError;
pub use user_repository::UserRepository;
