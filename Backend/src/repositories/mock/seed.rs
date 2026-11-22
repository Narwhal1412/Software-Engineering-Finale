//! Dữ liệu mẫu cho bản mock. Nội dung lấy theo dữ liệu demo của frontend để
//! giao diện hiển thị quen thuộc; SQL tương đương nằm trong docs/DATABASE_SPEC.md.

use chrono::{Duration, Utc};

use super::store::Tables;
use crate::middleware::password::hash_password;
use crate::models::category::Category;
use crate::models::enums::{OrderStatus, PaymentMethod, Role};
use crate::models::cart::CartItem;
use crate::models::order::{Order, OrderItem};
use crate::models::policy::Policy;
use crate::models::product::Product;
use crate::models::user::User;
use crate::repositories::RepoError;

/// Email tài khoản admin mẫu (trùng tài khoản demo của frontend).
pub const SEED_ADMIN_EMAIL: &str = "admin@pc.vn";
/// Email các tài khoản khách hàng mẫu; khách thứ ba chưa có đơn hàng nên xóa thử được.
pub const SEED_CUSTOMER_EMAILS: [&str; 3] = ["khach1@pc.vn", "khach2@pc.vn", "khach3@pc.vn"];
/// Mật khẩu của mọi tài khoản mẫu. Chỉ dùng để demo.
pub const SEED_PASSWORD: &str = "123456";

/// Mô tả một sản phẩm mẫu; thời điểm tính bằng "số ngày trước".
struct SeedProduct {
    name: &'static str,
    specs: &'static str,
    price: i64,
    stock: i32,
    category_id: i64,
    pinned_days_ago: Option<i64>,
    created_days_ago: i64,
}

/// Mô tả một đơn hàng mẫu.
struct SeedOrder {
    days_ago: i64,
    user_id: i64,
    status: OrderStatus,
    payment_method: PaymentMethod,
    /// Danh sách `(product_id, quantity)`.
    lines: &'static [(i64, i32)],
}

/// Mô tả một chính sách mẫu.
struct SeedPolicy {
    icon: &'static str,
    title: &'static str,
    content: &'static str,
    /// Danh mục áp dụng (id 1..=5); `None` = mọi sản phẩm.
    category_id: Option<i64>,
    sort_order: i32,
}

const SEED_POLICIES: [SeedPolicy; 11] = [
    SeedPolicy {
        icon: "shield",
        title: "Bảo hành chính hãng 36 tháng",
        content: "Lỗi do nhà sản xuất được 1 đổi 1 trong 30 ngày đầu, sau đó bảo hành theo chính sách của hãng.",
        category_id: Some(1),
        sort_order: 10,
    },
    SeedPolicy {
        icon: "shield",
        title: "Bảo hành chính hãng 36 tháng",
        content: "Lỗi do nhà sản xuất được 1 đổi 1 trong 30 ngày đầu, sau đó bảo hành theo chính sách của hãng.",
        category_id: Some(2),
        sort_order: 10,
    },
    SeedPolicy {
        icon: "shield",
        title: "Bảo hành chính hãng 36 tháng",
        content: "Lỗi do nhà sản xuất được 1 đổi 1 trong 30 ngày đầu, sau đó bảo hành theo chính sách của hãng.",
        category_id: Some(3),
        sort_order: 10,
    },
    SeedPolicy {
        icon: "shield",
        title: "Bảo hành chính hãng 36 tháng",
        content: "Lỗi do nhà sản xuất được 1 đổi 1 trong 30 ngày đầu, sau đó bảo hành theo chính sách của hãng.",
        category_id: Some(4),
        sort_order: 10,
    },
    SeedPolicy {
        icon: "shield",
        title: "Bảo hành PC lắp sẵn 24 tháng",
        content: "Bảo hành toàn bộ máy 24 tháng, linh kiện lỗi được thay thế trong 30 ngày đầu.",
        category_id: Some(5),
        sort_order: 10,
    },
    SeedPolicy {
        icon: "refresh",
        title: "Đổi trả trong 7 ngày",
        content: "Đổi hoặc hoàn tiền nếu sản phẩm lỗi kỹ thuật hoặc giao sai. Sản phẩm cần còn đủ hộp và phụ kiện.",
        category_id: None,
        sort_order: 20,
    },
    SeedPolicy {
        icon: "truck",
        title: "Giao hàng toàn quốc",
        content: "Miễn phí giao hàng. Nội thành giao trong 24 giờ, tỉnh khác 2 - 4 ngày.",
        category_id: None,
        sort_order: 30,
    },
    SeedPolicy {
        icon: "check",
        title: "Kiểm tra hàng khi nhận",
        content: "Được mở hộp kiểm tra ngoại quan và số seri trước khi thanh toán.",
        category_id: None,
        sort_order: 40,
    },
    SeedPolicy {
        icon: "card",
        title: "Thanh toán linh hoạt",
        content: "Thanh toán khi nhận hàng hoặc chuyển khoản ngân hàng.",
        category_id: None,
        sort_order: 50,
    },
    SeedPolicy {
        icon: "gift",
        title: "Ưu đãi thành viên",
        content: "Tài khoản thành viên theo dõi đơn hàng và nhận thông báo khuyến mãi sớm.",
        category_id: None,
        sort_order: 60,
    },
    SeedPolicy {
        icon: "phone",
        title: "Hỗ trợ kỹ thuật",
        content: "Tư vấn lắp đặt, cài đặt: gọi 1900 0001 (8:00 - 21:00).",
        category_id: None,
        sort_order: 70,
    },
];

