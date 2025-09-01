//! # Sharding Module
//! 
//! This module implements basic sharding functionality for the Gillean blockchain,
//! dividing the blockchain into multiple shards to parallelize transaction processing.
//! 
//! ## Features
//! 
//! - **Shard Assignment**: Transactions are assigned to shards based on sender address hash
//! - **Shard Management**: Each shard maintains its own blockchain state
//! - **Cross-Shard Transactions**: Simple two-phase commit protocol for cross-shard operations
//! - **Shard Synchronization**: Coordination between shards for consistency
//! 
//! ## Architecture
//! 
//! The sharding system consists of:
//! - `ShardManager`: Orchestrates all shards and handles cross-shard coordination
//! - `Shard`: Individual shard with its own blockchain and state
//! - `ShardTransaction`: Transaction with shard-specific metadata
//! - `CrossShardTransaction`: Transaction that affects multiple shards

use crate::{
    blockchain::Blockchain,
    storage::BlockchainStorage,
    error::{BlockchainError, Result},
    consensus::ConsensusType,
    transaction::Transaction,
};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
    fs,
};
use dashmap::DashMap;
use crossbeam_channel::{bounded, Sender, Receiver};
use log::{info, debug};

/// Number of shards in the system
pub const NUM_SHARDS: u32 = 4;

/// Shard transaction with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardTransaction {
    /// Original transaction
    pub transaction: Transaction,
    /// Source shard ID
    pub source_shard: u32,
    /// Target shard ID (for cross-shard transactions)
    pub target_shard: Option<u32>,
    /// Cross-shard transaction ID (if applicable)
    pub cross_shard_id: Option<String>,
    /// Transaction status
    pub status: ShardTransactionStatus,
    /// Timestamp when transaction was assigned to shard
    pub assigned_at: u64,
}

/// Status of a shard transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShardTransactionStatus {
    /// Transaction is pending processing
    Pending,
    /// Transaction is being processed
    Processing,
    /// Transaction has been committed
    Committed,
    /// Transaction failed
    Failed(String),
    /// Cross-shard transaction in prepare phase
    Prepare,
    /// Cross-shard transaction in commit phase
    Commit,
}

/// Cross-shard transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardTransaction {
    /// Unique identifier for the cross-shard transaction
    pub id: String,
    /// Source shard ID
    pub source_shard: u32,
    /// Target shard ID
    pub target_shard: u32,
    /// Original transaction
    pub transaction: Transaction,
    /// Status of the cross-shard transaction
    pub status: CrossShardStatus,
    /// Timestamp when transaction was created
    pub created_at: u64,
    /// Participants in the transaction
    pub participants: Vec<u32>,
}

/// Status of a cross-shard transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CrossShardStatus {
    /// Transaction is being prepared
    Preparing,
    /// All shards have prepared
    Prepared,
    /// Transaction is being committed
    Committing,
    /// Transaction has been committed
    Committed,
    /// Transaction failed
    Failed(String),
}

/// Individual shard with its own blockchain
#[derive(Debug)]
pub struct Shard {
    /// Shard ID
    pub id: u32,
    /// Blockchain instance for this shard
    pub blockchain: Blockchain,
    /// Pending transactions for this shard
    pub pending_transactions: Arc<RwLock<Vec<ShardTransaction>>>,
    /// Cross-shard transactions involving this shard
    pub cross_shard_transactions: Arc<RwLock<HashMap<String, CrossShardTransaction>>>,
    /// Transaction processing channel
    pub tx_sender: Sender<ShardTransaction>,
    /// Transaction processing receiver
    pub tx_receiver: Receiver<ShardTransaction>,
}

impl Shard {
    /// Create a new shard with the given ID
    pub fn new(id: u32, consensus_type: ConsensusType) -> Result<Self> {
        // Use unique database path for tests to avoid conflicts
        let db_path = if cfg!(test) {
            format!("data/shards/test_shard_{}_{}", id, std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos())
        } else {
            format!("data/shards/shard_{}", id)
        };
        
        // Ensure data directory exists
        fs::create_dir_all("data/shards")?;
        
        let _storage = BlockchainStorage::new(&db_path)?;
        let blockchain = match consensus_type {
            ConsensusType::ProofOfWork => Blockchain::new_pow(4, 50.0)?,
            ConsensusType::ProofOfStake => Blockchain::new_pos(50.0, 100.0, 5)?,
        };
        
        let (tx_sender, tx_receiver) = bounded(1000);
        
        Ok(Self {
            id,
            blockchain,
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
            cross_shard_transactions: Arc::new(RwLock::new(HashMap::new())),
            tx_sender,
            tx_receiver,
        })
    }

