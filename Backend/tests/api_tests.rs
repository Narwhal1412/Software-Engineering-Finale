//! Test tích hợp HTTP: gọi router thật (controllers -> services -> repository MariaDB)
//! bằng `tower::ServiceExt::oneshot`, không mở cổng mạng.
//!
//! Cần MariaDB đã nạp `init.sql`, nên mọi test đều `#[ignore]` (xem TESTING.md):
//!
//! ```text
//! cargo test --test api_tests -- --ignored
//! ```
//!
//! Mỗi test TỰ TẠO dữ liệu riêng (khách hàng đăng ký mới, danh mục và sản phẩm có tên duy nhất)
//! thay vì dựa vào dữ liệu mẫu, nên các test độc lập với nhau, không phụ thuộc thứ tự chạy và
//! chạy lại nhiều lần trên cùng một database vẫn đúng. Chỉ tài khoản `admin@pc.vn` từ `init.sql`
//! được dùng (chỉ để đăng nhập, không bị sửa hay xóa).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use computer_shop_backend::app::build_app;
use computer_shop_backend::app_state::{build_repositories, AppState};
use computer_shop_backend::config::Config;
use serde_json::{json, Value};
use tower::ServiceExt;

const ADMIN_EMAIL: &str = "admin@pc.vn";
const SEED_PASSWORD: &str = "123456";
/// Id chắc chắn không tồn tại.
const MISSING_ID: i64 = 9_000_000_000_000;

// ----------------------------------------------------------------------------
// Hạ tầng dùng chung
// ----------------------------------------------------------------------------