const SEED_CATEGORIES: [&str; 5] = ["CPU", "Card đồ họa", "RAM", "Ổ cứng", "PC lắp sẵn"];

const SEED_PRODUCTS: [SeedProduct; 7] = [
    SeedProduct {
        name: "Ryzen 7 7800X3D",
        specs: "8 nhân 16 luồng, 4.2 GHz, AM5",
        price: 9_490_000,
        stock: 10,
        category_id: 1,
        pinned_days_ago: Some(1),
        created_days_ago: 9,
    },
    SeedProduct {
        name: "Core i5-14600K",
        specs: "14 nhân, 5.3 GHz, LGA1700",
        price: 7_290_000,
        stock: 15,
        category_id: 1,
        pinned_days_ago: None,
        created_days_ago: 8,
    },
    SeedProduct {
        name: "GeForce RTX 4070 Super",
        specs: "12 GB GDDR6X, 3 quạt",
        price: 16_990_000,
        stock: 5,
        category_id: 2,
        pinned_days_ago: None,
        created_days_ago: 6,
    },
    SeedProduct {
        name: "Radeon RX 7800 XT",
        specs: "16 GB GDDR6, PCIe 4.0",
        price: 12_790_000,
        stock: 8,
        category_id: 2,
        pinned_days_ago: None,
        created_days_ago: 5,
    },
    SeedProduct {
        name: "Kingston Fury 32GB",
        specs: "DDR5 6000 MHz, 2x16 GB",
        price: 2_890_000,
        stock: 20,
        category_id: 3,
        pinned_days_ago: None,
        created_days_ago: 4,
    },
    SeedProduct {
        name: "Samsung 980 Pro 1TB",
        specs: "NVMe Gen4, đọc 7000 MB/s",
        price: 2_650_000,
        stock: 12,
        category_id: 4,
        pinned_days_ago: None,
        created_days_ago: 3,
    },
    SeedProduct {
        name: "PC Gaming Forge R7",
        specs: "R7 7800X3D, RTX 4070 Super, 32GB, 1TB",
        price: 36_900_000,
        stock: 3,
        category_id: 5,
        pinned_days_ago: None,
        created_days_ago: 2,
    },
];

const SEED_ORDERS: [SeedOrder; 7] = [
    SeedOrder {
        days_ago: 0,
        user_id: 2,
        status: OrderStatus::Completed,
        payment_method: PaymentMethod::Cod,
        lines: &[(1, 1), (5, 2)],
    },
    SeedOrder {
        days_ago: 1,
        user_id: 3,
        status: OrderStatus::Completed,
        payment_method: PaymentMethod::Bank,
        lines: &[(3, 1)],
    },
    SeedOrder {
        days_ago: 2,
        user_id: 2,
        status: OrderStatus::Shipped,
        payment_method: PaymentMethod::Cod,
        lines: &[(7, 1)],
    },
    SeedOrder {
        days_ago: 2,
        user_id: 3,
        status: OrderStatus::Confirmed,
        payment_method: PaymentMethod::Bank,
        lines: &[(2, 1), (6, 1)],
    },
    SeedOrder {
        days_ago: 4,
        user_id: 2,
        status: OrderStatus::Pending,
        payment_method: PaymentMethod::Cod,
        lines: &[(4, 1)],
    },
    SeedOrder {
        days_ago: 5,
        user_id: 3,
        status: OrderStatus::Cancelled,
        payment_method: PaymentMethod::Cod,
        lines: &[(5, 1)],
    },
    SeedOrder {
        days_ago: 6,
        user_id: 2,
        status: OrderStatus::Completed,
        payment_method: PaymentMethod::Bank,
        lines: &[(1, 1), (3, 1)],
    },
];

