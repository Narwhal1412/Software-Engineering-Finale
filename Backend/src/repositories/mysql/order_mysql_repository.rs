//! Repository MariaDB cho `orders` và `order_items` (đặc tả mục 5.5).

use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::mysql::MySqlRow;
use sqlx::{MySql, MySqlPool, Row};

use super::error::{decode_enum, last_insert_id, lost_row, map_sqlx_error, read_utc};
use crate::models::enums::{OrderStatus, PaymentMethod};
use crate::models::order::{Order, OrderDraft, OrderItem, OrderWithEmail};
use crate::repositories::{OrderRepository, RepoError};

/// Cột của `orders`. Enum đọc qua CAST thành chuỗi để decode ổn định.
const ORDER_SELECT: &str = "SELECT id, user_id, total, CAST(status AS CHAR) AS status, \
     CAST(payment_method AS CHAR) AS payment_method, full_name, phone, address, created_at \
     FROM orders";

/// Đọc một dòng `orders` (chưa có `items`).
fn order_from_row(row: &MySqlRow) -> Result<Order, RepoError> {
    let status: String = row.try_get("status").map_err(map_sqlx_error)?;
    let payment_method: String = row.try_get("payment_method").map_err(map_sqlx_error)?;
    Ok(Order {
        id: row.try_get("id").map_err(map_sqlx_error)?,
        user_id: row.try_get("user_id").map_err(map_sqlx_error)?,
        items: Vec::new(),
        total: row.try_get("total").map_err(map_sqlx_error)?,
        status: decode_enum::<OrderStatus>(&status)?,
        payment_method: decode_enum::<PaymentMethod>(&payment_method)?,
        full_name: row.try_get("full_name").map_err(map_sqlx_error)?,
        phone: row.try_get("phone").map_err(map_sqlx_error)?,
        address: row.try_get("address").map_err(map_sqlx_error)?,
        created_at: read_utc(row, "created_at")?,
    })
}

fn item_from_row(row: &MySqlRow) -> Result<OrderItem, RepoError> {
    Ok(OrderItem {
        id: row.try_get("id").map_err(map_sqlx_error)?,
        order_id: row.try_get("order_id").map_err(map_sqlx_error)?,
        product_id: row.try_get("product_id").map_err(map_sqlx_error)?,
        name: row.try_get("name").map_err(map_sqlx_error)?,
        price: row.try_get("price").map_err(map_sqlx_error)?,
        quantity: row.try_get("quantity").map_err(map_sqlx_error)?,
    })
}

/// Nạp `items` cho cả danh sách đơn bằng MỘT truy vấn `IN (...)` (tránh N+1), sắp theo `order_items.id`.
async fn attach_items(pool: &MySqlPool, orders: &mut [Order]) -> Result<(), RepoError> {
    if orders.is_empty() {
        return Ok(());
    }
    let placeholders = vec!["?"; orders.len()].join(", ");
    let sql = format!(
        "SELECT id, order_id, product_id, name, price, quantity FROM order_items \
         WHERE order_id IN ({placeholders}) ORDER BY id"
    );
    let mut query = sqlx::query(&sql);
    for order in orders.iter() {
        query = query.bind(order.id);
    }
    let rows = query.fetch_all(pool).await.map_err(map_sqlx_error)?;

    let mut by_order: HashMap<i64, Vec<OrderItem>> = HashMap::new();
    for row in &rows {
        let item = item_from_row(row)?;
        by_order.entry(item.order_id).or_default().push(item);
    }
    for order in orders.iter_mut() {
        order.items = by_order.remove(&order.id).unwrap_or_default();
    }
    Ok(())
}

/// Implementation MariaDB của [`OrderRepository`].
pub struct MySqlOrderRepository {
    pool: MySqlPool,
}

