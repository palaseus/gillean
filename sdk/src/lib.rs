use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tokio::sync::mpsc;

pub mod client;
pub mod wallet;
pub mod contracts;
pub mod transactions;
pub mod analytics;

pub use client::GilleanClient;
pub use wallet::WalletManager;
pub use contracts::ContractManager;
pub use transactions::TransactionManager;
pub use analytics::AnalyticsClient;

/// Main SDK struct for interacting with Gillean blockchain
pub struct GilleanSDK {
    /// HTTP client for API communication
    client: GilleanClient,
    /// Wallet manager
    wallet_manager: WalletManager,
    /// Contract manager
    contract_manager: ContractManager,
    /// Transaction manager
    transaction_manager: TransactionManager,
    /// Analytics client
    analytics_client: AnalyticsClient,
}

/// SDK configuration
#[derive(Debug, Clone)]
pub struct SDKConfig {
    /// API server URL
    pub api_url: String,
    /// WebSocket URL
    pub ws_url: String,
    /// API key (optional)
    pub api_key: Option<String>,
    /// Timeout for requests
    pub timeout: std::time::Duration,
    /// Retry attempts
    pub retry_attempts: u32,
}

/// SDK error types
#[derive(Error, Debug)]
pub enum SDKError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Wallet error: {0}")]
    WalletError(String),
    #[error("Contract error: {0}")]
    ContractError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
}

/// Result type for SDK operations
pub type SDKResult<T> = Result<T, SDKError>;

impl GilleanSDK {
    /// Create a new SDK instance
    pub async fn new(config: SDKConfig) -> SDKResult<Self> {
        let client = GilleanClient::new(config.clone()).await?;
        let wallet_manager = WalletManager::new(config.clone());
        let contract_manager = ContractManager::new(config.clone());
        let transaction_manager = TransactionManager::new(config.clone());
        let analytics_client = AnalyticsClient::new(config.clone());

        Ok(Self {
            client,
            wallet_manager,
            contract_manager,
            transaction_manager,
            analytics_client,
        })
    }

    /// Get blockchain status
    pub async fn get_blockchain_status(&self) -> SDKResult<BlockchainStatus> {
        self.client.get_blockchain_status().await
    }

    /// Get wallet balance
    pub async fn get_balance(&self, address: &str) -> SDKResult<f64> {
        self.client.get_balance(address).await
    }

    /// Create a new wallet
    pub async fn create_wallet(&self, password: &str, name: Option<&str>) -> SDKResult<WalletInfo> {
        self.wallet_manager.create_wallet(password, name).await
    }

    /// Send a transaction
    pub async fn send_transaction(
        &self,
        from: &str,
        to: &str,
        amount: f64,
        password: &str,
        memo: Option<&str>,
    ) -> SDKResult<TransactionResult> {
        self.transaction_manager.send_transaction(from, to, amount, password, memo).await
    }

    /// Create a private transaction with ZKP
    pub async fn create_private_transaction(
        &self,
        from: &str,
        to: &str,
        amount: f64,
        password: &str,
        memo: Option<&str>,
    ) -> SDKResult<PrivateTransactionResult> {
        self.transaction_manager.create_private_transaction(from, to, amount, password, memo).await
    }

    /// Deploy a smart contract
    pub async fn deploy_contract(
        &self,
        contract_name: &str,
        contract_code: &[u8],
        sender: &str,
        password: &str,
        gas_limit: u64,
    ) -> SDKResult<ContractDeployResult> {
        self.contract_manager.deploy_contract(contract_name, contract_code, sender, password, gas_limit).await
    }

    /// Call a smart contract
    pub async fn call_contract(
        &self,
        contract_address: &str,
        method: &str,
        params: &[u8],
        sender: &str,
        password: &str,
        amount: Option<f64>,
    ) -> SDKResult<ContractCallResult> {
        self.contract_manager.call_contract(contract_address, method, params, sender, password, amount).await
    }

    /// Open a state channel
    pub async fn open_state_channel(
        &self,
        participant: &str,
        counterparty: &str,
        initial_balance: f64,
        timeout: u64,
        password: &str,
    ) -> SDKResult<StateChannelResult> {
        self.transaction_manager.open_state_channel(participant, counterparty, initial_balance, timeout, password).await
    }

    /// Update state channel
    pub async fn update_state_channel(
        &self,
        channel_id: &str,
        new_balance: HashMap<String, f64>,
        password: &str,
    ) -> SDKResult<StateChannelUpdateResult> {
        self.transaction_manager.update_state_channel(channel_id, new_balance, password).await
    }

    /// Close state channel
    pub async fn close_state_channel(
        &self,
        channel_id: &str,
        final_balance: HashMap<String, f64>,
        password: &str,
    ) -> SDKResult<StateChannelCloseResult> {
        self.transaction_manager.close_state_channel(channel_id, final_balance, password).await
    }

