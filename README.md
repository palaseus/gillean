# Gillean Blockchain v2.0.0

A privacy-focused, enterprise-grade blockchain platform in Rust featuring zero-knowledge proofs, layer 2 scaling with state channels, smart contracts, Proof-of-Stake consensus, sharding for horizontal scalability, cross-chain interoperability, and a WebAssembly-based virtual machine.

## Features

### Core Blockchain
- **Block Creation & Mining**: Proof-of-Work consensus with configurable difficulty
- **Transaction Handling**: Secure value transfers with digital signatures
- **Chain Validation**: Comprehensive blockchain integrity checks
- **Persistent Storage**: Sled-based database for data persistence
- **REST API**: Full HTTP API for blockchain interaction

### Zero-Knowledge Proofs (ZKPs) for Privacy
- **Private Transactions**: Hide sender, receiver, and amount using ZKPs
- **RISC0 Integration**: Use RISC0 for efficient zk-SNARK generation and verification
- **Proof Caching**: Optimize performance with intelligent proof caching
- **Commitment Schemes**: Cryptographic commitments for transaction privacy
- **Encrypted Memos**: Optional encrypted messages for private transactions
- **Public Verification**: Verify transaction validity without revealing details

### Layer 2 Scaling with State Channels
- **Off-Chain Processing**: Conduct transactions off-chain for instant settlement
- **Two-Party Channels**: Secure state channels between two participants
- **State Updates**: Incremental state updates with cryptographic signatures
- **Dispute Resolution**: On-chain arbitration for channel disputes
- **Automatic Settlement**: Final settlement on-chain when channels close
- **Channel Management**: Open, update, and close channels with full lifecycle management

### Developer SDK
- **Rust SDK**: Comprehensive SDK for external application integration
- **Wallet Management**: Create, import, and manage encrypted wallets
- **Transaction APIs**: Send regular and private transactions
- **Smart Contract Integration**: Deploy and interact with WASM contracts
- **State Channel APIs**: Full state channel lifecycle management
- **Analytics Access**: Real-time and historical blockchain analytics
- **WebSocket Support**: Real-time event subscriptions
- **Error Handling**: Comprehensive error handling with retry logic

### Sharding for Scalability
- **Horizontal Scaling**: Divide blockchain into multiple shards for parallel processing
- **Shard Management**: Automatic transaction assignment to appropriate shards
- **Cross-Shard Transactions**: Two-phase commit protocol for inter-shard operations
- **Shard Synchronization**: Coordinated state management across shards
- **Load Balancing**: Dynamic shard assignment based on transaction volume

### Cross-Chain Interoperability
- **Bridge Protocol**: Simplified bridge for cross-chain asset transfers
- **Asset Locking**: Lock assets on Gillean and unlock on external chains
- **Cryptographic Verification**: Ed25519 signatures for cross-chain transactions
- **Transaction Relay**: Relay transactions between different blockchains
- **Status Tracking**: Monitor cross-chain transaction status in real-time

### WebAssembly Smart Contracts
- **WASM VM**: High-performance WebAssembly virtual machine
- **Rust Contracts**: Write smart contracts in Rust and compile to WASM
- **Gas System**: Configurable gas limits and pricing
- **Contract Deployment**: Deploy WASM contracts with custom code
- **Contract Calls**: Execute deployed contracts with parameters
- **Contract Templates**: Pre-built templates for common use cases

### Contract Development Toolkit
- **Contract Compilation**: Compile Rust contracts to WASM bytecode
- **Contract Testing**: Test contracts locally before deployment
- **Contract Deployment**: Deploy compiled contracts to the blockchain
- **Development Workflow**: Streamlined development process
- **Template System**: Pre-built contract templates for common use cases

### Proof-of-Stake Consensus
- **Validator Registration**: Register as a validator with stake
- **Staking System**: Stake/unstake tokens for validator participation
- **Validator Selection**: Weighted random selection based on stake
- **Performance Tracking**: Monitor validator performance and rewards
- **Slashing Mechanism**: Penalize misbehaving validators
- **Consensus Switching**: Seamlessly switch between PoW and PoS

