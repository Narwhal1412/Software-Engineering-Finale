# Backend
Rust API service. Organized in layers: Controllers -> Services -> Repositories

## Structure
```
Backend/
├── Dockerfile
├── Cargo.toml
└── src/
    ├── main.rs                  # starts server, wires routes
    ├── db.rs                    # database connection pool
    ├── controllers/             # HTTP routes + request validation
    │   ├── mod.rs
    │   ├── product_controller.rs
    │   ├── user_controller.rs
    │   └── order_controller.rs
    ├── services/                # Business Logic layer
    │   ├── mod.rs
    │   ├── product_service.rs
    │   ├── user_service.rs
    │   └── order_service.rs
    └── repositories/            # structs + SQL queries (Data Access layer)
        ├── mod.rs
        ├── product.rs
        ├── user.rs
        └── order.rs
```
## Request flow
`Frontend -> controller -> service -> repository -> database`
