-- =============================================================================
-- init.sql: tạo bảng + nạp dữ liệu mẫu cho MariaDB (>= 10.11).
-- Chạy TỰ ĐỘNG bởi image mariadb ở LẦN KHỞI TẠO ĐẦU TIÊN (khi thư mục dữ liệu còn trống),
-- nhờ compose.yaml mount thư mục "Database/init scripts" vào /docker-entrypoint-initdb.d.
-- Muốn chạy lại từ đầu:  docker compose down -v  (xóa volume db_data) rồi  docker compose up.
-- Nguồn: docs/DATABASE_SPEC.md mục 2.7 (bảng) và mục 6 (seed). Sửa ở đó trước, rồi đồng bộ về file này.
-- Mật khẩu của 4 tài khoản mẫu: 123456 (CHỈ để demo; hash Argon2id bên dưới).
-- =============================================================================

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

-- =============================================================================
-- DỮ LIỆU MẪU (khớp với mock trong Backend/src/repositories/mock/seed.rs)
-- Hash dưới đây là Argon2id (m=19456, t=2, p=1 = mặc định của crate argon2) của "123456".
-- Muốn đổi mật khẩu: trong thư mục Backend/ chạy  cargo run -- hash-password <mật khẩu>
-- rồi thay chuỗi trong dòng SET @pw bên dưới.
-- =============================================================================
SET @pw = '$argon2id$v=19$m=19456,t=2,p=1$nVQy4lT8yE41j9Vax2mfYA$hKPd1BMIvyViZycuaiulSTXoAGIbKAVL0aLCyDcWEOs';

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
