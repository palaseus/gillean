use crate::{Blockchain, Transaction, BlockchainError, WalletManager, EthereumBridge, DecentralizedIdentity, Governance, SimulationManager, BlockchainStorage};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Json, IntoResponse},
    routing::{get, post},
    Router,
    // body::Body, // Unused import
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
// use std::collections::HashMap; // Unused import
use log::{info, error};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use metrics::{counter, histogram};

/// API-related errors
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Blockchain error: {0}")]
    Blockchain(String),
    
    #[error("Wallet error: {0}")]
    Wallet(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Blockchain(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Wallet(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl From<BlockchainError> for ApiError {
    fn from(err: BlockchainError) -> Self {
        ApiError::Blockchain(err.to_string())
    }
}

impl From<crate::storage::StorageError> for ApiError {
    fn from(err: crate::storage::StorageError) -> Self {
        ApiError::Internal(err.to_string())
    }
}

/// API request/response structures
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedTransactionRequest {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub message: Option<String>,
    pub signature: String,
    pub public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MineRequest {
    pub miner_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerRequest {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub password: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionRequest {
    pub from_address: String,
    pub to_address: String,
    pub amount: f64,
    pub password: String,
    pub message: Option<String>,
}

// Ethereum Integration Requests
#[derive(Debug, Serialize, Deserialize)]
pub struct EthereumTransferRequest {
    pub from_address: String,
    pub to_ethereum_address: String,
    pub amount: f64,
    pub password: String,
}

// DID Requests
#[derive(Debug, Serialize, Deserialize)]
pub struct DIDCreationRequest {
    pub controller: Option<String>,
    pub service_endpoints: Vec<crate::did::ServiceEndpoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DIDLinkRequest {
    pub wallet_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DIDVerificationRequest {
    pub message: String,
    pub signature: String,
}

// Governance Requests
#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceProposalRequest {
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub proposal_type: crate::governance::ProposalType,
    pub contract_code: Option<String>,
    pub parameters: std::collections::HashMap<String, String>,
    pub voting_period: u64,
    pub quorum: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceVoteRequest {
    pub proposal_id: String,
    pub voter: String,
    pub vote: crate::governance::VoteChoice,
    pub stake_amount: f64,
}

// Simulation Requests
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationRunRequest {
    pub config: crate::simulation::SimulationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec<crate::Block>,
    pub total_blocks: usize,
    pub total_transactions: usize,
    pub difficulty: u32,
    pub mining_reward: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockResponse {
    pub block: crate::Block,
    pub transactions: Vec<crate::Transaction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MiningResponse {
    pub block: crate::Block,
    pub mining_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeersResponse {
    pub peers: Vec<String>,
    pub total_peers: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub total_blocks: usize,
    pub total_transactions: usize,
    pub pending_transactions: usize,
    pub current_difficulty: u32,
    pub mining_reward: f64,
    pub blockchain_size_bytes: usize,
    pub uptime_seconds: u64,
    pub api_requests_total: u64,
    pub api_errors_total: u64,
}

/// Application state shared across API handlers
#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub wallet_manager: Arc<Mutex<WalletManager>>,
    pub ethereum_bridge: Option<Arc<Mutex<EthereumBridge>>>,
    pub did_system: Option<Arc<Mutex<DecentralizedIdentity>>>,
    pub governance: Option<Arc<Mutex<Governance>>>,
    pub simulation_manager: Option<Arc<Mutex<SimulationManager>>>,
    pub storage: Arc<BlockchainStorage>,
    pub storage_path: String,
    pub start_time: std::time::Instant,
}

/// Create the API router
/// 
/// # Arguments
/// * `state` - Application state
/// 
/// # Returns
/// * `Router` - The configured router
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/chain", get(get_chain))
        .route("/chain/:start/:end", get(get_chain_range))
        .route("/block/:index", get(get_block))
        .route("/transaction", post(add_transaction))
        .route("/transaction/signed", post(add_signed_transaction))
        .route("/balance/:address", get(get_balance))
        .route("/mine", post(mine_block))
        .route("/peers", get(get_peers))
        .route("/peers", post(add_peer))
        .route("/wallet", post(create_wallet))
        .route("/wallet", get(list_wallets))
        .route("/wallet/:address/balance", get(get_wallet_balance))
        .route("/transaction/send", post(send_transaction))
        .route("/metrics", get(get_metrics))
        .route("/health", get(health_check))
        // Ethereum Integration endpoints
        .route("/eth/transfer", post(ethereum_transfer))
        .route("/eth/balance/:address", get(get_ethereum_balance))
        .route("/eth/transfer/:id/status", get(get_ethereum_transfer_status))
        .route("/eth/transfers/pending", get(get_pending_ethereum_transfers))
        .route("/eth/bridge/stats", get(get_ethereum_bridge_stats))
        // DID endpoints
        .route("/did/create", post(create_did))
        .route("/did/:did", get(get_did_document))
        .route("/did/:did/link", post(link_did_to_wallet))
        .route("/did/wallet/:address", get(get_did_for_wallet))
        .route("/did/:did/verify", post(verify_did_signature))
        .route("/did/all", get(get_all_dids))
        .route("/did/stats", get(get_did_stats))
        // Governance endpoints
        .route("/governance/propose", post(create_governance_proposal))
        .route("/governance/vote", post(vote_on_proposal))
        .route("/governance/proposal/:id/execute", post(execute_proposal))
        .route("/governance/proposal/:id", get(get_governance_proposal))
        .route("/governance/proposals", get(get_all_governance_proposals))
        .route("/governance/proposal/:id/votes", get(get_proposal_votes))
        .route("/governance/stats", get(get_governance_stats))
        // Simulation endpoints
        .route("/simulation/run", post(run_simulation))
        .route("/simulation/:id/progress", get(get_simulation_progress))
        .route("/simulation/:id/state", get(get_simulation_state))
        .route("/simulation/:id/stop", post(stop_simulation))
        .route("/simulation/:id/results", get(get_simulation_results))
        .route("/simulation/all", get(get_all_simulations))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Get the full blockchain
async fn get_chain(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<ChainResponse>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_chain");
    let start = std::time::Instant::now();
    
    let blockchain = state.blockchain.lock().unwrap();
    let response = ChainResponse {
        blocks: blockchain.blocks.clone(),
        total_blocks: blockchain.blocks.len(),
        total_transactions: blockchain.blocks.iter().map(|b| b.transactions.len()).sum(),
        difficulty: blockchain.difficulty,
        mining_reward: blockchain.mining_reward,
    };
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "get_chain");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: "Blockchain retrieved successfully".to_string(),
    }))
}

/// Get a range of blocks
async fn get_chain_range(
    State(state): State<AppState>,
    Path((start, end)): Path<(usize, usize)>,
) -> std::result::Result<Json<ApiResponse<ChainResponse>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_chain_range");
    let start_time = std::time::Instant::now();
    
    let blockchain = state.blockchain.lock().unwrap();
    
    if start >= blockchain.blocks.len() || end >= blockchain.blocks.len() || start > end {
        return Err(ApiError::InvalidRequest("Invalid block range".to_string()));
    }
    
    let blocks = blockchain.blocks[start..=end].to_vec();
    let response = ChainResponse {
        blocks,
        total_blocks: end - start + 1,
        total_transactions: blockchain.blocks[start..=end].iter().map(|b| b.transactions.len()).sum(),
        difficulty: blockchain.difficulty,
        mining_reward: blockchain.mining_reward,
    };
    
    histogram!("api_request_duration_ms", start_time.elapsed().as_millis() as f64, "endpoint" => "get_chain_range");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: "Block range retrieved successfully".to_string(),
    }))
}

/// Get a specific block
async fn get_block(
    State(state): State<AppState>,
    Path(index): Path<usize>,
) -> std::result::Result<Json<ApiResponse<BlockResponse>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_block");
    let start = std::time::Instant::now();
    
    let blockchain = state.blockchain.lock().unwrap();
    
    if index >= blockchain.blocks.len() {
        return Err(ApiError::InvalidRequest("Block index out of range".to_string()));
    }
    
    let block = blockchain.blocks[index].clone();
    let response = BlockResponse {
        block: block.clone(),
        transactions: block.transactions,
    };
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "get_block");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: "Block retrieved successfully".to_string(),
    }))
}

