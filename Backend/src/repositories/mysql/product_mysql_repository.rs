//! Repository MariaDB cho bảng `products` (đặc tả mục 5.3).

use async_trait::async_trait;
use chrono::Utc;
use sqlx::mysql::MySqlRow;
use sqlx::{MySql, MySqlPool, Row};

use super::error::{last_insert_id, lost_row, map_sqlx_error, read_opt_utc, read_utc};
use crate::models::product::{Product, ProductInput, ProductQuery, ProductSort};
use crate::repositories::{ProductRepository, RepoError};

/// Danh sách cột của `products`, đúng thứ tự mục 2.3.
const PRODUCT_COLUMNS: &str =
    "id, name, specs, price, stock, category_id, image_url, pinned_at, created_at";

/// Đọc một dòng `products` (các cột có tên đúng như trong bảng).
/// `pub(super)` để `cart_mysql_repository` dùng lại khi đọc sản phẩm trong giỏ.
pub(super) fn product_from_row(row: &MySqlRow) -> Result<Product, RepoError> {
    Ok(Product {
        id: row.try_get("id").map_err(map_sqlx_error)?,
        name: row.try_get("name").map_err(map_sqlx_error)?,
        specs: row.try_get("specs").map_err(map_sqlx_error)?,
        price: row.try_get("price").map_err(map_sqlx_error)?,
        stock: row.try_get("stock").map_err(map_sqlx_error)?,
        category_id: row.try_get("category_id").map_err(map_sqlx_error)?,
        image_url: row.try_get("image_url").map_err(map_sqlx_error)?,
        pinned_at: read_opt_utc(row, "pinned_at")?,
        created_at: read_utc(row, "created_at")?,
    })
}

/// Escape `\`, `%`, `_` trong từ khóa để LIKE coi chúng là ký tự thường.
fn escape_like(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if matches!(ch, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

/// Implementation MariaDB của [`ProductRepository`].
pub struct MySqlProductRepository {
    pool: MySqlPool,
}

impl MySqlProductRepository {
    /// Tạo repository dùng chung pool với các repository khác.
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Tìm theo id (dùng cả để đọc lại sau khi ghi).
    async fn fetch_by_id(&self, id: i64) -> Result<Option<Product>, RepoError> {
        let sql = format!("SELECT {PRODUCT_COLUMNS} FROM products WHERE id = ?");
        let row = sqlx::query(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        row.as_ref().map(product_from_row).transpose()
    }
}

#[async_trait]
impl ProductRepository for MySqlProductRepository {
    async fn search(&self, query: &ProductQuery) -> Result<Vec<Product>, RepoError> {
        let keyword: Option<String> = query
            .q
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(escape_like);

        // ORDER BY ghép từ danh sách trắng cố định, không bao giờ nối chuỗi do người dùng nhập.
        let sort_clause = match query.sort {
            ProductSort::Newest => "created_at DESC, id DESC",
            ProductSort::PriceAsc => "price ASC, id ASC",
            ProductSort::PriceDesc => "price DESC, id ASC",
        };
        // Đã ghim lên đầu (ghim mới nhất trước), rồi mới đến thứ tự `sort`.
        // Tham số theo thứ tự: category_id x2, keyword x3 (kiểm NULL, name, specs).
        let sql = format!(
            r"SELECT {PRODUCT_COLUMNS}
              FROM products
              WHERE (? IS NULL OR category_id = ?)
                AND (? IS NULL
                     OR name  LIKE CONCAT('%', ?, '%') ESCAPE '\\'
                     OR specs LIKE CONCAT('%', ?, '%') ESCAPE '\\')
              ORDER BY (pinned_at IS NULL) ASC, pinned_at DESC, {sort_clause}"
        );

        let rows = sqlx::query(&sql)
            .bind(query.category_id)
            .bind(query.category_id)
            .bind(keyword.clone())
            .bind(keyword.clone())
            .bind(keyword)
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        rows.iter().map(product_from_row).collect()
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Product>, RepoError> {
        self.fetch_by_id(id).await
    }

    async fn create(&self, input: ProductInput) -> Result<Product, RepoError> {
        let result = sqlx::query(
            "INSERT INTO products (name, specs, price, stock, category_id, image_url, pinned_at, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, NULL, UTC_TIMESTAMP(6))",
        )
        .bind(input.name)
        .bind(input.specs)
        .bind(input.price)
        .bind(input.stock)
        .bind(input.category_id)
        .bind(input.image_url)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?; // category_id không tồn tại -> InvalidReference

        let id = last_insert_id(&result)?;
        self.fetch_by_id(id)
            .await?
            .ok_or_else(|| lost_row("sản phẩm"))
    }

    async fn update(&self, id: i64, input: ProductInput) -> Result<Product, RepoError> {
        sqlx::query(
            "UPDATE products SET name = ?, specs = ?, price = ?, stock = ?, category_id = ?, image_url = ? \
             WHERE id = ?",
        )
        .bind(input.name)
        .bind(input.specs)
        .bind(input.price)
        .bind(input.stock)
        .bind(input.category_id)
        .bind(input.image_url)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        self.fetch_by_id(id)
            .await?
            .ok_or(RepoError::NotFound { entity: "product" })
    }

    async fn delete(&self, id: i64) -> Result<(), RepoError> {
        // Giỏ hàng chứa sản phẩm tự xóa (CASCADE); đã có trong đơn thì 1451 -> InUse.
        let result = sqlx::query("DELETE FROM products WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(RepoError::NotFound { entity: "product" });
        }
        Ok(())
    }

    async fn set_pinned(&self, id: i64, pinned: bool) -> Result<Product, RepoError> {
        // Ghim = thời điểm hiện tại (đưa lên đầu); bỏ ghim = NULL.
        let pinned_at = pinned.then(|| Utc::now().naive_utc());
        sqlx::query("UPDATE products SET pinned_at = ? WHERE id = ?")
            .bind(pinned_at)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;

        self.fetch_by_id(id)
            .await?
            .ok_or(RepoError::NotFound { entity: "product" })
    }

    async fn count(&self) -> Result<u64, RepoError> {
        let total = sqlx::query_scalar::<MySql, i64>("SELECT COUNT(*) FROM products")
            .fetch_one(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(u64::try_from(total).unwrap_or(0))
    }
}
