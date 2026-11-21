//! Test tích hợp cho 5 repository MariaDB (tầng DAL thật).
//!
//! Các test này cần một MariaDB đã nạp `init.sql`, nên mặc định bị `#[ignore]`
//! để `cargo test` thường vẫn chạy được khi không có database. Chạy chúng bằng:
//!
//! ```text
//! cargo test --test mysql_repository_tests -- --ignored --test-threads=1
//! ```
//!
//! (xem TESTING.md để biết cách dựng database và đặt biến DB_*).
//!
//! Mỗi test tự tạo dữ liệu riêng với tên/email duy nhất nên chạy lại nhiều lần không cần
//! reset database. Dùng `--test-threads=1` vì vài test (đếm đơn, doanh thu) so sánh số
//! liệu toàn cục trước và sau khi thao tác.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use computer_shop_backend::config::Config;
use computer_shop_backend::db;
use computer_shop_backend::middleware::password::verify_password;
use computer_shop_backend::models::enums::{OrderStatus, PaymentMethod, Role};
use computer_shop_backend::models::order::{Order, OrderDraft};
use computer_shop_backend::models::product::{Product, ProductInput, ProductQuery, ProductSort};
use computer_shop_backend::models::user::{NewUser, ProfileUpdate, User};
use computer_shop_backend::repositories::mysql::{
    MySqlCartRepository, MySqlCategoryRepository, MySqlOrderRepository, MySqlProductRepository,
    MySqlUserRepository,
};
use computer_shop_backend::repositories::{
    CartRepository, CategoryRepository, OrderRepository, ProductRepository, RepoError,
    UserRepository,
};

type TestResult = Result<(), RepoError>;

/// Id chắc chắn không tồn tại (AUTO_INCREMENT không bao giờ tới được).
const MISSING_ID: i64 = 9_000_000_000_000;

// ----------------------------------------------------------------------------
// Hạ tầng dùng chung
// ----------------------------------------------------------------------------

struct Repos {
    users: MySqlUserRepository,
    categories: MySqlCategoryRepository,
    products: MySqlProductRepository,
    carts: MySqlCartRepository,
    orders: MySqlOrderRepository,
}

