use gillean::blockchain::Blockchain;
use gillean::transaction::Transaction;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// GOVERNANCE SYSTEM IMPLEMENTATION
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalType {
    ParameterChange,
    ContractUpgrade,
    TokenMint,
    FeeAdjustment,
    EmergencyAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Failed,
    Executed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposal_type: ProposalType,
    pub proposer: String,
    pub created_at: u64,
    pub voting_start: u64,
    pub voting_end: u64,
    pub execution_delay: u64,
    pub quorum: f64,
    pub status: ProposalStatus,
    pub yes_votes: f64,
    pub no_votes: f64,
    pub abstain_votes: f64,
    pub total_votes: f64,
}

#[derive(Debug, Clone)]
pub struct Vote {
    pub proposal_id: String,
    pub voter: String,
    pub vote_type: VoteType,
    pub voting_power: f64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VoteType {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone)]
pub struct GovernanceToken {
    pub symbol: String,
    pub name: String,
    pub total_supply: f64,
    pub circulating_supply: f64,
    pub holders: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct TimelockContract {
    pub id: String,
    pub proposal_id: String,
    pub execution_time: u64,
    pub executed: bool,
    pub target_contract: String,
    pub function_call: String,
    pub parameters: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GovernanceManager {
    pub blockchain: Blockchain,
    pub proposals: HashMap<String, Proposal>,
    pub votes: HashMap<String, Vec<Vote>>,
    pub governance_token: GovernanceToken,
    pub timelock_contracts: HashMap<String, TimelockContract>,
    pub min_proposal_deposit: f64,
    pub voting_period: u64,
    pub execution_delay: u64,
}

impl GovernanceManager {
    pub fn new(blockchain: Blockchain) -> Self {
        let governance_token = GovernanceToken {
            symbol: "GOV".to_string(),
            name: "Governance Token".to_string(),
            total_supply: 1_000_000.0,
            circulating_supply: 0.0,
            holders: HashMap::new(),
        };

        Self {
            blockchain,
            proposals: HashMap::new(),
            votes: HashMap::new(),
            governance_token,
            timelock_contracts: HashMap::new(),
            min_proposal_deposit: 1000.0,
            voting_period: 7 * 24 * 60 * 60, // 7 days
            execution_delay: 2 * 24 * 60 * 60, // 2 days
        }
    }

    pub fn create_proposal(
        &mut self,
        title: &str,
        description: &str,
        proposal_type: ProposalType,
        proposer: &str,
        deposit: f64,
    ) -> Result<String, String> {
        if deposit < self.min_proposal_deposit {
            return Err("Insufficient deposit for proposal creation".to_string());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let proposal_id = format!("proposal_{}", now);
        let proposal = Proposal {
            id: proposal_id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            proposal_type,
            proposer: proposer.to_string(),
            created_at: now,
            voting_start: now + 24 * 60 * 60, // 1 day delay
            voting_end: now + 24 * 60 * 60 + self.voting_period,
            execution_delay: self.execution_delay,
            quorum: 0.1, // 10% of total supply
            status: ProposalStatus::Active,
            yes_votes: 0.0,
            no_votes: 0.0,
            abstain_votes: 0.0,
            total_votes: 0.0,
        };

        self.proposals.insert(proposal_id.clone(), proposal);
        Ok(proposal_id)
    }

    pub fn vote(
        &mut self,
        proposal_id: &str,
        voter: &str,
        vote_type: VoteType,
    ) -> Result<(), String> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now < proposal.voting_start || now > proposal.voting_end {
            return Err("Voting period is not active".to_string());
        }

        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not active for voting".to_string());
        }

        let voting_power = self.governance_token.holders.get(voter)
            .copied()
            .unwrap_or(0.0);

        if voting_power == 0.0 {
            return Err("No voting power available".to_string());
        }

        // Check if already voted
        let votes = self.votes.entry(proposal_id.to_string()).or_insert_with(Vec::new);
        if votes.iter().any(|v| v.voter == voter) {
            return Err("Already voted on this proposal".to_string());
        }

        let vote = Vote {
            proposal_id: proposal_id.to_string(),
            voter: voter.to_string(),
            vote_type: vote_type.clone(),
            voting_power,
            timestamp: now,
        };

        votes.push(vote);

        // Update proposal vote counts
        match vote_type {
            VoteType::Yes => proposal.yes_votes += voting_power,
            VoteType::No => proposal.no_votes += voting_power,
            VoteType::Abstain => proposal.abstain_votes += voting_power,
        }
        proposal.total_votes += voting_power;

        Ok(())
    }

    pub fn finalize_proposal(&mut self, proposal_id: &str) -> Result<(), String> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now < proposal.voting_end {
            return Err("Voting period has not ended".to_string());
        }

        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not active".to_string());
        }

