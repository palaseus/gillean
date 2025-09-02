use crate::error::BlockchainError;
use crate::storage::BlockchainStorage;
use crate::wallet::WalletManager;
use ethers::{
    core::k256::ecdsa::SigningKey,
    providers::{Http, Provider},
    signers::LocalWallet,
    types::{Address, TransactionRequest, U256},
};
use ethers_middleware::Middleware;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Configuration for Ethereum testnet integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub bridge_contract_address: Option<String>,
}

impl Default for EthereumConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://sepolia.infura.io/v3/your-project-id".to_string(),
            chain_id: 11155111, // Sepolia testnet
            gas_limit: 21000,
            gas_price: 20000000000, // 20 gwei
            bridge_contract_address: None,
        }
    }
}

/// Ethereum bridge for cross-chain interactions
pub struct EthereumBridge {
    provider: Provider<Http>,
    config: EthereumConfig,
    storage: Arc<BlockchainStorage>,
    pending_transfers: Arc<RwLock<HashMap<String, PendingTransfer>>>,
}

/// Pending cross-chain transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransfer {
    pub id: String,
    pub from_gillean: String,
    pub to_ethereum: Address,
    pub amount: f64,
    pub status: TransferStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub ethereum_tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

impl EthereumBridge {
    /// Create a new Ethereum bridge instance
    pub async fn new(config: EthereumConfig, storage: Arc<BlockchainStorage>) -> Result<Self, BlockchainError> {
        let provider = Provider::<Http>::try_from(&config.rpc_url)
            .map_err(|e| BlockchainError::NetworkError(format!("Failed to connect to Ethereum: {}", e)))?;

        let bridge = Self {
            provider,
            config,
            storage,
            pending_transfers: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load pending transfers from storage
        bridge.load_pending_transfers().await?;

        Ok(bridge)
    }

    /// Initiate a transfer from Gillean to Ethereum
    pub async fn transfer_to_ethereum(
        &self,
        _from_wallet: &WalletManager,
        from_gillean_address: &str,
        to_ethereum_address: &str,
        amount: f64,
        _password: &str,
    ) -> Result<String, BlockchainError> {
        // Validate Ethereum address
        let to_address = to_ethereum_address
            .parse::<Address>()
            .map_err(|_| BlockchainError::ValidatorError("Invalid Ethereum address".to_string()))?;

        // Create transfer ID
        let transfer_id = uuid::Uuid::new_v4().to_string();

        // Create pending transfer record
        let pending_transfer = PendingTransfer {
            id: transfer_id.clone(),
            from_gillean: from_gillean_address.to_string(),
            to_ethereum: to_address,
            amount,
            status: TransferStatus::Pending,
            created_at: chrono::Utc::now(),
            ethereum_tx_hash: None,
        };

        // Store pending transfer
        {
            let mut transfers = self.pending_transfers.write().await;
            transfers.insert(transfer_id.clone(), pending_transfer.clone());
        }

        // Save to persistent storage
        self.save_pending_transfer(&pending_transfer).await?;

        // Start background processing
        let bridge_clone = self.clone_for_background();
        let transfer_id_clone = transfer_id.clone();
        tokio::spawn(async move {
            if let Err(e) = bridge_clone.process_transfer(&transfer_id_clone).await {
                error!("Failed to process transfer {}: {}", transfer_id_clone, e);
            }
        });

        info!("Initiated transfer to Ethereum: {} -> {} ({} GIL)", 
              from_gillean_address, to_ethereum_address, amount);

        Ok(transfer_id)
    }

    /// Process a pending transfer in the background
    async fn process_transfer(&self, transfer_id: &str) -> Result<(), BlockchainError> {
        // Update status to processing
        {
            let mut transfers = self.pending_transfers.write().await;
            if let Some(transfer) = transfers.get_mut(transfer_id) {
                transfer.status = TransferStatus::Processing;
            }
        }

        // Get transfer details
        let transfer = {
            let transfers = self.pending_transfers.read().await;
            transfers.get(transfer_id)
                .cloned()
                .ok_or_else(|| BlockchainError::NotFound("Transfer not found".to_string()))?
        };

        // For now, use a placeholder private key - in production, you'd load the actual wallet
        // and convert the private key properly
        let ethereum_private_key = SigningKey::random(&mut rand::thread_rng());
        let _ethereum_wallet = LocalWallet::from(ethereum_private_key);

        // Create transaction request
        let amount_wei = U256::from((transfer.amount * 1e18) as u128);
        let tx_request = TransactionRequest::new()
            .to(transfer.to_ethereum)
            .value(amount_wei)
            .gas(self.config.gas_limit)
            .gas_price(self.config.gas_price);

        // Send transaction
        match self.provider.send_transaction(tx_request, None).await {
            Ok(pending_tx) => {
                let tx_hash = pending_tx.tx_hash();
                
                // Update transfer with transaction hash
                {
                    let mut transfers = self.pending_transfers.write().await;
                    if let Some(transfer) = transfers.get_mut(transfer_id) {
                        transfer.ethereum_tx_hash = Some(format!("{:?}", tx_hash));
                        transfer.status = TransferStatus::Completed;
                    }
                }

                info!("Ethereum transaction sent: {:?}", tx_hash);
                self.save_pending_transfer(&transfer).await?;
            }
            Err(e) => {
                // Update transfer status to failed
                {
                    let mut transfers = self.pending_transfers.write().await;
                    if let Some(transfer) = transfers.get_mut(transfer_id) {
                        transfer.status = TransferStatus::Failed(e.to_string());
                    }
                }

                error!("Failed to send Ethereum transaction: {}", e);
                return Err(BlockchainError::NetworkError(format!("Ethereum transaction failed: {}", e)));
            }
        }

        Ok(())
    }

    /// Get Ethereum balance for an address
    pub async fn get_ethereum_balance(&self, address: &str) -> Result<f64, BlockchainError> {
        let eth_address = address
            .parse::<Address>()
            .map_err(|_| BlockchainError::ValidatorError("Invalid Ethereum address".to_string()))?;

        let balance = self.provider
            .get_balance(eth_address, None)
            .await
            .map_err(|e| BlockchainError::NetworkError(format!("Failed to get balance: {}", e)))?;

        let balance_eth = balance.as_u128() as f64 / 1e18;
        Ok(balance_eth)
    }

    /// Get transfer status
    pub async fn get_transfer_status(&self, transfer_id: &str) -> Result<Option<TransferStatus>, BlockchainError> {
        let transfers = self.pending_transfers.read().await;
        Ok(transfers.get(transfer_id).map(|t| t.status.clone()))
    }

    /// Get all pending transfers
    pub async fn get_pending_transfers(&self) -> Result<Vec<PendingTransfer>, BlockchainError> {
        let transfers = self.pending_transfers.read().await;
        Ok(transfers.values().cloned().collect())
    }



    /// Clone bridge for background processing
    pub fn clone_for_background(&self) -> Self {
        Self {
            provider: self.provider.clone(),
            config: self.config.clone(),
            storage: self.storage.clone(),
            pending_transfers: self.pending_transfers.clone(),
        }
    }

    /// Save pending transfer to storage
    async fn save_pending_transfer(&self, transfer: &PendingTransfer) -> Result<(), BlockchainError> {
        let key = format!("ethereum_transfer:{}", transfer.id);
        let value = serde_json::to_string(transfer)
            .map_err(|e| BlockchainError::SerializationError(format!("Failed to serialize transfer: {}", e)))?;
        
        Ok(self.storage.set(&key, value.as_bytes())?)
    }

    /// Load pending transfers from storage
    async fn load_pending_transfers(&self) -> Result<(), BlockchainError> {
        let prefix = "ethereum_transfer:";
        let transfers = self.storage.get_by_prefix(prefix)?;
        
        let mut pending_transfers = self.pending_transfers.write().await;
        
        for (key, value) in transfers.iter() {
            if let Ok(transfer) = serde_json::from_str::<PendingTransfer>(&String::from_utf8_lossy(value)) {
                let id = key.strip_prefix(prefix).unwrap_or(key).to_string();
                pending_transfers.insert(id, transfer);
            }
        }

        Ok(())
    }

    /// Get bridge statistics
    pub async fn get_bridge_stats(&self) -> Result<BridgeStats, BlockchainError> {
        let transfers = self.pending_transfers.read().await;
        
        let mut stats = BridgeStats::default();
        for transfer in transfers.values() {
            stats.total_transfers += 1;
            stats.total_volume += transfer.amount;
            
            match transfer.status {
                TransferStatus::Completed => stats.completed_transfers += 1,
                TransferStatus::Failed(_) => stats.failed_transfers += 1,
                TransferStatus::Pending | TransferStatus::Processing => stats.pending_transfers += 1,
            }
        }

        Ok(stats)
    }

    /// Get bridge status
    pub async fn get_bridge_status(&self) -> Result<BridgeStatus, BlockchainError> {
        let transfers = self.pending_transfers.read().await;
        
        let mut status = BridgeStatus::default();
        status.is_operational = true;
        status.total_transfers = transfers.len() as u64;
        status.last_transfer_time = transfers.values()
            .map(|t| t.created_at)
            .max()
            .unwrap_or_else(|| chrono::Utc::now());
        
        Ok(status)
    }

    /// Get bridge configuration
    pub async fn get_config(&self) -> Result<EthereumConfig, BlockchainError> {
        Ok(self.config.clone())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BridgeStats {
    pub total_transfers: u64,
    pub completed_transfers: u64,
    pub failed_transfers: u64,
    pub pending_transfers: u64,
    pub total_volume: f64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BridgeStatus {
    pub is_operational: bool,
    pub total_transfers: u64,
    pub last_transfer_time: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::BlockchainStorage;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ethereum_bridge_creation() {
        let temp_dir = tempdir().unwrap();
        let storage = Arc::new(BlockchainStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
        let config = EthereumConfig::default();
        
        // This should succeed with the default config
        let result = EthereumBridge::new(config, storage).await;
        assert!(result.is_ok()); // Should succeed with default config
    }

    #[test]
    fn test_ethereum_config_default() {
        let config = EthereumConfig::default();
        assert_eq!(config.chain_id, 11155111); // Sepolia testnet
        assert_eq!(config.gas_limit, 21000);
    }
}
