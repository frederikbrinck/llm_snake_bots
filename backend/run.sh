#!/bin/bash

# Multiplayer Snake Game - Build and Run Script
# This script provides easy commands for development and deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DEFAULT_PORT=3000
DEFAULT_HOST="0.0.0.0"

# Helper functions
print_banner() {
    echo -e "${GREEN}"
    echo "🐍 Multiplayer Snake Game Backend"
    echo "=================================="
    echo -e "${NC}"
}

print_help() {
    echo -e "${BLUE}Usage: $0 [COMMAND]${NC}"
    echo ""
    echo "Commands:"
    echo "  dev          Start development server with auto-reload"
    echo "  run          Start production server"
    echo "  build        Build release binary"
    echo "  test         Run all tests"
    echo "  lint         Run clippy linter"
    echo "  format       Format code with rustfmt"
    echo "  clean        Clean build artifacts"
    echo "  deps         Install dependencies"
    echo "  docker       Build and run Docker container"
    echo "  check        Check code without building"
    echo "  help         Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  PORT         Server port (default: $DEFAULT_PORT)"
    echo "  HOST         Server host (default: $DEFAULT_HOST)"
    echo "  RUST_LOG     Logging level (debug, info, warn, error)"
    echo ""
    echo "Examples:"
    echo "  $0 dev                    # Start development server"
    echo "  PORT=8080 $0 run         # Start server on port 8080"
    echo "  RUST_LOG=debug $0 dev    # Start with debug logging"
}

check_rust() {
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}Error: Rust/Cargo not found. Please install Rust from https://rustup.rs/${NC}"
        exit 1
    fi
}

check_docker() {
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}Error: Docker not found. Please install Docker.${NC}"
        exit 1
    fi
}

install_cargo_watch() {
    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-watch for development...${NC}"
        cargo install cargo-watch
    fi
}

# Commands
cmd_dev() {
    print_banner
    echo -e "${BLUE}Starting development server with auto-reload...${NC}"
    
    check_rust
    install_cargo_watch
    
    export RUST_LOG=${RUST_LOG:-debug}
    export SERVER_PORT=${PORT:-$DEFAULT_PORT}
    export SERVER_HOST=${HOST:-$DEFAULT_HOST}
    
    echo -e "${GREEN}Server will be available at: http://$SERVER_HOST:$SERVER_PORT${NC}"
    echo -e "${GREEN}GUI Interface: http://$SERVER_HOST:$SERVER_PORT${NC}"
    echo -e "${GREEN}API Documentation: http://$SERVER_HOST:$SERVER_PORT/docs${NC}"
    echo -e "${GREEN}Swagger UI: http://$SERVER_HOST:$SERVER_PORT/swagger-ui${NC}"
    echo ""
    echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
    echo ""
    
    cargo watch -x run
}

cmd_run() {
    print_banner
    echo -e "${BLUE}Starting production server...${NC}"
    
    check_rust
    
    export RUST_LOG=${RUST_LOG:-info}
    export SERVER_PORT=${PORT:-$DEFAULT_PORT}
    export SERVER_HOST=${HOST:-$DEFAULT_HOST}
    
    echo -e "${GREEN}Building release binary...${NC}"
    cargo build --release
    
    echo -e "${GREEN}Server starting at: http://$SERVER_HOST:$SERVER_PORT${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
    echo ""
    
    cargo run --release
}

cmd_build() {
    print_banner
    echo -e "${BLUE}Building release binary...${NC}"
    
    check_rust
    cargo build --release
    
    echo -e "${GREEN}Build complete! Binary available at: target/release/backend${NC}"
}

cmd_test() {
    print_banner
    echo -e "${BLUE}Running tests...${NC}"
    
    check_rust
    cargo test --verbose
    
    echo -e "${GREEN}All tests passed!${NC}"
}

cmd_lint() {
    print_banner
    echo -e "${BLUE}Running clippy linter...${NC}"
    
    check_rust
    cargo clippy --all-targets --all-features -- -D warnings
    
    echo -e "${GREEN}Linting complete!${NC}"
}

