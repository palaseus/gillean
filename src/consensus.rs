use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use log::{debug, info, warn, error};
// Removed unused import
use chrono::Utc;
use crate::{Result, BlockchainError, crypto::DigitalSignature};
use sha2::{Sha256, Digest};

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
    /// Number of times validator has been slashed
    pub slash_count: u64,
    /// Last slashing timestamp
    pub last_slash_time: Option<i64>,
    /// Validator's reputation score (0.0 to 1.0)
    pub reputation_score: f64,
    /// Whether validator is jailed (temporarily inactive)
    pub jailed: bool,
    /// Jail end timestamp
    pub jail_end_time: Option<i64>,
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
    /// Finality threshold (percentage of validators needed for finality)
    pub finality_threshold: f64,
    /// Jail duration in seconds
    pub jail_duration: u64,
    /// Pending slashing evidence
    pub pending_slashings: Vec<SlashingEvidence>,
    /// Finalized blocks
    pub finalized_blocks: HashSet<String>,
    /// Current epoch info
    pub current_epoch_info: Option<EpochInfo>,
    /// Validator selection seed for current epoch
    pub selection_seed: String,
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

/// Finality proof for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalityProof {
    /// Block hash
    pub block_hash: String,
    /// Validator signatures
    pub signatures: Vec<DigitalSignature>,
    /// Validator addresses that signed
    pub signers: Vec<String>,
    /// Finality timestamp
    pub timestamp: i64,
    /// Epoch number
    pub epoch: u64,
}

/// Slashing evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvidence {
    /// Validator address
    pub validator_address: String,
    /// Type of slashing offense
    pub offense_type: SlashingOffense,
    /// Evidence data
    pub evidence: String,
    /// Reporter address
    pub reporter: String,
    /// Timestamp
    pub timestamp: i64,
}

/// Types of slashing offenses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashingOffense {
    /// Double signing (signing two different blocks at same height)
    DoubleSigning,
    /// Liveness failure (not producing blocks when selected)
    LivenessFailure,
    /// Invalid block production
    InvalidBlock,
    /// Unavailability (not responding to challenges)
    Unavailability,
}

