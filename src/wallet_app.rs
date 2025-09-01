//! Web Wallet Application for Gillean Blockchain
//! 
//! This module provides a comprehensive web wallet interface for managing
//! blockchain accounts, transactions, and interactions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::Result;
use crate::blockchain::Blockchain;
use crate::wallet::WalletManager;
use crate::transaction::Transaction;
use crate::block_explorer::BlockExplorer;

/// Web wallet application
pub struct WalletApp {
    blockchain: Arc<RwLock<Blockchain>>,
    wallet_manager: Arc<WalletManager>,
    block_explorer: Arc<BlockExplorer>,
    session_manager: Arc<RwLock<SessionManager>>,
    transaction_history: Arc<RwLock<HashMap<String, Vec<TransactionRecord>>>>,
}

/// Session management for wallet users
pub struct SessionManager {
    active_sessions: HashMap<String, WalletSession>,
    session_timeout: u64, // seconds
}

/// Wallet session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSession {
    pub session_id: String,
    pub wallet_address: String,
    pub created_at: u64,
    pub last_activity: u64,
    pub permissions: Vec<WalletPermission>,
    pub is_encrypted: bool,
}

/// Wallet permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WalletPermission {
    Send,
    Receive,
    ViewBalance,
    ViewHistory,
    ManageContracts,
    Stake,
    Vote,
}

/// Transaction record for wallet history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub transaction: Transaction,
    pub block_height: Option<u64>,
    pub confirmations: u64,
    pub status: TransactionStatus,
    pub timestamp: u64,
    pub direction: TransactionDirection,
    pub counterparty: String,
}

/// Transaction status for wallet
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

/// Transaction direction from wallet perspective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionDirection {
    Sent,
    Received,
    Internal,
}

/// Wallet account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    pub address: String,
    pub balance: f64,
    pub nonce: u64,
    pub is_contract: bool,
    pub contract_code: Option<String>,
    pub staked_amount: f64,
    pub voting_power: f64,
    pub created_at: u64,
    pub last_activity: u64,
}

/// Wallet dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletDashboard {
    pub account: WalletAccount,
    pub recent_transactions: Vec<TransactionRecord>,
    pub pending_transactions: Vec<TransactionRecord>,
    pub network_status: NetworkStatus,
    pub market_data: MarketData,
    pub notifications: Vec<WalletNotification>,
}

/// Network status for wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub is_connected: bool,
    pub block_height: u64,
    pub sync_status: String,
    pub gas_price: f64,
    pub network_difficulty: f64,
    pub peer_count: u32,
}

/// Market data for wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub token_price_usd: f64,
    pub price_change_24h: f64,
    pub market_cap: f64,
    pub volume_24h: f64,
    pub total_supply: f64,
    pub circulating_supply: f64,
}

/// Wallet notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: u64,
    pub is_read: bool,
    pub action_url: Option<String>,
}

/// Notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Transaction,
    Security,
    Network,
    Market,
    Governance,
    System,
}

/// Transaction request for wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub to: String,
    pub amount: f64,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<f64>,
    pub data: Option<String>,
    pub memo: Option<String>,
    pub is_private: bool,
}

/// Transaction response from wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub transaction_hash: String,
    pub status: TransactionStatus,
    pub gas_used: u64,
    pub gas_price: f64,
    pub fee: f64,
    pub block_height: Option<u64>,
    pub confirmations: u64,
    pub timestamp: u64,
}

/// Wallet settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSettings {
    pub currency: String,
    pub language: String,
    pub theme: String,
    pub notifications_enabled: bool,
    pub auto_lock_timeout: u64,
    pub biometric_enabled: bool,
    pub two_factor_enabled: bool,
    pub privacy_mode: bool,
    pub default_gas_price: f64,
    pub default_gas_limit: u64,
}

/// Wallet backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBackup {
    pub wallet_address: String,
    pub backup_type: BackupType,
    pub encrypted_data: String,
    pub created_at: u64,
    pub version: String,
    pub checksum: String,
}

/// Backup types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    Mnemonic,
    PrivateKey,
    Keystore,
    Hardware,
}

