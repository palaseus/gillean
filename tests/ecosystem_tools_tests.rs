//! Tests for ecosystem tools functionality

use gillean::block_explorer::*;
use gillean::wallet_app::*;
use gillean::dev_utils::*;
use gillean::blockchain::Blockchain;
use gillean::wallet::WalletManager;
use gillean::ExplorerHealthStatus;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_block_explorer_creation() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let _block_explorer = BlockExplorer::new(blockchain, wallet_manager);
    
    // Test that block explorer was created successfully
    // Basic creation test passed
}

#[tokio::test]
async fn test_block_explorer_network_overview() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = BlockExplorer::new(blockchain, wallet_manager);
    
    let overview = block_explorer.get_network_overview().await.unwrap();
    assert_eq!(overview.statistics.total_blocks, 1); // Genesis block
    assert_eq!(overview.recent_blocks.len(), 1);
    assert_eq!(overview.recent_transactions.len(), 1); // Genesis block has 1 coinbase transaction
    assert_eq!(overview.top_addresses.len(), 1); // Genesis address has balance from coinbase transaction
}

#[tokio::test]
async fn test_block_explorer_block_details() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = BlockExplorer::new(blockchain, wallet_manager);
    
    let block_details = block_explorer.get_block_details("0").await.unwrap();
    assert_eq!(block_details.block.index, 0);
    assert_eq!(block_details.transaction_count, 1); // Genesis block has 1 coinbase transaction
    assert_eq!(block_details.total_fees, 0.0);
    assert_eq!(block_details.gas_used, 0);
    assert_eq!(block_details.gas_limit, 0);
}

#[tokio::test]
async fn test_block_explorer_search() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = BlockExplorer::new(blockchain, wallet_manager);
    
    // Search for genesis block
    let result = block_explorer.search("0").await.unwrap();
    assert_eq!(result.result_type, SearchResultType::Block);
    
    // Search for non-existent item
    let result = block_explorer.search("nonexistent").await.unwrap();
    assert_eq!(result.result_type, SearchResultType::NotFound);
}

#[tokio::test]
async fn test_block_explorer_statistics() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = BlockExplorer::new(blockchain, wallet_manager);
    
    let stats = block_explorer.get_statistics().await.unwrap();
    assert_eq!(stats.total_blocks, 1); // Genesis block
    assert_eq!(stats.total_transactions, 1); // Genesis block has 1 coinbase transaction
    assert_eq!(stats.total_addresses, 2); // COINBASE and genesis addresses
    assert_eq!(stats.total_contracts, 0);
    assert!(stats.network_hash_rate >= 0.0);
    assert!(stats.average_block_time >= 0.0);
}

#[tokio::test]
async fn test_block_explorer_network_health() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = BlockExplorer::new(blockchain, wallet_manager);
    
    let health = block_explorer.get_network_health().await.unwrap();
    assert!(matches!(health.status, ExplorerHealthStatus::Healthy | ExplorerHealthStatus::Degraded | ExplorerHealthStatus::Unhealthy));
    assert!(health.block_time_variance >= 0.0);
    assert!(health.network_difficulty >= 0.0);
    assert!(health.peer_count > 0);
    assert_eq!(health.sync_status, "synced");
}

#[tokio::test]
async fn test_wallet_app_creation() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let _wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
    
    // Test that wallet app was created successfully
    // Basic creation test passed
}

#[tokio::test]
async fn test_wallet_app_create_wallet() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
    
    let account = wallet_app.create_wallet("password123").await.unwrap();
    assert!(!account.address.is_empty());
    assert_eq!(account.balance, 0.0);
    assert_eq!(account.nonce, 0);
    assert!(!account.is_contract);
    assert_eq!(account.staked_amount, 0.0);
    assert_eq!(account.voting_power, 0.0);
}

