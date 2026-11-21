//! Test logic thuần, KHÔNG cần database: băm mật khẩu, JWT, validate đầu vào,
//! phân quyền, upload ảnh, enum và kiểu sắp xếp.
//!
//! Chạy: `cargo test --test logic_tests`

use std::path::PathBuf;
use std::str::FromStr;

use computer_shop_backend::config::Config;
use computer_shop_backend::error::AppError;
use computer_shop_backend::middleware::auth::AuthUser;
use computer_shop_backend::middleware::password::{hash_password, verify_password};
use computer_shop_backend::middleware::token::{issue_token, verify_token, Claims};
use computer_shop_backend::models::enums::{OrderStatus, PaymentMethod, Role};
use computer_shop_backend::models::product::ProductSort;
use computer_shop_backend::services::authz::require_admin;
use computer_shop_backend::services::upload_service::{UploadService, MAX_IMAGE_BYTES};
use computer_shop_backend::services::validation::{
    optional_text, required_text, validate_email, validate_phone,
};

/// Cấu hình dùng cho test, khóa JWT cố định để kết quả không phụ thuộc môi trường.
fn test_config() -> Config {
    let mut config = Config::from_env().expect("cấu hình");
    config.jwt_secret = "test-secret-0123456789".to_owned();
    config.jwt_ttl_hours = 1;
    config
}

fn admin() -> AuthUser {
    AuthUser {
        id: 1,
        role: Role::Admin,
    }
}

fn customer() -> AuthUser {
    AuthUser {
        id: 2,
        role: Role::User,
    }
}

// ----------------------------------------------------------------------------
// Mật khẩu (Argon2id)
// ----------------------------------------------------------------------------

#[test]
fn password_hash_verifies_only_with_the_right_password() -> Result<(), AppError> {
    let hash = hash_password("123456")?;

    assert!(hash.starts_with("$argon2id$"));
    assert!(verify_password("123456", &hash));
    assert!(!verify_password("654321", &hash));
    Ok(())
}

#[test]
fn password_hash_uses_a_random_salt() -> Result<(), AppError> {
    let first = hash_password("same-password")?;
    let second = hash_password("same-password")?;

    assert_ne!(first, second);
    Ok(())
}

#[test]
fn corrupted_stored_hash_is_treated_as_mismatch() {
    assert!(!verify_password("123456", "not-a-phc-string"));
    assert!(!verify_password("123456", ""));
}

// ----------------------------------------------------------------------------
// JWT
// ----------------------------------------------------------------------------

#[test]
fn token_round_trip_keeps_user_id_and_role() -> Result<(), AppError> {
    let config = test_config();

    let token = issue_token(&config, 7, Role::Admin)?;
    let claims = verify_token(&config, &token)?;

    assert_eq!(claims.user_id()?, 7);
    assert_eq!(claims.role, Role::Admin);
    assert!(claims.exp > claims.iat);
    Ok(())
}

#[test]
fn token_signed_with_another_secret_is_rejected() -> Result<(), AppError> {
    let config = test_config();
    let token = issue_token(&config, 7, Role::User)?;

    let mut other = test_config();
    other.jwt_secret = "a-completely-different-secret".to_owned();

    assert!(matches!(
        verify_token(&other, &token),
        Err(AppError::Unauthorized(_))
    ));
    Ok(())
}

#[test]
fn garbage_token_is_rejected() {
    let config = test_config();

    assert!(matches!(
        verify_token(&config, "abc.def.ghi"),
        Err(AppError::Unauthorized(_))
    ));
    assert!(matches!(
        verify_token(&config, ""),
        Err(AppError::Unauthorized(_))
    ));
}

#[test]
fn claims_with_non_numeric_subject_are_invalid() {
    let claims = Claims {
        sub: "abc".to_owned(),
        role: Role::User,
        iat: 0,
        exp: 0,
    };

    assert!(matches!(claims.user_id(), Err(AppError::Unauthorized(_))));
}

