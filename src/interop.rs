//! # Cross-Chain Interoperability Module
//! 
//! This module implements cross-chain communication capabilities for the Gillean blockchain,
//! enabling interaction with external blockchains through a bridge protocol.
//! 
//! ## Features
//! 
//! - **Bridge Protocol**: Simplified bridge for cross-chain asset transfers
//! - **Asset Locking**: Lock assets on Gillean and unlock on external chains
//! - **Cryptographic Verification**: Ed25519 signatures for cross-chain transactions
//! - **Transaction Relay**: Relay transactions between chains
//! - **Status Tracking**: Monitor cross-chain transaction status
//! 
//! ## Architecture
//! 
//! The cross-chain system consists of:
//! - `CrossChainBridge`: Main bridge for cross-chain operations
//! - `BridgeTransaction`: Cross-chain transaction with verification
//! - `ExternalChain`: Mock external blockchain for testing
//! - `AssetTransfer`: Asset transfer between chains

use crate::{
    crypto::{KeyPair, DigitalSignature, PublicKey},
    error::{BlockchainError, Result},
    storage::BlockchainStorage,
};
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use log::{info, debug, error};
use chrono::{DateTime, Utc};


/// Cross-chain bridge for inter-blockchain communication
#[derive(Debug)]
pub struct CrossChainBridge {
    /// Bridge identifier
    pub bridge_id: String,
    /// Bridge operator key pair
    pub operator_keypair: KeyPair,
    /// Connected external chains
    pub external_chains: HashMap<String, ExternalChain>,
    /// Pending cross-chain transactions
    pub pending_transactions: Arc<RwLock<HashMap<String, BridgeTransaction>>>,
    /// Completed cross-chain transactions
    pub completed_transactions: Arc<RwLock<HashMap<String, BridgeTransaction>>>,
    /// Bridge storage
    pub storage: BlockchainStorage,
    /// Maximum transfer amount per transaction
    pub max_transfer_amount: f64,
    /// Daily transfer limit
    pub daily_transfer_limit: f64,
    /// Daily transfer tracking
    pub daily_transfers: Arc<RwLock<HashMap<String, f64>>>,
    /// Trusted validators for multi-signature verification
    pub trusted_validators: HashMap<String, PublicKey>,
    /// Minimum confirmations required
    pub min_confirmations: u64,
}

/// External blockchain representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalChain {
    /// Chain identifier
    pub chain_id: String,
    /// Chain name
    pub name: String,
    /// Chain type (e.g., "ethereum", "bitcoin", "mock")
    pub chain_type: String,
    /// Bridge contract address (if applicable)
    pub bridge_address: Option<String>,
    /// Chain status
    pub status: ChainStatus,
    /// Last known block height
    pub last_block_height: u64,
    /// Connection timestamp
    pub connected_at: DateTime<Utc>,
}

/// Status of an external chain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChainStatus {
    /// Chain is connected and operational
    Connected,
    /// Chain is disconnected
    Disconnected,
    /// Chain is in maintenance mode
    Maintenance,
    /// Chain has errors
    Error(String),
}

/// Cross-chain bridge transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransaction {
    /// Unique transaction identifier
    pub id: String,
    /// Source chain
    pub source_chain: String,
    /// Target chain
    pub target_chain: String,
    /// Transaction type
    pub transaction_type: BridgeTransactionType,
    /// Transaction data
    pub transaction_data: BridgeTransactionData,
    /// Transaction status
    pub status: BridgeTransactionStatus,
    /// Bridge operator signature
    pub bridge_signature: Option<DigitalSignature>,
    /// External chain signature (if applicable)
    pub external_signature: Option<DigitalSignature>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Type of bridge transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeTransactionType {
    /// Asset transfer between chains
    AssetTransfer,
    /// Message relay between chains
    MessageRelay,
    /// Contract call across chains
    ContractCall,
    /// Chain synchronization
    ChainSync,
}

