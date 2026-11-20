# BACKEND_ASSUMPTIONS – Giả định và bàn giao giữa các agent

File chung: các agent cùng ghi vào đây thay vì tự quyết im lặng. Mục "Giả định" của Agent 1 đánh số `A1…`;
Agent 2 và Agent 3 thêm mục của mình bên dưới (đừng sửa mục của người khác).

## Giả định

### Agent 1 – Nền tảng & DAL

**Cấu trúc và công cụ**

- **A1. Thư mục là `Backend/` (chữ B hoa), không phải `backend/`.** Repo thật đã có scaffold `Backend/` (Cargo.toml, Dockerfile, README, các file rỗng) nên code nằm ở đó; `docs/` ở gốc repo.
- **A2. Có thêm 3 file ngoài danh sách phạm vi**, vì bắt buộc kỹ thuật: `src/lib.rs` (tách lib + bin để `tests/` import được và để mọi `pub` chưa dùng không bị báo dead code làm hỏng `clippy -D warnings`), `src/app.rs` (hàm `build_app(state)` dựng router; test tích hợp gọi trực tiếp được), `Backend/.gitignore`.
- **A3. Các file rỗng có sẵn trong scaffold nhưng không được khai báo `mod`**, trình biên dịch bỏ qua: `Backend/src/db.rs` (dành cho người làm DB), `repositories/{user,product,order}.rs` (tên cũ, đã thay bằng `<entity>_repository.rs`; có thể xóa). Các `controllers/*_controller.rs` rỗng và `services/*_service.rs` rỗng là của Agent 2.
- **A4. `Cargo.lock` chưa có** (sandbox không có mạng). Hãy commit nó sau lần `cargo build` đầu tiên.

**Hợp đồng API (API.md mơ hồ → quyết định)**

- **A5. ID là số nguyên `i64` (JSON number), không phải `Uuid`.** API.md không nêu kiểu ID, nhưng frontend dùng `+f.categoryId` (Admin.jsx) và so sánh chặt `p.categoryId === c.id`, nên chuỗi UUID sẽ làm hỏng giao diện. Database dùng `BIGINT AUTO_INCREMENT`.
- **A6. Định dạng lỗi là `{ "message": "..." }`** đúng như API.md, không dùng định dạng dự phòng `{error:{code,message}}`. Lỗi 5xx trả thông báo chung, chi tiết chỉ ghi log. Lưu ý: lỗi do chính axum sinh ra (body JSON sai cú pháp, thiếu field) mặc định trả text thường, không phải `{message}`; frontend vẫn chịu được (rơi về thông báo mặc định) nhưng Agent 2 nên bọc extractor nếu muốn đồng nhất.
- **A7. 401 và 403 phải phân biệt.** Frontend tự đăng xuất khi nhận 401 lúc đang có token. Vì vậy: thiếu token/token sai/hết hạn/tài khoản đã bị xóa → **401**; đã đăng nhập nhưng không phải admin → **403** (`"Cần quyền quản trị"`). Extractor `AuthUser`/`AdminUser` đã làm đúng điều này.
- **A8. Đăng nhập sai mật khẩu nên trả 400, không phải 401** (khuyến nghị cho Agent 2): nếu trình duyệt còn token cũ, một 401 từ `/auth/login` sẽ kích hoạt đăng xuất sai ngữ cảnh. Thông báo: `"Sai email hoặc mật khẩu"`.
- **A9. `GET /api/products` và `GET /api/categories` là công khai** (trang cửa hàng xem được khi chưa đăng nhập). API.md chỉ đánh dấu `(admin)` cho các lệnh ghi.
- **A10. Cổng mặc định `3000`**, vì `vite.config.js` và `nginx.conf` của frontend trỏ tới 3000 (README gốc ví dụ 8080; đặt `APP_PORT` nếu muốn khác và sửa proxy tương ứng).
- **A11. Giá là số nguyên đồng VND** (`i64`), khớp dữ liệu demo của frontend.
- **A12. Không phân trang**: API.md không có tham số phân trang nên `ProductRepository::search` trả toàn bộ.

**Dữ liệu và ràng buộc**

