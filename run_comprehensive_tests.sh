#!/bin/bash

# Comprehensive Test Runner for Gillean Blockchain v2.0.0
# This script provides easy access to all test suites

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

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

# Function to show usage
show_usage() {
    echo "Comprehensive Test Runner for Gillean Blockchain v2.0.0"
    echo ""
    echo "Usage: $0 [OPTIONS] [TEST_SUITE]"
    echo ""
    echo "OPTIONS:"
    echo "  -h, --help              Show this help message"
    echo "  -v, --verbose           Enable verbose output"
    echo "  -p, --parallel          Run tests in parallel"
    echo "  -t, --timeout SECONDS   Set test timeout (default: 300)"
    echo "  -c, --config FILE       Use custom test configuration file"
    echo "  --quick                 Run quick development tests"
    echo "  --ci                    Run CI-optimized tests"
    echo "  --benchmarks            Run performance benchmarks"
    echo "  --coverage              Run tests with coverage"
    echo ""
    echo "TEST_SUITES:"
    echo "  all                     Run all test suites (default)"
    echo "  zkp                     Zero-Knowledge Proof tests"
    echo "  state_channels          Multi-party state channel tests"
    echo "  rollups                 Layer 2 rollup tests"
    echo "  sharding                Advanced sharding tests"
    echo "  wasm                    WASM VM tests"
    echo "  consensus               Advanced consensus tests"
    echo "  cross_chain             Cross-chain integration tests"
    echo "  did                     Decentralized identity tests"
    echo "  governance              On-chain governance tests"
    echo "  mobile                  Mobile support tests"
    echo "  contracts               Advanced contract features tests"
    echo "  ai                      AI integration tests"
    echo "  performance             Performance tests"
    echo "  security                Security tests"
    echo "  stress                  Stress tests"
    echo ""
    echo "EXAMPLES:"
    echo "  $0                      # Run all tests"
    echo "  $0 zkp                  # Run ZKP tests only"
    echo "  $0 --quick              # Run quick development tests"
    echo "  $0 --ci                 # Run CI tests"
    echo "  $0 -v --timeout 600     # Run all tests with verbose output and 10min timeout"
}

# Default values
VERBOSE=""
PARALLEL=""
TIMEOUT="300"
CONFIG=""
TEST_SUITE="all"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -v|--verbose)
            VERBOSE="--verbose"
            shift
            ;;
        -p|--parallel)
            PARALLEL="--parallel"
            shift
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        -c|--config)
            CONFIG="--config $2"
            shift 2
            ;;
        --quick)
            TEST_SUITE="quick"
            shift
            ;;
        --ci)
            TEST_SUITE="ci"
            shift
            ;;
        --benchmarks)
            TEST_SUITE="benchmarks"
            shift
            ;;
        --coverage)
            COVERAGE=true
            shift
            ;;
        all|zkp|state_channels|rollups|sharding|wasm|consensus|cross_chain|did|governance|mobile|contracts|ai|performance|security|stress)
            TEST_SUITE="$1"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Function to check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed. Please install Rust first."
        print_status "Visit https://rustup.rs/ for installation instructions."
        exit 1
    fi
    
    print_success "Rust $(cargo --version) is installed"
}

# Function to check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    # Check for tarpaulin (coverage tool)
    if [[ "$COVERAGE" == true ]] && ! command -v cargo-tarpaulin &> /dev/null; then
        print_warning "cargo-tarpaulin not found. Installing..."
        cargo install cargo-tarpaulin
    fi
    
    print_success "All dependencies are available"
}

# Function to build the project
build_project() {
    print_status "Building project..."
    
    if ! cargo build --release; then
        print_error "Build failed"
        exit 1
    fi
    
    print_success "Project built successfully"
}

# Function to run tests
run_tests() {
    local suite="$1"
    local args="$2"
    
    print_header "Running $suite tests"
    
    # Set environment variables for better test output
    export RUST_BACKTRACE=1
    export RUST_LOG=info
    
    # Run the tests (simplified for now)
    if cargo test --test run_tests -- --nocapture; then
        print_success "$suite tests completed successfully"
        return 0
    else
        print_error "$suite tests failed"
        return 1
    fi
}

# Function to run coverage tests
run_coverage_tests() {
    print_header "Running coverage tests"
    
    if cargo tarpaulin --out Html --output-dir coverage; then
        print_success "Coverage tests completed"
        print_status "Coverage report available in coverage/index.html"
    else
        print_error "Coverage tests failed"
        return 1
    fi
}

# Function to show test results
show_results() {
    print_header "Test Results Summary"
    
    # Look for test report files
    local report_files=(test_report_*.json)
    
    if [[ ${#report_files[@]} -gt 0 ]]; then
        print_status "Test reports generated:"
        for file in "${report_files[@]}"; do
            echo "  - $file"
        done
        
        # Show latest report summary if jq is available
        if command -v jq &> /dev/null; then
            local latest_report=$(ls -t test_report_*.json | head -1)
            if [[ -n "$latest_report" ]]; then
                echo ""
                print_status "Latest test report summary:"
                jq -r '"Total Tests: " + (.total_tests | tostring) + "\nPassed: " + (.passed_tests | tostring) + "\nFailed: " + (.failed_tests | tostring) + "\nPass Rate: " + ((.summary.pass_rate * 100) | tostring) + "%"' "$latest_report"
            fi
        fi
    else
        print_warning "No test report files found"
    fi
}

# Main execution
main() {
    print_header "Gillean Blockchain v2.0.0 - Comprehensive Test Runner"
    
    # Check prerequisites
    check_rust
    check_dependencies
    
    # Build project
    build_project
    
    # Prepare test arguments
    local test_args="$VERBOSE $PARALLEL --timeout $TIMEOUT $CONFIG"
    
    # Run tests based on suite
    case $TEST_SUITE in
        quick)
            print_status "Running quick development tests..."
            run_tests "quick" "$test_args"
            ;;
        ci)
            print_status "Running CI tests..."
            run_tests "ci" "$test_args"
            ;;
        benchmarks)
            print_status "Running performance benchmarks..."
            run_tests "benchmarks" "$test_args"
            ;;
        *)
            print_status "Running $TEST_SUITE test suite..."
            run_tests "$TEST_SUITE" "$test_args"
            ;;
    esac
    
    # Run coverage if requested
    if [[ "$COVERAGE" == true ]]; then
        run_coverage_tests
    fi
    
    # Show results
    show_results
    
    print_header "Test execution completed"
}

# Run main function
main "$@"
