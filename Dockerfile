# Multi-stage build for minimal final image
FROM rust:1.89-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty project
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/open-gateway /usr/local/bin/open-gateway

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Create config directory
RUN mkdir -p /app/config

# Expose the gateway port
EXPOSE 8080

# Set the default command
ENTRYPOINT ["open-gateway"]
CMD ["--help"]