- **A13. Xác thực = JWT HS256** (`middleware/token.rs`), claims `sub` (id), `role`, `iat`, `exp`; hạn mặc định 24 giờ (`JWT_TTL_HOURS`). **Quyền thật luôn đọc lại từ database**, không tin `role` trong token. Mật khẩu băm **Argon2id** (`middleware/password.rs`). Service PHẢI dùng `issue_token`, `hash_password`, `verify_password` ở đó (seed cũng dùng cùng hàm băm).
- **A14. Email duy nhất, không phân biệt hoa/thường.** Repository so sánh đúng như vậy nhưng KHÔNG tự chuẩn hóa; service nên `trim` email trước khi gọi.
- **A15. Tên danh mục duy nhất, không phân biệt hoa/thường** (API.md không nói; thêm cho tránh trùng lặp).
- **A16. Không xóa được sản phẩm đã nằm trong đơn hàng** (`RepoError::InUse` → HTTP 409) và **không xóa được thành viên đã có đơn** (409). API.md chỉ nói "không xóa được admin". Lý do: đơn cần `product_id`/`user_id` hợp lệ và doanh thu phải còn nguyên. Nếu nhóm muốn cho xóa, đổi `RESTRICT` thành `SET NULL`/`CASCADE` và cập nhật mock + DATABASE_SPEC.
- **A17. Cấm xóa admin là việc của service** (`UserService` kiểm tra `role == Admin` trước khi gọi `delete`). Repository không biết quy tắc này.
- **A18. Xóa sản phẩm tự xóa các dòng giỏ hàng chứa nó; xóa thành viên tự xóa giỏ của họ; xóa danh mục đặt `category_id = NULL`** (đúng API.md).
- **A19. Giỏ hàng:** `POST /cart/items` với sản phẩm đã có thì **cộng dồn**; `PATCH` **đặt** số lượng tuyệt đối và trả `NotFound` nếu dòng chưa có (frontend mock im lặng bỏ qua; backend báo lỗi rõ hơn); danh sách giỏ sắp theo `product_id`. Không có khái niệm tồn kho, không giới hạn trên của `quantity` (nếu cần giới hạn, đặt ở service).
- **A20. Sắp xếp đơn hàng: mới nhất trước** cho cả `GET /orders` và `GET /admin/orders`. API.md không nêu thứ tự; frontend mock trả đơn của người dùng theo thứ tự cũ→mới nhưng đảo ngược cho admin.
- **A21. Luật chuyển trạng thái đơn không có trong API.md.** Repository cho đổi sang mọi trạng thái hợp lệ; luật nghiệp vụ do service quyết định (xem B5).
- **A22. Thống kê admin:** `products`, `users` (gồm cả admin), `orders` (gồm cả đơn đã hủy) là số đếm thuần; `revenue` bỏ đơn `cancelled`. Khớp frontend mock.
- **A23. Validation nằm ở service**, dùng các hằng số trong `models/limits.rs` (độ dài cột). Quy tắc từ frontend mock cần giữ: số điện thoại `^[0-9]{9,11}$`, mật khẩu nhập lại phải khớp (`confirmPassword`), `displayName` bắt buộc, `quantity >= 1`, `paymentMethod ∈ {cod, bank}`, họ tên/địa chỉ đơn hàng bắt buộc. Mock chỉ mô phỏng giới hạn cột và CHECK, không thay thế validation.
- **A24. Upload ảnh không đi qua DB.** `POST /api/uploads` (Agent 2) ghi tệp vào `config.upload_dir` với tên ngẫu nhiên và trả `{url: "/uploads/<tên>"}`; `app.rs` đã phục vụ `/uploads/*` bằng `ServeDir`. Agent 2 phải tự kiểm tra loại tệp (jpg/png/webp), giới hạn 2 MB và đặt `DefaultBodyLimit` đủ lớn cho multipart (mặc định axum là 2 MB, thấp hơn 2 MB + phần đầu multipart). Cargo đã bật feature `multipart` của axum.
- **A25. `userEmail` chỉ có trong `GET /api/admin/orders`**; lấy từ `OrderRepository::list_all_with_user_email`.

**Mock**

- **A26. Mock mất dữ liệu khi tắt server.** Seed lại mỗi lần khởi động. `main.rs` in tài khoản seed ra log (chỉ phù hợp dev; bỏ dòng log này khi dùng DB thật).
- **A27. Mock tìm kiếm phân biệt dấu**, còn MariaDB (`utf8mb4_unicode_ci`) thì không; DB thật hành xử rộng hơn, chấp nhận được.

### Agent 2 – Services & Controllers

**Hợp đồng HTTP (API.md không nêu rõ → quyết định)**

