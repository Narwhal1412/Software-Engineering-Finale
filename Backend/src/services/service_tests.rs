//! Unit test cho tầng service, chạy trên repository mock rỗng (không seed) để mỗi test tự dựng dữ liệu.

use std::sync::Arc;

use super::Services;
use crate::app_state::Repositories;
use crate::config::Config;
use crate::dtos::auth_dto::{LoginRequest, RegisterRequest};
use crate::dtos::cart_dto::{AddCartItemRequest, SetQuantityRequest};
use crate::dtos::category_dto::CategoryRequest;
use crate::dtos::order_dto::{CreateOrderRequest, UpdateOrderStatusRequest};
use crate::dtos::policy_dto::PolicyRequest;
use crate::dtos::product_dto::{ProductQueryParams, ProductRequest};
use crate::dtos::user_dto::UpdateProfileRequest;
use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::enums::{OrderStatus, Role};
use crate::models::product::ProductInput;
use crate::models::user::NewUser;
use crate::repositories::mock::{
    InMemoryCartRepository, InMemoryCategoryRepository, InMemoryOrderRepository,
    InMemoryPolicyRepository, InMemoryProductRepository, InMemoryUserRepository, MockStore,
};

/// Dựng repository mock rỗng và service trên đó.
fn fixture() -> (Repositories, Services) {
    let store = MockStore::empty();
    let repos = Repositories {
        users: Arc::new(InMemoryUserRepository::new(store.clone())),
        categories: Arc::new(InMemoryCategoryRepository::new(store.clone())),
        products: Arc::new(InMemoryProductRepository::new(store.clone())),
        carts: Arc::new(InMemoryCartRepository::new(store.clone())),
        orders: Arc::new(InMemoryOrderRepository::new(store.clone())),
        policies: Arc::new(InMemoryPolicyRepository::new(store)),
    };
    let config = Config {
        port: 0,
        jwt_secret: "test-secret-0123456789".to_owned(),
        jwt_ttl_hours: 1,
        upload_dir: std::env::temp_dir()
            .join("computer_shop_service_tests")
            .to_string_lossy()
            .into_owned(),
        cors_allowed_origins: Vec::new(),
        db_host: String::new(),
        db_port: 0,
        db_user: String::new(),
        db_password: String::new(),
        db_name: String::new(),
    };
    let services = Services::new(&Arc::new(config), &repos);
    (repos, services)
}

async fn make_user(repos: &Repositories, email: &str, role: Role) -> AuthUser {
    let user = repos
        .users
        .create(NewUser {
            email: email.to_owned(),
            display_name: "Test".to_owned(),
            password_hash: "x".to_owned(),
            phone: String::new(),
            address: String::new(),
            role,
        })
        .await
        .expect("tạo user test");
    AuthUser {
        id: user.id,
        role: user.role,
    }
}

async fn make_product(repos: &Repositories, name: &str, price: i64) -> i64 {
    repos
        .products
        .create(ProductInput {
            name: name.to_owned(),
            specs: "specs".to_owned(),
            price,
            stock: 1000,
            category_id: None,
            image_url: None,
        })
        .await
        .expect("tạo sản phẩm test")
        .id
}

fn register_request(email: &str, password: &str, confirm: &str) -> RegisterRequest {
    RegisterRequest {
        email: email.to_owned(),
        display_name: "Nguyen Van A".to_owned(),
        password: password.to_owned(),
        confirm_password: confirm.to_owned(),
    }
}

fn product_request(name: &str, price: i64) -> ProductRequest {
    ProductRequest {
        name: name.to_owned(),
        specs: String::new(),
        price,
        stock: 10,
        category_id: None,
        image_url: None,
    }
}

fn order_request(phone: &str, payment: &str) -> CreateOrderRequest {
    CreateOrderRequest {
        full_name: "Nguyen Van A".to_owned(),
        phone: phone.to_owned(),
        address: "HCM".to_owned(),
        payment_method: payment.to_owned(),
    }
}

fn add_request(product_id: i64, quantity: Option<i32>) -> AddCartItemRequest {
    AddCartItemRequest {
        product_id,
        quantity,
    }
}

