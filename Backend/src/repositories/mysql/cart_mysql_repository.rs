//! Repository MariaDB cho bảng `cart_items` (đặc tả mục 5.4).

use async_trait::async_trait;
use sqlx::mysql::MySqlRow;
use sqlx::{MySqlPool, Row};

use super::error::{lost_row, map_sqlx_error};
use super::product_mysql_repository::product_from_row;
use crate::models::cart::{CartItem, CartLine};
use crate::repositories::{CartRepository, RepoError};

/// Giỏ kèm sản phẩm. Tên cột không trùng nhau nên đọc được bằng `try_get`.
const CART_LINES: &str = "SELECT ci.product_id, ci.quantity, \
     p.id, p.name, p.specs, p.price, p.stock, p.category_id, p.image_url, p.pinned_at, p.created_at \
     FROM cart_items ci JOIN products p ON p.id = ci.product_id \
     WHERE ci.user_id = ? ORDER BY ci.product_id";

/// Số lượng phải tối thiểu 1 (mô phỏng `CHECK (quantity >= 1)` như mock).
fn check_quantity(quantity: i32) -> Result<(), RepoError> {
    if quantity < 1 {
        Err(RepoError::Constraint("quantity must be >= 1".to_owned()))
    } else {
        Ok(())
    }
}

fn cart_item_from_row(row: &MySqlRow) -> Result<CartItem, RepoError> {
    Ok(CartItem {
        user_id: row.try_get("user_id").map_err(map_sqlx_error)?,
        product_id: row.try_get("product_id").map_err(map_sqlx_error)?,
        quantity: row.try_get("quantity").map_err(map_sqlx_error)?,
    })
}

fn cart_line_from_row(row: &MySqlRow) -> Result<CartLine, RepoError> {
    Ok(CartLine {
        product_id: row.try_get("product_id").map_err(map_sqlx_error)?,
        quantity: row.try_get("quantity").map_err(map_sqlx_error)?,
        product: product_from_row(row)?,
    })
}

/// Implementation MariaDB của [`CartRepository`].
pub struct MySqlCartRepository {
    pool: MySqlPool,
}

impl MySqlCartRepository {
    /// Tạo repository dùng chung pool với các repository khác.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Đọc một dòng giỏ (dùng để trả kết quả sau khi ghi).
    async fn fetch_item(
        &self,
        user_id: i64,
        product_id: i64,
    ) -> Result<Option<CartItem>, RepoError> {
        let row = sqlx::query(
            "SELECT user_id, product_id, quantity FROM cart_items WHERE user_id = ? AND product_id = ?",
        )
        .bind(user_id)
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        row.as_ref().map(cart_item_from_row).transpose()
    }
}

#[async_trait]
impl CartRepository for MySqlCartRepository {
    async fn list_with_products(&self, user_id: i64) -> Result<Vec<CartLine>, RepoError> {
        let rows = sqlx::query(CART_LINES)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        rows.iter().map(cart_line_from_row).collect()
    }

    async fn add_item(
        &self,
        user_id: i64,
        product_id: i64,
        quantity: i32,
    ) -> Result<CartItem, RepoError> {
        check_quantity(quantity)?;
        // Đã có dòng thì cộng dồn số lượng (khóa chính ghép (user_id, product_id)).
        sqlx::query(
            "INSERT INTO cart_items (user_id, product_id, quantity) VALUES (?, ?, ?) \
             ON DUPLICATE KEY UPDATE quantity = quantity + VALUES(quantity)",
        )
        .bind(user_id)
        .bind(product_id)
        .bind(quantity)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?; // FK user/product -> InvalidReference

        self.fetch_item(user_id, product_id)
            .await?
            .ok_or_else(|| lost_row("dòng giỏ hàng"))
    }

    async fn set_quantity(
        &self,
        user_id: i64,
        product_id: i64,
        quantity: i32,
    ) -> Result<CartItem, RepoError> {
        check_quantity(quantity)?;
        sqlx::query("UPDATE cart_items SET quantity = ? WHERE user_id = ? AND product_id = ?")
            .bind(quantity)
            .bind(user_id)
            .bind(product_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        self.fetch_item(user_id, product_id)
            .await?
            .ok_or(RepoError::NotFound {
                entity: "cart item",
            })
    }

    async fn remove_item(&self, user_id: i64, product_id: i64) -> Result<(), RepoError> {
        // Idempotent: không có dòng thì DELETE không làm gì, vẫn trả Ok.
        sqlx::query("DELETE FROM cart_items WHERE user_id = ? AND product_id = ?")
            .bind(user_id)
            .bind(product_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn clear(&self, user_id: i64) -> Result<(), RepoError> {
        sqlx::query("DELETE FROM cart_items WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }
}