- **B1. Mã thành công:** `POST` tạo mới (`/auth/register`, `/categories`, `/products`, `/orders`, `/uploads`) → **201**; `DELETE` sản phẩm/danh mục/thành viên → **204** không body; mọi lệnh còn lại → **200**. Các lệnh giỏ hàng (kể cả `POST` và `DELETE`) → 200 kèm giỏ hàng mới, đúng API.md. Frontend chỉ kiểm tra `r.ok` và đọc JSON có `.catch`, nên 204 an toàn.
- **B2. Mã lỗi:** 400 dữ liệu sai/thiếu; 401 thiếu hoặc sai token; 403 không đủ quyền (và xóa admin); 404 không có tài nguyên; 409 trùng hoặc đang được tham chiếu hoặc đổi trạng thái đơn đã chốt; 413 ảnh > 2 MB; 415 ảnh sai định dạng. Mọi lỗi có dạng `{message}` tiếng Việt, kể cả body JSON sai cú pháp (extractor `ApiJson`). Lỗi `Path` không phải số (ví dụ `/products/abc`) vẫn là văn bản thuần 400 của axum.
- **B3. Đăng nhập sai trả 400** `"Sai email hoặc mật khẩu"` (thực hiện A8). Đăng ký công khai luôn tạo vai trò `user`.
- **B4. Phân quyền nằm ở service**, không ở route: controller chỉ khai báo `AuthUser` (xác thực), service gọi `require_admin`. Nhờ vậy quy tắc test được không cần HTTP. Extractor `AdminUser` của Agent 1 hiện chưa được dùng.
- **B5. Luật đổi trạng thái đơn:** `completed` và `cancelled` là trạng thái cuối, không chuyển sang trạng thái khác (409); đặt lại đúng trạng thái hiện tại thì được; các chuyển khác (kể cả nhảy cóc) cho phép vì UI admin cho chọn tự do. Nếu nhóm muốn luật chặt hơn (ví dụ chỉ tiến theo thứ tự), sửa duy nhất `ensure_transition_allowed` trong `order_service.rs`.
- **B6. Không có tồn kho** (API.md và schema không có trường tồn kho) nên không kiểm tra tồn kho khi đặt hàng. Tổng tiền do repository tính từ giá hiện hành, không lấy từ client.
- **B7. Validation cụ thể:** mật khẩu 6–128 ký tự; email đúng dạng cơ bản và ≤ 254 ký tự; số điện thoại 9–11 chữ số (hồ sơ cho phép để trống); giá sản phẩm `0 < price ≤ 100 tỷ`; `specs` ≤ 5000 ký tự; số lượng giỏ hàng 1–99 (cộng dồn vượt 99 thì kẹp về 99, không báo lỗi); `imageUrl` rỗng → `null`, còn lại phải bắt đầu bằng `/uploads/`, `http://` hoặc `https://`.
- **B8. `GET /products`:** `categoryId` rỗng (frontend luôn gửi `categoryId=`) nghĩa là không lọc; không phải số → 400; `sort` lạ → coi như `new`. `PATCH /products/:id/pin` bắt buộc có `pinned` (thiếu → 400, tránh vô tình bỏ ghim).
- **B9. Upload:** định dạng nhận diện bằng chữ ký byte đầu tệp (JPEG/PNG/WebP), không tin `Content-Type` client gửi; tên tệp do server sinh; tệp lưu ở `UPLOAD_DIR`, trả `{url: "/uploads/<tên>"}`. Đã đặt giới hạn body riêng cho route này (2 MB + 64 KB).
- **B10. Băm mật khẩu chạy trong `spawn_blocking`** để Argon2 không làm nghẽn các request khác.

**Thay đổi ngoài phạm vi thuần túy (tối thiểu, có lý do)**

- `Cargo.toml`: thêm `[dev-dependencies] tower = "=0.5.2"` (feature `util`) cho test `oneshot`.
- `lib.rs`: thêm `pub mod dtos;`. `app.rs`: thêm `.merge(build_router())`. `app_state.rs`: thêm trường `services` và dựng trong `AppState::new`. `main.rs` không đổi.
- Hai file khai báo ở mục này không đụng tới `models/` hay `repositories/`.

**Yêu cầu bổ sung cho DAL**

- Không có. Mọi nghiệp vụ dùng được các hàm trait hiện có (không thêm hàm nào vào trait hay mock).


## Hợp đồng bàn giao cho Agent 2 (Services / Controllers / DTOs)