/// Add a new transaction
async fn add_transaction(
    State(state): State<AppState>,
    Json(request): Json<TransactionRequest>,
) -> std::result::Result<Json<ApiResponse<Transaction>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "add_transaction");
    let start = std::time::Instant::now();
    
    let mut blockchain = state.blockchain.lock().unwrap();
    
    blockchain.add_transaction(
        request.sender,
        request.receiver,
        request.amount,
        request.message,
    )?;
    
    // Save to storage
    state.storage.save_blockchain(&blockchain)?;
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "add_transaction");
    
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: "Transaction added successfully".to_string(),
    }))
}

/// Add a signed transaction
async fn add_signed_transaction(
    State(state): State<AppState>,
    Json(request): Json<SignedTransactionRequest>,
) -> std::result::Result<Json<ApiResponse<Transaction>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "add_signed_transaction");
    let start = std::time::Instant::now();
    
    let mut blockchain = state.blockchain.lock().unwrap();
    
    // Create transaction
    let mut transaction = Transaction::new_transfer(
        request.sender,
        request.receiver,
        request.amount,
        request.message,
    )?;
    
    // Verify signature
    let public_key_bytes = crate::utils::hex_to_bytes(&request.public_key)?;
    let signature_bytes = crate::utils::hex_to_bytes(&request.signature)?;
    
    let public_key = crate::PublicKey::from_bytes(public_key_bytes.clone())?;
    let signature = crate::DigitalSignature::new(signature_bytes, public_key_bytes);
    
    let _ = transaction.set_signature(signature, public_key);
    
    if !transaction.verify_signature()? {
        return Err(ApiError::InvalidRequest("Invalid signature".to_string()));
    }
    
    // Add to blockchain
    blockchain.add_transaction_object(transaction.clone())?;
    
    // Save to storage
    state.storage.save_blockchain(&blockchain)?;
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "add_signed_transaction");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(transaction),
        message: "Signed transaction added successfully".to_string(),
    }))
}

