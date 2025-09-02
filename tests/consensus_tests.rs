// Advanced Consensus Test Suite
// Tests for DPoS, PBFT, and hybrid consensus mechanisms

use gillean::{Result, Blockchain, BlockchainError};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusType {
    PoW,
    PoS,
    DPoS,
    PBFT,
    Hybrid,
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub id: String,
    pub address: String,
    pub stake: f64,
    pub delegated_stake: f64,
    pub is_active: bool,
    pub reputation_score: f64,
    pub last_block_time: u64,
    pub total_blocks_produced: u64,
}

#[derive(Debug, Clone)]
pub struct Delegator {
    pub id: String,
    pub address: String,
    pub stake: f64,
    pub delegated_to: String,
    pub rewards_earned: f64,
}

#[derive(Debug, Clone)]
pub enum PBFTMessage {
    PrePrepare { view: u64, sequence: u64, block_hash: String },
    Prepare { view: u64, sequence: u64, block_hash: String, validator: String },
    Commit { view: u64, sequence: u64, block_hash: String, validator: String },
}

#[derive(Debug, Clone)]
pub struct PBFTState {
    pub view: u64,
    pub sequence: u64,
    pub primary: String,
    pub validators: Vec<String>,
    pub prepared: HashMap<String, u64>, // block_hash -> sequence
    pub committed: HashMap<String, u64>, // block_hash -> sequence
    pub messages: Vec<PBFTMessage>,
}

#[derive(Debug, Clone)]
pub struct ConsensusManager {
    pub consensus_type: ConsensusType,
    pub validators: HashMap<String, Validator>,
    pub delegators: HashMap<String, Delegator>,
    pub pbft_state: Option<PBFTState>,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub min_stake: f64,
    pub max_validators: usize,
    pub block_time: u64,
    pub view_timeout: u64,
}