/// Dựng ứng dụng trên MariaDB thật (đọc DB_* từ môi trường hoặc `.env`).
async fn app() -> Router {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("cấu hình");
    let repos = build_repositories(&config)
        .await
        .expect("không kết nối được MariaDB: kiểm tra DB_HOST/DB_PORT/DB_USER/DB_PASSWORD/DB_NAME");
    build_app(AppState::new(config, repos))
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Chuỗi duy nhất (chữ thường) để dữ liệu của các test không đụng nhau.
fn unique(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{nanos:x}-{sequence}")
}

/// Gửi một request, trả về status và body JSON (`Null` nếu body rỗng).
async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }
    let request_body = match body {
        Some(json) => {
            builder = builder.header("Content-Type", "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(builder.body(request_body).expect("request"))
        .await
        .expect("response");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// Đăng nhập, trả về (token, id người dùng).
async fn login(app: &Router, email: &str) -> (String, i64) {
    let (status, body) = send(
        app,
        "POST",
        "/api/auth/login",
        None,
        Some(json!({ "email": email, "password": SEED_PASSWORD })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    (
        body["token"].as_str().expect("token").to_owned(),
        body["user"]["id"].as_i64().expect("id người dùng"),
    )
}

/// Token admin (tài khoản mẫu từ init.sql).
async fn admin_token(app: &Router) -> String {
    login(app, ADMIN_EMAIL).await.0
}

/// Đăng ký một khách hàng mới, trả về (token, id, email).
async fn register_customer(app: &Router) -> (String, i64, String) {
    let email = format!("{}@test.vn", unique("kh"));
    let (status, body) = send(
        app,
        "POST",
        "/api/auth/register",
        None,
        Some(json!({
            "email": email,
            "displayName": "Khách thử",
            "password": SEED_PASSWORD,
            "confirmPassword": SEED_PASSWORD
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    (
        body["token"].as_str().expect("token").to_owned(),
        body["user"]["id"].as_i64().expect("id người dùng"),
        email,
    )
}

/// Admin tạo danh mục mới, trả về (id, tên).
async fn create_category(app: &Router, admin: &str) -> (i64, String) {
    let name = unique("dm");
    let (status, body) = send(
        app,
        "POST",
        "/api/categories",
        Some(admin),
        Some(json!({ "name": name })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    (body["id"].as_i64().expect("id danh mục"), name)
}

/// Admin tạo sản phẩm mới, trả về id.
async fn create_product(
    app: &Router,
    admin: &str,
    name: &str,
    price: i64,
    category_id: Option<i64>,
) -> i64 {
    let (status, body) = send(
        app,
        "POST",
        "/api/products",
        Some(admin),
        Some(json!({
            "name": name,
            "specs": "thông số thử",
            "price": price,
            "stock": 1000,
            "categoryId": category_id
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_i64().expect("id sản phẩm")
}

fn order_body() -> Value {
    json!({
        "fullName": "Nguyễn Văn Thử",
        "phone": "0901234567",
        "address": "TP.HCM",
        "paymentMethod": "cod"
    })
}

/// Bỏ sản phẩm vào giỏ, kiểm tra 200 và trả về giỏ mới.
async fn add_to_cart(app: &Router, token: &str, product_id: i64, quantity: i32) -> Value {
    let (status, cart) = send(
        app,
        "POST",
        "/api/cart/items",
        Some(token),
        Some(json!({ "productId": product_id, "quantity": quantity })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    cart
}

/// Đặt hàng từ giỏ hiện tại, kiểm tra 201 và trả về đơn.
async fn place_order(app: &Router, token: &str) -> Value {
    let (status, order) = send(app, "POST", "/api/orders", Some(token), Some(order_body())).await;
    assert_eq!(status, StatusCode::CREATED);
    order
}

// ----------------------------------------------------------------------------
// Sức khỏe và xác thực
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn health_returns_ok() {
    let (status, body) = send(&app().await, "GET", "/health", None, None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn login_returns_user_without_password() {
    let app = app().await;

    let (status, body) = send(
        &app,
        "POST",
        "/api/auth/login",
        None,
        Some(json!({ "email": ADMIN_EMAIL, "password": SEED_PASSWORD })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["role"], "admin");
    assert_eq!(body["user"]["displayName"], "Quản trị");
    assert!(body["token"].is_string());
    assert!(body["user"].get("password").is_none());
    assert!(body["user"].get("passwordHash").is_none());
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn bad_login_and_bad_json_return_message() {
    let app = app().await;

    let (status, body) = send(
        &app,
        "POST",
        "/api/auth/login",
        None,
        Some(json!({ "email": ADMIN_EMAIL, "password": "wrong" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["message"], "Sai email hoặc mật khẩu");

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("Content-Type", "application/json")
        .body(Body::from("{not json"))
        .expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn register_creates_a_customer_who_is_logged_in_immediately() {
    let app = app().await;
    let (token, id, email) = register_customer(&app).await;

    let (status, me) = send(&app, "GET", "/api/me", Some(&token), None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["id"], id);
    assert_eq!(me["email"], email);
    // Đăng ký công khai không bao giờ tạo admin.
    assert_eq!(me["role"], "user");

    // Đăng nhập lại bằng chính tài khoản vừa đăng ký.
    let (relogin_token, relogin_id) = login(&app, &email).await;
    assert!(!relogin_token.is_empty());
    assert_eq!(relogin_id, id);
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn register_rejects_duplicate_email_weak_password_and_mismatch() {
    let app = app().await;
    let (_, _, email) = register_customer(&app).await;
    let payload = |email: &str, password: &str, confirm: &str| {
        json!({
            "email": email,
            "displayName": "Khách thử",
            "password": password,
            "confirmPassword": confirm
        })
    };

    // Email đã tồn tại (không phân biệt hoa thường) -> 409.
    let (status, _) = send(
        &app,
        "POST",
        "/api/auth/register",
        None,
        Some(payload(&email.to_uppercase(), SEED_PASSWORD, SEED_PASSWORD)),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let fresh = format!("{}@test.vn", unique("kh"));
    let (short, _) = send(
        &app,
        "POST",
        "/api/auth/register",
        None,
        Some(payload(&fresh, "123", "123")),
    )
    .await;
    assert_eq!(short, StatusCode::BAD_REQUEST);

    let (mismatch, body) = send(
        &app,
        "POST",
        "/api/auth/register",
        None,
        Some(payload(&fresh, SEED_PASSWORD, "khac-mat-khau")),
    )
    .await;
    assert_eq!(mismatch, StatusCode::BAD_REQUEST);
    assert_eq!(body["message"], "Mật khẩu nhập lại không khớp");

    let (bad_email, _) = send(
        &app,
        "POST",
        "/api/auth/register",
        None,
        Some(payload("khong-phai-email", SEED_PASSWORD, SEED_PASSWORD)),
    )
    .await;
    assert_eq!(bad_email, StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn customer_can_update_own_profile() {
    let app = app().await;
    let (token, _, _) = register_customer(&app).await;

    let (status, updated) = send(
        &app,
        "PUT",
        "/api/me",
        Some(&token),
        Some(json!({
            "displayName": "Tên mới",
            "phone": "0912345678",
            "address": "Hà Nội"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["displayName"], "Tên mới");
    assert_eq!(updated["phone"], "0912345678");

    let (_, me) = send(&app, "GET", "/api/me", Some(&token), None).await;
    assert_eq!(me["address"], "Hà Nội");

    let (bad_phone, _) = send(
        &app,
        "PUT",
        "/api/me",
        Some(&token),
        Some(json!({ "displayName": "Tên mới", "phone": "abc", "address": "" })),
    )
    .await;
    assert_eq!(bad_phone, StatusCode::BAD_REQUEST);
}

// ----------------------------------------------------------------------------
// Phân quyền
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn protected_routes_check_token_and_role() {
    let app = app().await;
    let (customer, _, _) = register_customer(&app).await;

    let (status, _) = send(&app, "GET", "/api/admin/stats", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(&app, "GET", "/api/admin/stats", Some("garbage"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(&app, "GET", "/api/cart", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Đã đăng nhập nhưng không phải admin -> 403 (không phải 401, vì 401 làm frontend tự đăng xuất).
    let (status, body) = send(&app, "GET", "/api/admin/stats", Some(&customer), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["message"], "Cần quyền quản trị");
    let (status, _) = send(&app, "GET", "/api/admin/users", Some(&customer), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = send(&app, "GET", "/api/admin/orders", Some(&customer), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let admin = admin_token(&app).await;
    let (status, stats) = send(&app, "GET", "/api/admin/stats", Some(&admin), None).await;
    assert_eq!(status, StatusCode::OK);
    for key in ["products", "users", "orders", "revenue"] {
        assert!(stats[key].is_number(), "thiếu số liệu thống kê `{key}`");
    }
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn only_admin_can_manage_categories_and_products() {
    let app = app().await;
    let (customer, _, _) = register_customer(&app).await;

    let (status, _) = send(
        &app,
        "POST",
        "/api/categories",
        Some(&customer),
        Some(json!({ "name": unique("dm") })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = send(
        &app,
        "POST",
        "/api/products",
        Some(&customer),
        Some(json!({ "name": unique("sp"), "specs": "x", "price": 1000 })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

// ----------------------------------------------------------------------------
// Danh mục và sản phẩm
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn admin_can_create_rename_and_delete_categories() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (id, name) = create_category(&app, &admin).await;

    // Danh sách danh mục là công khai.
    let (status, list) = send(&app, "GET", "/api/categories", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(list
        .as_array()
        .expect("mảng")
        .iter()
        .any(|category| category["id"] == id));

    // Trùng tên (không phân biệt hoa thường) -> 409.
    let (status, _) = send(
        &app,
        "POST",
        "/api/categories",
        Some(&admin),
        Some(json!({ "name": name.to_uppercase() })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    let new_name = unique("dm-moi");
    let (status, renamed) = send(
        &app,
        "PUT",
        &format!("/api/categories/{id}"),
        Some(&admin),
        Some(json!({ "name": new_name })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(renamed["name"], new_name);

    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/categories/{id}"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/categories/{id}"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn admin_deleting_category_keeps_its_products() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (category_id, _) = create_category(&app, &admin).await;
    let tag = unique("giu");
    let product_id = create_product(
        &app,
        &admin,
        &format!("{tag} sp"),
        1_000_000,
        Some(category_id),
    )
    .await;

    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/categories/{category_id}"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, products) = send(
        &app,
        "GET",
        &format!("/api/products?q={tag}&categoryId=&sort=new"),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let products = products.as_array().expect("mảng");
    assert_eq!(products.len(), 1);
    assert_eq!(products[0]["id"], product_id);
    assert!(products[0]["categoryId"].is_null());
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn products_are_public_pinned_first_and_filterable() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (category_id, _) = create_category(&app, &admin).await;
    let tag = unique("tim");
    let cheap = create_product(&app, &admin, &format!("{tag} re"), 100_000, Some(category_id)).await;
    let pricey = create_product(&app, &admin, &format!("{tag} dat"), 200_000, Some(category_id)).await;
    let outside = create_product(&app, &admin, &format!("{tag} ngoai"), 300_000, None).await;

    // Ghim sản phẩm đắt: nó phải đứng đầu dù sắp xếp theo giá tăng dần.
    let (status, _) = send(
        &app,
        "PATCH",
        &format!("/api/products/{pricey}/pin"),
        Some(&admin),
        Some(json!({ "pinned": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let list_uri = format!("/api/products?q={tag}&categoryId={category_id}&sort=priceAsc");
    let (status, body) = send(&app, "GET", &list_uri, None, None).await;
    assert_eq!(status, StatusCode::OK);
    let products = body.as_array().expect("mảng");
    assert_eq!(products.len(), 2, "lọc danh mục phải loại sản phẩm `ngoai`");
    assert_eq!(products[0]["id"], pricey);
    assert!(products[0]["pinnedAt"].is_string());
    assert_eq!(products[1]["id"], cheap);
    assert!(products[1]["pinnedAt"].is_null());
    assert!(products[0].get("categoryId").is_some());

    // Chỉ lọc theo từ khóa thì thấy cả 3.
    let (_, all) = send(
        &app,
        "GET",
        &format!("/api/products?q={tag}&categoryId=&sort=priceDesc"),
        None,
        None,
    )
    .await;
    let ids: Vec<i64> = all
        .as_array()
        .expect("mảng")
        .iter()
        .map(|product| product["id"].as_i64().expect("id sản phẩm"))
        .collect();
    // Sản phẩm đang ghim đứng đầu, phần còn lại theo giá giảm dần.
    assert_eq!(ids, vec![pricey, outside, cheap]);

    // Bỏ ghim thì quay về thứ tự theo giá.
    let (status, _) = send(
        &app,
        "PATCH",
        &format!("/api/products/{pricey}/pin"),
        Some(&admin),
        Some(json!({ "pinned": false })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, body) = send(&app, "GET", &list_uri, None, None).await;
    assert_eq!(body[0]["id"], cheap);
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn admin_product_crud_and_validation() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let name = unique("sp");
    let id = create_product(&app, &admin, &name, 5_000_000, None).await;

    // Giá phải lớn hơn 0; thiếu tên bị từ chối.
    let (status, _) = send(
        &app,
        "POST",
        "/api/products",
        Some(&admin),
        Some(json!({ "name": unique("sp"), "specs": "x", "price": 0 })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send(
        &app,
        "POST",
        "/api/products",
        Some(&admin),
        Some(json!({ "name": "  ", "specs": "x", "price": 1000 })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Danh mục không tồn tại bị từ chối.
    let (status, _) = send(
        &app,
        "POST",
        "/api/products",
        Some(&admin),
        Some(json!({
            "name": unique("sp"), "specs": "x", "price": 1000, "categoryId": MISSING_ID
        })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, updated) = send(
        &app,
        "PUT",
        &format!("/api/products/{id}"),
        Some(&admin),
        Some(json!({ "name": name, "specs": "đã sửa", "price": 4_500_000 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["price"], 4_500_000);
    assert_eq!(updated["specs"], "đã sửa");

    let (status, _) = send(
        &app,
        "PUT",
        &format!("/api/products/{MISSING_ID}"),
        Some(&admin),
        Some(json!({ "name": "x", "specs": "x", "price": 1000 })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/products/{id}"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/products/{id}"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn product_that_was_ordered_cannot_be_deleted() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (customer, _, _) = register_customer(&app).await;
    let product_id = create_product(&app, &admin, &unique("sp"), 1_000_000, None).await;
    add_to_cart(&app, &customer, product_id, 1).await;
    place_order(&app, &customer).await;

    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/products/{product_id}"),
        Some(&admin),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

// ----------------------------------------------------------------------------
// Giỏ hàng và đơn hàng
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn customer_can_fill_cart_and_place_order() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (token, _, _) = register_customer(&app).await;
    let name = unique("sp");
    let price = 2_000_000;
    let product_id = create_product(&app, &admin, &name, price, None).await;

    let cart = add_to_cart(&app, &token, product_id, 2).await;
    assert_eq!(cart[0]["quantity"], 2);
    assert_eq!(cart[0]["product"]["name"], name);

    // Thêm tiếp thì cộng dồn, rồi đặt lại số lượng tuyệt đối bằng PATCH.
    let cart = add_to_cart(&app, &token, product_id, 1).await;
    assert_eq!(cart[0]["quantity"], 3);
    let (status, cart) = send(
        &app,
        "PATCH",
        &format!("/api/cart/items/{product_id}"),
        Some(&token),
        Some(json!({ "quantity": 4 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cart[0]["quantity"], 4);

    let order = place_order(&app, &token).await;
    assert_eq!(order["status"], "pending");
    assert_eq!(order["total"], 4 * price);
    assert_eq!(order["items"][0]["productId"], product_id);
    assert_eq!(order["items"][0]["name"], name);
    assert_eq!(order["paymentMethod"], "cod");

    let (_, cart) = send(&app, "GET", "/api/cart", Some(&token), None).await;
    assert_eq!(cart, json!([]));
    let (_, mine) = send(&app, "GET", "/api/orders", Some(&token), None).await;
    let mine = mine.as_array().expect("mảng");
    assert_eq!(mine.len(), 1);
    assert_eq!(mine[0]["id"], order["id"]);
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn cart_validates_quantity_product_and_removal() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (token, _, _) = register_customer(&app).await;
    let product_id = create_product(&app, &admin, &unique("sp"), 1_000_000, None).await;

    let (status, _) = send(
        &app,
        "POST",
        "/api/cart/items",
        Some(&token),
        Some(json!({ "productId": product_id, "quantity": 0 })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = send(
        &app,
        "POST",
        "/api/cart/items",
        Some(&token),
        Some(json!({ "productId": MISSING_ID, "quantity": 1 })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Sửa số lượng của dòng chưa có trong giỏ -> 404.
    let (status, _) = send(
        &app,
        "PATCH",
        &format!("/api/cart/items/{product_id}"),
        Some(&token),
        Some(json!({ "quantity": 2 })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    add_to_cart(&app, &token, product_id, 1).await;
    let (status, cart) = send(
        &app,
        "DELETE",
        &format!("/api/cart/items/{product_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cart, json!([]));
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn placing_an_order_validates_cart_and_shipping_info() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (token, _, _) = register_customer(&app).await;

    // Giỏ trống.
    let (status, body) = send(&app, "POST", "/api/orders", Some(&token), Some(order_body())).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["message"], "Giỏ hàng đang trống");

    // Thông tin giao hàng sai: giỏ không bị mất.
    let product_id = create_product(&app, &admin, &unique("sp"), 1_000_000, None).await;
    add_to_cart(&app, &token, product_id, 1).await;
    let mut bad = order_body();
    bad["phone"] = json!("12");
    let (status, _) = send(&app, "POST", "/api/orders", Some(&token), Some(bad)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let mut bad = order_body();
    bad["paymentMethod"] = json!("bitcoin");
    let (status, _) = send(&app, "POST", "/api/orders", Some(&token), Some(bad)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (_, cart) = send(&app, "GET", "/api/cart", Some(&token), None).await;
    assert_eq!(cart.as_array().expect("mảng").len(), 1);
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn order_keeps_the_price_at_purchase_time() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (token, _, _) = register_customer(&app).await;
    let name = unique("sp");
    let product_id = create_product(&app, &admin, &name, 3_000_000, None).await;
    add_to_cart(&app, &token, product_id, 1).await;
    let order = place_order(&app, &token).await;

    // Admin đổi giá sau khi khách đã đặt: đơn cũ không đổi.
    let (status, _) = send(
        &app,
        "PUT",
        &format!("/api/products/{product_id}"),
        Some(&admin),
        Some(json!({ "name": name, "specs": "x", "price": 1_000, "stock": 1000 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (_, mine) = send(&app, "GET", "/api/orders", Some(&token), None).await;
    assert_eq!(mine[0]["id"], order["id"]);
    assert_eq!(mine[0]["total"], 3_000_000);
    assert_eq!(mine[0]["items"][0]["price"], 3_000_000);
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn customer_can_cancel_own_pending_order_only() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (customer, _, _) = register_customer(&app).await;
    let (stranger, _, _) = register_customer(&app).await;
    let product_id = create_product(&app, &admin, &unique("sp"), 1_000_000, None).await;
    add_to_cart(&app, &customer, product_id, 1).await;
    let order = place_order(&app, &customer).await;
    let uri = format!("/api/orders/{}/cancel", order["id"]);

    let (status, _) = send(&app, "PATCH", &uri, None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(&app, "PATCH", &uri, Some(&stranger), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, cancelled) = send(&app, "PATCH", &uri, Some(&customer), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cancelled["status"], "cancelled");
    let (status, body) = send(&app, "PATCH", &uri, Some(&customer), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["message"].is_string());
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn admin_updates_order_status_and_final_states_are_locked() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (customer, _, email) = register_customer(&app).await;
    let product_id = create_product(&app, &admin, &unique("sp"), 1_000_000, None).await;
    add_to_cart(&app, &customer, product_id, 1).await;
    let order = place_order(&app, &customer).await;
    let uri = format!("/api/admin/orders/{}", order["id"]);

    // Admin thấy đơn kèm email người đặt.
    let (status, all) = send(&app, "GET", "/api/admin/orders", Some(&admin), None).await;
    assert_eq!(status, StatusCode::OK);
    let entry = all
        .as_array()
        .expect("mảng")
        .iter()
        .find(|candidate| candidate["id"] == order["id"])
        .expect("đơn phải có trong danh sách của admin");
    assert_eq!(entry["userEmail"], email);

    // Khách không được đổi trạng thái.
    let (status, _) = send(
        &app,
        "PATCH",
        &uri,
        Some(&customer),
        Some(json!({ "status": "confirmed" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, updated) = send(
        &app,
        "PATCH",
        &uri,
        Some(&admin),
        Some(json!({ "status": "confirmed" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["status"], "confirmed");

    let (status, _) = send(
        &app,
        "PATCH",
        &uri,
        Some(&admin),
        Some(json!({ "status": "khong-hop-le" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send(
        &app,
        "PATCH",
        &format!("/api/admin/orders/{MISSING_ID}"),
        Some(&admin),
        Some(json!({ "status": "confirmed" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Hoàn tất rồi thì không mở lại được (409), nhưng đặt lại đúng trạng thái đó thì được.
    let (status, _) = send(
        &app,
        "PATCH",
        &uri,
        Some(&admin),
        Some(json!({ "status": "completed" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send(
        &app,
        "PATCH",
        &uri,
        Some(&admin),
        Some(json!({ "status": "pending" })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = send(
        &app,
        "PATCH",
        &uri,
        Some(&admin),
        Some(json!({ "status": "completed" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

// ----------------------------------------------------------------------------
// Quản lý thành viên
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn admin_cannot_delete_admin_or_user_with_orders() {
    let app = app().await;
    let (admin, admin_id) = login(&app, ADMIN_EMAIL).await;
    let (buyer, buyer_id, _) = register_customer(&app).await;
    let (_, idle_id, _) = register_customer(&app).await;
    let product_id = create_product(&app, &admin, &unique("sp"), 1_000_000, None).await;
    add_to_cart(&app, &buyer, product_id, 1).await;
    place_order(&app, &buyer).await;

    // Không xóa được admin (403) và người đã có đơn hàng (409).
    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/admin/users/{admin_id}"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/admin/users/{buyer_id}"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Người chưa có đơn thì xóa được; xóa lần hai là 404.
    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/admin/users/{idle_id}"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/admin/users/{idle_id}"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Danh sách thành viên của admin không lộ mật khẩu.
    let (status, users) = send(&app, "GET", "/api/admin/users", Some(&admin), None).await;
    assert_eq!(status, StatusCode::OK);
    let users = users.as_array().expect("mảng");
    assert!(users.iter().any(|user| user["id"] == buyer_id));
    assert!(users.iter().all(|user| user.get("passwordHash").is_none()));
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn deleted_users_token_stops_working() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (token, id, _) = register_customer(&app).await;

    let (status, _) = send(&app, "GET", "/api/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/admin/users/{id}"),
        Some(&admin),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Token vẫn còn hạn nhưng tài khoản đã bị xóa -> 401 để frontend tự đăng xuất.
    let (status, _) = send(&app, "GET", "/api/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn cart_and_order_cannot_exceed_stock() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (token, _, _) = register_customer(&app).await;
    let name = unique("sp");
    let (status, body) = send(
        &app,
        "POST",
        "/api/products",
        Some(&admin),
        Some(json!({ "name": name, "specs": "x", "price": 1_000, "stock": 3 })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["stock"], 3);
    let product_id = body["id"].as_i64().expect("id sản phẩm");

    // Thêm vượt tồn kho -> 409 kèm message.
    let (status, body) = send(
        &app,
        "POST",
        "/api/cart/items",
        Some(&token),
        Some(json!({ "productId": product_id, "quantity": 4 })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["message"].as_str().is_some_and(|m| !m.is_empty()));

    // Đặt đủ 3 -> kho về 0; sản phẩm hết hàng không thêm được nữa.
    add_to_cart(&app, &token, product_id, 3).await;
    place_order(&app, &token).await;
    let (status, _) = send(
        &app,
        "POST",
        "/api/cart/items",
        Some(&token),
        Some(json!({ "productId": product_id, "quantity": 1 })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Kho giảm xuống 0 sau khi đặt được phản ánh ở danh sách sản phẩm.
    let (_, list) = send(&app, "GET", &format!("/api/products?q={name}"), None, None).await;
    assert_eq!(list[0]["stock"], 0);
}

// ----------------------------------------------------------------------------
// Chính sách và ưu đãi
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn admin_manages_policies_and_everyone_can_read_them() {
    let app = app().await;
    let admin = admin_token(&app).await;
    let (customer, _, _) = register_customer(&app).await;
    let (category_id, _) = create_category(&app, &admin).await;
    let title = unique("cs");
    let body = json!({
        "icon": "shield",
        "title": title,
        "content": "Nội dung thử",
        "categoryId": category_id,
        "sortOrder": 5
    });

    // Khách không được tạo.
    let (status, _) = send(&app, "POST", "/api/policies", Some(&customer), Some(body.clone())).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Dữ liệu sai: biểu tượng lạ, danh mục không tồn tại.
    let mut bad_icon = body.clone();
    bad_icon["icon"] = json!("khong-co");
    let (status, _) = send(&app, "POST", "/api/policies", Some(&admin), Some(bad_icon)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let mut bad_category = body.clone();
    bad_category["categoryId"] = json!(MISSING_ID);
    let (status, _) = send(&app, "POST", "/api/policies", Some(&admin), Some(bad_category)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Admin tạo; ai cũng đọc được danh sách.
    let (status, created) = send(&app, "POST", "/api/policies", Some(&admin), Some(body)).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_i64().expect("id chính sách");
    assert_eq!(created["categoryId"], category_id);
    let (status, list) = send(&app, "GET", "/api/policies", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(list
        .as_array()
        .expect("mảng")
        .iter()
        .any(|policy| policy["id"] == id));

    // Sửa thành chính sách chung (categoryId = null).
    let (status, updated) = send(
        &app,
        "PUT",
        &format!("/api/policies/{id}"),
        Some(&admin),
        Some(json!({
            "icon": "truck",
            "title": "Giao nhanh",
            "content": "Nội dung mới",
            "categoryId": null,
            "sortOrder": 6
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["icon"], "truck");
    assert!(updated["categoryId"].is_null());

    // Xóa: lần hai là 404.
    let (status, _) = send(&app, "DELETE", &format!("/api/policies/{id}"), Some(&admin), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = send(&app, "DELETE", &format!("/api/policies/{id}"), Some(&admin), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
