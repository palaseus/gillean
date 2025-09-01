use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use log::{debug, info, warn, error};
use rand::Rng;
use chrono::Utc;
use crate::{Result, BlockchainError, crypto::DigitalSignature};

/// Consensus mechanism types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConsensusType {
    /// Proof of Work consensus
    ProofOfWork,
    /// Proof of Stake consensus
    ProofOfStake,
}

impl std::fmt::Display for ConsensusType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusType::ProofOfWork => write!(f, "pow"),
            ConsensusType::ProofOfStake => write!(f, "pos"),
        }
    }
}

/// Represents a validator in the Proof-of-Stake system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    /// Validator's public key
    pub public_key: String,
    /// Validator's address
    pub address: String,
    /// Amount of tokens staked
    pub stake_amount: f64,
    /// When the validator started staking
    pub staking_since: i64,
    /// Whether the validator is active
    pub active: bool,
    /// Validator's performance score (0.0 to 1.0)
    pub performance_score: f64,
    /// Number of blocks validated
    pub blocks_validated: u64,
    /// Number of validation failures
    pub validation_failures: u64,
}

/// Proof-of-Stake consensus implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfStake {
    /// List of validators
    pub validators: HashMap<String, Validator>,
    /// Minimum stake required to become a validator
    pub min_stake: f64,
    /// Maximum number of validators
    pub max_validators: usize,
    /// Current epoch number
    pub current_epoch: u64,
    /// Epoch duration in seconds
    pub epoch_duration: u64,
    /// Last epoch change timestamp
    pub last_epoch_change: i64,
    /// Staking rewards rate (percentage per epoch)
    pub staking_reward_rate: f64,
    /// Slashing penalty rate (percentage of stake)
    pub slashing_penalty_rate: f64,
}

/// Block validation result for PoS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosValidationResult {
    /// Whether validation was successful
    pub valid: bool,
    /// Validator who validated the block
    pub validator: String,
    /// Validator's signature
    pub signature: Option<DigitalSignature>,
    /// Validation timestamp
    pub timestamp: i64,
    /// Error message if validation failed
    pub error: Option<String>,
}

/// Staking transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingTransaction {
    /// Transaction ID
    pub id: String,
    /// Validator's address
    pub validator_address: String,
    /// Amount being staked
    pub stake_amount: f64,
    /// Transaction timestamp
    pub timestamp: i64,
    /// Transaction signature
    pub signature: Option<DigitalSignature>,
    /// Whether this is a stake or unstake operation
    pub is_stake: bool,
}

impl Validator {
    /// Create a new validator
    pub fn new(public_key: String, address: String, stake_amount: f64) -> Self {
        Validator {
            public_key,
            address,
            stake_amount,
            staking_since: Utc::now().timestamp(),
            active: true,
            performance_score: 1.0,
            blocks_validated: 0,
            validation_failures: 0,
        }
    }

    /// Update performance score based on validation success/failure
    pub fn update_performance(&mut self, success: bool) {
        if success {
            self.blocks_validated += 1;
            // Increase performance score slightly
            self.performance_score = (self.performance_score + 0.01).min(1.0);
        } else {
            self.validation_failures += 1;
            // Decrease performance score significantly
            self.performance_score = (self.performance_score - 0.1).max(0.0);
        }
    }

    /// Calculate validator's weight for selection
    pub fn calculate_weight(&self) -> f64 {
        self.stake_amount * self.performance_score
    }

    /// Add stake to validator
    pub fn add_stake(&mut self, amount: f64) -> Result<()> {
        if amount <= 0.0 {
            return Err(BlockchainError::ConsensusError(
                "Stake amount must be positive".to_string(),
            ));
        }
        self.stake_amount += amount;
        Ok(())
    }

    /// Remove stake from validator
    pub fn remove_stake(&mut self, amount: f64) -> Result<()> {
        if amount <= 0.0 {
            return Err(BlockchainError::ConsensusError(
                "Unstake amount must be positive".to_string(),
            ));
        }
        if amount > self.stake_amount {
            return Err(BlockchainError::ConsensusError(
                "Cannot unstake more than current stake".to_string(),
            ));
        }
        self.stake_amount -= amount;
        Ok(())
    }
}