    /// Get analytics data
    pub async fn get_analytics(&self, metric_type: AnalyticsMetric) -> SDKResult<AnalyticsData> {
        self.analytics_client.get_analytics(metric_type).await
    }

    /// Subscribe to real-time updates
    pub async fn subscribe_to_updates(&self, event_types: Vec<EventType>) -> SDKResult<mpsc::Receiver<Event>> {
        self.client.subscribe_to_updates(event_types).await
    }

    /// Get SDK version
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

/// Blockchain status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStatus {
    pub total_blocks: usize,
    pub total_transactions: usize,
    pub pending_transactions: usize,
    pub current_difficulty: u32,
    pub consensus_type: String,
    pub is_synced: bool,
    pub uptime: u64,
}

/// Wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub name: Option<String>,
    pub balance: f64,
    pub created_at: i64,
}

/// Transaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub transaction_hash: String,
    pub status: TransactionStatus,
    pub block_number: Option<usize>,
    pub gas_used: Option<u64>,
    pub timestamp: i64,
}

/// Private transaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateTransactionResult {
    pub transaction_hash: String,
    pub zk_proof_id: String,
    pub status: TransactionStatus,
    pub timestamp: i64,
}

/// Contract deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeployResult {
    pub contract_address: String,
    pub transaction_hash: String,
    pub gas_used: u64,
    pub status: TransactionStatus,
    pub timestamp: i64,
}

/// Contract call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallResult {
    pub transaction_hash: String,
    pub return_value: Vec<u8>,
    pub gas_used: u64,
    pub status: TransactionStatus,
    pub timestamp: i64,
}

/// State channel result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChannelResult {
    pub channel_id: String,
    pub participants: Vec<String>,
    pub initial_balance: HashMap<String, f64>,
    pub status: ChannelStatus,
    pub created_at: i64,
}

/// State channel update result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChannelUpdateResult {
    pub channel_id: String,
    pub new_balance: HashMap<String, f64>,
    pub state_version: u64,
    pub timestamp: i64,
}

/// State channel close result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChannelCloseResult {
    pub channel_id: String,
    pub final_balance: HashMap<String, f64>,
    pub settlement_transaction: Option<String>,
    pub timestamp: i64,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

/// Channel status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChannelStatus {
    Open,
    Closing,
    Closed,
    Disputed,
}

/// Analytics metric types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalyticsMetric {
    TransactionVolume,
    ZKPProofGeneration,
    StateChannelActivity,
    ShardPerformance,
    CrossChainTransfers,
    ContractDeployments,
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub gas_used: u64,
    pub block_number: u64,
    pub timestamp: i64,
    pub status: TransactionStatus,
}

/// Block information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: i64,
    pub transactions: Vec<String>,
    pub gas_used: u64,
    pub gas_limit: u64,
}

/// Shard information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    pub shard_id: usize,
    pub status: String,
    pub transaction_count: u64,
    pub gas_used: u64,
    pub validators: Vec<String>,
}

/// Bridge status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatus {
    pub is_active: bool,
    pub total_transfers: u64,
    pub pending_transfers: u64,
    pub last_transfer_timestamp: i64,
}

/// Contract information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub address: String,
    pub name: String,
    pub bytecode: String,
    pub abi: String,
    pub creator: String,
    pub creation_timestamp: i64,
}

/// Metrics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    pub total_transactions: u64,
    pub total_blocks: u64,
    pub active_validators: u64,
    pub average_block_time: f64,
    pub gas_price: f64,
    pub network_hashrate: f64,
}

/// Analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsData {
    pub metric_type: AnalyticsMetric,
    pub data_points: Vec<DataPoint>,
    pub summary: AnalyticsSummary,
    pub timestamp: i64,
}

/// Data point for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: i64,
    pub value: f64,
    pub label: Option<String>,
}

/// Analytics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

/// Event types for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    NewBlock,
    NewTransaction,
    ZKPProofGenerated,
    StateChannelOpened,
    StateChannelUpdated,
    StateChannelClosed,
    ContractDeployed,
    ContractCalled,
}

/// Real-time event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub data: serde_json::Value,
    pub timestamp: i64,
}

impl Default for SDKConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:3000".to_string(),
            ws_url: "ws://localhost:3000/ws".to_string(),
            api_key: None,
            timeout: std::time::Duration::from_secs(30),
            retry_attempts: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sdk_creation() {
        let config = SDKConfig::default();
        let sdk = GilleanSDK::new(config).await;
        assert!(sdk.is_ok());
    }

    #[test]
    fn test_sdk_version() {
        let version = GilleanSDK::version();
        assert!(!version.is_empty());
    }
}
