//! # Gillean Blockchain v2.0.0
//! 
//! A privacy-focused, enterprise-grade blockchain platform in Rust featuring zero-knowledge proofs, 
//! layer 2 scaling with state channels, smart contracts, Proof-of-Stake consensus, sharding for scalability, 
//! cross-chain interoperability, and a WebAssembly-based virtual machine.
//! 
//! This crate provides a complete blockchain implementation with:
//! - Block creation and mining with proof-of-work
//! - Transaction handling (public and private)
//! - Chain validation
//! - CLI interface for testing
//! - Persistent storage using sled
//! - REST API using axum
//! - Wallet management with encryption
//! - Smart contracts with WebAssembly VM
//! - Proof-of-Stake consensus mechanism
//! - Frontend UI with Yew and advanced analytics
//! - **Zero-Knowledge Proofs for private transactions**
//! - **Layer 2 scaling with state channels**
//! - **Developer SDK for external integration**
//! - **Sharding for horizontal scalability**
//! - **Cross-chain interoperability**
//! - **Contract development toolkit**
//! - **Ethereum testnet integration**
//! - **Decentralized Identity (DID) system**
//! - **On-chain governance framework**
//! - **TypeScript SDK support**
//! - **Simulation mode for testing**
//! 
//! ## Architecture
//! 
//! The project follows a modular architecture with clear separation of concerns:
//! 
//! - **Blockchain**: Main orchestrator managing the chain of blocks
//! - **Block**: Individual blocks containing transactions and metadata
//! - **Transaction**: Value transfers between addresses (public and private)
//! - **ZKP**: Zero-knowledge proofs for private transactions
//! - **State Channels**: Layer 2 scaling for off-chain transactions
//! - **Proof of Work**: Mining algorithm for consensus
//! - **Consensus**: PoS consensus mechanism
//! - **Smart Contract**: WebAssembly-based virtual machine for smart contracts
//! - **Storage**: Persistent storage using sled database
//! - **Wallet**: Wallet management with encryption
//! - **API**: REST API for blockchain interaction
//! - **Utils**: Helper functions for hashing and validation
//! - **Sharding**: Horizontal scaling through blockchain sharding
//! - **Interop**: Cross-chain communication and bridge protocol
//! - **Contract Toolkit**: Developer tools for WASM contract development
//! - **Ethereum**: Integration with Ethereum testnets for cross-chain transfers
//! - **DID**: Decentralized Identity system for user authentication
//! - **Governance**: On-chain governance for decentralized decision-making
//! - **Simulation**: Testing framework for blockchain scenarios

pub mod blockchain;
pub mod block;
pub mod transaction;
pub mod zkp;
pub mod state_channels;
pub mod proof_of_work;
pub mod consensus;
pub mod smart_contract;
pub mod utils;
pub mod error;
pub mod merkle;
pub mod crypto;
pub mod monitor;
pub mod network;
pub mod storage;
pub mod wallet;
pub mod api;
pub mod sharding;
pub mod interop;
pub mod contract_toolkit;
pub mod ethereum;
pub mod did;
pub mod governance;
pub mod simulation;
pub mod performance;
pub mod security;
pub mod developer_tools;