impl ProofOfStake {
    /// Create a new Proof-of-Stake consensus system
    /// 
    /// # Arguments
    /// * `min_stake` - Minimum stake required to become a validator
    /// * `max_validators` - Maximum number of validators
    /// * `staking_reward_rate` - Annual staking reward rate (as percentage)
    /// * `slashing_penalty_rate` - Penalty rate for misbehavior (as percentage)
    /// 
    /// # Returns
    /// * `Result<ProofOfStake>` - The created PoS system or an error
    pub fn new(
        min_stake: f64,
        max_validators: usize,
        staking_reward_rate: f64,
        slashing_penalty_rate: f64,
    ) -> Result<Self> {
        if min_stake <= 0.0 {
            return Err(BlockchainError::ConsensusError(
                "Minimum stake must be positive".to_string(),
            ));
        }

        if max_validators == 0 {
            return Err(BlockchainError::ConsensusError(
                "Maximum validators must be greater than 0".to_string(),
            ));
        }

        let pos = ProofOfStake {
            validators: HashMap::new(),
            min_stake,
            max_validators,
            current_epoch: 0,
            epoch_duration: 86400, // 24 hours in seconds
            last_epoch_change: Utc::now().timestamp(),
            staking_reward_rate,
            slashing_penalty_rate,
        };

        info!("Created Proof-of-Stake consensus with min_stake={}, max_validators={}", 
              min_stake, max_validators);
        Ok(pos)
    }

    /// Create a PoS system with default settings
    pub fn new_default() -> Result<Self> {
        Self::new(1000.0, 100, 5.0, 10.0)
    }

    /// Register a new validator
    /// 
    /// # Arguments
    /// * `public_key` - Validator's public key
    /// * `address` - Validator's address
    /// * `stake_amount` - Amount to stake
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if registered successfully, error otherwise
    pub fn register_validator(&mut self, public_key: String, address: String, stake_amount: f64) -> Result<()> {
        if stake_amount < self.min_stake {
            return Err(BlockchainError::ConsensusError(
                format!("Stake amount {} is below minimum required {}", stake_amount, self.min_stake),
            ));
        }

        if self.validators.len() >= self.max_validators {
            return Err(BlockchainError::ConsensusError(
                "Maximum number of validators reached".to_string(),
            ));
        }

        if self.validators.contains_key(&address) {
            return Err(BlockchainError::ConsensusError(
                "Validator already registered".to_string(),
            ));
        }

        let validator = Validator::new(public_key, address.clone(), stake_amount);
        self.validators.insert(address.clone(), validator);

        info!("Registered validator: {} with stake: {}", address, stake_amount);
        Ok(())
    }

    /// Select the next validator for block creation
    /// 
    /// # Arguments
    /// * `block_height` - Current block height
    /// * `previous_block_hash` - Hash of the previous block
    /// 
    /// # Returns
    /// * `Option<String>` - Selected validator address or None if no validators
    pub fn select_validator(&self, block_height: u64, previous_block_hash: &str) -> Option<String> {
        if self.validators.is_empty() {
            return None;
        }

        // Create a deterministic seed based on block height and previous hash
        let _seed = format!("{}{}", block_height, previous_block_hash);
        let mut rng = rand::thread_rng();
        
        // Calculate total weight of all validators
        let total_weight: f64 = self.validators.values()
            .filter(|v| v.active)
            .map(|v| v.calculate_weight())
            .sum();

        if total_weight == 0.0 {
            return None;
        }

        // Use weighted random selection
        let mut random_value = rng.gen::<f64>() * total_weight;
        
        for validator in self.validators.values() {
            if !validator.active {
                continue;
            }
            
            let weight = validator.calculate_weight();
            if random_value <= weight {
                return Some(validator.address.clone());
            }
            random_value -= weight;
        }

        // Fallback to first active validator
        self.validators.values()
            .find(|v| v.active)
            .map(|v| v.address.clone())
    }

