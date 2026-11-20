//! Kết nối database thật (MariaDB) – HIỆN CHƯA CÓ CODE, backend đang chạy bằng mock.
//!
//! File này được khai báo sẵn (`pub mod db;` trong `lib.rs`) để người làm database
//! chỉ cần điền nội dung, không phải sửa thêm chỗ nào khác. Khi tích hợp DB, đặt vào đây:
//! - `connect(config) -> Result<MySqlPool, RepoError>`: dựng pool từ `config.db_*`;
//! - `connect_with_retry(...)`: thử lại vài lần vì khi `docker compose up`, MariaDB
//!   thường khởi động chậm hơn backend.
//!
//! Mã mẫu đầy đủ và giải thích nằm ở `docs/DATABASE_SPEC.md`, mục 7.3.