impl ConsensusManager {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>, consensus_type: ConsensusType) -> Self {
        Self {
            consensus_type,
            validators: HashMap::new(),
            delegators: HashMap::new(),
            pbft_state: None,
            blockchain,
            min_stake: 1000.0,
            max_validators: 21,
            block_time: 5000, // 5 seconds
            view_timeout: 10000, // 10 seconds
        }
    }

    pub fn register_validator(&mut self, address: String, stake: f64) -> Result<()> {
        if stake < self.min_stake {
            return Err(BlockchainError::InvalidInput(
                format!("Stake must be at least {}", self.min_stake)
            ));
        }

        if self.validators.len() >= self.max_validators {
            return Err(BlockchainError::InvalidInput(
                "Maximum number of validators reached".to_string()
            ));
        }

        let validator = Validator {
            id: format!("validator_{}", uuid::Uuid::new_v4()),
            address: address.clone(),
            stake,
            delegated_stake: 0.0,
            is_active: true,
            reputation_score: 1.0,
            last_block_time: 0,
            total_blocks_produced: 0,
        };

        self.validators.insert(address, validator);
        Ok(())
    }

    pub fn delegate_stake(&mut self, delegator_address: String, validator_address: String, amount: f64) -> Result<()> {
        if !self.validators.contains_key(&validator_address) {
            return Err(BlockchainError::InvalidInput("Validator not found".to_string()));
        }

        let validator = self.validators.get_mut(&validator_address).unwrap();
        validator.delegated_stake += amount;

        let delegator = Delegator {
            id: format!("delegator_{}", uuid::Uuid::new_v4()),
            address: delegator_address.clone(),
            stake: amount,
            delegated_to: validator_address,
            rewards_earned: 0.0,
        };

        self.delegators.insert(delegator_address, delegator);
        Ok(())
    }

    pub fn select_validator(&self) -> Option<String> {
        match self.consensus_type {
            ConsensusType::DPoS => self.select_dpos_validator(),
            ConsensusType::PBFT => self.select_pbft_primary(),
            _ => None,
        }
    }

    pub fn propose_block(&mut self, validator_address: &str, transactions: Vec<String>) -> Result<String> {
        match self.consensus_type {
            ConsensusType::DPoS => self.propose_dpos_block(validator_address, transactions),
            ConsensusType::PBFT => self.propose_pbft_block(validator_address, transactions),
            _ => Err(BlockchainError::InvalidInput("Unsupported consensus type".to_string())),
        }
    }

    pub fn validate_block(&mut self, block_hash: &str, validator_address: &str) -> Result<bool> {
        match self.consensus_type {
            ConsensusType::DPoS => self.validate_dpos_block(block_hash, validator_address),
            ConsensusType::PBFT => self.validate_pbft_block(block_hash, validator_address),
            _ => Err(BlockchainError::InvalidInput("Unsupported consensus type".to_string())),
        }
    }

    pub fn finalize_block(&mut self, block_hash: &str) -> Result<()> {
        match self.consensus_type {
            ConsensusType::DPoS => self.finalize_dpos_block(block_hash),
            ConsensusType::PBFT => self.finalize_pbft_block(block_hash),
            _ => Err(BlockchainError::InvalidInput("Unsupported consensus type".to_string())),
        }
    }

    pub fn distribute_rewards(&mut self) -> Result<()> {
        let total_rewards = 100.0; // Fixed reward per block
        let mut validator_rewards = HashMap::new();

        // Calculate rewards based on stake and performance
        for (address, validator) in &self.validators {
            if validator.is_active {
                let total_stake = validator.stake + validator.delegated_stake;
                let reward_share = total_stake / self.get_total_active_stake();
                let reward = total_rewards * reward_share * validator.reputation_score;
                validator_rewards.insert(address.clone(), reward);
            }
        }

        // Distribute rewards to validators
        for (address, reward) in &validator_rewards {
            if let Some(_validator) = self.validators.get_mut(address) {
                // In a real implementation, this would transfer tokens
                println!("Validator {} earned {} rewards", address, reward);
            }
        }

        // Distribute rewards to delegators
        for (address, delegator) in &mut self.delegators {
            if let Some(validator) = self.validators.get(&delegator.delegated_to) {
                let reward_share = delegator.stake / (validator.stake + validator.delegated_stake);
                let reward = validator_rewards.get(&delegator.delegated_to).unwrap_or(&0.0) * reward_share;
                delegator.rewards_earned += reward;
                println!("Delegator {} earned {} rewards", address, reward);
            }
        }

        Ok(())
    }

    fn select_dpos_validator(&self) -> Option<String> {
        // Select validator based on stake and reputation
        let mut best_validator = None;
        let mut best_score = 0.0;

        for (address, validator) in &self.validators {
            if validator.is_active {
                let total_stake = validator.stake + validator.delegated_stake;
                let score = total_stake * validator.reputation_score;
                if score > best_score {
                    best_score = score;
                    best_validator = Some(address.clone());
                }
            }
        }

        best_validator
    }

    fn select_pbft_primary(&self) -> Option<String> {
        self.pbft_state.as_ref().map(|pbft_state| pbft_state.primary.clone())
    }

    fn propose_dpos_block(&mut self, validator_address: &str, _transactions: Vec<String>) -> Result<String> {
        if !self.validators.contains_key(validator_address) {
            return Err(BlockchainError::InvalidInput("Validator not found".to_string()));
        }

        let validator = self.validators.get_mut(validator_address).unwrap();
        if !validator.is_active {
            return Err(BlockchainError::InvalidInput("Validator is not active".to_string()));
        }

        // Create block hash (simplified)
        let block_hash = format!("block_{}", uuid::Uuid::new_v4());
        
        // Update validator stats
        validator.last_block_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        validator.total_blocks_produced += 1;

        println!("DPoS block proposed by {}: {}", validator_address, block_hash);
        Ok(block_hash)
    }

    fn propose_pbft_block(&mut self, validator_address: &str, _transactions: Vec<String>) -> Result<String> {
        if let Some(pbft_state) = &mut self.pbft_state {
            if pbft_state.primary != validator_address {
                return Err(BlockchainError::InvalidInput("Only primary can propose blocks".to_string()));
            }

            let block_hash = format!("block_{}", uuid::Uuid::new_v4());
            pbft_state.sequence += 1;

            // Send pre-prepare message
            let pre_prepare = PBFTMessage::PrePrepare {
                view: pbft_state.view,
                sequence: pbft_state.sequence,
                block_hash: block_hash.clone(),
            };
            pbft_state.messages.push(pre_prepare);

            println!("PBFT block proposed by primary {}: {}", validator_address, block_hash);
            Ok(block_hash)
        } else {
            Err(BlockchainError::InvalidInput("PBFT state not initialized".to_string()))
        }
    }

    fn validate_dpos_block(&mut self, _block_hash: &str, validator_address: &str) -> Result<bool> {
        if !self.validators.contains_key(validator_address) {
            return Err(BlockchainError::InvalidInput("Validator not found".to_string()));
        }

        // Simulate block validation
        let is_valid = true; // In real implementation, this would validate the block
        
        if is_valid {
            let validator = self.validators.get_mut(validator_address).unwrap();
            validator.reputation_score = (validator.reputation_score + 0.1).min(1.0);
        }

        Ok(is_valid)
    }

    fn validate_pbft_block(&mut self, block_hash: &str, validator_address: &str) -> Result<bool> {
        if let Some(pbft_state) = &mut self.pbft_state {
            if !pbft_state.validators.contains(&validator_address.to_string()) {
                return Err(BlockchainError::InvalidInput("Validator not in PBFT set".to_string()));
            }

            // Simulate block validation
            let is_valid = true;

            if is_valid {
                // Send prepare message
                let prepare = PBFTMessage::Prepare {
                    view: pbft_state.view,
                    sequence: pbft_state.sequence,
                    block_hash: block_hash.to_string(),
                    validator: validator_address.to_string(),
                };
                pbft_state.messages.push(prepare);

                // Check if we have enough prepare messages
                let prepare_count = pbft_state.messages.iter()
                    .filter(|msg| {
                        if let PBFTMessage::Prepare { view, sequence, block_hash: msg_hash, .. } = msg {
                            *view == pbft_state.view && *sequence == pbft_state.sequence && msg_hash == block_hash
                        } else {
                            false
                        }
                    })
                    .count();

                if prepare_count > (2 * pbft_state.validators.len() / 3) {
                    pbft_state.prepared.insert(block_hash.to_string(), pbft_state.sequence);
                }
            }

            Ok(is_valid)
        } else {
            Err(BlockchainError::InvalidInput("PBFT state not initialized".to_string()))
        }
    }

    fn finalize_dpos_block(&mut self, block_hash: &str) -> Result<()> {
        println!("DPoS block finalized: {}", block_hash);
        
        // Distribute rewards
        self.distribute_rewards()?;
        
        Ok(())
    }

    fn finalize_pbft_block(&mut self, block_hash: &str) -> Result<()> {
        if let Some(pbft_state) = &mut self.pbft_state {
            if pbft_state.prepared.contains_key(block_hash) {
                // Send commit messages from all validators
                for validator in &pbft_state.validators {
                    let commit = PBFTMessage::Commit {
                        view: pbft_state.view,
                        sequence: pbft_state.sequence,
                        block_hash: block_hash.to_string(),
                        validator: validator.clone(),
                    };
                    pbft_state.messages.push(commit);
                }

                // Check if we have enough commit messages
                let commit_count = pbft_state.messages.iter()
                    .filter(|msg| {
                        if let PBFTMessage::Commit { view, sequence, block_hash: msg_hash, .. } = msg {
                            *view == pbft_state.view && *sequence == pbft_state.sequence && msg_hash == block_hash
                        } else {
                            false
                        }
                    })
                    .count();

                if commit_count > (2 * pbft_state.validators.len() / 3) {
                    pbft_state.committed.insert(block_hash.to_string(), pbft_state.sequence);
                    println!("PBFT block finalized: {}", block_hash);
                } else {
                    return Err(BlockchainError::InvalidInput("Insufficient commit messages".to_string()));
                }
            } else {
                return Err(BlockchainError::InvalidInput("Block not prepared".to_string()));
            }
        } else {
            return Err(BlockchainError::InvalidInput("PBFT state not initialized".to_string()));
        }

        Ok(())
    }

    fn get_total_active_stake(&self) -> f64 {
        self.validators.values()
            .filter(|v| v.is_active)
            .map(|v| v.stake + v.delegated_stake)
            .sum()
    }

    pub fn initialize_pbft(&mut self, validators: Vec<String>) -> Result<()> {
        if validators.len() < 4 {
            return Err(BlockchainError::InvalidInput("PBFT requires at least 4 validators".to_string()));
        }

        let primary = validators[0].clone();
        let pbft_state = PBFTState {
            view: 0,
            sequence: 0,
            primary,
            validators,
            prepared: HashMap::new(),
            committed: HashMap::new(),
            messages: Vec::new(),
        };

        self.pbft_state = Some(pbft_state);
        Ok(())
    }

    pub fn change_view(&mut self) -> Result<()> {
        if let Some(pbft_state) = &mut self.pbft_state {
            pbft_state.view += 1;
            let new_primary_index = (pbft_state.view as usize) % pbft_state.validators.len();
            pbft_state.primary = pbft_state.validators[new_primary_index].clone();
            println!("PBFT view changed to {}, new primary: {}", pbft_state.view, pbft_state.primary);
        }
        Ok(())
    }
}

