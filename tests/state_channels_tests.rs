// Multi-Party State Channels Test Suite
// Tests for state channels with support for more than two participants

use gillean::{Result, Blockchain, BlockchainError};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StateChannel {
    pub id: String,
    pub participants: Vec<String>,
    pub balances: HashMap<String, f64>,
    pub state_version: u64,
    pub is_open: bool,
    pub dispute_period: u64,
}

#[derive(Debug, Clone)]
pub struct StateChannelManager {
    pub channels: HashMap<String, StateChannel>,
    pub blockchain: Arc<Mutex<Blockchain>>,
}

impl StateChannelManager {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            channels: HashMap::new(),
            blockchain,
        }
    }

    pub fn create_channel(&mut self, participants: Vec<String>, initial_balances: HashMap<String, f64>) -> Result<String> {
        if participants.len() < 2 {
            return Err(BlockchainError::InvalidInput("State channel requires at least 2 participants".to_string()));
        }

        let channel_id = format!("channel_{}", uuid::Uuid::new_v4());
        
        let channel = StateChannel {
            id: channel_id.clone(),
            participants: participants.clone(),
            balances: initial_balances,
            state_version: 0,
            is_open: true,
            dispute_period: 1000, // 1000 blocks dispute period
        };

        self.channels.insert(channel_id.clone(), channel);
        Ok(channel_id)
    }

    pub fn update_state(&mut self, channel_id: &str, new_balances: HashMap<String, f64>, _signature: String) -> Result<()> {
        let channel = self.channels.get_mut(channel_id)
            .ok_or_else(|| BlockchainError::InvalidInput("Channel not found".to_string()))?;

        if !channel.is_open {
            return Err(BlockchainError::InvalidInput("Channel is closed".to_string()));
        }

        // Verify all participants are accounted for
        for participant in &channel.participants {
            if !new_balances.contains_key(participant) {
                return Err(BlockchainError::InvalidInput(format!("Missing balance for participant: {}", participant)));
            }
        }

        // Verify total balance is preserved
        let old_total: f64 = channel.balances.values().sum();
        let new_total: f64 = new_balances.values().sum();
        
        if (old_total - new_total).abs() > 0.001 {
            return Err(BlockchainError::InvalidInput("Total balance must be preserved".to_string()));
        }

        // TODO: Verify signature from all participants
        // For now, we'll just update the state
        channel.balances = new_balances;
        channel.state_version += 1;

        Ok(())
    }

    pub fn close_channel(&mut self, channel_id: &str, final_balances: HashMap<String, f64>) -> Result<()> {
        let channel = self.channels.get_mut(channel_id)
            .ok_or_else(|| BlockchainError::InvalidInput("Channel not found".to_string()))?;

        if !channel.is_open {
            return Err(BlockchainError::InvalidInput("Channel is already closed".to_string()));
        }

        // Verify final balances
        for participant in &channel.participants {
            if !final_balances.contains_key(participant) {
                return Err(BlockchainError::InvalidInput(format!("Missing final balance for participant: {}", participant)));
            }
        }

        // Update final state and close
        channel.balances = final_balances;
        channel.is_open = false;

        Ok(())
    }

    pub fn initiate_dispute(&mut self, channel_id: &str) -> Result<()> {
        let channel = self.channels.get_mut(channel_id)
            .ok_or_else(|| BlockchainError::InvalidInput("Channel not found".to_string()))?;

        if !channel.is_open {
            return Err(BlockchainError::InvalidInput("Cannot dispute closed channel".to_string()));
        }

        // TODO: Implement dispute logic
        // For now, we'll just mark the channel as disputed
        Ok(())
    }
}

pub struct StateChannelsSuite {
    _manager: StateChannelManager,
}

impl StateChannelsSuite {
    pub fn new(blockchain: Arc<Mutex<Blockchain>>) -> Self {
        Self {
            _manager: StateChannelManager::new(blockchain),
        }
    }

