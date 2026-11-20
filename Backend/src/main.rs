//! Điểm khởi động: nạp cấu hình, dựng AppState, chạy server axum trên tokio.

use std::error::Error;
use std::net::SocketAddr;

use computer_shop_backend::app::build_app;
use computer_shop_backend::app_state::{build_repositories, AppState};
use computer_shop_backend::config::Config;
use computer_shop_backend::middleware::logging::init_tracing;
use computer_shop_backend::middleware::password::hash_password;
use tokio::net::TcpListener;
use tokio::signal;

/// Tên lệnh con để băm mật khẩu, dùng khi chuẩn bị dữ liệu seed cho database thật.
const HASH_PASSWORD_COMMAND: &str = "hash-password";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some(HASH_PASSWORD_COMMAND) {
        return print_password_hash(&args);
    }

    // File .env là tùy chọn (production dùng biến môi trường thật), nên bỏ qua lỗi.
    dotenvy::dotenv().ok();
    init_tracing();

    let config = Config::from_env()?;
    if config.uses_default_jwt_secret() {
        tracing::warn!("đang dùng JWT_SECRET mặc định, hãy đặt khóa riêng khi triển khai thật");
    }
    tokio::fs::create_dir_all(&config.upload_dir).await?;

    let port = config.port;
    let repos = build_repositories(&config).await?;
    let app = build_app(AppState::new(config, repos));

    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;
    tracing::info!(%port, "backend đã sẵn sàng");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Lệnh `cargo run -- hash-password <mật khẩu>`: in chuỗi băm Argon2 ra stdout.
fn print_password_hash(args: &[String]) -> Result<(), Box<dyn Error>> {
    let plain = args
        .get(2)
        .ok_or("cách dùng: computer_shop_backend hash-password <mật khẩu>")?;
    println!("{}", hash_password(plain)?);
    Ok(())
}

/// Chờ Ctrl+C hoặc SIGTERM (Docker stop) để tắt server êm.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = signal::ctrl_c().await {
            tracing::error!(%err, "không lắng nghe được Ctrl+C");
            // Nếu không đăng ký được tín hiệu thì không được tắt server vì lỗi này.
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(err) => {
                tracing::error!(%err, "không lắng nghe được SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
