# =============================================================================
# Dockerfile của Backend (Rust/axum). Build ngữ cảnh = thư mục Backend/
# (khớp .woodpecker/test-publish.yaml: dockerfile Backend/Dockerfile, context Backend).
# Hai giai đoạn: (1) biên dịch bản release, (2) image chạy nhỏ gọn, không chạy bằng root.
# =============================================================================

# ---- Giai đoạn 1: biên dịch ----
# "rust:1-bookworm" = Rust 1.x mới nhất. Sau khi build thành công lần đầu, nên ghim số cụ thể
# (ví dụ rust:1.88-bookworm) để mọi người và CI build giống nhau.
FROM rust:1-bookworm AS build
WORKDIR /app

# Bước đệm để cache thư viện: chỉ chép Cargo.toml (+ Cargo.lock nếu đã commit), dựng mã giả rồi build.
# Nhờ đó khi chỉ sửa code trong src/, Docker dùng lại lớp đã biên dịch xong các thư viện (nhanh hơn nhiều).
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src \
 && echo '' > src/lib.rs \
 && echo 'fn main() {}' > src/main.rs \
 && cargo build --release \
 && rm -rf src

# Chép mã thật. `touch` để cargo nhận ra file đã đổi so với mã giả ở trên.
COPY src ./src
RUN touch src/lib.rs src/main.rs && cargo build --release

# ---- Giai đoạn 2: image chạy ----
FROM debian:bookworm-slim

# curl chỉ để HEALTHCHECK gọi /health; ca-certificates cho kết nối TLS (nếu sau này DB/API ngoài dùng TLS).
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*

# Chạy bằng user thường, không dùng root.
RUN useradd --system --uid 10001 --home-dir /app app
WORKDIR /app

COPY --from=build /app/target/release/computer_shop_backend /usr/local/bin/computer_shop_backend

# Thư mục lưu ảnh upload (UPLOAD_DIR). Tạo sẵn và gán quyền cho user "app" để volume
# gắn vào /app/uploads thừa hưởng đúng quyền ghi ở lần mount đầu tiên.
RUN mkdir -p /app/uploads && chown -R app:app /app
USER app

# Cổng PHẢI là 3000 vì Frontend/nginx.conf proxy tới http://backend:3000.
ENV APP_PORT=3000 \
    UPLOAD_DIR=/app/uploads
EXPOSE 3000

HEALTHCHECK --interval=15s --timeout=3s --start-period=10s --retries=3 \
  CMD curl -fsS http://localhost:3000/health || exit 1

CMD ["computer_shop_backend"]