/// Get balance for an address
async fn get_balance(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> std::result::Result<Json<ApiResponse<BalanceResponse>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_balance");
    let start = std::time::Instant::now();
    
    let blockchain = state.blockchain.lock().unwrap();
    let balance = blockchain.get_balance(&address);
    
    let response = BalanceResponse {
        address,
        balance,
    };
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "get_balance");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: "Balance retrieved successfully".to_string(),
    }))
}

/// Mine a new block
async fn mine_block(
    State(state): State<AppState>,
    Json(request): Json<MineRequest>,
) -> std::result::Result<Json<ApiResponse<MiningResponse>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "mine_block");
    let start = std::time::Instant::now();
    
    let mut blockchain = state.blockchain.lock().unwrap();
    
    if blockchain.pending_transactions.is_empty() {
        return Err(ApiError::InvalidRequest("No pending transactions to mine".to_string()));
    }
    
    let mining_start = std::time::Instant::now();
    let block = blockchain.mine_block(request.miner_address)?;
    let mining_time = mining_start.elapsed();
    
    // Save to storage
    state.storage.save_blockchain(&blockchain)?;
    
    let response = MiningResponse {
        block,
        mining_time_ms: mining_time.as_millis() as u64,
    };
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "mine_block");
    histogram!("mining_duration_ms", mining_time.as_millis() as f64);
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: "Block mined successfully".to_string(),
    }))
}

/// Get connected peers
async fn get_peers(
    State(_state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<PeersResponse>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_peers");
    let start = std::time::Instant::now();
    
    // TODO: Implement peer management
    let peers = vec![]; // Placeholder
    
    let response = PeersResponse {
        peers,
        total_peers: 0,
    };
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "get_peers");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: "Peers retrieved successfully".to_string(),
    }))
}

/// Add a new peer
async fn add_peer(
    State(_state): State<AppState>,
    Json(request): Json<PeerRequest>,
) -> std::result::Result<Json<ApiResponse<()>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "add_peer");
    let start = std::time::Instant::now();
    
    // TODO: Implement peer connection
    info!("Adding peer: {}", request.address);
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "add_peer");
    
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: "Peer added successfully".to_string(),
    }))
}

