//! Thành phần dùng chung cho tầng Presentation: xác thực, CORS, logging,
//! cùng các tiện ích bảo mật (băm mật khẩu, JWT) mà xác thực cần.

pub mod auth;
pub mod cors;
pub mod logging;
pub mod password;
pub mod token;
