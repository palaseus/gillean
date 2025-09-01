use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainClient {
    base_url: String,
    client: Client,
}

impl BlockchainClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    pub async fn get_block(&self, block_hash: &str) -> Result<Block, Box<dyn std::error::Error>> {
        let url = format!("{}/blocks/{}", self.base_url, block_hash);
        let response = self.client.get(&url).send().await?;
        let block: Block = response.json().await?;
        Ok(block)
    }

    pub async fn send_transaction(&self, transaction: Transaction) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/transactions", self.base_url);
        let response = self.client.post(&url).json(&transaction).send().await?;
        let result: HashMap<String, String> = response.json().await?;
        Ok(result.get("tx_hash").unwrap().clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub data: Option<String>,
}
