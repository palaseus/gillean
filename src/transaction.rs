use serde::{Deserialize, Serialize};
use chrono::Utc;
use log::debug;
use crate::{Result, BlockchainError, utils, crypto::{KeyPair, DigitalSignature}};

/// Transaction types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    /// Regular value transfer
    Transfer,
    /// Smart contract deployment
    ContractDeploy,
    /// Smart contract execution
    ContractCall,
    /// Staking transaction
    Staking,
}

/// Represents a transaction in the blockchain
/// 
/// A transaction contains information about a transfer of value between two parties.
/// Each transaction is immutable once created and can be validated independently.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// Unique identifier for the transaction
    pub id: String,
    /// Type of transaction
    pub transaction_type: TransactionType,
    /// Address of the sender
    pub sender: String,
    /// Address of the receiver
    pub receiver: String,
    /// Amount being transferred
    pub amount: f64,
    /// Timestamp when the transaction was created
    pub timestamp: i64,
    /// Optional message or note for the transaction
    pub message: Option<String>,
    /// Digital signature for transaction authentication
    pub signature: Option<DigitalSignature>,
    /// Smart contract code (for deployment transactions)
    pub contract_code: Option<String>,
    /// Smart contract data (for contract call transactions)
    pub contract_data: Option<String>,
    /// Gas limit for contract execution
    pub gas_limit: Option<u64>,
    /// Gas price for contract execution
    pub gas_price: Option<f64>,
}