// ----------------------------------------------------------------------------
// Validate đầu vào
// ----------------------------------------------------------------------------

#[test]
fn required_text_trims_and_rejects_empty() -> Result<(), AppError> {
    assert_eq!(required_text("  Ryzen 7  ", "tên", 50)?, "Ryzen 7");
    assert!(matches!(
        required_text("   ", "tên", 50),
        Err(AppError::BadRequest(_))
    ));
    assert!(matches!(
        required_text("", "tên", 50),
        Err(AppError::BadRequest(_))
    ));
    Ok(())
}

#[test]
fn text_length_is_counted_in_characters_not_bytes() -> Result<(), AppError> {
    // 5 ký tự tiếng Việt nhưng nhiều hơn 5 byte: vẫn hợp lệ với giới hạn 5.
    assert_eq!(required_text("ễễễễễ", "tên", 5)?, "ễễễễễ");
    assert!(matches!(
        required_text("ễễễễễễ", "tên", 5),
        Err(AppError::BadRequest(_))
    ));
    Ok(())
}

#[test]
fn optional_text_allows_empty_but_not_too_long() -> Result<(), AppError> {
    assert_eq!(optional_text("   ", "địa chỉ", 10)?, "");
    assert!(matches!(
        optional_text("12345678901", "địa chỉ", 10),
        Err(AppError::BadRequest(_))
    ));
    Ok(())
}

#[test]
fn phone_must_be_9_to_11_digits() {
    assert!(validate_phone("0901234567").is_ok());
    assert!(validate_phone("123456789").is_ok());
    assert!(validate_phone("12345678901").is_ok());

    for bad in ["12345678", "123456789012", "09a1234567", "+84901234567", ""] {
        assert!(
            matches!(validate_phone(bad), Err(AppError::BadRequest(_))),
            "số điện thoại {bad:?} lẽ ra phải bị từ chối"
        );
    }
}

#[test]
fn email_validation_accepts_normal_and_rejects_malformed() {
    for good in ["a@b.vn", "khach1@pc.vn", "first.last@mail.example.com"] {
        assert!(validate_email(good).is_ok(), "{good:?} lẽ ra hợp lệ");
    }

    let too_long = format!("{}@x.vn", "a".repeat(250));
    for bad in [
        "abc",
        "@b.vn",
        "a@b",
        "a@@b.vn",
        "a b@c.vn",
        "a@.vn",
        "a@b.",
        too_long.as_str(),
    ] {
        assert!(
            matches!(validate_email(bad), Err(AppError::BadRequest(_))),
            "email {bad:?} lẽ ra phải bị từ chối"
        );
    }
}

// ----------------------------------------------------------------------------
// Phân quyền
// ----------------------------------------------------------------------------

#[test]
fn only_admin_passes_require_admin() {
    assert!(require_admin(admin()).is_ok());
    // 403 chứ không phải 401, vì frontend tự đăng xuất khi nhận 401.
    assert!(matches!(
        require_admin(customer()),
        Err(AppError::Forbidden(_))
    ));
}

// ----------------------------------------------------------------------------
// Upload ảnh (ghi ra thư mục tạm)
// ----------------------------------------------------------------------------

const PNG_BYTES: [u8; 12] = [
    0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0,
];
const JPEG_BYTES: [u8; 6] = [0xFF, 0xD8, 0xFF, 0xE0, 0, 0];

/// Thư mục tạm riêng cho từng test để các test không đè lên nhau.
fn temp_upload_dir(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!("shop_upload_test_{}_{tag}", std::process::id()))
}

