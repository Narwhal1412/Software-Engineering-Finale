# Backend – Computer Shop API

A REST API for a computer-store management website, written in **Rust (axum + tokio)** with a 3-layer architecture:

```
Frontend ──HTTP/JSON──▶ Controllers ──▶ Services ──▶ Repositories ──▶ Database
                        (Presentation)  (Business     (Data Access
                                         Logic)        Layer – DAL)
```

> **Current status: the backend runs on a FAKE database (in-memory mock).** Data lives in RAM, is reloaded from the sample data every time the server starts, and is **lost when the server stops**.
> Instructions for removing the mock and plugging in a real MariaDB: [`../docs/DATABASE_SPEC.md`](../docs/DATABASE_SPEC.md), **section 7** (see [Removing the mock and plugging in a real database](#removing-the-mock-and-plugging-in-a-real-database)).

## Table of contents

1. [Quick start](#quick-start)
2. [Mock accounts and data](#mock-accounts-and-data)
3. [Directory structure](#directory-structure)
4. [Lifecycle of a request](#lifecycle-of-a-request)
5. [Endpoint list](#endpoint-list)
6. [Environment variables](#environment-variables)
7. [Tests](#tests)
8. [Removing the mock and plugging in a real database](#removing-the-mock-and-plugging-in-a-real-database)
9. [Related documents](#related-documents)

---

## Quick start

**Option 1 – Docker (frontend + backend + MariaDB)**, from the repo root:

```bash
cp .env.example .env          # then edit the values (DB_*, JWT_SECRET)
docker compose up --build
```
UI: `http://localhost` · API: `http://localhost:3000` (see `compose.override.yaml`).
At this point the backend still uses the mock; the MariaDB service runs alongside it so you can verify `init.sql` before plugging it in for real.

**Option 2 – Run directly with Cargo** (requires the Rust toolchain), inside `Backend/`:

```bash
cp .env.example .env          # optional; without this file it still runs with default values
cargo run
curl http://localhost:3000/health      # -> {"status":"ok"}
```

The first thing to do on a machine with Rust (the code has never been compiled):
`cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test && cargo run`.

---

## Mock accounts and data

The password of **all 4 accounts is `123456`** (demo only). When running on the mock, these emails are also printed to the log at startup.

| Email | Name | Role | What it has in the sample data | Use it to try |
|---|---|---|---|---|
| `admin@pc.vn` | Quản trị (Administrator) | `admin` | – | Every `/api/admin/*` endpoint, category/product management, uploads |
| `khach1@pc.vn` | Nguyễn Văn An | `user` | 4 orders + 2 cart lines | Cart, checkout; **cannot be deleted** (has orders → 409) |
| `khach2@pc.vn` | Trần Thị Bình | `user` | 3 orders | Viewing another user's orders |
| `khach3@pc.vn` | Lê Hoàng Cường | `user` | no orders, no cart | **Successful member deletion** (no orders) |

The rest of the sample data (matches `Database/init scripts/init.sql` line by line):

- **5 categories:** CPU, Card đồ họa (Graphics cards), RAM, Ổ cứng (Storage), PC lắp sẵn (Prebuilt PCs).
- **7 products** (Ryzen 7 7800X3D is pinned to the top, Core i5-14600K, RTX 4070 Super, RX 7800 XT, Kingston Fury 32GB, Samsung 980 Pro 1TB, PC Gaming Forge R7). Every sample product appears in at least one order, so none of them **can be deleted** (409); to try deleting, create a new product.
- **7 orders** (covering all statuses: `pending`, `confirmed`, `shipped`, `completed`, `cancelled`).
- **Check figures:** `GET /api/admin/stats` → `{"products":7,"users":4,"orders":7,"revenue":118370000}` (revenue excludes cancelled orders).

Things to know about the mock: data is lost when the server stops; search (`q`) is **diacritic-sensitive**, whereas MariaDB is not; all important constraints (foreign keys, duplicate names, no deleting rows that are still referenced) are still simulated exactly as a real database would enforce them.

---

## Directory structure

```
Backend/
├── Cargo.toml              # Package + dependency declarations (versions pinned with "=x.y.z"); lib + bin share the name computer_shop_backend
├── .env.example            # Environment variable template when running with cargo (copy to .env)
├── .dockerignore           # Excludes target/, uploads/, .env from the Docker build context
├── Dockerfile              # 2-stage build (compile Rust -> debian-slim image running as a normal user, port 3000, healthcheck)
├── README.md               # (this file)
├── src/
│   ├── main.rs             # Entry point: load .env/config, create the upload directory, build repositories + AppState, run the server,
│   │                       #   graceful shutdown on Ctrl+C/SIGTERM. Has a `hash-password <pw>` subcommand to generate Argon2 hashes for seed data
│   ├── lib.rs              # Module declarations; the lib/bin split lets integration tests import the code
│   ├── app.rs              # Assembles the router: /health, all of /api, serves images at /uploads, adds CORS + request logging
│   ├── app_state.rs        # Shared AppState (config + repositories + services). The `build_repositories()` function is the ONLY PLACE
│   │                       #   that chooses mock or real DB  ⟵ where the mock gets removed
│   ├── config.rs           # Reads environment variables into the `Config` struct (port, JWT, upload, CORS, DB_*), with default values
│   ├── error.rs            # `AppError` + how it is turned into an HTTP `{message}` response (codes 400/401/403/404/409/413/415/500)
│   ├── db.rs               # (Empty/comments only) where the MariaDB connection goes when the real DB is plugged in
│   │
│   ├── controllers/        # PRESENTATION LAYER: axum handlers; only take params/body, call a service, return JSON. No business logic
│   │   ├── mod.rs              # `build_router()`: gathers every route under the /api prefix
│   │   ├── api_json.rs         # JSON extractor: a syntactically invalid body also returns a `{message}` error in Vietnamese
│   │   ├── auth_controller.rs      # POST /auth/register, /auth/login
│   │   ├── user_controller.rs      # GET/PUT /me; GET /admin/users; DELETE /admin/users/:id
│   │   ├── category_controller.rs  # Category CRUD
│   │   ├── product_controller.rs   # Product CRUD, search/sort, pinning
│   │   ├── upload_controller.rs    # POST /uploads (multipart)
│   │   ├── cart_controller.rs      # Shopping cart
│   │   ├── order_controller.rs     # Checkout, my orders, order management (admin)
│   │   └── stats_controller.rs     # GET /admin/stats
│   │
│   ├── dtos/               # Request/response structs matching every field in docs/API.md (JSON camelCase)
│   │   ├── mod.rs
│   │   ├── auth_dto.rs  user_dto.rs  category_dto.rs  product_dto.rs
│   │   └── cart_dto.rs  order_dto.rs  stats_dto.rs  upload_dto.rs
│   │
│   ├── services/           # BUSINESS LOGIC LAYER: validation, authorization, business rules. Knows only the repository traits
│   │   ├── mod.rs              # `Services`: groups the services, built from the repositories
│   │   ├── validation.rs       # Shared validation functions (email, password, phone number, price...), messages in Vietnamese
│   │   ├── authz.rs            # Authorization rules (`require_admin`)
│   │   ├── auth_service.rs  user_service.rs  category_service.rs  product_service.rs
│   │   ├── cart_service.rs  order_service.rs  stats_service.rs  upload_service.rs
│   │   └── service_tests.rs    # Unit tests for the services (run on an empty mock)
│   │
│   ├── repositories/       # DATA ACCESS LAYER (DAL): data-access traits + their implementations
│   │   ├── mod.rs              # Module declarations, re-exports the traits
│   │   ├── repo_error.rs       # `RepoError` (NotFound, Duplicate, InvalidReference, InUse, Constraint, Internal), independent of the DB type
│   │   ├── user_repository.rs  category_repository.rs  product_repository.rs
│   │   ├── cart_repository.rs  order_repository.rs   # 5 traits = the contract a real DB must fulfil (31 functions)
│   │   └── mock/               # FAKE in-RAM implementation (what gets removed from build_repositories once a DB exists)
│   │       ├── mod.rs              # Exports the mock repositories + SEED_* constants
│   │       ├── store.rs            # `MockStore`: the shared "tables", with a single lock so cross-table constraints stay atomic
│   │       ├── constraints.rs      # Simulates column constraints (length, CHECK) like a real DB
│   │       ├── seed.rs             # Sample data (4 accounts, 5 categories, 7 products, 7 orders)
│   │       ├── user_mock_repository.rs  category_mock_repository.rs  product_mock_repository.rs
│   │       ├── cart_mock_repository.rs  order_mock_repository.rs
│   │       └── tests.rs            # Tests: the mock must behave like a real DB (foreign keys, duplicate names, RESTRICT...)
│   │
│   ├── models/             # Entities matching the DB tables (not used to return JSON directly)
│   │   ├── mod.rs
│   │   ├── user.rs  category.rs  product.rs  cart.rs  order.rs   # User, Category, Product, CartItem, Order + OrderItem
│   │   ├── enums.rs            # Role, OrderStatus, PaymentMethod (strings identical to the ENUM columns)
│   │   └── limits.rs           # Maximum column lengths (services validate against these constants)
│   │
│   └── middleware/         # Components shared by the Presentation layer
│       ├── mod.rs
│       ├── auth.rs             # `AuthUser` extractor (reads the Bearer token, re-fetches the real role from the DB) and `AdminUser` (currently NOT used:
│       │                       #   admin authorization is done in the service via `authz::require_admin`, see BACKEND_ASSUMPTIONS B4)
│       ├── token.rs            # JWT issuing/verification (HS256, default lifetime 24 hours)
│       ├── password.rs         # Argon2id password hashing/verification
│       ├── cors.rs             # CORS configuration driven by CORS_ALLOWED_ORIGINS
│       └── logging.rs          # `tracing` initialization and HTTP request logging
│
└── tests/
    └── api_tests.rs        # Integration tests: call the real router (with the mock seed) via tower::ServiceExt::oneshot
```

Outside the `Backend/` directory (but closely related):

| Location | Contents |
|---|---|
| `../docs/API.md` | API contract (paths, fields, `{message}` errors) |
| `../docs/DATABASE_SPEC.md` | DB specification + **guide to removing the mock and plugging in a real DB (section 7)** |
| `../docs/BACKEND_ASSUMPTIONS.md` | Every backend assumption/decision (A1.., B1..) that API.md does not state |
| `../docs/bruno/` | API test suite for Bruno (128 requests) + `bruno_coverage.md` |
| `../Database/init scripts/init.sql` | Creates 6 tables + seed data, runs automatically the first time MariaDB initializes |
| `../compose.yaml`, `compose.override.yaml`, `compose.prod.yaml` | Run the whole system with Docker (shared / local / server) |

---

## Lifecycle of a request

Example: `POST /api/orders` (checkout):

1. **`app.rs`** receives the request through the CORS + logging layers and routes it using `controllers::build_router()`.
2. **`order_controller.rs`** uses the `AuthUser` extractor (`middleware/auth.rs`) to authenticate the token, reads the body into `dtos::order_dto`, and calls `OrderService`.
3. **`order_service.rs`** validates (full name, address, payment method), then calls the `OrderRepository::create_from_cart` trait method.
4. **The repository** (the mock today, MariaDB tomorrow) creates the order from the cart, snapshots the current prices, clears the cart, and returns the `Order`.
5. Errors from the repository (`RepoError`) are converted by the service/`error.rs` into an HTTP status + a Vietnamese `{message}`; on success the DTO is serialized to JSON.

Because services only know the **traits**, switching from the mock to MariaDB does not touch steps 1–3.

---

## Endpoint list

Prefix `/api` (except `/health`). Field details: `docs/API.md`. Success/error codes: `docs/BACKEND_ASSUMPTIONS.md` (B1, B2).

| Group | Endpoint | Access |
|---|---|---|
| System | `GET /health` (outside `/api`) | public |
| Authentication | `POST /auth/register`, `POST /auth/login` | public |
| Profile | `GET /me`, `PUT /me` | logged in |
| Members | `GET /admin/users`, `DELETE /admin/users/:id` | admin |
| Categories | `GET /categories` (public); `POST`, `PUT /:id`, `DELETE /:id` | admin |
| Products | `GET /products` (`q`, `categoryId`, `sort=new\|priceAsc\|priceDesc`, public); `POST`, `PUT /:id`, `DELETE /:id`, `PATCH /:id/pin` | admin |
| Image upload | `POST /uploads` (multipart, field `file`, jpg/png/webp ≤ 2 MB) | admin |
| Cart | `GET /cart`, `POST /cart/items`, `PATCH /cart/items/:productId`, `DELETE /cart/items/:productId` | logged in |
| Orders | `POST /orders`, `GET /orders` | logged in |
| Order management | `GET /admin/orders`, `PATCH /admin/orders/:id` | admin |
| Statistics | `GET /admin/stats` | admin |

Conventions: 201 for `POST` that creates something, 204 for `DELETE`, 200 for everything else; errors are always `{message}`; 401 for a missing/invalid token, 403 for insufficient permission, 404 for not found, 409 for a duplicate or a row that is still referenced.

---

## Environment variables

Read in `src/config.rs`; templates in `.env.example` (running with cargo) and `../.env.example` (running with Docker).

| Variable | Meaning | Default |
|---|---|---|
| `APP_PORT` | Listening port. **Keep 3000** (the frontend's nginx/vite point to 3000) | `3000` |
| `JWT_SECRET` | JWT signing key, ≥ 16 characters. **Must be changed for a real deployment** | dev key (a warning is logged) |
| `JWT_TTL_HOURS` | Token lifetime (hours) | `24` |
| `UPLOAD_DIR` | Directory where uploaded images are stored, served at `/uploads/...` | `uploads` |
| `CORS_ALLOWED_ORIGINS` | Origins allowed to call the API, comma-separated; empty or `*` = all | all |
| `RUST_LOG` | Log level | `info,tower_http=debug` |
| `DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASSWORD`, `DB_NAME` | MariaDB connection details. **Not used while the mock is active** | `localhost`, `3306`, `app`, (empty), `appdb` |

---

## Tests

```bash
cargo test
```

- `src/repositories/mock/tests.rs`: checks that the mock simulates DB constraints correctly.
- `src/services/service_tests.rs`: business-logic unit tests on an empty mock.
- `tests/api_tests.rs`: integration tests that call the real router with the seed data.
- Manual/automated API testing with Bruno: `../docs/bruno/` (see `bruno_coverage.md`; base URL `http://localhost:3000`).

These tests use the mock, so they run without a database; keep the mock around after the real DB is plugged in.

---

## Removing the mock and plugging in a real database

**All instructions are in [`../docs/DATABASE_SPEC.md`](../docs/DATABASE_SPEC.md), section 7** (plus section 5 – the 31 functions of the 5 traits, and section 8 – the acceptance checklist). Summary:

| # | Task | Where to edit |
|---|---|---|
| 1 | Set up MariaDB, verify `init.sql` (`docker compose up -d db`) | – (already provided) |
| 2 | Add `sqlx = "=0.8.2"` | `Cargo.toml` |
| 3 | Write the connection + retry while the DB is not ready | `src/db.rs` (already declared in `lib.rs`) |
| 4 | Write the 5 MariaDB repositories | new directory `src/repositories/mysql/` + `repositories/mod.rs` |
| 5 | **Change the body of `build_repositories()`** (the only place that chooses mock/DB) | `src/app_state.rs` |
| 6 | Enable the backend's `depends_on: db` | `../compose.yaml` |

No changes are needed in the services, controllers, or middleware. The "running on mock data" log line lives inside `build_repositories()`, so it disappears automatically when the mock is replaced.

---

## Related documents

- [`../docs/API.md`](../docs/API.md) – API contract
- [`../docs/DATABASE_SPEC.md`](../docs/DATABASE_SPEC.md) – database specification and guide to plugging in the DB
- [`../docs/BACKEND_ASSUMPTIONS.md`](../docs/BACKEND_ASSUMPTIONS.md) – assumptions/decisions
- [`../docs/bruno/bruno_coverage.md`](../docs/bruno/bruno_coverage.md) – endpoint → Bruno test coverage table
