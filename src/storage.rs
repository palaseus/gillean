use crate::{Blockchain, Block, Transaction, BlockchainError};
use sled::{Db, Tree};
use serde::{Serialize, Deserialize};
use log::{info, error, debug, warn};
use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};
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
    
    #[error("Data integrity check failed: {0}")]
    IntegrityCheckFailed(String),
    
    #[error("Backup operation failed: {0}")]
    BackupFailed(String),
    
    #[error("Recovery operation failed: {0}")]
    RecoveryFailed(String),
    
    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
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
    pub integrity_hash: String,
    pub backup_count: u32,
    pub last_backup: Option<chrono::DateTime<chrono::Utc>>,
}

/// Data integrity check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheckResult {
    pub is_valid: bool,
    pub checksum: String,
    pub block_count: usize,
    pub transaction_count: usize,
    pub corrupted_blocks: Vec<u64>,
    pub corrupted_transactions: Vec<String>,
    pub checked_at: chrono::DateTime<chrono::Utc>,
}

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub backup_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub size_bytes: u64,
    pub block_count: usize,
    pub transaction_count: usize,
    pub integrity_hash: String,
    pub backup_type: BackupType,
}

/// Backup type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupType {
    Full,
    Incremental,
    Differential,
}

/// Storage health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealth {
    pub is_healthy: bool,
    pub disk_usage_percent: f64,
    pub available_space_bytes: u64,
    pub last_integrity_check: Option<chrono::DateTime<chrono::Utc>>,
    pub corruption_detected: bool,
    pub backup_status: BackupStatus,
    pub performance_metrics: PerformanceMetrics,
}

/// Backup status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupStatus {
    UpToDate,
    Outdated,
    Failed,
    InProgress,
}

