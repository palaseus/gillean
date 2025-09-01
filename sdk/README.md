# Gillean SDK

A Rust SDK for interacting with the Gillean blockchain platform. This SDK provides easy-to-use interfaces for wallet management, transaction handling, smart contract operations, state channels, and analytics.

## Features

- **Wallet Management**: Create, import, and manage wallets with secure key storage
- **Transaction Handling**: Send regular and private transactions with ZKP support
- **Smart Contracts**: Deploy and interact with WebAssembly smart contracts
- **State Channels**: Open, update, and close off-chain state channels
- **Analytics**: Access real-time and historical blockchain analytics
- **Real-time Updates**: Subscribe to blockchain events via WebSocket

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
gillean-sdk = "2.0.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use gillean_sdk::{GilleanSDK, SDKConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create SDK configuration
    let config = SDKConfig {
        api_url: "http://localhost:3000".to_string(),
        ws_url: "ws://localhost:3000/ws".to_string(),
        api_key: None,
        timeout: std::time::Duration::from_secs(30),
        retry_attempts: 3,
    };

    // Initialize SDK
    let sdk = GilleanSDK::new(config).await?;

    // Get blockchain status
    let status = sdk.get_blockchain_status().await?;
    println!("Blockchain status: {:?}", status);

    Ok(())
}
```

## Usage Examples

### Wallet Management

```rust
// Create a new wallet
let wallet = sdk.create_wallet("my_password", Some("My Wallet")).await?;
println!("Created wallet: {}", wallet.address);

// Import existing wallet
let imported_wallet = sdk.import_wallet(
    "private_key_hex",
    "my_password",
    Some("Imported Wallet")
).await?;

// List all wallets
let wallets = sdk.list_wallets().await?;
for wallet in wallets {
    println!("Wallet: {} - Balance: {}", wallet.address, wallet.balance);
}
```

### Sending Transactions

```rust
// Send a regular transaction
let result = sdk.send_transaction(
    "sender_address",
    "receiver_address",
    100.0,
    "my_password",
    Some("Payment for services")
).await?;

println!("Transaction sent: {}", result.transaction_hash);

// Create a private transaction with ZKP
let private_result = sdk.create_private_transaction(
    "sender_address",
    "receiver_address",
    50.0,
    "my_password",
    Some("Private payment")
).await?;

println!("Private transaction created: {}", private_result.zk_proof_id);
```

### Smart Contracts

```rust
// Deploy a smart contract
let contract_code = std::fs::read("contract.wasm")?;
let deploy_result = sdk.deploy_contract(
    "MyContract",
    &contract_code,
    "deployer_address",
    "my_password",
    1000000
).await?;

println!("Contract deployed: {}", deploy_result.contract_address);

// Call a smart contract
let call_result = sdk.call_contract(
    &deploy_result.contract_address,
    "increment",
    &[],
    "caller_address",
    "my_password",
    None
).await?;

println!("Contract call result: {:?}", call_result.return_value);
```

### State Channels

```rust
// Open a state channel
let channel_result = sdk.open_state_channel(
    "alice",
    "bob",
    1000.0,
    3600, // 1 hour timeout
    "my_password"
).await?;

println!("State channel opened: {}", channel_result.channel_id);

// Update channel state
let new_balance = HashMap::from([
    ("alice".to_string(), 600.0),
    ("bob".to_string(), 400.0),
]);

let update_result = sdk.update_state_channel(
    &channel_result.channel_id,
    new_balance,
    "my_password"
).await?;

// Close the channel
let close_result = sdk.close_state_channel(
    &channel_result.channel_id,
    new_balance,
    "my_password"
).await?;
```

### Analytics

```rust
// Get transaction volume analytics
let analytics = sdk.get_analytics(AnalyticsMetric::TransactionVolume).await?;
println!("Total transactions: {}", analytics.summary.total);

// Get real-time analytics
let realtime = sdk.get_realtime_analytics().await?;
println!("TPS: {}", realtime.get("transactions_per_second").unwrap_or(&0.0));

// Subscribe to real-time updates
let mut updates = sdk.subscribe_to_updates(vec![
    EventType::NewBlock,
    EventType::NewTransaction,
    EventType::ZKPProofGenerated,
]).await?;

while let Some(event) = updates.recv().await {
    println!("Received event: {:?}", event.event_type);
}
```

## Error Handling

The SDK uses a custom error type `SDKError` that provides detailed error information:

```rust
use gillean_sdk::SDKError;

match sdk.send_transaction(from, to, amount, password, None).await {
    Ok(result) => println!("Transaction successful: {}", result.transaction_hash),
    Err(SDKError::NetworkError(msg)) => println!("Network error: {}", msg),
    Err(SDKError::AuthError(msg)) => println!("Authentication error: {}", msg),
    Err(SDKError::InvalidInput(msg)) => println!("Invalid input: {}", msg),
    Err(e) => println!("Other error: {}", e),
}
```

## Configuration

The `SDKConfig` struct allows you to customize the SDK behavior:

```rust
let config = SDKConfig {
    api_url: "https://api.gillean.com".to_string(),
    ws_url: "wss://api.gillean.com/ws".to_string(),
    api_key: Some("your_api_key".to_string()),
    timeout: std::time::Duration::from_secs(60),
    retry_attempts: 5,
};
```

## Testing

The SDK includes comprehensive tests. Run them with:

```bash
cargo test
```

## API Reference

### Core Types

- `GilleanSDK`: Main SDK interface
- `SDKConfig`: Configuration for the SDK
- `SDKError`: Error types returned by the SDK
- `SDKResult<T>`: Result type for SDK operations

### Wallet Management

- `WalletInfo`: Information about a wallet
- `create_wallet()`: Create a new wallet
- `import_wallet()`: Import an existing wallet
- `list_wallets()`: List all wallets
- `sign_transaction()`: Sign transaction data

### Transactions

- `TransactionResult`: Result of a regular transaction
- `PrivateTransactionResult`: Result of a private transaction
- `send_transaction()`: Send a regular transaction
- `create_private_transaction()`: Create a private transaction
- `get_transaction_status()`: Get transaction status

### Smart Contracts

- `ContractDeployResult`: Result of contract deployment
- `ContractCallResult`: Result of contract call
- `deploy_contract()`: Deploy a smart contract
- `call_contract()`: Call a smart contract method

### State Channels

- `StateChannelResult`: Result of opening a state channel
- `StateChannelUpdateResult`: Result of updating a state channel
- `StateChannelCloseResult`: Result of closing a state channel
- `open_state_channel()`: Open a new state channel
- `update_state_channel()`: Update channel state
- `close_state_channel()`: Close a state channel

### Analytics

- `AnalyticsData`: Analytics data with data points and summary
- `AnalyticsMetric`: Types of analytics metrics
- `get_analytics()`: Get analytics for a specific metric
- `get_realtime_analytics()`: Get real-time analytics
- `subscribe_to_updates()`: Subscribe to real-time events

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions:

- Create an issue on GitHub
- Check the documentation
- Join our community discussions

## Changelog

### v2.0.0
- Initial release with core functionality
- Wallet management
- Transaction handling
- Smart contract support
- State channels
- Analytics and real-time updates
- ZKP support for private transactions