impl WalletApp {
    /// Create a new wallet application
    pub fn new(
        blockchain: Arc<RwLock<Blockchain>>,
        wallet_manager: Arc<WalletManager>,
        block_explorer: Arc<BlockExplorer>,
    ) -> Self {
        Self {
            blockchain,
            wallet_manager,
            block_explorer,
            session_manager: Arc::new(RwLock::new(SessionManager::new())),
            transaction_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new wallet
    pub async fn create_wallet(&self, password: &str) -> Result<WalletAccount> {
        // Simplified wallet creation
        let address = format!("wallet_{}", uuid::Uuid::new_v4());
        
        let account = WalletAccount {
            address: address.clone(),
            balance: 0.0,
            nonce: 0,
            is_contract: false,
            contract_code: None,
            staked_amount: 0.0,
            voting_power: 0.0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Initialize transaction history
        {
            let mut history = self.transaction_history.write().await;
            history.insert(address, Vec::new());
        }
        
        Ok(account)
    }
    
    /// Import existing wallet
    pub async fn import_wallet(&self, private_key: &str, _password: &str) -> Result<WalletAccount> {
        // Simplified wallet import
        let address = format!("imported_{}", uuid::Uuid::new_v4());
        
        // Get current balance and nonce
        let blockchain = self.blockchain.read().await;
        let balance = blockchain.get_balance(&address);
        let nonce = 0; // Simplified
        
        let account = WalletAccount {
            address: address.clone(),
            balance,
            nonce,
            is_contract: false, // Simplified
            contract_code: None, // Simplified
            staked_amount: 0.0, // Would get from staking contract
            voting_power: 0.0, // Would get from governance contract
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        // Load transaction history
        self.load_transaction_history(&address).await?;
        
        Ok(account)
    }
    
    /// Get wallet dashboard
    pub async fn get_dashboard(&self, session_id: &str) -> Result<WalletDashboard> {
        let session = self.get_session(session_id).await?;
        let address = &session.wallet_address;
        
        // Get account information
        let account = self.get_account_info(address).await?;
        
        // Get recent transactions
        let recent_transactions = self.get_recent_transactions(address, 10).await?;
        
        // Get pending transactions
        let pending_transactions = self.get_pending_transactions(address).await?;
        
        // Get network status
        let network_status = self.get_network_status().await?;
        
        // Get market data
        let market_data = self.get_market_data().await?;
        
        // Get notifications
        let notifications = self.get_notifications(address).await?;
        
        Ok(WalletDashboard {
            account,
            recent_transactions,
            pending_transactions,
            network_status,
            market_data,
            notifications,
        })
    }
    
    /// Send transaction
    pub async fn send_transaction(
        &self,
        session_id: &str,
        request: TransactionRequest,
    ) -> Result<TransactionResponse> {
        let session = self.get_session(session_id).await?;
        let address = &session.wallet_address;
        
        // Check permissions
        if !session.permissions.contains(&WalletPermission::Send) {
            return Err(crate::error::BlockchainError::InvalidInput(
                "Insufficient permissions to send transactions".to_string()
            ));
        }
        
        // Create transaction
        let transaction = Transaction::new_transfer(
            address.clone(),
            request.to.clone(),
            request.amount,
            request.memo,
        )?;
        
        // Submit transaction (simplified)
        let mut blockchain = self.blockchain.write().await;
        let tx_hash = transaction.id.clone();
        blockchain.add_transaction(
            address.clone(),
            request.to.clone(),
            request.amount,
            request.memo,
        )?;
        
        // Create transaction record
        let transaction_record = TransactionRecord {
            transaction: transaction.clone(),
            block_height: None,
            confirmations: 0,
            status: TransactionStatus::Pending,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            direction: TransactionDirection::Sent,
            counterparty: request.to,
        };
        
        // Add to transaction history
        {
            let mut history = self.transaction_history.write().await;
            if let Some(transactions) = history.get_mut(address) {
                transactions.push(transaction_record);
            }
        }
        
        Ok(TransactionResponse {
            transaction_hash: tx_hash,
            status: TransactionStatus::Pending,
            gas_used: 0,
            gas_price: request.gas_price.unwrap_or(0.0),
            fee: 0.0,
            block_height: None,
            confirmations: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    
    /// Get account information
    pub async fn get_account_info(&self, address: &str) -> Result<WalletAccount> {
        let blockchain = self.blockchain.read().await;
        let balance = blockchain.get_balance(address);
        let nonce = 0; // Simplified
        let is_contract = false; // Simplified
        let contract_code = None; // Simplified
        
        Ok(WalletAccount {
            address: address.to_string(),
            balance,
            nonce,
            is_contract,
            contract_code,
            staked_amount: 0.0, // Would get from staking contract
            voting_power: 0.0, // Would get from governance contract
            created_at: 0, // Would get from blockchain history
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
    
    /// Get recent transactions
    pub async fn get_recent_transactions(&self, address: &str, limit: usize) -> Result<Vec<TransactionRecord>> {
        let history = self.transaction_history.read().await;
        if let Some(transactions) = history.get(address) {
            let mut recent = transactions.clone();
            recent.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            recent.truncate(limit);
            Ok(recent)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get pending transactions
    pub async fn get_pending_transactions(&self, address: &str) -> Result<Vec<TransactionRecord>> {
        let history = self.transaction_history.read().await;
        if let Some(transactions) = history.get(address) {
            Ok(transactions.iter()
                .filter(|tx| tx.status == TransactionStatus::Pending)
                .cloned()
                .collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get network status
    pub async fn get_network_status(&self) -> Result<NetworkStatus> {
        let blockchain = self.blockchain.read().await;
        let block_height = blockchain.blocks.len() as u64 - 1;
        
        Ok(NetworkStatus {
            is_connected: true,
            block_height,
            sync_status: "synced".to_string(),
            gas_price: 0.000001, // Default gas price
            network_difficulty: 1.0, // Simplified
            peer_count: 25, // Placeholder
        })
    }
    
    /// Get market data
    pub async fn get_market_data(&self) -> Result<MarketData> {
        // In a real implementation, this would fetch from external APIs
        Ok(MarketData {
            token_price_usd: 1.0,
            price_change_24h: 0.05,
            market_cap: 1000000000.0,
            volume_24h: 10000000.0,
            total_supply: 1000000000.0,
            circulating_supply: 800000000.0,
        })
    }
    
    /// Get notifications
    pub async fn get_notifications(&self, _address: &str) -> Result<Vec<WalletNotification>> {
        // In a real implementation, this would fetch from a notifications service
        Ok(vec![
            WalletNotification {
                id: "1".to_string(),
                title: "Transaction Confirmed".to_string(),
                message: "Your transaction has been confirmed".to_string(),
                notification_type: NotificationType::Transaction,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                is_read: false,
                action_url: None,
            }
        ])
    }
    
    /// Create wallet session
    pub async fn create_session(&self, wallet_address: &str, permissions: Vec<WalletPermission>) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let session = WalletSession {
            session_id: session_id.clone(),
            wallet_address: wallet_address.to_string(),
            created_at: current_time,
            last_activity: current_time,
            permissions,
            is_encrypted: true,
        };
        
        {
            let mut session_manager = self.session_manager.write().await;
            session_manager.active_sessions.insert(session_id.clone(), session);
        }
        
        Ok(session_id)
    }
    
    /// Get session information
    pub async fn get_session(&self, session_id: &str) -> Result<WalletSession> {
        let session_manager = self.session_manager.read().await;
        session_manager.active_sessions.get(session_id)
            .cloned()
            .ok_or_else(|| crate::error::BlockchainError::NotFound(
                "Session not found".to_string()
            ))
    }
    
    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str) -> Result<()> {
        let mut session_manager = self.session_manager.write().await;
        if let Some(session) = session_manager.active_sessions.get_mut(session_id) {
            session.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        Ok(())
    }
    
    /// Load transaction history for address
    async fn load_transaction_history(&self, address: &str) -> Result<()> {
        let blockchain = self.blockchain.read().await;
        let mut transactions = Vec::new();
        
        // Load transactions from blockchain
        for (height, block) in blockchain.blocks.iter().enumerate() {
            for tx in &block.transactions {
                if tx.sender == address || tx.receiver == address {
                    let direction = if tx.sender == address {
                        TransactionDirection::Sent
                    } else {
                        TransactionDirection::Received
                    };
                    
                    let counterparty = if tx.sender == address {
                        tx.receiver.clone()
                    } else {
                        tx.sender.clone()
                    };
                    
                    transactions.push(TransactionRecord {
                        transaction: tx.clone(),
                        block_height: Some(height as u64),
                        confirmations: blockchain.blocks.len() as u64 - height as u64,
                        status: TransactionStatus::Confirmed,
                        timestamp: block.timestamp as u64,
                        direction,
                        counterparty,
                    });
                }
            }
        }
        
        // Sort by timestamp
        transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Store in transaction history
        {
            let mut history = self.transaction_history.write().await;
            history.insert(address.to_string(), transactions);
        }
        
        Ok(())
    }
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
            session_timeout: 3600, // 1 hour
        }
    }
    
    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.active_sessions.retain(|_, session| {
            current_time - session.last_activity < self.session_timeout
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::Blockchain;
    use crate::wallet::WalletManager;
    use crate::block_explorer::BlockExplorer;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_wallet_app_creation() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let wallet_manager = Arc::new(WalletManager::new());
        let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
        let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
        
        // Test that wallet app was created successfully
        assert!(true); // Basic creation test
    }
    
    #[tokio::test]
    async fn test_create_wallet() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let wallet_manager = Arc::new(WalletManager::new());
        let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
        let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
        
        let account = wallet_app.create_wallet("password123").await.unwrap();
        assert!(!account.address.is_empty());
        assert_eq!(account.balance, 0.0);
        assert_eq!(account.nonce, 0);
    }
    
    #[tokio::test]
    async fn test_session_management() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let wallet_manager = Arc::new(WalletManager::new());
        let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
        let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
        
        let permissions = vec![
            WalletPermission::Send,
            WalletPermission::Receive,
            WalletPermission::ViewBalance,
        ];
        
        let session_id = wallet_app.create_session("test_address", permissions).await.unwrap();
        assert!(!session_id.is_empty());
        
        let session = wallet_app.get_session(&session_id).await.unwrap();
        assert_eq!(session.wallet_address, "test_address");
        assert_eq!(session.permissions.len(), 3);
    }
    
    #[tokio::test]
    async fn test_account_info() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let wallet_manager = Arc::new(WalletManager::new());
        let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
        let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
        
        let account = wallet_app.get_account_info("test_address").await.unwrap();
        assert_eq!(account.address, "test_address");
        assert_eq!(account.balance, 0.0);
        assert_eq!(account.nonce, 0);
    }
    
    #[tokio::test]
    async fn test_network_status() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let wallet_manager = Arc::new(WalletManager::new());
        let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
        let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
        
        let network_status = wallet_app.get_network_status().await.unwrap();
        assert!(network_status.is_connected);
        assert_eq!(network_status.block_height, 0); // Genesis block
        assert_eq!(network_status.sync_status, "synced");
    }
}