pub struct ConsensusSuite {
    _manager: ConsensusManager,
}

impl ConsensusSuite {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            _manager: ConsensusManager::new(blockchain, ConsensusType::DPoS),
        }
    }

    pub async fn test_dpos_validator_registration(&self) -> Result<()> {
        println!("🧪 Testing DPoS validator registration...");

        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            ConsensusType::DPoS
        );

        // Register validators
        manager.register_validator("validator1".to_string(), 5000.0)?;
        manager.register_validator("validator2".to_string(), 3000.0)?;
        manager.register_validator("validator3".to_string(), 2000.0)?;

        assert_eq!(manager.validators.len(), 3);
        assert!(manager.validators.contains_key("validator1"));
        assert!(manager.validators.contains_key("validator2"));
        assert!(manager.validators.contains_key("validator3"));

        println!("✅ DPoS validator registration test passed!");
        Ok(())
    }

    pub async fn test_dpos_delegation(&self) -> Result<()> {
        println!("🧪 Testing DPoS delegation...");

        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            ConsensusType::DPoS
        );

        // Register validator
        manager.register_validator("validator1".to_string(), 5000.0)?;

        // Delegate stake
        manager.delegate_stake("delegator1".to_string(), "validator1".to_string(), 1000.0)?;
        manager.delegate_stake("delegator2".to_string(), "validator1".to_string(), 500.0)?;

        assert_eq!(manager.delegators.len(), 2);
        assert_eq!(manager.validators["validator1"].delegated_stake, 1500.0);

        println!("✅ DPoS delegation test passed!");
        Ok(())
    }

    pub async fn test_dpos_block_proposal(&self) -> Result<()> {
        println!("🧪 Testing DPoS block proposal...");

        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            ConsensusType::DPoS
        );

        // Register validator
        manager.register_validator("validator1".to_string(), 5000.0)?;

        // Propose block
        let transactions = vec!["tx1".to_string(), "tx2".to_string()];
        let block_hash = manager.propose_block("validator1", transactions)?;

        assert!(!block_hash.is_empty());
        assert_eq!(manager.validators["validator1"].total_blocks_produced, 1);

        println!("✅ DPoS block proposal test passed!");
        Ok(())
    }

    pub async fn test_pbft_initialization(&self) -> Result<()> {
        println!("🧪 Testing PBFT initialization...");

        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            ConsensusType::PBFT
        );

        let validators = vec![
            "validator1".to_string(),
            "validator2".to_string(),
            "validator3".to_string(),
            "validator4".to_string(),
        ];

        manager.initialize_pbft(validators)?;

        assert!(manager.pbft_state.is_some());
        let pbft_state = manager.pbft_state.as_ref().unwrap();
        assert_eq!(pbft_state.validators.len(), 4);
        assert_eq!(pbft_state.primary, "validator1");

        println!("✅ PBFT initialization test passed!");
        Ok(())
    }

    pub async fn test_pbft_block_proposal(&self) -> Result<()> {
        println!("🧪 Testing PBFT block proposal...");

        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            ConsensusType::PBFT
        );

        let validators = vec![
            "validator1".to_string(),
            "validator2".to_string(),
            "validator3".to_string(),
            "validator4".to_string(),
        ];

        manager.initialize_pbft(validators)?;

        // Propose block (only primary can propose)
        let transactions = vec!["tx1".to_string(), "tx2".to_string()];
        let block_hash = manager.propose_block("validator1", transactions.clone())?;

        assert!(!block_hash.is_empty());

        // Try to propose with non-primary validator (should fail)
        let result = manager.propose_block("validator2", transactions);
        assert!(result.is_err());

        println!("✅ PBFT block proposal test passed!");
        Ok(())
    }

    pub async fn test_pbft_consensus(&self) -> Result<()> {
        println!("🧪 Testing PBFT consensus...");

        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            ConsensusType::PBFT
        );

        let validators = vec![
            "validator1".to_string(),
            "validator2".to_string(),
            "validator3".to_string(),
            "validator4".to_string(),
        ];

        manager.initialize_pbft(validators)?;

        // Propose block
        let transactions = vec!["tx1".to_string(), "tx2".to_string()];
        let block_hash = manager.propose_block("validator1", transactions)?;

        // Validate block (prepare phase)
        for validator in &["validator1", "validator2", "validator3", "validator4"] {
            manager.validate_block(&block_hash, validator)?;
        }

        // Finalize block (commit phase)
        manager.finalize_block(&block_hash)?;

        let pbft_state = manager.pbft_state.as_ref().unwrap();
        assert!(pbft_state.committed.contains_key(&block_hash));

        println!("✅ PBFT consensus test passed!");
        Ok(())
    }

    pub async fn test_reward_distribution(&self) -> Result<()> {
        println!("🧪 Testing reward distribution...");

        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            ConsensusType::DPoS
        );

        // Register validators
        manager.register_validator("validator1".to_string(), 5000.0)?;
        manager.register_validator("validator2".to_string(), 3000.0)?;

        // Delegate stake
        manager.delegate_stake("delegator1".to_string(), "validator1".to_string(), 1000.0)?;

        // Distribute rewards
        manager.distribute_rewards()?;

        // Check that delegators earned rewards
        assert!(manager.delegators["delegator1"].rewards_earned > 0.0);

        println!("✅ Reward distribution test passed!");
        Ok(())
    }

    pub async fn test_view_change(&self) -> Result<()> {
        println!("🧪 Testing PBFT view change...");

        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            ConsensusType::PBFT
        );

        let validators = vec![
            "validator1".to_string(),
            "validator2".to_string(),
            "validator3".to_string(),
            "validator4".to_string(),
        ];

        manager.initialize_pbft(validators)?;

        let initial_primary = manager.pbft_state.as_ref().unwrap().primary.clone();
        
        // Change view
        manager.change_view()?;
        
        let new_primary = manager.pbft_state.as_ref().unwrap().primary.clone();
        assert_ne!(initial_primary, new_primary);

        println!("✅ PBFT view change test passed!");
        Ok(())
    }

    pub async fn test_invalid_operations(&self) -> Result<()> {
        println!("🧪 Testing invalid operations...");

        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            ConsensusType::DPoS
        );

        // Test registering validator with insufficient stake
        let result = manager.register_validator("validator1".to_string(), 100.0);
        assert!(result.is_err());

        // Test delegating to non-existent validator
        let result = manager.delegate_stake("delegator1".to_string(), "non_existent".to_string(), 1000.0);
        assert!(result.is_err());

        // Test PBFT with insufficient validators
        let mut pbft_manager = ConsensusManager::new(
            Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)),
            ConsensusType::PBFT
        );
        
        let result = pbft_manager.initialize_pbft(vec!["v1".to_string(), "v2".to_string(), "v3".to_string()]);
        assert!(result.is_err());

        println!("✅ Invalid operations test passed!");
        Ok(())
    }

    pub async fn run_all_tests(&self) -> Result<()> {
        println!("🚀 Running Advanced Consensus test suite...");
        
        self.test_dpos_validator_registration().await?;
        self.test_dpos_delegation().await?;
        self.test_dpos_block_proposal().await?;
        self.test_pbft_initialization().await?;
        self.test_pbft_block_proposal().await?;
        self.test_pbft_consensus().await?;
        self.test_reward_distribution().await?;
        self.test_view_change().await?;
        self.test_invalid_operations().await?;

        println!("✅ All Advanced Consensus tests passed!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_manager_creation() {
        let blockchain = Blockchain::new_pow(2, 50.0).unwrap();
        let _manager = ConsensusManager::new(
            Arc::new(Mutex::new(blockchain)),
            ConsensusType::DPoS
        );
        assert!(true); // Basic test to ensure manager can be created
    }

    #[test]
    fn test_dpos_validator_registration() {
        let blockchain = Blockchain::new_pow(2, 50.0).unwrap();
        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(blockchain)),
            ConsensusType::DPoS
        );
        
        let result = manager.register_validator("validator1".to_string(), 5000.0);
        assert!(result.is_ok());
        
        assert!(manager.validators.contains_key("validator1"));
    }

    #[test]
    fn test_pbft_initialization() {
        let blockchain = Blockchain::new_pow(2, 50.0).unwrap();
        let mut manager = ConsensusManager::new(
            Arc::new(Mutex::new(blockchain)),
            ConsensusType::PBFT
        );
        
        let validators = vec![
            "validator1".to_string(),
            "validator2".to_string(),
            "validator3".to_string(),
            "validator4".to_string(),
        ];
        
        let result = manager.initialize_pbft(validators);
        assert!(result.is_ok());
    }
}