/// Epoch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochInfo {
    /// Epoch number
    pub epoch: u64,
    /// Start timestamp
    pub start_time: i64,
    /// End timestamp
    pub end_time: i64,
    /// Validators in this epoch
    pub validators: Vec<String>,
    /// Epoch hash (for deterministic selection)
    pub epoch_hash: String,
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
            slash_count: 0,
            last_slash_time: None,
            reputation_score: 1.0,
            jailed: false,
            jail_end_time: None,
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
        if self.jailed || !self.active {
            return 0.0;
        }
        self.stake_amount * self.performance_score * self.reputation_score
    }

    /// Check if validator is eligible for selection
    pub fn is_eligible(&self) -> bool {
        self.active && !self.jailed && self.stake_amount > 0.0
    }

    /// Update reputation score based on behavior
    pub fn update_reputation(&mut self, positive: bool) {
        if positive {
            self.reputation_score = (self.reputation_score + 0.01).min(1.0);
        } else {
            self.reputation_score = (self.reputation_score - 0.05).max(0.0);
        }
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
            finality_threshold: 0.67, // 2/3 of validators
            jail_duration: 86400, // 24 hours
            pending_slashings: Vec::new(),
            finalized_blocks: HashSet::new(),
            current_epoch_info: None,
            selection_seed: String::new(),
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

    /// Select the next validator for block creation using secure deterministic selection
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

        // Create deterministic seed using block height, previous hash, and epoch
        let seed_data = format!("{}{}{}{}", block_height, previous_block_hash, self.current_epoch, self.selection_seed);
        let mut hasher = Sha256::new();
        hasher.update(seed_data.as_bytes());
        let seed_hash = hasher.finalize();
        
        // Convert hash to deterministic "random" value
        let seed_value = u64::from_le_bytes([
            seed_hash[0], seed_hash[1], seed_hash[2], seed_hash[3],
            seed_hash[4], seed_hash[5], seed_hash[6], seed_hash[7],
        ]);
        
        // Get eligible validators (active, not jailed, with stake)
        let eligible_validators: Vec<&Validator> = self.validators.values()
            .filter(|v| v.is_eligible())
            .collect();

        if eligible_validators.is_empty() {
            return None;
        }

        // Calculate total weight of eligible validators
        let total_weight: f64 = eligible_validators.iter()
            .map(|v| v.calculate_weight())
            .sum();

        if total_weight == 0.0 {
            return None;
        }

        // Use deterministic weighted selection
        let mut selection_value = (seed_value as f64 / u64::MAX as f64) * total_weight;
        
        for validator in &eligible_validators {
            let weight = validator.calculate_weight();
            if selection_value <= weight {
                return Some(validator.address.clone());
            }
            selection_value -= weight;
        }

        // Fallback to first eligible validator
        eligible_validators.first().map(|v| v.address.clone())
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

    /// Slash a validator for misbehavior with enhanced security
    /// 
    /// # Arguments
    /// * `validator_address` - Address of the validator to slash
    /// * `evidence` - Slashing evidence
    /// 
    /// # Returns
    /// * `Result<f64>` - Amount slashed or error
    pub fn slash_validator(&mut self, evidence: SlashingEvidence) -> Result<f64> {
        let validator = self.validators.get_mut(&evidence.validator_address)
            .ok_or_else(|| BlockchainError::ConsensusError(
                "Validator not found".to_string(),
            ))?;

        // Calculate slash amount based on offense type
        let slash_percentage = match evidence.offense_type {
            SlashingOffense::DoubleSigning => 0.5, // 50% for double signing
            SlashingOffense::LivenessFailure => 0.01, // 1% for liveness failure
            SlashingOffense::InvalidBlock => 0.1, // 10% for invalid blocks
            SlashingOffense::Unavailability => 0.05, // 5% for unavailability
        };

        let slash_amount = validator.stake_amount * slash_percentage;
        validator.stake_amount -= slash_amount;
        validator.slash_count += 1;
        validator.last_slash_time = Some(evidence.timestamp);
        
        // Update reputation and performance
        validator.reputation_score = 0.0; // Reset reputation
        validator.performance_score = (validator.performance_score * 0.5).max(0.1);
        
        // Jail validator for serious offenses
        if matches!(evidence.offense_type, SlashingOffense::DoubleSigning | SlashingOffense::InvalidBlock) {
            validator.jailed = true;
            validator.jail_end_time = Some(evidence.timestamp + self.jail_duration as i64);
            validator.active = false;
        }

        // Deactivate if stake drops below minimum
        if validator.stake_amount < self.min_stake {
            validator.active = false;
        }

        error!("Slashed validator {}: {:?} (amount: {}, slash_count: {})", 
               evidence.validator_address, evidence.offense_type, slash_amount, validator.slash_count);
        
        Ok(slash_amount)
    }

    /// Submit slashing evidence
    /// 
    /// # Arguments
    /// * `evidence` - Slashing evidence to submit
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if submitted successfully
    pub fn submit_slashing_evidence(&mut self, evidence: SlashingEvidence) -> Result<()> {
        // Validate evidence
        if evidence.validator_address.is_empty() || evidence.evidence.is_empty() {
            return Err(BlockchainError::ConsensusError(
                "Invalid slashing evidence".to_string(),
            ));
        }

        // Check if validator exists
        if !self.validators.contains_key(&evidence.validator_address) {
            return Err(BlockchainError::ConsensusError(
                "Validator not found".to_string(),
            ));
        }

        // Add to pending slashings
        let validator_address = evidence.validator_address.clone();
        self.pending_slashings.push(evidence);
        
        info!("Submitted slashing evidence for validator: {}", validator_address);
        Ok(())
    }

    /// Process pending slashing evidence
    /// 
    /// # Returns
    /// * `Result<Vec<f64>>` - List of slash amounts
    pub fn process_pending_slashings(&mut self) -> Result<Vec<f64>> {
        let mut slash_amounts = Vec::new();
        let mut processed_validators = Vec::new();

        // Clone the evidence to avoid borrowing issues
        let evidence_to_process: Vec<SlashingEvidence> = self.pending_slashings.clone();

        for evidence in evidence_to_process {
            match self.slash_validator(evidence.clone()) {
                Ok(amount) => {
                    slash_amounts.push(amount);
                    processed_validators.push(evidence.validator_address);
                }
                Err(e) => {
                    warn!("Failed to process slashing evidence: {}", e);
                }
            }
        }

        // Remove processed evidence
        for validator_address in processed_validators {
            self.pending_slashings.retain(|e| e.validator_address != validator_address);
        }

        Ok(slash_amounts)
    }

    /// Update epoch if needed and generate new selection seed
    pub fn update_epoch(&mut self) {
        let current_time = Utc::now().timestamp();
        if current_time - self.last_epoch_change >= self.epoch_duration as i64 {
            self.current_epoch += 1;
            self.last_epoch_change = current_time;
            
            // Generate new selection seed for deterministic validator selection
            let mut hasher = Sha256::new();
            hasher.update(self.current_epoch.to_string().as_bytes());
            hasher.update(current_time.to_string().as_bytes());
            hasher.update("epoch_seed".as_bytes());
            self.selection_seed = format!("{:x}", hasher.finalize());
            
            // Create epoch info
            self.current_epoch_info = Some(EpochInfo {
                epoch: self.current_epoch,
                start_time: current_time,
                end_time: current_time + self.epoch_duration as i64,
                validators: self.validators.keys().cloned().collect(),
                epoch_hash: self.selection_seed.clone(),
            });
            
            info!("Advanced to epoch {} with seed: {}", self.current_epoch, self.selection_seed);
        }
    }

    /// Finalize a block with validator signatures
    /// 
    /// # Arguments
    /// * `block_hash` - Hash of the block to finalize
    /// * `signatures` - Validator signatures
    /// * `signers` - Validator addresses that signed
    /// 
    /// # Returns
    /// * `Result<FinalityProof>` - Finality proof or error
    pub fn finalize_block(
        &mut self,
        block_hash: String,
        signatures: Vec<DigitalSignature>,
        signers: Vec<String>,
    ) -> Result<FinalityProof> {
        // Check if we have enough signatures for finality
        let total_validators = self.validators.len();
        let required_signatures = (total_validators as f64 * self.finality_threshold).ceil() as usize;
        
        if signatures.len() < required_signatures {
            return Err(BlockchainError::ConsensusError(
                format!("Insufficient signatures for finality: {} < {}", signatures.len(), required_signatures),
            ));
        }

        // Validate that all signers are valid validators
        for signer in &signers {
            if !self.validators.contains_key(signer) {
                return Err(BlockchainError::ConsensusError(
                    format!("Invalid validator signer: {}", signer),
                ));
            }
        }

        let finality_proof = FinalityProof {
            block_hash: block_hash.clone(),
            signatures,
            signers,
            timestamp: Utc::now().timestamp(),
            epoch: self.current_epoch,
        };

        // Mark block as finalized
        let block_hash_clone = block_hash.clone();
        self.finalized_blocks.insert(block_hash);
        
        info!("Block {} finalized with {} signatures", block_hash_clone, finality_proof.signatures.len());
        Ok(finality_proof)
    }

    /// Check if a block is finalized
    /// 
    /// # Arguments
    /// * `block_hash` - Hash of the block to check
    /// 
    /// # Returns
    /// * `bool` - True if block is finalized
    pub fn is_block_finalized(&self, block_hash: &str) -> bool {
        self.finalized_blocks.contains(block_hash)
    }

    /// Unjail validators whose jail period has expired
    pub fn unjail_validators(&mut self) {
        let current_time = Utc::now().timestamp();
        let mut unjailed_count = 0;

        for validator in self.validators.values_mut() {
            if validator.jailed {
                if let Some(jail_end_time) = validator.jail_end_time {
                    if current_time >= jail_end_time {
                        validator.jailed = false;
                        validator.jail_end_time = None;
                        validator.active = true;
                        unjailed_count += 1;
                        
                        info!("Unjailed validator: {}", validator.address);
                    }
                }
            }
        }

        if unjailed_count > 0 {
            info!("Unjailed {} validators", unjailed_count);
        }
    }

    /// Get validator statistics
    pub fn get_validator_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        stats.insert("total_validators".to_string(), self.validators.len() as f64);
        stats.insert("active_validators".to_string(), 
                    self.validators.values().filter(|v| v.active).count() as f64);
        stats.insert("jailed_validators".to_string(), 
                    self.validators.values().filter(|v| v.jailed).count() as f64);
        stats.insert("total_stake".to_string(), 
                    self.validators.values().map(|v| v.stake_amount).sum());
        stats.insert("average_performance".to_string(), 
                    self.validators.values().map(|v| v.performance_score).sum::<f64>() / 
                    self.validators.len().max(1) as f64);
        stats.insert("average_reputation".to_string(), 
                    self.validators.values().map(|v| v.reputation_score).sum::<f64>() / 
                    self.validators.len().max(1) as f64);
        stats.insert("total_slashings".to_string(), 
                    self.validators.values().map(|v| v.slash_count as f64).sum());
        stats.insert("finalized_blocks".to_string(), self.finalized_blocks.len() as f64);
        stats.insert("pending_slashings".to_string(), self.pending_slashings.len() as f64);
        
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
        assert_eq!(pos.finality_threshold, 0.67);
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
        
        let validator = &pos.validators["validator1"];
        assert_eq!(validator.reputation_score, 1.0);
        assert!(!validator.jailed);
    }

    #[test]
    fn test_deterministic_validator_selection() {
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

        // Test deterministic selection - same inputs should produce same result
        let selected1 = pos.select_validator(1, "prev_hash");
        let selected2 = pos.select_validator(1, "prev_hash");
        assert_eq!(selected1, selected2);
        
        assert!(selected1.is_some());
        assert!(pos.validators.contains_key(&selected1.unwrap()));
    }

    #[test]
    fn test_slashing_mechanisms() {
        let mut pos = ProofOfStake::new(1000.0, 10, 5.0, 10.0).unwrap();
        
        pos.register_validator(
            "pubkey1".to_string(),
            "validator1".to_string(),
            2000.0,
        ).unwrap();

        let evidence = SlashingEvidence {
            validator_address: "validator1".to_string(),
            offense_type: SlashingOffense::DoubleSigning,
            evidence: "double_signature_proof".to_string(),
            reporter: "reporter1".to_string(),
            timestamp: Utc::now().timestamp(),
        };

        let slash_amount = pos.slash_validator(evidence).unwrap();
        assert!(slash_amount > 0.0);
        
        let validator = &pos.validators["validator1"];
        assert!(validator.jailed);
        assert!(!validator.active);
        assert_eq!(validator.reputation_score, 0.0);
        assert_eq!(validator.slash_count, 1);
    }

    #[test]
    fn test_finality_mechanisms() {
        let mut pos = ProofOfStake::new(1000.0, 10, 5.0, 10.0).unwrap();
        
        // Register validators
        for i in 1..=5 {
            pos.register_validator(
                format!("pubkey{}", i),
                format!("validator{}", i),
                1000.0,
            ).unwrap();
        }

        let block_hash = "test_block_hash".to_string();
        let signatures = vec![
            DigitalSignature { signature: vec![1, 2, 3], public_key: vec![1, 2, 3, 4] },
            DigitalSignature { signature: vec![4, 5, 6], public_key: vec![5, 6, 7, 8] },
            DigitalSignature { signature: vec![7, 8, 9], public_key: vec![9, 10, 11, 12] },
            DigitalSignature { signature: vec![10, 11, 12], public_key: vec![13, 14, 15, 16] },
        ];
        let signers = vec![
            "validator1".to_string(),
            "validator2".to_string(),
            "validator3".to_string(),
            "validator4".to_string(),
        ];

        let finality_proof = pos.finalize_block(block_hash.clone(), signatures, signers).unwrap();
        assert_eq!(finality_proof.block_hash, block_hash);
        assert_eq!(finality_proof.signatures.len(), 4);
        
        assert!(pos.is_block_finalized(&block_hash));
    }

    #[test]
    fn test_epoch_management() {
        let mut pos = ProofOfStake::new(1000.0, 10, 5.0, 10.0).unwrap();
        
        let initial_epoch = pos.current_epoch;
        pos.update_epoch();
        
        // Epoch shouldn't change immediately
        assert_eq!(pos.current_epoch, initial_epoch);
        
        // Test epoch info creation
        assert!(pos.current_epoch_info.is_none());
    }

    #[test]
    fn test_validator_jailing_and_unjailing() {
        let mut pos = ProofOfStake::new(1000.0, 10, 5.0, 10.0).unwrap();
        
        pos.register_validator(
            "pubkey1".to_string(),
            "validator1".to_string(),
            2000.0,
        ).unwrap();

        // Jail validator
        let evidence = SlashingEvidence {
            validator_address: "validator1".to_string(),
            offense_type: SlashingOffense::InvalidBlock,
            evidence: "invalid_block_proof".to_string(),
            reporter: "reporter1".to_string(),
            timestamp: Utc::now().timestamp(),
        };

        pos.slash_validator(evidence).unwrap();
        
        let validator = &pos.validators["validator1"];
        assert!(validator.jailed);
        assert!(!validator.active);
        
        // Test unjailing (would need to manipulate time in real implementation)
        pos.unjail_validators();
        // Validator should still be jailed since jail period hasn't expired
        let validator = &pos.validators["validator1"];
        assert!(validator.jailed);
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
        assert!(!staking_tx.id.is_empty());
    }

    #[test]
    fn test_validator_eligibility() {
        let mut pos = ProofOfStake::new(1000.0, 10, 5.0, 10.0).unwrap();
        
        pos.register_validator(
            "pubkey1".to_string(),
            "validator1".to_string(),
            2000.0,
        ).unwrap();

        let validator = &pos.validators["validator1"];
        assert!(validator.is_eligible());
        
        // Test jailed validator
        let evidence = SlashingEvidence {
            validator_address: "validator1".to_string(),
            offense_type: SlashingOffense::DoubleSigning,
            evidence: "proof".to_string(),
            reporter: "reporter".to_string(),
            timestamp: Utc::now().timestamp(),
        };
        
        pos.slash_validator(evidence).unwrap();
        let validator = &pos.validators["validator1"];
        assert!(!validator.is_eligible());
    }

    #[test]
    fn test_validator_statistics() {
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

        let stats = pos.get_validator_stats();
        assert_eq!(stats["total_validators"], 2.0);
        assert_eq!(stats["active_validators"], 2.0);
        assert_eq!(stats["jailed_validators"], 0.0);
        assert_eq!(stats["total_stake"], 5000.0);
        assert_eq!(stats["finalized_blocks"], 0.0);
    }
}