fn status_request(status: &str) -> UpdateOrderStatusRequest {
    UpdateOrderStatusRequest {
        status: status.to_owned(),
    }
}

// ---------- AuthService ----------

#[tokio::test]
async fn register_creates_customer_and_returns_token() {
    let (_, services) = fixture();
    let response = services
        .auth
        .register(register_request("a@x.vn", "secret1", "secret1"))
        .await
        .expect("đăng ký");
    assert!(!response.token.is_empty());
    assert_eq!(response.user.role, Role::User);
    assert_eq!(response.user.email, "a@x.vn");
}

#[tokio::test]
async fn register_rejects_invalid_input() {
    let (_, services) = fixture();
    let cases = [
        register_request("", "secret1", "secret1"),
        register_request("not-an-email", "secret1", "secret1"),
        register_request("a@x.vn", "123", "123"),
        register_request("a@x.vn", "secret1", "different"),
    ];
    for case in cases {
        let err = services.auth.register(case).await.unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)), "{err}");
    }
}

#[tokio::test]
async fn register_rejects_duplicate_email() {
    let (_, services) = fixture();
    services
        .auth
        .register(register_request("a@x.vn", "secret1", "secret1"))
        .await
        .expect("đăng ký lần 1");
    let err = services
        .auth
        .register(register_request("A@X.vn", "secret1", "secret1"))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
}

#[tokio::test]
async fn login_succeeds_only_with_correct_password() {
    let (_, services) = fixture();
    services
        .auth
        .register(register_request("a@x.vn", "secret1", "secret1"))
        .await
        .expect("đăng ký");

    let login = |email: &str, password: &str| LoginRequest {
        email: email.to_owned(),
        password: password.to_owned(),
    };
    assert!(services.auth.login(login("a@x.vn", "secret1")).await.is_ok());
    for bad in [login("a@x.vn", "wrong"), login("nobody@x.vn", "secret1")] {
        let err = services.auth.login(bad).await.unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}

// ---------- UserService ----------

#[tokio::test]
async fn update_me_validates_and_saves_profile() {
    let (repos, services) = fixture();
    let user = make_user(&repos, "a@x.vn", Role::User).await;
    let request = |phone: &str| UpdateProfileRequest {
        display_name: Some("Tên mới".to_owned()),
        phone: Some(phone.to_owned()),
        address: Some("HCM".to_owned()),
    };

    let err = services.users.update_me(user, request("12ab")).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));

    let updated = services
        .users
        .update_me(user, request("0901234567"))
        .await
        .expect("cập nhật");
    assert_eq!(updated.display_name, "Tên mới");
    assert_eq!(updated.phone, "0901234567");
}

#[tokio::test]
async fn admin_user_management_enforces_rules() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let customer = make_user(&repos, "c@x.vn", Role::User).await;

    let err = services.users.list_users(customer).await.unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
    let err = services.users.delete_user(admin, admin.id).await.unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
    let err = services.users.delete_user(admin, 999).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));

    services
        .users
        .delete_user(admin, customer.id)
        .await
        .expect("xóa khách");
    assert_eq!(services.users.list_users(admin).await.expect("list").len(), 1);
}

// ---------- CategoryService ----------

#[tokio::test]
async fn category_rules() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let customer = make_user(&repos, "c@x.vn", Role::User).await;
    let named = |name: &str| CategoryRequest {
        name: name.to_owned(),
    };

    let err = services.categories.create(customer, named("CPU")).await.unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
    let err = services.categories.create(admin, named("   ")).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));

    let created = services.categories.create(admin, named(" CPU ")).await.expect("tạo");
    assert_eq!(created.name, "CPU");
    let err = services.categories.create(admin, named("cpu")).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
    let err = services.categories.rename(admin, 999, named("X")).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));

    services.categories.delete(admin, created.id).await.expect("xóa");
    assert!(services.categories.list().await.expect("list").is_empty());
}

// ---------- PolicyService ----------

