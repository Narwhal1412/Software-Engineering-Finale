//! DTO: struct request/response khớp từng field của docs/API.md (JSON dùng camelCase).
//! Tách khỏi `models` để đổi database không làm đổi hợp đồng API, và để
//! không bao giờ vô tình trả `password_hash` ra ngoài.

pub mod auth_dto;
pub mod cart_dto;
pub mod category_dto;
pub mod order_dto;
pub mod policy_dto;
pub mod product_dto;
pub mod stats_dto;
pub mod upload_dto;
pub mod user_dto;