/// Create a new wallet
async fn create_wallet(
    State(state): State<AppState>,
    Json(request): Json<CreateWalletRequest>,
) -> std::result::Result<Json<ApiResponse<crate::wallet::WalletInfo>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "create_wallet");
    let start = std::time::Instant::now();
    
    let mut wallet_manager = state.wallet_manager.lock().unwrap();
    let wallet_info = wallet_manager.create_wallet(&request.password, request.name)?;
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "create_wallet");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(wallet_info),
        message: "Wallet created successfully".to_string(),
    }))
}

/// List all wallets
async fn list_wallets(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<Vec<crate::wallet::WalletInfo>>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "list_wallets");
    let start = std::time::Instant::now();
    
    let wallet_manager = state.wallet_manager.lock().unwrap();
    let wallets = wallet_manager.list_wallets()?;
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "list_wallets");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(wallets),
        message: "Wallets retrieved successfully".to_string(),
    }))
}

/// Get wallet balance
async fn get_wallet_balance(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> std::result::Result<Json<ApiResponse<BalanceResponse>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_wallet_balance");
    let start = std::time::Instant::now();
    
    let blockchain = state.blockchain.lock().unwrap();
    let wallet_manager = state.wallet_manager.lock().unwrap();
    
    let balance = wallet_manager.get_balance(&address, &blockchain);
    
    let response = BalanceResponse {
        address,
        balance,
    };
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "get_wallet_balance");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: "Wallet balance retrieved successfully".to_string(),
    }))
}

/// Send a transaction using a wallet
async fn send_transaction(
    State(state): State<AppState>,
    Json(request): Json<SendTransactionRequest>,
) -> std::result::Result<Json<ApiResponse<Transaction>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "send_transaction");
    let start = std::time::Instant::now();
    
    let mut blockchain = state.blockchain.lock().unwrap();
    let mut wallet_manager = state.wallet_manager.lock().unwrap();
    
    // Create transaction
    let mut transaction = Transaction::new_transfer(
        request.from_address.clone(),
        request.to_address,
        request.amount,
        request.message,
    )?;
    
    // Sign transaction
    let transaction_data = transaction.to_bytes()?;
    let signature = wallet_manager.sign_transaction(&request.from_address, &request.password, &transaction_data)?;
    
    // Set signature
    let public_key = wallet_manager.load_wallet(&request.from_address, &request.password)?.public_key;
    let public_key_bytes = crate::utils::hex_to_bytes(&public_key)?;
    let public_key_obj = crate::PublicKey::from_bytes(public_key_bytes)?;
    
    let _ = transaction.set_signature(signature, public_key_obj);
    
    // Add to blockchain
    blockchain.add_transaction_object(transaction.clone())?;
    
    // Save to storage
    state.storage.save_blockchain(&blockchain)?;
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "send_transaction");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(transaction),
        message: "Transaction sent successfully".to_string(),
    }))
}

/// Get API metrics
async fn get_metrics(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<MetricsResponse>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_metrics");
    let start = std::time::Instant::now();
    
    let blockchain = state.blockchain.lock().unwrap();
    let uptime = state.start_time.elapsed();
    
    let response = MetricsResponse {
        total_blocks: blockchain.blocks.len(),
        total_transactions: blockchain.blocks.iter().map(|b| b.transactions.len()).sum(),
        pending_transactions: blockchain.pending_transactions.len(),
        current_difficulty: blockchain.difficulty,
        mining_reward: blockchain.mining_reward,
        blockchain_size_bytes: blockchain.blocks.iter().map(|b| serde_json::to_string(b).unwrap_or_default().len()).sum::<usize>(),
        uptime_seconds: uptime.as_secs(),
        api_requests_total: 0, // TODO: Implement request counting
        api_errors_total: 0,   // TODO: Implement error counting
    };
    
    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "get_metrics");
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(response),
        message: "Metrics retrieved successfully".to_string(),
    }))
}

/// Health check endpoint
async fn health_check(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<()>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "health_check");
    
    // Check if blockchain is accessible
    let _blockchain = state.blockchain.lock().unwrap();
    
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: "API is healthy".to_string(),
    }))
}

// Ethereum Integration Handlers