#[tokio::test]
async fn policy_rules() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let customer = make_user(&repos, "c@x.vn", Role::User).await;
    let request = |icon: &str, title: &str| PolicyRequest {
        icon: icon.to_owned(),
        title: title.to_owned(),
        content: "nội dung".to_owned(),
        category_id: None,
        sort_order: 1,
    };

    // Chỉ admin được ghi.
    let err = services
        .policies
        .create(customer, request("shield", "Bảo hành"))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));

    // Validate đầu vào.
    let err = services
        .policies
        .create(admin, request("khong-co", "Bảo hành"))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    let err = services
        .policies
        .create(admin, request("shield", "   "))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    let mut bad_order = request("shield", "X");
    bad_order.sort_order = -1;
    let err = services.policies.create(admin, bad_order).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    let mut bad_category = request("shield", "X");
    bad_category.category_id = Some(999);
    let err = services.policies.create(admin, bad_category).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));

    // Tạo (cắt khoảng trắng), sửa, xem công khai.
    let created = services
        .policies
        .create(admin, request("shield", " Bảo hành "))
        .await
        .expect("tạo");
    assert_eq!(created.title, "Bảo hành");
    let err = services
        .policies
        .update(admin, 999, request("truck", "Giao hàng"))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
    let updated = services
        .policies
        .update(admin, created.id, request("truck", "Giao hàng"))
        .await
        .expect("sửa");
    assert_eq!(updated.icon, "truck");
    let list = services.policies.list().await.expect("list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].title, "Giao hàng");

    // Xóa.
    let err = services.policies.delete(customer, created.id).await.unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
    services.policies.delete(admin, created.id).await.expect("xóa");
    assert!(services.policies.list().await.expect("list").is_empty());
}

// ---------- ProductService ----------

#[tokio::test]
async fn product_validation_and_permissions() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let customer = make_user(&repos, "c@x.vn", Role::User).await;

    let err = services.products.create(customer, product_request("CPU", 100)).await.unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
    for bad in [product_request("", 100), product_request("CPU", 0)] {
        let err = services.products.create(admin, bad).await.unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }

    let mut bad_category = product_request("CPU", 100);
    bad_category.category_id = Some(999);
    let err = services.products.create(admin, bad_category).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));

    let mut bad_image = product_request("CPU", 100);
    bad_image.image_url = Some("javascript:alert(1)".to_owned());
    let err = services.products.create(admin, bad_image).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));

    let err = services.products.update(admin, 999, product_request("CPU", 100)).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));
}

#[tokio::test]
async fn product_search_filters_sorts_and_pins() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let cheap = make_product(&repos, "Ryzen 5", 100).await;
    make_product(&repos, "Core i9", 900).await;

    let params = |q: &str, category: &str, sort: &str| ProductQueryParams {
        q: Some(q.to_owned()),
        category_id: Some(category.to_owned()),
        sort: Some(sort.to_owned()),
    };

    let found = services.products.search(params("ryzen", "", "new")).await.expect("tìm");
    assert_eq!(found.len(), 1);
    let sorted = services.products.search(params("", "", "priceDesc")).await.expect("sắp xếp");
    assert_eq!(sorted[0].name, "Core i9");

    let err = services.products.search(params("", "abc", "new")).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));

    services.products.set_pinned(admin, cheap, true).await.expect("ghim");
    let pinned = services.products.search(params("", "", "priceDesc")).await.expect("tìm");
    assert_eq!(pinned[0].id, cheap);
}

// ---------- CartService ----------

#[tokio::test]
async fn cart_add_accumulates_and_validates() {
    let (repos, services) = fixture();
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let product = make_product(&repos, "RAM", 100).await;

    services.carts.add_item(user, add_request(product, None)).await.expect("thêm");
    let cart = services
        .carts
        .add_item(user, add_request(product, Some(2)))
        .await
        .expect("cộng dồn");
    assert_eq!(cart[0].quantity, 3);

    let err = services.carts.add_item(user, add_request(product, Some(0))).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    let err = services.carts.add_item(user, add_request(999, Some(1))).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));

    let cart = services
        .carts
        .add_item(user, add_request(product, Some(99)))
        .await
        .expect("kẹp trần");
    assert_eq!(cart[0].quantity, 99);
}

