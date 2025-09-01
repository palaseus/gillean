// Advanced Sharding Test Suite
// Tests for dynamic shard allocation and rebalancing

use gillean::{Result, Blockchain, Transaction, BlockchainError};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Shard {
    pub id: String,
    pub nodes: Vec<String>,
    pub transactions: Vec<Transaction>,
    pub load: f64, // Current load percentage
    pub capacity: f64, // Maximum capacity
    pub state_root: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct CrossShardTransaction {
    pub id: String,
    pub from_shard: String,
    pub to_shard: String,
    pub transaction: Transaction,
    pub status: CrossShardStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CrossShardStatus {
    Pending,
    Committed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ShardingManager {
    pub shards: HashMap<String, Shard>,
    pub cross_shard_txs: HashMap<String, CrossShardTransaction>,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub total_shards: usize,
    pub rebalance_threshold: f64,
}

impl ShardingManager {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>, initial_shards: usize) -> Self {
        let mut shards = HashMap::new();
        
        for i in 0..initial_shards {
            let shard_id = format!("shard_{}", i);
            shards.insert(shard_id.clone(), Shard {
                id: shard_id,
                nodes: vec![format!("node_{}_0", i), format!("node_{}_1", i)],
                transactions: Vec::new(),
                load: 0.0,
                capacity: 100.0,
                state_root: format!("root_{}", i),
                is_active: true,
            });
        }

        Self {
            shards,
            cross_shard_txs: HashMap::new(),
            blockchain,
            total_shards: initial_shards,
            rebalance_threshold: 0.8, // 80% load threshold
        }
    }

    pub fn add_shard(&mut self) -> Result<String> {
        let shard_id = format!("shard_{}", self.total_shards);
        
        let new_shard = Shard {
            id: shard_id.clone(),
            nodes: vec![format!("node_{}_0", self.total_shards), format!("node_{}_1", self.total_shards)],
            transactions: Vec::new(),
            load: 0.0,
            capacity: 100.0,
            state_root: format!("root_{}", self.total_shards),
            is_active: true,
        };

        self.shards.insert(shard_id.clone(), new_shard);
        self.total_shards += 1;
        
        Ok(shard_id)
    }

    pub fn remove_shard(&mut self, shard_id: &str) -> Result<()> {
        let shard = self.shards.get(shard_id)
            .ok_or_else(|| BlockchainError::InvalidInput("Shard not found".to_string()))?;

        if shard.load > 0.0 {
            return Err(BlockchainError::InvalidInput("Cannot remove shard with active load".to_string()));
        }

        // Redistribute nodes to other shards
        self.redistribute_nodes(shard_id)?;
        
        self.shards.remove(shard_id);
        self.total_shards -= 1;
        
        Ok(())
    }

    pub fn add_transaction_to_shard(&mut self, shard_id: &str, transaction: Transaction) -> Result<()> {
        let shard = self.shards.get_mut(shard_id)
            .ok_or_else(|| BlockchainError::InvalidInput("Shard not found".to_string()))?;

        if !shard.is_active {
            return Err(BlockchainError::InvalidInput("Shard is not active".to_string()));
        }

        // Calculate new load
        let new_load = (shard.transactions.len() as f64 + 1.0) / shard.capacity;
        
        if new_load > 1.0 {
            return Err(BlockchainError::InvalidInput("Shard capacity exceeded".to_string()));
        }

        shard.transactions.push(transaction);
        shard.load = new_load;

        // Check if rebalancing is needed
        if shard.load > self.rebalance_threshold {
            self.trigger_rebalancing(shard_id)?;
        }

        Ok(())
    }

    pub fn create_cross_shard_transaction(&mut self, from_shard: &str, to_shard: &str, transaction: Transaction) -> Result<String> {
        if from_shard == to_shard {
            return Err(BlockchainError::InvalidInput("Cross-shard transaction must be between different shards".to_string()));
        }

        if !self.shards.contains_key(from_shard) || !self.shards.contains_key(to_shard) {
            return Err(BlockchainError::InvalidInput("Invalid shard IDs".to_string()));
        }

        let tx_id = format!("cross_tx_{}", uuid::Uuid::new_v4());
        
        let cross_tx = CrossShardTransaction {
            id: tx_id.clone(),
            from_shard: from_shard.to_string(),
            to_shard: to_shard.to_string(),
            transaction,
            status: CrossShardStatus::Pending,
        };

        self.cross_shard_txs.insert(tx_id.clone(), cross_tx);
        
        Ok(tx_id)
    }

    pub fn commit_cross_shard_transaction(&mut self, tx_id: &str) -> Result<()> {
        // Get transaction data first
        let (to_shard, transaction) = {
            let cross_tx = self.cross_shard_txs.get(tx_id)
                .ok_or_else(|| BlockchainError::InvalidInput("Cross-shard transaction not found".to_string()))?;

            if cross_tx.status != CrossShardStatus::Pending {
                return Err(BlockchainError::InvalidInput("Transaction is not pending".to_string()));
            }

            (cross_tx.to_shard.clone(), cross_tx.transaction.clone())
        };

        // Simulate two-phase commit
        // Phase 1: Prepare
        self.prepare_cross_shard_tx_simple(tx_id)?;
        
        // Phase 2: Commit
        if let Some(cross_tx) = self.cross_shard_txs.get_mut(tx_id) {
            cross_tx.status = CrossShardStatus::Committed;
        }
        
        // Add transaction to destination shard
        self.add_transaction_to_shard(&to_shard, transaction)?;

        Ok(())
    }

    pub fn get_shard_load_distribution(&self) -> HashMap<String, f64> {
        let mut distribution = HashMap::new();
        for (shard_id, shard) in &self.shards {
            distribution.insert(shard_id.clone(), shard.load);
        }
        distribution
    }

    pub fn rebalance_shards(&mut self) -> Result<()> {
        println!("🔄 Starting shard rebalancing...");
        
        let distribution = self.get_shard_load_distribution();
        let avg_load: f64 = distribution.values().sum::<f64>() / distribution.len() as f64;
        
        let mut overloaded_shards = Vec::new();
        let mut underloaded_shards = Vec::new();

        for (shard_id, load) in distribution {
            if load > avg_load * 1.2 {
                overloaded_shards.push(shard_id);
            } else if load < avg_load * 0.8 {
                underloaded_shards.push(shard_id);
            }
        }

        // Redistribute transactions
        for overloaded_id in overloaded_shards {
            if let Some(underloaded_id) = underloaded_shards.pop() {
                self.redistribute_transactions(&overloaded_id, &underloaded_id)?;
            }
        }

        println!("✅ Shard rebalancing completed");
        Ok(())
    }

    fn trigger_rebalancing(&mut self, shard_id: &str) -> Result<()> {
        println!("⚠️  High load detected on shard {}, triggering rebalancing", shard_id);
        self.rebalance_shards()
    }

    fn redistribute_nodes(&mut self, removed_shard_id: &str) -> Result<()> {
        // Simplified node redistribution
        // In a real implementation, this would involve complex node assignment logic
        println!("🔄 Redistributing nodes from shard {}", removed_shard_id);
        Ok(())
    }

    fn redistribute_transactions(&mut self, from_shard: &str, to_shard: &str) -> Result<()> {
        // Get the transactions to move first
        let transactions_to_move = {
            let from_shard_data = self.shards.get(from_shard)
                .ok_or_else(|| BlockchainError::InvalidInput("Source shard not found".to_string()))?;
            (from_shard_data.transactions.len() as f64 * 0.3) as usize
        };
        
        if transactions_to_move > 0 {
            // Move transactions
            let moved_txs: Vec<Transaction> = {
                let from_shard_data = self.shards.get_mut(from_shard)
                    .ok_or_else(|| BlockchainError::InvalidInput("Source shard not found".to_string()))?;
                
                if !from_shard_data.transactions.is_empty() {
                    from_shard_data.transactions
                        .drain(..std::cmp::min(transactions_to_move, from_shard_data.transactions.len()))
                        .collect()
                } else {
                    Vec::new()
                }
            };
            
            // Add to destination shard
            if !moved_txs.is_empty() {
                let to_shard_data = self.shards.get_mut(to_shard)
                    .ok_or_else(|| BlockchainError::InvalidInput("Destination shard not found".to_string()))?;
                to_shard_data.transactions.extend(moved_txs);
            }
            
            // Recalculate loads
            let from_shard_data = self.shards.get_mut(from_shard)
                .ok_or_else(|| BlockchainError::InvalidInput("Source shard not found".to_string()))?;
            from_shard_data.load = from_shard_data.transactions.len() as f64 / from_shard_data.capacity;
            
            let to_shard_data = self.shards.get_mut(to_shard)
                .ok_or_else(|| BlockchainError::InvalidInput("Destination shard not found".to_string()))?;
            to_shard_data.load = to_shard_data.transactions.len() as f64 / to_shard_data.capacity;
        }

        Ok(())
    }

    fn prepare_cross_shard_tx_simple(&self, tx_id: &str) -> Result<()> {
        // Simulate prepare phase of two-phase commit
        // In a real implementation, this would coordinate with all participating shards
        println!("🔄 Preparing cross-shard transaction {}", tx_id);
        Ok(())
    }
}

pub struct ShardingSuite {
    manager: ShardingManager,
}

impl ShardingSuite {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            manager: ShardingManager::new(blockchain, 4), // Start with 4 shards
        }
    }

    pub async fn test_shard_creation_and_removal(&self) -> Result<()> {
        println!("🧪 Testing shard creation and removal...");

        let mut manager = ShardingManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            2
        );

        // Test initial shards
        assert_eq!(manager.total_shards, 2);
        assert!(manager.shards.contains_key("shard_0"));
        assert!(manager.shards.contains_key("shard_1"));

        // Add a new shard
        let new_shard_id = manager.add_shard()?;
        assert_eq!(manager.total_shards, 3);
        assert!(manager.shards.contains_key(&new_shard_id));

        // Remove an empty shard
        manager.remove_shard("shard_0")?;
        assert_eq!(manager.total_shards, 2);
        assert!(!manager.shards.contains_key("shard_0"));

        println!("✅ Shard creation and removal test passed!");
        Ok(())
    }

    pub async fn test_dynamic_load_distribution(&self) -> Result<()> {
        println!("🧪 Testing dynamic load distribution...");

        let mut manager = ShardingManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            3
        );

        // Add transactions to create load
        for i in 0..50 {
            let tx = Transaction::new_transfer(
                format!("user_{}", i),
                format!("user_{}", i + 1),
                10.0,
                Some(format!("tx_{}", i))
            )?;
            
            let shard_id = format!("shard_{}", i % 3);
            manager.add_transaction_to_shard(&shard_id, tx)?;
        }

        // Check load distribution
        let distribution = manager.get_shard_load_distribution();
        assert_eq!(distribution.len(), 3);
        
        // Verify loads are reasonable
        for (_, load) in &distribution {
            assert!(*load >= 0.0 && *load <= 1.0);
        }

        println!("✅ Dynamic load distribution test passed!");
        Ok(())
    }

    pub async fn test_cross_shard_transactions(&self) -> Result<()> {
        println!("🧪 Testing cross-shard transactions...");

        let mut manager = ShardingManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            2
        );

        // Create a cross-shard transaction
        let tx = Transaction::new_transfer("alice".to_string(), "bob".to_string(), 100.0, Some("cross_tx".to_string()))?;
        let cross_tx_id = manager.create_cross_shard_transaction("shard_0", "shard_1", tx)?;

        // Verify transaction was created
        assert!(manager.cross_shard_txs.contains_key(&cross_tx_id));
        let cross_tx = &manager.cross_shard_txs[&cross_tx_id];
        assert_eq!(cross_tx.status, CrossShardStatus::Pending);

        // Commit the transaction
        manager.commit_cross_shard_transaction(&cross_tx_id)?;
        
        // Verify transaction was committed
        let cross_tx = &manager.cross_shard_txs[&cross_tx_id];
        assert_eq!(cross_tx.status, CrossShardStatus::Committed);

        println!("✅ Cross-shard transactions test passed!");
        Ok(())
    }

    pub async fn test_shard_rebalancing(&self) -> Result<()> {
        println!("🧪 Testing shard rebalancing...");

        let mut manager = ShardingManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            3
        );

        // Overload shard_0
        for i in 0..90 {
            let tx = Transaction::new_transfer(
                format!("user_{}", i),
                format!("user_{}", i + 1),
                10.0,
                Some(format!("tx_{}", i))
            )?;
            
            manager.add_transaction_to_shard("shard_0", tx)?;
        }

        // Check initial load
        let initial_distribution = manager.get_shard_load_distribution();
        assert!(initial_distribution["shard_0"] > 0.8); // Should be overloaded

        // Trigger rebalancing
        manager.rebalance_shards()?;

        // Check final load distribution
        let final_distribution = manager.get_shard_load_distribution();
        let avg_load: f64 = final_distribution.values().sum::<f64>() / final_distribution.len() as f64;
        
        // Verify load is more balanced
        for (_, load) in &final_distribution {
            assert!((*load - avg_load).abs() < 0.3); // Should be more balanced
        }

        println!("✅ Shard rebalancing test passed!");
        Ok(())
    }

    pub async fn test_invalid_operations(&self) -> Result<()> {
        println!("🧪 Testing invalid operations...");

        let mut manager = ShardingManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            2
        );

        // Test adding transaction to non-existent shard
        let tx = Transaction::new_transfer("alice".to_string(), "bob".to_string(), 10.0, Some("tx".to_string()))?;
        let result = manager.add_transaction_to_shard("non_existent", tx);
        assert!(result.is_err());

        // Test cross-shard transaction within same shard
        let tx = Transaction::new_transfer("alice".to_string(), "bob".to_string(), 10.0, Some("tx".to_string()))?;
        let result = manager.create_cross_shard_transaction("shard_0", "shard_0", tx);
        assert!(result.is_err());

        // Test removing shard with load
        for i in 0..10 {
            let tx = Transaction::new_transfer(
                format!("user_{}", i),
                format!("user_{}", i + 1),
                10.0,
                Some(format!("tx_{}", i))
            )?;
            manager.add_transaction_to_shard("shard_0", tx)?;
        }
        
        let result = manager.remove_shard("shard_0");
        assert!(result.is_err());

        println!("✅ Invalid operations test passed!");
        Ok(())
    }

    pub async fn test_load_monitoring(&self) -> Result<()> {
        println!("🧪 Testing load monitoring...");

        let mut manager = ShardingManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            4
        );

        // Add varying loads to different shards
        for i in 0..30 {
            let tx = Transaction::new_transfer(
                format!("user_{}", i),
                format!("user_{}", i + 1),
                10.0,
                Some(format!("tx_{}", i))
            )?;
            
            let shard_id = format!("shard_{}", i % 4);
            manager.add_transaction_to_shard(&shard_id, tx)?;
        }

        // Monitor load distribution
        let distribution = manager.get_shard_load_distribution();
        
        // Verify monitoring data
        assert_eq!(distribution.len(), 4);
        let total_load: f64 = distribution.values().sum();
        assert!(total_load > 0.0);

        // Check individual shard loads
        for (shard_id, load) in &distribution {
            println!("Shard {}: {:.2}% load", shard_id, load * 100.0);
            assert!(*load >= 0.0 && *load <= 1.0);
        }

        println!("✅ Load monitoring test passed!");
        Ok(())
    }

    pub async fn run_all_tests(&self) -> Result<()> {
        println!("🚀 Running Advanced Sharding test suite...");
        
        self.test_shard_creation_and_removal().await?;
        self.test_dynamic_load_distribution().await?;
        self.test_cross_shard_transactions().await?;
        self.test_shard_rebalancing().await?;
        self.test_invalid_operations().await?;
        self.test_load_monitoring().await?;

        println!("✅ All Advanced Sharding tests passed!");
        Ok(())
    }
}
