# Gillean Smart Contracts

This directory contains WebAssembly (WASM) smart contracts for the Gillean blockchain platform.

## Overview

The contracts are written in Rust and compiled to WASM bytecode for execution on the Gillean blockchain. Each contract follows the Gillean contract framework and provides specific functionality.

## Contract Examples

### Counter Contract
A simple counter contract with increment and decrement functionality.

**Features:**
- Increment counter
- Decrement counter
- Reset counter
- Get current value

**Usage:**
```bash
cd examples/counter
cargo build --target wasm32-unknown-unknown --release
```

### Voting Contract
A governance contract for proposal creation and voting.

**Features:**
- Create proposals
- Vote on proposals
- Check voting results
- Proposal management

**Usage:**
```bash
cd examples/voting
cargo build --target wasm32-unknown-unknown --release
```

### Escrow Contract
A secure escrow system for transactions between parties.

**Features:**
- Create escrow transactions
- Release funds to seller
- Refund buyer
- Timeout handling

**Usage:**
```bash
cd examples/escrow
cargo build --target wasm32-unknown-unknown --release
```

### Token Contract
An ERC-20 compatible token contract.

**Features:**
- Token transfers
- Balance checking
- Approval system
- Total supply management

**Usage:**
```bash
cd examples/token
cargo build --target wasm32-unknown-unknown --release
```

## Development Setup

### Prerequisites

1. **Rust Toolchain**: Install the latest stable Rust
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **WASM Target**: Add the WASM target
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **wasm-pack** (optional): For advanced WASM development
   ```bash
   cargo install wasm-pack
   ```

### Building Contracts

To build all contracts:
```bash
cargo build --target wasm32-unknown-unknown --release
```

To build a specific contract:
```bash
cd examples/counter
cargo build --target wasm32-unknown-unknown --release
```

### Testing Contracts

To test all contracts:
```bash
cargo test
```

To test a specific contract:
```bash
cd examples/counter
cargo test
```

## Contract Framework

### Contract Structure

All contracts follow this basic structure:

```rust
use gillean_contract::*;

#[contract]
pub struct MyContract {
    // Contract state
}

impl MyContract {
    #[constructor]
    pub fn new() -> Self {
        // Constructor logic
    }
    
    #[view]
    pub fn get_value(&self) -> u64 {
        // Read-only function
    }
    
    #[payable]
    pub fn update_value(&mut self, new_value: u64) {
        // State-changing function
    }
}
```

### Available Attributes

- `#[contract]`: Marks a struct as a smart contract
- `#[constructor]`: Marks a function as the contract constructor
- `#[view]`: Marks a function as read-only (no state changes)
- `#[payable]`: Marks a function as state-changing (can modify contract state)

### Contract Lifecycle

1. **Deployment**: Contract is deployed to the blockchain
2. **Initialization**: Constructor is called with initial parameters
3. **Execution**: Contract functions can be called by users
4. **State Management**: Contract state is stored on the blockchain

## Deployment

### Using the Contract Toolkit

The Gillean blockchain includes a contract development toolkit for easy deployment:

```bash
# Compile a contract
cargo run -- compile-contract examples/counter/src/lib.rs counter

# Test a contract
cargo run -- test-contract counter

# Deploy a contract
cargo run -- deploy-wasm-contract counter
```

### Manual Deployment

1. Build the contract to WASM:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Deploy using the blockchain API:
   ```bash
   curl -X POST http://localhost:3000/api/contracts/deploy \
     -H "Content-Type: application/json" \
     -d '{
       "name": "counter",
       "wasm_bytecode": "base64_encoded_wasm",
       "constructor_args": []
     }'
   ```

## Best Practices

### Security

1. **Input Validation**: Always validate function parameters
2. **Access Control**: Implement proper access control mechanisms
3. **Reentrancy Protection**: Protect against reentrancy attacks
4. **Gas Optimization**: Optimize contract code for gas efficiency

### Code Quality

1. **Documentation**: Document all public functions
2. **Testing**: Write comprehensive tests for all functionality
3. **Error Handling**: Implement proper error handling
4. **Code Review**: Review code before deployment

### Performance

1. **Gas Efficiency**: Minimize gas usage in functions
2. **Storage Optimization**: Optimize storage patterns
3. **Batch Operations**: Use batch operations when possible
4. **Caching**: Cache frequently accessed data

## Troubleshooting

### Common Issues

1. **Compilation Errors**: Ensure all dependencies are properly specified
2. **WASM Size**: Keep contract size under the maximum limit (1MB)
3. **Gas Limits**: Ensure functions don't exceed gas limits
4. **State Management**: Properly handle contract state

### Debugging

1. **Logs**: Use the blockchain's logging system for debugging
2. **Tests**: Write unit tests to verify functionality
3. **Simulation**: Test contracts in a local environment first
4. **Monitoring**: Monitor contract execution and gas usage

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your contract
4. Add tests
5. Submit a pull request

### Contract Guidelines

1. Follow the established contract patterns
2. Include comprehensive documentation
3. Write unit tests for all functions
4. Ensure security best practices
5. Optimize for gas efficiency

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.
