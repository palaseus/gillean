# Smart Contract Development Guide

This guide provides comprehensive information for developing smart contracts on the Gillean blockchain platform using WebAssembly (WASM).

## Overview

Gillean supports smart contracts written in any language that can compile to WebAssembly, with full WASI (WebAssembly System Interface) support for enhanced functionality.

## Getting Started

### Prerequisites

- **Rust**: Version 1.70.0 or higher (recommended for contract development)
- **wasm-pack**: For building WASM contracts
- **Node.js**: For testing and deployment tools
- **Gillean CLI**: For contract deployment and interaction

### Development Environment Setup

1. **Install wasm-pack**
   ```bash
   cargo install wasm-pack
   ```

2. **Install Gillean CLI**
   ```bash
   cargo install gillean-cli
   ```

3. **Create a New Contract Project**
   ```bash
   # Create a new contract directory
   mkdir my-contract
   cd my-contract
   
   # Initialize a new Rust project
   cargo init --lib
   ```

4. **Configure Cargo.toml**
   ```toml
   [package]
   name = "my-contract"
   version = "0.1.0"
   edition = "2021"
   
   [lib]
   crate-type = ["cdylib"]
   
   [dependencies]
   gillean-contract = "2.0.0"
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   wasm-bindgen = "0.2"
   
   [dev-dependencies]
   wasm-bindgen-test = "0.3"
   ```

## Contract Structure

### Basic Contract Template

```rust
use gillean_contract::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractState {
    pub owner: String,
    pub balance: u64,
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
}

#[wasm_bindgen]
pub struct MyContract {
    state: ContractState,
}

#[wasm_bindgen]
impl MyContract {
    /// Initialize the contract
    #[wasm_bindgen(constructor)]
    pub fn new(owner: String) -> Self {
        let state = ContractState {
            owner,
            balance: 0,
            transactions: Vec::new(),
        };
        
        Self { state }
    }
    
    /// Get contract state
    pub fn get_state(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.state).unwrap()
    }
    
    /// Transfer funds
    pub fn transfer(&mut self, to: String, amount: u64) -> Result<(), JsValue> {
        // Validate transaction
        if amount > self.state.balance {
            return Err("Insufficient balance".into());
        }
        
        // Update state
        self.state.balance -= amount;
        
        // Record transaction
        let tx = Transaction {
            from: self.state.owner.clone(),
            to,
            amount,
            timestamp: get_current_timestamp(),
        };
        self.state.transactions.push(tx);
        
        Ok(())
    }
    
    /// Deposit funds
    pub fn deposit(&mut self, amount: u64) {
        self.state.balance += amount;
    }
    
    /// Get transaction history
    pub fn get_transactions(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.state.transactions).unwrap()
    }
}

// Helper function to get current timestamp
fn get_current_timestamp() -> u64 {
    // In a real contract, this would come from the blockchain
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_contract_creation() {
        let contract = MyContract::new("alice".to_string());
        let state: ContractState = serde_wasm_bindgen::from_value(contract.get_state()).unwrap();
        
        assert_eq!(state.owner, "alice");
        assert_eq!(state.balance, 0);
    }
    
    #[wasm_bindgen_test]
    fn test_transfer() {
        let mut contract = MyContract::new("alice".to_string());
        contract.deposit(100);
        
        let result = contract.transfer("bob".to_string(), 50);
        assert!(result.is_ok());
        
        let state: ContractState = serde_wasm_bindgen::from_value(contract.get_state()).unwrap();
        assert_eq!(state.balance, 50);
    }
}
```

## Contract Development Patterns

