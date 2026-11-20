//! Kho dữ liệu dùng chung của các repository mock.

use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use std::sync::Arc;

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::seed::seed_tables;
use crate::models::cart::CartItem;
use crate::models::category::Category;
use crate::models::order::Order;
use crate::models::product::Product;
use crate::models::user::User;
use crate::repositories::RepoError;

/// Bộ sinh id tự tăng, mô phỏng `AUTO_INCREMENT`; id đã cấp không bao giờ được dùng lại.
#[derive(Debug, Default)]
pub(super) struct Sequence(i64);

impl Sequence {
    /// Cấp id kế tiếp (bắt đầu từ 1).
    pub(super) fn next(&mut self) -> i64 {
        self.0 += 1;
        self.0
    }
}

/// Bộ sinh id cho từng bảng.
#[derive(Debug, Default)]
pub(super) struct Sequences {
    pub(super) user: Sequence,
    pub(super) category: Sequence,
    pub(super) product: Sequence,
    pub(super) order: Sequence,
    pub(super) order_item: Sequence,
}

/// Các "bảng". `BTreeMap` giữ thứ tự khóa tăng dần nên kết quả ổn định giữa các lần chạy.
#[derive(Debug, Default)]
pub(super) struct Tables {
    pub(super) users: BTreeMap<i64, User>,
    pub(super) categories: BTreeMap<i64, Category>,
    pub(super) products: BTreeMap<i64, Product>,
    /// Khóa là `(user_id, product_id)`.
    pub(super) cart_items: BTreeMap<(i64, i64), CartItem>,
    /// Mỗi `Order` chứa luôn các `OrderItem` của nó.
    pub(super) orders: BTreeMap<i64, Order>,
    pub(super) sequences: Sequences,
}

/// Kho dữ liệu in-memory an toàn đa luồng; nhân bản (`clone`) chỉ nhân bản con trỏ.
#[derive(Clone, Default)]
pub struct MockStore {
    tables: Arc<RwLock<Tables>>,
}

impl MockStore {
    /// Kho rỗng, dùng cho test.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Kho đã có dữ liệu mẫu để demo mọi endpoint.
    pub fn seeded() -> Result<Self, RepoError> {
        Ok(Self {
            tables: Arc::new(RwLock::new(seed_tables()?)),
        })
    }

    /// Khóa đọc toàn bộ bảng.
    pub(super) async fn read(&self) -> RwLockReadGuard<'_, Tables> {
        self.tables.read().await
    }

    /// Khóa ghi toàn bộ bảng; mỗi thao tác ghi giữ khóa trọn vẹn nên là một "giao dịch".
    pub(super) async fn write(&self) -> RwLockWriteGuard<'_, Tables> {
        self.tables.write().await
    }
}

/// Khoảng khóa `(user_id, *)` để duyệt các dòng giỏ hàng của một người dùng.
pub(super) fn user_cart_range(user_id: i64) -> RangeInclusive<(i64, i64)> {
    (user_id, i64::MIN)..=(user_id, i64::MAX)
}

/// Đổi độ dài collection sang `u64` mà không dùng phép ép kiểu có thể mất dữ liệu.
pub(super) fn to_count(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
}