- Struct service kỳ vọng (mỗi struct giữ `Arc<dyn <Entity>Repository>` lấy từ `AppState.repos`):

  | Struct | Repository dùng | Endpoint phụ trách |
  |---|---|---|
  | `AuthService` | `UserRepository` (+ `issue_token`, `hash_password`, `verify_password`) | `POST /api/auth/register`, `POST /api/auth/login` |
  | `UserService` | `UserRepository` | `GET/PUT /api/me`, `GET /api/admin/users`, `DELETE /api/admin/users/:id` |
  | `CategoryService` | `CategoryRepository` | `/api/categories*` |
  | `ProductService` | `ProductRepository`, `CategoryRepository` | `/api/products*` |
  | `UploadService` | (không dùng repository; ghi đĩa) | `POST /api/uploads` |
  | `CartService` | `CartRepository`, `ProductRepository` | `/api/cart*` |
  | `OrderService` | `OrderRepository`, `CartRepository` | `POST/GET /api/orders`, `GET /api/admin/orders`, `PATCH /api/admin/orders/:id` |
  | `StatsService` | `ProductRepository`, `UserRepository`, `OrderRepository` | `GET /api/admin/stats` |

  Tên cụ thể do Agent 2 quyết định; bảng chỉ là kỳ vọng để Agent 3 viết tài liệu/test.
- Mỗi controller khai báo `pub fn router() -> Router<AppState>`; `controllers/mod.rs` có `build_router()` gộp các router và tự thêm tiền tố `/api` (dùng `Router::nest("/api", ...)`).
- **Nối vào ứng dụng:** trong `src/app.rs`, thêm đúng một dòng `.merge(crate::controllers::build_router())` sau route `/health` (đã có chú thích); trong `src/lib.rs` thêm `pub mod dtos;`.
- **Thêm services vào `AppState`:** `app_state.rs` có chú thích chỗ thêm. Agent 2 được phép sửa DUY NHẤT struct `AppState` và `AppState::new` để thêm trường `services`; không đụng `build_repositories`.
- Controller dùng extractor `AuthUser` (cần đăng nhập) / `AdminUser(admin)` từ `middleware::auth`; không tự đọc header.
- Service trả `Result<T, AppError>`; dùng `?` trên `Result<_, RepoError>` (đã có `From<RepoError> for AppError` với thông báo tiếng Việt). `map_err` khi cần thông báo cụ thể hơn.
- DTO dùng `#[serde(rename_all = "camelCase")]`; `User` KHÔNG derive `Serialize` — phải map sang DTO để không lộ `password_hash`. Enum (`Role`, `OrderStatus`, `PaymentMethod`) đã có serde chữ thường, dùng thẳng được. `DateTime<Utc>` serialize ra ISO-8601 (có feature `serde` của chrono).
- `OrderRepository::create_from_cart` đã làm trọn gói (đọc giỏ, snapshot giá, tạo đơn, xóa giỏ). Service chỉ validate thông tin giao hàng rồi gọi nó; **không** tự đọc giỏ rồi tạo đơn từng bước.

## Ghi chú cho Agent 3 (Docs / Bruno)

- Tài khoản seed: `admin@pc.vn` (admin), `khach1@pc.vn`, `khach2@pc.vn`, `khach3@pc.vn` (user); mật khẩu chung `123456`. `khach1` có giỏ hàng 2 dòng và 4 đơn; `khach3` không có đơn.
- Số liệu thống kê mong đợi trên dữ liệu seed: `{products: 7, users: 4, orders: 7, revenue: 118370000}`.
- Thứ tự `GET /api/products` mặc định trên seed: "Ryzen 7 7800X3D" (đang ghim) đứng đầu, sau đó mới → cũ: PC Gaming Forge R7, Samsung 980 Pro 1TB, Kingston Fury 32GB, Radeon RX 7800 XT, GeForce RTX 4070 Super, Core i5-14600K.
- ER diagram: theo mục 2–3 của `DATABASE_SPEC.md`.

## Việc Agent 1 chưa kiểm chứng được

Môi trường của Agent 1 và Agent 2 không có Rust toolchain và không có mạng, nên **chưa chạy** `cargo build`, `cargo fmt`, `cargo clippy -- -D warnings` và `cargo test`. Code (cả DAL, services, controllers và test) đã được rà soát thủ công kỹ nhưng chưa được biên dịch hay chạy thử. Việc đầu tiên cần làm trên máy có Rust: chạy `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test && cargo run`, rồi `curl localhost:3000/health`. Nếu có lỗi biên dịch/version, sửa tại chỗ và ghi lại ở đây.
