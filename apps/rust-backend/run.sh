#!/bin/bash
set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Ant Colony Simulator - Rust Backend${NC}"
echo "========================================"

if ! command -v cargo &> /dev/null; then
    echo "Rust/Cargo not found. Install: https://rustup.rs/"
    exit 1
fi

COMMAND="${1:-run}"

case "$COMMAND" in
    build)
        echo -e "${BLUE}Building (release)...${NC}"
        cargo build --release
        echo -e "${GREEN}Done.${NC}"
        ;;
    run)
        echo -e "${BLUE}Building and starting simulation...${NC}"
        echo "WebSocket server will listen on ws://127.0.0.1:8080/ws"
        cargo run --release
        ;;
    dev)
        echo -e "${BLUE}Starting in debug mode...${NC}"
        cargo run
        ;;
    *)
        echo "Usage: $0 [build|run|dev]"
        exit 1
        ;;
esac