### 1. State Management

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractState {
    pub owner: String,
    pub balances: HashMap<String, u64>,
    pub permissions: HashMap<String, Vec<String>>,
    pub metadata: ContractMetadata,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl ContractState {
    pub fn new(owner: String, name: String) -> Self {
        let mut balances = HashMap::new();
        balances.insert(owner.clone(), 0);
        
        let mut permissions = HashMap::new();
        permissions.insert(owner.clone(), vec!["admin".to_string()]);
        
        let metadata = ContractMetadata {
            name,
            version: "1.0.0".to_string(),
            description: "Smart contract".to_string(),
            created_at: get_current_timestamp(),
            updated_at: get_current_timestamp(),
        };
        
        Self {
            owner,
            balances,
            permissions,
            metadata,
        }
    }
    
    pub fn has_permission(&self, address: &str, permission: &str) -> bool {
        if let Some(permissions) = self.permissions.get(address) {
            permissions.contains(&permission.to_string())
        } else {
            false
        }
    }
    
    pub fn update_metadata(&mut self) {
        self.metadata.updated_at = get_current_timestamp();
    }
}
```

### 2. Access Control

```rust
pub trait AccessControl {
    fn require_owner(&self, caller: &str) -> Result<(), String>;
    fn require_permission(&self, caller: &str, permission: &str) -> Result<(), String>;
    fn grant_permission(&mut self, target: String, permission: String) -> Result<(), String>;
    fn revoke_permission(&mut self, target: String, permission: String) -> Result<(), String>;
}

impl AccessControl for MyContract {
    fn require_owner(&self, caller: &str) -> Result<(), String> {
        if caller != self.state.owner {
            return Err("Caller is not the owner".to_string());
        }
        Ok(())
    }
    
    fn require_permission(&self, caller: &str, permission: &str) -> Result<(), String> {
        if !self.state.has_permission(caller, permission) {
            return Err(format!("Caller lacks permission: {}", permission));
        }
        Ok(())
    }
    
    fn grant_permission(&mut self, target: String, permission: String) -> Result<(), String> {
        self.require_owner(&get_caller_address())?;
        
        let permissions = self.state.permissions.entry(target).or_insert_with(Vec::new);
        if !permissions.contains(&permission) {
            permissions.push(permission);
        }
        
        Ok(())
    }
    
    fn revoke_permission(&mut self, target: String, permission: String) -> Result<(), String> {
        self.require_owner(&get_caller_address())?;
        
        if let Some(permissions) = self.state.permissions.get_mut(&target) {
            permissions.retain(|p| p != &permission);
        }
        
        Ok(())
    }
}
```

### 3. Event System

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ContractEvent {
    Transfer {
        from: String,
        to: String,
        amount: u64,
        timestamp: u64,
    },
    PermissionGranted {
        target: String,
        permission: String,
        granted_by: String,
    },
    PermissionRevoked {
        target: String,
        permission: String,
        revoked_by: String,
    },
}

pub trait EventEmitter {
    fn emit_event(&self, event: ContractEvent);
}

impl EventEmitter for MyContract {
    fn emit_event(&self, event: ContractEvent) {
        // In a real contract, this would emit to the blockchain
        log::info!("Event: {:?}", event);
    }
}
```

### 4. Upgradeable Contracts

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpgradeableContract {
    pub implementation: String,
    pub admin: String,
    pub upgrade_timelock: u64,
    pub pending_upgrade: Option<PendingUpgrade>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PendingUpgrade {
    pub new_implementation: String,
    pub proposed_at: u64,
    pub proposed_by: String,
}

impl UpgradeableContract {
    pub fn new(admin: String) -> Self {
        Self {
            implementation: "1.0.0".to_string(),
            admin,
            upgrade_timelock: 24 * 60 * 60, // 24 hours
            pending_upgrade: None,
        }
    }
    
    pub fn propose_upgrade(&mut self, new_implementation: String) -> Result<(), String> {
        self.require_admin(&get_caller_address())?;
        
        let pending = PendingUpgrade {
            new_implementation,
            proposed_at: get_current_timestamp(),
            proposed_by: get_caller_address(),
        };
        
        self.pending_upgrade = Some(pending);
        Ok(())
    }
    
    pub fn execute_upgrade(&mut self) -> Result<(), String> {
        self.require_admin(&get_caller_address())?;
        
        if let Some(pending) = &self.pending_upgrade {
            let now = get_current_timestamp();
            if now >= pending.proposed_at + self.upgrade_timelock {
                self.implementation = pending.new_implementation.clone();
                self.pending_upgrade = None;
                Ok(())
            } else {
                Err("Timelock not expired".to_string())
            }
        } else {
            Err("No pending upgrade".to_string())
        }
    }
    
    fn require_admin(&self, caller: &str) -> Result<(), String> {
        if caller != self.admin {
            return Err("Caller is not admin".to_string());
        }
        Ok(())
    }
}
```

## Advanced Features

### 1. Contract Libraries

```rust
// lib.rs
pub mod math;
pub mod crypto;
pub mod storage;

pub use math::*;
pub use crypto::*;
pub use storage::*;

// math.rs
pub mod math {
    pub fn add(a: u64, b: u64) -> u64 {
        a + b
    }
    
    pub fn multiply(a: u64, b: u64) -> u64 {
        a * b
    }
    
    pub fn safe_add(a: u64, b: u64) -> Result<u64, String> {
        a.checked_add(b).ok_or_else(|| "Overflow".to_string())
    }
}

// crypto.rs
pub mod crypto {
    use sha2::{Sha256, Digest};
    
    pub fn hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
    
    pub fn verify_signature(message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
        // Implement signature verification
        true // Placeholder
    }
}

// storage.rs
pub mod storage {
    use std::collections::HashMap;
    
    pub struct Storage {
        data: HashMap<String, Vec<u8>>,
    }
    
    impl Storage {
        pub fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }
        
        pub fn set(&mut self, key: String, value: Vec<u8>) {
            self.data.insert(key, value);
        }
        
        pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
            self.data.get(key)
        }
        
        pub fn delete(&mut self, key: &str) {
            self.data.remove(key);
        }
    }
}
```

### 2. Contract Inheritance

```rust
// base_contract.rs
pub trait BaseContract {
    fn get_owner(&self) -> &str;
    fn get_balance(&self) -> u64;
    fn transfer(&mut self, to: String, amount: u64) -> Result<(), String>;
}

