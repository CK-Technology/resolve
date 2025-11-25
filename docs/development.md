# Resolve Development Guide

## Prerequisites
- Rust 1.75+ with edition 2024 support
- PostgreSQL 13+
- Node.js 18+ (for frontend development)

## Environment Setup

### 1. Database Setup
```bash
# Create PostgreSQL database
createdb resolve

# Set environment variable for SQLx compile-time checking
export DATABASE_URL="postgresql://resolve:resolve@localhost:5432/resolve"
```

### 2. Backend Development

The backend uses SQLx with compile-time SQL verification. You must have `DATABASE_URL` set when building:

```bash
# Option 1: Use the build script
./build.sh

# Option 2: Set env manually
export DATABASE_URL="postgresql://resolve:resolve@localhost:5432/resolve"
cargo build

# Option 3: Create .env file
cd backend
cargo build
```

### 3. Run Migrations
```bash
cd backend
cargo run -- migrate
```

### 4. Start Development Server
```bash
cd backend
cargo run
```

The API will be available at `http://localhost:8080`

## API Endpoints

### Authentication
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - Login
- `GET /api/v1/auth/me` - Get current user

### Clients
- `GET /api/v1/clients` - List clients
- `POST /api/v1/clients` - Create client
- `GET /api/v1/clients/:id` - Get client
- `PUT /api/v1/clients/:id` - Update client
- `DELETE /api/v1/clients/:id` - Delete client

### Tickets
- `GET /api/v1/tickets` - List tickets
- `POST /api/v1/tickets` - Create ticket
- `GET /api/v1/tickets/:id` - Get ticket
- `PUT /api/v1/tickets/:id` - Update ticket
- `PATCH /api/v1/tickets/:id/assign` - Assign ticket
- `PATCH /api/v1/tickets/:id/close` - Close ticket

### Time Tracking
- `POST /api/v1/time/timer/start` - Start timer
- `POST /api/v1/time/timer/stop` - Stop timer
- `GET /api/v1/time/timer/active` - Get active timers
- `GET /api/v1/time/entries` - List time entries
- `GET /api/v1/time/stats` - Get time statistics

## Frontend Development

```bash
cd frontend
trunk serve
```

The frontend will be available at `http://localhost:8080`

## Docker Development

```bash
# Navigate to docker directory
cd deploy/docker

# Build and run with docker-compose
docker compose up -d

# View logs
docker compose logs -f

# Stop services
docker compose down

# Production deployment
docker compose -f docker-compose.prod.yml up -d
```

## Common Issues

### SQLx Compilation Errors
If you see "set DATABASE_URL to use query macros online", ensure:
1. PostgreSQL is running
2. Database exists: `createdb resolve`
3. DATABASE_URL is set: `export DATABASE_URL="postgresql://..."`

### Edition 2024 Errors
Ensure all Cargo.toml files use `edition = "2024"`

## Project Structure
```
resolve/
├── backend/          # Rust backend with Axum
├── frontend/         # WebAssembly frontend with Yew
├── shared/           # Shared types between backend and frontend
├── docker/           # Docker configuration files
├── docs/             # Documentation
└── scripts/          # Build and utility scripts
```

## Testing

```bash
# Run all tests
cargo test

# Run backend tests only
cd backend && cargo test

# Run with logging
RUST_LOG=debug cargo test
```

## Code Style

- Use `cargo fmt` before committing
- Run `cargo clippy` to check for common issues
- Follow Rust naming conventions
