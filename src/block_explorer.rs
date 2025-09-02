//! Block Explorer for Gillean Blockchain
//! 
//! This module provides a comprehensive block explorer interface for browsing
//! blockchain data, transactions, addresses, and network statistics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::Result;
use crate::blockchain::Blockchain;
use crate::block::Block;
use crate::transaction::Transaction;
use crate::wallet::WalletManager;

/// Block explorer data structures and operations
pub struct BlockExplorer {
    blockchain: Arc<RwLock<Blockchain>>,
    #[allow(dead_code)]
    wallet_manager: Arc<WalletManager>,
    search_cache: Arc<RwLock<HashMap<String, SearchResult>>>,
    #[allow(dead_code)]
    statistics: Arc<RwLock<ExplorerStatistics>>,
}

/// Search result for block explorer queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub result_type: SearchResultType,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

/// Types of search results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchResultType {
    Block,
    Transaction,
    Address,
    Contract,
    NotFound,
}

/// Block explorer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerStatistics {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub total_addresses: u64,
    pub total_contracts: u64,
    pub network_hash_rate: f64,
    pub average_block_time: f64,
    pub transaction_volume_24h: f64,
    pub active_addresses_24h: u64,
    pub last_updated: u64,
}

/// Block details for explorer display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDetails {
    pub block: Block,
    pub transaction_count: usize,
    pub total_fees: f64,
    pub size_bytes: usize,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub timestamp: u64,
    pub confirmations: u64,
    pub next_block_hash: Option<String>,
    pub previous_block_hash: Option<String>,
}

/// Transaction details for explorer display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetails {
    pub transaction: Transaction,
    pub block_height: u64,
    pub block_hash: String,
    pub confirmations: u64,
    pub gas_used: u64,
    pub gas_price: f64,
    pub status: TransactionStatus,
    pub timestamp: u64,
    pub fee: f64,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Dropped,
}

/// Address information for explorer display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub balance: f64,
    pub transaction_count: u64,
    pub first_seen: u64,
    pub last_seen: u64,
    pub is_contract: bool,
    pub contract_code: Option<String>,
    pub nonce: u64,
}

/// Network overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOverview {
    pub statistics: ExplorerStatistics,
    pub recent_blocks: Vec<BlockSummary>,
    pub recent_transactions: Vec<TransactionSummary>,
    pub top_addresses: Vec<AddressSummary>,
    pub network_health: NetworkHealth,
}

/// Block summary for overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub height: u64,
    pub hash: String,
    pub timestamp: u64,
    pub transaction_count: usize,
    pub size_bytes: usize,
    pub miner: String,
}

/// Transaction summary for overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: u64,
    pub status: TransactionStatus,
    pub fee: f64,
}

/// Address summary for overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressSummary {
    pub address: String,
    pub balance: f64,
    pub transaction_count: u64,
    pub rank: u64,
}

/// Network health indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealth {
    pub status: HealthStatus,
    pub block_time_variance: f64,
    pub network_difficulty: f64,
    pub peer_count: u32,
    pub sync_status: String,
    pub last_block_time: u64,
}

/// Health status for network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Search filters for block explorer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub block_range: Option<(u64, u64)>,
    pub transaction_type: Option<String>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
    pub date_range: Option<(u64, u64)>,
    pub status: Option<TransactionStatus>,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u64,
    pub limit: u64,
    pub sort_by: String,
    pub sort_order: SortOrder,
}

