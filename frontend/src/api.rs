use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "http://localhost:3000";

pub struct BlockchainApi;

impl BlockchainApi {
    /// Get blockchain status
    pub async fn get_status() -> Result<BlockchainStatus, Box<dyn std::error::Error>> {
        let response = Request::get(&format!("{}/api/blockchain/status", API_BASE_URL))
            .send()
            .await?;
        
        if response.ok() {
            let data: BlockchainStatus = response.json().await?;
            Ok(data)
        } else {
            Err(format!("HTTP error: {}", response.status()).into())
        }
    }

    /// Create a new transaction
    pub async fn create_transaction(request: TransactionRequest) -> Result<(), Box<dyn std::error::Error>> {
        let response = Request::post(&format!("{}/api/transactions", API_BASE_URL))
            .json(&request)?
            .send()
            .await?;
        
        if response.ok() {
            Ok(())
        } else {
            Err(format!("HTTP error: {}", response.status()).into())
        }
    }

    /// Deploy a smart contract
    pub async fn deploy_contract(request: ContractDeployRequest) -> Result<String, Box<dyn std::error::Error>> {
        let response = Request::post(&format!("{}/api/contracts/deploy", API_BASE_URL))
            .json(&request)?
            .send()
            .await?;
        
        if response.ok() {
            let result: ContractDeployResponse = response.json().await?;
            Ok(result.contract_address)
        } else {
            Err(format!("HTTP error: {}", response.status()).into())
        }
    }

    /// Get wallet information
    pub async fn get_wallets() -> Result<Vec<WalletInfo>, Box<dyn std::error::Error>> {
        let response = Request::get(&format!("{}/api/wallets", API_BASE_URL))
            .send()
            .await?;
        
        if response.ok() {
            let data: Vec<WalletInfo> = response.json().await?;
            Ok(data)
        } else {
            Err(format!("HTTP error: {}", response.status()).into())
        }
    }

    /// Create a new wallet
    pub async fn create_wallet(request: CreateWalletRequest) -> Result<WalletInfo, Box<dyn std::error::Error>> {
        let response = Request::post(&format!("{}/api/wallets", API_BASE_URL))
            .json(&request)?
            .send()
            .await?;
        
        if response.ok() {
            let data: WalletInfo = response.json().await?;
            Ok(data)
        } else {
            Err(format!("HTTP error: {}", response.status()).into())
        }
    }

    /// Get blockchain metrics
    pub async fn get_metrics() -> Result<BlockchainMetrics, Box<dyn std::error::Error>> {
        let response = Request::get(&format!("{}/api/metrics", API_BASE_URL))
            .send()
            .await?;
        
        if response.ok() {
            let data: BlockchainMetrics = response.json().await?;
            Ok(data)
        } else {
            Err(format!("HTTP error: {}", response.status()).into())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStatus {
    pub blocks: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub consensus_type: String,
    pub difficulty: u32,
    pub mining_reward: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub hash: String,
    pub previous_hash: String,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub consensus_type: String,
    pub validator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub transaction_type: String,
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeployRequest {
    pub sender: String,
    pub contract_code: String,
    pub gas_limit: u64,
    pub gas_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeployResponse {
    pub contract_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub balance: f64,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainMetrics {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub pending_transactions: u64,
    pub total_contracts: u64,
    pub contract_deployments: u64,
    pub contract_calls: u64,
    pub total_gas_used: u64,
    pub consensus_type: String,
    pub validators: Option<u64>,
    pub total_stake: Option<f64>,
    pub average_performance: Option<f64>,
}
