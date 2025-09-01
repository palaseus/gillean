use crate::{Blockchain, Block, Transaction, BlockchainError};
// use crate::Result; // Unused import
use sled::{Db, Tree};
use serde::{Serialize, Deserialize};
use log::{info, error, debug};
use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Storage-related errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sled::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Database corruption detected: {0}")]
    Corruption(String),
    
    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },
    
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),
}

impl From<StorageError> for BlockchainError {
    fn from(err: StorageError) -> Self {
        BlockchainError::StorageError(err.to_string())
    }
}

/// Metadata about the blockchain stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainMetadata {
    pub version: String,
    pub difficulty: u32,
    pub mining_reward: f64,
    pub total_blocks: usize,
    pub total_transactions: usize,
    pub last_block_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Persistent storage for the blockchain using sled
#[derive(Debug)]
pub struct BlockchainStorage {
    db: Arc<Db>,
    blocks_tree: Tree,
    transactions_tree: Tree,
    balances_tree: Tree,
    metadata_tree: Tree,
    wallets_tree: Tree,
}

impl BlockchainStorage {
    /// Create a new storage instance
    /// 
    /// # Arguments
    /// * `path` - Path to the database directory
    /// 
    /// # Returns
    /// * `Result<BlockchainStorage>` - The storage instance or an error
    pub fn new<P: AsRef<Path>>(path: P) -> std::result::Result<Self, StorageError> {
        let db = Arc::new(sled::open(path)?);
        
        let blocks_tree = db.open_tree("blocks")?;
        let transactions_tree = db.open_tree("transactions")?;
        let balances_tree = db.open_tree("balances")?;
        let metadata_tree = db.open_tree("metadata")?;
        let wallets_tree = db.open_tree("wallets")?;
        
        info!("Initialized blockchain storage");
        
        Ok(BlockchainStorage {
            db,
            blocks_tree,
            transactions_tree,
            balances_tree,
            metadata_tree,
            wallets_tree,
        })
    }
    
    /// Initialize the database with default metadata
    /// 
    /// # Arguments
    /// * `difficulty` - Mining difficulty
    /// * `mining_reward` - Mining reward amount
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if initialized successfully
    pub fn initialize(&self, difficulty: u32, mining_reward: f64) -> std::result::Result<(), StorageError> {
        let metadata = BlockchainMetadata {
            version: crate::BLOCKCHAIN_VERSION.to_string(),
            difficulty,
            mining_reward,
            total_blocks: 0,
            total_transactions: 0,
            last_block_hash: crate::GENESIS_HASH.to_string(),
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        };
        
        self.save_metadata(&metadata)?;
        info!("Initialized blockchain metadata");
        Ok(())
    }
    
    /// Save a block to storage
    /// 
    /// # Arguments
    /// * `block` - The block to save
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if saved successfully
    pub fn save_block(&self, block: &Block) -> std::result::Result<(), StorageError> {
        let key = block.index.to_string();
        let value = serde_json::to_vec(block)?;
        
        self.blocks_tree.insert(key, value)?;
        self.flush()?;
        
        debug!("Saved block #{} to storage", block.index);
        Ok(())
    }
    
