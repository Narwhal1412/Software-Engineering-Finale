//! Test các ràng buộc mà mock phải mô phỏng đúng như database thật.

use super::*;
use crate::models::enums::{OrderStatus, PaymentMethod, Role};
use crate::models::order::OrderDraft;
use crate::models::product::{ProductInput, ProductQuery};
use crate::models::user::NewUser;
use crate::repositories::{
    CartRepository, CategoryRepository, OrderRepository, ProductRepository, RepoError,
    UserRepository,
};

fn new_user(email: &str) -> NewUser {
    NewUser {
        email: email.to_owned(),
        display_name: "Test".to_owned(),
        password_hash: "hash".to_owned(),
        phone: String::new(),
        address: String::new(),
        role: Role::User,
    }
}

fn product_input(name: &str, price: i64, category_id: Option<i64>) -> ProductInput {
    ProductInput {
        name: name.to_owned(),
        specs: "specs".to_owned(),
        price,
        stock: 100,
        category_id,
        image_url: None,
    }
}

fn draft() -> OrderDraft {
    OrderDraft {
        full_name: "Nguyen Van A".to_owned(),
        phone: "0901234567".to_owned(),
        address: "HCM".to_owned(),
        payment_method: PaymentMethod::Cod,
    }
}

#[tokio::test]
async fn duplicate_email_is_rejected_case_insensitively() -> Result<(), RepoError> {
    let users = InMemoryUserRepository::new(MockStore::empty());
    users.create(new_user("a@x.vn")).await?;

    let err = users.create(new_user("A@X.vn")).await.unwrap_err();
    assert!(matches!(err, RepoError::Duplicate { field: "email" }));
    Ok(())
}

#[tokio::test]
async fn product_with_unknown_category_is_rejected() {
    let products = InMemoryProductRepository::new(MockStore::empty());

    let err = products
        .create(product_input("CPU", 100, Some(99)))
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        RepoError::InvalidReference {
            field: "category_id"
        }
    ));
}

#[tokio::test]
async fn deleting_category_unsets_product_category() -> Result<(), RepoError> {
    let store = MockStore::empty();
    let categories = InMemoryCategoryRepository::new(store.clone());
    let products = InMemoryProductRepository::new(store);

    let category = categories.create("CPU").await?;
    let product = products
        .create(product_input("Ryzen", 100, Some(category.id)))
        .await?;
    categories.delete(category.id).await?;

    let reloaded = products.find_by_id(product.id).await?;
    assert_eq!(reloaded.map(|p| p.category_id), Some(None));
    Ok(())
}

#[tokio::test]
async fn order_snapshots_prices_clears_cart_and_blocks_product_delete() -> Result<(), RepoError> {
    let store = MockStore::empty();
    let users = InMemoryUserRepository::new(store.clone());
    let products = InMemoryProductRepository::new(store.clone());
    let carts = InMemoryCartRepository::new(store.clone());
    let orders = InMemoryOrderRepository::new(store);

    let user = users.create(new_user("a@x.vn")).await?;
    let product = products.create(product_input("Ryzen", 1_000, None)).await?;
    carts.add_item(user.id, product.id, 2).await?;
    carts.add_item(user.id, product.id, 1).await?;

    let order = orders.create_from_cart(user.id, draft()).await?;
    assert_eq!(order.total, 3_000);
    assert_eq!(order.status, OrderStatus::Pending);
    assert!(carts.list_with_products(user.id).await?.is_empty());

    // Đổi giá sau đó không được làm đổi đơn cũ.
    products
        .update(product.id, product_input("Ryzen", 5_000, None))
        .await?;
    let stored = orders.find_by_id(order.id).await?;
    assert_eq!(stored.map(|o| o.items[0].price), Some(1_000));

    let err = products.delete(product.id).await.unwrap_err();
    assert!(matches!(err, RepoError::InUse { .. }));
    let err = users.delete(user.id).await.unwrap_err();
    assert!(matches!(err, RepoError::InUse { .. }));
    Ok(())
}

#[tokio::test]
async fn order_from_empty_cart_is_rejected() -> Result<(), RepoError> {
    let store = MockStore::empty();
    let users = InMemoryUserRepository::new(store.clone());
    let orders = InMemoryOrderRepository::new(store);

    let user = users.create(new_user("a@x.vn")).await?;
    let err = orders.create_from_cart(user.id, draft()).await.unwrap_err();
    assert!(matches!(err, RepoError::Constraint(_)));
    Ok(())
}

#[tokio::test]
async fn pinned_products_come_first_in_search() -> Result<(), RepoError> {
    let products = InMemoryProductRepository::new(MockStore::empty());
    let cheap = products.create(product_input("Cheap", 1, None)).await?;
    products.create(product_input("Dear", 9, None)).await?;
    products.set_pinned(cheap.id, true).await?;

    let rows = products
        .search(&ProductQuery {
            sort: crate::models::product::ProductSort::PriceDesc,
            ..ProductQuery::default()
        })
        .await?;
    assert_eq!(rows[0].id, cheap.id);
    Ok(())
}

