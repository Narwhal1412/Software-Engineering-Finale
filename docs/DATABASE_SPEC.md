# DATABASE_SPEC – Đặc tả database cho backend

Tài liệu dành cho thành viên phụ trách database. Đọc xong có thể tạo bảng, viết repository thật
và tích hợp vào backend mà **không cần đọc code**. Mọi tên bảng/cột/hàm dưới đây khớp với
`Backend/src/models/` và `Backend/src/repositories/`.

---

## 1. Mục đích và cách backend kết nối database

Backend (Rust, axum) hiện chạy bằng repository **mock in-memory**. Khi có database thật, chỉ cần
viết các struct repository mới implement đúng 5 trait ở mục 5 và đổi **một chỗ duy nhất**
(`build_repositories()` trong `src/app_state.rs`). Services và controllers không phải sửa.

| Hạng mục | Đề xuất |
|---|---|
| Hệ quản trị | **MariaDB 10.11 LTS trở lên** (README gốc của repo đã chọn MariaDB) |
| Charset / collation | `utf8mb4` / `utf8mb4_unicode_ci` (so sánh không phân biệt hoa/thường) |
| Thư viện Rust | `sqlx` (feature `runtime-tokio`, `mysql`, `chrono`; driver `mysql` dùng được cho MariaDB) |
| Kiểu truy vấn | Dùng `sqlx::query` / `query_as` (runtime). **Không** dùng macro `query!` vì cần DB lúc build, gây khó cho CI/Docker |
| Kết nối | Dựng bằng `MySqlConnectOptions::new().host().port().username().password().database()` + `MySqlPoolOptions`. Không ghép chuỗi URL vì mật khẩu có ký tự đặc biệt phải mã hóa |
| Giờ | Lưu **UTC** trong cột `DATETIME(6)`. Đọc bằng `chrono::NaiveDateTime` rồi `.and_utc()` ra `DateTime<Utc>` |
| Kiểu số nguyên | Khóa và tiền là `BIGINT` (có dấu) ↔ `i64`; số lượng là `INT` ↔ `i32` |

Biến môi trường (đã có trong `Backend/.env.example` và `Config`):

| Biến | Ý nghĩa | Mặc định |
|---|---|---|
| `DB_HOST` | Máy chủ DB (trong docker compose là tên service DB) | `localhost` |
| `DB_PORT` | Cổng | `3306` |
| `DB_USER` | Tài khoản | `app` |
| `DB_PASSWORD` | Mật khẩu | rỗng |
| `DB_NAME` | Tên database | `appdb` |

Các giá trị này đọc qua `config.db_host`, `config.db_port`, `config.db_user`, `config.db_password`, `config.db_name`.

**Quy ước chung**
- Tiền là số nguyên **đồng VND**, không có phần thập phân (`BIGINT`).
- Mọi cột chuỗi `NOT NULL`; "chưa có" biểu diễn bằng chuỗi rỗng `''` (riêng `image_url`, `category_id`, `pinned_at` cho phép `NULL`).
- Độ dài tối đa của cột nằm ở `src/models/limits.rs`; service validate theo các hằng số đó nên DB không bao giờ nhận chuỗi dài hơn.
- Đặt tên ràng buộc rõ ràng (`fk_...`, `uq_...`, `ck_...`) để repository phân biệt lỗi dựa vào tên.

---

## 2. Danh sách bảng

### 2.1 `users` – tài khoản

| Cột | Kiểu | NULL? | Mặc định | Khóa / ràng buộc | Ghi chú |
|---|---|---|---|---|---|
| `id` | `BIGINT` | NOT NULL | `AUTO_INCREMENT` | **PK** | |
| `email` | `VARCHAR(254)` | NOT NULL | | **UNIQUE** (`uq_users_email`) | Không phân biệt hoa/thường nhờ collation |
| `display_name` | `VARCHAR(100)` | NOT NULL | | | |
| `password_hash` | `VARCHAR(255)` | NOT NULL | | | Chuỗi PHC Argon2id (`$argon2id$...`), ~97 ký tự |
| `phone` | `VARCHAR(20)` | NOT NULL | `''` | | |
| `address` | `VARCHAR(500)` | NOT NULL | `''` | | |
| `role` | `ENUM('user','admin')` | NOT NULL | `'user'` | | |
| `created_at` | `DATETIME(6)` | NOT NULL | `(UTC_TIMESTAMP(6))` | | UTC |

Index: PK, `uq_users_email`. CHECK: không có.

### 2.2 `categories` – danh mục

| Cột | Kiểu | NULL? | Mặc định | Khóa / ràng buộc | Ghi chú |
|---|---|---|---|---|---|
| `id` | `BIGINT` | NOT NULL | `AUTO_INCREMENT` | **PK** | |
| `name` | `VARCHAR(100)` | NOT NULL | | **UNIQUE** (`uq_categories_name`) | |

### 2.3 `products` – sản phẩm

| Cột | Kiểu | NULL? | Mặc định | Khóa / ràng buộc | Ghi chú |
|---|---|---|---|---|---|
| `id` | `BIGINT` | NOT NULL | `AUTO_INCREMENT` | **PK** | |
| `name` | `VARCHAR(200)` | NOT NULL | | | |
| `specs` | `TEXT` | NOT NULL | | | Backend luôn truyền giá trị (có thể `''`) |
| `price` | `BIGINT` | NOT NULL | | `CHECK (price >= 0)` (`ck_products_price`) | VND |
| `category_id` | `BIGINT` | **NULL** | `NULL` | **FK** → `categories(id)` `ON DELETE SET NULL` | NULL = chưa phân loại |
| `image_url` | `VARCHAR(512)` | **NULL** | `NULL` | | |
| `pinned_at` | `DATETIME(6)` | **NULL** | `NULL` | | NULL = không ghim |
| `created_at` | `DATETIME(6)` | NOT NULL | `(UTC_TIMESTAMP(6))` | | |

Index đề xuất: `idx_products_category (category_id)`, `idx_products_created (created_at)`, `idx_products_price (price)`.

### 2.4 `cart_items` – giỏ hàng

| Cột | Kiểu | NULL? | Mặc định | Khóa / ràng buộc | Ghi chú |
|---|---|---|---|---|---|
| `user_id` | `BIGINT` | NOT NULL | | **PK (1/2)**, **FK** → `users(id)` `ON DELETE CASCADE` | |
| `product_id` | `BIGINT` | NOT NULL | | **PK (2/2)**, **FK** → `products(id)` `ON DELETE CASCADE` | |
| `quantity` | `INT` | NOT NULL | | `CHECK (quantity >= 1)` (`ck_cart_items_quantity`) | |

Khóa chính ghép `(user_id, product_id)` đảm bảo mỗi sản phẩm chỉ một dòng trong giỏ. Index: `idx_cart_items_product (product_id)` (phục vụ FK).

### 2.5 `orders` – đơn hàng

| Cột | Kiểu | NULL? | Mặc định | Khóa / ràng buộc | Ghi chú |
|---|---|---|---|---|---|
| `id` | `BIGINT` | NOT NULL | `AUTO_INCREMENT` | **PK** | |
| `user_id` | `BIGINT` | NOT NULL | | **FK** → `users(id)` `ON DELETE RESTRICT` | |
| `total` | `BIGINT` | NOT NULL | | `CHECK (total >= 0)` (`ck_orders_total`) | = Σ `price × quantity` của `order_items`, lưu cố định |
| `status` | `ENUM('pending','confirmed','shipped','completed','cancelled')` | NOT NULL | `'pending'` | | |
| `payment_method` | `ENUM('cod','bank')` | NOT NULL | | | |
| `full_name` | `VARCHAR(100)` | NOT NULL | | | Người nhận |
| `phone` | `VARCHAR(20)` | NOT NULL | | | |
| `address` | `VARCHAR(500)` | NOT NULL | | | |
| `created_at` | `DATETIME(6)` | NOT NULL | `(UTC_TIMESTAMP(6))` | | |

