# syntax=docker/dockerfile:1

# ============================================================================
# Builder Stage - Build with cache optimization
# ============================================================================
FROM rustlang/rust:nightly-bookworm-slim AS builder

WORKDIR /app

# Install required dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev curl && \
    rm -rf /var/lib/apt/lists/*

# Copy all source code
COPY Cargo.toml Cargo.lock ./
COPY backend backend/

# Build application with BuildKit cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release -p cobalt-stack-backend && \
    cp /app/target/release/cobalt-stack-backend /app/cobalt-stack-backend

# ============================================================================
# Runtime Stage - Debian Slim for compatibility
# ============================================================================
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Copy binary from builder
COPY --from=builder --chown=appuser:appuser /app/cobalt-stack-backend /app/cobalt-stack-backend

# Expose port
EXPOSE 3000

# Run as non-root user
USER appuser

# Start application
CMD ["/app/cobalt-stack-backend"]