        // Check quorum
        if proposal.total_votes < self.governance_token.total_supply * proposal.quorum {
            proposal.status = ProposalStatus::Failed;
            return Ok(());
        }

        // Determine outcome
        if proposal.yes_votes > proposal.no_votes {
            proposal.status = ProposalStatus::Passed;
            
            // Create timelock contract for execution
            let timelock_id = format!("timelock_{}", proposal_id);
            let timelock = TimelockContract {
                id: timelock_id.clone(),
                proposal_id: proposal_id.to_string(),
                execution_time: now + proposal.execution_delay,
                executed: false,
                target_contract: "governance".to_string(),
                function_call: "execute_proposal".to_string(),
                parameters: vec![proposal_id.to_string()],
            };

            self.timelock_contracts.insert(timelock_id, timelock);
        } else {
            proposal.status = ProposalStatus::Failed;
        }

        Ok(())
    }

    pub fn execute_timelock(&mut self, timelock_id: &str) -> Result<(), String> {
        let timelock = self.timelock_contracts.get_mut(timelock_id)
            .ok_or("Timelock contract not found")?;

        if timelock.executed {
            return Err("Timelock already executed".to_string());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now < timelock.execution_time {
            return Err("Timelock delay has not passed".to_string());
        }

        // Execute the proposal
        let proposal = self.proposals.get_mut(&timelock.proposal_id)
            .ok_or("Proposal not found")?;

        if proposal.status != ProposalStatus::Passed {
            return Err("Proposal is not passed".to_string());
        }

        // Execute based on proposal type
        match proposal.proposal_type {
            ProposalType::ParameterChange => {
                // Update blockchain parameters
                println!("Executing parameter change: {}", proposal.title);
            }
            ProposalType::ContractUpgrade => {
                // Upgrade smart contract
                println!("Executing contract upgrade: {}", proposal.title);
            }
            ProposalType::TokenMint => {
                // Mint new tokens
                println!("Executing token mint: {}", proposal.title);
            }
            ProposalType::FeeAdjustment => {
                // Adjust fees
                println!("Executing fee adjustment: {}", proposal.title);
            }
            ProposalType::EmergencyAction => {
                // Emergency action
                println!("Executing emergency action: {}", proposal.title);
            }
        }

        proposal.status = ProposalStatus::Executed;
        timelock.executed = true;

        Ok(())
    }

    pub fn mint_governance_tokens(&mut self, recipient: &str, amount: f64) -> Result<(), String> {
        if self.governance_token.circulating_supply + amount > self.governance_token.total_supply {
            return Err("Would exceed total supply".to_string());
        }

        *self.governance_token.holders.entry(recipient.to_string()).or_insert(0.0) += amount;
        self.governance_token.circulating_supply += amount;

        Ok(())
    }

    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    pub fn get_votes(&self, proposal_id: &str) -> Option<&Vec<Vote>> {
        self.votes.get(proposal_id)
    }

    pub fn get_governance_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert("total_proposals".to_string(), self.proposals.len() as f64);
        stats.insert("active_proposals".to_string(), 
            self.proposals.values().filter(|p| p.status == ProposalStatus::Active).count() as f64);
        stats.insert("passed_proposals".to_string(),
            self.proposals.values().filter(|p| p.status == ProposalStatus::Passed).count() as f64);
        stats.insert("total_votes".to_string(),
            self.votes.values().map(|v| v.len()).sum::<usize>() as f64);
        stats.insert("circulating_supply".to_string(), self.governance_token.circulating_supply);
        stats.insert("total_holders".to_string(), self.governance_token.holders.len() as f64);
        stats
    }
}

// ============================================================================
// TEST SUITE IMPLEMENTATION
// ============================================================================