Index đề xuất: `idx_orders_user_created (user_id, created_at)`, `idx_orders_created (created_at)`, `idx_orders_status (status)`.

### 2.6 `order_items` – dòng hàng của đơn (snapshot)

| Cột | Kiểu | NULL? | Mặc định | Khóa / ràng buộc | Ghi chú |
|---|---|---|---|---|---|
| `id` | `BIGINT` | NOT NULL | `AUTO_INCREMENT` | **PK** | |
| `order_id` | `BIGINT` | NOT NULL | | **FK** → `orders(id)` `ON DELETE CASCADE` | |
| `product_id` | `BIGINT` | NOT NULL | | **FK** → `products(id)` `ON DELETE RESTRICT` | |
| `name` | `VARCHAR(200)` | NOT NULL | | | Tên sản phẩm lúc đặt (snapshot) |
| `price` | `BIGINT` | NOT NULL | | `CHECK (price >= 0)` (`ck_order_items_price`) | Đơn giá lúc đặt (snapshot) |
| `quantity` | `INT` | NOT NULL | | `CHECK (quantity >= 1)` (`ck_order_items_quantity`) | |

Index: `idx_order_items_order (order_id)`, `idx_order_items_product (product_id)`.

### 2.7 Script tạo bảng đầy đủ (dán vào `Database/init scripts/init.sql`)

