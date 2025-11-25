#!/bin/bash
#
# Resolve MSP Platform - Installation Script
# https://github.com/CK-Technology/resolve
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/CK-Technology/resolve/main/install.sh | bash
#
# Or download and run:
#   wget -O install.sh https://raw.githubusercontent.com/CK-Technology/resolve/main/install.sh
#   chmod +x install.sh
#   sudo ./install.sh
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RESOLVE_VERSION="${RESOLVE_VERSION:-latest}"
RESOLVE_USER="resolve"
RESOLVE_GROUP="resolve"
RESOLVE_HOME="/opt/resolve"
RESOLVE_DATA="/var/lib/resolve"
RESOLVE_LOG="/var/log/resolve"
RESOLVE_REPO="https://github.com/CK-Technology/resolve.git"

# Functions
print_banner() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║                                                           ║"
    echo "║   ██████╗ ███████╗███████╗ ██████╗ ██╗    ██╗   ██╗███████╗║"
    echo "║   ██╔══██╗██╔════╝██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝║"
    echo "║   ██████╔╝█████╗  ███████╗██║   ██║██║    ██║   ██║█████╗  ║"
    echo "║   ██╔══██╗██╔══╝  ╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝  ║"
    echo "║   ██║  ██║███████╗███████║╚██████╔╝███████╗╚████╔╝ ███████╗║"
    echo "║   ╚═╝  ╚═╝╚══════╝╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝║"
    echo "║                                                           ║"
    echo "║          MSP Management Platform Installer                ║"
    echo "║                                                           ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

check_os() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS=$ID
        OS_VERSION=$VERSION_ID
        log_info "Detected OS: $OS $OS_VERSION"
    else
        log_error "Cannot detect operating system"
        exit 1
    fi

    case $OS in
        ubuntu|debian)
            PKG_MANAGER="apt-get"
            ;;
        rocky|almalinux|rhel|centos)
            PKG_MANAGER="dnf"
            ;;
        *)
            log_error "Unsupported operating system: $OS"
            log_info "Supported: Ubuntu, Debian, Rocky Linux, AlmaLinux, RHEL"
            exit 1
            ;;
    esac
}

install_dependencies() {
    log_info "Installing dependencies..."

    case $PKG_MANAGER in
        apt-get)
            apt-get update -qq
            apt-get install -y -qq \
                curl \
                wget \
                git \
                ca-certificates \
                gnupg \
                lsb-release \
                openssl \
                sudo
            ;;
        dnf)
            dnf install -y -q \
                curl \
                wget \
                git \
                ca-certificates \
                openssl \
                sudo
            ;;
    esac
}

install_docker() {
    if command -v docker &> /dev/null; then
        log_info "Docker already installed: $(docker --version)"
        return
    fi

    log_info "Installing Docker..."

    case $PKG_MANAGER in
        apt-get)
            # Add Docker's official GPG key
            install -m 0755 -d /etc/apt/keyrings
            curl -fsSL https://download.docker.com/linux/$OS/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
            chmod a+r /etc/apt/keyrings/docker.gpg

            # Set up the repository
            echo \
                "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/$OS \
                $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

            apt-get update -qq
            apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-compose-plugin
            ;;
        dnf)
            dnf config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo
            dnf install -y -q docker-ce docker-ce-cli containerd.io docker-compose-plugin
            ;;
    esac

    systemctl enable docker
    systemctl start docker
    log_info "Docker installed successfully"
}

create_user() {
    if id "$RESOLVE_USER" &>/dev/null; then
        log_info "User $RESOLVE_USER already exists"
        return
    fi

    log_info "Creating system user: $RESOLVE_USER"
    groupadd -r $RESOLVE_GROUP 2>/dev/null || true
    useradd -r -g $RESOLVE_GROUP -d $RESOLVE_HOME -s /bin/false $RESOLVE_USER
    usermod -aG docker $RESOLVE_USER
}

create_directories() {
    log_info "Creating directories..."

    mkdir -p $RESOLVE_HOME
    mkdir -p $RESOLVE_DATA/{postgres,redis,app,backups}
    mkdir -p $RESOLVE_LOG
    mkdir -p $RESOLVE_HOME/secrets

    chown -R $RESOLVE_USER:$RESOLVE_GROUP $RESOLVE_HOME
    chown -R $RESOLVE_USER:$RESOLVE_GROUP $RESOLVE_DATA
    chown -R $RESOLVE_USER:$RESOLVE_GROUP $RESOLVE_LOG
    chmod 700 $RESOLVE_HOME/secrets
}

