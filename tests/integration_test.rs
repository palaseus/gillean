use gillean::{
    Blockchain, Result, Transaction, TransactionType, ConsensusType,
    DEFAULT_GAS_LIMIT, DEFAULT_GAS_PRICE
};
use gillean::smart_contract::examples;
use tempfile::TempDir;

#[tokio::test]
async fn test_smart_contract_deployment_and_execution() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let _db_path = temp_dir.path().join("test_db");
    
    // Create a new blockchain
    let mut blockchain = Blockchain::new_pow(2, 50.0)?;
    
    // Give alice and bob some initial balance using coinbase
    blockchain.add_transaction_object(Transaction::new_transfer(
        "COINBASE".to_string(),
        "alice".to_string(),
        1000.0,
        None,
    )?)?;
    
    blockchain.add_transaction_object(Transaction::new_transfer(
        "COINBASE".to_string(),
        "bob".to_string(),
        1000.0,
        None,
    )?)?;
    
    // Mine a block to process the initial transactions
    blockchain.mine_block("miner".to_string())?;
    
        // Deploy a simple counter contract
    let contract_code = r#"
# Simple Counter Contract
PUSH 0
STORE counter
PUSH 1
STORE increment_value
RETURN
"#;

    let contract_address = blockchain.deploy_contract(
        "alice".to_string(),
        contract_code.to_string(),
        DEFAULT_GAS_LIMIT,
        DEFAULT_GAS_PRICE,
    )?;
    
    // Verify contract was deployed
    let contract = blockchain.get_contract(&contract_address);
    assert!(contract.is_some());
    
    // Call the contract
    let _result = blockchain.call_contract(
        "bob".to_string(),
        contract_address.clone(),
        "increment".to_string(),
        1.0, // Small amount for the call
        DEFAULT_GAS_LIMIT,
        DEFAULT_GAS_PRICE,
    )?;
    
    // Verify contract execution
    let contract = blockchain.get_contract(&contract_address).unwrap();
    assert!(contract.storage.contains_key("counter"));
    
    Ok(())
}

#[tokio::test]
async fn test_proof_of_stake_consensus() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let _db_path = temp_dir.path().join("test_db");
    
    // Create a PoS blockchain
    let mut blockchain = Blockchain::new_pos(50.0, 100.0, 10)?;
    
    // Register validators
    blockchain.register_validator(
        "validator1".to_string(),
        "validator1".to_string(),
        1000.0,
    )?;
    
    blockchain.register_validator(
        "validator2".to_string(),
        "validator2".to_string(),
        2000.0,
    )?;
    
    // Verify validators were registered
    let pos_stats = blockchain.get_pos_stats();
    assert!(pos_stats.is_some());
    
    let stats = pos_stats.unwrap();
    assert_eq!(stats.get("total_validators").unwrap(), &2.0);
    assert_eq!(stats.get("total_stake").unwrap(), &3000.0);
    
    // Test validator selection
    let selected_validator = blockchain.select_validator();
    assert!(selected_validator.is_some());
    
    let validator = selected_validator.unwrap();
    assert!(validator == "validator1" || validator == "validator2");
    
    Ok(())
}

#[tokio::test]
async fn test_transaction_types() -> Result<()> {
    let _blockchain = Blockchain::new_pow(2, 50.0)?;
    
    // Test regular transfer transaction
    let transfer_tx = Transaction::new_transfer(
        "alice".to_string(),
        "bob".to_string(),
        100.0,
        Some("Payment".to_string()),
    )?;
    
    assert_eq!(transfer_tx.transaction_type, TransactionType::Transfer);
    assert_eq!(transfer_tx.amount, 100.0);
    
    // Test contract deployment transaction
    let contract_tx = Transaction::new_contract_deploy(
        "alice".to_string(),
        "PUSH 0\nSTORE counter\nRETURN".to_string(),
        DEFAULT_GAS_LIMIT,
        DEFAULT_GAS_PRICE,
    )?;
    
    assert_eq!(contract_tx.transaction_type, TransactionType::ContractDeploy);
    assert!(contract_tx.contract_code.is_some());
    
    // Test contract call transaction
    let call_tx = Transaction::new_contract_call(
        "bob".to_string(),
        "contract_address".to_string(),
        "increment".to_string(),
        0.0,
        DEFAULT_GAS_LIMIT,
        DEFAULT_GAS_PRICE,
    )?;
    
    assert_eq!(call_tx.transaction_type, TransactionType::ContractCall);
    assert!(call_tx.contract_data.is_some());
    
    // Test staking transaction
    let stake_tx = Transaction::new_staking(
        "validator1".to_string(),
        1000.0,
        true,
    )?;
    
    assert_eq!(stake_tx.transaction_type, TransactionType::Staking);
    assert_eq!(stake_tx.amount, 1000.0);
    
    Ok(())
}