```sql
SET NAMES utf8mb4;

CREATE TABLE users (
  id            BIGINT        NOT NULL AUTO_INCREMENT,
  email         VARCHAR(254)  NOT NULL,
  display_name  VARCHAR(100)  NOT NULL,
  password_hash VARCHAR(255)  NOT NULL,
  phone         VARCHAR(20)   NOT NULL DEFAULT '',
  address       VARCHAR(500)  NOT NULL DEFAULT '',
  role          ENUM('user','admin') NOT NULL DEFAULT 'user',
  created_at    DATETIME(6)   NOT NULL DEFAULT (UTC_TIMESTAMP(6)),
  PRIMARY KEY (id),
  CONSTRAINT uq_users_email UNIQUE (email)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE categories (
  id   BIGINT       NOT NULL AUTO_INCREMENT,
  name VARCHAR(100) NOT NULL,
  PRIMARY KEY (id),
  CONSTRAINT uq_categories_name UNIQUE (name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE products (
  id          BIGINT       NOT NULL AUTO_INCREMENT,
  name        VARCHAR(200) NOT NULL,
  specs       TEXT         NOT NULL,
  price       BIGINT       NOT NULL,
  category_id BIGINT       NULL DEFAULT NULL,
  image_url   VARCHAR(512) NULL DEFAULT NULL,
  pinned_at   DATETIME(6)  NULL DEFAULT NULL,
  created_at  DATETIME(6)  NOT NULL DEFAULT (UTC_TIMESTAMP(6)),
  PRIMARY KEY (id),
  KEY idx_products_category (category_id),
  KEY idx_products_created (created_at),
  KEY idx_products_price (price),
  CONSTRAINT ck_products_price CHECK (price >= 0),
  CONSTRAINT fk_products_category FOREIGN KEY (category_id)
    REFERENCES categories (id) ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE cart_items (
  user_id    BIGINT NOT NULL,
  product_id BIGINT NOT NULL,
  quantity   INT    NOT NULL,
  PRIMARY KEY (user_id, product_id),
  KEY idx_cart_items_product (product_id),
  CONSTRAINT ck_cart_items_quantity CHECK (quantity >= 1),
  CONSTRAINT fk_cart_items_user FOREIGN KEY (user_id)
    REFERENCES users (id) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT fk_cart_items_product FOREIGN KEY (product_id)
    REFERENCES products (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE orders (
  id             BIGINT       NOT NULL AUTO_INCREMENT,
  user_id        BIGINT       NOT NULL,
  total          BIGINT       NOT NULL,
  status         ENUM('pending','confirmed','shipped','completed','cancelled')
                              NOT NULL DEFAULT 'pending',
  payment_method ENUM('cod','bank') NOT NULL,
  full_name      VARCHAR(100) NOT NULL,
  phone          VARCHAR(20)  NOT NULL,
  address        VARCHAR(500) NOT NULL,
  created_at     DATETIME(6)  NOT NULL DEFAULT (UTC_TIMESTAMP(6)),
  PRIMARY KEY (id),
  KEY idx_orders_user_created (user_id, created_at),
  KEY idx_orders_created (created_at),
  KEY idx_orders_status (status),
  CONSTRAINT ck_orders_total CHECK (total >= 0),
  CONSTRAINT fk_orders_user FOREIGN KEY (user_id)
    REFERENCES users (id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE order_items (
  id         BIGINT       NOT NULL AUTO_INCREMENT,
  order_id   BIGINT       NOT NULL,
  product_id BIGINT       NOT NULL,
  name       VARCHAR(200) NOT NULL,
  price      BIGINT       NOT NULL,
  quantity   INT          NOT NULL,
  PRIMARY KEY (id),
  KEY idx_order_items_order (order_id),
  KEY idx_order_items_product (product_id),
  CONSTRAINT ck_order_items_price CHECK (price >= 0),
  CONSTRAINT ck_order_items_quantity CHECK (quantity >= 1),
  CONSTRAINT fk_order_items_order FOREIGN KEY (order_id)
    REFERENCES orders (id) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT fk_order_items_product FOREIGN KEY (product_id)
    REFERENCES products (id) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

---

## 3. Quan hệ giữa các bảng

| Quan hệ | Loại | Bảng trung gian | ON DELETE | ON UPDATE |
|---|---|---|---|---|
| `categories` 1 — N `products` | 1-N (tùy chọn phía product) | – | **SET NULL** (xóa danh mục không xóa sản phẩm – theo API.md) | CASCADE |
| `users` 1 — N `orders` | 1-N | – | **RESTRICT** (người đã có đơn không xóa được) | CASCADE |
| `orders` 1 — N `order_items` | 1-N | – | **CASCADE** | CASCADE |
| `products` 1 — N `order_items` | 1-N | – | **RESTRICT** (sản phẩm đã từng được bán không xóa được) | CASCADE |
| `users` N — N `products` | N-N | **`cart_items`** (có thêm cột `quantity`) | **CASCADE** cả hai phía | CASCADE |

Giải thích các lựa chọn RESTRICT (đây là **giả định**, API.md không nói rõ – xem `BACKEND_ASSUMPTIONS.md`):
đơn hàng đã lưu snapshot tên/giá nhưng vẫn cần `product_id` hợp lệ và doanh thu phải tính được, nên không cho xóa
sản phẩm/người dùng đã có trong đơn. Backend sẽ trả HTTP 409 kèm thông báo tiếng Việt.

---

## 4. Ánh xạ bảng ↔ struct ↔ trait

| Bảng | Struct (`src/models/`) | Trait repository (`src/repositories/`) | Mock hiện có |
|---|---|---|---|
| `users` | `user.rs`: `User`, `NewUser`, `ProfileUpdate` | `UserRepository` | `InMemoryUserRepository` |
| `categories` | `category.rs`: `Category` | `CategoryRepository` | `InMemoryCategoryRepository` |
| `products` | `product.rs`: `Product`, `ProductInput`, `ProductQuery`, `ProductSort` | `ProductRepository` | `InMemoryProductRepository` |
| `cart_items` | `cart.rs`: `CartItem`, `CartLine` (giỏ + sản phẩm) | `CartRepository` | `InMemoryCartRepository` |
| `orders` + `order_items` | `order.rs`: `Order` (chứa `items: Vec<OrderItem>`), `OrderItem`, `OrderDraft`, `OrderWithEmail` | `OrderRepository` | `InMemoryOrderRepository` |

Enum (`models/enums.rs`): `Role` ↔ `users.role`; `OrderStatus` ↔ `orders.status`; `PaymentMethod` ↔ `orders.payment_method`.
Mỗi enum có `as_str()` (khi ghi) và `FromStr` (khi đọc), giá trị chuỗi khớp đúng `ENUM` ở trên.

Lỗi trả về là `RepoError` (`repositories/repo_error.rs`), bảng đổi mã lỗi MariaDB → `RepoError`:

| Mã lỗi MariaDB | Ý nghĩa | `RepoError` |
|---|---|---|
| 1062 `ER_DUP_ENTRY` | Vi phạm UNIQUE | `Duplicate { field }` (`"email"` cho `uq_users_email`, `"name"` cho `uq_categories_name`) |
| 1452 `ER_NO_REFERENCED_ROW_2` | FK trỏ tới bản ghi không có | `InvalidReference { field }` (từ tên ràng buộc: `fk_products_category` → `"category_id"`, `fk_cart_items_user`/`fk_orders_user` → `"user_id"`, `fk_cart_items_product`/`fk_order_items_product` → `"product_id"`) |
| 1451 `ER_ROW_IS_REFERENCED_2` | Xóa bản ghi đang bị tham chiếu | `InUse { entity, referenced_by }` (`products` → `("product","order")`, `users` → `("user","order")`) |
| 4025 `ER_CONSTRAINT_FAILED` (MariaDB) / 3819 (MySQL) | Vi phạm CHECK | `Constraint(chi tiết)` |
| 1406 `ER_DATA_TOO_LONG` | Chuỗi quá dài | `Constraint(chi tiết)` |
| Lỗi khác (mất kết nối, cú pháp...) | | `Internal(chi tiết)` |
| `UPDATE`/`DELETE`/`SELECT` không thấy dòng | | `NotFound { entity }` |

> Lưu ý `UPDATE`: MariaDB có thể báo `rows_affected = 0` khi giá trị mới giống giá trị cũ, nên **đừng** dùng `rows_affected`
> để quyết định `NotFound` cho `UPDATE`. Hãy `SELECT` lại bản ghi sau khi `UPDATE` và trả `NotFound` nếu không có dòng.
> Với `DELETE` thì `rows_affected = 0` nghĩa là không tồn tại, dùng được.

---

## 5. Các hàm của từng trait Repository

Mọi hàm là `async`, trả `Result<T, RepoError>`. Dưới đây `…cols` nghĩa là liệt kê đủ cột của struct tương ứng theo đúng thứ tự mục 2.

### 5.1 `UserRepository`

| Chữ ký | Ý nghĩa | SQL gợi ý |
|---|---|---|
| `create(NewUser) -> User` | Tạo tài khoản; 1062 → `Duplicate{email}` | `INSERT INTO users (email, display_name, password_hash, phone, address, role, created_at) VALUES (?,?,?,?,?,?, UTC_TIMESTAMP(6));` rồi `SELECT … WHERE id = LAST_INSERT_ID()` |
| `find_by_id(i64) -> Option<User>` | Tìm theo id | `SELECT … FROM users WHERE id = ?` |
| `find_by_email(&str) -> Option<User>` | Tìm theo email (không phân biệt hoa/thường nhờ collation) | `SELECT … FROM users WHERE email = ?` |
| `list() -> Vec<User>` | Mọi tài khoản, id tăng dần | `SELECT … FROM users ORDER BY id ASC` |
| `update_profile(i64, ProfileUpdate) -> User` | Sửa tên, SĐT, địa chỉ; không có → `NotFound{user}` | `UPDATE users SET display_name=?, phone=?, address=? WHERE id=?` rồi `SELECT` lại |
| `delete(i64)` | Xóa tài khoản (giỏ hàng tự xóa theo CASCADE); 1451 → `InUse{user, order}`; 0 dòng → `NotFound{user}` | `DELETE FROM users WHERE id = ?` |
| `count() -> u64` | Tổng số tài khoản | `SELECT COUNT(*) FROM users` |

### 5.2 `CategoryRepository`

| Chữ ký | Ý nghĩa | SQL gợi ý |
|---|---|---|
| `list() -> Vec<Category>` | Mọi danh mục, id tăng dần | `SELECT id, name FROM categories ORDER BY id ASC` |
| `find_by_id(i64) -> Option<Category>` | | `SELECT id, name FROM categories WHERE id = ?` |
| `create(&str) -> Category` | 1062 → `Duplicate{name}` | `INSERT INTO categories (name) VALUES (?)` rồi `SELECT` lại |
| `rename(i64, &str) -> Category` | Không có → `NotFound{category}`; trùng → `Duplicate{name}` | `UPDATE categories SET name=? WHERE id=?` rồi `SELECT` lại |
| `delete(i64)` | Sản phẩm thuộc danh mục tự về `category_id = NULL` nhờ `ON DELETE SET NULL`; 0 dòng → `NotFound{category}` | `DELETE FROM categories WHERE id = ?` |

### 5.3 `ProductRepository`

| Chữ ký | Ý nghĩa | SQL gợi ý |
|---|---|---|
| `search(&ProductQuery) -> Vec<Product>` | Lọc + tìm + sắp xếp (xem dưới) | xem khối SQL bên dưới |
| `find_by_id(i64) -> Option<Product>` | | `SELECT … FROM products WHERE id = ?` |
| `create(ProductInput) -> Product` | `pinned_at = NULL`; 1452 → `InvalidReference{category_id}` | `INSERT INTO products (name, specs, price, category_id, image_url, pinned_at, created_at) VALUES (?,?,?,?,?, NULL, UTC_TIMESTAMP(6))` rồi `SELECT` lại |
| `update(i64, ProductInput) -> Product` | Ghi đè 5 trường nhập; `NotFound{product}` / `InvalidReference{category_id}` | `UPDATE products SET name=?, specs=?, price=?, category_id=?, image_url=? WHERE id=?` rồi `SELECT` lại |
| `delete(i64)` | Giỏ hàng chứa nó tự xóa (CASCADE); 1451 → `InUse{product, order}`; 0 dòng → `NotFound{product}` | `DELETE FROM products WHERE id = ?` |
| `set_pinned(i64, bool) -> Product` | Ghim: `pinned_at = bây giờ`; bỏ ghim: `NULL`. Ghim lại sản phẩm đã ghim thì đưa nó lên đầu | `UPDATE products SET pinned_at = ? WHERE id = ?` (bind `Utc::now()` hoặc `NULL`) rồi `SELECT` lại |
| `count() -> u64` | | `SELECT COUNT(*) FROM products` |

`search` – quy tắc: (1) lọc `category_id` nếu có; (2) từ khóa `q` (bỏ khoảng trắng đầu/cuối, bỏ qua nếu rỗng) tìm trong **name và specs**;
(3) sản phẩm đã ghim đứng đầu, ghim mới nhất trước; (4) phần còn lại theo `sort`.

```sql
SELECT id, name, specs, price, category_id, image_url, pinned_at, created_at
FROM products
WHERE (:category_id IS NULL OR category_id = :category_id)
  AND (:kw IS NULL
       OR name  LIKE CONCAT('%', :kw, '%') ESCAPE '\\'
       OR specs LIKE CONCAT('%', :kw, '%') ESCAPE '\\')
ORDER BY (pinned_at IS NULL) ASC,   -- đã ghim trước
         pinned_at DESC,            -- ghim mới nhất trước
         <ORDER_BY_THEO_SORT>;