### Enhanced Frontend with Analytics
- **Modern Web UI**: Built with Yew (Rust + WebAssembly)
- **Real-time Updates**: WebSocket support for live blockchain updates
- **Advanced Analytics**: Visualize ZKP metrics, state channel activity, and shard performance
- **Interactive Charts**: Real-time charts for transaction volume, proof generation times
- **State Channel Monitoring**: Track open channels, transaction volume, and channel lifecycle
- **Shard Performance**: Real-time shard throughput and latency visualization
- **Cross-Chain Tracking**: Monitor cross-chain transfers with status tracking
- **Blockchain Explorer**: View blocks, transactions, and network status
- **Transaction Creation**: Create and send transactions (public and private)
- **Smart Contract Deployment**: Deploy contracts through web interface
- **Wallet Management**: Create and manage wallets
- **Network Metrics**: Real-time blockchain statistics

### Advanced Features
- **Wallet Management**: Encrypted wallet creation and management
- **Network Layer**: P2P networking for multi-node operation
- **Enhanced Monitoring**: Comprehensive metrics for ZKPs, state channels, and SDK usage
- **CLI Interface**: Full command-line interface for all operations
- **Enhanced Logging**: Detailed logging for debugging and monitoring

## Quick Start

### Prerequisites
- Rust (latest stable version)
- Cargo
- Trunk (for frontend development)
- wasm-pack (for WASM contract development)

### Installation

1. **Clone the repository**:
```bash
git clone https://github.com/yourusername/gillean.git
cd gillean
```

2. **Install frontend dependencies**:
```bash
cargo install trunk
cargo install wasm-bindgen-cli
cargo install wasm-pack
```

3. **Add WASM target**:
```bash
rustup target add wasm32-unknown-unknown
```

4. **Build the project**:
```bash
cargo build --release
```

### Running the Enhanced Demo

Start the interactive demo to see all v2.0.0 features in action:

```bash
# Enhanced Demo with all features
cargo run -- demo

# Privacy Features Demo
cargo run -- create-private-transaction --sender alice --receiver bob --amount 100.0 --password mypassword --memo "Private payment"

# State Channels Demo
cargo run -- open-channel --participant1 alice --participant2 bob --initial-balance 200.0 --timeout 3600

# SDK Generation
cargo run -- sdk-generate ./my_sdk_client

# SDK Testing
cargo run -- sdk-test
```

This will demonstrate:
- **Zero-Knowledge Proofs**: Private transaction creation and verification
- **State Channels**: Channel lifecycle (open, update, close)
- **SDK Integration**: Client code generation and testing
- **Sharding**: Transaction processing across multiple shards
- **Cross-Chain Transfers**: Asset transfers between different blockchains
- **WASM Contracts**: Compilation, testing, and deployment of smart contracts
- **Real-time UI**: Live updates via WebSocket connections
- **Enhanced Monitoring**: Comprehensive metrics for all new features

### Running with Different Modes

**Proof-of-Work (default)**:
```bash
cargo run -- --consensus pow --difficulty 4 --reward 50.0
```

**Proof-of-Stake**:
```bash
cargo run -- --consensus pos --min-stake 100.0 --max-validators 10
```

**Sharded Mode**:
```bash
cargo run -- start-sharded --consensus pow --num-shards 4
```

### Starting the Enhanced API Server

```bash
cargo run -- start-api --address 127.0.0.1:3000
```

### Starting the Enhanced Frontend

```bash
cd frontend
trunk serve
```

Then open `http://localhost:8080` in your browser to see the enhanced UI with real-time updates and advanced analytics.

## Usage Examples

### Zero-Knowledge Proofs

**Create Private Transaction**:
```bash
cargo run -- create-private-transaction \
  --sender alice \
  --receiver bob \
  --amount 100.0 \
  --password mypassword \
  --memo "Private payment"
```

**Verify ZKP Proof**:
```bash
cargo run -- verify-zkp --proof-data <hex_proof_data>
```

### State Channels

**Open State Channel**:
```bash
cargo run -- open-channel \
  --participant1 alice \
  --participant2 bob \
  --initial-balance 200.0 \
  --timeout 3600
```