#[tokio::test]
async fn test_blockchain_with_contracts_and_pos() -> Result<()> {
    let mut blockchain = Blockchain::new_pos(50.0, 100.0, 5)?;
    
    // Give alice some initial balance using coinbase
    blockchain.add_transaction_object(Transaction::new_transfer(
        "COINBASE".to_string(),
        "alice".to_string(),
        1000.0,
        None,
    )?)?;
    
    // Register a validator first (required for PoS)
    blockchain.register_validator(
        "validator1".to_string(),
        "validator1".to_string(),
        1000.0,
    )?;
    
    // Mine a block to process the initial transaction
    blockchain.mine_block("validator1".to_string())?;
    
        // Deploy a contract
    let contract_code = r#"
# Crowdfunding Contract
PUSH 1000
STORE goal
PUSH 0
STORE total_raised
PUSH 0
STORE funded
RETURN
"#;

    let _contract_address = blockchain.deploy_contract(
        "alice".to_string(),
        contract_code.to_string(),
        DEFAULT_GAS_LIMIT,
        DEFAULT_GAS_PRICE,
    )?;
    
    // Create a transaction
    let tx = Transaction::new_transfer(
        "alice".to_string(),
        "bob".to_string(),
        50.0,
        None,
    )?;
    
    blockchain.add_transaction_object(tx)?;
    
    // Verify blockchain state
    assert_eq!(blockchain.get_consensus_type(), ConsensusType::ProofOfStake);
    assert_eq!(blockchain.get_contracts().len(), 1);
    assert_eq!(blockchain.pending_transactions.len(), 1);
    
    // Check contract metrics
    let metrics = blockchain.get_contract_metrics();
    assert_eq!(metrics.get("deployments").unwrap(), &1);
    
    Ok(())
}

#[tokio::test]
async fn test_contract_examples() -> Result<()> {
    let _blockchain = Blockchain::new_pow(2, 50.0)?;
    
    // Test crowdfunding contract
    let crowdfunding_contract = examples::crowdfunding_contract(1000.0, 1234567890);
    assert_eq!(crowdfunding_contract.owner, "crowdfunding_owner");
    assert!(crowdfunding_contract.code.contains("1000"));
    
    // Test multisig contract
    let multisig_contract = examples::multisig_contract(3);
    assert_eq!(multisig_contract.owner, "multisig_owner");
    assert!(multisig_contract.code.contains("3"));
    
    // Test timelock contract
    let timelock_contract = examples::timelock_contract(1234567890);
    assert_eq!(timelock_contract.owner, "timelock_owner");
    assert!(timelock_contract.code.contains("1234567890"));
    
    Ok(())
}

#[tokio::test]
async fn test_pos_validator_performance() -> Result<()> {
    let mut blockchain = Blockchain::new_pos(50.0, 100.0, 5)?;
    
    // Register a validator
    blockchain.register_validator(
        "validator1".to_string(),
        "validator1".to_string(),
        1000.0,
    )?;
    
    // Get validator stats
    let stats = blockchain.get_pos_stats().unwrap();
    assert_eq!(stats.get("total_validators").unwrap(), &1.0);
    assert_eq!(stats.get("active_validators").unwrap(), &1.0);
    assert_eq!(stats.get("total_stake").unwrap(), &1000.0);
    
    // Test validator selection multiple times
    let mut selections = Vec::new();
    for _ in 0..10 {
        if let Some(validator) = blockchain.select_validator() {
            selections.push(validator);
        }
    }
    
    // Should always select the same validator since there's only one
    assert!(!selections.is_empty());
    assert!(selections.iter().all(|v| v == "validator1"));
    
    Ok(())
}
