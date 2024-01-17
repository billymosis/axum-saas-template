# Old ~100mb
# FROM rust:1.75-bookworm as builder
# WORKDIR /app
# COPY . .
# ENV SQLX_OFFLINE true
# RUN cargo build --release --bin app
#
# # Runtime stage
# FROM debian:bookworm-slim AS runtime
# RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
# COPY --from=builder /app/target/release/app /
# COPY entrypoint.sh /entrypoint.sh
# RUN chmod +x /entrypoint.sh
# ENTRYPOINT ["/entrypoint.sh"]

# Using the `rust-musl-builder` as base image, instead of 
# the official Rust toolchain

# New ~30mb
FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --target x86_64-unknown-linux-musl --bin axum-saas-template

FROM alpine AS runtime
RUN addgroup -S myuser && adduser -S myuser -G myuser
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/axum-saas-template /usr/local/bin/
USER myuser
COPY entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
