use crate::error::BlockchainError;
use crate::storage::BlockchainStorage;
use crate::consensus::ProofOfStake;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub proposal_type: ProposalType,
    pub contract_code: Option<String>,
    pub parameters: HashMap<String, String>,
    pub voting_period: u64, // in blocks
    pub quorum: f64, // percentage of total stake required
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub voting_start: u64, // block number
    pub voting_end: u64, // block number
    pub status: ProposalStatus,
    pub total_votes: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub executed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Proposal types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    ProtocolUpgrade,
    ParameterChange,
    ContractDeployment,
    EmergencyAction,
    TreasuryAllocation,
}

/// Proposal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Failed,
    Executed,
    Cancelled,
}

/// Vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub proposal_id: String,
    pub voter: String,
    pub vote: VoteChoice,
    pub stake_amount: f64,
    pub voted_at: chrono::DateTime<chrono::Utc>,
    pub block_number: u64,
}

/// Vote choices
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

/// Governance system
pub struct Governance {
    storage: Arc<BlockchainStorage>,
    consensus: Arc<ProofOfStake>,
    proposals: Arc<RwLock<HashMap<String, GovernanceProposal>>>,
    votes: Arc<RwLock<HashMap<String, Vec<Vote>>>>, // proposal_id -> votes
    current_block: Arc<RwLock<u64>>,
}

/// Proposal creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalCreationRequest {
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub contract_code: Option<String>,
    pub parameters: HashMap<String, String>,
    pub voting_period: u64,
    pub quorum: f64,
}

/// Vote request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRequest {
    pub proposal_id: String,
    pub vote: VoteChoice,
    pub stake_amount: f64,
}

impl Governance {
    /// Create a new governance system
    pub async fn new(storage: Arc<BlockchainStorage>, consensus: Arc<ProofOfStake>) -> Result<Self, BlockchainError> {
        let governance = Self {
            storage,
            consensus,
            proposals: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            current_block: Arc::new(RwLock::new(0)),
        };

        // Load existing proposals and votes from storage
        governance.load_governance_data().await?;

        Ok(governance)
    }

    /// Create a new proposal
    pub async fn create_proposal(
        &self,
        proposer: &str,
        request: ProposalCreationRequest,
    ) -> Result<String, BlockchainError> {
        // Validate proposer has sufficient stake
        let proposer_stake = *self.consensus.get_validator_stats().get(proposer).unwrap_or(&0.0);
        if proposer_stake < 1000.0 {
            return Err(BlockchainError::ValidatorError(
                "Insufficient stake to create proposal (minimum 1000 GIL)".to_string()
            ));
        }

        // Generate proposal ID
        let proposal_id = uuid::Uuid::new_v4().to_string();

        // Get current block number
        let current_block = *self.current_block.read().await;

        // Create proposal
        let proposal = GovernanceProposal {
            id: proposal_id.clone(),
            title: request.title,
            description: request.description,
            proposer: proposer.to_string(),
            proposal_type: request.proposal_type,
            contract_code: request.contract_code,
            parameters: request.parameters,
            voting_period: request.voting_period,
            quorum: request.quorum,
            created_at: chrono::Utc::now(),
            voting_start: current_block,
            voting_end: current_block + request.voting_period,
            status: ProposalStatus::Active,
            total_votes: 0,
            yes_votes: 0,
            no_votes: 0,
            executed_at: None,
        };

        // Store proposal
        {
            let mut proposals = self.proposals.write().await;
            proposals.insert(proposal_id.clone(), proposal.clone());
        }

        // Initialize votes for this proposal
        {
            let mut votes = self.votes.write().await;
            votes.insert(proposal_id.clone(), Vec::new());
        }

        // Save to persistent storage
        self.save_proposal(&proposal).await?;

        info!("Created governance proposal: {} - {}", proposal_id, proposal.title);
        Ok(proposal_id)
    }