// token_contract.rs
pub struct TokenContract {
    base: BaseContractImpl,
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u64,
}

impl TokenContract {
    pub fn new(name: String, symbol: String, decimals: u8, total_supply: u64) -> Self {
        let base = BaseContractImpl::new();
        
        Self {
            base,
            name,
            symbol,
            decimals,
            total_supply,
        }
    }
    
    pub fn mint(&mut self, to: String, amount: u64) -> Result<(), String> {
        self.base.require_owner(&get_caller_address())?;
        self.base.transfer(to, amount)
    }
    
    pub fn burn(&mut self, from: String, amount: u64) -> Result<(), String> {
        self.base.require_owner(&get_caller_address())?;
        // Implement burn logic
        Ok(())
    }
}

impl BaseContract for TokenContract {
    fn get_owner(&self) -> &str {
        self.base.get_owner()
    }
    
    fn get_balance(&self) -> u64 {
        self.base.get_balance()
    }
    
    fn transfer(&mut self, to: String, amount: u64) -> Result<(), String> {
        self.base.transfer(to, amount)
    }
}
```

### 3. Multi-Signature Contracts

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MultiSigContract {
    pub owners: Vec<String>,
    pub required_signatures: usize,
    pub pending_transactions: HashMap<String, PendingTransaction>,
    pub executed_transactions: Vec<ExecutedTransaction>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PendingTransaction {
    pub to: String,
    pub value: u64,
    pub data: Vec<u8>,
    pub signatures: Vec<String>,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExecutedTransaction {
    pub tx_id: String,
    pub to: String,
    pub value: u64,
    pub data: Vec<u8>,
    pub executed_at: u64,
    pub executed_by: String,
}

impl MultiSigContract {
    pub fn new(owners: Vec<String>, required_signatures: usize) -> Self {
        Self {
            owners,
            required_signatures,
            pending_transactions: HashMap::new(),
            executed_transactions: Vec::new(),
        }
    }
    
    pub fn propose_transaction(&mut self, to: String, value: u64, data: Vec<u8>) -> Result<String, String> {
        self.require_owner(&get_caller_address())?;
        
        let tx_id = generate_tx_id(&to, &value, &data);
        let pending = PendingTransaction {
            to,
            value,
            data,
            signatures: vec![get_caller_address()],
            created_at: get_current_timestamp(),
        };
        
        self.pending_transactions.insert(tx_id.clone(), pending);
        Ok(tx_id)
    }
    
    pub fn sign_transaction(&mut self, tx_id: String) -> Result<(), String> {
        self.require_owner(&get_caller_address())?;
        
        if let Some(pending) = self.pending_transactions.get_mut(&tx_id) {
            if !pending.signatures.contains(&get_caller_address()) {
                pending.signatures.push(get_caller_address());
            }
        } else {
            return Err("Transaction not found".to_string());
        }
        
        Ok(())
    }
    
    pub fn execute_transaction(&mut self, tx_id: String) -> Result<(), String> {
        self.require_owner(&get_caller_address())?;
        
        if let Some(pending) = self.pending_transactions.get(&tx_id) {
            if pending.signatures.len() >= self.required_signatures {
                // Execute the transaction
                let executed = ExecutedTransaction {
                    tx_id: tx_id.clone(),
                    to: pending.to.clone(),
                    value: pending.value,
                    data: pending.data.clone(),
                    executed_at: get_current_timestamp(),
                    executed_by: get_caller_address(),
                };
                
                self.executed_transactions.push(executed);
                self.pending_transactions.remove(&tx_id);
                Ok(())
            } else {
                Err("Insufficient signatures".to_string())
            }
        } else {
            Err("Transaction not found".to_string())
        }
    }
    
    fn require_owner(&self, caller: &str) -> Result<(), String> {
        if !self.owners.contains(&caller.to_string()) {
            return Err("Caller is not an owner".to_string());
        }
        Ok(())
    }
}

fn generate_tx_id(to: &str, value: &u64, data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(to.as_bytes());
    hasher.update(value.to_le_bytes());
    hasher.update(data);
    
    format!("{:x}", hasher.finalize())
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_contract_creation() {
        let contract = MyContract::new("alice".to_string());
        let state: ContractState = serde_wasm_bindgen::from_value(contract.get_state()).unwrap();
        
        assert_eq!(state.owner, "alice");
        assert_eq!(state.balance, 0);
    }
    
    #[wasm_bindgen_test]
    fn test_transfer_success() {
        let mut contract = MyContract::new("alice".to_string());
        contract.deposit(100);
        
        let result = contract.transfer("bob".to_string(), 50);
        assert!(result.is_ok());
        
        let state: ContractState = serde_wasm_bindgen::from_value(contract.get_state()).unwrap();
        assert_eq!(state.balance, 50);
    }
    
    #[wasm_bindgen_test]
    fn test_transfer_insufficient_balance() {
        let mut contract = MyContract::new("alice".to_string());
        contract.deposit(50);
        
        let result = contract.transfer("bob".to_string(), 100);
        assert!(result.is_err());
        
        let state: ContractState = serde_wasm_bindgen::from_value(contract.get_state()).unwrap();
        assert_eq!(state.balance, 50); // Balance unchanged
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
use gillean_contract::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_contract_deployment() {
    // Test contract deployment
    let contract = MyContract::new("alice".to_string());
    
    // Verify initial state
    let state: ContractState = serde_wasm_bindgen::from_value(contract.get_state()).unwrap();
    assert_eq!(state.owner, "alice");
}

#[wasm_bindgen_test]
async fn test_complete_workflow() {
    let mut contract = MyContract::new("alice".to_string());
    
    // Deposit funds
    contract.deposit(1000);
    
    // Transfer funds
    contract.transfer("bob".to_string(), 300).unwrap();
    contract.transfer("charlie".to_string(), 200).unwrap();
    
    // Verify final state
    let state: ContractState = serde_wasm_bindgen::from_value(contract.get_state()).unwrap();
    assert_eq!(state.balance, 500);
    assert_eq!(state.transactions.len(), 2);
}
```