```
`<ORDER_BY_THEO_SORT>` ghép từ **danh sách trắng** (không bao giờ nối chuỗi người dùng nhập):

| `ProductSort` | Đoạn ORDER BY |
|---|---|
| `Newest` (`sort=new`, mặc định) | `created_at DESC, id DESC` |
| `PriceAsc` | `price ASC, id ASC` |
| `PriceDesc` | `price DESC, id ASC` |

Với `:kw`, hãy escape ký tự `\`, `%`, `_` trong từ khóa trước khi bind. *Khác biệt có chủ ý:* collation `utf8mb4_unicode_ci`
không phân biệt dấu (tìm "ban" ra cả "bàn"), trong khi mock chỉ bỏ phân biệt hoa/thường; DB thật hành xử "rộng" hơn là chấp nhận được.

### 5.4 `CartRepository`

| Chữ ký | Ý nghĩa | SQL gợi ý |
|---|---|---|
| `list_with_products(user_id) -> Vec<CartLine>` | Giỏ kèm sản phẩm, sắp theo `product_id` | `SELECT ci.product_id, ci.quantity, p.id, p.name, p.specs, p.price, p.category_id, p.image_url, p.pinned_at, p.created_at FROM cart_items ci JOIN products p ON p.id = ci.product_id WHERE ci.user_id = ? ORDER BY ci.product_id` |
| `add_item(user_id, product_id, quantity) -> CartItem` | Chưa có thì thêm, có rồi thì **cộng dồn**. `quantity < 1` → `Constraint` (kiểm tra trong code trước khi gọi SQL); 1452 → `InvalidReference{user_id/product_id}` | `INSERT INTO cart_items (user_id, product_id, quantity) VALUES (?,?,?) ON DUPLICATE KEY UPDATE quantity = quantity + VALUES(quantity)` rồi `SELECT` lại |
| `set_quantity(user_id, product_id, quantity) -> CartItem` | **Đặt** số lượng tuyệt đối; `quantity < 1` → `Constraint`; chưa có dòng → `NotFound{cart item}` | `UPDATE cart_items SET quantity=? WHERE user_id=? AND product_id=?` rồi `SELECT` lại (xem lưu ý `rows_affected` ở mục 4) |
| `remove_item(user_id, product_id)` | Xóa một dòng; **idempotent** (không có dòng cũng `Ok`) | `DELETE FROM cart_items WHERE user_id=? AND product_id=?` |
| `clear(user_id)` | Xóa cả giỏ | `DELETE FROM cart_items WHERE user_id = ?` |

### 5.5 `OrderRepository`

Mọi danh sách sắp **mới nhất trước**: `ORDER BY created_at DESC, id DESC`. `Order.items` luôn được nạp sẵn, sắp theo `order_items.id`.
Nạp items bằng **một** truy vấn `WHERE order_id IN (…)` cho cả danh sách (tránh N+1).

| Chữ ký | Ý nghĩa | SQL gợi ý |
|---|---|---|
| `create_from_cart(user_id, OrderDraft) -> Order` | Tạo đơn từ giỏ **trong một transaction** (xem khối dưới) | xem dưới |
| `list_by_user(user_id) -> Vec<Order>` | Đơn của một người | `SELECT … FROM orders WHERE user_id = ? ORDER BY created_at DESC, id DESC` + nạp items |
| `list_all_with_user_email() -> Vec<OrderWithEmail>` | Mọi đơn kèm email người đặt | `SELECT o.…, u.email FROM orders o LEFT JOIN users u ON u.id = o.user_id ORDER BY o.created_at DESC, o.id DESC` + nạp items |
| `find_by_id(i64) -> Option<Order>` | | `SELECT … FROM orders WHERE id = ?` + nạp items |
| `update_status(i64, OrderStatus) -> Order` | Không có → `NotFound{order}`. Không kiểm tra luật chuyển trạng thái (việc của service) | `UPDATE orders SET status=? WHERE id=?` rồi `SELECT` lại |
| `count() -> u64` | Đếm cả đơn đã hủy | `SELECT COUNT(*) FROM orders` |
| `revenue_excluding_cancelled() -> i64` | Doanh thu, bỏ đơn đã hủy | `SELECT CAST(COALESCE(SUM(total), 0) AS SIGNED) FROM orders WHERE status <> 'cancelled'` |

`create_from_cart` – bắt buộc nguyên tử (thất bại thì không đổi gì; giỏ vẫn còn):

```sql
START TRANSACTION;

-- 1. Đọc giỏ cùng tên/giá HIỆN HÀNH của sản phẩm (đây chính là snapshot)
SELECT ci.product_id, ci.quantity, p.name, p.price
FROM cart_items ci JOIN products p ON p.id = ci.product_id
WHERE ci.user_id = :user_id
ORDER BY ci.product_id
FOR UPDATE;
--    Không có dòng nào  => ROLLBACK, trả RepoError::Constraint("cart is empty")

-- 2. Trong Rust: total = Σ price * quantity

-- 3. Tạo đơn (status luôn 'pending'); 1452 trên fk_orders_user => InvalidReference{user_id}
INSERT INTO orders (user_id, total, status, payment_method, full_name, phone, address, created_at)
VALUES (:user_id, :total, 'pending', :payment_method, :full_name, :phone, :address, UTC_TIMESTAMP(6));
--    order_id = LAST_INSERT_ID()

-- 4. Mỗi dòng giỏ -> một dòng order_items (cùng thứ tự product_id)
INSERT INTO order_items (order_id, product_id, name, price, quantity) VALUES (:order_id, ?, ?, ?, ?);

-- 5. Xóa giỏ
DELETE FROM cart_items WHERE user_id = :user_id;

COMMIT;
```
Sau `COMMIT`, đọc lại đơn (kèm items) để trả về.

---

## 6. Dữ liệu seed (khớp với mock)

Mock (`repositories/mock/seed.rs`) tạo đúng bộ dữ liệu này. **Mật khẩu của cả 4 tài khoản là `123456`** (chỉ để demo).

| Id | Email | Tên | Vai trò | Ghi chú |
|---|---|---|---|---|
| 1 | `admin@pc.vn` | Quản trị | admin | trùng tài khoản demo của frontend |
| 2 | `khach1@pc.vn` | Nguyễn Văn An | user | có 4 đơn + 2 dòng trong giỏ |
| 3 | `khach2@pc.vn` | Trần Thị Bình | user | có 3 đơn |
| 4 | `khach3@pc.vn` | Lê Hoàng Cường | user | không có đơn → dùng để thử xóa thành viên |

**Trạng thái hiện tại:** các câu lệnh dưới đây đã được chép vào `Database/init scripts/init.sql`, trong đó biến `@pw` đã chứa hash Argon2id thật của `123456`
(xem mục 7.7), nên khi chạy `docker compose` không phải dán gì thêm. Nếu muốn đổi mật khẩu mẫu, sinh hash bằng chính thuật toán của backend (trong thư mục `Backend/`):

```bash
cargo run -- hash-password 123456
```
Lệnh in ra chuỗi `$argon2id$v=19$...`; thay vào biến `@pw` (một hash dùng chung cho cả 4 tài khoản là đủ). Khối SQL dưới đây giữ placeholder để làm tài liệu:

```sql
SET NAMES utf8mb4;
SET @pw = '<DÁN_CHUỖI_HASH_Ở_ĐÂY>';

INSERT INTO users (id, email, display_name, password_hash, phone, address, role, created_at) VALUES
 (1, 'admin@pc.vn',   'Quản trị',       @pw, '', '', 'admin', UTC_TIMESTAMP(6) - INTERVAL 10 DAY),
 (2, 'khach1@pc.vn',  'Nguyễn Văn An',  @pw, '', '', 'user',  UTC_TIMESTAMP(6) - INTERVAL 10 DAY),
 (3, 'khach2@pc.vn',  'Trần Thị Bình',  @pw, '', '', 'user',  UTC_TIMESTAMP(6) - INTERVAL 10 DAY),
 (4, 'khach3@pc.vn',  'Lê Hoàng Cường', @pw, '', '', 'user',  UTC_TIMESTAMP(6) - INTERVAL 10 DAY);

INSERT INTO categories (id, name) VALUES
 (1, 'CPU'), (2, 'Card đồ họa'), (3, 'RAM'), (4, 'Ổ cứng'), (5, 'PC lắp sẵn');