/// Sort order for results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl BlockExplorer {
    /// Create a new block explorer
    pub fn new(blockchain: Arc<RwLock<Blockchain>>, wallet_manager: Arc<WalletManager>) -> Self {
        Self {
            blockchain,
            wallet_manager,
            search_cache: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(ExplorerStatistics::default())),
        }
    }
    
    /// Get network overview
    pub async fn get_network_overview(&self) -> Result<NetworkOverview> {
        let _blockchain = self.blockchain.read().await;
        let statistics = self.get_statistics().await?;
        
        // Get recent blocks
        let recent_blocks = self.get_recent_blocks(10).await?;
        
        // Get recent transactions
        let recent_transactions = self.get_recent_transactions(10).await?;
        
        // Get top addresses
        let top_addresses = self.get_top_addresses(10).await?;
        
        // Get network health
        let network_health = self.get_network_health().await?;
        
        Ok(NetworkOverview {
            statistics,
            recent_blocks,
            recent_transactions,
            top_addresses,
            network_health,
        })
    }
    
    /// Get block details by height or hash
    pub async fn get_block_details(&self, identifier: &str) -> Result<BlockDetails> {
        let blockchain = self.blockchain.read().await;
        let block = if let Ok(height) = identifier.parse::<u64>() {
            blockchain.blocks.get(height as usize).cloned().ok_or_else(|| 
                crate::error::BlockchainError::NotFound("Block not found".to_string())
            )?
        } else {
            blockchain.blocks.iter().find(|b| b.hash == identifier).cloned().ok_or_else(|| 
                crate::error::BlockchainError::NotFound("Block not found".to_string())
            )?
        };
        
        let transaction_count = block.transactions.len();
        let total_fees = 0.0; // Simplified - fees not tracked in current implementation
        
        let size_bytes = serde_json::to_string(&block)?.len();
        let gas_used = 0; // Simplified - gas not tracked in current implementation
        let gas_limit = 0; // Simplified
        
        let current_height = blockchain.blocks.len() as u64 - 1;
        let confirmations = current_height - block.index + 1;
        
        let next_block_hash = if block.index < current_height {
            blockchain.blocks.get((block.index + 1) as usize).map(|b| b.hash.clone())
        } else {
            None
        };
        
        let previous_block_hash = if block.index > 0 {
            blockchain.blocks.get((block.index - 1) as usize).map(|b| b.hash.clone())
        } else {
            None
        };
        
        Ok(BlockDetails {
            block,
            transaction_count,
            total_fees,
            size_bytes,
            gas_used,
            gas_limit,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            confirmations,
            next_block_hash,
            previous_block_hash,
        })
    }
    
    /// Get transaction details by hash
    pub async fn get_transaction_details(&self, tx_hash: &str) -> Result<TransactionDetails> {
        let blockchain = self.blockchain.read().await;
        
        // Find transaction in blockchain
        let mut found_transaction = None;
        let mut block_height = 0;
        let mut block_hash = String::new();
        
        for (height, block) in blockchain.blocks.iter().enumerate() {
            for tx in &block.transactions {
                if tx.id == tx_hash {
                    found_transaction = Some(tx.clone());
                    block_height = height as u64;
                    block_hash = block.hash.clone();
                    break;
                }
            }
            if found_transaction.is_some() {
                break;
            }
        }
        
        let transaction = found_transaction.ok_or_else(|| 
            crate::error::BlockchainError::NotFound("Transaction not found".to_string())
        )?;
        
        let current_height = blockchain.blocks.len() as u64 - 1;
        let confirmations = current_height - block_height + 1;
        
        let gas_used = 0; // Simplified
        let gas_price = 0.0; // Simplified
        let fee = gas_used as f64 * gas_price;
        
        let status = if confirmations > 0 {
            TransactionStatus::Confirmed
        } else {
            TransactionStatus::Pending
        };
        
        Ok(TransactionDetails {
            transaction,
            block_height,
            block_hash,
            confirmations,
            gas_used,
            gas_price,
            status,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            fee,
        })
    }
    
    /// Get address information
    pub async fn get_address_info(&self, address: &str) -> Result<AddressInfo> {
        let blockchain = self.blockchain.read().await;
        let balance = blockchain.get_balance(address);
        
        // Count transactions for this address
        let mut transaction_count = 0;
        let mut first_seen = u64::MAX;
        let mut last_seen = 0;
        
        for block in blockchain.blocks.iter() {
            for tx in &block.transactions {
                if tx.sender == address || tx.receiver == address {
                    transaction_count += 1;
                    let tx_time = block.timestamp as u64;
                    if tx_time < first_seen {
                        first_seen = tx_time;
                    }
                    if tx_time > last_seen {
                        last_seen = tx_time;
                    }
                }
            }
        }
        
        let is_contract = false; // Simplified - would check contract registry
        let contract_code = None; // Simplified
        let nonce = 0; // Simplified
        
        Ok(AddressInfo {
            address: address.to_string(),
            balance,
            transaction_count,
            first_seen: if first_seen == u64::MAX { 0 } else { first_seen },
            last_seen,
            is_contract,
            contract_code,
            nonce,
        })
    }
    
    /// Search for blocks, transactions, or addresses
    pub async fn search(&self, query: &str) -> Result<SearchResult> {
        // Check cache first
        {
            let cache = self.search_cache.read().await;
            if let Some(cached_result) = cache.get(query) {
                return Ok(cached_result.clone());
            }
        }
        
        let result = self.perform_search(query).await?;
        
        // Cache the result
        {
            let mut cache = self.search_cache.write().await;
            cache.insert(query.to_string(), result.clone());
        }
        
        Ok(result)
    }
    
    /// Get recent blocks
    pub async fn get_recent_blocks(&self, limit: usize) -> Result<Vec<BlockSummary>> {
        let blockchain = self.blockchain.read().await;
        let current_height = blockchain.blocks.len() as u64 - 1;
        let mut blocks = Vec::new();
        
        let start_height = if current_height >= limit as u64 {
            current_height - limit as u64 + 1
        } else {
            0
        };
        
        for height in (start_height..=current_height).rev() {
            if let Some(block) = blockchain.blocks.get(height as usize) {
                blocks.push(BlockSummary {
                    height: block.index,
                    hash: block.hash.clone(),
                    timestamp: block.timestamp as u64,
                    transaction_count: block.transactions.len(),
                    size_bytes: serde_json::to_string(&block)?.len(),
                    miner: "unknown".to_string(), // Simplified - miner field not available
                });
            }
        }
        
        Ok(blocks)
    }
    
    /// Get recent transactions
    pub async fn get_recent_transactions(&self, limit: usize) -> Result<Vec<TransactionSummary>> {
        let blockchain = self.blockchain.read().await;
        let current_height = blockchain.blocks.len() as u64 - 1;
        let mut transactions = Vec::new();
        
        // Collect transactions from recent blocks
        for height in (0..=current_height).rev() {
            if let Some(block) = blockchain.blocks.get(height as usize) {
                for tx in &block.transactions {
                    if transactions.len() >= limit {
                        break;
                    }
                    
                    transactions.push(TransactionSummary {
                        hash: tx.id.clone(),
                        from: tx.sender.clone(),
                        to: tx.receiver.clone(),
                        amount: tx.amount,
                        timestamp: block.timestamp as u64,
                        status: TransactionStatus::Confirmed,
                        fee: 0.0, // Simplified
                    });
                }
            }
            
            if transactions.len() >= limit {
                break;
            }
        }
        
        Ok(transactions)
    }
    
    /// Get top addresses by balance
    pub async fn get_top_addresses(&self, limit: usize) -> Result<Vec<AddressSummary>> {
        let blockchain = self.blockchain.read().await;
        let mut addresses = Vec::new();
        
        // This is a simplified implementation
        // In a real implementation, you would maintain an index of addresses and balances
        for block in blockchain.blocks.iter() {
            for tx in &block.transactions {
                // Add sender and receiver to addresses list
                if !addresses.iter().any(|a: &AddressSummary| a.address == tx.sender) {
                    let balance = blockchain.get_balance(&tx.sender);
                    if balance > 0.0 {
                        addresses.push(AddressSummary {
                            address: tx.sender.clone(),
                            balance,
                            transaction_count: 0, // Would need to count separately
                            rank: 0,
                        });
                    }
                }
                
                if !addresses.iter().any(|a: &AddressSummary| a.address == tx.receiver) {
                    let balance = blockchain.get_balance(&tx.receiver);
                    if balance > 0.0 {
                        addresses.push(AddressSummary {
                            address: tx.receiver.clone(),
                            balance,
                            transaction_count: 0, // Would need to count separately
                            rank: 0,
                        });
                    }
                }
            }
        }
        
        // Sort by balance and take top addresses
        addresses.sort_by(|a, b| b.balance.partial_cmp(&a.balance).unwrap());
        addresses.truncate(limit);
        
        // Assign ranks
        for (i, address) in addresses.iter_mut().enumerate() {
            address.rank = i as u64 + 1;
        }
        
        Ok(addresses)
    }
    
    /// Get network health status
    pub async fn get_network_health(&self) -> Result<NetworkHealth> {
        let blockchain = self.blockchain.read().await;
        let current_height = blockchain.blocks.len() as u64 - 1;
        
        // Calculate block time variance
        let mut block_times = Vec::new();
        for height in 1..=current_height {
            if let (Some(current_block), Some(prev_block)) = (
                blockchain.blocks.get(height as usize),
                blockchain.blocks.get((height - 1) as usize)
            ) {
                let block_time = current_block.timestamp - prev_block.timestamp;
                block_times.push(block_time as f64);
            }
        }
        
        let average_block_time = if !block_times.is_empty() {
            block_times.iter().sum::<f64>() / block_times.len() as f64
        } else {
            12.0 // Default block time
        };
        
        let block_time_variance = if block_times.len() > 1 {
            let variance = block_times.iter()
                .map(|&time| (time - average_block_time).powi(2))
                .sum::<f64>() / (block_times.len() - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        };
        
        let status = if block_time_variance < 2.0 {
            HealthStatus::Healthy
        } else if block_time_variance < 5.0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };
        
        let last_block_time = if let Some(last_block) = blockchain.blocks.get(current_height as usize) {
            last_block.timestamp
        } else {
            0
        };
        
        Ok(NetworkHealth {
            status,
            block_time_variance,
            network_difficulty: blockchain.difficulty as f64,
            peer_count: 25, // Placeholder - would get from network layer
            sync_status: "synced".to_string(),
            last_block_time: last_block_time as u64,
        })
    }
    
    /// Get explorer statistics
    pub async fn get_statistics(&self) -> Result<ExplorerStatistics> {
        let blockchain = self.blockchain.read().await;
        let current_height = blockchain.blocks.len() as u64 - 1;
        
        let mut total_transactions = 0;
        let mut total_contracts = 0;
        let mut transaction_volume_24h = 0.0;
        
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let day_ago = current_time - 86400; // 24 hours ago
        
        let mut seen_addresses = std::collections::HashSet::new();
        let mut active_addresses = std::collections::HashSet::new();
        
        for height in 0..=current_height {
            if let Some(block) = blockchain.blocks.get(height as usize) {
                total_transactions += block.transactions.len();
                
                for tx in &block.transactions {
                    seen_addresses.insert(tx.sender.clone());
                    seen_addresses.insert(tx.receiver.clone());
                    
                    if block.timestamp >= day_ago as i64 {
                        transaction_volume_24h += tx.amount;
                        active_addresses.insert(tx.sender.clone());
                        active_addresses.insert(tx.receiver.clone());
                    }
                }
            }
        }
        
        let total_addresses = seen_addresses.len() as u64;
        let active_addresses_24h = active_addresses.len() as u64;
        
        // Count contracts (simplified)
        for _address in &seen_addresses {
            if false { // Simplified - would check contract registry
                total_contracts += 1;
            }
        }
        
        Ok(ExplorerStatistics {
            total_blocks: current_height + 1,
            total_transactions: total_transactions as u64,
            total_addresses,
            total_contracts,
            network_hash_rate: 1000000.0, // Placeholder
            average_block_time: 12.0, // Placeholder
            transaction_volume_24h,
            active_addresses_24h,
            last_updated: current_time,
        })
    }
    
    /// Perform actual search
    async fn perform_search(&self, query: &str) -> Result<SearchResult> {
        let blockchain = self.blockchain.read().await;
        
        // Try to parse as block height
        if let Ok(height) = query.parse::<u64>() {
            if blockchain.blocks.get(height as usize).is_some() {
                let block_details = self.get_block_details(query).await?;
                return Ok(SearchResult {
                    result_type: SearchResultType::Block,
                    data: serde_json::to_value(block_details)?,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                });
            }
        }
        
        // Try to find as block hash
        if blockchain.blocks.iter().any(|b| b.hash == query) {
            let block_details = self.get_block_details(query).await?;
            return Ok(SearchResult {
                result_type: SearchResultType::Block,
                data: serde_json::to_value(block_details)?,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
        
        // Try to find as transaction hash
        if blockchain.blocks.iter().any(|b| b.transactions.iter().any(|tx| tx.id == query)) {
            let tx_details = self.get_transaction_details(query).await?;
            return Ok(SearchResult {
                result_type: SearchResultType::Transaction,
                data: serde_json::to_value(tx_details)?,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
        
        // Try to find as address (only if it has a non-zero balance or appears in transactions)
        let balance = blockchain.get_balance(query);
        let has_transactions = blockchain.blocks.iter().any(|b| 
            b.transactions.iter().any(|tx| tx.sender == query || tx.receiver == query)
        );
        
        if balance > 0.0 || has_transactions {
            let address_info = self.get_address_info(query).await?;
            let result_type = if address_info.is_contract {
                SearchResultType::Contract
            } else {
                SearchResultType::Address
            };
            
            return Ok(SearchResult {
                result_type,
                data: serde_json::to_value(address_info)?,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
        
        // Not found
        Ok(SearchResult {
            result_type: SearchResultType::NotFound,
            data: serde_json::Value::Null,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
}

impl Default for ExplorerStatistics {
    fn default() -> Self {
        Self {
            total_blocks: 0,
            total_transactions: 0,
            total_addresses: 0,
            total_contracts: 0,
            network_hash_rate: 0.0,
            average_block_time: 0.0,
            transaction_volume_24h: 0.0,
            active_addresses_24h: 0,
            last_updated: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::Blockchain;
    use crate::wallet::WalletManager;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_block_explorer_creation() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
        let wallet_manager = Arc::new(WalletManager::new());
        let _explorer = BlockExplorer::new(blockchain, wallet_manager);
        
        // Test that explorer was created successfully
        // Basic creation test passed
    }
    
    #[tokio::test]
    async fn test_network_overview() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
        let wallet_manager = Arc::new(WalletManager::new());
        let explorer = BlockExplorer::new(blockchain, wallet_manager);
        
        let overview = explorer.get_network_overview().await.unwrap();
        assert_eq!(overview.statistics.total_blocks, 1); // Genesis block
        assert_eq!(overview.recent_blocks.len(), 1);
    }
    
    #[tokio::test]
    async fn test_block_details() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
        let wallet_manager = Arc::new(WalletManager::new());
        let explorer = BlockExplorer::new(blockchain, wallet_manager);
        
        let block_details = explorer.get_block_details("0").await.unwrap();
        assert_eq!(block_details.block.index, 0);
        assert_eq!(block_details.transaction_count, 1); // Genesis block has 1 coinbase transaction
    }
    
    #[tokio::test]
    async fn test_search_functionality() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
        let wallet_manager = Arc::new(WalletManager::new());
        let explorer = BlockExplorer::new(blockchain, wallet_manager);
        
        // Search for genesis block
        let result = explorer.search("0").await.unwrap();
        assert_eq!(result.result_type, SearchResultType::Block);
        
        // Search for non-existent item
        let result = explorer.search("nonexistent").await.unwrap();
        assert_eq!(result.result_type, SearchResultType::NotFound);
    }
    
    #[tokio::test]
    async fn test_statistics() {
        let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
        let wallet_manager = Arc::new(WalletManager::new());
        let explorer = BlockExplorer::new(blockchain, wallet_manager);
        
        let stats = explorer.get_statistics().await.unwrap();
        assert_eq!(stats.total_blocks, 1); // Genesis block
        assert_eq!(stats.total_transactions, 1); // Genesis block has 1 coinbase transaction
        assert_eq!(stats.total_addresses, 2); // COINBASE and genesis addresses
    }
}