**Update Channel State**:
```bash
cargo run -- update-channel \
  --channel-id <channel_id> \
  --balance1 80.0 \
  --balance2 120.0 \
  --password mypassword
```

**Close State Channel**:
```bash
cargo run -- close-channel \
  --channel-id <channel_id> \
  --balance1 70.0 \
  --balance2 130.0 \
  --password mypassword
```

**View Channel Statistics**:
```bash
cargo run -- channel-stats
```

### SDK Integration

**Generate SDK Client**:
```bash
cargo run -- sdk-generate ./my_sdk_client
```

**Run SDK Tests**:
```bash
cargo run -- sdk-test
```

**Using the SDK in Your Application**:
```rust
use gillean_sdk::{GilleanSDK, SDKConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SDKConfig::default();
    let sdk = GilleanSDK::new(config).await?;

    // Create wallet
    let wallet = sdk.create_wallet("password", Some("My Wallet")).await?;
    
    // Send private transaction
    let result = sdk.create_private_transaction(
        &wallet.address,
        "bob",
        50.0,
        "password",
        Some("Private payment")
    ).await?;
    
    println!("Private transaction: {}", result.transaction_hash);
    Ok(())
}
```

### Sharding Operations

**Start Sharded Blockchain**:
```bash
# Start with 4 shards using PoW consensus
cargo run -- start-sharded --consensus pow --num-shards 4

# Start with 8 shards using PoS consensus
cargo run -- start-sharded --consensus pos --num-shards 8
```

**View Shard Statistics**:
```bash
cargo run -- shard-stats
```

### Cross-Chain Operations

**Initiate Cross-Chain Transfer**:
```bash
cargo run -- cross-chain-transfer \
  --source-chain gillean \
  --target-chain ethereum \
  --sender alice \
  --receiver bob \
  --amount 100.0 \
  --asset-type GIL
```

**Check Bridge Status**:
```bash
cargo run -- bridge-status
```

### WASM Smart Contracts

**Compile Contract**:
```bash
cargo run -- compile-contract \
  contracts/examples/counter/src/lib.rs \
  counter
```

**Test Contract**:
```bash
cargo run -- test-contract counter
```

**Deploy Contract**:
```bash
cargo run -- deploy-wasm-contract \
  counter \
  --private-key <private_key_hex>
```

**View Contract Templates**:
```bash
cargo run -- contract-templates
```

### Smart Contracts (Legacy Stack-based)

**Deploy a Contract**:
```bash
# Create a simple contract file
echo "PUSH 0\nSTORE counter\nRETURN" > examples/counter.txt

# Deploy the contract
cargo run -- deploy-contract \
  --sender alice \
  --code-file examples/counter.txt \
  --gas-limit 1000000 \
  --gas-price 1.0
```

**Call a Contract**:
```bash
# Call the deployed contract
cargo run -- call-contract \
  --sender bob \
  --contract <contract_address> \
  --data "increment" \
  --amount 1.0 \
  --gas-limit 1000000 \
  --gas-price 1.0
```

### Proof-of-Stake

**Register as Validator**:
```bash
cargo run -- register-validator \
  --address validator1 \
  --public-key <public_key_hex> \
  --stake 5000.0
```

**Stake Tokens**:
```bash
cargo run -- stake \
  --address validator1 \
  --amount 1000.0
```

**View Validators**:
```bash
cargo run -- validators
```

### Transactions

**Create Transaction**:
```bash
cargo run -- add-transaction \
  --sender alice \
  --receiver bob \
  --amount 100.0 \
  --message "Payment for services"
```

**View Blockchain Status**:
```bash
cargo run -- stats
cargo run -- balances
cargo run -- pending
```

### Wallet Operations

**Create Wallet**:
```bash
cargo run -- create-wallet \
  --password mypassword \
  --name mywallet
```

**Send Transaction from Wallet**:
```bash
cargo run -- send-transaction \
  --from alice \
  --to bob \
  --amount 50.0 \
  --password mypassword
```

## Architecture