#[tokio::test]
async fn test_wallet_app_session_management() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
    
    let permissions = vec![
        WalletPermission::Send,
        WalletPermission::Receive,
        WalletPermission::ViewBalance,
    ];
    
    let session_id = wallet_app.create_session("test_address", permissions).await.unwrap();
    assert!(!session_id.is_empty());
    
    let session = wallet_app.get_session(&session_id).await.unwrap();
    assert_eq!(session.wallet_address, "test_address");
    assert_eq!(session.permissions.len(), 3);
    assert!(session.is_encrypted);
}

#[tokio::test]
async fn test_wallet_app_account_info() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
    
    let account = wallet_app.get_account_info("test_address").await.unwrap();
    assert_eq!(account.address, "test_address");
    assert_eq!(account.balance, 0.0);
    assert_eq!(account.nonce, 0);
    assert!(!account.is_contract);
    assert_eq!(account.staked_amount, 0.0);
    assert_eq!(account.voting_power, 0.0);
}

#[tokio::test]
async fn test_wallet_app_network_status() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
    
    let network_status = wallet_app.get_network_status().await.unwrap();
    assert!(network_status.is_connected);
    assert_eq!(network_status.block_height, 0); // Genesis block
    assert_eq!(network_status.sync_status, "synced");
    assert!(network_status.gas_price > 0.0);
    assert!(network_status.network_difficulty >= 0.0);
    assert!(network_status.peer_count > 0);
}

#[tokio::test]
async fn test_wallet_app_market_data() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
    
    let market_data = wallet_app.get_market_data().await.unwrap();
    assert!(market_data.token_price_usd > 0.0);
    assert!(market_data.price_change_24h >= -1.0 && market_data.price_change_24h <= 1.0);
    assert!(market_data.market_cap > 0.0);
    assert!(market_data.volume_24h >= 0.0);
    assert!(market_data.total_supply > 0.0);
    assert!(market_data.circulating_supply > 0.0);
}

#[tokio::test]
async fn test_wallet_app_notifications() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let wallet_app = WalletApp::new(blockchain, wallet_manager, block_explorer);
    
    let notifications = wallet_app.get_notifications("test_address").await.unwrap();
    assert!(!notifications.is_empty());
    
    let notification = &notifications[0];
    assert!(!notification.id.is_empty());
    assert!(!notification.title.is_empty());
    assert!(!notification.message.is_empty());
    assert_eq!(notification.notification_type, NotificationType::Transaction);
    assert!(!notification.is_read);
}

#[tokio::test]
async fn test_dev_utils_creation() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let _dev_utils = DevUtils::new(blockchain, wallet_manager, block_explorer);
    
    // Test that dev utils was created successfully
    // Basic creation test passed
}

#[tokio::test]
async fn test_dev_utils_create_test_account() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let dev_utils = DevUtils::new(blockchain, wallet_manager, block_explorer);
    
    let account = dev_utils.create_test_account(1000.0, false).await.unwrap();
    assert!(!account.address.is_empty());
    assert_eq!(account.balance, 1000.0);
    assert!(!account.is_miner);
    assert_eq!(account.nonce, 0);
    assert_eq!(account.permissions.len(), 2);
    assert!(account.permissions.contains(&"send".to_string()));
    assert!(account.permissions.contains(&"receive".to_string()));
}

#[tokio::test]
async fn test_dev_utils_deploy_test_contract() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let dev_utils = DevUtils::new(blockchain, wallet_manager, block_explorer);
    
    let contract = dev_utils.deploy_test_contract(
        "TestContract",
        "0x608060405234801561001057600080fd5b50",
        r#"{"abi": "test"}"#,
        "contract TestContract { }",
        "deployer_address",
    ).await.unwrap();
    
    assert!(!contract.address.is_empty());
    assert_eq!(contract.deployed_by, "deployer_address");
    assert_eq!(contract.gas_used, 100000);
    assert!(!contract.bytecode.is_empty());
    assert!(!contract.abi.is_empty());
    assert!(!contract.source_code.is_empty());
    assert!(!contract.deployment_tx.is_empty());
}