impl MySqlOrderRepository {
    /// Tạo repository dùng chung pool với các repository khác.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Tìm một đơn kèm items (dùng cả để đọc lại sau khi ghi).
    async fn fetch_by_id(&self, id: i64) -> Result<Option<Order>, RepoError> {
        let sql = format!("{ORDER_SELECT} WHERE id = ?");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        let Some(row) = row else {
            return Ok(None);
        };
        let mut orders = vec![order_from_row(&row)?];
        attach_items(&self.pool, &mut orders).await?;
        Ok(orders.pop())
    }
}

#[async_trait]
impl OrderRepository for MySqlOrderRepository {
    async fn create_from_cart(&self, user_id: i64, draft: OrderDraft) -> Result<Order, RepoError> {
        // Mọi bước dưới đây nằm trong MỘT transaction: lỗi ở bất kỳ bước nào thì
        // `tx` bị hủy (ROLLBACK), không có đơn nào được tạo và giỏ hàng vẫn còn.
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        // Khớp thứ tự lỗi của mock: người dùng không tồn tại báo trước.
        let user_exists = sqlx::query_scalar::<MySql, i64>("SELECT id FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        if user_exists.is_none() {
            return Err(RepoError::InvalidReference { field: "user_id" });
        }

        // 1. Đọc giỏ cùng tên/giá/tồn kho HIỆN HÀNH của sản phẩm (đây là snapshot).
        //    FOR UPDATE khóa cả dòng giỏ lẫn dòng sản phẩm nên hai đơn đặt cùng lúc không trừ kho trùng.
        let lines = sqlx::query(
            "SELECT ci.product_id, ci.quantity, p.name, p.price, p.stock \
             FROM cart_items ci JOIN products p ON p.id = ci.product_id \
             WHERE ci.user_id = ? ORDER BY ci.product_id FOR UPDATE",
        )
        .bind(user_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;
        if lines.is_empty() {
            return Err(RepoError::Constraint("cart is empty".to_owned()));
        }

        // 2. Kiểm tra và trừ tồn kho, tính total = Σ price × quantity, gom các dòng hàng cần ghi.
        //    Thiếu hàng ở bất kỳ sản phẩm nào -> trả lỗi, `tx` bị hủy nên kho không bị trừ.
        let mut snapshot: Vec<(i64, String, i64, i32)> = Vec::with_capacity(lines.len());
        let mut total: i64 = 0;
        for line in &lines {
            let product_id: i64 = line.try_get("product_id").map_err(map_sqlx_error)?;
            let quantity: i32 = line.try_get("quantity").map_err(map_sqlx_error)?;
            let name: String = line.try_get("name").map_err(map_sqlx_error)?;
            let price: i64 = line.try_get("price").map_err(map_sqlx_error)?;
            let stock: i32 = line.try_get("stock").map_err(map_sqlx_error)?;
            if stock < quantity {
                return Err(RepoError::OutOfStock {
                    product: name,
                    available: stock,
                });
            }
            sqlx::query("UPDATE products SET stock = stock - ? WHERE id = ?")
                .bind(quantity)
                .bind(product_id)
                .execute(&mut *tx)
                .await
                .map_err(map_sqlx_error)?;
            total = total.saturating_add(price.saturating_mul(i64::from(quantity)));
            snapshot.push((product_id, name, price, quantity));
        }

        // 3. Tạo đơn, trạng thái luôn là pending.
        let inserted = sqlx::query(
            "INSERT INTO orders (user_id, total, status, payment_method, full_name, phone, address, created_at) \
             VALUES (?, ?, 'pending', ?, ?, ?, ?, UTC_TIMESTAMP(6))",
        )
        .bind(user_id)
        .bind(total)
        .bind(draft.payment_method.as_str())
        .bind(draft.full_name)
        .bind(draft.phone)
        .bind(draft.address)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;
        let order_id = last_insert_id(&inserted)?;

        // 4. Mỗi dòng giỏ thành một dòng order_items (snapshot tên và giá).
        for (product_id, name, price, quantity) in snapshot {
            sqlx::query(
                "INSERT INTO order_items (order_id, product_id, name, price, quantity) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(order_id)
            .bind(product_id)
            .bind(name)
            .bind(price)
            .bind(quantity)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        // 5. Xóa giỏ rồi commit.
        sqlx::query("DELETE FROM cart_items WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        tx.commit().await.map_err(map_sqlx_error)?;

        self.fetch_by_id(order_id)
            .await?
            .ok_or_else(|| lost_row("đơn hàng"))
    }

    async fn list_by_user(&self, user_id: i64) -> Result<Vec<Order>, RepoError> {
        let sql = format!("{ORDER_SELECT} WHERE user_id = ? ORDER BY created_at DESC, id DESC");
        let rows = sqlx::query(&sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        let mut orders = rows
            .iter()
            .map(order_from_row)
            .collect::<Result<Vec<_>, _>>()?;
        attach_items(&self.pool, &mut orders).await?;
        Ok(orders)
    }

    async fn list_all_with_user_email(&self) -> Result<Vec<OrderWithEmail>, RepoError> {
        // LEFT JOIN: đơn của tài khoản đã mất vẫn hiện, với email = None.
        let rows = sqlx::query(
            "SELECT o.id, o.user_id, o.total, CAST(o.status AS CHAR) AS status, \
             CAST(o.payment_method AS CHAR) AS payment_method, o.full_name, o.phone, o.address, \
             o.created_at, u.email AS user_email \
             FROM orders o LEFT JOIN users u ON u.id = o.user_id \
             ORDER BY o.created_at DESC, o.id DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut orders = Vec::with_capacity(rows.len());
        let mut emails = Vec::with_capacity(rows.len());
        for row in &rows {
            orders.push(order_from_row(row)?);
            emails.push(
                row.try_get::<Option<String>, _>("user_email")
                    .map_err(map_sqlx_error)?,
            );
        }
        attach_items(&self.pool, &mut orders).await?;

        Ok(orders
            .into_iter()
            .zip(emails)
            .map(|(order, user_email)| OrderWithEmail { order, user_email })
            .collect())
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Order>, RepoError> {
        self.fetch_by_id(id).await
    }

    async fn update_status(&self, id: i64, status: OrderStatus) -> Result<Order, RepoError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;

        // Khóa dòng đơn để biết trạng thái hiện tại và tránh hoàn kho hai lần khi hủy đồng thời.
        let current = sqlx::query_scalar::<MySql, String>(
            "SELECT CAST(status AS CHAR) FROM orders WHERE id = ? FOR UPDATE",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(RepoError::NotFound { entity: "order" })?;
        let current = decode_enum::<OrderStatus>(&current)?;

        // Hủy đơn (từ trạng thái khác `cancelled`) thì hoàn lại tồn kho.
        if status == OrderStatus::Cancelled && current != OrderStatus::Cancelled {
            sqlx::query(
                "UPDATE products p JOIN order_items oi ON oi.product_id = p.id \
                 SET p.stock = p.stock + oi.quantity WHERE oi.order_id = ?",
            )
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        sqlx::query("UPDATE orders SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(map_sqlx_error)?;
        tx.commit().await.map_err(map_sqlx_error)?;

        self.fetch_by_id(id)
            .await?
            .ok_or(RepoError::NotFound { entity: "order" })
    }

    async fn count(&self) -> Result<u64, RepoError> {
        let total = sqlx::query_scalar::<MySql, i64>("SELECT COUNT(*) FROM orders")
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(u64::try_from(total).unwrap_or(0))
    }

    async fn revenue_excluding_cancelled(&self) -> Result<i64, RepoError> {
        sqlx::query_scalar::<MySql, i64>(
            "SELECT CAST(COALESCE(SUM(total), 0) AS SIGNED) FROM orders WHERE status <> 'cancelled'",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)
    }
}
