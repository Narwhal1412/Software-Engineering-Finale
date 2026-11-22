# Backend – Computer Shop REST API

REST API for the computer shop, written in Rust with [axum](https://github.com/tokio-rs/axum) 0.7 and tokio. It stores data in MariaDB through sqlx and serves product images from the local upload directory.

## What Works

**Authentication** (`/api/auth`)
- Register and log in; returns a JWT and the user object.
- Passwords hashed with Argon2.

**Current user** (`/api/me`, requires login)
- Get and update profile (display name, phone, address).

**Catalog** (`/api/products`, `/api/categories`)
- Public: list products with search (`q`, matches name and specs), category filter, and sort (`new`, `priceAsc`, `priceDesc`). Pinned products are listed first. Every product has a `stock` field (units left in the warehouse).
- Admin: create, update, delete products; pin or unpin products. `stock` is set by the admin on create/update (integer from 0 to 1,000,000; a missing value is treated as 0 = sold out).
- Admin: create, rename, delete categories. Deleting a category keeps its products and sets their category to `null`.

**Policies and offers** (`/api/policies`)
- Public: `GET /api/policies` lists every policy (warranty, returns, delivery...), sorted by `sortOrder` then `id`. The storefront shows them in the product detail dialog.
- Admin: create (`POST`), update (`PUT /:id`), delete (`DELETE /:id`). A policy is `{id, icon, title, content, categoryId, sortOrder}`.
- `categoryId = null` means the policy applies to every product; a number means it applies only to products of that category (for example a different warranty per category). Deleting a category also deletes the policies that belong only to it.
- `icon` must be one of `shield`, `refresh`, `truck`, `check`, `card`, `gift`, `phone`, `box` (same names as the frontend icons). `title` is up to 150 characters, `content` up to 500, `sortOrder` from 0 to 9999.

**Uploads** (`/api/uploads`, admin)
- Accepts JPG, PNG, WebP up to 2 MB; files are served from `/uploads/...`.

**Cart** (`/api/cart`, requires login)
- Get the cart, add items, change quantity (1 to 99), remove items. Each change returns the updated cart.
- **Stock limit:** the quantity of a product in the cart can never exceed its `stock` (adding to an existing line counts what is already in the cart). Going over, or adding a sold-out product, returns `409` with a message such as `Sản phẩm "X" chỉ còn 3 trong kho` / `Sản phẩm "X" đã hết hàng`.

**Orders** (`/api/orders`)
- Place an order from the cart. Prices are read from the database and stored as a snapshot; the cart is cleared; the status starts as `pending`.
- **Stock check:** placing an order re-checks the stock of every product and subtracts the ordered quantities in the same database transaction (rows are locked with `FOR UPDATE`, so two customers cannot buy the last item at once). If any product has too little stock the whole order is rejected with `409`, and nothing changes (stock and cart stay as they were).
- **Stock is returned** when an order becomes `cancelled` (by the customer or by an admin), exactly once per order.
- List the current user's orders.
- Customer: cancel their own order with `PATCH /api/orders/:id/cancel`, only while it is `pending` (`404` for someone else's order, `409` once it has moved past `pending`).
- Admin: list all orders with the customer's email, update the status (`pending`, `confirmed`, `shipped`, `completed`, `cancelled`).

**Administration** (`/api/admin`, admin only)
- `/admin/stats`: product, user, order counts and revenue (cancelled orders excluded).
- `/admin/users`: list members; delete a member (admin accounts cannot be deleted).

**Health check**
- `GET /health` returns `{"status":"ok"}`. Used by the Docker HEALTHCHECK.

The full request and response contract is in [docs/API.md](../docs/API.md).

## Layout

```
src/
├── main.rs            # Entry point; also the `hash-password` helper command
├── lib.rs             # Library root used by the binary and tests
├── app.rs             # Router assembly, /health, /uploads static files, middleware
├── app_state.rs       # Shared state; build_repositories() selects MariaDB
├── config.rs          # Reads configuration from environment variables
├── db.rs              # MariaDB connection pool with retry
├── controllers/       # HTTP handlers (auth, products, cart, orders, users, stats, uploads…)
├── services/          # Business logic and authorization checks
├── repositories/      # Traits; mysql/ (production) and mock/ (in-memory, tests)
├── models/            # Domain types and limits
├── dtos/              # Request and response shapes
└── middleware/        # Auth/token checks, CORS, request logging, password hashing
```

## Database Note (upgrading an existing database)

The `products` table gained a `stock` column (`INT NOT NULL DEFAULT 0`, `CHECK (stock >= 0)`). `init.sql` only runs on the first start of an empty database, so an existing database must be recreated (`docker compose down -v`, then `docker compose up --build`; this deletes its data) or migrated by hand:

```sql
ALTER TABLE products
  ADD COLUMN stock INT NOT NULL DEFAULT 0 AFTER price,
  ADD CONSTRAINT ck_products_stock CHECK (stock >= 0);
UPDATE products SET stock = 10;  -- optional: give existing products some stock
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `APP_PORT` | `3000` | Port to listen on |
| `JWT_SECRET` | dev value | Signing key, at least 16 characters. A warning is logged if the default is used. |
| `JWT_TTL_HOURS` | `24` | Token lifetime |
| `UPLOAD_DIR` | `uploads` | Directory for uploaded images |
| `CORS_ALLOWED_ORIGINS` | `*` | Comma-separated origins; `*` allows all |
| `DB_HOST` | `localhost` | MariaDB host |
| `DB_PORT` | `3306` | MariaDB port |
| `DB_USER` | `app` | Database user |
| `DB_PASSWORD` | empty | Database password |
| `DB_NAME` | `appdb` | Database name |
| `RUST_LOG` | – | Log filter, e.g. `info,tower_http=debug` |

## Running

**Docker:** the backend is started by the root `compose.yaml`; see the [root README](../README.md).

**Manually:** start MariaDB with `Database/init scripts/init.sql` loaded, set the variables above (or put them in `Backend/.env`), then:

```bash
cd Backend
cargo run
```

Helper command to generate a password hash for seed data:

```bash
cargo run -- hash-password <password>
```

## Tests

Backend tests are run through the root test script, which starts a MariaDB container and runs `cargo test` against it:

```bash
./run-tests.sh          # Linux / macOS / Git Bash
.\run-tests.ps1         # Windows PowerShell
```

Test files:
- `tests/api_tests.rs`: HTTP-level tests against the router (includes the cart/order stock limit).
- `tests/logic_tests.rs`: pure business-rule tests.
- `tests/mysql_repository_tests.rs`: repository tests against a real MariaDB (includes stock decrement, oversell rejection and restore on cancel).
- `src/services/service_tests.rs` and `src/repositories/mock/tests.rs`: service and in-memory repository tests (stock limit, atomic rejection, restore on cancel). These need no database.

## Docker Image

`Backend/Dockerfile` builds a release binary in a Rust stage and runs it as a non-root user on `debian:bookworm-slim`. It listens on port 3000 and exposes a health check on `/health`.