    /// Process a transaction within this shard
    pub fn process_transaction(&mut self, shard_tx: ShardTransaction) -> Result<()> {
        debug!("Processing transaction in shard {}: {:?}", self.id, shard_tx.transaction.id);
        
        // Add to pending transactions
        {
            let mut pending = self.pending_transactions.write().unwrap();
            pending.push(shard_tx.clone());
        }

        // Process the transaction
        match shard_tx.target_shard {
            None => {
                // Single-shard transaction
                self.process_single_shard_transaction(shard_tx)
            }
            Some(_target_shard) => {
                // Cross-shard transaction
                self.process_cross_shard_transaction(shard_tx)
            }
        }
    }

    /// Process a single-shard transaction
    fn process_single_shard_transaction(&mut self, mut shard_tx: ShardTransaction) -> Result<()> {
        shard_tx.status = ShardTransactionStatus::Processing;
        
        // Add transaction to blockchain
        self.blockchain.add_transaction(
            shard_tx.transaction.sender.clone(),
            shard_tx.transaction.receiver.clone(),
            shard_tx.transaction.amount,
            shard_tx.transaction.message.clone(),
        )?;
        
        // Try to mine a block if we have enough transactions
        if self.blockchain.pending_transactions.len() >= 10 {
            self.blockchain.mine_block("shard_miner".to_string())?;
        }
        
        shard_tx.status = ShardTransactionStatus::Committed;
        
        // Update pending transactions
        {
            let mut pending = self.pending_transactions.write().unwrap();
            if let Some(pos) = pending.iter().position(|tx| tx.transaction.id == shard_tx.transaction.id) {
                pending[pos] = shard_tx;
            }
        }
        
        Ok(())
    }