pub struct GovernanceSuite {
    _manager: GovernanceManager,
}

impl GovernanceSuite {
    pub fn new() -> Result<Self, String> {
        let blockchain = Blockchain::new_pos(10.0, 100.0, 21)?;
        let manager = GovernanceManager::new(blockchain);
        
        Ok(Self {
            _manager: manager,
        })
    }

    pub async fn test_proposal_creation() -> Result<(), String> {
        println!("🧪 Testing proposal creation...");
        
        let blockchain = Blockchain::new_pos(10.0, 100.0, 21)?;
        let mut manager = GovernanceManager::new(blockchain);

        // Test successful proposal creation
        let proposal_id = manager.create_proposal(
            "Increase Block Reward",
            "Proposal to increase block reward from 10 to 15 tokens",
            ProposalType::ParameterChange,
            "alice",
            1500.0,
        )?;

        let proposal = manager.get_proposal(&proposal_id)
            .ok_or("Proposal not found")?;

        assert_eq!(proposal.title, "Increase Block Reward");
        assert_eq!(proposal.proposer, "alice");
        assert_eq!(proposal.status, ProposalStatus::Active);

        // Test insufficient deposit
        let result = manager.create_proposal(
            "Invalid Proposal",
            "Proposal with insufficient deposit",
            ProposalType::ParameterChange,
            "bob",
            500.0,
        );
        assert!(result.is_err());

        println!("✅ Proposal creation tests passed");
        Ok(())
    }

    pub async fn test_voting_mechanism() -> Result<(), String> {
        println!("🧪 Testing voting mechanism...");
        
        let blockchain = Blockchain::new_pos(10.0, 100.0, 21)?;
        let mut manager = GovernanceManager::new(blockchain);

        // Mint tokens for voting
        manager.mint_governance_tokens("alice", 5000.0)?;
        manager.mint_governance_tokens("bob", 3000.0)?;
        manager.mint_governance_tokens("charlie", 2000.0)?;

        // Create proposal
        let proposal_id = manager.create_proposal(
            "Fee Reduction",
            "Reduce transaction fees by 20%",
            ProposalType::FeeAdjustment,
            "alice",
            1500.0,
        )?;

        // Vote on proposal
        manager.vote(&proposal_id, "alice", VoteType::Yes)?;
        manager.vote(&proposal_id, "bob", VoteType::No)?;
        manager.vote(&proposal_id, "charlie", VoteType::Abstain)?;

        let proposal = manager.get_proposal(&proposal_id)
            .ok_or("Proposal not found")?;

        assert_eq!(proposal.yes_votes, 5000.0);
        assert_eq!(proposal.no_votes, 3000.0);
        assert_eq!(proposal.abstain_votes, 2000.0);
        assert_eq!(proposal.total_votes, 10000.0);

        // Test double voting
        let result = manager.vote(&proposal_id, "alice", VoteType::No);
        assert!(result.is_err());

        // Test voting without tokens
        let result = manager.vote(&proposal_id, "dave", VoteType::Yes);
        assert!(result.is_err());

        println!("✅ Voting mechanism tests passed");
        Ok(())
    }

    pub async fn test_proposal_finalization() -> Result<(), String> {
        println!("🧪 Testing proposal finalization...");
        
        let blockchain = Blockchain::new_pos(10.0, 100.0, 21)?;
        let mut manager = GovernanceManager::new(blockchain);

        // Mint tokens for voting
        manager.mint_governance_tokens("alice", 8000.0)?;
        manager.mint_governance_tokens("bob", 2000.0)?;

        // Create proposal
        let proposal_id = manager.create_proposal(
            "Contract Upgrade",
            "Upgrade smart contract to v2.0",
            ProposalType::ContractUpgrade,
            "alice",
            1500.0,
        )?;

        // Vote (majority yes)
        manager.vote(&proposal_id, "alice", VoteType::Yes)?;
        manager.vote(&proposal_id, "bob", VoteType::No)?;

        // Finalize proposal
        manager.finalize_proposal(&proposal_id)?;

        let proposal = manager.get_proposal(&proposal_id)
            .ok_or("Proposal not found")?;

        assert_eq!(proposal.status, ProposalStatus::Passed);
        assert!(proposal.total_votes >= manager.governance_token.total_supply * proposal.quorum);

        // Test timelock creation
        let timelock_id = format!("timelock_{}", proposal_id);
        assert!(manager.timelock_contracts.contains_key(&timelock_id));

        println!("✅ Proposal finalization tests passed");
        Ok(())
    }

