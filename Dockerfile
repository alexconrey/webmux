# Multi-stage build for WebMux

# Stage 1: Builder
FROM rust:1.85-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binaries
RUN cargo build --release --bins

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    socat \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /usr/src/app/target/release/webmux /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/webmux-cli /usr/local/bin/
COPY --from=builder /usr/src/app/target/release/mock_device /usr/local/bin/

# Copy configuration files
COPY config.example.yaml ./
COPY config.virtual.yaml ./

# Copy scripts
COPY scripts ./scripts
RUN chmod +x scripts/*.sh

# Copy static files for web frontend
COPY static ./static

# Create logs directory
RUN mkdir -p /app/logs

# Expose the web server port
EXPOSE 8080

# Set the entrypoint
ENTRYPOINT ["/app/scripts/docker-entrypoint.sh"]

# Default command
CMD ["config.virtual.yaml"]
