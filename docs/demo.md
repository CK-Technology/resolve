# Resolve Demo Deployment

A comprehensive MSP management platform built with Rust + WebAssembly, featuring all the tools you need to manage clients, tickets, assets, documentation, and finances.

## Features Included

### Core MSP Features
- **Client Management** - Complete client database with contacts and locations
- **Ticketing System** - Full-featured helpdesk with SLA tracking
- **Asset Management** - Hardware inventory with health monitoring
- **Time Tracking** - Billable hours with project and ticket integration
- **Financial Management** - Invoicing, recurring billing, and profitability

### Advanced Features
- **Documentation System** - Rich knowledge base with templates (like Hudu)
- **Password Vault** - Secure credential management with sharing
- **Asset Discovery** - Network scanning and auto-discovery
- **Communication Hub** - Email-to-ticket, SMS, client portal
- **Automation & Workflows** - Scheduled tasks and alert rules
- **Reporting & Analytics** - Executive dashboards and KPIs

## Quick Start

### Prerequisites
- Docker Engine 20.10+
- Docker Compose v2
- 4GB RAM minimum
- 10GB disk space

### 1. Clone and Deploy
```bash
git clone https://github.com/CK-Technology/resolve.git
cd resolve

# Start the demo (includes database, app, and optional services)
docker compose -f docker-compose.demo.yml up -d

# Optional: Start with admin tools
docker compose -f docker-compose.demo.yml --profile admin --profile mail up -d
```

### 2. Access the Application
- **Main Interface**: http://localhost:8080
- **Admin Login**: `admin@resolve.demo` / `demo123`
- **Tech Login**: `tech@resolve.demo` / `demo123`

### 3. Optional Services
- **Database Admin**: http://localhost:8081 (Adminer)
- **Mail Catcher**: http://localhost:8025 (MailHog)

## Demo Accounts & Data

### User Accounts
| Email | Password | Role | Description |
|-------|----------|------|-------------|
| admin@resolve.demo | demo123 | Admin | Full system access |
| tech@resolve.demo | demo123 | Technician | Technical support role |
| sarah@resolve.demo | demo123 | Technician | Technical support role |
| mike@resolve.demo | demo123 | Manager | Team management |
| billing@resolve.demo | demo123 | Billing | Financial operations |

### Sample Clients
- **Acme Corporation** - Large enterprise (200+ employees)
- **TechStart Inc** - Growing startup with cloud infrastructure
- **Local Law Firm** - Professional services requiring high security

### Demo Data Includes
- **25+ Tickets** across different priorities and statuses
- **Asset Inventory** with health scores and warranties
- **Documentation** with templates and client-specific content
- **Password Vault** with sample secure entries
- **Financial Data** including invoices and recurring billing
- **Time Entries** and project tracking
- **KPI Metrics** and dashboard analytics

## Architecture

### Technology Stack
- **Backend**: Rust (Axum web framework)
- **Frontend**: WebAssembly (Yew framework)
- **Database**: PostgreSQL 15 with full-text search
- **Cache**: Redis for sessions and caching
- **Deployment**: Docker with Nginx reverse proxy

### Security Features
- JWT-based authentication
- Password hashing with bcrypt
- Encrypted credential storage
- SQL injection protection
- CORS and security headers
- Input validation and sanitization

## Feature Comparison

| Feature | Resolve | ITFlow | Hudu | Commercial BMS |
|---------|---------|--------|------|----------------|
| **Open Source** | Yes | Yes | No | No |
| **Self-Hosted** | Yes | Yes | Yes | No |
| **Modern Tech Stack** | Yes (Rust/WASM) | No (PHP) | No (Ruby) | No |
| **Rich Documentation** | Yes | Basic | Yes | Yes |
| **Asset Discovery** | Yes | Basic | Yes | Yes |
| **Password Management** | Yes | Basic | No | Yes |
| **Automation/Workflows** | Yes | Basic | No | Yes |
| **Client Portal** | Yes | Yes | Yes | Yes |
| **Advanced Reporting** | Yes | Basic | Basic | Yes |

## Configuration

### Environment Variables
```bash
# Database
DATABASE_URL=postgresql://resolve:resolve@db:5432/resolve
REDIS_URL=redis://redis:6379

# Security (CHANGE IN PRODUCTION!)
JWT_SECRET=your-super-secret-jwt-key-change-in-production
INTEGRATION_ENCRYPTION_KEY=64-char-hex-key-for-encrypted-storage

# Features
DEMO_MODE=true
RUST_LOG=info
SERVER_ADDR=127.0.0.1:8080
```

### Production Deployment
For production use:
1. Change all default passwords and secrets
2. Use proper SSL certificates
3. Configure backup strategies
4. Set up monitoring and alerting
5. Review security settings
6. Configure email SMTP settings

## API Endpoints

- **Health Check**: `GET /health`
- **Clients**: `GET|POST|PUT|DELETE /api/v1/clients`
- **Tickets**: `GET|POST|PUT|DELETE /api/v1/tickets`
- **Assets**: `GET|POST|PUT|DELETE /api/v1/assets`
- **Documentation**: `GET|POST|PUT|DELETE /api/v1/documentation`
- **Passwords**: `GET|POST|PUT|DELETE /api/v1/passwords`
- **Reports**: `GET /api/v1/reports`

### Database Schema
- **88+ Tables** with comprehensive indexing
- **Full-text search** capabilities
- **Audit logging** for security
- **Data encryption** for sensitive fields
- **Foreign key constraints** for data integrity

## Development

### Local Development Setup
```bash
# Backend development
cd backend
cargo install sqlx-cli
export DATABASE_URL=postgresql://resolve:resolve@localhost:5432/resolve
sqlx migrate run
cargo run

# Frontend development
cd frontend
cargo install trunk
trunk serve
```

## Troubleshooting

### Common Issues

**Port Conflicts**
```bash
# Check for port conflicts
netstat -tulpn | grep :8080
# Change ports in docker-compose.demo.yml if needed
```

**Database Connection Issues**
```bash
# Check database logs
docker compose -f docker-compose.demo.yml logs db

# Reset database
docker compose -f docker-compose.demo.yml down -v
docker compose -f docker-compose.demo.yml up -d
```

**Performance Issues**
```bash
# Check resource usage
docker stats

# Increase memory limits in docker-compose.demo.yml
```

### Getting Help
- **Documentation**: Check the `/docs` directory
- **Issues**: Report bugs on [GitHub Issues](https://github.com/CK-Technology/resolve/issues)
- **Discussions**: Use GitHub Discussions for questions

## License

GPL-3.0 License - see LICENSE file for details.