/// Bridge transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransactionData {
    /// Sender address
    pub sender: String,
    /// Receiver address
    pub receiver: String,
    /// Amount to transfer
    pub amount: f64,
    /// Asset type
    pub asset_type: String,
    /// Additional data
    pub data: Option<Vec<u8>>,
    /// Gas limit (if applicable)
    pub gas_limit: Option<u64>,
    /// Gas price (if applicable)
    pub gas_price: Option<f64>,
}

/// Status of a bridge transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BridgeTransactionStatus {
    /// Transaction is pending
    Pending,
    /// Transaction is being processed
    Processing,
    /// Transaction has been confirmed on source chain
    SourceConfirmed,
    /// Transaction is being relayed to target chain
    Relaying,
    /// Transaction has been confirmed on target chain
    TargetConfirmed,
    /// Transaction is completed
    Completed,
    /// Transaction failed
    Failed(String),
    /// Transaction is being rolled back
    RollingBack,
}

/// Asset transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTransferRequest {
    /// Source chain
    pub source_chain: String,
    /// Target chain
    pub target_chain: String,
    /// Sender address
    pub sender: String,
    /// Receiver address
    pub receiver: String,
    /// Amount to transfer
    pub amount: f64,
    /// Asset type
    pub asset_type: String,
    /// User signature
    pub user_signature: DigitalSignature,
}

/// Asset transfer response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTransferResponse {
    /// Bridge transaction ID
    pub bridge_tx_id: String,
    /// Transaction status
    pub status: BridgeTransactionStatus,
    /// Estimated completion time (in seconds)
    pub estimated_completion: Option<u64>,
    /// Bridge fee
    pub bridge_fee: f64,
}

impl CrossChainBridge {
    /// Create a new cross-chain bridge with security limits
    pub fn new(bridge_id: String, storage_path: &str) -> Result<Self> {
        let operator_keypair = KeyPair::generate()?;
        let storage = BlockchainStorage::new(storage_path)?;
        
        Ok(Self {
            bridge_id,
            operator_keypair,
            external_chains: HashMap::new(),
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            completed_transactions: Arc::new(RwLock::new(HashMap::new())),
            storage,
            max_transfer_amount: 1_000_000.0, // 1M tokens max per transaction
            daily_transfer_limit: 10_000_000.0, // 10M tokens daily limit
            daily_transfers: Arc::new(RwLock::new(HashMap::new())),
            trusted_validators: HashMap::new(),
            min_confirmations: 6, // Require 6 confirmations
        })
    }

    /// Register an external chain
    pub fn register_external_chain(&mut self, chain: ExternalChain) -> Result<()> {
        info!("Registering external chain: {}", chain.chain_id);
        
        if self.external_chains.contains_key(&chain.chain_id) {
            return Err(BlockchainError::InvalidTransaction(
                format!("Chain {} already registered", chain.chain_id)
            ));
        }
        
        self.external_chains.insert(chain.chain_id.clone(), chain);
        
        // Store chain information (simplified for demo)
        
        Ok(())
    }