#[tokio::test]
async fn seeded_store_matches_documented_numbers() -> Result<(), RepoError> {
    let store = MockStore::seeded()?;
    let users = InMemoryUserRepository::new(store.clone());
    let products = InMemoryProductRepository::new(store.clone());
    let orders = InMemoryOrderRepository::new(store);

    assert_eq!(users.count().await?, 4);
    assert_eq!(products.count().await?, 7);
    assert_eq!(orders.count().await?, 7);
    assert_eq!(orders.revenue_excluding_cancelled().await?, 118_370_000);
    assert!(users.find_by_email(SEED_ADMIN_EMAIL).await?.is_some());
    Ok(())
}

#[tokio::test]
async fn order_is_rejected_when_stock_is_insufficient_and_nothing_changes() -> Result<(), RepoError> {
    let store = MockStore::empty();
    let users = InMemoryUserRepository::new(store.clone());
    let products = InMemoryProductRepository::new(store.clone());
    let carts = InMemoryCartRepository::new(store.clone());
    let orders = InMemoryOrderRepository::new(store);

    let user = users.create(new_user("a@x.vn")).await?;
    let enough = products.create(product_input("RAM", 100, None)).await?; // kho 100
    let scarce = products
        .create(ProductInput {
            stock: 2,
            ..product_input("GPU", 500, None)
        })
        .await?;
    carts.add_item(user.id, enough.id, 5).await?;
    carts.add_item(user.id, scarce.id, 3).await?; // vượt kho (còn 2)

    let err = orders.create_from_cart(user.id, draft()).await.unwrap_err();
    assert!(matches!(
        err,
        RepoError::OutOfStock { available: 2, .. }
    ));
    // Không có gì thay đổi: không có đơn, kho giữ nguyên, giỏ còn nguyên.
    assert_eq!(orders.count().await?, 0);
    assert_eq!(products.find_by_id(enough.id).await?.map(|p| p.stock), Some(100));
    assert_eq!(products.find_by_id(scarce.id).await?.map(|p| p.stock), Some(2));
    assert_eq!(carts.list_with_products(user.id).await?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn order_decrements_stock_and_cancel_restores_it_once() -> Result<(), RepoError> {
    let store = MockStore::empty();
    let users = InMemoryUserRepository::new(store.clone());
    let products = InMemoryProductRepository::new(store.clone());
    let carts = InMemoryCartRepository::new(store.clone());
    let orders = InMemoryOrderRepository::new(store);

    let user = users.create(new_user("a@x.vn")).await?;
    let product = products.create(product_input("RAM", 100, None)).await?; // kho 100
    carts.add_item(user.id, product.id, 30).await?;
    let order = orders.create_from_cart(user.id, draft()).await?;
    assert_eq!(products.find_by_id(product.id).await?.map(|p| p.stock), Some(70));

    // Đổi sang trạng thái khác `cancelled` không đụng tới kho.
    orders.update_status(order.id, OrderStatus::Confirmed).await?;
    assert_eq!(products.find_by_id(product.id).await?.map(|p| p.stock), Some(70));

    // Hủy thì hoàn kho; đặt lại `cancelled` lần nữa không hoàn thêm.
    orders.update_status(order.id, OrderStatus::Cancelled).await?;
    assert_eq!(products.find_by_id(product.id).await?.map(|p| p.stock), Some(100));
    orders.update_status(order.id, OrderStatus::Cancelled).await?;
    assert_eq!(products.find_by_id(product.id).await?.map(|p| p.stock), Some(100));
    Ok(())
}

#[tokio::test]
async fn negative_stock_is_rejected_like_the_check_constraint() {
    let products = InMemoryProductRepository::new(MockStore::empty());
    let err = products
        .create(ProductInput {
            stock: -1,
            ..product_input("RAM", 100, None)
        })
        .await
        .unwrap_err();
    assert!(matches!(err, RepoError::Constraint(_)));
}

#[tokio::test]
async fn policies_are_sorted_validate_category_and_follow_category_delete() -> Result<(), RepoError> {
    use crate::models::policy::PolicyInput;
    use crate::repositories::PolicyRepository;

    let store = MockStore::empty();
    let categories = InMemoryCategoryRepository::new(store.clone());
    let policies = InMemoryPolicyRepository::new(store);
    let category = categories.create("CPU").await?;
    let input = |title: &str, category_id: Option<i64>, sort_order: i32| PolicyInput {
        icon: "shield".to_owned(),
        title: title.to_owned(),
        content: "nội dung".to_owned(),
        category_id,
        sort_order,
    };

    // Sắp theo sort_order, không theo thứ tự tạo.
    let late = policies.create(input("late", None, 20)).await?;
    let early = policies.create(input("early", Some(category.id), 10)).await?;
    let ids: Vec<i64> = policies.list().await?.iter().map(|policy| policy.id).collect();
    assert_eq!(ids, vec![early.id, late.id]);

    // Khóa ngoại và không tìm thấy.
    let err = policies.create(input("x", Some(999), 0)).await.unwrap_err();
    assert!(matches!(
        err,
        RepoError::InvalidReference {
            field: "category_id"
        }
    ));
    let err = policies.update(999, input("x", None, 0)).await.unwrap_err();
    assert!(matches!(err, RepoError::NotFound { entity: "policy" }));

    // Xóa danh mục thì chính sách riêng của nó xóa theo (ON DELETE CASCADE); chính sách chung còn.
    categories.delete(category.id).await?;
    let left = policies.list().await?;
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].id, late.id);

    policies.delete(late.id).await?;
    let err = policies.delete(late.id).await.unwrap_err();
    assert!(matches!(err, RepoError::NotFound { entity: "policy" }));
    Ok(())
}
