#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🧪 Starting Resolve Test Suite${NC}"

# Function to cleanup on exit
cleanup() {
    echo -e "${YELLOW}🧹 Cleaning up test environment...${NC}"
    docker-compose -f test-docker-compose.yml down -v >/dev/null 2>&1 || true
}

# Setup cleanup trap
trap cleanup EXIT

# Start test services
echo -e "${YELLOW}🚀 Starting test services...${NC}"
docker-compose -f test-docker-compose.yml up -d

# Wait for services to be healthy
echo -e "${YELLOW}⏳ Waiting for test services to be ready...${NC}"
timeout=60
while [ $timeout -gt 0 ]; do
    if docker-compose -f test-docker-compose.yml ps | grep -q "healthy"; then
        echo -e "${GREEN}✅ Test services are ready${NC}"
        break
    fi
    sleep 2
    ((timeout -= 2))
done

if [ $timeout -le 0 ]; then
    echo -e "${RED}❌ Test services failed to start${NC}"
    exit 1
fi

# Set test environment variables
export TEST_DATABASE_URL="postgresql://test:test@localhost:5433/resolve_test"
export REDIS_URL="redis://localhost:6380"
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Run database migrations for tests
echo -e "${YELLOW}📦 Running test database migrations...${NC}"
cd backend
sqlx database create --database-url=$TEST_DATABASE_URL || true
sqlx migrate run --database-url=$TEST_DATABASE_URL

# Run different types of tests based on arguments
case "${1:-all}" in
    "unit")
        echo -e "${GREEN}🔬 Running unit tests...${NC}"
        cargo test --lib tests::unit
        ;;
    "integration")
        echo -e "${GREEN}🔗 Running integration tests...${NC}"
        cargo test --test '*' tests::integration
        ;;
    "performance")
        echo -e "${GREEN}⚡ Running performance tests...${NC}"
        cargo test --release --lib tests::performance
        ;;
    "coverage")
        echo -e "${GREEN}📊 Running tests with coverage...${NC}"
        cargo tarpaulin --out Html --output-dir ../coverage --exclude-files "src/tests/*" --timeout 300
        echo -e "${GREEN}📈 Coverage report generated in coverage/tarpaulin-report.html${NC}"
        ;;
    "all"|*)
        echo -e "${GREEN}🚀 Running all tests...${NC}"
        
        echo -e "${YELLOW}📋 Unit Tests${NC}"
        cargo test --lib tests::unit
        
        echo -e "${YELLOW}🔗 Integration Tests${NC}"
        cargo test --test '*' tests::integration
        
        echo -e "${YELLOW}📊 Test Summary${NC}"
        cargo test --lib --quiet 2>&1 | grep -E "test result:|passed|failed" || true
        ;;
esac

echo -e "${GREEN}✅ Test suite completed successfully${NC}"