    pub async fn test_multi_party_channel_creation(&self) -> Result<()> {
        println!("🧪 Testing multi-party channel creation...");

        let mut manager = StateChannelManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));
        
        // Create a 3-party channel
        let participants = vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()];
        let initial_balances = HashMap::from([
            ("alice".to_string(), 100.0),
            ("bob".to_string(), 150.0),
            ("charlie".to_string(), 200.0),
        ]);

        let channel_id = manager.create_channel(participants.clone(), initial_balances.clone())?;
        
        // Verify channel was created
        assert!(manager.channels.contains_key(&channel_id));
        let channel = &manager.channels[&channel_id];
        assert_eq!(channel.participants, participants);
        assert_eq!(channel.balances, initial_balances);
        assert!(channel.is_open);
        assert_eq!(channel.state_version, 0);

        println!("✅ Multi-party channel creation test passed!");
        Ok(())
    }

    pub async fn test_state_updates(&self) -> Result<()> {
        println!("🧪 Testing state updates...");

        let mut manager = StateChannelManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));
        
        // Create a 4-party channel
        let participants = vec!["alice".to_string(), "bob".to_string(), "charlie".to_string(), "dave".to_string()];
        let initial_balances = HashMap::from([
            ("alice".to_string(), 100.0),
            ("bob".to_string(), 100.0),
            ("charlie".to_string(), 100.0),
            ("dave".to_string(), 100.0),
        ]);

        let channel_id = manager.create_channel(participants.clone(), initial_balances)?;
        
        // Update state (simulate a transaction)
        let new_balances = HashMap::from([
            ("alice".to_string(), 90.0),
            ("bob".to_string(), 110.0),
            ("charlie".to_string(), 95.0),
            ("dave".to_string(), 105.0),
        ]);

        manager.update_state(&channel_id, new_balances.clone(), "signature".to_string())?;
        
        // Verify state was updated
        let channel = &manager.channels[&channel_id];
        assert_eq!(channel.balances, new_balances);
        assert_eq!(channel.state_version, 1);
        assert!(channel.is_open);

        println!("✅ State updates test passed!");
        Ok(())
    }

    pub async fn test_channel_closure(&self) -> Result<()> {
        println!("🧪 Testing channel closure...");

        let mut manager = StateChannelManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));
        
        // Create a 3-party channel
        let participants = vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()];
        let initial_balances = HashMap::from([
            ("alice".to_string(), 100.0),
            ("bob".to_string(), 100.0),
            ("charlie".to_string(), 100.0),
        ]);

        let channel_id = manager.create_channel(participants.clone(), initial_balances)?;
        
        // Close the channel
        let final_balances = HashMap::from([
            ("alice".to_string(), 80.0),
            ("bob".to_string(), 120.0),
            ("charlie".to_string(), 100.0),
        ]);

        manager.close_channel(&channel_id, final_balances.clone())?;
        
        // Verify channel is closed
        let channel = &manager.channels[&channel_id];
        assert!(!channel.is_open);
        assert_eq!(channel.balances, final_balances);

        println!("✅ Channel closure test passed!");
        Ok(())
    }

    pub async fn test_invalid_operations(&self) -> Result<()> {
        println!("🧪 Testing invalid operations...");

        let mut manager = StateChannelManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));
        
        // Test creating channel with insufficient participants
        let single_participant = vec!["alice".to_string()];
        let result = manager.create_channel(single_participant, HashMap::new());
        assert!(result.is_err());

        // Test updating non-existent channel
        let result = manager.update_state("non_existent", HashMap::new(), "signature".to_string());
        assert!(result.is_err());

        // Test closing non-existent channel
        let result = manager.close_channel("non_existent", HashMap::new());
        assert!(result.is_err());

        println!("✅ Invalid operations test passed!");
        Ok(())
    }

    pub async fn test_balance_preservation(&self) -> Result<()> {
        println!("🧪 Testing balance preservation...");

        let mut manager = StateChannelManager::new(Arc::new(Mutex::new(Blockchain::new_pow(2, 50.0)?)));
        
        // Create a 5-party channel
        let participants = vec!["alice".to_string(), "bob".to_string(), "charlie".to_string(), "dave".to_string(), "eve".to_string()];
        let initial_balances = HashMap::from([
            ("alice".to_string(), 100.0),
            ("bob".to_string(), 100.0),
            ("charlie".to_string(), 100.0),
            ("dave".to_string(), 100.0),
            ("eve".to_string(), 100.0),
        ]);

        let channel_id = manager.create_channel(participants.clone(), initial_balances)?;
        
        // Try to update with invalid total balance
        let invalid_balances = HashMap::from([
            ("alice".to_string(), 200.0), // This would increase total
            ("bob".to_string(), 100.0),
            ("charlie".to_string(), 100.0),
            ("dave".to_string(), 100.0),
            ("eve".to_string(), 100.0),
        ]);

        let result = manager.update_state(&channel_id, invalid_balances, "signature".to_string());
        assert!(result.is_err());

        // Try with valid balance preservation
        let valid_balances = HashMap::from([
            ("alice".to_string(), 90.0),
            ("bob".to_string(), 110.0),
            ("charlie".to_string(), 95.0),
            ("dave".to_string(), 105.0),
            ("eve".to_string(), 100.0),
        ]);

        let result = manager.update_state(&channel_id, valid_balances, "signature".to_string());
        assert!(result.is_ok());

        println!("✅ Balance preservation test passed!");
        Ok(())
    }

    pub async fn run_all_tests(&self) -> Result<()> {
        println!("🚀 Running Multi-Party State Channels test suite...");
        
        self.test_multi_party_channel_creation().await?;
        self.test_state_updates().await?;
        self.test_channel_closure().await?;
        self.test_invalid_operations().await?;
        self.test_balance_preservation().await?;

        println!("✅ All Multi-Party State Channels tests passed!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_channel_manager_creation() {
        let blockchain = Blockchain::new_pow(2, 50.0).unwrap();
        let _manager = StateChannelManager::new(Arc::new(Mutex::new(blockchain)));
        assert!(true); // Basic test to ensure manager can be created
    }

    #[test]
    fn test_channel_creation() {
        let blockchain = Blockchain::new_pow(2, 50.0).unwrap();
        let mut manager = StateChannelManager::new(Arc::new(Mutex::new(blockchain)));
        
        let participants = vec!["alice".to_string(), "bob".to_string()];
        let initial_balances = HashMap::from([
            ("alice".to_string(), 100.0),
            ("bob".to_string(), 100.0),
        ]);
        
        let channel_id = manager.create_channel(participants, initial_balances).unwrap();
        
        assert!(!channel_id.is_empty());
        assert!(manager.channels.contains_key(&channel_id));
    }

    #[test]
    fn test_state_update() {
        let blockchain = Blockchain::new_pow(2, 50.0).unwrap();
        let mut manager = StateChannelManager::new(Arc::new(Mutex::new(blockchain)));
        
        let participants = vec!["alice".to_string(), "bob".to_string()];
        let initial_balances = HashMap::from([
            ("alice".to_string(), 100.0),
            ("bob".to_string(), 100.0),
        ]);
        
        let channel_id = manager.create_channel(participants, initial_balances).unwrap();
        
        let new_balances = HashMap::from([
            ("alice".to_string(), 90.0),
            ("bob".to_string(), 110.0),
        ]);
        
        let result = manager.update_state(&channel_id, new_balances, "signature".to_string());
        assert!(result.is_ok());
    }
}
