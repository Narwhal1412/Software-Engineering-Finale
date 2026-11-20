//! TẦNG PRESENTATION: router và handler axum. Handler chỉ trích xuất tham số,
//! gọi service rồi trả response; mọi logic nghiệp vụ nằm ở `services`.

pub mod api_json;
pub mod auth_controller;
pub mod cart_controller;
pub mod category_controller;
pub mod order_controller;
pub mod product_controller;
pub mod stats_controller;
pub mod upload_controller;
pub mod user_controller;

use axum::Router;

use crate::app_state::AppState;

/// Gộp router của mọi feature dưới tiền tố `/api`.
///
/// Route cần đăng nhập khai báo tham số `AuthUser`; phân quyền admin do service kiểm tra.
pub fn build_router() -> Router<AppState> {
    let api = Router::new()
        .merge(auth_controller::router())
        .merge(user_controller::router())
        .merge(category_controller::router())
        .merge(product_controller::router())
        .merge(upload_controller::router())
        .merge(cart_controller::router())
        .merge(order_controller::router())
        .merge(stats_controller::router());
    Router::new().nest("/api", api)
}
