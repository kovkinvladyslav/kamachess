FROM rust:1.88-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release --features postgres --no-default-features

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/kamachess /usr/local/bin/kamachess

RUN mkdir -p /app/logs

ENV LOG_DIR=/app/logs
ENV RUST_LOG=info
ENV IMAGE_CACHE_SIZE_MB=100

CMD ["kamachess"]