```
gillean/
├── src/
│   ├── blockchain.rs      # Main blockchain orchestrator
│   ├── block.rs          # Block structure and validation
│   ├── transaction.rs    # Transaction handling (public and private)
│   ├── zkp.rs           # Zero-knowledge proofs for privacy
│   ├── state_channels.rs # Layer 2 scaling with state channels
│   ├── smart_contract.rs # WASM-based smart contract VM
│   ├── consensus.rs      # PoS consensus mechanism
│   ├── proof_of_work.rs  # PoW mining algorithm
│   ├── wallet.rs         # Wallet management
│   ├── api.rs           # REST API server with WebSocket support
│   ├── storage.rs       # Persistent storage
│   ├── network.rs       # P2P networking
│   ├── monitor.rs       # Metrics and monitoring
│   ├── crypto.rs        # Cryptographic operations
│   ├── merkle.rs        # Merkle tree implementation
│   ├── sharding.rs      # Sharding implementation
│   ├── interop.rs       # Cross-chain interoperability
│   ├── contract_toolkit.rs # Contract development tools
│   └── main.rs          # Enhanced CLI interface
├── sdk/                 # Developer SDK
│   ├── src/
│   │   ├── lib.rs       # Main SDK interface
│   │   ├── client.rs    # HTTP client for API communication
│   │   ├── wallet.rs    # Wallet management
│   │   ├── contracts.rs # Smart contract integration
│   │   ├── transactions.rs # Transaction handling
│   │   └── analytics.rs # Analytics and monitoring
│   ├── Cargo.toml       # SDK dependencies
│   └── README.md        # SDK documentation
├── frontend/            # Enhanced web interface with analytics
│   ├── src/
│   │   ├── app.rs       # Main application
│   │   ├── components/  # UI components
│   │   └── api.rs       # API client with WebSocket support
│   ├── index.html       # HTML template
│   └── styles.css       # Application styles
├── contracts/           # WASM smart contracts
│   ├── examples/        # Contract examples
│   │   ├── counter/     # Counter contract
│   │   ├── voting/      # Voting contract
│   │   ├── escrow/      # Escrow contract
│   │   └── token/       # Token contract
│   └── README.md        # Contract development guide
└── tests/              # Integration tests
```

## Configuration

### ZKP Settings

- `--zkp-timeout`: ZKP proof generation timeout (default: 30 seconds)
- `--zkp-cache-size`: ZKP proof cache size (default: 1000)
- `--zkp-verification-key`: Path to ZKP verification key

### State Channel Settings

- `--channel-timeout`: Default state channel timeout (default: 1 hour)
- `--channel-dispute-period`: Dispute period for channels (default: 24 hours)
- `--max-channel-balance`: Maximum balance per channel

### SDK Settings

- `--sdk-api-url`: API server URL for SDK
- `--sdk-ws-url`: WebSocket URL for SDK
- `--sdk-timeout`: Request timeout for SDK
- `--sdk-retry-attempts`: Number of retry attempts for SDK

### Sharding Settings

- `--num-shards`: Number of shards in the system (default: 4)
- `--shard-consensus`: Consensus type for shards (pow/pos)
- `--cross-shard-timeout`: Timeout for cross-shard transactions

### Cross-Chain Settings

- `--bridge-fee`: Bridge fee percentage (default: 0.1%)
- `--external-chains`: List of connected external chains
- `--bridge-timeout`: Bridge transaction timeout

### WASM Contract Settings

- `--max-contract-size`: Maximum contract size in bytes (default: 1MB)
- `--gas-limit`: Maximum gas for contract execution
- `--gas-price`: Price per gas unit
- `--contract-timeout`: Contract execution timeout

### Consensus Settings

**Proof-of-Work**:
- `--difficulty`: Mining difficulty (number of leading zeros)
- `--reward`: Mining reward amount

**Proof-of-Stake**:
- `--min-stake`: Minimum stake required to become validator
- `--max-validators`: Maximum number of validators
- `--staking-reward-rate`: Annual staking reward rate (%)

### Network Settings