## Building and Deployment

### Building Contracts

```bash
# Build the contract
wasm-pack build --target web

# Build for production
wasm-pack build --target web --release
```

### Deploying Contracts

```bash
# Deploy using Gillean CLI
gillean contract deploy ./pkg/my_contract.wasm

# Deploy with initial parameters
gillean contract deploy ./pkg/my_contract.wasm --args '["alice"]'

# Deploy with gas limit
gillean contract deploy ./pkg/my_contract.wasm --gas 1000000
```

### Interacting with Contracts

```bash
# Call contract function
gillean contract call <contract_address> transfer --args '["bob", 100]'

# Query contract state
gillean contract query <contract_address> get_state

# Get transaction history
gillean contract query <contract_address> get_transactions
```

## Best Practices

### 1. Security

- **Input Validation**: Always validate all inputs
- **Access Control**: Implement proper access control mechanisms
- **Reentrancy Protection**: Use reentrancy guards
- **Overflow Protection**: Use checked arithmetic operations
- **Gas Optimization**: Optimize for gas efficiency

### 2. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },
    
    #[error("Access denied: caller {caller} lacks permission {permission}")]
    AccessDenied { caller: String, permission: String },
    
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
    
    #[error("Contract state error: {message}")]
    StateError { message: String },
}

impl From<ContractError> for JsValue {
    fn from(error: ContractError) -> Self {
        JsValue::from_str(&error.to_string())
    }
}
```

### 3. Gas Optimization

```rust
// Use efficient data structures
use std::collections::HashMap;

