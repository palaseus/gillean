#!/bin/bash

# Fast Development Test Runner for Gillean Blockchain
# This script uses cargo watch for continuous testing during development

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
API_ADDRESS="127.0.0.1:3000"
DB_PATH="./data/dev_db"
NODE_PID_FILE="./dev_node.pid"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${PURPLE}================================${NC}"
    echo -e "${PURPLE}$1${NC}"
    echo -e "${PURPLE}================================${NC}"
}

# Function to cleanup on exit
cleanup() {
    print_status "Cleaning up..."
    
    # Stop the dev node if running
    if [[ -f "$NODE_PID_FILE" ]]; then
        local pid=$(cat "$NODE_PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            print_status "Stopping dev node (PID: $pid)..."
            kill "$pid" 2>/dev/null || true
            sleep 2
            kill -9 "$pid" 2>/dev/null || true
        fi
        rm -f "$NODE_PID_FILE"
    fi
    
    print_success "Cleanup completed"
}

# Set up trap to cleanup on exit
trap cleanup EXIT INT TERM

# Function to check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed. Please install Rust first."
        exit 1
    fi
    print_success "Rust $(cargo --version) is installed"
    
    # Check cargo-watch
    if ! command -v cargo-watch &> /dev/null; then
        print_warning "cargo-watch is not installed. Installing..."
        cargo install cargo-watch
    fi
    print_success "cargo-watch is available"
    
    # Check curl for API testing
    if ! command -v curl &> /dev/null; then
        print_error "curl is not installed. Please install curl for API testing."
        exit 1
    fi
    print_success "curl is available for API testing"
}

# Function to start dev node
start_dev_node() {
    print_header "Starting Development Node"
    
    # Clean up any existing dev database
    if [[ -d "$DB_PATH" ]]; then
        rm -rf "$DB_PATH"
    fi
    
    # Create dev database directory
    mkdir -p "$DB_PATH"
    
    print_status "Starting API server on $API_ADDRESS..."
    
    # Start the API server in the background
    cargo run -- start-api --address "$API_ADDRESS" --db-path "$DB_PATH" > dev_node.log 2>&1 &
    local node_pid=$!
    
    # Save PID for cleanup
    echo "$node_pid" > "$NODE_PID_FILE"
    
    # Wait for server to start
    print_status "Waiting for server to start..."
    local attempts=0
    local max_attempts=30
    
    while [[ $attempts -lt $max_attempts ]]; do
        if curl -s "http://$API_ADDRESS/health" > /dev/null 2>&1; then
            print_success "Dev node started successfully (PID: $node_pid)"
            return 0
        fi
        
        sleep 1
        attempts=$((attempts + 1))
    done
    
    print_error "Failed to start dev node"
    exit 1
}

# Function to run fast tests
run_fast_tests() {
    print_header "Running Fast Tests"
    
    print_status "Running unit tests only (fast mode)..."
    if cargo test --lib --tests --bins --examples --no-fail-fast; then
        print_success "Fast tests passed"
    else
        print_error "Fast tests failed"
        exit 1
    fi
}

# Function to run watch mode
run_watch_mode() {
    print_header "Starting Watch Mode"
    
    print_status "Starting cargo watch for continuous testing..."
    print_status "Press Ctrl+C to stop"
    
    # Start cargo watch to run tests on file changes
    cargo watch -x check -x test --lib --tests --bins --examples --no-fail-fast
}

# Function to run single test file
run_single_test() {
    local test_file="$1"
    
    if [[ -z "$test_file" ]]; then
        print_error "Please specify a test file to run"
        print_status "Usage: $0 single <test_file>"
        print_status "Example: $0 single integration_tests"
        exit 1
    fi
    
    print_header "Running Single Test: $test_file"
    
    if cargo test --test "$test_file" --no-fail-fast; then
        print_success "$test_file passed"
    else
        print_error "$test_file failed"
        exit 1
    fi
}

# Function to run API tests only
run_api_tests() {
    print_header "Running API Tests Only"
    
    # Start dev node if not running
    if [[ ! -f "$NODE_PID_FILE" ]] || ! kill -0 "$(cat "$NODE_PID_FILE")" 2>/dev/null; then
        start_dev_node
    fi
    
    print_status "Testing API endpoints..."
    
    # Test basic endpoints
    local endpoints=(
        "/health"
        "/chain"
        "/stats"
        "/network"
        "/storage"
        "/metrics"
    )
    
    for endpoint in "${endpoints[@]}"; do
        print_status "Testing $endpoint..."
        if curl -s "http://$API_ADDRESS$endpoint" > /dev/null 2>&1; then
            print_success "$endpoint: OK"
        else
            print_error "$endpoint: FAILED"
        fi
    done
    
    print_success "API tests completed"
}

# Function to show help
show_help() {
    echo "Gillean Blockchain Development Test Runner"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  fast        Run fast tests (unit tests only)"
    echo "  watch       Start watch mode for continuous testing"
    echo "  single      Run a single test file"
    echo "  api         Run API tests only"
    echo "  node        Start development node only"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 fast                           # Run fast tests"
    echo "  $0 watch                          # Start watch mode"
    echo "  $0 single integration_tests       # Run specific test file"
    echo "  $0 api                            # Test API endpoints"
    echo "  $0 node                           # Start dev node"
}

# Main execution function
main() {
    local command="${1:-help}"
    
    case "$command" in
        "fast")
            check_prerequisites
            run_fast_tests
            ;;
        "watch")
            check_prerequisites
            run_watch_mode
            ;;
        "single")
            check_prerequisites
            run_single_test "$2"
            ;;
        "api")
            check_prerequisites
            run_api_tests
            ;;
        "node")
            check_prerequisites
            start_dev_node
            print_status "Dev node is running. Press Ctrl+C to stop."
            wait
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# Run main function
main "$@"