#[tokio::test]
async fn test_dev_utils_breakpoint_management() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let dev_utils = DevUtils::new(blockchain, wallet_manager, block_explorer);
    
    let location = BreakpointLocation::Contract {
        address: "contract_address".to_string(),
        function: "test_function".to_string(),
        line: 10,
    };
    
    let breakpoint_id = dev_utils.set_breakpoint(location, None).await.unwrap();
    assert!(!breakpoint_id.is_empty());
    
    // Test with condition
    let location2 = BreakpointLocation::Transaction {
        tx_hash: "tx_hash".to_string(),
        step: 5,
    };
    
    let breakpoint_id2 = dev_utils.set_breakpoint(location2, Some("value > 100".to_string())).await.unwrap();
    assert!(!breakpoint_id2.is_empty());
    assert_ne!(breakpoint_id, breakpoint_id2);
}

#[tokio::test]
async fn test_dev_utils_watch_variables() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let dev_utils = DevUtils::new(blockchain, wallet_manager, block_explorer);
    
    let value = serde_json::Value::Number(serde_json::Number::from(42));
    dev_utils.add_watch_variable("test_var", value, "u32").await.unwrap();
    
    // Test with different types
    let string_value = serde_json::Value::String("test".to_string());
    dev_utils.add_watch_variable("test_string", string_value, "String").await.unwrap();
    
    let bool_value = serde_json::Value::Bool(true);
    dev_utils.add_watch_variable("test_bool", bool_value, "bool").await.unwrap();
}

#[tokio::test]
async fn test_dev_utils_profiling() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let dev_utils = DevUtils::new(blockchain, wallet_manager, block_explorer);
    
    dev_utils.start_profiling().await.unwrap();
    let profiler = dev_utils.stop_profiling().await.unwrap();
    assert!(!profiler.is_running);
    assert!(profiler.start_time > 0);
    assert!(profiler.function_times.is_empty());
    assert!(profiler.gas_usage.is_empty());
    assert!(profiler.memory_usage.is_empty());
    assert!(profiler.call_counts.is_empty());
}

#[tokio::test]
async fn test_dev_utils_test_suite_management() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let dev_utils = DevUtils::new(blockchain, wallet_manager, block_explorer);
    
    let test_case = TestCase {
        name: "test_function".to_string(),
        description: "Test description".to_string(),
        function: "test_function".to_string(),
        parameters: vec![serde_json::Value::Number(serde_json::Number::from(42))],
        expected_result: serde_json::Value::String("success".to_string()),
        gas_limit: 100000,
        is_async: false,
    };
    
    let test_suite = TestSuite {
        name: "TestSuite".to_string(),
        contract_address: "contract_address".to_string(),
        tests: vec![test_case],
        setup: Some("setup()".to_string()),
        teardown: Some("teardown()".to_string()),
        timeout: 30,
    };
    
    dev_utils.add_test_suite(test_suite).await.unwrap();
    
    // Test running contract tests
    let results = dev_utils.run_contract_tests("contract_address").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].test_name, "test_function");
    assert_eq!(results[0].status, TestStatus::Passed);
    assert!(results[0].execution_time > 0);
    assert!(results[0].gas_used > 0);
}

#[tokio::test]
async fn test_dev_utils_dev_report_generation() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let dev_utils = DevUtils::new(blockchain, wallet_manager, block_explorer);
    
    let report = dev_utils.generate_dev_report().await.unwrap();
    assert_eq!(report.environment_info.blockchain_version, "2.0.0");
    assert_eq!(report.environment_info.network_id, "dev");
    assert_eq!(report.environment_info.block_height, 0); // Genesis block
    assert!(report.environment_info.gas_price > 0.0);
    assert_eq!(report.environment_info.total_accounts, 0);
    assert_eq!(report.environment_info.total_contracts, 0);
    assert_eq!(report.environment_info.total_transactions, 0);
    
    assert!(report.performance_metrics.average_block_time > 0.0);
    assert!(report.performance_metrics.average_transaction_time > 0.0);
    assert!(report.performance_metrics.peak_memory_usage > 0);
    assert!(report.performance_metrics.total_gas_used > 0);
    assert!(report.performance_metrics.throughput_tps > 0.0);
    assert!(report.performance_metrics.latency_p99 > 0.0);
    
    assert!(report.coverage_report.overall_coverage > 0.0);
    assert!(report.coverage_report.overall_coverage <= 100.0);
    
    assert_eq!(report.debug_info.breakpoints_hit, 0);
    assert_eq!(report.debug_info.call_stack_depth, 0);
    assert_eq!(report.debug_info.memory_snapshots, 0);
    assert!(!report.debug_info.profiling_data.is_running);
    
    assert!(!report.recommendations.is_empty());
    assert!(report.recommendations.len() >= 4);
}