- `--api-address`: REST API server address
- `--network-address`: P2P network address
- `--db-path`: Database storage path
- `--websocket-port`: WebSocket server port

## Testing

Run the comprehensive test suite:

```bash
cargo test
```

Run integration tests:

```bash
cargo test --test integration_test
```

Run with coverage:

```bash
cargo tarpaulin
```

Test specific features:

```bash
# Test ZKP functionality
cargo test --package gillean --lib zkp

# Test state channels
cargo test --package gillean --lib state_channels

# Test SDK
cargo test --package gillean-sdk

# Test sharding
cargo test --package gillean --lib sharding

# Test cross-chain functionality
cargo test --package gillean --lib interop

# Test contract toolkit
cargo test --package gillean --lib contract_toolkit
```

## Enhanced Monitoring

### Metrics Endpoints

- `GET /api/metrics` - Comprehensive blockchain metrics
- `GET /api/health` - Health status
- `GET /api/blockchain/status` - Blockchain status
- `GET /api/zkp/stats` - ZKP statistics
- `GET /api/channels/stats` - State channel statistics
- `GET /api/sharding/stats` - Sharding statistics
- `GET /api/bridge/status` - Cross-chain bridge status
- `GET /api/contracts/templates` - Available contract templates
- `WS /api/ws` - WebSocket endpoint for real-time updates

### Key Metrics

- **Blockchain**: Total blocks, transactions, pending transactions
- **ZKP**: Proof generation time, verification success rate, cache hit rate
- **State Channels**: Open channels, total updates, average channel lifetime
- **Sharding**: Shard throughput, cross-shard latency, shard utilization
- **Cross-Chain**: Bridge transactions, transfer success rate, external chain status
- **WASM Contracts**: Deployments, calls, gas usage, execution time
- **PoS**: Validators, total stake, performance scores
- **Network**: Connected peers, sync status, WebSocket connections
- **SDK**: API calls, error rates, response times

## Security Features

- **Digital Signatures**: Ed25519 for transaction authentication
- **Encrypted Wallets**: AES-GCM encryption for wallet storage
- **Zero-Knowledge Proofs**: Privacy-preserving transactions
- **State Channel Security**: Cryptographic state verification
- **Input Validation**: Comprehensive validation for all inputs
- **Gas Limits**: Protection against infinite loops in contracts
- **Slashing**: Penalties for validator misbehavior
- **Contract Validation**: Secure contract execution environment
- **Cross-Chain Security**: Cryptographic verification for bridge transactions
- **Shard Isolation**: Secure isolation between shards
- **SDK Security**: Secure API communication with retry logic

## Future Enhancements

- **Advanced ZKP Schemes**: Support for more ZKP protocols (Bulletproofs, STARKs)
- **Multi-Party State Channels**: Support for more than two participants
- **Rollups**: Layer 2 scaling with optimistic and zk-rollups
- **Advanced Sharding**: Dynamic shard allocation and rebalancing
- **Full WebAssembly VM**: Complete WebAssembly support with WASI
- **Advanced Consensus**: DPoS, PBFT, and other consensus mechanisms
- **Real Cross-chain Integration**: Interoperability with actual blockchain networks
- **Decentralized Identity**: Self-sovereign identity system
- **Governance**: On-chain governance mechanisms
- **Mobile Support**: Cross-platform mobile applications
- **Advanced Contract Features**: Contract upgrades, libraries, and inheritance
- **AI Integration**: Machine learning for transaction analysis and fraud detection

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Install development dependencies
cargo install cargo-watch
cargo install cargo-tarpaulin

# Run with hot reload
cargo watch -x run

# Run frontend with hot reload
cd frontend && trunk serve

# Test WASM contracts
cd contracts && cargo test

# Test SDK
cd sdk && cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Bitcoin and Ethereum architectures
- Built with Rust ecosystem tools
- Uses modern web technologies for the frontend
- Yew framework for WebAssembly-based frontend
- WebAssembly for high-performance smart contracts
- RISC0 for zero-knowledge proofs
- Cross-chain interoperability concepts from Polkadot and Cosmos
- State channel concepts from Lightning Network and Raiden

