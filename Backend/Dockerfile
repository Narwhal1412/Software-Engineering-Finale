# 2 stage rust build
# stage 1: compile the rust binary
FROM rust:1-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

# Stage 2: Tiny runtime container
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/backend_api /app/backend_api
EXPOSE 8000
CMD ["/app/backend_api"]

