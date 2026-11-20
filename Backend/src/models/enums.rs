//! Các enum lưu dưới dạng chuỗi (cột `ENUM` trong database, chuỗi thường trong JSON).

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Lỗi khi chuỗi không tương ứng với giá trị enum nào.
#[derive(Debug, Error)]
#[error("giá trị {kind} không hợp lệ: {value}")]
pub struct ParseEnumError {
    /// Tên kiểu enum đang parse.
    pub kind: &'static str,
    /// Chuỗi bị từ chối.
    pub value: String,
}

impl ParseEnumError {
    fn new(kind: &'static str, value: &str) -> Self {
        Self {
            kind,
            value: value.to_owned(),
        }
    }
}

/// Vai trò của tài khoản.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Khách hàng thông thường.
    User,
    /// Quản trị viên.
    Admin,
}

impl Role {
    /// Chuỗi lưu trong database và JSON.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Admin => "admin",
        }
    }
}

impl FromStr for Role {
    type Err = ParseEnumError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "user" => Ok(Self::User),
            "admin" => Ok(Self::Admin),
            other => Err(ParseEnumError::new("role", other)),
        }
    }
}

/// Trạng thái của đơn hàng.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// Chờ xác nhận (trạng thái khi mới tạo).
    Pending,
    /// Đã xác nhận.
    Confirmed,
    /// Đang giao.
    Shipped,
    /// Hoàn tất.
    Completed,
    /// Đã hủy (không tính vào doanh thu).
    Cancelled,
}

impl OrderStatus {
    /// Chuỗi lưu trong database và JSON.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Confirmed => "confirmed",
            Self::Shipped => "shipped",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl FromStr for OrderStatus {
    type Err = ParseEnumError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "pending" => Ok(Self::Pending),
            "confirmed" => Ok(Self::Confirmed),
            "shipped" => Ok(Self::Shipped),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(ParseEnumError::new("order status", other)),
        }
    }
}

/// Phương thức thanh toán.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    /// Thanh toán khi nhận hàng.
    Cod,
    /// Chuyển khoản ngân hàng.
    Bank,
}

impl PaymentMethod {
    /// Chuỗi lưu trong database và JSON.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cod => "cod",
            Self::Bank => "bank",
        }
    }
}

impl FromStr for PaymentMethod {
    type Err = ParseEnumError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw {
            "cod" => Ok(Self::Cod),
            "bank" => Ok(Self::Bank),
            other => Err(ParseEnumError::new("payment method", other)),
        }
    }
}