cmd_format() {
    print_banner
    echo -e "${BLUE}Formatting code...${NC}"
    
    check_rust
    cargo fmt --all
    
    echo -e "${GREEN}Code formatted successfully!${NC}"
}

cmd_clean() {
    print_banner
    echo -e "${BLUE}Cleaning build artifacts...${NC}"
    
    check_rust
    cargo clean
    
    # Clean additional files
    rm -rf target/
    rm -f Cargo.lock
    
    echo -e "${GREEN}Clean complete!${NC}"
}

cmd_deps() {
    print_banner
    echo -e "${BLUE}Installing dependencies...${NC}"
    
    check_rust
    
    # Update Rust toolchain
    rustup update
    
    # Install useful development tools
    echo -e "${YELLOW}Installing development tools...${NC}"
    cargo install cargo-watch || true
    cargo install cargo-audit || true
    cargo install cargo-outdated || true
    
    # Update dependencies
    cargo update
    
    echo -e "${GREEN}Dependencies installed!${NC}"
}

cmd_docker() {
    print_banner
    echo -e "${BLUE}Building and running Docker container...${NC}"
    
    check_docker
    
    # Build Docker image
    echo -e "${YELLOW}Building Docker image...${NC}"
    docker build -t snake-game .
    
    # Run container
    echo -e "${YELLOW}Starting container...${NC}"
    PORT=${PORT:-$DEFAULT_PORT}
    docker run -p $PORT:3000 --rm -it snake-game
}

cmd_check() {
    print_banner
    echo -e "${BLUE}Checking code...${NC}"
    
    check_rust
    cargo check --all-targets --all-features
    
    echo -e "${GREEN}Check complete!${NC}"
}

cmd_audit() {
    print_banner
    echo -e "${BLUE}Auditing dependencies for security vulnerabilities...${NC}"
    
    check_rust
    
    if ! command -v cargo-audit &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-audit...${NC}"
        cargo install cargo-audit
    fi
    
    cargo audit
    
    echo -e "${GREEN}Security audit complete!${NC}"
}

cmd_outdated() {
    print_banner
    echo -e "${BLUE}Checking for outdated dependencies...${NC}"
    
    check_rust
    
    if ! command -v cargo-outdated &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-outdated...${NC}"
        cargo install cargo-outdated
    fi
    
    cargo outdated
}

cmd_benchmark() {
    print_banner
    echo -e "${BLUE}Running benchmarks...${NC}"
    
    check_rust
    cargo bench
    
    echo -e "${GREEN}Benchmarks complete!${NC}"
}

cmd_coverage() {
    print_banner
    echo -e "${BLUE}Generating test coverage report...${NC}"
    
    check_rust
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-tarpaulin...${NC}"
        cargo install cargo-tarpaulin
    fi
    
    cargo tarpaulin --out Html --output-dir coverage
    
    echo -e "${GREEN}Coverage report generated in coverage/tarpaulin-report.html${NC}"
}

# Main script logic
case "${1:-help}" in
    "dev"|"develop"|"development")
        cmd_dev
        ;;
    "run"|"start"|"serve")
        cmd_run
        ;;
    "build"|"compile")
        cmd_build
        ;;
    "test"|"tests")
        cmd_test
        ;;
    "lint"|"clippy")
        cmd_lint
        ;;
    "format"|"fmt")
        cmd_format
        ;;
    "clean")
        cmd_clean
        ;;
    "deps"|"dependencies")
        cmd_deps
        ;;
    "docker")
        cmd_docker
        ;;
    "check")
        cmd_check
        ;;
    "audit")
        cmd_audit
        ;;
    "outdated")
        cmd_outdated
        ;;
    "benchmark"|"bench")
        cmd_benchmark
        ;;
    "coverage"|"cov")
        cmd_coverage
        ;;
    "help"|"--help"|"-h")
        print_help
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        echo ""
        print_help
        exit 1
        ;;
esac