/// Transfer tokens to Ethereum
async fn ethereum_transfer(
    State(state): State<AppState>,
    Json(request): Json<EthereumTransferRequest>,
) -> std::result::Result<Json<ApiResponse<crate::ethereum::PendingTransfer>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "ethereum_transfer");
    let start = std::time::Instant::now();

    let ethereum_bridge = state.ethereum_bridge
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Ethereum bridge not configured".to_string()))?;

    // Load wallet first
    {
        let mut wallet_manager = state.wallet_manager.lock().unwrap();
        wallet_manager.load_wallet(&request.from_address, &request.password)?;
    }
    
    // Get bridge clone for async operations
    let bridge_clone = {
        let bridge_guard = ethereum_bridge.lock().unwrap();
        bridge_guard.clone_for_background()
    };
    
    // Get wallet manager clone for async operations
    let wallet_clone = {
        let wallet_guard = state.wallet_manager.lock().unwrap();
        wallet_guard.clone_for_background()
    };
    
    // Initiate transfer
    let transfer_id = bridge_clone.transfer_to_ethereum(
        &wallet_clone,
        &request.from_address,
        &request.to_ethereum_address,
        request.amount,
        &request.password,
    ).await?;

    // Get transfer details
    let transfers = bridge_clone.get_pending_transfers().await?;
    
    let transfer = transfers.into_iter()
        .find(|t| t.id == transfer_id)
        .ok_or_else(|| ApiError::Internal("Transfer not found".to_string()))?;

    histogram!("api_request_duration_ms", start.elapsed().as_millis() as f64, "endpoint" => "ethereum_transfer");

    Ok(Json(ApiResponse {
        success: true,
        data: Some(transfer),
        message: "Ethereum transfer initiated successfully".to_string(),
    }))
}

/// Get Ethereum balance
async fn get_ethereum_balance(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> std::result::Result<Json<ApiResponse<f64>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_ethereum_balance");

    let ethereum_bridge = state.ethereum_bridge
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Ethereum bridge not configured".to_string()))?;

    let bridge_clone = {
        let bridge = ethereum_bridge.lock().unwrap();
        bridge.clone_for_background()
    };
    
    let balance = bridge_clone.get_ethereum_balance(&address).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(balance),
        message: "Ethereum balance retrieved successfully".to_string(),
    }))
}

/// Get Ethereum transfer status
async fn get_ethereum_transfer_status(
    State(state): State<AppState>,
    Path(transfer_id): Path<String>,
) -> std::result::Result<Json<ApiResponse<Option<crate::ethereum::TransferStatus>>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_ethereum_transfer_status");

    let ethereum_bridge = state.ethereum_bridge
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Ethereum bridge not configured".to_string()))?;

    let bridge_clone = {
        let bridge = ethereum_bridge.lock().unwrap();
        bridge.clone_for_background()
    };
    
    let status = bridge_clone.get_transfer_status(&transfer_id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(status),
        message: "Transfer status retrieved successfully".to_string(),
    }))
}

/// Get pending Ethereum transfers
async fn get_pending_ethereum_transfers(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<Vec<crate::ethereum::PendingTransfer>>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_pending_ethereum_transfers");

    let ethereum_bridge = state.ethereum_bridge
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Ethereum bridge not configured".to_string()))?;

    let bridge_clone = {
        let bridge = ethereum_bridge.lock().unwrap();
        bridge.clone_for_background()
    };
    
    let transfers = bridge_clone.get_pending_transfers().await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(transfers),
        message: "Pending transfers retrieved successfully".to_string(),
    }))
}

/// Get Ethereum bridge statistics
async fn get_ethereum_bridge_stats(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<crate::ethereum::BridgeStats>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_ethereum_bridge_stats");

    let ethereum_bridge = state.ethereum_bridge
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Ethereum bridge not configured".to_string()))?;

    let bridge_clone = {
        let bridge = ethereum_bridge.lock().unwrap();
        bridge.clone_for_background()
    };
    
    let stats = bridge_clone.get_bridge_stats().await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(stats),
        message: "Bridge statistics retrieved successfully".to_string(),
    }))
}

// DID Handlers

