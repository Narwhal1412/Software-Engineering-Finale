//! Entity khớp với các bảng trong database. Chúng KHÔNG derive `Serialize`
//! (trừ các enum) để không vô tình trả `password_hash` ra ngoài; DTO ở tầng
//! controller chịu trách nhiệm định dạng JSON theo API.md.

pub mod cart;
pub mod category;
pub mod enums;
pub mod limits;
pub mod order;
pub mod policy;
pub mod product;
pub mod user;
