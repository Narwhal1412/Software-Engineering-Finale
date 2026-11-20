//! Test tích hợp: gọi router thật (với dữ liệu seed của mock) bằng `tower::ServiceExt::oneshot`.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use computer_shop_backend::app::build_app;
use computer_shop_backend::app_state::{build_mock_repositories, AppState};
use computer_shop_backend::config::Config;
use serde_json::{json, Value};
use tower::ServiceExt;

const ADMIN_EMAIL: &str = "admin@pc.vn";
const CUSTOMER_EMAIL: &str = "khach3@pc.vn";
const SEED_PASSWORD: &str = "123456";

/// Dựng ứng dụng với repository mock đã seed.
async fn app() -> Router {
    let config = Config::from_env().expect("cấu hình");
    let repos = build_mock_repositories().expect("repository mock");
    build_app(AppState::new(config, repos))
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
    let bytes = to_bytes(response.into_body(), usize::MAX).await.expect("body");
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

async fn login(app: &Router, email: &str) -> String {
    let (status, body) = send(
        app,
        "POST",
        "/api/auth/login",
        None,
        Some(json!({ "email": email, "password": SEED_PASSWORD })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    body["token"].as_str().expect("token").to_owned()
}

#[tokio::test]
async fn health_returns_ok() {
    let (status, body) = send(&app().await, "GET", "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
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
    assert!(body["user"].get("password").is_none());
    assert!(body["user"].get("passwordHash").is_none());
}

#[tokio::test]
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
async fn products_are_public_pinned_first_and_filterable() {
    let app = app().await;
    let (status, body) = send(&app, "GET", "/api/products?q=&categoryId=&sort=new", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let products = body.as_array().expect("mảng");
    assert_eq!(products.len(), 7);
    assert_eq!(products[0]["name"], "Ryzen 7 7800X3D");
    assert!(products[0]["pinnedAt"].is_string());
    assert!(products[1]["pinnedAt"].is_null());
    assert!(products[0].get("categoryId").is_some());

    let (_, body) = send(&app, "GET", "/api/products?q=rtx&categoryId=2&sort=priceAsc", None, None).await;
    assert_eq!(body.as_array().expect("mảng").len(), 1);
}

#[tokio::test]
async fn protected_routes_check_token_and_role() {
    let app = app().await;
    let (status, _) = send(&app, "GET", "/api/admin/stats", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(&app, "GET", "/api/admin/stats", Some("garbage"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let customer = login(&app, CUSTOMER_EMAIL).await;
    let (status, body) = send(&app, "GET", "/api/admin/stats", Some(&customer), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["message"], "Cần quyền quản trị");

    let admin = login(&app, ADMIN_EMAIL).await;
    let (status, body) = send(&app, "GET", "/api/admin/stats", Some(&admin), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({ "products": 7, "users": 4, "orders": 7, "revenue": 118_370_000 })
    );
}

#[tokio::test]
async fn customer_can_fill_cart_and_place_order() {
    let app = app().await;
    let token = login(&app, CUSTOMER_EMAIL).await;

    let (status, cart) = send(
        &app,
        "POST",
        "/api/cart/items",
        Some(&token),
        Some(json!({ "productId": 1, "quantity": 2 })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cart[0]["quantity"], 2);
    assert_eq!(cart[0]["product"]["name"], "Ryzen 7 7800X3D");

    let order_body = json!({
        "fullName": "Lê Hoàng Cường", "phone": "0901234567",
        "address": "TP.HCM", "paymentMethod": "cod"
    });
    let (status, order) = send(&app, "POST", "/api/orders", Some(&token), Some(order_body)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(order["status"], "pending");
    assert_eq!(order["total"], 2 * 9_490_000);
    assert_eq!(order["items"][0]["productId"], 1);

    let (_, cart) = send(&app, "GET", "/api/cart", Some(&token), None).await;
    assert_eq!(cart, json!([]));
    let (_, mine) = send(&app, "GET", "/api/orders", Some(&token), None).await;
    assert_eq!(mine.as_array().expect("mảng").len(), 1);
}

#[tokio::test]
async fn admin_deleting_category_keeps_products() {
    let app = app().await;
    let admin = login(&app, ADMIN_EMAIL).await;

    let (status, _) = send(&app, "DELETE", "/api/categories/1", Some(&admin), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, products) = send(&app, "GET", "/api/products?categoryId=&sort=new", None, None).await;
    let products = products.as_array().expect("mảng");
    assert_eq!(products.len(), 7);
    let ryzen = products.iter().find(|p| p["id"] == 1).expect("sản phẩm 1");
    assert!(ryzen["categoryId"].is_null());
}

#[tokio::test]
async fn admin_cannot_delete_admin_or_user_with_orders() {
    let app = app().await;
    let admin = login(&app, ADMIN_EMAIL).await;

    let (status, _) = send(&app, "DELETE", "/api/admin/users/1", Some(&admin), None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = send(&app, "DELETE", "/api/admin/users/2", Some(&admin), None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = send(&app, "DELETE", "/api/admin/users/4", Some(&admin), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}
