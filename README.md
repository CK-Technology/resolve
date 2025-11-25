<div align="center">

# Resolve

**Self-Hosted MSP Management Platform - Built with Rust + WebAssembly**

*Hudu meets Kaseya BMS meets ITFlow - Combined for MSPs as a SaaS Application*

</div>

![rust](https://img.shields.io/badge/Backend-Rust-orange?logo=rust)
![wasm](https://img.shields.io/badge/Frontend-WebAssembly-654ff0?logo=webassembly)
![yew](https://img.shields.io/badge/UI-Yew-green)
![postgres](https://img.shields.io/badge/Database-PostgreSQL-336791?logo=postgresql)
![docker](https://img.shields.io/badge/Deploy-Docker-2496ED?logo=docker)

---

## Overview

**Resolve** is a modern, self-hostable MSP (Managed Service Provider) management platform built with Rust and WebAssembly. Designed from the ground up for performance, security, and ease of deployment.

### Key Features

**Core MSP Features**
- **Client Management** - Organize clients with contacts, locations, and detailed information
- **Asset Tracking** - Track devices, servers, and infrastructure with relationship mapping
- **Ticketing System** - Support ticket management with SLA tracking and time billing
- **Invoicing & Billing** - Generate invoices, track payments, manage recurring billing
- **Time Tracking** - Billable hours with project and ticket integration

**Documentation & Security**
- **Knowledge Base** - Rich documentation system with templates (Hudu-style)
- **Password Vault** - Secure credential management with encrypted storage
- **Network Documentation** - Topology mapping and asset relationships

**Integrations**
- **Microsoft 365** - Tenant management and monitoring
- **Azure** - Resource tracking and alerting
- **Bitwarden** - Password manager integration
- **FortiCloud** - Fortinet device management
- **OAuth2/OIDC/SAML** - Enterprise authentication

**Platform**
- **Self-Hosted** - Complete control over your data
- **Modern Architecture** - Rust backend, WebAssembly frontend
- **Docker Ready** - Easy deployment with Docker and docker-compose
- **Client Portal** - Customer-facing portal for tickets and documentation

---

## Disclaimer

> **EXPERIMENTAL SOFTWARE - FOR LAB/PERSONAL USE**
>
> This is experimental software under active development. It is intended for research, learning, and personal projects. The API is subject to change.

---

## Quick Start

### One-Line Install (Ubuntu/Debian)

```bash
curl -sSL https://raw.githubusercontent.com/CK-Technology/resolve/main/install.sh | sudo bash
```

### Using Docker (Recommended)

1. Clone the repository:
```bash
git clone https://github.com/CK-Technology/resolve.git
cd resolve/deploy/docker
```

2. Start with docker-compose:
```bash
docker compose up -d
```

3. Access Resolve at `http://localhost`

### Production Deployment

```bash
cd deploy/docker
docker compose -f docker-compose.prod.yml up -d
```

See [docs/deployment.md](docs/deployment.md) for detailed production deployment guides.

---

## Architecture

| Component | Technology |
|-----------|------------|
| Backend | Axum web framework with SQLx |
| Frontend | Yew framework compiled to WebAssembly |
| Database | PostgreSQL with full-text search |
| Cache | Redis for sessions and caching |
| Deployment | Docker with Nginx reverse proxy |

### Project Structure
```
resolve/
├── backend/          # Rust backend (Axum)
├── frontend/         # WebAssembly frontend (Yew)
├── shared/           # Shared types between backend/frontend
├── deploy/           # Deployment configurations
│   ├── docker/       # Dockerfile and docker-compose
│   ├── systemd/      # Systemd service files
│   └── kubernetes/   # K8s manifests (future)
├── docs/             # Documentation
├── scripts/          # Build and utility scripts
└── install.sh        # One-line installation script
```

---

## Documentation

- [Development Guide](docs/development.md) - Local setup and development
- [Deployment Guide](docs/deployment.md) - Production deployment
- [API Reference](docs/api/README.md) - REST API documentation
- [Demo Guide](docs/demo.md) - Running the demo environment

---

## Security

- JWT-based authentication
- Password hashing with bcrypt/Argon2
- Encrypted credential storage (AES-GCM)
- SQL injection protection with SQLx
- CORS and security headers configured
- Input validation and sanitization

---

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

---

## License

GPL-3.0 License - see LICENSE file for details.

---

## Acknowledgments

Inspired by ITFlow, Hudu, and Kaseya BMS. Resolve aims to provide a modern, performant, open-source alternative built with Rust and WebAssembly.