/// Create a new DID
async fn create_did(
    State(state): State<AppState>,
    Json(request): Json<DIDCreationRequest>,
) -> std::result::Result<Json<ApiResponse<(String, String)>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "create_did");

    let did_system = state.did_system
        .as_ref()
        .ok_or_else(|| ApiError::Internal("DID system not configured".to_string()))?;

    let system_clone = {
        let system = did_system.lock().unwrap();
        system.clone_for_background()
    };
    let did_request = crate::did::DIDCreationRequest {
        controller: request.controller,
        service_endpoints: request.service_endpoints,
    };
    let (did, _keypair) = system_clone.create_did(did_request).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some((did, "Keypair generated successfully".to_string())),
        message: "DID created successfully".to_string(),
    }))
}

/// Get DID document
async fn get_did_document(
    State(state): State<AppState>,
    Path(did): Path<String>,
) -> std::result::Result<Json<ApiResponse<Option<crate::did::DIDDocument>>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_did_document");

    let did_system = state.did_system
        .as_ref()
        .ok_or_else(|| ApiError::Internal("DID system not configured".to_string()))?;

    let system_clone = {
        let system = did_system.lock().unwrap();
        system.clone_for_background()
    };
    
    let document = system_clone.get_did_document(&did).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(document),
        message: "DID document retrieved successfully".to_string(),
    }))
}

/// Link DID to wallet
async fn link_did_to_wallet(
    State(state): State<AppState>,
    Path(did): Path<String>,
    Json(request): Json<DIDLinkRequest>,
) -> std::result::Result<Json<ApiResponse<()>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "link_did_to_wallet");

    let did_system = state.did_system
        .as_ref()
        .ok_or_else(|| ApiError::Internal("DID system not configured".to_string()))?;

    let system_clone = {
        let system = did_system.lock().unwrap();
        system.clone_for_background()
    };
    
    system_clone.link_did_to_wallet(&did, &request.wallet_address).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: "DID linked to wallet successfully".to_string(),
    }))
}

/// Get DID for wallet
async fn get_did_for_wallet(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> std::result::Result<Json<ApiResponse<Option<String>>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_did_for_wallet");

    let did_system = state.did_system
        .as_ref()
        .ok_or_else(|| ApiError::Internal("DID system not configured".to_string()))?;

    let system_clone = {
        let system = did_system.lock().unwrap();
        system.clone_for_background()
    };
    
    let did = system_clone.get_did_for_wallet(&address).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(did),
        message: "DID for wallet retrieved successfully".to_string(),
    }))
}

/// Verify DID signature
async fn verify_did_signature(
    State(state): State<AppState>,
    Path(did): Path<String>,
    Json(request): Json<DIDVerificationRequest>,
) -> std::result::Result<Json<ApiResponse<crate::did::DIDVerificationResult>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "verify_did_signature");

    let did_system = state.did_system
        .as_ref()
        .ok_or_else(|| ApiError::Internal("DID system not configured".to_string()))?;

    let system_clone = {
        let system = did_system.lock().unwrap();
        system.clone_for_background()
    };
    
    let result = system_clone.verify_did_signature(
        &did,
        request.message.as_bytes(),
        request.signature.as_bytes(),
    ).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: "DID signature verification completed".to_string(),
    }))
}

/// Get all DIDs
async fn get_all_dids(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_all_dids");

    let did_system = state.did_system
        .as_ref()
        .ok_or_else(|| ApiError::Internal("DID system not configured".to_string()))?;

    let system_clone = {
        let system = did_system.lock().unwrap();
        system.clone_for_background()
    };
    
    let dids = system_clone.get_all_dids().await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(dids),
        message: "All DIDs retrieved successfully".to_string(),
    }))
}

/// Get DID statistics
async fn get_did_stats(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<crate::did::DIDStats>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_did_stats");

    let did_system = state.did_system
        .as_ref()
        .ok_or_else(|| ApiError::Internal("DID system not configured".to_string()))?;

    let system_clone = {
        let system = did_system.lock().unwrap();
        system.clone_for_background()
    };
    
    let stats = system_clone.get_did_stats().await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(stats),
        message: "DID statistics retrieved successfully".to_string(),
    }))
}

// Governance Handlers