    /// Load a block from storage
    /// 
    /// # Arguments
    /// * `index` - Block index to load
    /// 
    /// # Returns
    /// * `Result<Option<Block>>` - The block if found, None otherwise
    pub fn load_block(&self, index: u64) -> std::result::Result<Option<Block>, StorageError> {
        let key = index.to_string();
        
        if let Some(value) = self.blocks_tree.get(key)? {
            let block: Block = serde_json::from_slice(&value)?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }
    
    /// Save all blocks from a blockchain
    /// 
    /// # Arguments
    /// * `blockchain` - The blockchain containing blocks to save
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if saved successfully
    pub fn save_all_blocks(&self, blockchain: &Blockchain) -> std::result::Result<(), StorageError> {
        for block in &blockchain.blocks {
            self.save_block(block)?;
        }
        
        info!("Saved {} blocks to storage", blockchain.blocks.len());
        Ok(())
    }
    
    /// Load all blocks from storage
    /// 
    /// # Returns
    /// * `Result<Vec<Block>>` - All blocks in order
    pub fn load_all_blocks(&self) -> std::result::Result<Vec<Block>, StorageError> {
        let mut blocks = Vec::new();
        
        for result in self.blocks_tree.iter() {
            let (_, value) = result?;
            let block: Block = serde_json::from_slice(&value)?;
            blocks.push(block);
        }
        
        // Sort blocks by index
        blocks.sort_by_key(|block| block.index);
        
        info!("Loaded {} blocks from storage", blocks.len());
        Ok(blocks)
    }
    
    /// Save pending transactions
    /// 
    /// # Arguments
    /// * `transactions` - Transactions to save
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if saved successfully
    pub fn save_pending_transactions(&self, transactions: &[Transaction]) -> std::result::Result<(), StorageError> {
        // Clear existing pending transactions
        self.transactions_tree.clear()?;
        
        // Save new pending transactions
        for (i, transaction) in transactions.iter().enumerate() {
            let key = format!("pending_{}", i);
            let value = serde_json::to_vec(transaction)?;
            self.transactions_tree.insert(key, value)?;
        }
        
        self.flush()?;
        debug!("Saved {} pending transactions to storage", transactions.len());
        Ok(())
    }
    
    /// Load pending transactions from storage
    /// 
    /// # Returns
    /// * `Result<Vec<Transaction>>` - Pending transactions
    pub fn load_pending_transactions(&self) -> std::result::Result<Vec<Transaction>, StorageError> {
        let mut transactions = Vec::new();
        
        for result in self.transactions_tree.iter() {
            let (key, value) = result?;
            let key_str = String::from_utf8_lossy(&key);
            
            if key_str.starts_with("pending_") {
                let transaction: Transaction = serde_json::from_slice(&value)?;
                transactions.push(transaction);
            }
        }
        
        debug!("Loaded {} pending transactions from storage", transactions.len());
        Ok(transactions)
    }
    
    /// Save balances
    /// 
    /// # Arguments
    /// * `balances` - Balances to save
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if saved successfully
    pub fn save_balances(&self, balances: &HashMap<String, f64>) -> std::result::Result<(), StorageError> {
        // Clear existing balances
        self.balances_tree.clear()?;
        
        // Save new balances
        for (address, balance) in balances {
            let value = serde_json::to_vec(balance)?;
            self.balances_tree.insert(address, value)?;
        }
        
        self.flush()?;
        debug!("Saved {} balances to storage", balances.len());
        Ok(())
    }
    
    /// Load balances from storage
    /// 
    /// # Returns
    /// * `Result<HashMap<String, f64>>` - Balances
    pub fn load_balances(&self) -> std::result::Result<HashMap<String, f64>, StorageError> {
        let mut balances = HashMap::new();
        
        for result in self.balances_tree.iter() {
            let (key, value) = result?;
            let address = String::from_utf8_lossy(&key).to_string();
            let balance: f64 = serde_json::from_slice(&value)?;
            balances.insert(address, balance);
        }
        
        debug!("Loaded {} balances from storage", balances.len());
        Ok(balances)
    }
    
    /// Save metadata
    /// 
    /// # Arguments
    /// * `metadata` - Metadata to save
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if saved successfully
    pub fn save_metadata(&self, metadata: &BlockchainMetadata) -> std::result::Result<(), StorageError> {
        let value = serde_json::to_vec(metadata)?;
        self.metadata_tree.insert("metadata", value)?;
        self.flush()?;
        Ok(())
    }
    
    /// Load metadata from storage
    /// 
    /// # Returns
    /// * `Result<Option<BlockchainMetadata>>` - Metadata if found
    pub fn load_metadata(&self) -> std::result::Result<Option<BlockchainMetadata>, StorageError> {
        if let Some(value) = self.metadata_tree.get("metadata")? {
            let metadata: BlockchainMetadata = serde_json::from_slice(&value)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }
    
    /// Save a wallet
    /// 
    /// # Arguments
    /// * `address` - Wallet address
    /// * `wallet_data` - Encrypted wallet data
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if saved successfully
    pub fn save_wallet(&self, address: &str, wallet_data: &[u8]) -> std::result::Result<(), StorageError> {
        self.wallets_tree.insert(address, wallet_data)?;
        self.flush()?;
        debug!("Saved wallet for address: {}", address);
        Ok(())
    }
    
    /// Load a wallet
    /// 
    /// # Arguments
    /// * `address` - Wallet address
    /// 
    /// # Returns
    /// * `Result<Option<Vec<u8>>>` - Encrypted wallet data if found
    pub fn load_wallet(&self, address: &str) -> std::result::Result<Option<Vec<u8>>, StorageError> {
        Ok(self.wallets_tree.get(address)?.map(|v| v.to_vec()))
    }
    
    /// List all wallet addresses
    /// 
    /// # Returns
    /// * `Result<Vec<String>>` - List of wallet addresses
    pub fn list_wallets(&self) -> std::result::Result<Vec<String>, StorageError> {
        let mut addresses = Vec::new();
        
        for result in self.wallets_tree.iter() {
            let (key, _) = result?;
            let address = String::from_utf8_lossy(&key).to_string();
            addresses.push(address);
        }
        
        Ok(addresses)
    }
    
    /// Load a complete blockchain from storage
    /// 
    /// # Arguments
    /// * `difficulty` - Mining difficulty
    /// * `mining_reward` - Mining reward amount
    /// 
    /// # Returns
    /// * `Result<Blockchain>` - The loaded blockchain
    pub fn load_blockchain(&self, difficulty: u32, mining_reward: f64) -> std::result::Result<Blockchain, StorageError> {
        // Check if database is initialized
        if let Some(metadata) = self.load_metadata()? {
            // Validate version compatibility
            if metadata.version != crate::BLOCKCHAIN_VERSION {
                                  return Err(StorageError::VersionMismatch {
                    expected: crate::BLOCKCHAIN_VERSION.to_string(),
                    found: metadata.version,
                });
            }
            
            info!("Loading blockchain from storage (version: {})", metadata.version);
        } else {
            // Initialize new database
            self.initialize(difficulty, mining_reward)?;
            info!("Initialized new blockchain storage");
        }
        
        // Load blocks
        let blocks = self.load_all_blocks()?;
        
        // Load pending transactions
        let pending_transactions = self.load_pending_transactions()?;
        
        // Load balances
        let balances = self.load_balances()?;
        
        // Create blockchain
        let blockchain = Blockchain {
            blocks,
            pending_transactions,
            difficulty,
            mining_reward,
            proof_of_work: crate::ProofOfWork::new(difficulty, 1_000_000).map_err(|e| StorageError::Corruption(e.to_string()))?,
            version: crate::BLOCKCHAIN_VERSION.to_string(),
            balances,
            consensus_type: crate::ConsensusType::ProofOfWork, // Default to PoW for backward compatibility
            proof_of_stake: None, // Default to None for backward compatibility
            contracts: HashMap::new(), // Default to empty for backward compatibility
            contract_metrics: HashMap::new(), // Default to empty for backward compatibility
        };
        
        info!("Successfully loaded blockchain from storage");
        Ok(blockchain)
    }
    
    /// Save a complete blockchain to storage
    /// 
    /// # Arguments
    /// * `blockchain` - The blockchain to save
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if saved successfully
    pub fn save_blockchain(&self, blockchain: &Blockchain) -> std::result::Result<(), StorageError> {
        // Save all blocks
        self.save_all_blocks(blockchain)?;
        
        // Save pending transactions
        self.save_pending_transactions(&blockchain.pending_transactions)?;
        
        // Save balances
        self.save_balances(&blockchain.balances)?;
        
        // Update metadata
        let metadata = BlockchainMetadata {
            version: blockchain.version.clone(),
            difficulty: blockchain.difficulty,
            mining_reward: blockchain.mining_reward,
            total_blocks: blockchain.blocks.len(),
            total_transactions: blockchain.blocks.iter().map(|b| b.transactions.len()).sum(),
            last_block_hash: blockchain.blocks.last().map(|b| b.hash.clone()).unwrap_or_default(),
            created_at: chrono::Utc::now(), // This should be preserved from original metadata
            last_updated: chrono::Utc::now(),
        };
        
        self.save_metadata(&metadata)?;
        
        info!("Successfully saved blockchain to storage");
        Ok(())
    }
    
    /// Flush all changes to disk
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if flushed successfully
    pub fn flush(&self) -> std::result::Result<(), StorageError> {
        self.db.flush()?;
        Ok(())
    }
    
    /// Get database size in bytes
    /// 
    /// # Returns
    /// * `Result<usize>` - Database size
    pub fn size(&self) -> std::result::Result<usize, StorageError> {
        Ok(self.db.size_on_disk()?.try_into().unwrap())
    }
    
    /// Compact the database to reclaim space
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if compacted successfully
    pub fn compact(&self) -> std::result::Result<(), StorageError> {
        // Note: sled doesn't have a compact method, so we'll just return Ok
        info!("Database compaction requested (not implemented in sled)");
        Ok(())
    }

    /// Set a key-value pair in the database
    /// 
    /// # Arguments
    /// * `key` - The key to set
    /// * `value` - The value to set
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if set successfully
    pub fn set(&self, key: &str, value: &[u8]) -> std::result::Result<(), StorageError> {
        self.blocks_tree.insert(key, value)?;
        self.flush()?;
        Ok(())
    }

    /// Get a value by key from the database
    /// 
    /// # Arguments
    /// * `key` - The key to get
    /// 
    /// # Returns
    /// * `Result<Option<Vec<u8>>>` - The value if found, None otherwise
    pub fn get(&self, key: &str) -> std::result::Result<Option<Vec<u8>>, StorageError> {
        Ok(self.blocks_tree.get(key)?.map(|v| v.to_vec()))
    }

    /// Get all values with a given prefix
    /// 
    /// # Arguments
    /// * `prefix` - The prefix to search for
    /// 
    /// # Returns
    /// * `Result<Vec<(String, Vec<u8>)>>` - All key-value pairs with the prefix
    pub fn get_by_prefix(&self, prefix: &str) -> std::result::Result<Vec<(String, Vec<u8>)>, StorageError> {
        let mut results = Vec::new();
        
        for result in self.blocks_tree.iter() {
            let (key, value) = result?;
            let key_str = String::from_utf8_lossy(&key);
            
            if key_str.starts_with(prefix) {
                results.push((key_str.to_string(), value.to_vec()));
            }
        }
        
        Ok(results)
    }

    /// Delete a key from the database
    /// 
    /// # Arguments
    /// * `key` - The key to delete
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if deleted successfully
    pub fn delete(&self, key: &str) -> std::result::Result<(), StorageError> {
        self.blocks_tree.remove(key)?;
        self.flush()?;
        Ok(())
    }
}

impl Drop for BlockchainStorage {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            error!("Failed to flush database on drop: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::Blockchain;
    
    #[test]
    fn test_storage_creation() {
        let temp_dir = tempdir().unwrap();
        let storage = BlockchainStorage::new(temp_dir.path()).unwrap();
        let _size = storage.size().unwrap(); // Verify storage size can be retrieved
    }
    
    #[test]
    fn test_blockchain_save_load() {
        let temp_dir = tempdir().unwrap();
        let storage = BlockchainStorage::new(temp_dir.path()).unwrap();
        
        // Create a blockchain
        let mut blockchain = Blockchain::new_pow(2, 50.0).unwrap();
        // Add initial balance to alice
        blockchain.balances.insert("alice".to_string(), 1000.0);
        blockchain.add_transaction("alice".to_string(), "bob".to_string(), 100.0, None).unwrap();
        blockchain.mine_block("miner".to_string()).unwrap();
        
        // Save blockchain
        storage.save_blockchain(&blockchain).unwrap();
        
        // Load blockchain
        let loaded_blockchain = storage.load_blockchain(2, 50.0).unwrap();
        
        // Verify they match
        assert_eq!(blockchain.blocks.len(), loaded_blockchain.blocks.len());
        assert_eq!(blockchain.pending_transactions.len(), loaded_blockchain.pending_transactions.len());
        assert_eq!(blockchain.balances.len(), loaded_blockchain.balances.len());
    }
    
    #[test]
    fn test_wallet_storage() {
        let temp_dir = tempdir().unwrap();
        let storage = BlockchainStorage::new(temp_dir.path()).unwrap();
        
        let address = "test_address";
        let wallet_data = b"encrypted_wallet_data";
        
        // Save wallet
        storage.save_wallet(address, wallet_data).unwrap();
        
        // Load wallet
        let loaded_data = storage.load_wallet(address).unwrap().unwrap();
        assert_eq!(wallet_data, loaded_data.as_slice());
        
        // List wallets
        let addresses = storage.list_wallets().unwrap();
        assert_eq!(addresses.len(), 1);
        assert_eq!(addresses[0], address);
    }
}
