# Multi-stage build for minimal final image
FROM rust:1.89-slim AS builder

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
COPY examples ./examples
COPY tests ./tests

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
COPY --from=builder /app/target/release/template-rust /usr/local/bin/template-rust

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Set default database path
ENV DATABASE_URL=/app/data/todo.db

# Create data directory
RUN mkdir -p /app/data

# Expose any necessary ports (if needed for future web features)
# EXPOSE 8080

# Set the default command
ENTRYPOINT ["template-rust"]
CMD ["--help"]