/// Dựng toàn bộ bảng với dữ liệu mẫu. Thứ tự gọi quan trọng vì có khóa ngoại.
pub(super) fn seed_tables() -> Result<Tables, RepoError> {
    let mut tables = Tables::default();
    seed_users(&mut tables)?;
    seed_categories(&mut tables);
    seed_policies(&mut tables);
    seed_products(&mut tables);
    seed_carts(&mut tables);
    for spec in &SEED_ORDERS {
        seed_order(&mut tables, spec)?;
    }
    Ok(tables)
}

/// Tạo admin (id 1) và ba khách hàng (id 2..=4), mỗi tài khoản có muối băm riêng.
fn seed_users(tables: &mut Tables) -> Result<(), RepoError> {
    let accounts = [
        (SEED_ADMIN_EMAIL, "Quản trị", Role::Admin),
        (SEED_CUSTOMER_EMAILS[0], "Nguyễn Văn An", Role::User),
        (SEED_CUSTOMER_EMAILS[1], "Trần Thị Bình", Role::User),
        (SEED_CUSTOMER_EMAILS[2], "Lê Hoàng Cường", Role::User),
    ];
    for (email, display_name, role) in accounts {
        let password_hash = hash_password(SEED_PASSWORD)
            .map_err(|err| RepoError::Internal(format!("không băm được mật khẩu seed: {err}")))?;
        let id = tables.sequences.user.next();
        tables.users.insert(
            id,
            User {
                id,
                email: email.to_owned(),
                display_name: display_name.to_owned(),
                password_hash,
                phone: String::new(),
                address: String::new(),
                role,
                created_at: Utc::now() - Duration::days(10),
            },
        );
    }
    Ok(())
}

/// Tạo các danh mục mẫu (id 1..=5).
fn seed_categories(tables: &mut Tables) {
    for name in SEED_CATEGORIES {
        let id = tables.sequences.category.next();
        tables.categories.insert(
            id,
            Category {
                id,
                name: name.to_owned(),
            },
        );
    }
}

/// Tạo các chính sách mẫu (id 1..=11); phải gọi sau `seed_categories` vì có khóa ngoại.
fn seed_policies(tables: &mut Tables) {
    for spec in &SEED_POLICIES {
        let id = tables.sequences.policy.next();
        tables.policies.insert(
            id,
            Policy {
                id,
                icon: spec.icon.to_owned(),
                title: spec.title.to_owned(),
                content: spec.content.to_owned(),
                category_id: spec.category_id,
                sort_order: spec.sort_order,
            },
        );
    }
}

/// Tạo các sản phẩm mẫu (id 1..=7), sản phẩm đầu tiên đang được ghim.
fn seed_products(tables: &mut Tables) {
    let now = Utc::now();
    for spec in &SEED_PRODUCTS {
        let id = tables.sequences.product.next();
        tables.products.insert(
            id,
            Product {
                id,
                name: spec.name.to_owned(),
                specs: spec.specs.to_owned(),
                price: spec.price,
                stock: spec.stock,
                category_id: Some(spec.category_id),
                image_url: None,
                pinned_at: spec.pinned_days_ago.map(|days| now - Duration::days(days)),
                created_at: now - Duration::days(spec.created_days_ago),
            },
        );
    }
}

/// Cho khách 1 vài món trong giỏ để demo `GET /api/cart`.
fn seed_carts(tables: &mut Tables) {
    for (product_id, quantity) in [(6, 1), (2, 2)] {
        tables.cart_items.insert(
            (2, product_id),
            CartItem {
                user_id: 2,
                product_id,
                quantity,
            },
        );
    }
}

/// Tạo một đơn mẫu, chụp tên và giá sản phẩm hiện tại như khi đặt hàng thật.
fn seed_order(tables: &mut Tables, spec: &SeedOrder) -> Result<(), RepoError> {
    let full_name = tables
        .users
        .get(&spec.user_id)
        .map(|user| user.display_name.clone())
        .ok_or(RepoError::NotFound { entity: "user" })?;

    let order_id = tables.sequences.order.next();
    let mut items = Vec::with_capacity(spec.lines.len());
    let mut total: i64 = 0;
    for &(product_id, quantity) in spec.lines {
        let product = tables
            .products
            .get(&product_id)
            .ok_or(RepoError::NotFound { entity: "product" })?;
        total += product.price * i64::from(quantity);
        items.push(OrderItem {
            id: tables.sequences.order_item.next(),
            order_id,
            product_id,
            name: product.name.clone(),
            price: product.price,
            quantity,
        });
    }

    tables.orders.insert(
        order_id,
        Order {
            id: order_id,
            user_id: spec.user_id,
            items,
            total,
            status: spec.status,
            payment_method: spec.payment_method,
            full_name,
            phone: "0900000000".to_owned(),
            address: "TP.HCM".to_owned(),
            created_at: Utc::now() - Duration::days(spec.days_ago),
        },
    );
    Ok(())
}
