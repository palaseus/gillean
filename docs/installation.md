# Installation Guide

This guide will help you install and set up the Gillean blockchain platform on your system.

## Prerequisites

### System Requirements

- **Operating System**: Linux, macOS, or Windows (WSL recommended for Windows)
- **Rust**: Version 1.70.0 or higher
- **Memory**: Minimum 4GB RAM (8GB recommended)
- **Storage**: At least 2GB free disk space
- **Network**: Internet connection for dependency downloads

### Required Software

1. **Rust Toolchain**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Reload shell configuration
   source ~/.cargo/env
   
   # Verify installation
   rustc --version
   cargo --version
   ```

2. **Build Tools** (Linux)
   ```bash
   # Ubuntu/Debian
   sudo apt update
   sudo apt install build-essential pkg-config libssl-dev
   
   # CentOS/RHEL/Fedora
   sudo yum groupinstall "Development Tools"
   sudo yum install openssl-devel
   ```

3. **Build Tools** (macOS)
   ```bash
   # Install Xcode Command Line Tools
   xcode-select --install
   ```

## Installation Methods

### Method 1: From Source (Recommended)

1. **Clone the Repository**
   ```bash
   git clone https://github.com/your-org/gillean.git
   cd gillean
   ```

2. **Build the Project**
   ```bash
   # Build in release mode for optimal performance
   cargo build --release
   
   # Or build in debug mode for development
   cargo build
   ```

3. **Run Tests** (Optional but Recommended)
   ```bash
   # Run all tests
   cargo test
   
   # Run comprehensive test suite
   ./run_comprehensive_tests.sh
   ```

4. **Install Binary** (Optional)
   ```bash
   # Install globally
   cargo install --path .
   
   # Verify installation
   gillean --version
   ```

### Method 2: Using Cargo Install

```bash
# Install directly from GitHub
cargo install --git https://github.com/your-org/gillean.git

# Verify installation
gillean --version
```

## Configuration

### Environment Variables

Create a `.env` file in the project root:

```bash
# Blockchain Configuration
BLOCKCHAIN_NETWORK=testnet
BLOCKCHAIN_PORT=3000
BLOCKCHAIN_RPC_PORT=3001
BLOCKCHAIN_WS_PORT=3002

# Database Configuration
DATABASE_PATH=./data/blockchain.db
DATABASE_BACKUP_PATH=./data/backups

# Security Configuration
ENCRYPTION_KEY=your-secure-encryption-key
JWT_SECRET=your-jwt-secret-key

# Network Configuration
P2P_PORT=4000
P2P_BOOTSTRAP_NODES=node1.example.com:4000,node2.example.com:4000

# Performance Configuration
MAX_CONNECTIONS=100
THREAD_POOL_SIZE=8
CACHE_SIZE_MB=512

# Logging Configuration
LOG_LEVEL=info
LOG_FILE=./logs/gillean.log
```

### Configuration File

Create `config.toml` in the project root:

```toml
[blockchain]
network = "testnet"
port = 3000
rpc_port = 3001
ws_port = 3002
difficulty = 4
mining_reward = 50.0
block_time = 12

[database]
path = "./data/blockchain.db"
backup_path = "./data/backups"
max_size_gb = 10

[security]
encryption_key = "your-secure-encryption-key"
jwt_secret = "your-jwt-secret-key"
enable_audit_logging = true

[network]
p2p_port = 4000
bootstrap_nodes = [
    "node1.example.com:4000",
    "node2.example.com:4000"
]
max_connections = 100

[performance]
thread_pool_size = 8
cache_size_mb = 512
enable_parallel_processing = true

[logging]
level = "info"
file = "./logs/gillean.log"
max_file_size_mb = 100
max_files = 5
```

## Quick Start

### 1. Start the Blockchain Node

```bash
# Start in development mode
cargo run

# Start in release mode
cargo run --release

# Start with custom configuration
cargo run -- --config config.toml
```

### 2. Verify Installation

```bash
# Check blockchain status
curl http://localhost:3000/api/v1/status

# Check node health
curl http://localhost:3000/api/v1/health

# View blockchain statistics
curl http://localhost:3000/api/v1/stats
```

### 3. Create Your First Wallet

```bash
# Using CLI
gillean wallet create --name "My Wallet" --password "secure-password"

# Using API
curl -X POST http://localhost:3000/api/v1/wallets \
  -H "Content-Type: application/json" \
  -d '{"name": "My Wallet", "password": "secure-password"}'
```

## Development Setup

### 1. Install Development Dependencies

```bash
# Install additional tools for development
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-tarpaulin
cargo install cargo-doc

# Install frontend dependencies (if working on frontend)
cd frontend
npm install
```

### 2. Setup IDE

**VS Code Extensions:**
- rust-analyzer
- CodeLLDB
- Even Better TOML
- GitLens

**IntelliJ IDEA:**
- Rust Plugin

### 3. Development Workflow

```bash
# Watch for changes and rebuild
cargo watch -x check -x test -x run

# Run tests with coverage
cargo tarpaulin

# Generate documentation
cargo doc --open

# Check for security vulnerabilities
cargo audit
```

## Troubleshooting

### Common Issues

1. **Build Errors**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build
   
   # Update Rust toolchain
   rustup update
   ```

2. **Port Already in Use**
   ```bash
   # Find process using port
   lsof -i :3000
   
   # Kill process
   kill -9 <PID>
   ```

3. **Permission Denied**
   ```bash
   # Fix permissions
   chmod +x run_comprehensive_tests.sh
   sudo chown -R $USER:$USER .
   ```

4. **Database Locked**
   ```bash
   # Remove lock file
   rm -f ./data/blockchain.db-lock
   ```

### Getting Help

- **Documentation**: Check the [API Reference](api.md) and [Architecture Overview](architecture.md)
- **Issues**: Report bugs on [GitHub Issues](https://github.com/your-org/gillean/issues)
- **Discussions**: Join our [Discord](https://discord.gg/gillean) or [Forum](https://forum.gillean.org)
- **Email**: Contact support at support@gillean.org

## Next Steps

After installation, explore these resources:

1. [Quick Start Tutorial](quickstart.md) - Get up and running quickly
2. [API Reference](api.md) - Complete API documentation
3. [Architecture Overview](architecture.md) - Understand the system design
4. [Development Guide](development.md) - Learn how to contribute
5. [Deployment Guide](deployment.md) - Deploy to production

## Support

For additional support:

- **Documentation**: [docs.gillean.org](https://docs.gillean.org)
- **Community**: [community.gillean.org](https://community.gillean.org)
- **Discord**: [discord.gg/gillean](https://discord.gg/gillean)
- **Email**: support@gillean.org