// Avoid unnecessary storage operations
pub fn optimized_transfer(&mut self, to: String, amount: u64) -> Result<(), ContractError> {
    // Batch operations when possible
    if amount > self.state.balance {
        return Err(ContractError::InsufficientBalance {
            required: amount,
            available: self.state.balance,
        });
    }
    
    // Single state update
    self.state.balance -= amount;
    
    // Only store essential data
    let tx = Transaction {
        from: self.state.owner.clone(),
        to,
        amount,
        timestamp: get_current_timestamp(),
    };
    self.state.transactions.push(tx);
    
    Ok(())
}
```

### 4. Testing Strategy

- **Unit Tests**: Test individual functions
- **Integration Tests**: Test complete workflows
- **Property-Based Tests**: Test with random inputs
- **Gas Tests**: Test gas consumption
- **Security Tests**: Test edge cases and vulnerabilities

## Debugging

### Logging

```rust
use log::{info, warn, error, debug};

impl MyContract {
    pub fn transfer(&mut self, to: String, amount: u64) -> Result<(), JsValue> {
        debug!("Transferring {} from {} to {}", amount, self.state.owner, to);
        
        if amount > self.state.balance {
            warn!("Insufficient balance for transfer");
            return Err("Insufficient balance".into());
        }
        
        self.state.balance -= amount;
        info!("Transfer completed: {} -> {} (amount: {})", self.state.owner, to, amount);
        
        Ok(())
    }
}
```

### Debugging Tools

```bash
# Enable debug logging
RUST_LOG=debug cargo test

# Use wasm-pack with debug symbols
wasm-pack build --target web --debug

# Use browser dev tools for debugging
# Open browser console to see logs
```

## Performance Optimization

### 1. Memory Management

```rust
// Use efficient data structures
use std::collections::HashMap;

// Avoid unnecessary allocations
pub fn efficient_operation(&self) -> Vec<u8> {
    let mut result = Vec::with_capacity(1024); // Pre-allocate
    // ... operations
    result
}
```

### 2. Caching

```rust
pub struct CachedContract {
    state: ContractState,
    cache: HashMap<String, JsValue>,
}

impl CachedContract {
    pub fn get_cached_value(&mut self, key: &str) -> Option<JsValue> {
        if let Some(cached) = self.cache.get(key) {
            return Some(cached.clone());
        }
        
        // Compute value
        let value = self.compute_value(key);
        self.cache.insert(key.to_string(), value.clone());
        Some(value)
    }
}
```

## Conclusion

This guide provides a comprehensive overview of smart contract development on the Gillean blockchain platform. By following these patterns and best practices, you can create secure, efficient, and maintainable smart contracts.

Key takeaways:
- Use WebAssembly for maximum performance and security
- Implement proper access control and error handling
- Write comprehensive tests
- Optimize for gas efficiency
- Follow security best practices
- Use the provided patterns and templates

For more information, see the [API Reference](api.md) and [Architecture Overview](architecture.md).