#[tokio::test]
async fn cart_set_and_remove() {
    let (repos, services) = fixture();
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let product = make_product(&repos, "RAM", 100).await;

    let err = services
        .carts
        .set_quantity(user, product, SetQuantityRequest { quantity: 2 })
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));

    services.carts.add_item(user, add_request(product, Some(1))).await.expect("thêm");
    let cart = services
        .carts
        .set_quantity(user, product, SetQuantityRequest { quantity: 5 })
        .await
        .expect("đặt số lượng");
    assert_eq!(cart[0].quantity, 5);

    let cart = services.carts.remove_item(user, product).await.expect("xóa");
    assert!(cart.is_empty());
}

// ---------- OrderService ----------

#[tokio::test]
async fn place_order_validates_input() {
    let (repos, services) = fixture();
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let product = make_product(&repos, "RAM", 100).await;

    let err = services.orders.place_order(user, order_request("0901234567", "cod")).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)), "giỏ trống");

    services.carts.add_item(user, add_request(product, Some(1))).await.expect("thêm");
    for bad in [order_request("abc", "cod"), order_request("0901234567", "paypal")] {
        let err = services.orders.place_order(user, bad).await.unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}

#[tokio::test]
async fn place_order_snapshots_prices_and_clears_cart() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let product = make_product(&repos, "RAM", 1_000).await;
    services.carts.add_item(user, add_request(product, Some(3))).await.expect("thêm");

    let order = services
        .orders
        .place_order(user, order_request("0901234567", "bank"))
        .await
        .expect("đặt hàng");
    assert_eq!(order.total, 3_000);
    assert_eq!(order.status, OrderStatus::Pending);
    assert!(services.carts.get(user).await.expect("giỏ").is_empty());

    services
        .products
        .update(admin, product, product_request("RAM", 9_999))
        .await
        .expect("đổi giá");
    let mine = services.orders.list_mine(user).await.expect("đơn của tôi");
    assert_eq!(mine[0].items[0].price, 1_000);
    assert!(services.orders.list_mine(admin).await.expect("đơn admin").is_empty());

    let err = services.products.delete(admin, product).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
    let err = services.users.delete_user(admin, user.id).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
}

#[tokio::test]
async fn order_status_rules() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let product = make_product(&repos, "RAM", 100).await;
    services.carts.add_item(user, add_request(product, Some(1))).await.expect("thêm");
    let order = services
        .orders
        .place_order(user, order_request("0901234567", "cod"))
        .await
        .expect("đặt hàng");

    let err = services.orders.update_status(user, order.id, status_request("shipped")).await.unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
    let err = services.orders.update_status(admin, order.id, status_request("lost")).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    let err = services.orders.update_status(admin, 999, status_request("shipped")).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));

    let shipped = services
        .orders
        .update_status(admin, order.id, status_request("shipped"))
        .await
        .expect("giao hàng");
    assert_eq!(shipped.status, OrderStatus::Shipped);
    services
        .orders
        .update_status(admin, order.id, status_request("cancelled"))
        .await
        .expect("hủy");
    let err = services.orders.update_status(admin, order.id, status_request("pending")).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)), "đơn đã hủy không mở lại được");

    let all = services.orders.list_all(admin).await.expect("admin list");
    assert_eq!(all[0].user_email.as_deref(), Some("c@x.vn"));
    let err = services.orders.list_all(user).await.unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
}