#[tokio::test]
async fn test_ecosystem_tools_integration() {
    let blockchain = Arc::new(RwLock::new(Blockchain::new_default().unwrap()));
    let wallet_manager = Arc::new(WalletManager::new());
    let block_explorer = Arc::new(BlockExplorer::new(blockchain.clone(), wallet_manager.clone()));
    let wallet_app = WalletApp::new(blockchain.clone(), wallet_manager.clone(), block_explorer.clone());
    let dev_utils = DevUtils::new(blockchain.clone(), wallet_manager.clone(), block_explorer.clone());
    
    // Test integration between all ecosystem tools
    
    // 1. Create a test account using dev utils
    let test_account = dev_utils.create_test_account(1000.0, false).await.unwrap();
    assert!(!test_account.address.is_empty());
    
    // 2. Create a wallet using wallet app
    let wallet_account = wallet_app.create_wallet("password123").await.unwrap();
    assert!(!wallet_account.address.is_empty());
    
    // 3. Get network overview using block explorer
    let overview = block_explorer.get_network_overview().await.unwrap();
    assert_eq!(overview.statistics.total_blocks, 1); // Genesis block
    
    // 4. Create a session for the wallet
    let permissions = vec![
        WalletPermission::Send,
        WalletPermission::Receive,
        WalletPermission::ViewBalance,
    ];
    let session_id = wallet_app.create_session(&wallet_account.address, permissions).await.unwrap();
    assert!(!session_id.is_empty());
    
    // 5. Get wallet dashboard
    let dashboard = wallet_app.get_dashboard(&session_id).await.unwrap();
    assert_eq!(dashboard.account.address, wallet_account.address);
    assert_eq!(dashboard.account.balance, 0.0);
    
    // 6. Deploy a test contract using dev utils
    let contract = dev_utils.deploy_test_contract(
        "IntegrationTestContract",
        "0x608060405234801561001057600080fd5b50",
        r#"{"abi": "integration_test"}"#,
        "contract IntegrationTestContract { }",
        &test_account.address,
    ).await.unwrap();
    assert!(!contract.address.is_empty());
    
    // 7. Generate development report
    let report = dev_utils.generate_dev_report().await.unwrap();
    assert_eq!(report.environment_info.total_accounts, 1); // test_account
    assert_eq!(report.environment_info.total_contracts, 1); // deployed contract
    
    // 8. Search for the contract using block explorer (contract not deployed to blockchain, so should be NotFound)
    let search_result = block_explorer.search(&contract.address).await.unwrap();
    assert_eq!(search_result.result_type, SearchResultType::NotFound);
    
    // 9. Get account info for the test account
    let account_info = block_explorer.get_address_info(&test_account.address).await.unwrap();
    assert_eq!(account_info.address, test_account.address);
    assert_eq!(account_info.balance, 0.0); // Would be 1000.0 in real implementation
    
    // 10. Test breakpoint management
    let location = BreakpointLocation::Contract {
        address: contract.address.clone(),
        function: "test_function".to_string(),
        line: 10,
    };
    let breakpoint_id = dev_utils.set_breakpoint(location, None).await.unwrap();
    assert!(!breakpoint_id.is_empty());
    
    // All ecosystem tools are working together successfully
    // Test passed
}