    pub async fn test_timelock_execution() -> Result<(), String> {
        println!("🧪 Testing timelock execution...");
        
        let blockchain = Blockchain::new_pos(10.0, 100.0, 21)?;
        let mut manager = GovernanceManager::new(blockchain);

        // Mint tokens and create proposal
        manager.mint_governance_tokens("alice", 6000.0)?;
        manager.mint_governance_tokens("bob", 4000.0)?;

        let proposal_id = manager.create_proposal(
            "Emergency Action",
            "Emergency protocol activation",
            ProposalType::EmergencyAction,
            "alice",
            1500.0,
        )?;

        // Vote and finalize
        manager.vote(&proposal_id, "alice", VoteType::Yes)?;
        manager.vote(&proposal_id, "bob", VoteType::Yes)?;
        manager.finalize_proposal(&proposal_id)?;

        let timelock_id = format!("timelock_{}", proposal_id);

        // Test execution before delay
        let result = manager.execute_timelock(&timelock_id);
        assert!(result.is_err());

        // Simulate time passing (in real implementation, this would be actual time)
        // For testing, we'll manually adjust the execution time
        if let Some(timelock) = manager.timelock_contracts.get_mut(&timelock_id) {
            timelock.execution_time = 0; // Set to past time for testing
        }

        // Execute timelock
        manager.execute_timelock(&timelock_id)?;

        let proposal = manager.get_proposal(&proposal_id)
            .ok_or("Proposal not found")?;
        assert_eq!(proposal.status, ProposalStatus::Executed);

        // Test double execution
        let result = manager.execute_timelock(&timelock_id);
        assert!(result.is_err());

        println!("✅ Timelock execution tests passed");
        Ok(())
    }

    pub async fn test_governance_token_management() -> Result<(), String> {
        println!("🧪 Testing governance token management...");
        
        let blockchain = Blockchain::new_pos(10.0, 100.0, 21)?;
        let mut manager = GovernanceManager::new(blockchain);

        // Test token minting
        manager.mint_governance_tokens("alice", 10000.0)?;
        manager.mint_governance_tokens("bob", 5000.0)?;

        assert_eq!(manager.governance_token.circulating_supply, 15000.0);
        assert_eq!(manager.governance_token.holders.get("alice"), Some(&10000.0));
        assert_eq!(manager.governance_token.holders.get("bob"), Some(&5000.0));

        // Test exceeding total supply
        let result = manager.mint_governance_tokens("charlie", 990000.0);
        assert!(result.is_err());

        // Test governance statistics
        let stats = manager.get_governance_stats();
        assert_eq!(stats.get("circulating_supply"), Some(&15000.0));
        assert_eq!(stats.get("total_holders"), Some(&2.0));

        println!("✅ Governance token management tests passed");
        Ok(())
    }

    pub async fn test_invalid_operations() -> Result<(), String> {
        println!("🧪 Testing invalid operations...");
        
        let blockchain = Blockchain::new_pos(10.0, 100.0, 21)?;
        let mut manager = GovernanceManager::new(blockchain);

        // Test voting on non-existent proposal
        let result = manager.vote("nonexistent", "alice", VoteType::Yes);
        assert!(result.is_err());

        // Test finalizing non-existent proposal
        let result = manager.finalize_proposal("nonexistent");
        assert!(result.is_err());

        // Test executing non-existent timelock
        let result = manager.execute_timelock("nonexistent");
        assert!(result.is_err());

        // Test proposal creation with invalid parameters
        let result = manager.create_proposal("", "Description", ProposalType::ParameterChange, "alice", 1500.0);
        assert!(result.is_err());

        println!("✅ Invalid operations tests passed");
        Ok(())
    }