#[tokio::test]
async fn customer_cancels_only_own_pending_order() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let other = make_user(&repos, "d@x.vn", Role::User).await;
    let product = make_product(&repos, "RAM", 100).await;

    services.carts.add_item(user, add_request(product, Some(1))).await.expect("thêm");
    let first = services.orders.place_order(user, order_request("0901234567", "cod")).await.expect("đặt hàng");
    services.carts.add_item(user, add_request(product, Some(1))).await.expect("thêm");
    let second = services.orders.place_order(user, order_request("0901234567", "cod")).await.expect("đặt hàng");

    let err = services.orders.cancel_order(other, first.id).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)), "không hủy được đơn của người khác");
    let err = services.orders.cancel_order(user, 999).await.unwrap_err();
    assert!(matches!(err, AppError::NotFound(_)));

    let cancelled = services.orders.cancel_order(user, first.id).await.expect("hủy đơn pending");
    assert_eq!(cancelled.status, OrderStatus::Cancelled);
    let err = services.orders.cancel_order(user, first.id).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)), "hủy lần hai bị từ chối");

    services
        .orders
        .update_status(admin, second.id, status_request("confirmed"))
        .await
        .expect("xác nhận");
    let err = services.orders.cancel_order(user, second.id).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)), "đơn đã xác nhận không tự hủy được");
}

// ---------- StatsService ----------

#[tokio::test]
async fn stats_require_admin_and_exclude_cancelled_revenue() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let product = make_product(&repos, "RAM", 500).await;
    services.carts.add_item(user, add_request(product, Some(2))).await.expect("thêm");
    let order = services
        .orders
        .place_order(user, order_request("0901234567", "cod"))
        .await
        .expect("đặt hàng");

    let err = services.stats.overview(user).await.unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
    let stats = services.stats.overview(admin).await.expect("thống kê");
    assert_eq!((stats.products, stats.users, stats.orders, stats.revenue), (1, 2, 1, 1_000));

    services
        .orders
        .update_status(admin, order.id, status_request("cancelled"))
        .await
        .expect("hủy");
    let stats = services.stats.overview(admin).await.expect("thống kê");
    assert_eq!((stats.orders, stats.revenue), (1, 0));
}

// ---------- UploadService ----------

#[tokio::test]
async fn upload_validates_permission_size_and_format() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let customer = make_user(&repos, "c@x.vn", Role::User).await;
    let png: [u8; 12] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];

    let err = services.uploads.save_image(customer, &png).await.unwrap_err();
    assert!(matches!(err, AppError::Forbidden(_)));
    let err = services.uploads.save_image(admin, &[]).await.unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    let err = services.uploads.save_image(admin, b"GIF89a....").await.unwrap_err();
    assert!(matches!(err, AppError::UnsupportedMediaType(_)));
    let oversized = vec![0xFF; super::upload_service::MAX_IMAGE_BYTES + 1];
    let err = services.uploads.save_image(admin, &oversized).await.unwrap_err();
    assert!(matches!(err, AppError::PayloadTooLarge(_)));

    let saved = services.uploads.save_image(admin, &png).await.expect("lưu ảnh");
    assert!(saved.url.starts_with("/uploads/") && saved.url.ends_with(".png"));
}

// ---------- Giới hạn theo tồn kho ----------

async fn make_product_with_stock(repos: &Repositories, name: &str, price: i64, stock: i32) -> i64 {
    repos
        .products
        .create(ProductInput {
            name: name.to_owned(),
            specs: "specs".to_owned(),
            price,
            stock,
            category_id: None,
            image_url: None,
        })
        .await
        .expect("tạo sản phẩm test")
        .id
}

async fn stock_of(repos: &Repositories, product: i64) -> i32 {
    repos
        .products
        .find_by_id(product)
        .await
        .expect("đọc sản phẩm")
        .expect("sản phẩm tồn tại")
        .stock
}

#[tokio::test]
async fn cart_rejects_quantity_above_stock() {
    let (repos, services) = fixture();
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let product = make_product_with_stock(&repos, "GPU", 100, 3).await;

    let err = services.carts.add_item(user, add_request(product, Some(4))).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)), "thêm vượt kho bị từ chối");

    services.carts.add_item(user, add_request(product, Some(2))).await.expect("thêm 2");
    let err = services.carts.add_item(user, add_request(product, Some(2))).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)), "cộng dồn vượt kho bị từ chối");
    let err = services
        .carts
        .set_quantity(user, product, SetQuantityRequest { quantity: 4 })
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)), "đặt số lượng vượt kho bị từ chối");

    let cart = services
        .carts
        .set_quantity(user, product, SetQuantityRequest { quantity: 3 })
        .await
        .expect("đúng bằng tồn kho thì được");
    assert_eq!(cart[0].quantity, 3);
}

