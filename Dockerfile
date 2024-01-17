# Build stage
FROM rust:1.75-bookworm as builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin app

# Runtime stage
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/app /
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