// Re-export main types for easy access
pub use blockchain::Blockchain;
pub use block::Block;
pub use transaction::{Transaction, TransactionType};
pub use zkp::{ZKPManager, ZKProof, PrivateTransaction, ZKPStats};
pub use state_channels::{StateChannelManager, StateChannel, ChannelState, ChannelStatus, ChannelUpdate, StateChannelStats};
pub use proof_of_work::ProofOfWork;
pub use consensus::{ConsensusType, ProofOfStake, Validator, StakingTransaction};
pub use smart_contract::{SmartContract, ContractContext, ContractResult};
pub use error::{BlockchainError, Result};
pub use merkle::{MerkleTree, MerkleProof, MerkleNode};
pub use crypto::{KeyPair, PublicKey, DigitalSignature};
pub use monitor::{BlockchainMonitor, BlockchainMetrics, HealthStatus};
pub use network::{Network, NetworkMessage, Peer};
pub use storage::{BlockchainStorage, BlockchainMetadata};
pub use wallet::{WalletManager, WalletInfo, EncryptedWallet};
pub use api::{AppState, start_server, create_router};
pub use sharding::{ShardManager, Shard, ShardTransaction, CrossShardTransaction, ShardStats};
pub use interop::{CrossChainBridge, BridgeTransaction, AssetTransferRequest, AssetTransferResponse, ExternalChain};
pub use contract_toolkit::{ContractToolkit, ContractTemplate, CompiledContract, CompilationResult, DeploymentResult};
pub use ethereum::{EthereumBridge, EthereumConfig, PendingTransfer, TransferStatus, BridgeStats};
pub use did::{DecentralizedIdentity, DIDDocument, VerificationMethod, ServiceEndpoint, DIDCreationRequest, DIDVerificationResult, DIDStats};
pub use governance::{Governance, GovernanceProposal, ProposalType, ProposalStatus, Vote, VoteChoice, ProposalCreationRequest, VoteRequest, GovernanceStats};
pub use simulation::{SimulationManager, SimulationConfig, SimulationResult, SimulationMetrics, SimulationEvent, SimulationEventType, NetworkConditions, ShardConfig, FailureScenario, NodePerformance, SimulationState};
pub use performance::{PerformanceManager, CacheManager, ParallelProcessor, MemoryOptimizer, MetricsCollector, PerformanceConfig, CacheConfig, ParallelConfig, MemoryUsage, MetricsConfig, PerformanceStats, OptimizationResult};
pub use security::{SecurityManager, CryptoManager, AuditSystem, FormalVerifier, ThreatDetector, SecurityConfig, CryptoConfig, AuditConfig, FormalVerificationConfig, ThreatDetectionConfig, SecurityStatus, SecurityAuditResult};
pub use developer_tools::{DeveloperToolsManager, Debugger, SDKGenerator, MonitoringDashboard, CodeAnalyzer, DeveloperToolsConfig, DebuggerConfig, SDKGeneratorConfig, MonitoringConfig, CodeAnalysisConfig, DeveloperToolsStatus, DeveloperReport};

/// Current version of the blockchain protocol
pub const BLOCKCHAIN_VERSION: &str = "2.0.0";

/// Default mining difficulty (number of leading zeros required)
pub const DEFAULT_DIFFICULTY: u32 = 4;

/// Maximum block size in bytes
pub const MAX_BLOCK_SIZE: usize = 1024 * 1024; // 1MB

/// Genesis block hash (hardcoded for simplicity)
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Default gas limit for smart contracts
pub const DEFAULT_GAS_LIMIT: u64 = 1_000_000;

/// Default gas price for smart contracts
pub const DEFAULT_GAS_PRICE: f64 = 0.000001;

/// Number of shards in the system
pub const NUM_SHARDS: u32 = 4;

/// Default bridge fee percentage (0.1%)
pub const DEFAULT_BRIDGE_FEE_PERCENTAGE: f64 = 0.001;

/// Default ZKP proof timeout (30 seconds)
pub const DEFAULT_ZKP_TIMEOUT: u64 = 30;

/// Default state channel timeout (1 hour)
pub const DEFAULT_STATE_CHANNEL_TIMEOUT: u64 = 3600;

/// Maximum contract size in bytes
pub const MAX_CONTRACT_SIZE: usize = 1024 * 1024; // 1MB

/// Default Ethereum gas limit
pub const DEFAULT_ETH_GAS_LIMIT: u64 = 21000;

/// Default Ethereum gas price (20 gwei)
pub const DEFAULT_ETH_GAS_PRICE: u64 = 20000000000;

/// Minimum stake required to create governance proposals
pub const MIN_PROPOSAL_STAKE: f64 = 1000.0;

/// Default governance voting period (100 blocks)
pub const DEFAULT_VOTING_PERIOD: u64 = 100;

/// Default governance quorum (50%)
pub const DEFAULT_QUORUM: f64 = 50.0;
