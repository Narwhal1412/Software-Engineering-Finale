//! TẦNG DAL (Data Access Layer): trait repository và các implementation.
//!
//! Services chỉ phụ thuộc vào các trait ở đây. Hiện chỉ có implementation mock
//! in-memory trong `mock/`; implementation database thật sẽ được thêm cạnh đó
//! và đổi ở đúng một chỗ: `app_state::build_repositories`.

pub mod cart_repository;
pub mod category_repository;
pub mod mock;
pub mod order_repository;
pub mod product_repository;
pub mod repo_error;
pub mod user_repository;

pub use cart_repository::CartRepository;
pub use category_repository::CategoryRepository;
pub use order_repository::OrderRepository;
pub use product_repository::ProductRepository;
pub use repo_error::RepoError;
pub use user_repository::UserRepository;
