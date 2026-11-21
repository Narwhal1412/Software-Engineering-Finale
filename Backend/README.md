# Backend – Computer Shop REST API

REST API for the computer shop, written in Rust with [axum](https://github.com/tokio-rs/axum) 0.7 and tokio. It stores data in MariaDB through sqlx and serves product images from the local upload directory.

## What Works

**Authentication** (`/api/auth`)
- Register and log in; returns a JWT and the user object.
- Passwords hashed with Argon2.

**Current user** (`/api/me`, requires login)
- Get and update profile (display name, phone, address).

**Catalog** (`/api/products`, `/api/categories`)
- Public: list products with search (`q`, matches name and specs), category filter, and sort (`new`, `priceAsc`, `priceDesc`). Pinned products are listed first.
- Admin: create, update, delete products; pin or unpin products.
- Admin: create, rename, delete categories. Deleting a category keeps its products and sets their category to `null`.

**Uploads** (`/api/uploads`, admin)
- Accepts JPG, PNG, WebP up to 2 MB; files are served from `/uploads/...`.

**Cart** (`/api/cart`, requires login)
- Get the cart, add items, change quantity (minimum 1), remove items. Each change returns the updated cart.

**Orders** (`/api/orders`)
- Place an order from the cart. Prices are read from the database and stored as a snapshot; the cart is cleared; the status starts as `pending`.
- List the current user's orders.
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
- `tests/api_tests.rs`: HTTP-level tests against the router with the in-memory repositories.
- `tests/logic_tests.rs`: service and business-rule tests.
- `tests/mysql_repository_tests.rs`: repository tests against a real MariaDB.

## Docker Image

`Backend/Dockerfile` builds a release binary in a Rust stage and runs it as a non-root user on `debian:bookworm-slim`. It listens on port 3000 and exposes a health check on `/health`.