    /// Vote on a proposal
    pub async fn vote_on_proposal(
        &self,
        voter: &str,
        request: VoteRequest,
    ) -> Result<(), BlockchainError> {
        // Get proposal
        let proposal = {
            let proposals = self.proposals.read().await;
            proposals.get(&request.proposal_id)
                .cloned()
                .ok_or_else(|| BlockchainError::NotFound("Proposal not found".to_string()))?
        };

        // Check if proposal is still active
        if proposal.status != ProposalStatus::Active {
            return Err(BlockchainError::ValidatorError("Proposal is not active".to_string()));
        }

        // Check if voting period has ended
        let current_block = *self.current_block.read().await;
        if current_block > proposal.voting_end {
            return Err(BlockchainError::ValidatorError("Voting period has ended".to_string()));
        }

        // Check if voter has already voted
        {
            let votes = self.votes.read().await;
            if let Some(proposal_votes) = votes.get(&request.proposal_id) {
                if proposal_votes.iter().any(|v| v.voter == voter) {
                    return Err(BlockchainError::ValidatorError("Already voted on this proposal".to_string()));
                }
            }
        }

        // Validate voter has sufficient stake
        let voter_stake = *self.consensus.get_validator_stats().get(voter).unwrap_or(&0.0);
        if voter_stake < request.stake_amount {
            return Err(BlockchainError::ValidatorError("Insufficient stake to vote".to_string()));
        }

        // Create vote
        let vote = Vote {
            proposal_id: request.proposal_id.clone(),
            voter: voter.to_string(),
            vote: request.vote.clone(),
            stake_amount: request.stake_amount,
            voted_at: chrono::Utc::now(),
            block_number: current_block,
        };

        // Store vote
        {
            let mut votes = self.votes.write().await;
            votes.entry(request.proposal_id.clone())
                .or_insert_with(Vec::new)
                .push(vote.clone());
        }

        // Update proposal vote counts
        {
            let mut proposals = self.proposals.write().await;
            if let Some(proposal) = proposals.get_mut(&request.proposal_id) {
                proposal.total_votes += 1;
                match request.vote {
                    VoteChoice::Yes => proposal.yes_votes += 1,
                    VoteChoice::No => proposal.no_votes += 1,
                    VoteChoice::Abstain => {}, // Abstain doesn't count towards yes/no
                }
            }
        }

        // Save vote to storage
        self.save_vote(&vote).await?;

        info!("Vote cast on proposal {}: {} voted {:?}", 
              request.proposal_id, voter, request.vote);

        Ok(())
    }

    /// Execute a passed proposal
    pub async fn execute_proposal(&self, proposal_id: &str) -> Result<(), BlockchainError> {
        // Get proposal
        let proposal = {
            let proposals = self.proposals.read().await;
            proposals.get(proposal_id)
                .ok_or_else(|| BlockchainError::NotFound("Proposal not found".to_string()))?
                .clone()
        };

        // Check if proposal has passed
        if proposal.status != ProposalStatus::Passed {
            return Err(BlockchainError::ValidatorError("Proposal has not passed".to_string()));
        }

        // Execute based on proposal type
        match proposal.proposal_type {
            ProposalType::ContractDeployment => {
                if let Some(_contract_code) = &proposal.contract_code {
                    // Deploy the contract
                    // This would integrate with the smart contract system
                    info!("Executing contract deployment proposal: {}", proposal_id);
                }
            }
            ProposalType::ParameterChange => {
                // Apply parameter changes
                for (key, value) in &proposal.parameters {
                    info!("Applying parameter change: {} = {}", key, value);
                    // This would update blockchain parameters
                }
            }
            ProposalType::ProtocolUpgrade => {
                // Trigger protocol upgrade
                info!("Executing protocol upgrade proposal: {}", proposal_id);
            }
            ProposalType::EmergencyAction => {
                // Execute emergency action
                info!("Executing emergency action proposal: {}", proposal_id);
            }
            ProposalType::TreasuryAllocation => {
                // Allocate treasury funds
                info!("Executing treasury allocation proposal: {}", proposal_id);
            }
        }

        // Update proposal status
        let mut updated_proposal = proposal.clone();
        updated_proposal.status = ProposalStatus::Executed;
        updated_proposal.executed_at = Some(chrono::Utc::now());

        // Save updated proposal
        self.save_proposal(&updated_proposal).await?;

        info!("Executed proposal: {}", proposal_id);
        Ok(())
    }