#[tokio::test]
async fn upload_saves_png_and_jpeg_with_server_generated_names() -> Result<(), AppError> {
    let dir = temp_upload_dir("ok");
    let service = UploadService::new(&dir.to_string_lossy());

    let png = service.save_image(admin(), &PNG_BYTES).await?;
    let jpg = service.save_image(admin(), &JPEG_BYTES).await?;

    assert!(png.url.starts_with("/uploads/") && png.url.ends_with(".png"));
    assert!(jpg.url.starts_with("/uploads/") && jpg.url.ends_with(".jpg"));
    assert_ne!(png.url, jpg.url);

    let saved_name = png.url.trim_start_matches("/uploads/");
    assert!(dir.join(saved_name).is_file());

    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[tokio::test]
async fn upload_accepts_webp_signature() -> Result<(), AppError> {
    let dir = temp_upload_dir("webp");
    let service = UploadService::new(&dir.to_string_lossy());

    let mut bytes = b"RIFF".to_vec();
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes.extend_from_slice(b"WEBP");
    let result = service.save_image(admin(), &bytes).await?;

    assert!(result.url.ends_with(".webp"));
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

#[tokio::test]
async fn upload_rejects_non_admin_empty_oversized_and_non_image() {
    let dir = temp_upload_dir("reject");
    let service = UploadService::new(&dir.to_string_lossy());

    assert!(matches!(
        service.save_image(customer(), &PNG_BYTES).await,
        Err(AppError::Forbidden(_))
    ));
    assert!(matches!(
        service.save_image(admin(), &[]).await,
        Err(AppError::BadRequest(_))
    ));
    let oversized = vec![0u8; MAX_IMAGE_BYTES + 1];
    assert!(matches!(
        service.save_image(admin(), &oversized).await,
        Err(AppError::PayloadTooLarge(_))
    ));
    // Chữ ký byte quyết định, không tin tên tệp hay Content-Type của client.
    assert!(matches!(
        service.save_image(admin(), b"<?php echo 1; ?>").await,
        Err(AppError::UnsupportedMediaType(_))
    ));

    let _ = std::fs::remove_dir_all(&dir);
}

// ----------------------------------------------------------------------------
// Enum và kiểu sắp xếp
// ----------------------------------------------------------------------------

#[test]
fn enums_round_trip_through_their_database_strings() {
    for role in [Role::User, Role::Admin] {
        assert_eq!(Role::from_str(role.as_str()).ok(), Some(role));
    }
    for status in [
        OrderStatus::Pending,
        OrderStatus::Confirmed,
        OrderStatus::Shipped,
        OrderStatus::Completed,
        OrderStatus::Cancelled,
    ] {
        assert_eq!(OrderStatus::from_str(status.as_str()).ok(), Some(status));
    }
    for method in [PaymentMethod::Cod, PaymentMethod::Bank] {
        assert_eq!(
            PaymentMethod::from_str(method.as_str()).ok(),
            Some(method)
        );
    }
}

#[test]
fn unknown_enum_strings_are_rejected() {
    assert!(Role::from_str("root").is_err());
    assert!(OrderStatus::from_str("refunded").is_err());
    assert!(PaymentMethod::from_str("paypal").is_err());
    // Phân biệt hoa thường, giống ENUM lưu trong database.
    assert!(Role::from_str("Admin").is_err());
}

#[test]
fn enums_serialize_as_lowercase_json_strings() {
    assert_eq!(
        serde_json::to_string(&OrderStatus::Cancelled).ok().as_deref(),
        Some("\"cancelled\"")
    );
    assert_eq!(
        serde_json::to_string(&PaymentMethod::Bank).ok().as_deref(),
        Some("\"bank\"")
    );
    assert_eq!(
        serde_json::to_string(&Role::Admin).ok().as_deref(),
        Some("\"admin\"")
    );
}

#[test]
fn product_sort_defaults_to_newest_for_unknown_values() {
    assert_eq!(ProductSort::from_query("priceAsc"), ProductSort::PriceAsc);
    assert_eq!(ProductSort::from_query("priceDesc"), ProductSort::PriceDesc);
    assert_eq!(ProductSort::from_query("new"), ProductSort::Newest);
    assert_eq!(ProductSort::from_query(""), ProductSort::Newest);
    assert_eq!(ProductSort::from_query("DROP TABLE"), ProductSort::Newest);
}