/// Create governance proposal
async fn create_governance_proposal(
    State(state): State<AppState>,
    Json(request): Json<GovernanceProposalRequest>,
) -> std::result::Result<Json<ApiResponse<String>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "create_governance_proposal");

    let governance = state.governance
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Governance system not configured".to_string()))?;

    let proposal_request = crate::governance::ProposalCreationRequest {
        title: request.title,
        description: request.description,
        proposal_type: request.proposal_type,
        contract_code: request.contract_code,
        parameters: request.parameters,
        voting_period: request.voting_period,
        quorum: request.quorum,
    };

    let gov_clone = {
        let gov = governance.lock().unwrap();
        gov.clone_for_background()
    };
    
    let proposal_id = gov_clone.create_proposal(&request.proposer, proposal_request).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(proposal_id),
        message: "Governance proposal created successfully".to_string(),
    }))
}

/// Vote on governance proposal
async fn vote_on_proposal(
    State(state): State<AppState>,
    Json(request): Json<GovernanceVoteRequest>,
) -> std::result::Result<Json<ApiResponse<()>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "vote_on_proposal");

    let governance = state.governance
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Governance system not configured".to_string()))?;

    let vote_request = crate::governance::VoteRequest {
        proposal_id: request.proposal_id,
        vote: request.vote,
        stake_amount: request.stake_amount,
    };

    let gov_clone = {
        let gov = governance.lock().unwrap();
        gov.clone_for_background()
    };
    
    gov_clone.vote_on_proposal(&request.voter, vote_request).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: "Vote cast successfully".to_string(),
    }))
}

/// Execute governance proposal
async fn execute_proposal(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
) -> std::result::Result<Json<ApiResponse<()>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "execute_proposal");

    let governance = state.governance
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Governance system not configured".to_string()))?;

    let gov_clone = {
        let gov = governance.lock().unwrap();
        gov.clone_for_background()
    };
    
    gov_clone.execute_proposal(&proposal_id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: "Proposal executed successfully".to_string(),
    }))
}

/// Get governance proposal
async fn get_governance_proposal(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
) -> std::result::Result<Json<ApiResponse<Option<crate::governance::GovernanceProposal>>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_governance_proposal");

    let governance = state.governance
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Governance system not configured".to_string()))?;

    let gov_clone = {
        let gov = governance.lock().unwrap();
        gov.clone_for_background()
    };
    
    let proposal = gov_clone.get_proposal(&proposal_id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(proposal),
        message: "Governance proposal retrieved successfully".to_string(),
    }))
}

/// Get all governance proposals
async fn get_all_governance_proposals(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<Vec<crate::governance::GovernanceProposal>>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_all_governance_proposals");

    let governance = state.governance
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Governance system not configured".to_string()))?;

    let gov_clone = {
        let gov = governance.lock().unwrap();
        gov.clone_for_background()
    };
    
    let proposals = gov_clone.get_all_proposals().await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(proposals),
        message: "All governance proposals retrieved successfully".to_string(),
    }))
}

/// Get proposal votes
async fn get_proposal_votes(
    State(state): State<AppState>,
    Path(proposal_id): Path<String>,
) -> std::result::Result<Json<ApiResponse<Vec<crate::governance::Vote>>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_proposal_votes");

    let governance = state.governance
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Governance system not configured".to_string()))?;

    let gov_clone = {
        let gov = governance.lock().unwrap();
        gov.clone_for_background()
    };
    
    let votes = gov_clone.get_proposal_votes(&proposal_id).await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(votes),
        message: "Proposal votes retrieved successfully".to_string(),
    }))
}

/// Get governance statistics
async fn get_governance_stats(
    State(state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<crate::governance::GovernanceStats>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_governance_stats");

    let governance = state.governance
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Governance system not configured".to_string()))?;

    let gov_clone = {
        let gov = governance.lock().unwrap();
        gov.clone_for_background()
    };
    
    let stats = gov_clone.get_governance_stats().await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(stats),
        message: "Governance statistics retrieved successfully".to_string(),
    }))
}

// Simulation Handlers

/// Run simulation
async fn run_simulation(
    State(state): State<AppState>,
    Json(_request): Json<SimulationRunRequest>,
) -> std::result::Result<Json<ApiResponse<crate::simulation::SimulationResult>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "run_simulation");

    let simulation_manager = state.simulation_manager
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Simulation manager not configured".to_string()))?;

    let sim_clone = {
        let sim = simulation_manager.lock().unwrap();
        sim.clone_for_background()
    };
    
    let result = sim_clone.run_simulation().await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(result),
        message: "Simulation completed successfully".to_string(),
    }))
}

