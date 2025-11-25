#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Resolve Production Setup${NC}"

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo -e "${RED}❌ This script should not be run as root${NC}"
   exit 1
fi

# Create necessary directories
echo -e "${YELLOW}📁 Creating directory structure...${NC}"
mkdir -p {data/{postgres,redis,app},logs,backups,secrets,docker/ssl}

# Generate secrets if they don't exist
echo -e "${YELLOW}🔐 Generating secrets...${NC}"

if [[ ! -f secrets/db_password.txt ]]; then
    openssl rand -base64 32 > secrets/db_password.txt
    echo -e "${GREEN}✅ Generated database password${NC}"
fi

if [[ ! -f secrets/jwt_secret.txt ]]; then
    openssl rand -base64 64 > secrets/jwt_secret.txt
    echo -e "${GREEN}✅ Generated JWT secret${NC}"
fi

if [[ ! -f secrets/encryption_key.txt ]]; then
    openssl rand -base64 32 > secrets/encryption_key.txt
    echo -e "${GREEN}✅ Generated encryption key${NC}"
fi

# Set proper permissions
chmod 600 secrets/*
chmod 700 secrets
echo -e "${GREEN}✅ Set secure permissions on secrets${NC}"

# Create PostgreSQL configuration
echo -e "${YELLOW}⚙️  Creating PostgreSQL configuration...${NC}"
cat > docker/postgresql.conf << 'EOF'
# PostgreSQL Production Configuration for Resolve

# Connection Settings
max_connections = 200
shared_buffers = 256MB
effective_cache_size = 1GB
work_mem = 4MB
maintenance_work_mem = 64MB

# WAL Settings
wal_buffers = 16MB
checkpoint_completion_target = 0.9
checkpoint_timeout = 10min
max_wal_size = 1GB
min_wal_size = 80MB

# Query Planner
random_page_cost = 1.1
effective_io_concurrency = 200
default_statistics_target = 100

# Logging
log_destination = 'stderr'
logging_collector = on
log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
log_rotation_age = 1d
log_rotation_size = 100MB
log_min_duration_statement = 1000
log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '
log_checkpoints = on
log_connections = on
log_disconnections = on
log_lock_waits = on

# Security
ssl = off
password_encryption = scram-sha-256

# Locale
lc_messages = 'en_US.UTF-8'
lc_monetary = 'en_US.UTF-8'
lc_numeric = 'en_US.UTF-8'
lc_time = 'en_US.UTF-8'
default_text_search_config = 'pg_catalog.english'
EOF

# Create Redis configuration
echo -e "${YELLOW}⚙️  Creating Redis configuration...${NC}"
cat > docker/redis.conf << 'EOF'
# Redis Production Configuration for Resolve

# Network
bind 0.0.0.0
port 6379
timeout 300
tcp-keepalive 300

# General
daemonize no
supervised no
loglevel notice
databases 16

# Persistence
save 900 1
save 300 10
save 60 10000
stop-writes-on-bgsave-error yes
rdbcompression yes
rdbchecksum yes
dbfilename dump.rdb
dir /data

# Security
requirepass ""
rename-command FLUSHDB ""
rename-command FLUSHALL ""
rename-command DEBUG ""
rename-command CONFIG "CONFIG_b8a9c0d6e1f7a4b2c3d5e8f9"

# Memory Management
maxmemory 256mb
maxmemory-policy allkeys-lru
maxmemory-samples 5

# Append Only File
appendonly yes
appendfilename "appendonly.aof"
appendfsync everysec
no-appendfsync-on-rewrite no
auto-aof-rewrite-percentage 100
auto-aof-rewrite-min-size 64mb
aof-load-truncated yes

# Slow Log
slowlog-log-slower-than 10000
slowlog-max-len 128
EOF

# Create database initialization script
cat > docker/postgres-init.sql << 'EOF'
-- Resolve Database Initialization
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "unaccent";

-- Create read-only user for reporting
CREATE USER resolve_readonly WITH PASSWORD 'readonly_password_change_me';
GRANT CONNECT ON DATABASE resolve TO resolve_readonly;
GRANT USAGE ON SCHEMA public TO resolve_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO resolve_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO resolve_readonly;
EOF

# Create backup script
echo -e "${YELLOW}💾 Creating backup script...${NC}"
cat > scripts/backup.sh << 'EOF'
#!/bin/bash

set -e

TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_DIR="/backup"
DB_NAME="resolve"
DB_USER="resolve"
DB_HOST="db"

# Create backup
echo "Starting backup at $(date)"
pg_dump -h $DB_HOST -U $DB_USER -d $DB_NAME -F c -b -v -f "$BACKUP_DIR/resolve_backup_$TIMESTAMP.sql"

# Compress backup
gzip "$BACKUP_DIR/resolve_backup_$TIMESTAMP.sql"

# Clean up old backups (keep last 7 days)
find $BACKUP_DIR -name "resolve_backup_*.sql.gz" -mtime +7 -delete

echo "Backup completed at $(date): resolve_backup_$TIMESTAMP.sql.gz"
EOF

chmod +x scripts/backup.sh

# Create systemd service file
echo -e "${YELLOW}🔧 Creating systemd service...${NC}"
cat > resolve.service << 'EOF'
[Unit]
Description=Resolve MSP Platform
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/resolve
ExecStart=/usr/bin/docker-compose -f docker-compose.prod.yml up -d
ExecStop=/usr/bin/docker-compose -f docker-compose.prod.yml down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF

echo -e "${BLUE}📋 Next steps:${NC}"
echo "1. Review and update secrets in the secrets/ directory"
echo "2. Configure SSL certificates in docker/ssl/"
echo "3. Update environment variables in docker-compose.prod.yml"
echo "4. Run: docker-compose -f docker-compose.prod.yml up -d"
echo "5. Optional: sudo cp resolve.service /etc/systemd/system/ && sudo systemctl enable resolve"
echo ""
echo -e "${GREEN}🎉 Production setup complete!${NC}"
echo -e "${YELLOW}⚠️  Remember to:${NC}"
echo "   - Change default passwords"
echo "   - Configure your domain and SSL"
echo "   - Set up monitoring and logging"
echo "   - Configure regular backups"