    /// Validate a block using PoS consensus
    /// 
    /// # Arguments
    /// * `block_hash` - Hash of the block to validate
    /// * `validator_address` - Address of the validator
    /// * `signature` - Validator's signature
    /// 
    /// # Returns
    /// * `Result<PosValidationResult>` - Validation result or error
    pub fn validate_block(
        &mut self,
        _block_hash: &str, // TODO: Use this parameter for validation
        validator_address: &str,
        signature: Option<DigitalSignature>,
    ) -> Result<PosValidationResult> {
        let validator = self.validators.get_mut(validator_address)
            .ok_or_else(|| BlockchainError::ConsensusError(
                "Validator not found".to_string(),
            ))?;

        if !validator.active {
            return Err(BlockchainError::ConsensusError(
                "Validator is not active".to_string(),
            ));
        }

        // In a real implementation, you would verify the signature here
        // For now, we'll assume the signature is valid if provided
        
        let timestamp = Utc::now().timestamp();
        let success = signature.is_some(); // Simplified validation

        if success {
            validator.update_performance(true);
            debug!("Block validated successfully by {}", validator_address);
        } else {
            validator.update_performance(false);
            warn!("Block validation failed by {}", validator_address);
        }

        Ok(PosValidationResult {
            valid: success,
            validator: validator_address.to_string(),
            signature,
            timestamp,
            error: if success { None } else { Some("Invalid signature".to_string()) },
        })
    }

    /// Process staking transaction
    /// 
    /// # Arguments
    /// * `staking_tx` - Staking transaction to process
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if processed successfully, error otherwise
    pub fn process_staking_transaction(&mut self, staking_tx: StakingTransaction) -> Result<()> {
        if staking_tx.is_stake {
            // Stake operation
            if let Some(validator) = self.validators.get_mut(&staking_tx.validator_address) {
                validator.add_stake(staking_tx.stake_amount)?;
            } else {
                // Register new validator
                self.register_validator(
                    "".to_string(), // Public key would be extracted from signature
                    staking_tx.validator_address.clone(),
                    staking_tx.stake_amount,
                )?;
            }
            info!("Staked {} tokens for validator {}", staking_tx.stake_amount, staking_tx.validator_address);
        } else {
            // Unstake operation
            if let Some(validator) = self.validators.get_mut(&staking_tx.validator_address) {
                validator.remove_stake(staking_tx.stake_amount)?;
                
                // Deactivate validator if stake drops below minimum
                if validator.stake_amount < self.min_stake {
                    validator.active = false;
                    info!("Deactivated validator {} due to insufficient stake", staking_tx.validator_address);
                }
            } else {
                return Err(BlockchainError::ConsensusError(
                    "Validator not found for unstaking".to_string(),
                ));
            }
            info!("Unstaked {} tokens for validator {}", staking_tx.stake_amount, staking_tx.validator_address);
        }
        Ok(())
    }

    /// Distribute staking rewards
    /// 
    /// # Arguments
    /// * `total_rewards` - Total rewards to distribute
    /// 
    /// # Returns
    /// * `HashMap<String, f64>` - Rewards per validator
    pub fn distribute_rewards(&self, total_rewards: f64) -> HashMap<String, f64> {
        let mut rewards = HashMap::new();
        
        if self.validators.is_empty() || total_rewards <= 0.0 {
            return rewards;
        }

        let total_stake: f64 = self.validators.values()
            .filter(|v| v.active)
            .map(|v| v.stake_amount)
            .sum();

        if total_stake == 0.0 {
            return rewards;
        }

        for validator in self.validators.values() {
            if validator.active {
                let validator_reward = (validator.stake_amount / total_stake) * total_rewards;
                rewards.insert(validator.address.clone(), validator_reward);
            }
        }

        rewards
    }