    /// Update proposal statuses based on current block
    pub async fn update_proposal_statuses(&self) -> Result<(), BlockchainError> {
        let current_block = *self.current_block.read().await;
        let total_stake: f64 = self.consensus.get_validator_stats().values().sum();

        let mut proposals = self.proposals.write().await;
        let votes = self.votes.read().await;

        for proposal in proposals.values_mut() {
            if proposal.status == ProposalStatus::Active && current_block > proposal.voting_end {
                // Calculate voting results
                let total_voted_stake = votes.get(&proposal.id)
                    .map(|v| v.iter().map(|vote| vote.stake_amount).sum::<f64>())
                    .unwrap_or(0.0);

                let quorum_met = total_voted_stake >= (total_stake * proposal.quorum / 100.0);
                let majority_yes = proposal.yes_votes > proposal.no_votes;

                if quorum_met && majority_yes {
                    proposal.status = ProposalStatus::Passed;
                    info!("Proposal {} passed", proposal.id);
                } else {
                    proposal.status = ProposalStatus::Failed;
                    info!("Proposal {} failed", proposal.id);
                }

                // Save updated proposal
                self.save_proposal(proposal).await?;
            }
        }

        Ok(())
    }

    /// Get proposal by ID
    pub async fn get_proposal(&self, proposal_id: &str) -> Result<Option<GovernanceProposal>, BlockchainError> {
        let proposals = self.proposals.read().await;
        Ok(proposals.get(proposal_id).cloned())
    }

    /// Get all proposals
    pub async fn get_all_proposals(&self) -> Result<Vec<GovernanceProposal>, BlockchainError> {
        let proposals = self.proposals.read().await;
        Ok(proposals.values().cloned().collect())
    }

    /// Get votes for a proposal
    pub async fn get_proposal_votes(&self, proposal_id: &str) -> Result<Vec<Vote>, BlockchainError> {
        let votes = self.votes.read().await;
        Ok(votes.get(proposal_id).cloned().unwrap_or_default())
    }

