#!/bin/bash

# Ant Colony Simulator - Rust Backend Runner
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DEFAULT_DATABASE_URL="postgresql://postgres:postgres@127.0.0.1:57322/postgres"
LOG_LEVEL="${LOG_LEVEL:-info}"

print_header() {
    echo -e "${BLUE}"
    echo "🚀 Ant Colony Simulator - Rust Backend"
    echo "========================================"
    echo -e "${NC}"
}

print_usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  build     Build the project (debug mode)"
    echo "  release   Build the project (release mode)"
    echo "  run       Build and run the simulator"
    echo "  test      Run tests"
    echo "  format    Format code"
    echo "  lint      Run clippy linter"
    echo "  clean     Clean build artifacts"
    echo "  docker    Build and run Docker container"
    echo ""
    echo "Options:"
    echo "  --simulation-id ID    Specific simulation ID to run"
    echo "  --database-url URL    Database connection URL"
    echo "  --log-level LEVEL     Log level (trace, debug, info, warn, error)"
    echo ""
    echo "Environment Variables:"
    echo "  DATABASE_URL          Database connection URL"
    echo "  LOG_LEVEL            Default log level"
    echo ""
    echo "Examples:"
    echo "  $0 run"
    echo "  $0 run --log-level debug"
    echo "  $0 run --simulation-id 12345678-1234-1234-1234-123456789012"
    echo "  $0 docker"
}

check_dependencies() {
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust/Cargo not found. Please install Rust: https://rustup.rs/${NC}"
        exit 1
    fi
    
    if ! command -v psql &> /dev/null; then
        echo -e "${YELLOW}⚠️  PostgreSQL client not found. Database connectivity cannot be verified.${NC}"
    fi
}

build_project() {
    local mode="$1"
    echo -e "${BLUE}🔨 Building project in ${mode} mode...${NC}"
    
    if [ "$mode" = "release" ]; then
        cargo build --release
    else
        cargo build
    fi
    
    echo -e "${GREEN}✅ Build complete${NC}"
}

run_tests() {
    echo -e "${BLUE}🧪 Running tests...${NC}"
    cargo test
    echo -e "${GREEN}✅ Tests passed${NC}"
}

format_code() {
    echo -e "${BLUE}🎨 Formatting code...${NC}"
    cargo fmt
    echo -e "${GREEN}✅ Code formatted${NC}"
}

lint_code() {
    echo -e "${BLUE}🔍 Running linter...${NC}"
    cargo clippy -- -D warnings
    echo -e "${GREEN}✅ Linting complete${NC}"
}

clean_project() {
    echo -e "${BLUE}🧹 Cleaning build artifacts...${NC}"
    cargo clean
    echo -e "${GREEN}✅ Clean complete${NC}"
}

run_simulator() {
    local database_url="$1"
    local log_level="$2"
    local simulation_id="$3"
    
    echo -e "${BLUE}🎮 Starting Ant Colony Simulator...${NC}"
    echo -e "${YELLOW}📊 Log level: ${log_level}${NC}"
    echo -e "${YELLOW}🔌 Database: ${database_url}${NC}"
    
    local cmd_args=(
        "--database-url" "$database_url"
        "--log-level" "$log_level"
    )
    
    if [ -n "$simulation_id" ]; then
        echo -e "${YELLOW}🎯 Simulation ID: ${simulation_id}${NC}"
        cmd_args+=("--simulation-id" "$simulation_id")
    else
        echo -e "${YELLOW}🔍 Auto-detecting active simulation...${NC}"
    fi
    
    echo ""
    cargo run --release -- "${cmd_args[@]}"
}

build_docker() {
    echo -e "${BLUE}🐳 Building Docker image...${NC}"
    docker build -t ant-colony-simulator:latest .
    echo -e "${GREEN}✅ Docker image built${NC}"
}

run_docker() {
    local database_url="$1"
    local log_level="$2"
    local simulation_id="$3"
    
    echo -e "${BLUE}🐳 Running Docker container...${NC}"
    
    local docker_args=(
        "run" "--rm" "-it"
        "-e" "DATABASE_URL=$database_url"
        "-e" "LOG_LEVEL=$log_level"
    )
    
    if [ -n "$simulation_id" ]; then
        docker_args+=("-e" "SIMULATION_ID=$simulation_id")
    fi
    
    docker_args+=("ant-colony-simulator:latest")
    
    if [ -n "$simulation_id" ]; then
        docker_args+=("--simulation-id" "$simulation_id")
    fi
    
    docker "${docker_args[@]}" --database-url "$database_url" --log-level "$log_level"
}

# Parse command line arguments
COMMAND=""
DATABASE_URL="$DEFAULT_DATABASE_URL"
SIMULATION_ID=""

while [[ $# -gt 0 ]]; do
    case $1 in
        build|release|run|test|format|lint|clean|docker)
            COMMAND="$1"
            shift
            ;;
        --simulation-id)
            SIMULATION_ID="$2"
            shift 2
            ;;
        --database-url)
            DATABASE_URL="$2"
            shift 2
            ;;
        --log-level)
            LOG_LEVEL="$2"
            shift 2
            ;;
        -h|--help)
            print_header
            print_usage
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            print_usage
            exit 1
            ;;
    esac
done

# Default command
if [ -z "$COMMAND" ]; then
    COMMAND="run"
fi

# Main execution
print_header

case "$COMMAND" in
    build)
        check_dependencies
        build_project "debug"
        ;;
    release)
        check_dependencies
        build_project "release"
        ;;
    run)
        check_dependencies
        build_project "release"
        run_simulator "$DATABASE_URL" "$LOG_LEVEL" "$SIMULATION_ID"
        ;;
    test)
        check_dependencies
        run_tests
        ;;
    format)
        check_dependencies
        format_code
        ;;
    lint)
        check_dependencies
        lint_code
        ;;
    clean)
        check_dependencies
        clean_project
        ;;
    docker)
        build_docker
        run_docker "$DATABASE_URL" "$LOG_LEVEL" "$SIMULATION_ID"
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $COMMAND${NC}"
        print_usage
        exit 1
        ;;
esac

echo -e "${GREEN}🎉 Done!${NC}" 