INSERT INTO products (id, name, specs, price, category_id, image_url, pinned_at, created_at) VALUES
 (1, 'Ryzen 7 7800X3D',        '8 nhân 16 luồng, 4.2 GHz, AM5',              9490000, 1, NULL, UTC_TIMESTAMP(6) - INTERVAL 1 DAY, UTC_TIMESTAMP(6) - INTERVAL 9 DAY),
 (2, 'Core i5-14600K',         '14 nhân, 5.3 GHz, LGA1700',                  7290000, 1, NULL, NULL, UTC_TIMESTAMP(6) - INTERVAL 8 DAY),
 (3, 'GeForce RTX 4070 Super', '12 GB GDDR6X, 3 quạt',                      16990000, 2, NULL, NULL, UTC_TIMESTAMP(6) - INTERVAL 6 DAY),
 (4, 'Radeon RX 7800 XT',      '16 GB GDDR6, PCIe 4.0',                     12790000, 2, NULL, NULL, UTC_TIMESTAMP(6) - INTERVAL 5 DAY),
 (5, 'Kingston Fury 32GB',     'DDR5 6000 MHz, 2x16 GB',                     2890000, 3, NULL, NULL, UTC_TIMESTAMP(6) - INTERVAL 4 DAY),
 (6, 'Samsung 980 Pro 1TB',    'NVMe Gen4, đọc 7000 MB/s',                   2650000, 4, NULL, NULL, UTC_TIMESTAMP(6) - INTERVAL 3 DAY),
 (7, 'PC Gaming Forge R7',     'R7 7800X3D, RTX 4070 Super, 32GB, 1TB',     36900000, 5, NULL, NULL, UTC_TIMESTAMP(6) - INTERVAL 2 DAY);

INSERT INTO cart_items (user_id, product_id, quantity) VALUES (2, 6, 1), (2, 2, 2);

INSERT INTO orders (id, user_id, total, status, payment_method, full_name, phone, address, created_at) VALUES
 (1, 2, 15270000, 'completed', 'cod',  'Nguyễn Văn An', '0900000000', 'TP.HCM', UTC_TIMESTAMP(6) - INTERVAL 0 DAY),
 (2, 3, 16990000, 'completed', 'bank', 'Trần Thị Bình', '0900000000', 'TP.HCM', UTC_TIMESTAMP(6) - INTERVAL 1 DAY),
 (3, 2, 36900000, 'shipped',   'cod',  'Nguyễn Văn An', '0900000000', 'TP.HCM', UTC_TIMESTAMP(6) - INTERVAL 2 DAY),
 (4, 3,  9940000, 'confirmed', 'bank', 'Trần Thị Bình', '0900000000', 'TP.HCM', UTC_TIMESTAMP(6) - INTERVAL 2 DAY),
 (5, 2, 12790000, 'pending',   'cod',  'Nguyễn Văn An', '0900000000', 'TP.HCM', UTC_TIMESTAMP(6) - INTERVAL 4 DAY),
 (6, 3,  2890000, 'cancelled', 'cod',  'Trần Thị Bình', '0900000000', 'TP.HCM', UTC_TIMESTAMP(6) - INTERVAL 5 DAY),
 (7, 2, 26480000, 'completed', 'bank', 'Nguyễn Văn An', '0900000000', 'TP.HCM', UTC_TIMESTAMP(6) - INTERVAL 6 DAY);

INSERT INTO order_items (id, order_id, product_id, name, price, quantity) VALUES
 (1,  1, 1, 'Ryzen 7 7800X3D',        9490000, 1),
 (2,  1, 5, 'Kingston Fury 32GB',     2890000, 2),
 (3,  2, 3, 'GeForce RTX 4070 Super', 16990000, 1),
 (4,  3, 7, 'PC Gaming Forge R7',     36900000, 1),
 (5,  4, 2, 'Core i5-14600K',         7290000, 1),
 (6,  4, 6, 'Samsung 980 Pro 1TB',    2650000, 1),
 (7,  5, 4, 'Radeon RX 7800 XT',      12790000, 1),
 (8,  6, 5, 'Kingston Fury 32GB',     2890000, 1),
 (9,  7, 1, 'Ryzen 7 7800X3D',        9490000, 1),
 (10, 7, 3, 'GeForce RTX 4070 Super', 16990000, 1);
```

Số liệu kiểm chứng sau khi nạp seed: `GET /api/admin/stats` phải trả `{products: 7, users: 4, orders: 7, revenue: 118370000}`
(doanh thu = tổng 7 đơn trừ đơn #6 đã hủy).

---

## 7. Hướng dẫn thay mock bằng repository thật

> **Quy ước đọc:** các đoạn code Rust trong mục này là **mã mẫu chưa được biên dịch** (môi trường soạn tài liệu không có Rust toolchain
> lẫn mạng để tải `sqlx`). Hãy chạy `cargo build` và sửa tại chỗ nếu có lệch phiên bản. Các file cấu hình (Dockerfile, compose, `init.sql`)
> đã nằm trong repo và đã được kiểm tra cú pháp YAML/SQL bằng mắt, **chưa chạy thử với Docker thật**.

### 7.0 Bức tranh tổng thể: cái gì đã có sẵn, cái gì còn phải làm

Nguyên tắc: service chỉ biết các **trait** repository (mục 5). Thay mock bằng DB thật = viết 5 struct mới implement 5 trait đó
rồi đổi **một chỗ** là thân hàm `build_repositories()` ở `Backend/src/app_state.rs`. Services, controllers, middleware không bị đụng tới.

| Hạng mục | Trạng thái | Vị trí |
|---|---|---|
| Script tạo bảng + seed (đã có hash mật khẩu thật) | **Đã có** | `Database/init scripts/init.sql` |
| Dockerfile backend (build Rust, chạy không bằng root, healthcheck) | **Đã có** | `Backend/Dockerfile`, `Backend/.dockerignore` |
| Compose 3 service: `frontend`, `backend`, `db` (MariaDB) | **Đã có** | `compose.yaml`, `compose.override.yaml` (local), `compose.prod.yaml` (server) |
| Biến môi trường `DB_*` đọc vào `Config` | **Đã có** | `Backend/src/config.rs`, `.env.example` |
| `pub mod db;` + file `db.rs` (đang chỉ có chú thích) | **Đã có** | `Backend/src/lib.rs`, `Backend/src/db.rs` |
| Log tài khoản mẫu đã chuyển vào nhánh mock | **Đã có** | `Backend/src/app_state.rs` (xem 7.8) |
| Thêm `sqlx` vào `Cargo.toml` | **Cần làm** | 7.2 |
| Viết `db.rs` (kết nối + thử lại) | **Cần làm** | 7.3 |
| Viết 5 repository MariaDB | **Cần làm** | 7.4 |
| Đổi thân `build_repositories()` | **Cần làm** | 7.5 |
| Bật `depends_on` của backend vào `db` | **Cần làm** | 7.6 |

Thứ tự làm đề xuất: **7.1 → 7.2 → 7.3 → 7.4 → 7.5 → 7.6 → checklist mục 8.**

### 7.1 Bước 1 – Dựng MariaDB và kiểm tra `init.sql` trước khi viết một dòng Rust

Làm bước này đầu tiên để biết schema/seed đúng, tránh đi tìm lỗi trong Rust mà thật ra lỗi nằm ở SQL.

```bash
cp .env.example .env                 # lần đầu; sửa mật khẩu nếu muốn
docker compose up -d db              # chỉ chạy database
docker compose logs -f db            # chờ dòng "ready for connections", không có "ERROR"
docker compose exec db mariadb -u app -p appdb -e "SELECT COUNT(*) AS users FROM users; SELECT COUNT(*) AS products FROM products; SELECT COUNT(*) AS orders FROM orders;"
```
(nhập mật khẩu `DB_PASSWORD` trong `.env`). Kết quả đúng: `users = 4`, `products = 7`, `orders = 7`.
Kết nối từ máy bằng DBeaver/HeidiSQL: host `127.0.0.1`, cổng `3306`, user/mật khẩu như `.env` (`compose.override.yaml` chỉ mở cổng này cho máy bạn).

**Điều quan trọng cần hiểu:** image `mariadb` chỉ chạy các file trong `/docker-entrypoint-initdb.d` **một lần duy nhất, khi thư mục dữ liệu còn trống**.
Nếu bạn sửa `init.sql` rồi `docker compose up` lại mà không thấy gì đổi, đó là vì volume `db_data` đã có dữ liệu. Muốn nạp lại từ đầu:

```bash
docker compose down -v               # -v xóa volume db_data (MẤT toàn bộ dữ liệu trong DB)
docker compose up -d db
```
Trên môi trường thật không dùng `down -v`; khi đó thay đổi schema phải đi bằng script migration riêng (nằm ngoài phạm vi đồ án này).

### 7.2 Bước 2 – Thêm `sqlx` vào `Backend/Cargo.toml`

Thêm vào mục `[dependencies]` (các thư viện khác trong file đều ghim phiên bản bằng dấu `=`, làm tương tự để mọi người build ra cùng kết quả):

```toml
sqlx = { version = "=0.8.2", default-features = false, features = ["runtime-tokio", "mysql", "chrono"] }
```

Giải thích từng phần:

| Phần | Lý do |
|---|---|
| `version = "=0.8.2"` | Ghim đúng một phiên bản. Dấu `=` cấm cargo tự nâng lên bản mới hơn. (Nếu `cargo build` báo không tìm thấy hoặc xung đột, thử `=0.8.3`, rồi ghi lại bản thật sự dùng được vào đây.) |
| `default-features = false` | Bỏ các tính năng mặc định không dùng (macro `query!`, migrate, json...) để build nhanh và nhẹ. Spec mục 1 đã yêu cầu dùng `sqlx::query`/`query_as` thay cho macro `query!` vì macro cần DB sống lúc build, gây khó cho CI/Docker. |
| `runtime-tokio` | Chạy trên tokio, cùng runtime với axum. |
| `mysql` | Driver MySQL; dùng được cho MariaDB. |
| `chrono` | Đọc/ghi cột `DATETIME(6)` thành `chrono::NaiveDateTime` (mục 1: lưu UTC rồi `.and_utc()`). |
| (không bật `tls-*`) | Trong compose, backend và db nói chuyện trong mạng nội bộ của Docker nên không cần TLS, và không phải cài thêm thư viện hệ thống cho Docker image. **Nếu DB nằm ở máy khác qua Internet** thì bật `tls-rustls` và đổi `MySqlSslMode::Disabled` thành `Required` ở 7.3. |

Sau lần `cargo build` đầu tiên thành công: **commit `Cargo.lock`**. `Dockerfile` đã sẵn sàng dùng `Cargo.lock` (nếu có) để cache và để CI build đúng bản đã thử.
Khi đã commit `Cargo.lock`, có thể đổi dòng build trong Dockerfile thành `cargo build --release --locked` để cấm cargo tự đổi phiên bản.

### 7.3 Bước 3 – Viết `Backend/src/db.rs`: kết nối và thử lại khi DB chưa sẵn sàng

`db.rs` đã được khai báo `pub mod db;` trong `lib.rs` (không cần sửa `lib.rs` nữa), hiện chỉ có chú thích. Thay nội dung bằng:

```rust
//! Kết nối MariaDB bằng sqlx.