/// Get simulation progress
async fn get_simulation_progress(
    State(state): State<AppState>,
    Path(_simulation_id): Path<String>,
) -> std::result::Result<Json<ApiResponse<f64>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_simulation_progress");

    let simulation_manager = state.simulation_manager
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Simulation manager not configured".to_string()))?;

    let sim_clone = {
        let sim = simulation_manager.lock().unwrap();
        sim.clone_for_background()
    };
    
    let progress = sim_clone.get_progress().await;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(progress),
        message: "Simulation progress retrieved successfully".to_string(),
    }))
}

/// Get simulation state
async fn get_simulation_state(
    State(state): State<AppState>,
    Path(_simulation_id): Path<String>,
) -> std::result::Result<Json<ApiResponse<crate::simulation::SimulationState>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_simulation_state");

    let simulation_manager = state.simulation_manager
        .as_ref()
        .ok_or_else(|| ApiError::Internal("Simulation manager not configured".to_string()))?;

    let sim_clone = {
        let sim = simulation_manager.lock().unwrap();
        sim.clone_for_background()
    };
    
    let state = sim_clone.get_current_state().await?;

    Ok(Json(ApiResponse {
        success: true,
        data: Some(state),
        message: "Simulation state retrieved successfully".to_string(),
    }))
}

/// Stop simulation
async fn stop_simulation(
    State(_state): State<AppState>,
    Path(_simulation_id): Path<String>,
) -> std::result::Result<Json<ApiResponse<()>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "stop_simulation");

    // Note: This would need to be implemented in the simulation manager
    // For now, we'll return a success response
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        message: "Simulation stop requested".to_string(),
    }))
}

/// Get simulation results
async fn get_simulation_results(
    State(_state): State<AppState>,
    Path(_simulation_id): Path<String>,
) -> std::result::Result<Json<ApiResponse<crate::simulation::SimulationResult>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_simulation_results");

    // Note: This would need to be implemented to store and retrieve simulation results
    // For now, we'll return an error
    Err(ApiError::Internal("Simulation results not yet implemented".to_string()))
}

/// Get all simulations
async fn get_all_simulations(
    State(_state): State<AppState>,
) -> std::result::Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    counter!("api_requests_total", 1, "endpoint" => "get_all_simulations");

    // Note: This would need to be implemented to track all simulations
    // For now, we'll return an empty list
    Ok(Json(ApiResponse {
        success: true,
        data: Some(vec![]),
        message: "All simulations retrieved successfully".to_string(),
    }))
}

/// Start the API server
/// 
/// # Arguments
/// * `state` - Application state
/// * `address` - Server address to bind to
/// 
/// # Returns
/// * `Result<()>` - Ok if server started successfully
pub async fn start_server(state: AppState, address: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let app = create_router(state);
    
    info!("Starting API server on {}", address);
    
    let listener = tokio::net::TcpListener::bind(address).await?;
    
    // Set up graceful shutdown
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    // Handle shutdown signals
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        info!("Received shutdown signal, closing server...");
        let _ = tx.send(());
    });
    
    // Start the server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            rx.await.ok();
        })
        .await?;
    
    info!("Server shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_health_check() {
        let temp_dir = tempdir().unwrap();
        let storage = Arc::new(crate::storage::BlockchainStorage::new(temp_dir.path()).unwrap());
        let blockchain = Blockchain::new_pow(2, 50.0).unwrap();
        let wallet_manager = WalletManager::new();
        
        let state = AppState {
            blockchain: Arc::new(Mutex::new(blockchain)),
            wallet_manager: Arc::new(Mutex::new(wallet_manager)),
            ethereum_bridge: None,
            did_system: None,
            governance: None,
            simulation_manager: None,
            storage: storage,
            storage_path: "./test_api_db".to_string(),
            start_time: std::time::Instant::now(),
        };
        
        let response = health_check(State(state)).await.unwrap();
        let response_body = response.0;
        
        assert!(response_body.success);
        assert_eq!(response_body.message, "API is healthy");
    }
}