    /// Slash a validator for misbehavior
    /// 
    /// # Arguments
    /// * `validator_address` - Address of the validator to slash
    /// * `reason` - Reason for slashing
    /// 
    /// # Returns
    /// * `Result<f64>` - Amount slashed or error
    pub fn slash_validator(&mut self, validator_address: &str, reason: &str) -> Result<f64> {
        let validator = self.validators.get_mut(validator_address)
            .ok_or_else(|| BlockchainError::ConsensusError(
                "Validator not found".to_string(),
            ))?;

        let slash_amount = validator.stake_amount * (self.slashing_penalty_rate / 100.0);
        validator.stake_amount -= slash_amount;
        validator.performance_score = 0.0; // Reset performance
        validator.active = false; // Deactivate validator

        error!("Slashed validator {}: {} (amount: {})", validator_address, reason, slash_amount);
        Ok(slash_amount)
    }

    /// Update epoch if needed
    pub fn update_epoch(&mut self) {
        let current_time = Utc::now().timestamp();
        if current_time - self.last_epoch_change >= self.epoch_duration as i64 {
            self.current_epoch += 1;
            self.last_epoch_change = current_time;
            info!("Advanced to epoch {}", self.current_epoch);
        }
    }

    /// Get validator statistics
    pub fn get_validator_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        stats.insert("total_validators".to_string(), self.validators.len() as f64);
        stats.insert("active_validators".to_string(), 
                    self.validators.values().filter(|v| v.active).count() as f64);
        stats.insert("total_stake".to_string(), 
                    self.validators.values().map(|v| v.stake_amount).sum());
        stats.insert("average_performance".to_string(), 
                    self.validators.values().map(|v| v.performance_score).sum::<f64>() / 
                    self.validators.len().max(1) as f64);
        
        stats
    }
}

impl StakingTransaction {
    /// Create a new staking transaction
    pub fn new(
        validator_address: String,
        stake_amount: f64,
        is_stake: bool,
    ) -> Result<Self> {
        if validator_address.is_empty() {
            return Err(BlockchainError::ConsensusError(
                "Validator address cannot be empty".to_string(),
            ));
        }

        if stake_amount <= 0.0 {
            return Err(BlockchainError::ConsensusError(
                "Stake amount must be positive".to_string(),
            ));
        }

        let id = Self::generate_id(&validator_address, stake_amount, is_stake);
        let timestamp = Utc::now().timestamp();

        Ok(StakingTransaction {
            id,
            validator_address,
            stake_amount,
            timestamp,
            signature: None,
            is_stake,
        })
    }

    /// Generate transaction ID
    fn generate_id(validator_address: &str, stake_amount: f64, is_stake: bool) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(validator_address.as_bytes());
        hasher.update(stake_amount.to_string().as_bytes());
        hasher.update(if is_stake { "stake" } else { "unstake" }.as_bytes());
        hasher.update(Utc::now().timestamp().to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_creation() {
        let pos = ProofOfStake::new(1000.0, 10, 5.0, 10.0).unwrap();
        assert_eq!(pos.min_stake, 1000.0);
        assert_eq!(pos.max_validators, 10);
        assert_eq!(pos.staking_reward_rate, 5.0);
    }

    #[test]
    fn test_validator_registration() {
        let mut pos = ProofOfStake::new(1000.0, 10, 5.0, 10.0).unwrap();
        
        pos.register_validator(
            "pubkey1".to_string(),
            "validator1".to_string(),
            2000.0,
        ).unwrap();

        assert_eq!(pos.validators.len(), 1);
        assert!(pos.validators.contains_key("validator1"));
    }

    #[test]
    fn test_validator_selection() {
        let mut pos = ProofOfStake::new(1000.0, 10, 5.0, 10.0).unwrap();
        
        pos.register_validator(
            "pubkey1".to_string(),
            "validator1".to_string(),
            2000.0,
        ).unwrap();

        pos.register_validator(
            "pubkey2".to_string(),
            "validator2".to_string(),
            3000.0,
        ).unwrap();

        let selected = pos.select_validator(1, "prev_hash");
        assert!(selected.is_some());
        assert!(pos.validators.contains_key(&selected.unwrap()));
    }

    #[test]
    fn test_staking_transaction() {
        let staking_tx = StakingTransaction::new(
            "validator1".to_string(),
            1000.0,
            true,
        ).unwrap();

        assert_eq!(staking_tx.validator_address, "validator1");
        assert_eq!(staking_tx.stake_amount, 1000.0);
        assert!(staking_tx.is_stake);
    }
}