use std::time::Duration;

use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlSslMode};
use sqlx::MySqlPool;

use crate::config::Config;
use crate::repositories::RepoError;

/// Số lần thử kết nối tối đa khi khởi động.
const MAX_ATTEMPTS: u32 = 10;
/// Chờ giữa hai lần thử.
const RETRY_DELAY: Duration = Duration::from_secs(3);
/// Số kết nối tối đa trong pool.
const MAX_CONNECTIONS: u32 = 10;
/// Chờ tối đa để lấy một kết nối.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Mở pool kết nối một lần (không thử lại).
pub async fn connect(config: &Config) -> Result<MySqlPool, RepoError> {
    // Dựng từng thành phần thay vì ghép chuỗi URL: mật khẩu có ký tự đặc biệt (@ : / ...)
    // sẽ làm hỏng URL nếu không mã hóa.
    let options = MySqlConnectOptions::new()
        .host(&config.db_host)
        .port(config.db_port)
        .username(&config.db_user)
        .password(&config.db_password)
        .database(&config.db_name)
        .charset("utf8mb4")
        .ssl_mode(MySqlSslMode::Disabled);

    MySqlPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_with(options)
        .await
        .map_err(|err| RepoError::Internal(format!("không kết nối được database: {err}")))
}

/// Mở pool, thử lại tối đa `MAX_ATTEMPTS` lần. Dùng ở `build_repositories()`.
pub async fn connect_with_retry(config: &Config) -> Result<MySqlPool, RepoError> {
    let mut attempt = 1;
    loop {
        match connect(config).await {
            Ok(pool) => {
                tracing::info!(attempt, "đã kết nối database");
                return Ok(pool);
            }
            Err(err) if attempt < MAX_ATTEMPTS => {
                tracing::warn!(attempt, max = MAX_ATTEMPTS, %err, "database chưa sẵn sàng, sẽ thử lại");
                tokio::time::sleep(RETRY_DELAY).await;
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}
```

**Vì sao phải thử lại?** Khi chạy `docker compose up`, backend (Rust, khởi động trong chưa tới 1 giây) thường chạy xong **trước** khi MariaDB sẵn sàng
nhận kết nối (lần đầu MariaDB còn phải khởi tạo dữ liệu và chạy `init.sql`, mất vài chục giây). Nếu chỉ kết nối một lần, backend sẽ thoát với lỗi
và Docker (`restart: unless-stopped`) phải khởi động lại nhiều lần. Có hai lớp bảo vệ, nên dùng **cả hai**:

1. **Thử lại trong code** (đoạn trên): 10 lần × 3 giây = tối đa ~30 giây. Hữu ích cả khi DB khởi động lại giữa chừng hoặc server (Komodo) dựng service theo thứ tự bất kỳ.
2. **`depends_on` + `healthcheck`** trong compose (xem 7.6): Docker chỉ khởi động backend khi DB báo "healthy".

Nếu hết `MAX_ATTEMPTS` mà vẫn không kết nối được, hàm trả lỗi, `main` kết thúc với thông báo lỗi rõ ràng thay vì chạy với kết nối hỏng.

### 7.4 Bước 4 – Viết 5 repository MariaDB

Tạo thư mục `Backend/src/repositories/mysql/` và thêm `pub mod mysql;` vào `repositories/mod.rs`. Mỗi entity một file (tên file/struct theo bảng dưới), cộng thêm `mod.rs` và `error.rs`:

| File | Nội dung | Implement trait |
|---|---|---|
| `mysql/mod.rs` | khai báo `mod`, `pub use` 5 struct | – |
| `mysql/error.rs` | hàm `map_sqlx_error` dùng chung (bên dưới) | – |
| `mysql/user_mysql_repository.rs` | `MySqlUserRepository { pool: MySqlPool }` | `UserRepository` (mục 5.1) |
| `mysql/category_mysql_repository.rs` | `MySqlCategoryRepository` | `CategoryRepository` (mục 5.2) |
| `mysql/product_mysql_repository.rs` | `MySqlProductRepository` | `ProductRepository` (mục 5.3) |
| `mysql/cart_mysql_repository.rs` | `MySqlCartRepository` | `CartRepository` (mục 5.4) |
| `mysql/order_mysql_repository.rs` | `MySqlOrderRepository` | `OrderRepository` (mục 5.5, có transaction) |

**`mysql/error.rs`** – đổi lỗi `sqlx` sang `RepoError` đúng bảng ở mục 4, dùng chung cho cả 5 file:

```rust
//! Đổi lỗi của sqlx/MariaDB sang `RepoError` (bảng ở DATABASE_SPEC mục 4).

use sqlx::mysql::MySqlDatabaseError;

use crate::repositories::RepoError;

/// Đổi một lỗi sqlx thành `RepoError`. Việc "không thấy dòng" (`NotFound`) do từng repository tự quyết
/// (dùng `fetch_optional`), vì chỉ repository mới biết entity nào.
pub(super) fn map_sqlx_error(err: sqlx::Error) -> RepoError {
    if let sqlx::Error::Database(db) = &err {
        let message = db.message();
        let number = db
            .try_downcast_ref::<MySqlDatabaseError>()
            .map(MySqlDatabaseError::number);
        match number {
            // Trùng UNIQUE: phân biệt theo tên ràng buộc trong thông báo lỗi.
            Some(1062) if message.contains("uq_users_email") => {
                return RepoError::Duplicate { field: "email" }
            }
            Some(1062) if message.contains("uq_categories_name") => {
                return RepoError::Duplicate { field: "name" }
            }
            // Khóa ngoại trỏ tới dòng không tồn tại.
            Some(1452) => return RepoError::InvalidReference { field: reference_field(message) },
            // Xóa dòng đang bị tham chiếu (ON DELETE RESTRICT).
            Some(1451) if message.contains("fk_order_items_product") => {
                return RepoError::InUse { entity: "product", referenced_by: "order" }
            }
            Some(1451) if message.contains("fk_orders_user") => {
                return RepoError::InUse { entity: "user", referenced_by: "order" }
            }
            // CHECK (4025 MariaDB, 3819 MySQL) hoặc chuỗi quá dài (1406).
            Some(4025 | 3819 | 1406) => return RepoError::Constraint(message.to_string()),
            _ => {}
        }
    }
    RepoError::Internal(err.to_string())
}

/// Tên trường khóa ngoại, suy ra từ tên ràng buộc (mục 4).
fn reference_field(message: &str) -> &'static str {
    if message.contains("fk_products_category") {
        "category_id"
    } else if message.contains("fk_cart_items_user") || message.contains("fk_orders_user") {
        "user_id"
    } else if message.contains("fk_cart_items_product") || message.contains("fk_order_items_product") {
        "product_id"
    } else {
        "reference"
    }
}
```
Vì mapping dựa trên **tên ràng buộc** (`uq_...`, `fk_...`), tuyệt đối không đổi tên các ràng buộc trong `init.sql` mà không sửa file này.

**Ví dụ một repository** (`category_mysql_repository.rs`, ba hàm đầu; `rename` và `delete` làm theo bảng mục 5.2):

```rust
//! Repository MariaDB cho bảng `categories`.

use async_trait::async_trait;
use sqlx::{mysql::MySqlRow, MySqlPool, Row};

use super::error::map_sqlx_error;
use crate::models::category::Category;
use crate::repositories::{CategoryRepository, RepoError};

pub struct MySqlCategoryRepository {
    pool: MySqlPool,
}

impl MySqlCategoryRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

fn row_to_category(row: &MySqlRow) -> Result<Category, RepoError> {
    Ok(Category {
        id: row.try_get("id").map_err(map_sqlx_error)?,
        name: row.try_get("name").map_err(map_sqlx_error)?,
    })
}

#[async_trait]
impl CategoryRepository for MySqlCategoryRepository {
    async fn list(&self) -> Result<Vec<Category>, RepoError> {
        let rows = sqlx::query("SELECT id, name FROM categories ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        rows.iter().map(row_to_category).collect()
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Category>, RepoError> {
        let row = sqlx::query("SELECT id, name FROM categories WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        row.as_ref().map(row_to_category).transpose()
    }

    async fn create(&self, name: &str) -> Result<Category, RepoError> {
        let result = sqlx::query("INSERT INTO categories (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?; // trùng tên -> Duplicate { field: "name" }
        Ok(Category { id: result.last_insert_id() as i64, name: name.to_owned() })
    }
    // rename, delete: xem mục 5.2
}
```
Lưu ý khi viết các repository còn lại: luôn dùng `.bind(...)` (tham số hóa) chứ không ghép chuỗi SQL, và đọc lại mục "Lưu ý `UPDATE`" ở cuối mục 4 (không dùng `rows_affected` để quyết định `NotFound` cho `UPDATE`).

### 7.5 Bước 5 – Đổi đúng một chỗ: `build_repositories()` trong `app_state.rs`

Thay thân hàm (và bỏ các `use` của mock, nếu không `cargo clippy -- -D warnings` sẽ báo import thừa):

```rust
pub async fn build_repositories(config: &Config) -> Result<Repositories, RepoError> {
    let pool = crate::db::connect_with_retry(config).await?;
    Ok(Repositories {
        users: Arc::new(MySqlUserRepository::new(pool.clone())),
        categories: Arc::new(MySqlCategoryRepository::new(pool.clone())),
        products: Arc::new(MySqlProductRepository::new(pool.clone())),
        carts: Arc::new(MySqlCartRepository::new(pool.clone())),
        orders: Arc::new(MySqlOrderRepository::new(pool)),
    })
}
```
(`MySqlPool` là một handle `Clone` rẻ, các bản sao dùng chung một pool, nên truyền `pool.clone()` cho từng repository là đúng.)
Không file nào khác trong services/controllers/middleware phải sửa. Thư mục `repositories/mock/` **giữ lại**: các bài test hiện có và test tích hợp (`tests/api_tests.rs`) vẫn chạy được không cần database.

### 7.6 Bước 6 – Docker và compose

**Hiện trạng:** repo đã có đủ Docker cho cả 3 tầng; stack `docker compose up --build` dựng được giao diện + backend (mock) + MariaDB ngay bây giờ.

| File | Vai trò và những điểm đáng chú ý |
|---|---|
| `Backend/Dockerfile` | Hai giai đoạn. **Giai đoạn build** dùng image `rust`, có bước "đệm cache": chép `Cargo.toml` rồi build một chương trình giả để biên dịch xong toàn bộ thư viện, sau đó mới chép `src/`. Nhờ vậy sửa code chỉ phải biên dịch lại phần của mình, không biên dịch lại axum/tokio/sqlx. **Giai đoạn chạy** dùng `debian:bookworm-slim`, chỉ chứa file nhị phân đã build, chạy bằng user thường `app` (không phải root). Có `HEALTHCHECK` gọi `GET /health`. Cổng cố định **3000** vì `Frontend/nginx.conf` proxy tới `backend:3000`. Thư mục `/app/uploads` được tạo sẵn và gán quyền cho `app` để ghi ảnh upload. |
| `Backend/.dockerignore` | Không đưa `target/` (rất nặng), `.env` (bí mật), `uploads/` vào ngữ cảnh build. |
| `compose.yaml` | Định nghĩa 3 service, **không mở cổng nào**. `backend` nhận biến môi trường (JWT, CORS, `DB_*`) từ `.env`; `DB_HOST` cố định là `db` (tên service, Docker tự phân giải). Biến bắt buộc dùng cú pháp `${BIEN:?thông báo}` nên thiếu thì compose dừng ngay kèm lời nhắc, không chạy với giá trị rỗng. Volume `uploads` giữ ảnh, volume `db_data` giữ dữ liệu MariaDB. `db` có healthcheck. |
| `compose.override.yaml` | Docker Compose tự đọc khi chạy `docker compose up`. Mở cổng cho máy local: giao diện `http://localhost:${WEB_PORT:-80}`, API `http://localhost:3000` (Bruno dùng cổng này), MariaDB `127.0.0.1:3306` (chỉ máy bạn). |
| `compose.prod.yaml` | Override cho server: gắn nhãn Traefik cho `frontend` (Host lấy từ `APP_HOST`) và nối `frontend` vào mạng `komodo_default`. Chỉ `frontend` ra ngoài; `backend` và `db` ở mạng nội bộ. Chạy kèm: `docker compose -f compose.yaml -f compose.prod.yaml up -d --build` (không kèm file override local). |
| `.env.example` | Mẫu biến môi trường (bảng bên dưới). |

Biến trong `.env`:

| Biến | Dùng cho | Ghi chú |
|---|---|---|
| `DB_ROOT_PASSWORD` | Mật khẩu root MariaDB | chỉ db dùng |
| `DB_USER`, `DB_PASSWORD`, `DB_NAME` | Tài khoản và tên database | vừa tạo DB (service `db`), vừa đưa cho backend (`config.db_*`) |
| `JWT_SECRET` | Khóa ký token | ≥ 16 ký tự, **bắt buộc đổi khi triển khai thật** (`openssl rand -hex 32`) |
| `JWT_TTL_HOURS`, `CORS_ALLOWED_ORIGINS`, `RUST_LOG` | Cấu hình backend | có giá trị mặc định |
| `WEB_PORT` | Cổng giao diện trên máy local | mặc định 80 |
| `APP_HOST` | Tên miền khi chạy trên server | chỉ `compose.prod.yaml` cần |

**Việc cần làm khi gắn DB thật:** mở `compose.yaml`, ở service `backend` bỏ dấu `#` của 3 dòng `depends_on: db: condition: service_healthy`.
Hiện đang để tắt có chủ đích: lúc này backend chạy mock và không cần DB, nếu `init.sql` có lỗi thì MariaDB không "healthy" và demo mock sẽ không khởi động được.

Lệnh thường dùng:

```bash
docker compose up --build            # chạy cả 3 service (lần đầu build Rust mất vài phút)
docker compose logs -f backend       # xem log backend
docker compose down                  # dừng, GIỮ dữ liệu
docker compose down -v               # dừng và XÓA dữ liệu (db_data, uploads)
```
Ghi chú CI: `.woodpecker/test-publish.yaml` build `Backend/Dockerfile` cho cả `linux/amd64` và `linux/arm64`; bản arm64 dựng bằng giả lập nên chậm. Dockerfile đã được viết cho đúng ngữ cảnh `Backend/` mà pipeline đang dùng.

### 7.7 Bước 7 – `init.sql` và dữ liệu seed

`Database/init scripts/init.sql` **đã được điền sẵn** (6 bảng ở mục 2.7 + seed ở mục 6), không còn chỗ trống phải dán.

- **Mật khẩu mẫu `123456`** của 4 tài khoản đã được thay bằng một hash Argon2id thật (`m=19456, t=2, p=1`, trùng tham số mặc định của crate `argon2` mà backend dùng).
  Backend đọc tham số từ chính chuỗi hash khi kiểm tra mật khẩu, nên hash này hợp lệ. Hash được tạo bằng thư viện khác (không phải bằng Rust) vì lúc soạn không có Rust, nên **hãy xác nhận bằng cách đăng nhập `admin@pc.vn` / `123456` sau khi gắn DB**; nếu bị từ chối, sinh lại bằng
  `cd Backend && cargo run -- hash-password 123456` rồi thay chuỗi trong dòng `SET @pw = '...'` của `init.sql` (và chạy lại 7.1 với `down -v`).
- Mock và `init.sql` có cùng dữ liệu, nên kết quả kiểm chứng giống nhau: `GET /api/admin/stats` = `{products: 7, users: 4, orders: 7, revenue: 118370000}` (mục 6).
- Hash trong file là dữ liệu demo, không phải bí mật. Khi triển khai thật phải đổi mật khẩu các tài khoản này (hoặc xóa seed).

### 7.8 Dọn dẹp sau khi gắn DB – dòng log tài khoản mẫu

Trước đây `main.rs` in email và mật khẩu tài khoản mẫu ra log mỗi lần khởi động (giả định A26). Điều này chỉ hợp lý với mock, và nếu quên xóa thì khi chạy DB thật log vẫn nói "đang chạy với dữ liệu mock" (sai) và lộ mật khẩu demo.
Đã xử lý bằng cách **chuyển dòng log vào chính `build_repositories()`**, ngay sau khi tạo mock. Hệ quả: khi bạn thay thân hàm ở 7.5, dòng log biến mất cùng mock, không phải nhớ xóa riêng.
Sau khi thay, chỉ cần bỏ 3 hằng `SEED_*` khỏi câu lệnh `use crate::repositories::mock::{...}` ở đầu `app_state.rs`.
Giữ lại lệnh con `hash-password` của `main.rs` vì vẫn cần để tạo hash seed.

### 7.9 Lỗi thường gặp

| Hiện tượng | Nguyên nhân thường gặp | Cách xử lý |
|---|---|---|
| `docker compose up` báo `Hãy đặt DB_PASSWORD trong file .env` | Chưa tạo `.env` | `cp .env.example .env` |
| Sửa `init.sql` nhưng DB không đổi | Volume `db_data` đã có dữ liệu | `docker compose down -v` rồi `up` lại (mất dữ liệu cũ) |
| `init.sql` báo lỗi khi DB khởi tạo | Lỗi cú pháp/thứ tự bảng | `docker compose logs db`; sửa rồi `down -v` |
| Backend log "database chưa sẵn sàng, sẽ thử lại" vài lần rồi chạy | Bình thường: DB khởi động chậm hơn backend | Không cần làm gì; bật `depends_on` để gọn hơn |
| Backend thoát sau 10 lần thử | Sai `DB_USER`/`DB_PASSWORD`/`DB_NAME`, hoặc `init.sql` làm DB không lên | Kiểm tra `.env` và `docker compose logs db` |
| `Access denied for user` | `.env` đổi mật khẩu sau khi DB đã khởi tạo (mật khẩu cũ nằm trong volume) | `down -v` hoặc đổi mật khẩu trong DB |
| Frontend báo lỗi mạng / 502 | Backend chưa chạy hoặc không nghe cổng 3000 | `docker compose logs backend`; không đổi `APP_PORT` trong compose |
| Upload ảnh báo lỗi ghi file | Quyền thư mục `/app/uploads` | Dùng volume `uploads` như compose (đã gán quyền cho user `app`) |

---

## 8. Checklist nghiệm thu

- [ ] `init.sql` chạy sạch trên MariaDB trống, tạo đủ 6 bảng, mọi khóa/ràng buộc đúng tên như mục 2.7.
- [ ] 5 struct repository thật implement đủ **31 hàm** của 5 trait (users 7, categories 5, products 7, cart 5, orders 7), biên dịch không lỗi.
- [ ] `cargo fmt`, `cargo clippy -- -D warnings`, `cargo build` sạch sau khi tích hợp.
- [ ] Chỉ `app_state.rs` (`build_repositories`), `db.rs`, `repositories/mod.rs`, `Cargo.toml` bị sửa trong code cũ (`lib.rs` đã khai báo sẵn `pub mod db;`); **không** sửa services/controllers.
- [ ] `docker compose up --build` từ repo sạch (đã có `.env`) dựng được cả 3 service; `docker compose down -v && docker compose up` nạp lại được seed.
- [ ] Khởi động backend **trước** khi DB sẵn sàng (ví dụ tắt `depends_on`) vẫn tự kết nối được sau vài lần thử lại.
- [ ] Log khởi động **không** còn dòng "dữ liệu mock in-memory" và không in mật khẩu mẫu.
- [ ] `Cargo.lock` đã được commit sau lần build đầu tiên có `sqlx`.
- [ ] Sau khi nạp seed, đăng nhập được bằng `admin@pc.vn` / `123456`; `GET /api/admin/stats` = `{products:7, users:4, orders:7, revenue:118370000}`.
- [ ] Đăng ký email đã có (kể cả khác hoa/thường) → 409; tạo danh mục trùng tên → 409.
- [ ] Tạo sản phẩm với `categoryId` không tồn tại → 400.
- [ ] Xóa danh mục → sản phẩm của nó còn nguyên, `categoryId = null`.
- [ ] Xóa sản phẩm đã nằm trong đơn (mọi sản phẩm seed đều thuộc ít nhất một đơn) → 409. Tạo sản phẩm mới, thêm vào giỏ rồi xóa → thành công và giỏ tự mất dòng đó.
- [ ] Xóa `khach1@pc.vn` (có đơn) → 409; xóa `khach3@pc.vn` → thành công.
- [ ] Đặt hàng từ giỏ: giỏ bị xóa, `total` đúng, `status = pending`; sau đó đổi giá sản phẩm thì đơn cũ **không** đổi giá.
- [ ] Giả lập lỗi giữa chừng khi tạo đơn (ví dụ ép lỗi ở bước 4) → không có đơn nào được tạo và giỏ vẫn còn (transaction rollback).
- [ ] `GET /api/products`: sản phẩm ghim đứng đầu; `sort=priceAsc/priceDesc/new` đúng; `q` tìm được trong cả `name` và `specs`.
- [ ] Hai request đặt hàng đồng thời của cùng một người không tạo ra hai đơn từ cùng một giỏ.