    pub async fn test_governance_lifecycle() -> Result<(), String> {
        println!("🧪 Testing complete governance lifecycle...");
        
        let blockchain = Blockchain::new_pos(10.0, 100.0, 21)?;
        let mut manager = GovernanceManager::new(blockchain);

        // Setup: Mint tokens to multiple stakeholders
        manager.mint_governance_tokens("alice", 4000.0)?;
        manager.mint_governance_tokens("bob", 3000.0)?;
        manager.mint_governance_tokens("charlie", 2000.0)?;
        manager.mint_governance_tokens("dave", 1000.0)?;

        // Phase 1: Create multiple proposals
        let proposal1_id = manager.create_proposal(
            "Increase Block Size",
            "Increase maximum block size to 2MB",
            ProposalType::ParameterChange,
            "alice",
            1500.0,
        )?;

        let proposal2_id = manager.create_proposal(
            "New Token Launch",
            "Launch new governance token",
            ProposalType::TokenMint,
            "bob",
            1500.0,
        )?;

        // Phase 2: Vote on proposals
        manager.vote(&proposal1_id, "alice", VoteType::Yes)?;
        manager.vote(&proposal1_id, "bob", VoteType::Yes)?;
        manager.vote(&proposal1_id, "charlie", VoteType::No)?;
        manager.vote(&proposal1_id, "dave", VoteType::Abstain)?;

        manager.vote(&proposal2_id, "alice", VoteType::No)?;
        manager.vote(&proposal2_id, "bob", VoteType::Yes)?;
        manager.vote(&proposal2_id, "charlie", VoteType::Yes)?;
        manager.vote(&proposal2_id, "dave", VoteType::Yes)?;

        // Phase 3: Finalize proposals
        manager.finalize_proposal(&proposal1_id)?;
        manager.finalize_proposal(&proposal2_id)?;

        let proposal1 = manager.get_proposal(&proposal1_id)
            .ok_or("Proposal 1 not found")?;
        let proposal2 = manager.get_proposal(&proposal2_id)
            .ok_or("Proposal 2 not found")?;

        // Proposal 1 should pass (Yes: 7000, No: 2000)
        assert_eq!(proposal1.status, ProposalStatus::Passed);
        
        // Proposal 2 should pass (Yes: 6000, No: 4000)
        assert_eq!(proposal2.status, ProposalStatus::Passed);

        // Phase 4: Execute timelocks
        let timelock1_id = format!("timelock_{}", proposal1_id);
        let timelock2_id = format!("timelock_{}", proposal2_id);

        // Simulate time passing for testing
        if let Some(timelock) = manager.timelock_contracts.get_mut(&timelock1_id) {
            timelock.execution_time = 0;
        }
        if let Some(timelock) = manager.timelock_contracts.get_mut(&timelock2_id) {
            timelock.execution_time = 0;
        }

        manager.execute_timelock(&timelock1_id)?;
        manager.execute_timelock(&timelock2_id)?;

        // Verify execution
        let proposal1 = manager.get_proposal(&proposal1_id)
            .ok_or("Proposal 1 not found")?;
        let proposal2 = manager.get_proposal(&proposal2_id)
            .ok_or("Proposal 2 not found")?;

        assert_eq!(proposal1.status, ProposalStatus::Executed);
        assert_eq!(proposal2.status, ProposalStatus::Executed);

        // Phase 5: Verify governance statistics
        let stats = manager.get_governance_stats();
        assert_eq!(stats.get("total_proposals"), Some(&2.0));
        assert_eq!(stats.get("passed_proposals"), Some(&2.0));
        assert_eq!(stats.get("total_votes"), Some(&8.0)); // 4 votes per proposal
        assert_eq!(stats.get("circulating_supply"), Some(&10000.0));

        println!("✅ Complete governance lifecycle test passed");
        Ok(())
    }
}

// ============================================================================
// TEST RUNNER INTEGRATION
// ============================================================================

pub async fn run_governance_tests() -> Result<(), String> {
    println!("🚀 Starting Governance Test Suite...");
    
    GovernanceSuite::test_proposal_creation().await?;
    GovernanceSuite::test_voting_mechanism().await?;
    GovernanceSuite::test_proposal_finalization().await?;
    GovernanceSuite::test_timelock_execution().await?;
    GovernanceSuite::test_governance_token_management().await?;
    GovernanceSuite::test_invalid_operations().await?;
    GovernanceSuite::test_governance_lifecycle().await?;

    println!("✅ All Governance tests completed successfully!");
    Ok(())
}
