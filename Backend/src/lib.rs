//! Backend của web quản lý cửa hàng máy tính.
//!
//! Kiến trúc layered: `controllers` (Presentation) -> `services` (Business Logic)
//! -> `repositories` (Data Access). Tách thành thư viện + binary để mã nguồn có
//! thể được import trong `tests/` và để trình biên dịch không coi mọi item `pub`
//! chưa dùng là dead code khi các tầng phía trên chưa được viết xong.

pub mod app;
pub mod app_state;
pub mod config;
pub mod controllers;
pub mod db;
pub mod dtos;
pub mod error;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod services;