/// Kết nối theo biến môi trường DB_* (hoặc file `.env`) và dựng 5 repository.
async fn repos() -> Repos {
    dotenvy::dotenv().ok();
    let config = Config::from_env().expect("cấu hình");
    let pool = db::connect(&config)
        .await
        .expect("không kết nối được MariaDB: kiểm tra DB_HOST/DB_PORT/DB_USER/DB_PASSWORD/DB_NAME");
    Repos {
        users: MySqlUserRepository::new(pool.clone()),
        categories: MySqlCategoryRepository::new(pool.clone()),
        products: MySqlProductRepository::new(pool.clone()),
        carts: MySqlCartRepository::new(pool.clone()),
        orders: MySqlOrderRepository::new(pool),
    }
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

fn new_user(email: &str, role: Role) -> NewUser {
    NewUser {
        email: email.to_owned(),
        display_name: "Người dùng thử".to_owned(),
        password_hash: "hash".to_owned(),
        phone: String::new(),
        address: String::new(),
        role,
    }
}

fn product_input(name: &str, price: i64, category_id: Option<i64>) -> ProductInput {
    ProductInput {
        name: name.to_owned(),
        specs: "thông số thử".to_owned(),
        price,
        category_id,
        image_url: None,
    }
}

fn draft(payment_method: PaymentMethod) -> OrderDraft {
    OrderDraft {
        full_name: "Nguyễn Văn Thử".to_owned(),
        phone: "0901234567".to_owned(),
        address: "TP.HCM".to_owned(),
        payment_method,
    }
}

async fn make_user(repos: &Repos) -> Result<User, RepoError> {
    let email = format!("{}@test.vn", unique("u"));
    repos.users.create(new_user(&email, Role::User)).await
}

async fn make_product(repos: &Repos, price: i64) -> Result<Product, RepoError> {
    let name = unique("p");
    repos
        .products
        .create(product_input(&name, price, None))
        .await
}

/// Tạo user + sản phẩm, bỏ vào giỏ rồi đặt đơn. Trả về (user, đơn).
async fn place_order(repos: &Repos, price: i64, quantity: i32) -> Result<(User, Order), RepoError> {
    let user = make_user(repos).await?;
    let product = make_product(repos, price).await?;
    repos.carts.add_item(user.id, product.id, quantity).await?;
    let order = repos
        .orders
        .create_from_cart(user.id, draft(PaymentMethod::Cod))
        .await?;
    Ok((user, order))
}

// ----------------------------------------------------------------------------
// Kết nối (không cần database)
// ----------------------------------------------------------------------------

#[tokio::test]
async fn connect_to_a_closed_port_returns_internal_error() {
    let mut config = Config::from_env().expect("cấu hình");
    config.db_host = "127.0.0.1".to_owned();
    config.db_port = 1;

    let err = db::connect(&config).await.unwrap_err();

    assert!(matches!(err, RepoError::Internal(_)));
}

// ----------------------------------------------------------------------------
// Dữ liệu mẫu từ init.sql
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn seed_accounts_exist_and_accept_the_documented_password() -> TestResult {
    let repos = repos().await;

    let admin = repos
        .users
        .find_by_email("admin@pc.vn")
        .await?
        .expect("thiếu tài khoản admin@pc.vn: đã nạp init.sql chưa?");
    assert_eq!(admin.role, Role::Admin);
    assert!(verify_password("123456", &admin.password_hash));

    let customer = repos
        .users
        .find_by_email("khach1@pc.vn")
        .await?
        .expect("thiếu tài khoản khach1@pc.vn");
    assert_eq!(customer.role, Role::User);
    assert!(verify_password("123456", &customer.password_hash));
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB đã nạp init.sql"]
async fn every_order_total_equals_the_sum_of_its_items() -> TestResult {
    let repos = repos().await;

    let all = repos.orders.list_all_with_user_email().await?;

    assert!(!all.is_empty(), "database chưa có đơn hàng nào");
    for entry in all {
        let sum: i64 = entry
            .order
            .items
            .iter()
            .map(|item| item.price * i64::from(item.quantity))
            .sum();
        assert_eq!(entry.order.total, sum, "đơn #{} lệch tổng tiền", entry.order.id);
    }
    Ok(())
}

// ----------------------------------------------------------------------------
// UserRepository
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn user_create_then_find_by_id_and_by_email_ignoring_case() -> TestResult {
    let repos = repos().await;
    let email = format!("{}@test.vn", unique("u"));

    let created = repos.users.create(new_user(&email, Role::User)).await?;

    assert!(created.id > 0);
    assert_eq!(created.email, email);
    assert_eq!(created.role, Role::User);
    assert_eq!(created.phone, "");
    assert_eq!(repos.users.find_by_id(created.id).await?, Some(created.clone()));
    let by_email = repos.users.find_by_email(&email.to_uppercase()).await?;
    assert_eq!(by_email.map(|user| user.id), Some(created.id));
    assert!(repos.users.find_by_id(MISSING_ID).await?.is_none());
    assert!(repos.users.find_by_email("khong-co@test.vn").await?.is_none());
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn user_duplicate_email_is_rejected_case_insensitively() -> TestResult {
    let repos = repos().await;
    let email = format!("{}@test.vn", unique("u"));
    repos.users.create(new_user(&email, Role::User)).await?;

    let err = repos
        .users
        .create(new_user(&email.to_uppercase(), Role::User))
        .await
        .unwrap_err();

    assert!(matches!(err, RepoError::Duplicate { field: "email" }));
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn user_update_profile_changes_fields_and_unknown_id_is_not_found() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;

    let updated = repos
        .users
        .update_profile(
            user.id,
            ProfileUpdate {
                display_name: "Tên mới".to_owned(),
                phone: "0912345678".to_owned(),
                address: "Hà Nội".to_owned(),
            },
        )
        .await?;
    assert_eq!(updated.display_name, "Tên mới");
    assert_eq!(updated.phone, "0912345678");
    assert_eq!(updated.address, "Hà Nội");
    assert_eq!(updated.email, user.email);

    // Cập nhật y hệt (MariaDB báo 0 dòng thay đổi) vẫn phải thành công.
    let same = repos
        .users
        .update_profile(
            user.id,
            ProfileUpdate {
                display_name: "Tên mới".to_owned(),
                phone: "0912345678".to_owned(),
                address: "Hà Nội".to_owned(),
            },
        )
        .await?;
    assert_eq!(same.id, user.id);

    let err = repos
        .users
        .update_profile(
            MISSING_ID,
            ProfileUpdate {
                display_name: "x".to_owned(),
                phone: String::new(),
                address: String::new(),
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(err, RepoError::NotFound { entity: "user" }));
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn user_list_contains_new_user_and_count_is_positive() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;

    let all = repos.users.list().await?;

    assert!(all.iter().any(|candidate| candidate.id == user.id));
    assert!(all.windows(2).all(|pair| pair[0].id < pair[1].id));
    assert!(repos.users.count().await? >= 1);
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn user_delete_also_removes_the_cart() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let product = make_product(&repos, 1_000).await?;
    repos.carts.add_item(user.id, product.id, 1).await?;

    repos.users.delete(user.id).await?;

    assert!(repos.users.find_by_id(user.id).await?.is_none());
    assert!(repos.carts.list_with_products(user.id).await?.is_empty());
    // Sản phẩm không bị ảnh hưởng.
    assert!(repos.products.find_by_id(product.id).await?.is_some());
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn user_with_orders_cannot_be_deleted() -> TestResult {
    let repos = repos().await;
    let (user, _order) = place_order(&repos, 1_000, 1).await?;

    let err = repos.users.delete(user.id).await.unwrap_err();

    assert!(matches!(
        err,
        RepoError::InUse {
            entity: "user",
            referenced_by: "order"
        }
    ));
    assert!(repos.users.find_by_id(user.id).await?.is_some());
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn user_delete_unknown_id_is_not_found() {
    let repos = repos().await;

    let err = repos.users.delete(MISSING_ID).await.unwrap_err();

    assert!(matches!(err, RepoError::NotFound { entity: "user" }));
}

// ----------------------------------------------------------------------------
// CategoryRepository
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn category_create_find_rename_and_list() -> TestResult {
    let repos = repos().await;
    let name = unique("c");

    let created = repos.categories.create(&name).await?;
    assert_eq!(created.name, name);
    assert_eq!(
        repos.categories.find_by_id(created.id).await?,
        Some(created.clone())
    );

    let new_name = unique("c-moi");
    let renamed = repos.categories.rename(created.id, &new_name).await?;
    assert_eq!(renamed.id, created.id);
    assert_eq!(renamed.name, new_name);

    let all = repos.categories.list().await?;
    assert!(all.iter().any(|category| category.id == created.id));
    assert!(all.windows(2).all(|pair| pair[0].id < pair[1].id));
    assert!(repos.categories.find_by_id(MISSING_ID).await?.is_none());
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn category_duplicate_name_is_rejected_case_insensitively() -> TestResult {
    let repos = repos().await;
    let name = unique("c");
    repos.categories.create(&name).await?;

    let err = repos
        .categories
        .create(&name.to_uppercase())
        .await
        .unwrap_err();

    assert!(matches!(err, RepoError::Duplicate { field: "name" }));
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn category_rename_errors_for_duplicate_name_and_unknown_id() -> TestResult {
    let repos = repos().await;
    let first = repos.categories.create(&unique("c")).await?;
    let second = repos.categories.create(&unique("c")).await?;

    let duplicate = repos
        .categories
        .rename(second.id, &first.name)
        .await
        .unwrap_err();
    assert!(matches!(duplicate, RepoError::Duplicate { field: "name" }));

    let missing = repos
        .categories
        .rename(MISSING_ID, &unique("c"))
        .await
        .unwrap_err();
    assert!(matches!(missing, RepoError::NotFound { entity: "category" }));
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn category_delete_keeps_products_but_clears_their_category() -> TestResult {
    let repos = repos().await;
    let category = repos.categories.create(&unique("c")).await?;
    let product = repos
        .products
        .create(product_input(&unique("p"), 1_000, Some(category.id)))
        .await?;
    assert_eq!(product.category_id, Some(category.id));

    repos.categories.delete(category.id).await?;

    assert!(repos.categories.find_by_id(category.id).await?.is_none());
    let after = repos
        .products
        .find_by_id(product.id)
        .await?
        .expect("sản phẩm phải còn sau khi xóa danh mục");
    assert_eq!(after.category_id, None);
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn category_delete_unknown_id_is_not_found() {
    let repos = repos().await;

    let err = repos.categories.delete(MISSING_ID).await.unwrap_err();

    assert!(matches!(err, RepoError::NotFound { entity: "category" }));
}

// ----------------------------------------------------------------------------
// ProductRepository
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_create_find_and_update_round_trip() -> TestResult {
    let repos = repos().await;
    let category = repos.categories.create(&unique("c")).await?;
    let name = unique("p");

    let created = repos
        .products
        .create(ProductInput {
            name: name.clone(),
            specs: "8 nhân".to_owned(),
            price: 9_490_000,
            category_id: Some(category.id),
            image_url: Some("https://example.com/a.jpg".to_owned()),
        })
        .await?;

    assert!(created.id > 0);
    assert_eq!(created.name, name);
    assert_eq!(created.price, 9_490_000);
    assert_eq!(created.category_id, Some(category.id));
    assert_eq!(created.image_url.as_deref(), Some("https://example.com/a.jpg"));
    assert!(created.pinned_at.is_none());
    assert_eq!(
        repos.products.find_by_id(created.id).await?,
        Some(created.clone())
    );

    let updated = repos
        .products
        .update(created.id, product_input(&name, 8_000_000, None))
        .await?;
    assert_eq!(updated.price, 8_000_000);
    assert_eq!(updated.category_id, None);
    assert_eq!(updated.image_url, None);
    assert_eq!(updated.created_at, created.created_at);
    assert!(repos.products.count().await? >= 1);
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_with_unknown_category_is_an_invalid_reference() -> TestResult {
    let repos = repos().await;

    let on_create = repos
        .products
        .create(product_input(&unique("p"), 1_000, Some(MISSING_ID)))
        .await
        .unwrap_err();
    assert!(matches!(
        on_create,
        RepoError::InvalidReference {
            field: "category_id"
        }
    ));

    let product = make_product(&repos, 1_000).await?;
    let on_update = repos
        .products
        .update(product.id, product_input(&product.name, 1_000, Some(MISSING_ID)))
        .await
        .unwrap_err();
    assert!(matches!(
        on_update,
        RepoError::InvalidReference {
            field: "category_id"
        }
    ));
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_negative_price_violates_the_check_constraint() {
    let repos = repos().await;

    let err = repos
        .products
        .create(product_input(&unique("p"), -1, None))
        .await
        .unwrap_err();

    assert!(matches!(err, RepoError::Constraint(_)));
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_unknown_id_is_not_found_for_update_delete_and_pin() {
    let repos = repos().await;
    let not_found = |err: RepoError| matches!(err, RepoError::NotFound { entity: "product" });

    let update = repos
        .products
        .update(MISSING_ID, product_input("x", 1, None))
        .await
        .unwrap_err();
    let delete = repos.products.delete(MISSING_ID).await.unwrap_err();
    let pin = repos.products.set_pinned(MISSING_ID, true).await.unwrap_err();

    assert!(not_found(update));
    assert!(not_found(delete));
    assert!(not_found(pin));
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_pin_and_unpin() -> TestResult {
    let repos = repos().await;
    let product = make_product(&repos, 1_000).await?;

    let pinned = repos.products.set_pinned(product.id, true).await?;
    assert!(pinned.pinned_at.is_some());

    let unpinned = repos.products.set_pinned(product.id, false).await?;
    assert!(unpinned.pinned_at.is_none());
    Ok(())
}

/// Tạo 3 sản phẩm cùng một từ khóa, giá 300 / 100 / 200 (theo thứ tự tạo).
async fn three_tagged_products(repos: &Repos, tag: &str) -> Result<[Product; 3], RepoError> {
    let expensive = repos
        .products
        .create(product_input(&format!("{tag} dat"), 300, None))
        .await?;
    let cheap = repos
        .products
        .create(product_input(&format!("{tag} re"), 100, None))
        .await?;
    let middle = repos
        .products
        .create(product_input(&format!("{tag} vua"), 200, None))
        .await?;
    Ok([expensive, cheap, middle])
}

fn prices(products: &[Product]) -> Vec<i64> {
    products.iter().map(|product| product.price).collect()
}

fn query(tag: &str, sort: ProductSort) -> ProductQuery {
    ProductQuery {
        q: Some(tag.to_owned()),
        category_id: None,
        sort,
    }
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_search_sorts_by_price_and_by_newest() -> TestResult {
    let repos = repos().await;
    let tag = unique("sort");
    let [expensive, cheap, middle] = three_tagged_products(&repos, &tag).await?;

    let asc = repos.products.search(&query(&tag, ProductSort::PriceAsc)).await?;
    assert_eq!(prices(&asc), vec![100, 200, 300]);

    let desc = repos
        .products
        .search(&query(&tag, ProductSort::PriceDesc))
        .await?;
    assert_eq!(prices(&desc), vec![300, 200, 100]);

    let newest = repos.products.search(&query(&tag, ProductSort::Newest)).await?;
    let ids: Vec<i64> = newest.iter().map(|product| product.id).collect();
    assert_eq!(ids, vec![middle.id, cheap.id, expensive.id]);
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_search_puts_pinned_products_first() -> TestResult {
    let repos = repos().await;
    let tag = unique("pin");
    let [expensive, _cheap, _middle] = three_tagged_products(&repos, &tag).await?;
    repos.products.set_pinned(expensive.id, true).await?;

    let result = repos.products.search(&query(&tag, ProductSort::PriceAsc)).await?;

    // Ghim đứng đầu dù đắt nhất, phần còn lại theo giá tăng dần.
    assert_eq!(prices(&result), vec![300, 100, 200]);
    assert!(result[0].pinned_at.is_some());
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_search_matches_name_and_specs_and_filters_by_category() -> TestResult {
    let repos = repos().await;
    let tag = unique("find");
    let category = repos.categories.create(&unique("c")).await?;
    let in_category = repos
        .products
        .create(product_input(&format!("{tag} trong danh muc"), 10, Some(category.id)))
        .await?;
    let outside = repos
        .products
        .create(ProductInput {
            name: "ten khong lien quan".to_owned(),
            specs: format!("co tu khoa {tag} o thong so"),
            price: 20,
            category_id: None,
            image_url: None,
        })
        .await?;

    // Từ khóa nằm ở tên hoặc thông số; không phân biệt hoa thường.
    let by_keyword = repos
        .products
        .search(&query(&tag.to_uppercase(), ProductSort::Newest))
        .await?;
    let ids: Vec<i64> = by_keyword.iter().map(|product| product.id).collect();
    assert!(ids.contains(&in_category.id));
    assert!(ids.contains(&outside.id));

    // Thêm lọc danh mục thì chỉ còn sản phẩm thuộc danh mục đó.
    let filtered = repos
        .products
        .search(&ProductQuery {
            q: Some(tag.clone()),
            category_id: Some(category.id),
            sort: ProductSort::Newest,
        })
        .await?;
    let filtered_ids: Vec<i64> = filtered.iter().map(|product| product.id).collect();
    assert_eq!(filtered_ids, vec![in_category.id]);

    // Từ khóa rỗng hoặc chỉ có khoảng trắng nghĩa là không lọc.
    let blank = repos
        .products
        .search(&ProductQuery {
            q: Some("   ".to_owned()),
            category_id: Some(category.id),
            sort: ProductSort::Newest,
        })
        .await?;
    assert_eq!(blank.len(), 1);
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_search_treats_like_wildcards_as_plain_characters() -> TestResult {
    let repos = repos().await;
    let tag = unique("wild");
    let with_underscore = repos
        .products
        .create(product_input(&format!("{tag}_x"), 1, None))
        .await?;
    repos
        .products
        .create(product_input(&format!("{tag}Zx"), 1, None))
        .await?;

    // `_` là ký tự thường: không được khớp với "Z".
    let underscore = repos
        .products
        .search(&query(&format!("{tag}_x"), ProductSort::Newest))
        .await?;
    assert_eq!(underscore.len(), 1);
    assert_eq!(underscore[0].id, with_underscore.id);

    // `%` là ký tự thường: không khớp "mọi thứ".
    let percent = repos
        .products
        .search(&query(&format!("{tag}%"), ProductSort::Newest))
        .await?;
    assert!(percent.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_delete_removes_it_from_carts() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let product = make_product(&repos, 1_000).await?;
    repos.carts.add_item(user.id, product.id, 2).await?;

    repos.products.delete(product.id).await?;

    assert!(repos.products.find_by_id(product.id).await?.is_none());
    assert!(repos.carts.list_with_products(user.id).await?.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn product_that_was_ordered_cannot_be_deleted() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let product = make_product(&repos, 1_000).await?;
    repos.carts.add_item(user.id, product.id, 1).await?;
    repos
        .orders
        .create_from_cart(user.id, draft(PaymentMethod::Bank))
        .await?;

    let err = repos.products.delete(product.id).await.unwrap_err();

    assert!(matches!(
        err,
        RepoError::InUse {
            entity: "product",
            referenced_by: "order"
        }
    ));
    assert!(repos.products.find_by_id(product.id).await?.is_some());
    Ok(())
}

// ----------------------------------------------------------------------------
// CartRepository
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn cart_add_item_accumulates_quantity_and_joins_product() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let product = make_product(&repos, 5_000).await?;

    let first = repos.carts.add_item(user.id, product.id, 2).await?;
    assert_eq!(first.quantity, 2);
    let second = repos.carts.add_item(user.id, product.id, 3).await?;
    assert_eq!(second.quantity, 5);

    let lines = repos.carts.list_with_products(user.id).await?;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].product_id, product.id);
    assert_eq!(lines[0].quantity, 5);
    assert_eq!(lines[0].product, product);
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn cart_lines_are_sorted_by_product_id() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let older = make_product(&repos, 1).await?;
    let newer = make_product(&repos, 2).await?;
    // Thêm sản phẩm mới trước để chứng minh thứ tự không phụ thuộc thứ tự thêm.
    repos.carts.add_item(user.id, newer.id, 1).await?;
    repos.carts.add_item(user.id, older.id, 1).await?;

    let lines = repos.carts.list_with_products(user.id).await?;

    let ids: Vec<i64> = lines.iter().map(|line| line.product_id).collect();
    assert_eq!(ids, vec![older.id, newer.id]);
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn cart_set_quantity_is_absolute_and_needs_an_existing_line() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let product = make_product(&repos, 1_000).await?;
    repos.carts.add_item(user.id, product.id, 5).await?;

    let item = repos.carts.set_quantity(user.id, product.id, 2).await?;
    assert_eq!(item.quantity, 2);

    let other_product = make_product(&repos, 1_000).await?;
    let err = repos
        .carts
        .set_quantity(user.id, other_product.id, 1)
        .await
        .unwrap_err();
    assert!(matches!(err, RepoError::NotFound { entity: "cart item" }));
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn cart_rejects_quantity_below_one() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let product = make_product(&repos, 1_000).await?;
    repos.carts.add_item(user.id, product.id, 1).await?;

    let add_zero = repos.carts.add_item(user.id, product.id, 0).await.unwrap_err();
    let add_negative = repos.carts.add_item(user.id, product.id, -3).await.unwrap_err();
    let set_zero = repos
        .carts
        .set_quantity(user.id, product.id, 0)
        .await
        .unwrap_err();

    assert!(matches!(add_zero, RepoError::Constraint(_)));
    assert!(matches!(add_negative, RepoError::Constraint(_)));
    assert!(matches!(set_zero, RepoError::Constraint(_)));
    // Số lượng cũ không bị đổi.
    let lines = repos.carts.list_with_products(user.id).await?;
    assert_eq!(lines[0].quantity, 1);
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn cart_add_item_with_unknown_product_or_user_is_an_invalid_reference() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let product = make_product(&repos, 1_000).await?;

    let bad_product = repos.carts.add_item(user.id, MISSING_ID, 1).await.unwrap_err();
    let bad_user = repos.carts.add_item(MISSING_ID, product.id, 1).await.unwrap_err();

    assert!(matches!(
        bad_product,
        RepoError::InvalidReference {
            field: "product_id"
        }
    ));
    assert!(matches!(
        bad_user,
        RepoError::InvalidReference { field: "user_id" }
    ));
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn cart_remove_is_idempotent_and_clear_empties_the_cart() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let first = make_product(&repos, 1).await?;
    let second = make_product(&repos, 2).await?;
    repos.carts.add_item(user.id, first.id, 1).await?;
    repos.carts.add_item(user.id, second.id, 1).await?;

    repos.carts.remove_item(user.id, first.id).await?;
    repos.carts.remove_item(user.id, first.id).await?; // lần hai vẫn Ok
    assert_eq!(repos.carts.list_with_products(user.id).await?.len(), 1);

    repos.carts.clear(user.id).await?;
    repos.carts.clear(user.id).await?; // giỏ trống vẫn Ok
    assert!(repos.carts.list_with_products(user.id).await?.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn carts_of_different_users_are_isolated() -> TestResult {
    let repos = repos().await;
    let alice = make_user(&repos).await?;
    let bob = make_user(&repos).await?;
    let product = make_product(&repos, 1_000).await?;
    repos.carts.add_item(alice.id, product.id, 4).await?;

    assert!(repos.carts.list_with_products(bob.id).await?.is_empty());
    repos.carts.clear(bob.id).await?;
    let alice_lines = repos.carts.list_with_products(alice.id).await?;
    assert_eq!(alice_lines.len(), 1);
    assert_eq!(alice_lines[0].quantity, 4);
    Ok(())
}

// ----------------------------------------------------------------------------
// OrderRepository
// ----------------------------------------------------------------------------

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn order_from_cart_snapshots_prices_computes_total_and_clears_cart() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let big = make_product(&repos, 1_000_000).await?;
    let small = make_product(&repos, 500).await?;
    repos.carts.add_item(user.id, big.id, 2).await?;
    repos.carts.add_item(user.id, small.id, 1).await?;

    let order = repos
        .orders
        .create_from_cart(user.id, draft(PaymentMethod::Bank))
        .await?;

    assert_eq!(order.user_id, user.id);
    assert_eq!(order.total, 2_000_500);
    assert_eq!(order.status, OrderStatus::Pending);
    assert_eq!(order.payment_method, PaymentMethod::Bank);
    assert_eq!(order.full_name, "Nguyễn Văn Thử");
    assert_eq!(order.items.len(), 2);
    let big_line = order
        .items
        .iter()
        .find(|item| item.product_id == big.id)
        .expect("thiếu dòng hàng của sản phẩm lớn");
    assert_eq!(big_line.name, big.name);
    assert_eq!(big_line.price, 1_000_000);
    assert_eq!(big_line.quantity, 2);
    assert!(repos.carts.list_with_products(user.id).await?.is_empty());

    // Đổi giá sản phẩm sau đó không được làm đổi đơn đã đặt (snapshot).
    repos
        .products
        .update(big.id, product_input(&big.name, 1, None))
        .await?;
    let reloaded = repos
        .orders
        .find_by_id(order.id)
        .await?
        .expect("đơn vừa tạo phải đọc lại được");
    assert_eq!(reloaded.total, 2_000_500);
    let reloaded_line = reloaded
        .items
        .iter()
        .find(|item| item.product_id == big.id)
        .expect("thiếu dòng hàng");
    assert_eq!(reloaded_line.price, 1_000_000);
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn order_from_empty_cart_is_rejected_and_creates_nothing() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;

    let err = repos
        .orders
        .create_from_cart(user.id, draft(PaymentMethod::Cod))
        .await
        .unwrap_err();

    assert!(matches!(err, RepoError::Constraint(_)));
    assert!(repos.orders.list_by_user(user.id).await?.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn order_for_unknown_user_is_an_invalid_reference() {
    let repos = repos().await;

    let err = repos
        .orders
        .create_from_cart(MISSING_ID, draft(PaymentMethod::Cod))
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        RepoError::InvalidReference { field: "user_id" }
    ));
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn order_update_status_persists_and_unknown_id_is_not_found() -> TestResult {
    let repos = repos().await;
    let (_user, order) = place_order(&repos, 1_000, 1).await?;

    let updated = repos
        .orders
        .update_status(order.id, OrderStatus::Confirmed)
        .await?;
    assert_eq!(updated.status, OrderStatus::Confirmed);
    assert_eq!(updated.items.len(), 1);
    let reloaded = repos.orders.find_by_id(order.id).await?;
    assert_eq!(reloaded.map(|found| found.status), Some(OrderStatus::Confirmed));

    let err = repos
        .orders
        .update_status(MISSING_ID, OrderStatus::Shipped)
        .await
        .unwrap_err();
    assert!(matches!(err, RepoError::NotFound { entity: "order" }));
    assert!(repos.orders.find_by_id(MISSING_ID).await?.is_none());
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn order_lists_are_newest_first_and_scoped_to_the_user() -> TestResult {
    let repos = repos().await;
    let user = make_user(&repos).await?;
    let product = make_product(&repos, 1_000).await?;
    repos.carts.add_item(user.id, product.id, 1).await?;
    let first = repos
        .orders
        .create_from_cart(user.id, draft(PaymentMethod::Cod))
        .await?;
    repos.carts.add_item(user.id, product.id, 1).await?;
    let second = repos
        .orders
        .create_from_cart(user.id, draft(PaymentMethod::Cod))
        .await?;
    let (_other_user, other_order) = place_order(&repos, 1_000, 1).await?;

    let mine = repos.orders.list_by_user(user.id).await?;

    let ids: Vec<i64> = mine.iter().map(|order| order.id).collect();
    assert_eq!(ids, vec![second.id, first.id]);
    assert!(!ids.contains(&other_order.id));
    assert!(mine.iter().all(|order| order.items.len() == 1));
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB"]
async fn admin_order_list_includes_the_buyer_email() -> TestResult {
    let repos = repos().await;
    let (user, order) = place_order(&repos, 1_000, 1).await?;

    let all = repos.orders.list_all_with_user_email().await?;

    let entry = all
        .iter()
        .find(|entry| entry.order.id == order.id)
        .expect("đơn vừa tạo phải có trong danh sách của admin");
    assert_eq!(entry.user_email.as_deref(), Some(user.email.as_str()));
    assert_eq!(entry.order.items.len(), 1);
    Ok(())
}

#[tokio::test]
#[ignore = "cần MariaDB (chạy với --test-threads=1)"]
async fn order_count_and_revenue_track_new_and_cancelled_orders() -> TestResult {
    let repos = repos().await;
    let count_before = repos.orders.count().await?;
    let revenue_before = repos.orders.revenue_excluding_cancelled().await?;

    let (_user, order) = place_order(&repos, 7_000, 1).await?;

    assert_eq!(repos.orders.count().await?, count_before + 1);
    assert_eq!(
        repos.orders.revenue_excluding_cancelled().await?,
        revenue_before + 7_000
    );

    repos
        .orders
        .update_status(order.id, OrderStatus::Cancelled)
        .await?;

    // Đơn hủy vẫn được đếm nhưng không tính doanh thu.
    assert_eq!(repos.orders.count().await?, count_before + 1);
    assert_eq!(
        repos.orders.revenue_excluding_cancelled().await?,
        revenue_before
    );
    Ok(())
}
