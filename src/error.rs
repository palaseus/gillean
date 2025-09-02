use std::fmt;

/// Custom error types for the blockchain implementation
#[derive(Debug, Clone)]
pub enum BlockchainError {
    /// Invalid block hash
    InvalidHash(String),
    /// Invalid proof of work
    InvalidProofOfWork(String),
    /// Invalid block index
    InvalidIndex { expected: u64, found: u64 },
    /// Invalid previous hash
    InvalidPreviousHash { expected: String, found: String },
    /// Block validation failed
    BlockValidationFailed(String),
    /// Chain validation failed
    ChainValidationFailed(String),
    /// Transaction validation failed
    TransactionValidationFailed(String),
    /// Serialization error
    SerializationError(String),
    /// Mining timeout
    MiningTimeout(u64),
    /// Invalid difficulty level
    InvalidDifficulty(u32),
    /// Block size exceeds limit
    BlockTooLarge { size: usize, limit: usize },
    /// Insufficient balance for transaction
    InsufficientBalance { address: String, balance: f64, required: f64 },
    /// Storage error
    StorageError(String),
    /// Wallet error
    WalletError(String),
    /// API error
    ApiError(String),
    /// Smart contract validation failed
    ContractValidationFailed(String),
    /// Smart contract execution error
    ContractExecutionError(String),
    /// Consensus error
    ConsensusError(String),
    /// Validator error
    ValidatorError(String),
    /// Staking error
    StakingError(String),
    /// Invalid transaction
    InvalidTransaction(String),
    /// Network error
    NetworkError(String),
    /// Sharding error
    ShardingError(String),
    /// Cross-chain error
    CrossChainError(String),
    /// Contract toolkit error
    ContractToolkitError(String),
    /// Invalid input
    InvalidInput(String),
    /// Resource not found
    NotFound(String),
    /// Invalid state
    InvalidState(String),
    /// Invalid signature
    InvalidSignature(String),
    /// State corruption detected
    StateCorruption(String),
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockchainError::InvalidHash(msg) => write!(f, "Invalid hash: {}", msg),
            BlockchainError::InvalidProofOfWork(msg) => write!(f, "Invalid proof of work: {}", msg),
            BlockchainError::InvalidIndex { expected, found } => {
                write!(f, "Invalid block index: expected {}, found {}", expected, found)
            }
            BlockchainError::InvalidPreviousHash { expected, found } => {
                write!(f, "Invalid previous hash: expected {}, found {}", expected, found)
            }
            BlockchainError::BlockValidationFailed(msg) => write!(f, "Block validation failed: {}", msg),
            BlockchainError::ChainValidationFailed(msg) => write!(f, "Chain validation failed: {}", msg),
            BlockchainError::TransactionValidationFailed(msg) => write!(f, "Transaction validation failed: {}", msg),
            BlockchainError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            BlockchainError::MiningTimeout(attempts) => write!(f, "Mining timeout after {} attempts", attempts),
            BlockchainError::InvalidDifficulty(difficulty) => write!(f, "Invalid difficulty level: {}", difficulty),
            BlockchainError::BlockTooLarge { size, limit } => {
                write!(f, "Block too large: {} bytes (limit: {} bytes)", size, limit)
            }
            BlockchainError::InsufficientBalance { address, balance, required } => {
                write!(f, "Insufficient balance for {}: have {}, need {}", address, balance, required)
            }
            BlockchainError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            BlockchainError::WalletError(msg) => write!(f, "Wallet error: {}", msg),
            BlockchainError::ApiError(msg) => write!(f, "API error: {}", msg),
            BlockchainError::ContractValidationFailed(msg) => write!(f, "Contract validation failed: {}", msg),
            BlockchainError::ContractExecutionError(msg) => write!(f, "Contract execution error: {}", msg),
            BlockchainError::ConsensusError(msg) => write!(f, "Consensus error: {}", msg),
            BlockchainError::ValidatorError(msg) => write!(f, "Validator error: {}", msg),
            BlockchainError::StakingError(msg) => write!(f, "Staking error: {}", msg),
            BlockchainError::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            BlockchainError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            BlockchainError::ShardingError(msg) => write!(f, "Sharding error: {}", msg),
            BlockchainError::CrossChainError(msg) => write!(f, "Cross-chain error: {}", msg),
            BlockchainError::ContractToolkitError(msg) => write!(f, "Contract toolkit error: {}", msg),
            BlockchainError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            BlockchainError::NotFound(msg) => write!(f, "Not found: {}", msg),
            BlockchainError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            BlockchainError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            BlockchainError::StateCorruption(msg) => write!(f, "State corruption: {}", msg),
        }
    }
}

impl std::error::Error for BlockchainError {}

/// Result type for blockchain operations
pub type Result<T> = std::result::Result<T, BlockchainError>;

impl From<serde_json::Error> for BlockchainError {
    fn from(err: serde_json::Error) -> Self {
        BlockchainError::SerializationError(err.to_string())
    }
}

impl From<std::io::Error> for BlockchainError {
    fn from(err: std::io::Error) -> Self {
        BlockchainError::SerializationError(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for BlockchainError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        BlockchainError::ApiError(err.to_string())
    }
}

impl From<toml::de::Error> for BlockchainError {
    fn from(err: toml::de::Error) -> Self {
        BlockchainError::SerializationError(err.to_string())
    }
}

impl From<regex::Error> for BlockchainError {
    fn from(err: regex::Error) -> Self {
        BlockchainError::ContractToolkitError(err.to_string())
    }
}