generate_secrets() {
    log_info "Generating secrets..."

    if [[ ! -f $RESOLVE_HOME/secrets/db_password.txt ]]; then
        openssl rand -base64 32 > $RESOLVE_HOME/secrets/db_password.txt
    fi

    if [[ ! -f $RESOLVE_HOME/secrets/jwt_secret.txt ]]; then
        openssl rand -base64 64 > $RESOLVE_HOME/secrets/jwt_secret.txt
    fi

    if [[ ! -f $RESOLVE_HOME/secrets/encryption_key.txt ]]; then
        openssl rand -hex 32 > $RESOLVE_HOME/secrets/encryption_key.txt
    fi

    chmod 600 $RESOLVE_HOME/secrets/*.txt
    chown $RESOLVE_USER:$RESOLVE_GROUP $RESOLVE_HOME/secrets/*.txt
}

download_resolve() {
    log_info "Downloading Resolve..."

    if [[ -d $RESOLVE_HOME/app ]]; then
        log_info "Updating existing installation..."
        cd $RESOLVE_HOME/app
        git pull origin main
    else
        git clone --depth 1 $RESOLVE_REPO $RESOLVE_HOME/app
    fi

    chown -R $RESOLVE_USER:$RESOLVE_GROUP $RESOLVE_HOME/app
}

configure_resolve() {
    log_info "Configuring Resolve..."

    cd $RESOLVE_HOME/app

    # Create .env file
    DB_PASSWORD=$(cat $RESOLVE_HOME/secrets/db_password.txt)
    JWT_SECRET=$(cat $RESOLVE_HOME/secrets/jwt_secret.txt)
    ENCRYPTION_KEY=$(cat $RESOLVE_HOME/secrets/encryption_key.txt)

    cat > $RESOLVE_HOME/.env << EOF
# Resolve Configuration
# Generated by install.sh on $(date)

# Database
POSTGRES_DB=resolve
POSTGRES_USER=resolve
POSTGRES_PASSWORD=$DB_PASSWORD
DATABASE_URL=postgresql://resolve:$DB_PASSWORD@db:5432/resolve

# Redis
REDIS_URL=redis://redis:6379

# Application
SERVER_ADDR=127.0.0.1:8080
JWT_SECRET=$JWT_SECRET
ENCRYPTION_KEY=$ENCRYPTION_KEY
RUST_LOG=info

# Paths
RESOLVE_DATA=$RESOLVE_DATA
RESOLVE_LOG=$RESOLVE_LOG
EOF

    chmod 600 $RESOLVE_HOME/.env
    chown $RESOLVE_USER:$RESOLVE_GROUP $RESOLVE_HOME/.env
}

setup_systemd() {
    log_info "Setting up systemd service..."

    cat > /etc/systemd/system/resolve.service << EOF
[Unit]
Description=Resolve MSP Platform
Documentation=https://github.com/CK-Technology/resolve
After=docker.service
Requires=docker.service

[Service]
Type=simple
User=$RESOLVE_USER
Group=$RESOLVE_GROUP
WorkingDirectory=$RESOLVE_HOME/app/deploy/docker
EnvironmentFile=$RESOLVE_HOME/.env
ExecStartPre=/usr/bin/docker compose pull
ExecStart=/usr/bin/docker compose -f docker-compose.prod.yml up
ExecStop=/usr/bin/docker compose -f docker-compose.prod.yml down
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable resolve.service
}

setup_nginx_proxy() {
    log_info "Do you want to set up Nginx as a reverse proxy with SSL? (y/n)"
    read -r SETUP_NGINX

    if [[ "$SETUP_NGINX" =~ ^[Yy]$ ]]; then
        log_info "Enter your domain name (e.g., resolve.example.com):"
        read -r DOMAIN

        log_info "Enter your email for Let's Encrypt certificates:"
        read -r EMAIL

        # Install Nginx and Certbot
        case $PKG_MANAGER in
            apt-get)
                apt-get install -y -qq nginx certbot python3-certbot-nginx
                ;;
            dnf)
                dnf install -y -q nginx certbot python3-certbot-nginx
                ;;
        esac

        # Configure Nginx
        cat > /etc/nginx/sites-available/resolve << EOF
server {
    listen 80;
    server_name $DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_cache_bypass \$http_upgrade;
    }
}
EOF

        ln -sf /etc/nginx/sites-available/resolve /etc/nginx/sites-enabled/
        rm -f /etc/nginx/sites-enabled/default

        systemctl enable nginx
        systemctl restart nginx

        # Get SSL certificate
        log_info "Obtaining SSL certificate..."
        certbot --nginx -d $DOMAIN --non-interactive --agree-tos -m $EMAIL

        log_info "SSL configured for $DOMAIN"
    fi
}

start_resolve() {
    log_info "Starting Resolve..."
    systemctl start resolve.service

    # Wait for services to be ready
    log_info "Waiting for services to start..."
    sleep 10

    # Check status
    if systemctl is-active --quiet resolve.service; then
        log_info "Resolve is running!"
    else
        log_error "Resolve failed to start. Check logs with: journalctl -u resolve.service"
        exit 1
    fi
}

print_success() {
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║           Resolve Installation Complete!                  ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "  ${BLUE}Access Resolve:${NC}"
    echo -e "    Local:  http://localhost"
    if [[ -n "$DOMAIN" ]]; then
        echo -e "    Domain: https://$DOMAIN"
    fi
    echo ""
    echo -e "  ${BLUE}Default Admin Credentials:${NC}"
    echo -e "    Email:    admin@resolve.local"
    echo -e "    Password: (check $RESOLVE_HOME/secrets/admin_password.txt)"
    echo ""
    echo -e "  ${BLUE}Useful Commands:${NC}"
    echo -e "    Start:    sudo systemctl start resolve"
    echo -e "    Stop:     sudo systemctl stop resolve"
    echo -e "    Status:   sudo systemctl status resolve"
    echo -e "    Logs:     sudo journalctl -u resolve -f"
    echo -e "    Update:   curl -sSL https://raw.githubusercontent.com/CK-Technology/resolve/main/update.sh | sudo bash"
    echo ""
    echo -e "  ${BLUE}Directories:${NC}"
    echo -e "    Install:  $RESOLVE_HOME"
    echo -e "    Data:     $RESOLVE_DATA"
    echo -e "    Logs:     $RESOLVE_LOG"
    echo -e "    Secrets:  $RESOLVE_HOME/secrets"
    echo ""
    echo -e "  ${YELLOW}Documentation: https://github.com/CK-Technology/resolve/docs${NC}"
    echo ""
}

# Main installation flow
main() {
    print_banner

    log_info "Starting Resolve installation..."
    echo ""

    check_root
    check_os
    install_dependencies
    install_docker
    create_user
    create_directories
    generate_secrets
    download_resolve
    configure_resolve
    setup_systemd
    setup_nginx_proxy
    start_resolve
    print_success
}

# Run main function
main "$@"