#[tokio::test]
async fn cart_rejects_out_of_stock_product() {
    let (repos, services) = fixture();
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let product = make_product_with_stock(&repos, "GPU", 100, 0).await;

    let err = services.carts.add_item(user, add_request(product, None)).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
    assert!(services.carts.get(user).await.expect("giỏ").is_empty());
}

#[tokio::test]
async fn place_order_decrements_stock_and_blocks_when_sold_out() {
    let (repos, services) = fixture();
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let other = make_user(&repos, "d@x.vn", Role::User).await;
    let product = make_product_with_stock(&repos, "GPU", 100, 5).await;

    services.carts.add_item(user, add_request(product, Some(3))).await.expect("thêm");
    services
        .orders
        .place_order(user, order_request("0901234567", "cod"))
        .await
        .expect("đặt hàng");
    assert_eq!(stock_of(&repos, product).await, 2);

    let err = services.carts.add_item(other, add_request(product, Some(3))).await.unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)), "chỉ còn 2 nên không thêm 3 được");
    services.carts.add_item(other, add_request(product, Some(2))).await.expect("thêm 2");
    services
        .orders
        .place_order(other, order_request("0901234567", "cod"))
        .await
        .expect("đặt nốt 2 cái cuối");
    assert_eq!(stock_of(&repos, product).await, 0);
}

#[tokio::test]
async fn place_order_fails_when_stock_dropped_after_adding_to_cart() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let product = make_product_with_stock(&repos, "GPU", 100, 5).await;
    services.carts.add_item(user, add_request(product, Some(3))).await.expect("thêm");

    // Admin giảm kho xuống 2 sau khi khách đã bỏ 3 vào giỏ.
    services
        .products
        .update(admin, product, ProductRequest { stock: 2, ..product_request("GPU", 100) })
        .await
        .expect("sửa kho");

    let err = services
        .orders
        .place_order(user, order_request("0901234567", "cod"))
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Conflict(_)));
    assert_eq!(stock_of(&repos, product).await, 2, "kho không bị trừ");
    assert_eq!(services.carts.get(user).await.expect("giỏ").len(), 1, "giỏ còn nguyên");
}

#[tokio::test]
async fn cancelling_an_order_restores_stock() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;
    let user = make_user(&repos, "c@x.vn", Role::User).await;
    let product = make_product_with_stock(&repos, "GPU", 100, 5).await;

    services.carts.add_item(user, add_request(product, Some(2))).await.expect("thêm");
    let first = services.orders.place_order(user, order_request("0901234567", "cod")).await.expect("đặt hàng");
    services.carts.add_item(user, add_request(product, Some(1))).await.expect("thêm");
    let second = services.orders.place_order(user, order_request("0901234567", "cod")).await.expect("đặt hàng");
    assert_eq!(stock_of(&repos, product).await, 2);

    services.orders.cancel_order(user, first.id).await.expect("khách hủy");
    assert_eq!(stock_of(&repos, product).await, 4);

    services
        .orders
        .update_status(admin, second.id, status_request("cancelled"))
        .await
        .expect("admin hủy");
    assert_eq!(stock_of(&repos, product).await, 5);
}

#[tokio::test]
async fn admin_product_stock_is_validated() {
    let (repos, services) = fixture();
    let admin = make_user(&repos, "admin@x.vn", Role::Admin).await;

    let err = services
        .products
        .create(admin, ProductRequest { stock: -1, ..product_request("RAM", 100) })
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    let err = services
        .products
        .create(admin, ProductRequest { stock: 1_000_001, ..product_request("RAM", 100) })
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::BadRequest(_)));
    let created = services
        .products
        .create(admin, ProductRequest { stock: 0, ..product_request("RAM", 100) })
        .await
        .expect("kho 0 là hợp lệ (hết hàng)");
    assert_eq!(created.stock, 0);
}
