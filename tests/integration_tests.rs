// Integration_tests Test Suite
// Tests for integration tests

use gillean::Blockchain;

pub struct IntegrationTestsSuite {
    // Placeholder for integration_tests test suite
}

impl IntegrationTestsSuite {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for IntegrationTestsSuite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_suite_creation() {
        let _suite = IntegrationTestsSuite::new();
        assert!(true); // Basic test to ensure suite can be created
    }

    #[test]
    fn test_basic_blockchain_integration() {
        let mut blockchain = Blockchain::new_pow(2, 50.0).unwrap();
        
        // Add initial balance to alice first
        blockchain.add_transaction("COINBASE".to_string(), "alice".to_string(), 100.0, Some("initial balance".to_string())).unwrap();
        blockchain.mine_block("miner".to_string()).unwrap();
        
        // Now test basic transaction
        blockchain.add_transaction("alice".to_string(), "bob".to_string(), 10.0, Some("test transaction".to_string())).unwrap();
        
        // Test mining again
        blockchain.mine_block("miner".to_string()).unwrap();
        
        assert_eq!(blockchain.blocks.len(), 3); // genesis block + 2 mined blocks
        assert_eq!(blockchain.get_balance("miner"), 100.0); // 2 mining rewards
    }

    #[test]
    fn test_transaction_validation_integration() {
        let mut blockchain = Blockchain::new_pow(2, 50.0).unwrap();
        
        // Add initial balance
        blockchain.add_transaction("COINBASE".to_string(), "alice".to_string(), 100.0, Some("initial balance".to_string())).unwrap();
        blockchain.mine_block("miner".to_string()).unwrap();
        
        // Valid transaction
        blockchain.add_transaction("alice".to_string(), "bob".to_string(), 50.0, Some("valid transaction".to_string())).unwrap();
        
        // Invalid transaction (insufficient balance)
        let result = blockchain.add_transaction("alice".to_string(), "bob".to_string(), 200.0, Some("invalid transaction".to_string()));
        assert!(result.is_err());
    }
}