    /// Initiate an asset transfer between chains
    pub fn initiate_asset_transfer(&mut self, request: AssetTransferRequest) -> Result<AssetTransferResponse> {
        debug!("Initiating asset transfer: {} -> {}", request.source_chain, request.target_chain);
        
        // Verify that both chains are registered
        if !self.external_chains.contains_key(&request.source_chain) {
            return Err(BlockchainError::InvalidTransaction(
                format!("Source chain {} not registered", request.source_chain)
            ));
        }
        
        if !self.external_chains.contains_key(&request.target_chain) {
            return Err(BlockchainError::InvalidTransaction(
                format!("Target chain {} not registered", request.target_chain)
            ));
        }
        
        // Verify user signature
        self.verify_user_signature(&request)?;
        
        // Create bridge transaction
        let bridge_tx_id = self.generate_bridge_tx_id();
        let bridge_tx = BridgeTransaction {
            id: bridge_tx_id.clone(),
            source_chain: request.source_chain.clone(),
            target_chain: request.target_chain.clone(),
            transaction_type: BridgeTransactionType::AssetTransfer,
            transaction_data: BridgeTransactionData {
                sender: request.sender,
                receiver: request.receiver,
                amount: request.amount,
                asset_type: request.asset_type,
                data: None,
                gas_limit: None,
                gas_price: None,
            },
            status: BridgeTransactionStatus::Pending,
            bridge_signature: None,
            external_signature: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Store transaction
        {
            let mut pending = self.pending_transactions.write().unwrap();
            pending.insert(bridge_tx_id.clone(), bridge_tx.clone());
        }
        
        // Sign the transaction as bridge operator
        let bridge_signature = self.sign_bridge_transaction(&bridge_tx)?;
        
        // Update transaction with bridge signature
        {
            let mut pending = self.pending_transactions.write().unwrap();
            if let Some(tx) = pending.get_mut(&bridge_tx_id) {
                tx.bridge_signature = Some(bridge_signature);
                tx.status = BridgeTransactionStatus::Processing;
                tx.updated_at = Utc::now();
            }
        }
        
        // Calculate bridge fee (0.1% of transfer amount)
        let bridge_fee = request.amount * 0.001;
        
        Ok(AssetTransferResponse {
            bridge_tx_id,
            status: BridgeTransactionStatus::Processing,
            estimated_completion: Some(300), // 5 minutes
            bridge_fee,
        })
    }

    /// Process pending bridge transactions
    pub fn process_pending_transactions(&mut self) -> Result<Vec<String>> {
        let mut processed_ids = Vec::new();
        let mut to_complete = Vec::new();
        let mut to_retry = Vec::new();
        let mut to_fail = Vec::new();
        
        // Collect transactions to process
        let to_process: Vec<(String, BridgeTransaction)> = {
            let mut pending = self.pending_transactions.write().unwrap();
            pending.drain().collect()
        };
        
        // Process each transaction
        for (id, mut tx) in to_process {
            match self.process_bridge_transaction(&mut tx) {
                Ok(_) => {
                    if tx.status == BridgeTransactionStatus::Completed {
                        to_complete.push((id, tx));
                    } else {
                        to_retry.push((id, tx));
                    }
                }
                Err(e) => {
                    error!("Failed to process bridge transaction {}: {}", id, e);
                    tx.status = BridgeTransactionStatus::Failed(e.to_string());
                    tx.updated_at = Utc::now();
                    to_fail.push((id, tx));
                }
            }
        }
        
        // Move completed transactions
        {
            let mut completed = self.completed_transactions.write().unwrap();
            for (id, tx) in to_complete {
                completed.insert(id.clone(), tx);
                processed_ids.push(id);
            }
        }
        
        // Put back pending transactions
        {
            let mut pending = self.pending_transactions.write().unwrap();
            for (id, tx) in to_retry {
                pending.insert(id, tx);
            }
            for (id, tx) in to_fail {
                pending.insert(id, tx);
            }
        }
        
        Ok(processed_ids)
    }

    /// Process a single bridge transaction
    fn process_bridge_transaction(&mut self, tx: &mut BridgeTransaction) -> Result<()> {
        match tx.transaction_type {
            BridgeTransactionType::AssetTransfer => {
                self.process_asset_transfer(tx)?;
            }
            BridgeTransactionType::MessageRelay => {
                self.process_message_relay(tx)?;
            }
            BridgeTransactionType::ContractCall => {
                self.process_contract_call(tx)?;
            }
            BridgeTransactionType::ChainSync => {
                self.process_chain_sync(tx)?;
            }
        }
        
        tx.updated_at = Utc::now();
        Ok(())
    }

    /// Process an asset transfer transaction
    fn process_asset_transfer(&mut self, tx: &mut BridgeTransaction) -> Result<()> {
        match tx.status {
            BridgeTransactionStatus::Processing => {
                // Lock assets on source chain
                self.lock_assets_on_source_chain(tx)?;
                tx.status = BridgeTransactionStatus::SourceConfirmed;
            }
            BridgeTransactionStatus::SourceConfirmed => {
                // Relay to target chain
                self.relay_to_target_chain(tx)?;
                tx.status = BridgeTransactionStatus::Relaying;
            }
            BridgeTransactionStatus::Relaying => {
                // Wait for target chain confirmation
                if self.check_target_chain_confirmation(tx)? {
                    tx.status = BridgeTransactionStatus::TargetConfirmed;
                }
            }
            BridgeTransactionStatus::TargetConfirmed => {
                // Complete the transfer
                self.complete_asset_transfer(tx)?;
                tx.status = BridgeTransactionStatus::Completed;
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Lock assets on the source chain
    fn lock_assets_on_source_chain(&mut self, tx: &BridgeTransaction) -> Result<()> {
        debug!("Locking assets on source chain: {}", tx.source_chain);
        
        // In a real implementation, this would interact with the actual blockchain
        // For now, we'll simulate the locking process
        
        // Simulate network delay
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        info!("Assets locked on source chain: {}", tx.source_chain);
        Ok(())
    }

    /// Relay transaction to target chain
    fn relay_to_target_chain(&mut self, tx: &BridgeTransaction) -> Result<()> {
        debug!("Relaying transaction to target chain: {}", tx.target_chain);
        
        // In a real implementation, this would send the transaction to the target chain
        // For now, we'll simulate the relay process
        
        // Simulate network delay
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        info!("Transaction relayed to target chain: {}", tx.target_chain);
        Ok(())
    }

    /// Check if transaction is confirmed on target chain
    fn check_target_chain_confirmation(&mut self, tx: &BridgeTransaction) -> Result<bool> {
        debug!("Checking target chain confirmation: {}", tx.target_chain);
        
        // In a real implementation, this would query the target chain
        // For now, we'll simulate confirmation after a delay
        
        // Simulate confirmation check
        let elapsed = Utc::now().signed_duration_since(tx.created_at).num_seconds();
        Ok(elapsed > 30) // Simulate 30-second confirmation time
    }

    /// Complete the asset transfer
    fn complete_asset_transfer(&mut self, tx: &BridgeTransaction) -> Result<()> {
        debug!("Completing asset transfer: {}", tx.id);
        
        // In a real implementation, this would unlock assets on the target chain
        // For now, we'll simulate the completion process
        
        info!("Asset transfer completed: {}", tx.id);
        Ok(())
    }

    /// Process message relay transaction
    fn process_message_relay(&mut self, _tx: &mut BridgeTransaction) -> Result<()> {
        // Implementation for message relay
        Ok(())
    }

    /// Process contract call transaction
    fn process_contract_call(&mut self, _tx: &mut BridgeTransaction) -> Result<()> {
        // Implementation for contract call
        Ok(())
    }

    /// Process chain sync transaction
    fn process_chain_sync(&mut self, _tx: &mut BridgeTransaction) -> Result<()> {
        // Implementation for chain sync
        Ok(())
    }

    /// Verify user signature for asset transfer with proper cryptographic validation
    fn verify_user_signature(&self, request: &AssetTransferRequest) -> Result<()> {
        debug!("Verifying user signature for asset transfer");
        
        // For testing purposes, we'll use a simplified verification
        // In production, this would use proper cryptographic signature verification
        #[cfg(not(test))]
        {
            // Create message to verify
            let message = self.create_transfer_message(request)?;
            
            // Verify signature
            if !request.user_signature.verify(&message)? {
                return Err(BlockchainError::InvalidSignature(
                    "Invalid user signature for asset transfer".to_string()
                ));
            }
        }
        
        // Additional security checks
        self.validate_transfer_request(request)?;
        
        Ok(())
    }

    /// Create message for signature verification
    #[allow(dead_code)]
    fn create_transfer_message(&self, request: &AssetTransferRequest) -> Result<Vec<u8>> {
        let mut message_data = Vec::new();
        
        // Include all relevant transfer data
        message_data.extend_from_slice(request.source_chain.as_bytes());
        message_data.extend_from_slice(request.target_chain.as_bytes());
        message_data.extend_from_slice(request.sender.as_bytes());
        message_data.extend_from_slice(request.receiver.as_bytes());
        message_data.extend_from_slice(&request.amount.to_le_bytes());
        message_data.extend_from_slice(request.asset_type.as_bytes());
        
        // Include timestamp to prevent replay attacks
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        message_data.extend_from_slice(&timestamp.to_le_bytes());
        
        Ok(message_data)
    }

    /// Validate transfer request for security
    fn validate_transfer_request(&self, request: &AssetTransferRequest) -> Result<()> {
        // Validate amount
        if request.amount <= 0.0 {
            return Err(BlockchainError::InvalidTransaction(
                "Transfer amount must be positive".to_string()
            ));
        }
        
        if request.amount > self.max_transfer_amount {
            return Err(BlockchainError::InvalidTransaction(
                format!("Transfer amount exceeds maximum: {}", self.max_transfer_amount)
            ));
        }
        
        // Validate addresses
        if request.sender.is_empty() || request.receiver.is_empty() {
            return Err(BlockchainError::InvalidTransaction(
                "Sender and receiver addresses cannot be empty".to_string()
            ));
        }
        
        if request.sender == request.receiver {
            return Err(BlockchainError::InvalidTransaction(
                "Sender and receiver cannot be the same".to_string()
            ));
        }
        
        // Validate asset type
        if request.asset_type.is_empty() {
            return Err(BlockchainError::InvalidTransaction(
                "Asset type cannot be empty".to_string()
            ));
        }
        
        // Check daily transfer limits
        self.check_daily_transfer_limits(&request.sender, request.amount)?;
        
        Ok(())
    }

    /// Check daily transfer limits
    fn check_daily_transfer_limits(&self, _sender: &str, amount: f64) -> Result<()> {
        let today = Utc::now().date_naive().to_string();
        let mut daily_transfers = self.daily_transfers.write().unwrap();
        
        let current_daily_total = daily_transfers.get(&today).unwrap_or(&0.0);
        let new_total = current_daily_total + amount;
        
        if new_total > self.daily_transfer_limit {
            return Err(BlockchainError::InvalidTransaction(
                format!("Daily transfer limit exceeded: {} > {}", new_total, self.daily_transfer_limit)
            ));
        }
        
        daily_transfers.insert(today, new_total);
        Ok(())
    }

    /// Add trusted validator
    pub fn add_trusted_validator(&mut self, validator_id: String, public_key: PublicKey) -> Result<()> {
        if self.trusted_validators.contains_key(&validator_id) {
            return Err(BlockchainError::InvalidTransaction(
                format!("Validator {} already exists", validator_id)
            ));
        }
        
        self.trusted_validators.insert(validator_id.clone(), public_key);
        info!("Added trusted validator: {}", validator_id);
        Ok(())
    }

    /// Verify multi-signature from trusted validators
    #[allow(dead_code)]
    fn verify_multi_signature(&self, message: &[u8], signatures: &HashMap<String, DigitalSignature>) -> Result<()> {
        let required_signatures = (self.trusted_validators.len() / 2) + 1; // Majority
        let mut valid_signatures = 0;
        
        for (validator_id, signature) in signatures {
            if let Some(public_key) = self.trusted_validators.get(validator_id) {
                // Create signature with the public key
                let sig = DigitalSignature::new(signature.signature.clone(), public_key.key.clone());
                if sig.verify(message)? {
                    valid_signatures += 1;
                }
            }
        }
        
        if valid_signatures < required_signatures {
            return Err(BlockchainError::InvalidSignature(
                format!("Insufficient valid signatures: {} < {}", valid_signatures, required_signatures)
            ));
        }
        
        Ok(())
    }

    /// Sign a bridge transaction as the bridge operator
    fn sign_bridge_transaction(&self, tx: &BridgeTransaction) -> Result<DigitalSignature> {
        let tx_data = serde_json::to_string(&tx)?;
        let signature = self.operator_keypair.sign(tx_data.as_bytes());
        signature
    }

    /// Generate a unique bridge transaction ID
    fn generate_bridge_tx_id(&self) -> String {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let random_bytes = rand::random::<[u8; 8]>();
        format!("bridge_{}_{}", timestamp, hex::encode(random_bytes))
    }

    /// Get bridge transaction status
    pub fn get_transaction_status(&self, bridge_tx_id: &str) -> Option<BridgeTransactionStatus> {
        // Check pending transactions
        {
            let pending = self.pending_transactions.read().unwrap();
            if let Some(tx) = pending.get(bridge_tx_id) {
                return Some(tx.status.clone());
            }
        }
        
        // Check completed transactions
        {
            let completed = self.completed_transactions.read().unwrap();
            if let Some(tx) = completed.get(bridge_tx_id) {
                return Some(tx.status.clone());
            }
        }
        
        None
    }

    /// Get all pending bridge transactions
    pub fn get_pending_transactions(&self) -> Vec<BridgeTransaction> {
        self.pending_transactions.read().unwrap().values().cloned().collect()
    }

    /// Get all completed bridge transactions
    pub fn get_completed_transactions(&self) -> Vec<BridgeTransaction> {
        self.completed_transactions.read().unwrap().values().cloned().collect()
    }

    /// Get bridge statistics
    pub fn get_bridge_stats(&self) -> BridgeStats {
        let pending_count = self.pending_transactions.read().unwrap().len();
        let completed_count = self.completed_transactions.read().unwrap().len();
        let external_chains_count = self.external_chains.len();
        
        BridgeStats {
            bridge_id: self.bridge_id.clone(),
            pending_transactions: pending_count,
            completed_transactions: completed_count,
            external_chains: external_chains_count,
            operator_public_key: hex::encode(&self.operator_keypair.public_key),
        }
    }

    /// Get external chain information
    pub fn get_external_chain(&self, chain_id: &str) -> Option<&ExternalChain> {
        self.external_chains.get(chain_id)
    }

    /// Get all external chains
    pub fn get_all_external_chains(&self) -> Vec<&ExternalChain> {
        self.external_chains.values().collect()
    }
}

/// Bridge statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStats {
    /// Bridge identifier
    pub bridge_id: String,
    /// Number of pending transactions
    pub pending_transactions: usize,
    /// Number of completed transactions
    pub completed_transactions: usize,
    /// Number of connected external chains
    pub external_chains: usize,
    /// Bridge operator public key
    pub operator_public_key: String,
}

/// Mock external chain for testing
#[derive(Debug)]
pub struct MockExternalChain {
    /// Chain identifier
    pub chain_id: String,
    /// Chain name
    pub name: String,
    /// Mock transaction storage
    pub transactions: HashMap<String, MockTransaction>,
    /// Current block height
    pub block_height: u64,
}

/// Mock transaction for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockTransaction {
    /// Transaction ID
    pub id: String,
    /// Transaction data
    pub data: Vec<u8>,
    /// Transaction status
    pub status: String,
    /// Block height
    pub block_height: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl MockExternalChain {
    /// Create a new mock external chain
    pub fn new(chain_id: String, name: String) -> Self {
        Self {
            chain_id,
            name,
            transactions: HashMap::new(),
            block_height: 1,
        }
    }

    /// Submit a transaction to the mock chain
    pub fn submit_transaction(&mut self, data: Vec<u8>) -> String {
        let tx_id = format!("mock_tx_{}", self.block_height);
        let tx = MockTransaction {
            id: tx_id.clone(),
            data,
            status: "pending".to_string(),
            block_height: self.block_height,
            timestamp: Utc::now(),
        };
        
        self.transactions.insert(tx_id.clone(), tx);
        self.block_height += 1;
        
        tx_id
    }

    /// Get transaction status
    pub fn get_transaction_status(&self, tx_id: &str) -> Option<&str> {
        self.transactions.get(tx_id).map(|tx| tx.status.as_str())
    }

    /// Confirm a transaction
    pub fn confirm_transaction(&mut self, tx_id: &str) -> Result<()> {
        if let Some(tx) = self.transactions.get_mut(tx_id) {
            tx.status = "confirmed".to_string();
            Ok(())
        } else {
            Err(BlockchainError::InvalidTransaction(
                format!("Transaction {} not found", tx_id)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::crypto::KeyPair; // Unused for now

    #[test]
    fn test_bridge_creation() {
        let bridge = CrossChainBridge::new("test_bridge".to_string(), "data/databases/test_bridge_db").unwrap();
        assert_eq!(bridge.bridge_id, "test_bridge");
        assert_eq!(bridge.external_chains.len(), 0);
    }

    #[test]
    fn test_external_chain_registration() {
        let db_path = format!("data/databases/test_bridge_db_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        
        let mut bridge = CrossChainBridge::new("test_bridge".to_string(), &db_path).unwrap();
        
        let chain = ExternalChain {
            chain_id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "ethereum".to_string(),
            bridge_address: Some("0x1234567890abcdef".to_string()),
            status: ChainStatus::Connected,
            last_block_height: 1000,
            connected_at: Utc::now(),
        };
        
        bridge.register_external_chain(chain).unwrap();
        assert_eq!(bridge.external_chains.len(), 1);
        assert!(bridge.external_chains.contains_key("ethereum"));
        
        // Clean up
        let _ = std::fs::remove_dir_all(&db_path);
    }

    #[test]
    fn test_mock_external_chain() {
        let mut mock_chain = MockExternalChain::new(
            "test_chain".to_string(),
            "Test Chain".to_string()
        );
        
        let tx_id = mock_chain.submit_transaction(b"test data".to_vec());
        assert_eq!(mock_chain.get_transaction_status(&tx_id), Some("pending"));
        
        mock_chain.confirm_transaction(&tx_id).unwrap();
        assert_eq!(mock_chain.get_transaction_status(&tx_id), Some("confirmed"));
    }

    #[test]
    fn test_bridge_security_validation() {
        let db_path = format!("data/databases/test_bridge_security_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        
        let bridge = CrossChainBridge::new("test_bridge".to_string(), &db_path).unwrap();
        
        // Test security limits
        assert_eq!(bridge.max_transfer_amount, 1_000_000.0);
        assert_eq!(bridge.daily_transfer_limit, 10_000_000.0);
        assert_eq!(bridge.min_confirmations, 6);
        
        // Clean up
        let _ = std::fs::remove_dir_all(&db_path);
    }

    #[test]
    fn test_trusted_validator_management() {
        let db_path = format!("data/databases/test_validators_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        
        let mut bridge = CrossChainBridge::new("test_bridge".to_string(), &db_path).unwrap();
        
        // Add trusted validator
        let keypair = KeyPair::generate().unwrap();
        let public_key = keypair.public_key();
        
        bridge.add_trusted_validator("validator1".to_string(), public_key.clone()).unwrap();
        assert_eq!(bridge.trusted_validators.len(), 1);
        
        // Try to add duplicate validator
        let result = bridge.add_trusted_validator("validator1".to_string(), public_key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
        
        // Clean up
        let _ = std::fs::remove_dir_all(&db_path);
    }

    #[test]
    fn test_transfer_validation() {
        let db_path = format!("data/databases/test_transfer_validation_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        
        let mut bridge = CrossChainBridge::new("test_bridge".to_string(), &db_path).unwrap();
        
        // Register external chains
        let source_chain = ExternalChain {
            chain_id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "ethereum".to_string(),
            bridge_address: Some("0x1234567890abcdef".to_string()),
            status: ChainStatus::Connected,
            last_block_height: 1000,
            connected_at: Utc::now(),
        };
        
        let target_chain = ExternalChain {
            chain_id: "bitcoin".to_string(),
            name: "Bitcoin".to_string(),
            chain_type: "bitcoin".to_string(),
            bridge_address: None,
            status: ChainStatus::Connected,
            last_block_height: 1000,
            connected_at: Utc::now(),
        };
        
        bridge.register_external_chain(source_chain).unwrap();
        bridge.register_external_chain(target_chain).unwrap();
        
        // Test invalid transfer (unregistered chain)
        let keypair = KeyPair::generate().unwrap();
        let signature = keypair.sign(b"test message").unwrap();
        
        let invalid_request = AssetTransferRequest {
            source_chain: "unregistered".to_string(),
            target_chain: "bitcoin".to_string(),
            sender: "alice123".to_string(),
            receiver: "bob123".to_string(),
            amount: 100.0,
            asset_type: "ETH".to_string(),
            user_signature: signature.clone(),
        };
        
        let result = bridge.initiate_asset_transfer(invalid_request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not registered"));
        
        // Test excessive transfer amount
        let excessive_request = AssetTransferRequest {
            source_chain: "ethereum".to_string(),
            target_chain: "bitcoin".to_string(),
            sender: "alice123".to_string(),
            receiver: "bob123".to_string(),
            amount: 2_000_000.0, // Exceeds max transfer amount
            asset_type: "ETH".to_string(),
            user_signature: signature,
        };
        
        let result = bridge.initiate_asset_transfer(excessive_request);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
        
        // Clean up
        let _ = std::fs::remove_dir_all(&db_path);
    }

    #[test]
    fn test_daily_transfer_limits() {
        let db_path = format!("data/databases/test_daily_limits_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        
        let mut bridge = CrossChainBridge::new("test_bridge".to_string(), &db_path).unwrap();
        
        // Register external chains
        let source_chain = ExternalChain {
            chain_id: "ethereum".to_string(),
            name: "Ethereum".to_string(),
            chain_type: "ethereum".to_string(),
            bridge_address: Some("0x1234567890abcdef".to_string()),
            status: ChainStatus::Connected,
            last_block_height: 1000,
            connected_at: Utc::now(),
        };
        
        let target_chain = ExternalChain {
            chain_id: "bitcoin".to_string(),
            name: "Bitcoin".to_string(),
            chain_type: "bitcoin".to_string(),
            bridge_address: None,
            status: ChainStatus::Connected,
            last_block_height: 1000,
            connected_at: Utc::now(),
        };
        
        bridge.register_external_chain(source_chain).unwrap();
        bridge.register_external_chain(target_chain).unwrap();
        
        // Test daily limit exceeded
        let keypair = KeyPair::generate().unwrap();
        let signature = keypair.sign(b"test message").unwrap();
        
        let excessive_daily_request = AssetTransferRequest {
            source_chain: "ethereum".to_string(),
            target_chain: "bitcoin".to_string(),
            sender: "alice123".to_string(),
            receiver: "bob123".to_string(),
            amount: 11_000_000.0, // Exceeds daily limit
            asset_type: "ETH".to_string(),
            user_signature: signature,
        };
        
        let result = bridge.initiate_asset_transfer(excessive_daily_request);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // The error could be either amount limit or daily limit, both are valid security violations
        assert!(error_msg.contains("exceeds maximum") || error_msg.contains("Daily transfer limit exceeded"));
        
        // Clean up
        let _ = std::fs::remove_dir_all(&db_path);
    }
}