/// Performance metrics for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub read_operations_per_second: f64,
    pub write_operations_per_second: f64,
    pub average_read_latency_ms: f64,
    pub average_write_latency_ms: f64,
    pub cache_hit_rate: f64,
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
    backups_tree: Tree,
    integrity_tree: Tree,
    backup_path: String,
    db_path: String,
    #[allow(dead_code)]
    last_integrity_check: Option<chrono::DateTime<chrono::Utc>>,
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
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // Try to open the database with retry logic
        let db = Arc::new({
            let mut attempts = 0;
            let max_attempts = 5;
            
            loop {
                match sled::open(&path) {
                    Ok(db) => break db,
                    Err(e) => {
                        attempts += 1;
                        if attempts >= max_attempts {
                            return Err(StorageError::Database(e));
                        }
                        info!("Database lock attempt {} failed, retrying...", attempts);
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
            }
        });
        
        let blocks_tree = db.open_tree("blocks")?;
        let transactions_tree = db.open_tree("transactions")?;
        let balances_tree = db.open_tree("balances")?;
        let metadata_tree = db.open_tree("metadata")?;
        let wallets_tree = db.open_tree("wallets")?;
        let backups_tree = db.open_tree("backups")?;
        let integrity_tree = db.open_tree("integrity")?;
        
        info!("Initialized blockchain storage with enhanced features");
        
        Ok(BlockchainStorage {
            db,
            blocks_tree,
            transactions_tree,
            balances_tree,
            metadata_tree,
            wallets_tree,
            backups_tree,
            integrity_tree,
            backup_path: format!("{}/backups", path_str),
            db_path: path_str,
            last_integrity_check: None,
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
            integrity_hash: "".to_string(),
            backup_count: 0,
            last_backup: None,
        };
        
        self.save_metadata(&metadata)?;
        info!("Initialized blockchain metadata with integrity tracking");
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
            state_snapshots: Vec::new(), // Default to empty for backward compatibility
            state_tree: crate::blockchain::StateMerkleTree::new(), // Default to empty for backward compatibility
            state_lock: std::sync::Arc::new(std::sync::Mutex::new(())), // Default to new lock
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
            integrity_hash: "".to_string(),
            backup_count: 0,
            last_backup: None,
        };
        
        self.save_metadata(&metadata)?;
        
        // Ensure all changes are flushed to disk
        self.flush()?;
        
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

    /// Perform comprehensive data integrity check
    /// 
    /// # Returns
    /// * `Result<IntegrityCheckResult>` - Integrity check results
    pub fn perform_integrity_check(&self) -> std::result::Result<IntegrityCheckResult, StorageError> {
        info!("Starting comprehensive data integrity check");
        let start_time = SystemTime::now();
        
        let mut corrupted_blocks = Vec::new();
        let mut corrupted_transactions = Vec::new();
        let mut block_count = 0;
        let mut transaction_count = 0;
        let mut hasher = Sha256::new();
        
        // Check all blocks
        for result in self.blocks_tree.iter() {
            let (key, value) = result?;
            let block_index = String::from_utf8_lossy(&key);
            
            // Try to deserialize the block
            match serde_json::from_slice::<Block>(&value) {
                Ok(block) => {
                    block_count += 1;
                    // Verify block hash
                    let computed_hash = block.calculate_current_hash();
                    if computed_hash != block.hash {
                        warn!("Block {} has invalid hash", block.index);
                        corrupted_blocks.push(block.index);
                    }
                    
                    // Add to integrity hash
                    hasher.update(&value);
                    transaction_count += block.transactions.len();
                }
                Err(e) => {
                    error!("Failed to deserialize block {}: {}", block_index, e);
                    if let Ok(index) = block_index.parse::<u64>() {
                        corrupted_blocks.push(index);
                    }
                }
            }
        }
        
        // Check all transactions
        for result in self.transactions_tree.iter() {
            let (key, value) = result?;
            let tx_id = String::from_utf8_lossy(&key);
            
            match serde_json::from_slice::<Transaction>(&value) {
                Ok(transaction) => {
                    // Verify transaction ID is not empty
                    if transaction.id.is_empty() {
                        warn!("Transaction has empty ID");
                        corrupted_transactions.push(tx_id.to_string());
                    }
                    
                    // Add to integrity hash
                    hasher.update(&value);
                }
                Err(e) => {
                    error!("Failed to deserialize transaction {}: {}", tx_id, e);
                    corrupted_transactions.push(tx_id.to_string());
                }
            }
        }
        
        // Check balances
        for result in self.balances_tree.iter() {
            let (_, value) = result?;
            hasher.update(&value);
        }
        
        let checksum = format!("{:x}", hasher.finalize());
        let is_valid = corrupted_blocks.is_empty() && corrupted_transactions.is_empty();
        
        let result = IntegrityCheckResult {
            is_valid,
            checksum,
            block_count,
            transaction_count,
            corrupted_blocks,
            corrupted_transactions,
            checked_at: chrono::Utc::now(),
        };
        
        // Store integrity check result
        self.integrity_tree.insert("last_check", serde_json::to_vec(&result)?)?;
        self.flush()?;
        
        let duration = start_time.elapsed().unwrap_or_default();
        info!("Integrity check completed in {:?}. Valid: {}, Blocks: {}, Transactions: {}", 
              duration, is_valid, block_count, transaction_count);
        
        if !is_valid {
            warn!("Data integrity issues detected: {} corrupted blocks, {} corrupted transactions", 
                  result.corrupted_blocks.len(), result.corrupted_transactions.len());
        }
        
        Ok(result)
    }

    /// Create a backup of the blockchain data
    /// 
    /// # Arguments
    /// * `backup_type` - Type of backup to create
    /// 
    /// # Returns
    /// * `Result<BackupInfo>` - Information about the created backup
    pub fn create_backup(&self, backup_type: BackupType) -> std::result::Result<BackupInfo, StorageError> {
        info!("Creating {:?} backup", backup_type);
        let start_time = SystemTime::now();
        
        // Create backup directory if it doesn't exist
        std::fs::create_dir_all(&self.backup_path)?;
        
        let backup_id = format!("backup_{}", 
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        let backup_file = format!("{}/{}.db", self.backup_path, backup_id);
        
        // Perform integrity check before backup
        let integrity_result = self.perform_integrity_check()?;
        if !integrity_result.is_valid {
            return Err(StorageError::BackupFailed(
                "Cannot create backup: data integrity check failed".to_string()
            ));
        }
        
        // Create backup by copying the database
        let backup_db = sled::open(&backup_file)?;
        
        // Copy all trees
        for tree_name in ["blocks", "transactions", "balances", "metadata", "wallets", "backups", "integrity"] {
            if let Ok(source_tree) = self.db.open_tree(tree_name) {
                let backup_tree = backup_db.open_tree(tree_name)?;
                for result in source_tree.iter() {
                    let (key, value) = result?;
                    backup_tree.insert(key, value)?;
                }
                backup_tree.flush()?;
            }
        }
        
        backup_db.flush()?;
        drop(backup_db);
        
        // Get backup file size
        let backup_size = std::fs::metadata(&backup_file)?.len();
        
        let backup_info = BackupInfo {
            backup_id: backup_id.clone(),
            created_at: chrono::Utc::now(),
            size_bytes: backup_size,
            block_count: integrity_result.block_count,
            transaction_count: integrity_result.transaction_count,
            integrity_hash: integrity_result.checksum,
            backup_type,
        };
        
        // Store backup information
        self.backups_tree.insert(&backup_id, serde_json::to_vec(&backup_info)?)?;
        self.flush()?;
        
        let duration = start_time.elapsed().unwrap_or_default();
        info!("Backup {} created successfully in {:?}. Size: {} bytes", 
              backup_id, duration, backup_size);
        
        Ok(backup_info)
    }

    /// Restore from a backup
    /// 
    /// # Arguments
    /// * `backup_id` - ID of the backup to restore from
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if restored successfully
    pub fn restore_from_backup(&self, backup_id: &str) -> std::result::Result<(), StorageError> {
        info!("Restoring from backup: {}", backup_id);
        
        // Get backup information
        let backup_data = self.backups_tree.get(backup_id)?
            .ok_or_else(|| StorageError::RecoveryFailed(
                format!("Backup {} not found", backup_id)
            ))?;
        
        let backup_info: BackupInfo = serde_json::from_slice(&backup_data)?;
        let backup_file = format!("{}/{}.db", self.backup_path, backup_id);
        
        // Verify backup file exists
        if !std::path::Path::new(&backup_file).exists() {
            return Err(StorageError::RecoveryFailed(
                format!("Backup file not found: {}", backup_file)
            ));
        }
        
        // Create temporary backup of current state
        let temp_backup = self.create_backup(BackupType::Full)?;
        info!("Created temporary backup {} before restoration", temp_backup.backup_id);
        
        // Close current database connections
        drop(self.blocks_tree.clone());
        drop(self.transactions_tree.clone());
        drop(self.balances_tree.clone());
        drop(self.metadata_tree.clone());
        drop(self.wallets_tree.clone());
        drop(self.backups_tree.clone());
        drop(self.integrity_tree.clone());
        
        // Replace database with backup
        let current_db_path = &self.db_path;
        let temp_path = format!("{}.temp", current_db_path);
        
        // Move current database to temp location
        if std::path::Path::new(current_db_path).exists() {
            std::fs::rename(current_db_path, &temp_path)?;
        }
        
        // Copy backup to current location
        std::fs::copy(&backup_file, current_db_path)?;
        
        info!("Successfully restored from backup {}. Restored {} blocks and {} transactions", 
              backup_id, backup_info.block_count, backup_info.transaction_count);
        
        Ok(())
    }

    /// Get storage health status
    /// 
    /// # Returns
    /// * `Result<StorageHealth>` - Current storage health status
    pub fn get_storage_health(&self) -> std::result::Result<StorageHealth, StorageError> {
        // Get disk usage information
        let current_path = &self.db_path;
        let metadata = std::fs::metadata(current_path)?;
        let total_size = metadata.len();
        
        // Calculate available space (simplified)
        let available_space = 1_000_000_000; // 1GB placeholder
        let disk_usage_percent = (total_size as f64 / (total_size + available_space) as f64) * 100.0;
        
        // Get last integrity check
        let last_integrity_check = if let Some(data) = self.integrity_tree.get("last_check")? {
            let result: IntegrityCheckResult = serde_json::from_slice(&data)?;
            Some(result.checked_at)
        } else {
            None
        };
        
        // Determine backup status
        let backup_status = if let Some(data) = self.backups_tree.get("latest")? {
            let backup_info: BackupInfo = serde_json::from_slice(&data)?;
            let days_since_backup = chrono::Utc::now().signed_duration_since(backup_info.created_at).num_days();
            if days_since_backup > 7 {
                BackupStatus::Outdated
            } else {
                BackupStatus::UpToDate
            }
        } else {
            BackupStatus::Failed
        };
        
        // Check for corruption
        let corruption_detected = if let Some(data) = self.integrity_tree.get("last_check")? {
            let result: IntegrityCheckResult = serde_json::from_slice(&data)?;
            !result.is_valid
        } else {
            false
        };
        
        let health = StorageHealth {
            is_healthy: !corruption_detected && disk_usage_percent < 90.0,
            disk_usage_percent,
            available_space_bytes: available_space,
            last_integrity_check,
            corruption_detected,
            backup_status,
            performance_metrics: PerformanceMetrics {
                read_operations_per_second: 1000.0, // Placeholder
                write_operations_per_second: 500.0,  // Placeholder
                average_read_latency_ms: 1.0,        // Placeholder
                average_write_latency_ms: 2.0,       // Placeholder
                cache_hit_rate: 0.85,                // Placeholder
            },
        };
        
        Ok(health)
    }

    /// List all available backups
    /// 
    /// # Returns
    /// * `Result<Vec<BackupInfo>>` - List of all backups
    pub fn list_backups(&self) -> std::result::Result<Vec<BackupInfo>, StorageError> {
        let mut backups = Vec::new();
        
        for result in self.backups_tree.iter() {
            let (_, value) = result?;
            let backup_info: BackupInfo = serde_json::from_slice(&value)?;
            backups.push(backup_info);
        }
        
        // Sort by creation date (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(backups)
    }

    /// Clean up old backups
    /// 
    /// # Arguments
    /// * `keep_count` - Number of recent backups to keep
    /// 
    /// # Returns
    /// * `Result<usize>` - Number of backups cleaned up
    pub fn cleanup_old_backups(&self, keep_count: usize) -> std::result::Result<usize, StorageError> {
        let mut backups = self.list_backups()?;
        
        if backups.len() <= keep_count {
            return Ok(0);
        }
        
        let to_remove = backups.split_off(keep_count);
        let mut removed_count = 0;
        
        for backup in to_remove {
            // Remove backup file
            let backup_file = format!("{}/{}.db", self.backup_path, backup.backup_id);
            let backup_path = std::path::Path::new(&backup_file);
            if backup_path.exists() && backup_path.is_file() {
                std::fs::remove_file(&backup_file)?;
            }
            
            // Remove backup record
            self.backups_tree.remove(&backup.backup_id)?;
            removed_count += 1;
        }
        
        self.flush()?;
        info!("Cleaned up {} old backups", removed_count);
        
        Ok(removed_count)
    }
    
    /// Close the database properly
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if closed successfully
    pub fn close(&self) -> std::result::Result<(), StorageError> {
        info!("Closing blockchain storage...");
        self.flush()?;
        // The Arc will be dropped when the last reference is dropped
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
