# Resolve MSP Platform - Deployment Guide

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Quick Start with Docker](#quick-start-with-docker)
3. [Production Deployment](#production-deployment)
4. [Environment Configuration](#environment-configuration)
5. [Database Setup](#database-setup)
6. [Reverse Proxy Configuration](#reverse-proxy-configuration)
7. [SSL/TLS Setup](#ssltls-setup)
8. [Scaling](#scaling)
9. [Monitoring](#monitoring)
10. [Backup and Recovery](#backup-and-recovery)
11. [Troubleshooting](#troubleshooting)

---

## System Requirements

### Minimum Requirements

| Component | Specification |
|-----------|---------------|
| CPU | 2 cores |
| RAM | 4 GB |
| Storage | 20 GB SSD |
| OS | Ubuntu 22.04+, Debian 12+, RHEL 8+ |

### Recommended Production

| Component | Specification |
|-----------|---------------|
| CPU | 4+ cores |
| RAM | 8+ GB |
| Storage | 100+ GB SSD |
| Database | Dedicated PostgreSQL 15+ |
| Cache | Redis 7+ |

### Software Requirements

- Docker 24+ and Docker Compose 2.20+
- PostgreSQL 15+
- Redis 7+ (optional, for caching)
- Nginx or Caddy (reverse proxy)

---

## Quick Start with Docker

### 1. Clone Repository

```bash
git clone https://github.com/your-org/resolve.git
cd resolve
```

### 2. Create Environment File

```bash
cp .env.example .env
```

Edit `.env` with your settings:

```env
# Database
DATABASE_URL=postgresql://resolve:your-password@postgres:5432/resolve

# Server
SERVER_ADDR=0.0.0.0:8080
RUST_LOG=info

# JWT
JWT_SECRET=your-secure-jwt-secret-at-least-32-chars

# Email (SMTP)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM=noreply@yourcompany.com

# Redis (optional)
REDIS_URL=redis://redis:6379
```

### 3. Start Services

```bash
docker compose up -d
```

### 4. Access Application

- **Frontend:** http://localhost:3000
- **API:** http://localhost:8080
- **API Docs:** http://localhost:8080/api/v1/docs/swagger

### Default Credentials

```
Email: admin@resolve.local
Password: admin123
```

**⚠️ Change these immediately after first login!**

---

## Production Deployment

### Docker Compose Production

Create `docker-compose.prod.yml`:

```yaml
version: '3.8'

services:
  backend:
    image: resolve/backend:latest
    restart: always
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - REDIS_URL=${REDIS_URL}
      - JWT_SECRET=${JWT_SECRET}
      - RUST_LOG=info
    ports:
      - "127.0.0.1:8080:8080"
    depends_on:
      - postgres
      - redis
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G

  frontend:
    image: resolve/frontend:latest
    restart: always
    ports:
      - "127.0.0.1:3000:80"
    depends_on:
      - backend

  postgres:
    image: postgres:15-alpine
    restart: always
    environment:
      - POSTGRES_USER=resolve
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
      - POSTGRES_DB=resolve
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U resolve"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    restart: always
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data

  nginx:
    image: nginx:alpine
    restart: always
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
    depends_on:
      - backend
      - frontend

volumes:
  postgres_data:
  redis_data:
```

### Build and Deploy

```bash
# Build images
docker compose -f docker-compose.prod.yml build

# Start services
docker compose -f docker-compose.prod.yml up -d

# View logs
docker compose -f docker-compose.prod.yml logs -f

# Scale backend
docker compose -f docker-compose.prod.yml up -d --scale backend=3
```

---

## Environment Configuration

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user:pass@host:5432/db` |
| `JWT_SECRET` | JWT signing secret (32+ chars) | `your-secure-random-string` |
| `SERVER_ADDR` | Server bind address | `0.0.0.0:8080` |

### Email Configuration

| Variable | Description | Example |
|----------|-------------|---------|
| `SMTP_HOST` | SMTP server hostname | `smtp.gmail.com` |
| `SMTP_PORT` | SMTP server port | `587` |
| `SMTP_USER` | SMTP username | `noreply@company.com` |
| `SMTP_PASSWORD` | SMTP password | `app-password` |
| `SMTP_FROM` | From email address | `Resolve <noreply@company.com>` |
| `SMTP_TLS` | Enable TLS | `true` |

### Optional Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `REDIS_URL` | Redis connection URL | `None` |
| `RUST_LOG` | Log level | `info` |
| `CORS_ORIGINS` | Allowed CORS origins | `*` |
| `MAX_UPLOAD_SIZE` | Max file upload (MB) | `50` |
| `SESSION_TTL` | Session timeout (hours) | `24` |

### Integration Variables

```env
# Microsoft 365
M365_CLIENT_ID=your-client-id
M365_CLIENT_SECRET=your-client-secret
M365_TENANT_ID=your-tenant-id

# Microsoft Teams
TEAMS_WEBHOOK_URL=https://outlook.office.com/webhook/...

# Azure
AZURE_CLIENT_ID=your-client-id
AZURE_CLIENT_SECRET=your-client-secret
AZURE_TENANT_ID=your-tenant-id

# Bitwarden
BITWARDEN_API_URL=https://your-bitwarden.com
BITWARDEN_API_KEY=your-api-key
```

---

## Database Setup

### PostgreSQL Installation (Ubuntu)

```bash
# Install PostgreSQL
sudo apt update
sudo apt install postgresql postgresql-contrib

# Create database and user
sudo -u postgres psql
```

```sql
CREATE USER resolve WITH PASSWORD 'your-secure-password';
CREATE DATABASE resolve OWNER resolve;
GRANT ALL PRIVILEGES ON DATABASE resolve TO resolve;
\q
```

### Database Migrations

Migrations run automatically on startup. To run manually:

```bash
# Using Docker
docker compose exec backend ./resolve-backend migrate

# Or directly
DATABASE_URL=postgresql://... ./resolve-backend migrate
```

### Performance Tuning

Edit `/etc/postgresql/15/main/postgresql.conf`:

```ini
# Memory
shared_buffers = 2GB
effective_cache_size = 6GB
work_mem = 256MB
maintenance_work_mem = 512MB

# Connections
max_connections = 200

# Write Ahead Log
wal_buffers = 64MB
checkpoint_completion_target = 0.9

# Query Planner
random_page_cost = 1.1
effective_io_concurrency = 200
```

---

## Reverse Proxy Configuration

### Nginx Configuration

Create `/etc/nginx/sites-available/resolve`:

```nginx
upstream backend {
    server 127.0.0.1:8080;
    keepalive 32;
}

upstream frontend {
    server 127.0.0.1:3000;
}

server {
    listen 80;
    server_name resolve.yourcompany.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name resolve.yourcompany.com;

    ssl_certificate /etc/letsencrypt/live/resolve.yourcompany.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/resolve.yourcompany.com/privkey.pem;
    ssl_session_timeout 1d;
    ssl_session_cache shared:SSL:50m;
    ssl_session_tickets off;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;

    # HSTS
    add_header Strict-Transport-Security "max-age=63072000" always;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # API
    location /api/ {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # WebSocket
    location /ws {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400;
    }

    # Frontend
    location / {
        proxy_pass http://frontend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Static files caching
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2)$ {
        proxy_pass http://frontend;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }

    # File uploads
    client_max_body_size 50M;
}
```

Enable the site:

```bash
sudo ln -s /etc/nginx/sites-available/resolve /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### Caddy Configuration

```caddyfile
resolve.yourcompany.com {
    # API
    handle /api/* {
        reverse_proxy localhost:8080
    }

    # WebSocket
    handle /ws {
        reverse_proxy localhost:8080
    }

    # Frontend
    handle {
        reverse_proxy localhost:3000
    }

    # Security headers
    header {
        X-Frame-Options "SAMEORIGIN"
        X-Content-Type-Options "nosniff"
        X-XSS-Protection "1; mode=block"
        Strict-Transport-Security "max-age=63072000"
    }
}
```

---

## SSL/TLS Setup

### Let's Encrypt with Certbot

```bash
# Install certbot
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d resolve.yourcompany.com

# Auto-renewal (added automatically)
sudo certbot renew --dry-run
```

### Manual Certificate

```bash
# Create certificate directory
sudo mkdir -p /etc/nginx/certs

# Copy your certificates
sudo cp your-cert.pem /etc/nginx/certs/fullchain.pem
sudo cp your-key.pem /etc/nginx/certs/privkey.pem

# Set permissions
sudo chmod 600 /etc/nginx/certs/*
```

---

## Scaling

### Horizontal Scaling

```yaml
# docker-compose.prod.yml
services:
  backend:
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '1'
          memory: 1G
```

### Load Balancing with Nginx

```nginx
upstream backend {
    least_conn;
    server backend1:8080 weight=5;
    server backend2:8080 weight=5;
    server backend3:8080 weight=5;
    keepalive 32;
}
```

### Database Connection Pooling

Use PgBouncer for connection pooling:

```ini
# pgbouncer.ini
[databases]
resolve = host=postgres port=5432 dbname=resolve

[pgbouncer]
listen_addr = 0.0.0.0
listen_port = 6432
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 20
```

---

## Monitoring

### Health Checks

```bash
# Backend health
curl http://localhost:8080/health

# Detailed health
curl http://localhost:8080/health/detailed

# Metrics (Prometheus format)
curl http://localhost:8080/metrics
```

### Prometheus Configuration

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'resolve-backend'
    static_configs:
      - targets: ['backend:8080']
    metrics_path: /metrics
```

### Grafana Dashboard

Import the provided dashboard from `docs/grafana-dashboard.json` for:
- Request rates and latency
- Error rates
- Database connections
- Memory/CPU usage
- SLA compliance metrics

### Log Aggregation

Configure log shipping to your preferred system:

```yaml
# docker-compose with Loki
services:
  loki:
    image: grafana/loki:2.9.0
    ports:
      - "3100:3100"

  promtail:
    image: grafana/promtail:2.9.0
    volumes:
      - /var/log:/var/log
      - ./promtail-config.yml:/etc/promtail/config.yml
```

---

## Backup and Recovery

### Database Backup

```bash
#!/bin/bash
# backup.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR=/backups
RETENTION_DAYS=30

# Create backup
pg_dump -h localhost -U resolve resolve | gzip > $BACKUP_DIR/resolve_$DATE.sql.gz

# Upload to S3 (optional)
aws s3 cp $BACKUP_DIR/resolve_$DATE.sql.gz s3://your-bucket/backups/

# Clean old backups
find $BACKUP_DIR -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete
```

Add to crontab:
```bash
0 2 * * * /opt/resolve/backup.sh >> /var/log/resolve-backup.log 2>&1
```

### Database Restore

```bash
# From local backup
gunzip -c resolve_20240228_020000.sql.gz | psql -h localhost -U resolve resolve

# From S3
aws s3 cp s3://your-bucket/backups/resolve_20240228_020000.sql.gz - | gunzip | psql -h localhost -U resolve resolve
```

### File Backup

```bash
# Backup uploaded files
tar -czvf files_backup_$DATE.tar.gz /data/resolve/uploads

# Sync to remote storage
rsync -avz /data/resolve/uploads/ backup-server:/backups/resolve/uploads/
```

---

## Troubleshooting

### Common Issues

#### Database Connection Failed

```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Check connection
psql -h localhost -U resolve -d resolve -c "SELECT 1"

# Check firewall
sudo ufw status
```

#### Backend Won't Start

```bash
# Check logs
docker compose logs backend

# Common issues:
# - DATABASE_URL not set or incorrect
# - JWT_SECRET not set
# - Port 8080 already in use
```

#### WebSocket Connection Issues

```nginx
# Ensure proxy headers are set
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection "upgrade";
proxy_read_timeout 86400;
```

#### High Memory Usage

```bash
# Check memory usage
docker stats

# Adjust limits in docker-compose.yml
deploy:
  resources:
    limits:
      memory: 2G
```

#### Slow Database Queries

```sql
-- Check slow queries
SELECT pid, now() - pg_stat_activity.query_start AS duration, query, state
FROM pg_stat_activity
WHERE (now() - pg_stat_activity.query_start) > interval '5 seconds';

-- Check missing indexes
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
ORDER BY idx_scan ASC;
```

### Getting Help

- **Documentation:** https://docs.resolve.io
- **GitHub Issues:** https://github.com/your-org/resolve/issues
- **Community Discord:** https://discord.gg/resolve
- **Email Support:** support@resolve.io

---

## Upgrade Guide

### Standard Upgrade

```bash
# Pull latest images
docker compose pull

# Stop and recreate containers
docker compose up -d

# Run migrations (automatic on startup)
docker compose exec backend ./resolve-backend migrate
```

### Major Version Upgrade

1. **Backup everything:**
   ```bash
   ./backup.sh
   ```

2. **Read release notes** for breaking changes

3. **Update environment variables** if needed

4. **Pull and deploy:**
   ```bash
   docker compose pull
   docker compose up -d
   ```

5. **Verify functionality:**
   ```bash
   curl http://localhost:8080/health/detailed
   ```

### Rollback

```bash
# Restore database
gunzip -c resolve_backup.sql.gz | psql -h localhost -U resolve resolve

# Use previous image version
docker compose pull resolve/backend:v1.2.3
docker compose up -d
```
