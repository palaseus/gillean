use serde::{Deserialize, Serialize};
use chrono::Utc;
use log::{debug, info};
use crate::{Result, BlockchainError, Transaction, ProofOfWork, utils, merkle::MerkleTree, crypto::DigitalSignature, GENESIS_HASH, MAX_BLOCK_SIZE};

/// Represents a block in the blockchain
/// 
/// Each block contains a list of transactions and is linked to the previous block
/// through its hash. The block is immutable once created and mined.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    /// Block index in the chain
    pub index: u64,
    /// Timestamp when the block was created
    pub timestamp: i64,
    /// List of transactions in this block
    pub transactions: Vec<Transaction>,
    /// Hash of the previous block
    pub previous_hash: String,
    /// Hash of this block
    pub hash: String,
    /// Nonce used for proof of work
    pub nonce: u64,
    /// Merkle tree for efficient transaction verification
    pub merkle_tree: Option<MerkleTree>,
    /// Block version
    pub version: String,
    /// Validator address (for PoS consensus)
    pub validator: Option<String>,
    /// Validator signature (for PoS consensus)
    pub validator_signature: Option<DigitalSignature>,
    /// Consensus type used for this block
    pub consensus_type: String,
}

impl Block {
    /// Create a new block
    /// 
    /// # Arguments
    /// * `index` - Block index in the chain
    /// * `transactions` - List of transactions to include
    /// * `previous_hash` - Hash of the previous block
    /// * `version` - Block version
    /// * `consensus_type` - Type of consensus used
    /// 
    /// # Returns
    /// * `Result<Block>` - The created block or an error
    /// 
    /// # Example
    /// ```
    /// use gillean::block::Block;
    /// use gillean::transaction::Transaction;
    /// 
    /// let tx = Transaction::new_transfer("alice".to_string(), "bob".to_string(), 100.0, None).unwrap();
    /// let block = Block::new(1, vec![tx], "previous_hash".to_string(), "1.0".to_string(), "pow".to_string()).unwrap();
    /// 
    /// assert_eq!(block.index, 1);
    /// assert_eq!(block.transactions.len(), 1);
    /// ```
    pub fn new(
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
        version: String,
        consensus_type: String,
    ) -> Result<Self> {
        // Validate inputs
        if previous_hash.is_empty() {
            return Err(BlockchainError::BlockValidationFailed(
                "Previous hash cannot be empty".to_string(),
            ));
        }

        // Validate all transactions
        for transaction in &transactions {
            transaction.validate()?;
        }

        let timestamp = Utc::now().timestamp();
        
        // Create Merkle tree from transactions
        let merkle_tree = if transactions.is_empty() {
            None
        } else {
            MerkleTree::new(&transactions).ok()
        };
        
        let hash = Self::calculate_hash(index, timestamp, &transactions, &previous_hash, 0);

        let block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash,
            nonce: 0,
            merkle_tree,
            version,
            validator: None,
            validator_signature: None,
            consensus_type,
        };

