# Papaya CMS - Multi-stage Docker Build
# Build stage
FROM rust:1.89-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml ./
COPY backend/Cargo.toml backend/

# Create dummy source files to cache dependencies
RUN mkdir -p backend/src && echo "fn main() {}" > backend/src/main.rs
RUN cargo build --release --bin papaya-backend 2>/dev/null || true

# Copy actual source code
COPY backend/src ./backend/src

# Build the actual application
RUN cargo clean && cargo build --release --bin papaya-backend

# Runtime stage
FROM debian:trixie-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -u 1000 papaya

# Copy binary from builder
COPY --from=builder /app/target/release/papaya-backend /usr/local/bin/papaya-backend

# Copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Create templates directory
RUN mkdir -p /templates && chown -R papaya:papaya /templates

USER papaya

EXPOSE 8080

ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["papaya-backend"]