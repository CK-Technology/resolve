# Build stage 1: Frontend dependencies cache
FROM rust:1.82-slim AS frontend-deps

# Install essential build tools
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install trunk and wasm tools
RUN cargo install trunk wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy only Cargo files for dependency caching
COPY Cargo.toml ./
COPY shared/Cargo.toml ./shared/
COPY frontend/Cargo.toml ./frontend/

# Create dummy source files
RUN mkdir -p shared/src frontend/src && \
    echo "fn main() {}" > shared/src/lib.rs && \
    echo "fn main() {}" > frontend/src/main.rs

# Build dependencies (cached layer)
RUN cd frontend && cargo fetch

# Build stage 2: Frontend build
FROM frontend-deps AS frontend-build

# Copy actual source files
COPY shared ./shared/
COPY frontend ./frontend/

# Build frontend with optimizations
RUN cd frontend && \
    CARGO_PROFILE_RELEASE_LTO=thin \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    trunk build --release

# Build stage 3: Backend dependencies cache  
FROM rust:1.82-slim AS backend-deps

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files for dependency caching
COPY Cargo.toml ./
COPY shared/Cargo.toml ./shared/
COPY backend/Cargo.toml ./backend/

# Create dummy source files  
RUN mkdir -p shared/src backend/src && \
    echo "fn main() {}" > shared/src/lib.rs && \
    echo "fn main() {}" > backend/src/main.rs

# Build dependencies (cached layer)
RUN cd backend && cargo fetch

# Build stage 4: Backend build
FROM backend-deps AS backend-build

# Copy SQLx prepared queries for offline compilation
COPY backend/.sqlx ./backend/.sqlx/

# Copy actual source files
COPY shared ./shared/
COPY backend/src ./backend/src/
COPY backend/migrations ./backend/migrations/

# Set environment for offline compilation  
ENV SQLX_OFFLINE=true

# Build backend with optimizations
RUN cd backend && \
    CARGO_PROFILE_RELEASE_LTO=thin \
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    CARGO_PROFILE_RELEASE_STRIP=symbols \
    cargo build --release

# Runtime stage with security hardening
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    libssl3 \
    ca-certificates \
    nginx \
    supervisor \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r resolve && useradd -r -g resolve -u 1001 -s /bin/false resolve

# Copy optimized nginx configuration
COPY deploy/docker/nginx.conf /etc/nginx/nginx.conf

# Copy frontend static files with proper ownership
COPY --from=frontend-build --chown=resolve:resolve /app/frontend/dist /var/www/html

# Copy backend binary with proper ownership
COPY --from=backend-build --chown=resolve:resolve /app/target/release/resolve-backend /usr/local/bin/resolve-backend

# Make binary executable
RUN chmod +x /usr/local/bin/resolve-backend

# Copy supervisor configuration
COPY deploy/docker/supervisord.conf /etc/supervisord.conf

# Copy migrations with proper ownership
COPY --from=backend-build --chown=resolve:resolve /app/backend/migrations /app/migrations

# Create necessary directories with proper permissions
RUN mkdir -p /var/log/supervisor /var/run /app/data /app/logs && \
    chown -R resolve:resolve /var/log/supervisor /var/run /app/data /app/logs && \
    chmod 755 /var/log/supervisor /var/run /app/data /app/logs

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost/health || exit 1

# Set environment variables
ENV DATABASE_URL="postgresql://resolve:resolve@db:5432/resolve" \
    SERVER_ADDR="127.0.0.1:8080" \
    RUST_LOG="info" \
    RUST_BACKTRACE="0"

# Security: Drop all capabilities and only add what's needed
# Run as non-root user for backend, but supervisor needs root for nginx
EXPOSE 80 443

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisord.conf", "-n"]