        debug!("Created block {} with {} transactions", index, block.transactions.len());
        Ok(block)
    }

    /// Create a new PoS block
    /// 
    /// # Arguments
    /// * `index` - Block index in the chain
    /// * `transactions` - List of transactions to include
    /// * `previous_hash` - Hash of the previous block
    /// * `version` - Block version
    /// * `validator` - Validator address
    /// 
    /// # Returns
    /// * `Result<Block>` - The created block or an error
    pub fn new_pos(
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: String,
        version: String,
        validator: String,
    ) -> Result<Self> {
        let mut block = Self::new(index, transactions, previous_hash, version, "pos".to_string())?;
        block.validator = Some(validator);
        Ok(block)
    }

    /// Create the genesis block (first block in the chain)
    /// 
    /// # Returns
    /// * `Result<Block>` - The genesis block or an error
    pub fn genesis() -> Result<Self> {
        let coinbase_tx = Transaction::new_transfer(
            "COINBASE".to_string(),
            "genesis".to_string(),
            1000.0,
            Some("Genesis block reward".to_string()),
        )?;

        let block = Block::new(
            0,
            vec![coinbase_tx],
            GENESIS_HASH.to_string(),
            "1.0".to_string(),
            "pow".to_string(),
        )?;

        info!("Created genesis block");
        Ok(block)
    }

    /// Calculate the hash of the current block
    /// 
    /// # Returns
    /// * `String` - The calculated hash
    pub fn calculate_current_hash(&self) -> String {
        Self::calculate_hash(
            self.index,
            self.timestamp,
            &self.transactions,
            &self.previous_hash,
            self.nonce,
        )
    }

    /// Calculate the hash of a block
    /// 
    /// # Arguments
    /// * `index` - Block index
    /// * `timestamp` - Block timestamp
    /// * `transactions` - List of transactions
    /// * `previous_hash` - Hash of previous block
    /// * `nonce` - Nonce value
    /// 
    /// # Returns
    /// * `String` - The calculated hash
    pub fn calculate_hash(
        index: u64,
        timestamp: i64,
        transactions: &[Transaction],
        previous_hash: &str,
        nonce: u64,
    ) -> String {
        // Create a simplified representation of transactions for hashing
        let tx_data: Vec<String> = transactions
            .iter()
            .map(|tx| format!("{}:{}:{}", tx.sender, tx.receiver, tx.amount))
            .collect();
        let tx_string = tx_data.join("|");

        let data = format!("{}:{}:{}:{}:{}", index, timestamp, tx_string, previous_hash, nonce);
        utils::calculate_hash(data)
    }

    /// Mine the block with proof of work
    /// 
    /// # Arguments
    /// * `pow` - Proof of work instance
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if mining successful, error otherwise
    /// 
    /// # Example
    /// ```
    /// use gillean::block::Block;
    /// use gillean::proof_of_work::ProofOfWork;
    /// 
    /// let mut block = Block::new(1, vec![], "previous_hash".to_string(), "1.0".to_string(), "pow".to_string()).unwrap();
    /// let pow = ProofOfWork::new(2, 1000).unwrap();
    /// block.mine(&pow).unwrap();
    /// assert!(block.hash.starts_with("00"));
    /// ```
    pub fn mine(&mut self, pow: &ProofOfWork) -> Result<()> {
        info!("Mining block {} with difficulty {}", self.index, pow.difficulty);

        // Prepare block data for mining (without nonce)
        let tx_data: Vec<String> = self.transactions
            .iter()
            .map(|tx| format!("{}:{}:{}", tx.sender, tx.receiver, tx.amount))
            .collect();
        let tx_string = tx_data.join("|");
        let block_data = format!("{}:{}:{}", self.index, self.timestamp, tx_string);

        // Mine the block
        let (nonce, hash) = pow.mine(&block_data, &self.previous_hash)?;

        // Update block with mining results
        self.nonce = nonce;
        self.hash = hash;

        info!("Block {} mined successfully with nonce {}", self.index, nonce);
        Ok(())
    }

    /// Validate the block
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if valid, error otherwise
    pub fn validate(&self) -> Result<()> {
        // Validate block size
        let block_size = self.size();
        if block_size > MAX_BLOCK_SIZE {
            return Err(BlockchainError::BlockTooLarge {
                size: block_size,
                limit: MAX_BLOCK_SIZE,
            });
        }

        // Validate all transactions
        for transaction in &self.transactions {
            transaction.validate()?;
        }

        // Validate hash
        let expected_hash = Self::calculate_hash(
            self.index,
            self.timestamp,
            &self.transactions,
            &self.previous_hash,
            self.nonce,
        );

        if self.hash != expected_hash {
            return Err(BlockchainError::InvalidHash(format!(
                "Block hash mismatch: expected {}, got {}",
                expected_hash, self.hash
            )));
        }

        // Validate previous hash format
        if !utils::is_valid_hex(&self.previous_hash) {
            return Err(BlockchainError::InvalidHash(
                "Previous hash is not a valid hex string".to_string(),
            ));
        }

        // Validate hash format
        if !utils::is_valid_hex(&self.hash) {
            return Err(BlockchainError::InvalidHash(
                "Block hash is not a valid hex string".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the block as a JSON string
    /// 
    /// # Returns
    /// * `Result<String>` - The JSON representation or an error
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(BlockchainError::from)
    }

    /// Create a block from JSON string
    /// 
    /// # Arguments
    /// * `json` - The JSON string to parse
    /// 
    /// # Returns
    /// * `Result<Block>` - The parsed block or an error
    pub fn from_json(json: &str) -> Result<Self> {
        let block: Block = serde_json::from_str(json)?;
        block.validate()?;
        Ok(block)
    }

    /// Get the formatted timestamp
    /// 
    /// # Returns
    /// * `String` - The formatted timestamp
    pub fn formatted_timestamp(&self) -> String {
        utils::format_timestamp(self.timestamp)
    }

    /// Check if this is the genesis block
    /// 
    /// # Returns
    /// * `bool` - True if this is the genesis block
    pub fn is_genesis(&self) -> bool {
        self.index == 0 && self.previous_hash == GENESIS_HASH
    }

    /// Get the total amount of transactions in this block
    /// 
    /// # Returns
    /// * `f64` - Total transaction amount
    pub fn total_amount(&self) -> f64 {
        self.transactions.iter().map(|tx| tx.amount).sum()
    }

    /// Get the block size in bytes (approximate)
    /// 
    /// # Returns
    /// * `usize` - The approximate size in bytes
    pub fn size(&self) -> usize {
        self.to_json().map(|json| json.len()).unwrap_or(0)
    }

    /// Get a short hash for display purposes
    /// 
    /// # Returns
    /// * `String` - First 8 characters of the hash
    pub fn short_hash(&self) -> String {
        self.hash[..8].to_string()
    }

    /// Get the number of transactions in this block
    /// 
    /// # Returns
    /// * `usize` - Number of transactions
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// Get the Merkle root hash
    /// 
    /// # Returns
    /// * `Option<String>` - The Merkle root hash if available, None otherwise
    pub fn merkle_root(&self) -> Option<String> {
        self.merkle_tree.as_ref().and_then(|tree| tree.root_hash())
    }

    /// Verify a transaction is included in this block using Merkle proof
    /// 
    /// # Arguments
    /// * `transaction` - The transaction to verify
    /// * `index` - Index of the transaction in the block
    /// 
    /// # Returns
    /// * `Result<bool>` - True if transaction is verified, error otherwise
    pub fn verify_transaction_inclusion(&self, transaction: &Transaction, index: usize) -> Result<bool> {
        if let Some(ref merkle_tree) = self.merkle_tree {
            let proof = merkle_tree.generate_proof(index)?;
            merkle_tree.verify_transaction(transaction, &proof, index)
        } else {
            Ok(false)
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Block #{} ({}): {} txs, hash: {}",
            self.index,
            self.formatted_timestamp(),
            self.transaction_count(),
            self.short_hash()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let tx = Transaction::new_transfer("alice".to_string(), "bob".to_string(), 100.0, None).unwrap();
        let block = Block::new(1, vec![tx], "0000000000000000000000000000000000000000000000000000000000000000".to_string(), "1.0".to_string(), "pow".to_string()).unwrap();

        assert_eq!(block.index, 1);
        assert_eq!(block.transactions.len(), 1);
        assert_eq!(block.previous_hash, "0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis().unwrap();
        assert_eq!(genesis.index, 0);
        assert!(genesis.is_genesis());
        assert_eq!(genesis.previous_hash, GENESIS_HASH);
    }

    #[test]
    fn test_block_mining() {
        let mut block = Block::new(1, vec![], "0000000000000000000000000000000000000000000000000000000000000000".to_string(), "1.0".to_string(), "pow".to_string()).unwrap();
        let pow = ProofOfWork::new(1, 1000).unwrap();
        
        block.mine(&pow).unwrap();
        assert!(block.hash.starts_with('0'));
        assert!(block.nonce > 0);
    }

    #[test]
    fn test_block_validation() {
        let block = Block::new(1, vec![], "0000000000000000000000000000000000000000000000000000000000000000".to_string(), "1.0".to_string(), "pow".to_string()).unwrap();
        assert!(block.validate().is_ok());
    }

    #[test]
    fn test_block_json_serialization() {
        let block = Block::new(1, vec![], "0000000000000000000000000000000000000000000000000000000000000000".to_string(), "1.0".to_string(), "pow".to_string()).unwrap();
        let json = block.to_json().unwrap();
        let deserialized = Block::from_json(&json).unwrap();

        assert_eq!(block.index, deserialized.index);
        assert_eq!(block.previous_hash, deserialized.previous_hash);
    }

    #[test]
    fn test_block_total_amount() {
        let tx1 = Transaction::new_transfer("alice".to_string(), "bob".to_string(), 100.0, None).unwrap();
        let tx2 = Transaction::new_transfer("bob".to_string(), "charlie".to_string(), 50.0, None).unwrap();
        
        let block = Block::new(1, vec![tx1, tx2], "0000000000000000000000000000000000000000000000000000000000000000".to_string(), "1.0".to_string(), "pow".to_string()).unwrap();
        assert_eq!(block.total_amount(), 150.0);
    }

    #[test]
    fn test_block_size() {
        let block = Block::new(1, vec![], "0000000000000000000000000000000000000000000000000000000000000000".to_string(), "1.0".to_string(), "pow".to_string()).unwrap();
        assert!(block.size() > 0);
    }

    #[test]
    fn test_short_hash() {
        let block = Block::new(1, vec![], "0000000000000000000000000000000000000000000000000000000000000000".to_string(), "1.0".to_string(), "pow".to_string()).unwrap();
        let short = block.short_hash();
        assert_eq!(short.len(), 8);
    }
}
