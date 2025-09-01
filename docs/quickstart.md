# Quick Start Tutorial

This tutorial will guide you through setting up and using the Gillean blockchain platform in just a few minutes.

## Prerequisites

- Rust 1.70.0+ installed
- Basic familiarity with command line
- 5-10 minutes of time

## Step 1: Setup

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/your-org/gillean.git
cd gillean

# Build the project
cargo build --release

# Run tests to ensure everything works
./run_comprehensive_tests.sh
```

### Start the Blockchain Node

```bash
# Start the blockchain node
cargo run --release
```

You should see output like:
```
🚀 Starting Gillean Blockchain v2.0.0...
📡 API server listening on http://localhost:3000
🔗 WebSocket server listening on ws://localhost:3000/ws
⛓️  Blockchain node started successfully
```

## Step 2: Create Your First Wallet

### Using the CLI

```bash
# In a new terminal, create a wallet
gillean wallet create --name "My First Wallet" --password "secure123"
```

Output:
```
✅ Wallet created successfully!
📝 Wallet Details:
   Address: 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6
   Name: My First Wallet
   Balance: 0.0 GIL
   Created: 2025-08-31 21:50:00 UTC
```

### Using the API

```bash
# Create wallet via API
curl -X POST http://localhost:3000/api/v1/wallets \
  -H "Content-Type: application/json" \
  -d '{
    "name": "API Wallet",
    "password": "secure123"
  }'
```

## Step 3: Send Your First Transaction

### Get Some Test Tokens

```bash
# Mine a block to get rewards (if you're running a miner)
gillean mine --wallet "My First Wallet" --password "secure123"

# Or transfer from the genesis wallet
gillean transfer \
  --from "Genesis" \
  --to "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6" \
  --amount 100.0 \
  --password "genesis"
```

### Send a Transaction

```bash
# Create a second wallet
gillean wallet create --name "Recipient Wallet" --password "secure456"

# Send tokens
gillean transfer \
  --from "My First Wallet" \
  --to "0x1234567890123456789012345678901234567890" \
  --amount 10.0 \
  --password "secure123"
```

## Step 4: Explore the Blockchain

### View Blockchain Status

```bash
# Check blockchain status
curl http://localhost:3000/api/v1/status

# View latest blocks
curl http://localhost:3000/api/v1/blocks/latest

# Get wallet balance
curl http://localhost:3000/api/v1/wallets/0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6/balance
```

### Use the Web Interface

Open your browser and navigate to `http://localhost:3000` to access the web interface.

## Step 5: Deploy Your First Smart Contract

### Create a Simple Contract

Create a file `contracts/simple_counter.wasm` (or use the example):

```rust
// Simple counter contract
#[no_mangle]
pub extern "C" fn increment() -> i32 {
    // Contract logic here
    42
}
```

### Deploy the Contract

```bash
# Deploy the contract
gillean contract deploy \
  --wallet "My First Wallet" \
  --password "secure123" \
  --file contracts/simple_counter.wasm \
  --name "Simple Counter"
```

### Interact with the Contract

```bash
# Call the contract
gillean contract call \
  --wallet "My First Wallet" \
  --password "secure123" \
  --contract "Simple Counter" \
  --function "increment" \
  --args ""
```

## Step 6: Try Advanced Features

### Zero-Knowledge Proofs

```bash
# Create a private transaction
gillean zkp create-private-transaction \
  --from "My First Wallet" \
  --to "0x1234567890123456789012345678901234567890" \
  --amount 5.0 \
  --password "secure123"
```

### State Channels

```bash
# Open a state channel
gillean channel open \
  --wallet "My First Wallet" \
  --password "secure123" \
  --participant "0x1234567890123456789012345678901234567890" \
  --amount 50.0
```

### Cross-Chain Transfer

```bash
# Initiate cross-chain transfer to Ethereum
gillean bridge transfer \
  --from "My First Wallet" \
  --to "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6" \
  --amount 10.0 \
  --target-chain "ethereum" \
  --password "secure123"
```

## Step 7: Monitor and Analytics

### View Real-time Metrics

```bash
# Get blockchain metrics
curl http://localhost:3000/api/v1/metrics

# Get performance statistics
curl http://localhost:3000/api/v1/performance/stats

# Get security audit results
curl http://localhost:3000/api/v1/security/audit
```

### Use the Monitoring Dashboard

Navigate to `http://localhost:3000/dashboard` for real-time monitoring.

## Common Commands Reference

### Wallet Management

```bash
# List all wallets
gillean wallet list

# Get wallet details
gillean wallet info --address 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6

# Export wallet
gillean wallet export --name "My First Wallet" --password "secure123"

# Import wallet
gillean wallet import --file wallet.json --password "secure123"
```

### Transaction Management

```bash
# View transaction history
gillean transaction history --wallet "My First Wallet"

# Get transaction details
gillean transaction info --tx-hash 0x123...

# View pending transactions
gillean transaction pending
```

### Blockchain Information

```bash
# Get latest block
gillean block latest

# Get block by height
gillean block get --height 100

# Get blockchain statistics
gillean stats

# Get network status
gillean network status
```

### Smart Contracts

```bash
# List deployed contracts
gillean contract list

# Get contract details
gillean contract info --name "Simple Counter"

# View contract events
gillean contract events --name "Simple Counter"
```

## Next Steps

Now that you've completed the quick start tutorial, explore these advanced topics:

1. **[API Reference](api.md)** - Complete API documentation
2. **[Smart Contract Development](contracts.md)** - Build and deploy contracts
3. **[Zero-Knowledge Proofs](zkp.md)** - Privacy-preserving transactions
4. **[State Channels](state-channels.md)** - Layer 2 scaling
5. **[Cross-Chain Bridges](cross-chain.md)** - Interoperability
6. **[Decentralized Identity](did.md)** - Self-sovereign identity
7. **[Governance](governance.md)** - On-chain governance
8. **[AI Integration](ai-integration.md)** - Machine learning features
9. **[Mobile Development](mobile.md)** - Mobile applications

## Troubleshooting

### Common Issues

**Node won't start:**
```bash
# Check if port is in use
lsof -i :3000

# Kill existing process
kill -9 <PID>

# Start with different port
cargo run -- --port 3001
```

**Transaction fails:**
```bash
# Check wallet balance
gillean wallet info --name "My First Wallet"

# Check transaction status
gillean transaction info --tx-hash <hash>
```

**Contract deployment fails:**
```bash
# Check contract syntax
gillean contract validate --file contracts/simple_counter.wasm

# Check gas estimation
gillean contract estimate-gas --file contracts/simple_counter.wasm
```

### Getting Help

- **Documentation**: [docs.gillean.org](https://docs.gillean.org)
- **Community**: [community.gillean.org](https://community.gillean.org)
- **Discord**: [discord.gg/gillean](https://discord.gg/gillean)
- **GitHub Issues**: [github.com/your-org/gillean/issues](https://github.com/your-org/gillean/issues)

## Congratulations! 🎉

You've successfully:
- ✅ Set up the Gillean blockchain
- ✅ Created your first wallet
- ✅ Sent transactions
- ✅ Deployed a smart contract
- ✅ Explored advanced features

You're now ready to build decentralized applications on Gillean!
