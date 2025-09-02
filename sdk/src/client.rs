use super::{SDKResult, SDKError, SDKConfig, BlockchainStatus, TransactionInfo, BlockInfo, ShardInfo, BridgeStatus, ContractInfo, MetricsData};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;

/// HTTP client for interacting with Gillean blockchain API
pub struct GilleanClient {
    client: Client,
    config: SDKConfig,
}

impl GilleanClient {
    /// Create a new client instance
    pub async fn new(config: SDKConfig) -> SDKResult<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| SDKError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, config })
    }

    /// Get blockchain status
    pub async fn get_blockchain_status(&self) -> SDKResult<BlockchainStatus> {
        let url = format!("{}/api/status", self.config.api_url);
        self.make_request::<BlockchainStatus>(&url).await
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: &str) -> SDKResult<f64> {
        let url = format!("{}/api/balance/{}", self.config.api_url, address);
        let response: BalanceResponse = self.make_request(&url).await?;
        Ok(response.balance)
    }

    /// Get block by index
    pub async fn get_block(&self, index: usize) -> SDKResult<BlockInfo> {
        let url = format!("{}/api/block/{}", self.config.api_url, index);
        self.make_request(&url).await
    }

    /// Get transaction by hash
    pub async fn get_transaction(&self, hash: &str) -> SDKResult<TransactionInfo> {
        let url = format!("{}/api/transaction/{}", self.config.api_url, hash);
        self.make_request(&url).await
    }

    /// Get pending transactions
    pub async fn get_pending_transactions(&self) -> SDKResult<Vec<TransactionInfo>> {
        let url = format!("{}/api/pending", self.config.api_url);
        self.make_request(&url).await
    }

    /// Get shard information
    pub async fn get_shard_info(&self, shard_id: usize) -> SDKResult<ShardInfo> {
        let url = format!("{}/api/shard/{}", self.config.api_url, shard_id);
        self.make_request(&url).await
    }

    /// Get bridge status
    pub async fn get_bridge_status(&self) -> SDKResult<BridgeStatus> {
        let url = format!("{}/api/bridge/status", self.config.api_url);
        self.make_request(&url).await
    }

    /// Get contract information
    pub async fn get_contract_info(&self, address: &str) -> SDKResult<ContractInfo> {
        let url = format!("{}/api/contract/{}", self.config.api_url, address);
        self.make_request(&url).await
    }

    /// Get metrics data
    pub async fn get_metrics(&self) -> SDKResult<MetricsData> {
        let url = format!("{}/api/metrics", self.config.api_url);
        self.make_request(&url).await
    }

    /// Subscribe to real-time updates
    pub async fn subscribe_to_updates(&self, _event_types: Vec<super::EventType>) -> SDKResult<mpsc::Receiver<super::Event>> {
        // In a real implementation, this would establish a WebSocket connection
        // For now, we'll return a dummy receiver
        let (tx, rx) = mpsc::channel(100);
        
        // Spawn a task to simulate real-time updates
        let _tx = tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                let _ = _tx.send(super::Event {
                    event_type: super::EventType::NewBlock,
                    data: serde_json::json!({"block_number": 123}),
                    timestamp: chrono::Utc::now().timestamp(),
                }).await;
            }
        });

        Ok(rx)
    }

    /// Make HTTP request with retry logic
    async fn make_request<T>(&self, url: &str) -> SDKResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut attempts = 0;
        let max_attempts = self.config.retry_attempts;

        loop {
            attempts += 1;

            let response = self.client
                .get(url)
                .header("User-Agent", "Gillean-SDK/2.0.0")
                .send()
                .await;

            match response {
                Ok(response) => {
                    if response.status().is_success() {
                        let data = response.json::<T>().await
                            .map_err(SDKError::RequestError)?;
                        return Ok(data);
                    } else if response.status().is_client_error() {
                        return Err(SDKError::InvalidInput(format!("Client error: {}", response.status())));
                    } else if response.status().is_server_error() {
                        if attempts < max_attempts {
                            tokio::time::sleep(Duration::from_secs(attempts as u64)).await;
                            continue;
                        }
                        return Err(SDKError::NetworkError(format!("Server error: {}", response.status())));
                    }
                }
                Err(e) => {
                    if attempts < max_attempts {
                        tokio::time::sleep(Duration::from_secs(attempts as u64)).await;
                        continue;
                    }
                    return Err(SDKError::RequestError(e));
                }
            }
        }
    }
}

/// Balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BalanceResponse {
    balance: f64,
}



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = crate::SDKConfig::default();
        let client = GilleanClient::new(config).await;
        assert!(client.is_ok());
    }

    #[test]
    fn test_balance_response_deserialization() {
        let json = r#"{"balance": 100.5}"#;
        let response: BalanceResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.balance, 100.5);
    }
}
