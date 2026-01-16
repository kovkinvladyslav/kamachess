FROM rust:1.88-bookworm AS builder

WORKDIR /app

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./



# Copy actual source code
COPY src ./src
COPY migrations ./migrations

# Build the actual application
RUN cargo build --release --features postgres --no-default-features

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/kamachess /usr/local/bin/kamachess

# Create logs directory
RUN mkdir -p /app/logs

ENV LOG_DIR=/app/logs
ENV RUST_LOG=info
ENV IMAGE_CACHE_SIZE_MB=100

CMD ["kamachess"]