    /// Clone for background processing
    pub fn clone_for_background(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            proposals: self.proposals.clone(),
            consensus: self.consensus.clone(),
            current_block: self.current_block.clone(),
            votes: self.votes.clone(),
        }
    }

    /// Get governance statistics
    pub async fn get_governance_stats(&self) -> Result<GovernanceStats, BlockchainError> {
        let proposals = self.proposals.read().await;
        let votes = self.votes.read().await;

        let mut stats = GovernanceStats::default();
        
        for proposal in proposals.values() {
            stats.total_proposals += 1;
            
            match proposal.status {
                ProposalStatus::Active => stats.active_proposals += 1,
                ProposalStatus::Passed => stats.passed_proposals += 1,
                ProposalStatus::Failed => stats.failed_proposals += 1,
                ProposalStatus::Executed => stats.executed_proposals += 1,
                ProposalStatus::Cancelled => stats.cancelled_proposals += 1,
            }
        }

        for proposal_votes in votes.values() {
            stats.total_votes += proposal_votes.len() as u64;
        }

        Ok(stats)
    }

    /// Update current block number
    pub async fn update_block_number(&self, block_number: u64) -> Result<(), BlockchainError> {
        *self.current_block.write().await = block_number;
        
        // Update proposal statuses
        self.update_proposal_statuses().await?;
        
        Ok(())
    }

    /// Save proposal to storage
    async fn save_proposal(&self, proposal: &GovernanceProposal) -> Result<(), BlockchainError> {
        let key = format!("proposal:{}", proposal.id);
        let value = serde_json::to_string(proposal)
            .map_err(|e| BlockchainError::SerializationError(format!("Failed to serialize proposal: {}", e)))?;
        
        Ok(self.storage.set(&key, value.as_bytes())?)
    }

    /// Save vote to storage
    async fn save_vote(&self, vote: &Vote) -> Result<(), BlockchainError> {
        let key = format!("vote:{}:{}", vote.proposal_id, vote.voter);
        let value = serde_json::to_string(vote)
            .map_err(|e| BlockchainError::SerializationError(format!("Failed to serialize vote: {}", e)))?;
        
        Ok(self.storage.set(&key, value.as_bytes())?)
    }

    /// Load governance data from storage
    async fn load_governance_data(&self) -> Result<(), BlockchainError> {
        // Load proposals
        let proposal_prefix = "proposal:";
        let proposals_data = self.storage.get_by_prefix(proposal_prefix)?;
        
        let mut proposals = self.proposals.write().await;
        
        for (key, value) in proposals_data.iter() {
            if let Ok(proposal) = serde_json::from_str::<GovernanceProposal>(&String::from_utf8_lossy(value)) {
                let id = key.strip_prefix(proposal_prefix).unwrap_or(key).to_string();
                proposals.insert(id, proposal);
            }
        }

        // Load votes
        let vote_prefix = "vote:";
        let votes_data = self.storage.get_by_prefix(vote_prefix)?;
        
        let mut votes = self.votes.write().await;
        
        for (_key, value) in votes_data.iter() {
            if let Ok(vote) = serde_json::from_str::<Vote>(&String::from_utf8_lossy(value)) {
                votes.entry(vote.proposal_id.clone())
                    .or_insert_with(Vec::new)
                    .push(vote);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GovernanceStats {
    pub total_proposals: u64,
    pub active_proposals: u64,
    pub passed_proposals: u64,
    pub failed_proposals: u64,
    pub executed_proposals: u64,
    pub cancelled_proposals: u64,
    pub total_votes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::BlockchainStorage;
    use crate::consensus::ProofOfStake;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_proposal_creation() {
        let temp_dir = tempdir().unwrap();
        let storage = Arc::new(BlockchainStorage::new(temp_dir.path().to_str().unwrap()).unwrap());
        let consensus = Arc::new(ProofOfStake::new(50.0, 10, 0.1, 0.1).unwrap());
        let governance = Governance::new(storage, consensus).await.unwrap();

        let request = ProposalCreationRequest {
            title: "Test Proposal".to_string(),
            description: "A test proposal".to_string(),
            proposal_type: ProposalType::ParameterChange,
            contract_code: None,
            parameters: HashMap::new(),
            voting_period: 100,
            quorum: 50.0,
        };

        // This will fail due to insufficient stake, but we can test the structure
        let result = governance.create_proposal("test_proposer", request).await;
        assert!(result.is_err()); // Expected to fail due to insufficient stake
    }

    #[test]
    fn test_proposal_status() {
        let proposal = GovernanceProposal {
            id: "test".to_string(),
            title: "Test".to_string(),
            description: "Test".to_string(),
            proposer: "test".to_string(),
            proposal_type: ProposalType::ParameterChange,
            contract_code: None,
            parameters: HashMap::new(),
            voting_period: 100,
            quorum: 50.0,
            created_at: chrono::Utc::now(),
            voting_start: 0,
            voting_end: 100,
            status: ProposalStatus::Active,
            total_votes: 0,
            yes_votes: 0,
            no_votes: 0,
            executed_at: None,
        };

        assert_eq!(proposal.status, ProposalStatus::Active);
    }
}