    /// Process a cross-shard transaction
    fn process_cross_shard_transaction(&mut self, shard_tx: ShardTransaction) -> Result<()> {
        let cross_shard_id = shard_tx.cross_shard_id.clone()
            .ok_or_else(|| BlockchainError::InvalidTransaction("Missing cross-shard ID".to_string()))?;
        
        let target_shard = shard_tx.target_shard
            .ok_or_else(|| BlockchainError::InvalidTransaction("Missing target shard".to_string()))?;
        
        // Create or update cross-shard transaction
        {
            let mut cross_shard_txs = self.cross_shard_transactions.write().unwrap();
            let cross_shard_tx = cross_shard_txs.entry(cross_shard_id.clone()).or_insert_with(|| {
                CrossShardTransaction {
                    id: cross_shard_id.clone(),
                    source_shard: shard_tx.source_shard,
                    target_shard,
                    transaction: shard_tx.transaction.clone(),
                    status: CrossShardStatus::Preparing,
                    created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    participants: vec![shard_tx.source_shard, target_shard],
                }
            });
            
            // Update status based on current phase
            match cross_shard_tx.status {
                CrossShardStatus::Preparing => {
                    cross_shard_tx.status = CrossShardStatus::Prepared;
                }
                CrossShardStatus::Prepared => {
                    cross_shard_tx.status = CrossShardStatus::Committing;
                }
                CrossShardStatus::Committing => {
                    cross_shard_tx.status = CrossShardStatus::Committed;
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    /// Get shard statistics
    pub fn get_stats(&self) -> ShardStats {
        let pending_count = self.pending_transactions.read().unwrap().len();
        let cross_shard_count = self.cross_shard_transactions.read().unwrap().len();
        let blockchain_stats = self.blockchain.get_stats();
        
        ShardStats {
            shard_id: self.id,
            pending_transactions: pending_count,
            cross_shard_transactions: cross_shard_count,
            total_blocks: blockchain_stats.block_count as u64,
            total_transactions: blockchain_stats.total_transactions as u64,
            current_difficulty: blockchain_stats.difficulty,
        }
    }

    /// Get all pending transactions for this shard
    pub fn get_pending_transactions(&self) -> Vec<ShardTransaction> {
        self.pending_transactions.read().unwrap().clone()
    }

    /// Get all cross-shard transactions for this shard
    pub fn get_cross_shard_transactions(&self) -> Vec<CrossShardTransaction> {
        self.cross_shard_transactions.read().unwrap().values().cloned().collect()
    }
}

/// Statistics for a shard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStats {
    /// Shard ID
    pub shard_id: u32,
    /// Number of pending transactions
    pub pending_transactions: usize,
    /// Number of cross-shard transactions
    pub cross_shard_transactions: usize,
    /// Total blocks in this shard
    pub total_blocks: u64,
    /// Total transactions in this shard
    pub total_transactions: u64,
    /// Current mining difficulty
    pub current_difficulty: u32,
}

/// Manager for all shards in the system
#[derive(Debug)]
pub struct ShardManager {
    /// All shards in the system
    pub shards: DashMap<u32, Arc<RwLock<Shard>>>,
    /// Consensus type for all shards
    pub consensus_type: ConsensusType,
    /// Cross-shard transaction coordinator
    pub cross_shard_coordinator: Arc<RwLock<CrossShardCoordinator>>,
}

impl ShardManager {
    /// Create a new shard manager
    pub fn new(consensus_type: ConsensusType) -> Result<Self> {
        let shards = DashMap::new();
        let cross_shard_coordinator = Arc::new(RwLock::new(CrossShardCoordinator::new()));
        
        // Create all shards
        for shard_id in 0..NUM_SHARDS {
            let shard = Shard::new(shard_id, consensus_type)?;
            shards.insert(shard_id, Arc::new(RwLock::new(shard)));
        }
        
        Ok(Self {
            shards,
            consensus_type,
            cross_shard_coordinator,
        })
    }

    /// Assign a transaction to the appropriate shard
    pub fn assign_transaction(&self, transaction: Transaction) -> Result<u32> {
        let shard_id = self.calculate_shard_id(&transaction.sender);
        debug!("Assigned transaction {} to shard {}", transaction.id, shard_id);
        Ok(shard_id)
    }

    /// Calculate which shard a transaction should be assigned to
    pub fn calculate_shard_id(&self, sender: &str) -> u32 {
        let mut hasher = Sha256::new();
        hasher.update(sender.as_bytes());
        let result = hasher.finalize();
        
        // Use the first 4 bytes to determine shard
        u32::from_be_bytes([
            result[0], result[1], result[2], result[3]
        ]) % NUM_SHARDS
    }

    /// Process a transaction in the appropriate shard
    pub fn process_transaction(&self, transaction: Transaction) -> Result<()> {
        let shard_id = self.assign_transaction(transaction.clone())?;
        
        // Check if this is a cross-shard transaction
        let target_shard_id = self.calculate_shard_id(&transaction.receiver);
        let is_cross_shard = shard_id != target_shard_id;
        
        let shard_tx = ShardTransaction {
            transaction,
            source_shard: shard_id,
            target_shard: if is_cross_shard { Some(target_shard_id) } else { None },
            cross_shard_id: if is_cross_shard {
                Some(format!("cross_{}_{}_{}", shard_id, target_shard_id, 
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()))
            } else {
                None
            },
            status: ShardTransactionStatus::Pending,
            assigned_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        };
        
        // Get the shard and process the transaction
        if let Some(shard_arc) = self.shards.get(&shard_id) {
            let mut shard = shard_arc.write().unwrap();
            shard.process_transaction(shard_tx.clone())?;
        } else {
            return Err(BlockchainError::InvalidTransaction(
                format!("Shard {} not found", shard_id)
            ));
        }
        
        // If it's a cross-shard transaction, also process it in the target shard
        if is_cross_shard {
            if let Some(target_shard_arc) = self.shards.get(&target_shard_id) {
                let mut target_shard = target_shard_arc.write().unwrap();
                let mut target_shard_tx = shard_tx.clone();
                target_shard_tx.source_shard = target_shard_id;
                target_shard_tx.target_shard = Some(shard_id);
                target_shard.process_transaction(target_shard_tx)?;
            }
        }
        
        Ok(())
    }

    /// Get statistics for all shards
    pub fn get_all_stats(&self) -> Vec<ShardStats> {
        let mut stats = Vec::new();
        
        for shard_entry in self.shards.iter() {
            let shard = shard_entry.value().read().unwrap();
            stats.push(shard.get_stats());
        }
        
        stats
    }

    /// Get a specific shard
    pub fn get_shard(&self, shard_id: u32) -> Option<Arc<RwLock<Shard>>> {
        self.shards.get(&shard_id).map(|entry| entry.value().clone())
    }

    /// Mine blocks in all shards
    pub fn mine_all_shards(&self) -> Result<Vec<u32>> {
        let mut mined_shards = Vec::new();
        
        for shard_entry in self.shards.iter() {
            let shard_id = *shard_entry.key();
            let mut shard = shard_entry.value().write().unwrap();
            
            if !shard.blockchain.pending_transactions.is_empty() {
                shard.blockchain.mine_block("shard_miner".to_string())?;
                mined_shards.push(shard_id);
                info!("Mined block in shard {}", shard_id);
            }
        }
        
        Ok(mined_shards)
    }

    /// Get all pending transactions across all shards
    pub fn get_all_pending_transactions(&self) -> Vec<ShardTransaction> {
        let mut all_transactions = Vec::new();
        
        for shard_entry in self.shards.iter() {
            let shard = shard_entry.value().read().unwrap();
            all_transactions.extend(shard.get_pending_transactions());
        }
        
        all_transactions
    }

    /// Get all cross-shard transactions across all shards
    pub fn get_all_cross_shard_transactions(&self) -> Vec<CrossShardTransaction> {
        let mut all_cross_shard = Vec::new();
        
        for shard_entry in self.shards.iter() {
            let shard = shard_entry.value().read().unwrap();
            all_cross_shard.extend(shard.get_cross_shard_transactions());
        }
        
        all_cross_shard
    }
}

/// Coordinator for cross-shard transactions
#[derive(Debug)]
pub struct CrossShardCoordinator {
    /// Active cross-shard transactions
    pub active_transactions: HashMap<String, CrossShardTransaction>,
    /// Transaction status tracking
    pub transaction_status: HashMap<String, CrossShardStatus>,
}

impl Default for CrossShardCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossShardCoordinator {
    /// Create a new cross-shard coordinator
    pub fn new() -> Self {
        Self {
            active_transactions: HashMap::new(),
            transaction_status: HashMap::new(),
        }
    }

    /// Register a new cross-shard transaction
    pub fn register_transaction(&mut self, transaction: CrossShardTransaction) {
        self.active_transactions.insert(transaction.id.clone(), transaction.clone());
        self.transaction_status.insert(transaction.id.clone(), transaction.status);
    }

    /// Update transaction status
    pub fn update_status(&mut self, transaction_id: &str, status: CrossShardStatus) {
        if let Some(tx) = self.active_transactions.get_mut(transaction_id) {
            tx.status = status.clone();
        }
        self.transaction_status.insert(transaction_id.to_string(), status);
    }

    /// Get transaction status
    pub fn get_status(&self, transaction_id: &str) -> Option<&CrossShardStatus> {
        self.transaction_status.get(transaction_id)
    }

    /// Get all active cross-shard transactions
    pub fn get_active_transactions(&self) -> Vec<&CrossShardTransaction> {
        self.active_transactions.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::ConsensusType;

    #[test]
    fn test_shard_assignment() {
        let manager = ShardManager::new(ConsensusType::ProofOfWork).unwrap();
        
        // Test that same sender always goes to same shard
        let shard1 = manager.calculate_shard_id("alice");
        let shard2 = manager.calculate_shard_id("alice");
        assert_eq!(shard1, shard2);
        
        // Test that different senders can go to different shards
        let _shard3 = manager.calculate_shard_id("bob");
        // Note: This might be the same shard due to hash collision, but that's okay
    }

    #[test]
    fn test_shard_creation() {
        let shard = Shard::new(0, ConsensusType::ProofOfWork).unwrap();
        assert_eq!(shard.id, 0);
        assert_eq!(shard.get_stats().pending_transactions, 0);
        
        // Note: Cleanup not needed for tests as they use unique paths
    }

    #[test]
    fn test_shard_manager_creation() {
        let manager = ShardManager::new(ConsensusType::ProofOfWork).unwrap();
        assert_eq!(manager.shards.len(), NUM_SHARDS as usize);
        
        for i in 0..NUM_SHARDS {
            assert!(manager.shards.contains_key(&i));
        }
        
        // Note: Cleanup not needed for tests as they use unique paths
    }
}