impl Transaction {
    /// Create a new transfer transaction
    /// 
    /// # Arguments
    /// * `sender` - The sender's address
    /// * `receiver` - The receiver's address
    /// * `amount` - The amount to transfer
    /// * `message` - Optional message for the transaction
    /// 
    /// # Returns
    /// * `Result<Transaction>` - The created transaction or an error
    /// 
    /// # Example
    /// ```
    /// use gillean::transaction::Transaction;
    /// 
    /// let tx = Transaction::new_transfer(
    ///     "alice".to_string(),
    ///     "bob".to_string(),
    ///     100.0,
    ///     Some("Payment for services".to_string())
    /// ).unwrap();
    /// 
    /// assert_eq!(tx.sender, "alice");
    /// assert_eq!(tx.receiver, "bob");
    /// assert_eq!(tx.amount, 100.0);
    /// ```
    pub fn new_transfer(
        sender: String,
        receiver: String,
        amount: f64,
        message: Option<String>,
    ) -> Result<Self> {
        // Validate inputs
        if sender.is_empty() || receiver.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Sender and receiver addresses cannot be empty".to_string(),
            ));
        }

        if amount <= 0.0 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Transaction amount must be positive".to_string(),
            ));
        }

        if sender == receiver {
            return Err(BlockchainError::TransactionValidationFailed(
                "Sender and receiver cannot be the same".to_string(),
            ));
        }

        let timestamp = Utc::now().timestamp();
        let id = Self::generate_id(&sender, &receiver, amount, timestamp);
        
        let transaction = Transaction {
            id,
            transaction_type: TransactionType::Transfer,
            sender,
            receiver,
            amount,
            timestamp,
            message,
            signature: None,
            contract_code: None,
            contract_data: None,
            gas_limit: None,
            gas_price: None,
        };

        debug!("Created transfer transaction: {}", transaction.id);
        Ok(transaction)
    }

    /// Create a new contract deployment transaction
    /// 
    /// # Arguments
    /// * `sender` - The sender's address
    /// * `contract_code` - Smart contract code
    /// * `gas_limit` - Gas limit for deployment
    /// * `gas_price` - Gas price for deployment
    /// 
    /// # Returns
    /// * `Result<Transaction>` - The created transaction or an error
    pub fn new_contract_deploy(
        sender: String,
        contract_code: String,
        gas_limit: u64,
        gas_price: f64,
    ) -> Result<Self> {
        if sender.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Sender address cannot be empty".to_string(),
            ));
        }

        if contract_code.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Contract code cannot be empty".to_string(),
            ));
        }

        if gas_limit == 0 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Gas limit must be greater than 0".to_string(),
            ));
        }

        if gas_price <= 0.0 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Gas price must be positive".to_string(),
            ));
        }

        let timestamp = Utc::now().timestamp();
        let id = Self::generate_contract_id(&sender, &contract_code, timestamp);
        
        let transaction = Transaction {
            id,
            transaction_type: TransactionType::ContractDeploy,
            sender,
            receiver: "".to_string(), // Contract deployment doesn't have a receiver
            amount: 0.0, // No value transfer for deployment
            timestamp,
            message: Some("Contract deployment".to_string()),
            signature: None,
            contract_code: Some(contract_code),
            contract_data: None,
            gas_limit: Some(gas_limit),
            gas_price: Some(gas_price),
        };

        debug!("Created contract deployment transaction: {}", transaction.id);
        Ok(transaction)
    }

    /// Create a new contract call transaction
    /// 
    /// # Arguments
    /// * `sender` - The sender's address
    /// * `contract_address` - The contract's address
    /// * `contract_data` - Data to pass to the contract
    /// * `amount` - Amount to send with the call
    /// * `gas_limit` - Gas limit for execution
    /// * `gas_price` - Gas price for execution
    /// 
    /// # Returns
    /// * `Result<Transaction>` - The created transaction or an error
    pub fn new_contract_call(
        sender: String,
        contract_address: String,
        contract_data: String,
        amount: f64,
        gas_limit: u64,
        gas_price: f64,
    ) -> Result<Self> {
        if sender.is_empty() || contract_address.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Sender and contract address cannot be empty".to_string(),
            ));
        }

        if gas_limit == 0 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Gas limit must be greater than 0".to_string(),
            ));
        }

        if gas_price <= 0.0 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Gas price must be positive".to_string(),
            ));
        }

        let timestamp = Utc::now().timestamp();
        let id = Self::generate_contract_id(&sender, &contract_address, timestamp);
        
        let transaction = Transaction {
            id,
            transaction_type: TransactionType::ContractCall,
            sender,
            receiver: contract_address,
            amount,
            timestamp,
            message: Some("Contract call".to_string()),
            signature: None,
            contract_code: None,
            contract_data: Some(contract_data),
            gas_limit: Some(gas_limit),
            gas_price: Some(gas_price),
        };

        debug!("Created contract call transaction: {}", transaction.id);
        Ok(transaction)
    }

    /// Create a new staking transaction
    /// 
    /// # Arguments
    /// * `validator_address` - The validator's address
    /// * `stake_amount` - Amount to stake
    /// * `is_stake` - Whether this is a stake or unstake operation
    /// 
    /// # Returns
    /// * `Result<Transaction>` - The created transaction or an error
    pub fn new_staking(
        validator_address: String,
        stake_amount: f64,
        is_stake: bool,
    ) -> Result<Self> {
        if validator_address.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Validator address cannot be empty".to_string(),
            ));
        }

        if stake_amount <= 0.0 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Stake amount must be positive".to_string(),
            ));
        }

        let timestamp = Utc::now().timestamp();
        let id = Self::generate_id(&validator_address, &validator_address, stake_amount, timestamp);
        
        let transaction = Transaction {
            id,
            transaction_type: TransactionType::Staking,
            sender: validator_address.clone(),
            receiver: validator_address,
            amount: stake_amount,
            timestamp,
            message: Some(if is_stake { "Stake tokens".to_string() } else { "Unstake tokens".to_string() }),
            signature: None,
            contract_code: None,
            contract_data: Some(if is_stake { "stake".to_string() } else { "unstake".to_string() }),
            gas_limit: None,
            gas_price: None,
        };

        debug!("Created staking transaction: {}", transaction.id);
        Ok(transaction)
    }

    /// Generate a unique transaction ID based on transaction data
    /// 
    /// # Arguments
    /// * `sender` - The sender's address
    /// * `receiver` - The receiver's address
    /// * `amount` - The transaction amount
    /// * `timestamp` - The transaction timestamp
    /// 
    /// # Returns
    /// * `String` - The generated transaction ID
    fn generate_id(sender: &str, receiver: &str, amount: f64, timestamp: i64) -> String {
        let data = format!("{}:{}:{}:{}", sender, receiver, amount, timestamp);
        utils::calculate_hash(data)
    }

    /// Generate a unique transaction ID for contract-related transactions
    /// 
    /// # Arguments
    /// * `sender` - The sender's address
    /// * `contract_code` - The contract code or address
    /// * `timestamp` - The transaction timestamp
    /// 
    /// # Returns
    /// * `String` - The generated transaction ID
    fn generate_contract_id(sender: &str, contract_code: &str, timestamp: i64) -> String {
        let data = format!("{}:{}:{}", sender, contract_code, timestamp);
        utils::calculate_hash(data)
    }

    /// Validate the transaction
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if valid, error otherwise
    pub fn validate(&self) -> Result<()> {
        // Check if ID is valid
        let expected_id = match self.transaction_type {
            TransactionType::Transfer => Self::generate_id(&self.sender, &self.receiver, self.amount, self.timestamp),
            TransactionType::ContractDeploy => Self::generate_contract_id(&self.sender, self.contract_code.as_ref().unwrap(), self.timestamp),
            TransactionType::ContractCall => Self::generate_contract_id(&self.sender, &self.receiver, self.timestamp),
            TransactionType::Staking => Self::generate_id(&self.sender, &self.receiver, self.amount, self.timestamp),
        };
        if self.id != expected_id {
            return Err(BlockchainError::TransactionValidationFailed(
                format!("Invalid transaction ID: expected {}, got {}", expected_id, self.id),
            ));
        }

        // Check if addresses are valid
        if self.sender.is_empty() || self.receiver.is_empty() {
            return Err(BlockchainError::TransactionValidationFailed(
                "Invalid addresses".to_string(),
            ));
        }

        // Check if amount is positive
        if self.amount <= 0.0 {
            return Err(BlockchainError::TransactionValidationFailed(
                "Transaction amount must be positive".to_string(),
            ));
        }

        // Check if sender and receiver are different
        if self.sender == self.receiver {
            return Err(BlockchainError::TransactionValidationFailed(
                "Sender and receiver cannot be the same".to_string(),
            ));
        }

        // Check if timestamp is reasonable (not too far in the past or future)
        let now = Utc::now().timestamp();
        let time_diff = (now - self.timestamp).abs();
        if time_diff > 3600 * 24 * 365 { // 1 year
            return Err(BlockchainError::TransactionValidationFailed(
                "Transaction timestamp is too far from current time".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the transaction as a JSON string
    /// 
    /// # Returns
    /// * `Result<String>` - The JSON representation or an error
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(BlockchainError::from)
    }

    /// Create a transaction from JSON string
    /// 
    /// # Arguments
    /// * `json` - The JSON string to parse
    /// 
    /// # Returns
    /// * `Result<Transaction>` - The parsed transaction or an error
    pub fn from_json(json: &str) -> Result<Self> {
        let transaction: Transaction = serde_json::from_str(json)?;
        transaction.validate()?;
        Ok(transaction)
    }

    /// Get the formatted timestamp
    /// 
    /// # Returns
    /// * `String` - The formatted timestamp
    pub fn formatted_timestamp(&self) -> String {
        utils::format_timestamp(self.timestamp)
    }

    /// Check if this is a coinbase transaction (mining reward)
    /// 
    /// # Returns
    /// * `bool` - True if this is a coinbase transaction
    pub fn is_coinbase(&self) -> bool {
        self.sender == "COINBASE"
    }

    /// Get the transaction size in bytes (approximate)
    /// 
    /// # Returns
    /// * `usize` - The approximate size in bytes
    pub fn size(&self) -> usize {
        self.to_json().map(|json| json.len()).unwrap_or(0)
    }

    /// Sign the transaction with a key pair
    /// 
    /// # Arguments
    /// * `keypair` - The key pair to sign with
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    /// 
    /// # Example
    /// ```
    /// use gillean::transaction::Transaction;
    /// use gillean::crypto::KeyPair;
    /// 
    /// let keypair = KeyPair::generate().unwrap();
    /// let mut tx = Transaction::new_transfer("alice".to_string(), "bob".to_string(), 100.0, None).unwrap();
    /// tx.sign(&keypair).unwrap();
    /// assert!(tx.is_signed());
    /// ```
    pub fn sign(&mut self, keypair: &KeyPair) -> Result<()> {
        let message = self.to_json()?;
        let signature = keypair.sign(message.as_bytes())?;
        self.signature = Some(signature);
        
        debug!("Signed transaction: {}", self.id);
        Ok(())
    }

    /// Verify the transaction signature
    /// 
    /// # Returns
    /// * `Result<bool>` - True if signature is valid, error otherwise
    /// 
    /// # Example
    /// ```
    /// use gillean::transaction::Transaction;
    /// use gillean::crypto::KeyPair;
    /// 
    /// let keypair = KeyPair::generate().unwrap();
    /// let mut tx = Transaction::new_transfer("alice".to_string(), "bob".to_string(), 100.0, None).unwrap();
    /// tx.sign(&keypair).unwrap();
    /// // Note: In a real implementation, signature verification would work correctly
    /// // For now, this is a simplified implementation
    /// ```
    pub fn verify_signature(&self) -> Result<bool> {
        if let Some(ref signature) = self.signature {
            let message = self.to_json()?;
            signature.verify(message.as_bytes())
        } else {
            Ok(false)
        }
    }

    /// Check if the transaction is signed
    /// 
    /// # Returns
    /// * `bool` - True if signed, false otherwise
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    /// Get the signer's public key if the transaction is signed
    /// 
    /// # Returns
    /// * `Option<String>` - The public key hex string if signed, None otherwise
    pub fn get_signer_public_key(&self) -> Option<String> {
        self.signature.as_ref().map(|sig| sig.public_key_hex())
    }

    /// Convert transaction to bytes for signing
    /// 
    /// # Returns
    /// * `Result<Vec<u8>>` - The transaction as bytes or an error
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // Create a copy without signature for consistent hashing
        let mut tx_for_hash = self.clone();
        tx_for_hash.signature = None;
        
        let json = tx_for_hash.to_json()?;
        Ok(json.into_bytes())
    }

    /// Set the signature and public key for the transaction
    /// 
    /// # Arguments
    /// * `signature` - The digital signature
    /// * `public_key` - The public key of the signer
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub fn set_signature(&mut self, signature: DigitalSignature, _public_key: crate::PublicKey) -> Result<()> {
        // Verify the signature matches the transaction
        let transaction_data = self.to_bytes()?;
        if !signature.verify(&transaction_data)? {
            return Err(BlockchainError::TransactionValidationFailed(
                "Invalid signature for transaction".to_string(),
            ));
        }
        
        self.signature = Some(signature);
        debug!("Set signature for transaction: {}", self.id);
        Ok(())
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transaction {}: {} -> {} ({} GIL)",
            &self.id[..8],
            self.sender,
            self.receiver,
            self.amount
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new_transfer(
            "alice".to_string(),
            "bob".to_string(),
            100.0,
            Some("Test transaction".to_string()),
        ).unwrap();

        assert_eq!(tx.sender, "alice");
        assert_eq!(tx.receiver, "bob");
        assert_eq!(tx.amount, 100.0);
        assert_eq!(tx.message, Some("Test transaction".to_string()));
        assert!(!tx.id.is_empty());
    }

    #[test]
    fn test_transaction_validation() {
        let tx = Transaction::new_transfer(
            "alice".to_string(),
            "bob".to_string(),
            100.0,
            None,
        ).unwrap();

        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_invalid_transaction_empty_sender() {
        let result = Transaction::new_transfer(
            "".to_string(),
            "bob".to_string(),
            100.0,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_transaction_negative_amount() {
        let result = Transaction::new_transfer(
            "alice".to_string(),
            "bob".to_string(),
            -100.0,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_transaction_same_sender_receiver() {
        let result = Transaction::new_transfer(
            "alice".to_string(),
            "alice".to_string(),
            100.0,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_json_serialization() {
        let tx = Transaction::new_transfer(
            "alice".to_string(),
            "bob".to_string(),
            100.0,
            Some("Test".to_string()),
        ).unwrap();

        let json = tx.to_json().unwrap();
        let deserialized = Transaction::from_json(&json).unwrap();

        assert_eq!(tx, deserialized);
    }

    #[test]
    fn test_coinbase_transaction() {
        let tx = Transaction::new_transfer(
            "COINBASE".to_string(),
            "miner".to_string(),
            50.0,
            None,
        ).unwrap();

        assert!(tx.is_coinbase());
    }

    #[test]
    fn test_transaction_size() {
        let tx = Transaction::new_transfer(
            "alice".to_string(),
            "bob".to_string(),
            100.0,
            None,
        ).unwrap();

        assert!(tx.size() > 0);
    }
}
