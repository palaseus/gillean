use crate::{Result, BlockchainError};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use log::info;

/// State channel for off-chain transaction processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChannel {
    /// Channel ID
    pub id: String,
    /// Participant addresses
    pub participants: Vec<String>,
    /// Channel state
    pub state: ChannelState,
    /// Channel balance
    pub balance: HashMap<String, f64>,
    /// Channel nonce
    pub nonce: u64,
    /// Channel status
    pub status: ChannelStatus,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
}

/// Channel state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    /// Current state hash
    pub state_hash: Vec<u8>,
    /// State version
    pub version: u64,
    /// State data
    pub data: Vec<u8>,
}

/// Channel status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChannelStatus {
    Open,
    Closing,
    Closed,
    Disputed,
}

/// State channel update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUpdate {
    /// Channel ID
    pub channel_id: String,
    /// Update nonce
    pub nonce: u64,
    /// New state
    pub new_state: ChannelState,
    /// Participant signatures
    pub signatures: HashMap<String, Vec<u8>>,
    /// Update timestamp
    pub timestamp: i64,
}

/// State channel manager
pub struct StateChannelManager {
    /// Active channels
    channels: Arc<Mutex<HashMap<String, StateChannel>>>,
    /// Channel updates
    updates: Arc<Mutex<HashMap<String, Vec<ChannelUpdate>>>>,
    /// Message sender for network communication
    message_sender: mpsc::Sender<ChannelMessage>,
}

/// Channel message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelMessage {
    OpenChannel(OpenChannelRequest),
    UpdateChannel(ChannelUpdate),
    CloseChannel(CloseChannelRequest),
    DisputeChannel(DisputeChannelRequest),
}

/// Open channel request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenChannelRequest {
    pub participants: Vec<String>,
    pub initial_balance: HashMap<String, f64>,
    pub timeout: u64,
}

/// Close channel request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseChannelRequest {
    pub channel_id: String,
    pub final_state: ChannelState,
    pub signatures: HashMap<String, Vec<u8>>,
}

/// Dispute channel request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeChannelRequest {
    pub channel_id: String,
    pub disputed_state: ChannelState,
    pub evidence: Vec<u8>,
}

impl StateChannelManager {
    /// Create a new state channel manager
    pub fn new() -> (Self, mpsc::Receiver<ChannelMessage>) {
        let (message_sender, message_receiver) = mpsc::channel(100);
        
        let manager = Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
            updates: Arc::new(Mutex::new(HashMap::new())),
            message_sender,
        };

        (manager, message_receiver)
    }

    /// Open a new state channel
    pub async fn open_channel(
        &self,
        participants: Vec<String>,
        initial_balance: HashMap<String, f64>,
        timeout: u64,
    ) -> Result<String> {
        info!("Opening state channel for participants: {:?}", participants);

        // Validate participants
        if participants.len() != 2 {
            return Err(BlockchainError::InvalidInput("State channels require exactly 2 participants".to_string()));
        }

        // Generate channel ID
        let channel_id = self.generate_channel_id(&participants);

        // Create initial state
        let initial_state = ChannelState {
            state_hash: self.compute_state_hash(&initial_balance),
            version: 0,
            data: serde_json::to_vec(&initial_balance)?,
        };

        let channel = StateChannel {
            id: channel_id.clone(),
            participants: participants.clone(),
            state: initial_state,
            balance: initial_balance.clone(),
            nonce: 0,
            status: ChannelStatus::Open,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        // Store channel
        {
            let mut channels = self.channels.lock().unwrap();
            channels.insert(channel_id.clone(), channel);
        }

        // Send open message (ignore errors for demo)
        let open_request = OpenChannelRequest {
            participants,
            initial_balance,
            timeout,
        };

        let _ = self.message_sender.send(ChannelMessage::OpenChannel(open_request)).await;

        info!("State channel opened: {}", channel_id);
        Ok(channel_id)
    }

    /// Update channel state
    pub async fn update_channel(
        &self,
        channel_id: &str,
        new_balance: HashMap<String, f64>,
        signatures: HashMap<String, Vec<u8>>,
    ) -> Result<()> {
        info!("Updating state channel: {}", channel_id);

        // Get channel and verify it exists and is open
        let (current_state_version, current_nonce) = {
            let mut channels = self.channels.lock().unwrap();
            let channel = channels.get_mut(channel_id)
                .ok_or_else(|| BlockchainError::NotFound("Channel not found".to_string()))?;

            // Verify channel is open
            if channel.status != ChannelStatus::Open {
                return Err(BlockchainError::InvalidState("Channel is not open".to_string()));
            }

            // Verify signatures
            self.verify_update_signatures(channel, &new_balance, &signatures)?;

            (channel.state.version, channel.nonce)
        };

        // Create new state
        let new_state = ChannelState {
            state_hash: self.compute_state_hash(&new_balance),
            version: current_state_version + 1,
            data: serde_json::to_vec(&new_balance)?,
        };

        // Update channel (drop mutex guard before await)
        {
            let mut channels = self.channels.lock().unwrap();
            let channel = channels.get_mut(channel_id).unwrap();
            channel.state = new_state.clone();
            channel.balance = new_balance;
            channel.nonce = current_nonce + 1;
            channel.updated_at = chrono::Utc::now().timestamp();
        }

        // Create update
        let update = ChannelUpdate {
            channel_id: channel_id.to_string(),
            nonce: current_nonce + 1,
            new_state,
            signatures,
            timestamp: chrono::Utc::now().timestamp(),
        };

        // Store update
        {
            let mut updates = self.updates.lock().unwrap();
            updates.entry(channel_id.to_string())
                .or_default()
                .push(update.clone());
        }

        // Send update message (ignore errors for demo)
        let _ = self.message_sender.send(ChannelMessage::UpdateChannel(update)).await;

        info!("State channel updated: {}", channel_id);
        Ok(())
    }

    /// Close a state channel
    pub async fn close_channel(
        &self,
        channel_id: &str,
        final_balance: HashMap<String, f64>,
        signatures: HashMap<String, Vec<u8>>,
    ) -> Result<()> {
        info!("Closing state channel: {}", channel_id);

        // Get channel and verify it exists and is open
        let current_state_version = {
            let mut channels = self.channels.lock().unwrap();
            let channel = channels.get_mut(channel_id)
                .ok_or_else(|| BlockchainError::NotFound("Channel not found".to_string()))?;

            // Verify channel is open
            if channel.status != ChannelStatus::Open {
                return Err(BlockchainError::InvalidState("Channel is not open".to_string()));
            }

            // Verify signatures
            self.verify_update_signatures(channel, &final_balance, &signatures)?;

            channel.state.version
        };

        // Create final state
        let final_state = ChannelState {
            state_hash: self.compute_state_hash(&final_balance),
            version: current_state_version + 1,
            data: serde_json::to_vec(&final_balance)?,
        };

        // Update channel (drop mutex guard before await)
        {
            let mut channels = self.channels.lock().unwrap();
            let channel = channels.get_mut(channel_id).unwrap();
            channel.state = final_state.clone();
            channel.balance = final_balance;
            channel.status = ChannelStatus::Closing;
            channel.updated_at = chrono::Utc::now().timestamp();
        }

        // Create close request
        let close_request = CloseChannelRequest {
            channel_id: channel_id.to_string(),
            final_state,
            signatures,
        };

        // Send close message (ignore errors for demo)
        let _ = self.message_sender.send(ChannelMessage::CloseChannel(close_request)).await;

        info!("State channel closing: {}", channel_id);
        Ok(())
    }

    /// Dispute a state channel
    pub async fn dispute_channel(
        &self,
        channel_id: &str,
        disputed_balance: HashMap<String, f64>,
        evidence: Vec<u8>,
    ) -> Result<()> {
        info!("Disputing state channel: {}", channel_id);

        // Get channel and get current state version
        let current_state_version = {
            let mut channels = self.channels.lock().unwrap();
            let channel = channels.get_mut(channel_id)
                .ok_or_else(|| BlockchainError::NotFound("Channel not found".to_string()))?;

            channel.state.version
        };

        // Create disputed state
        let disputed_state = ChannelState {
            state_hash: self.compute_state_hash(&disputed_balance),
            version: current_state_version + 1,
            data: serde_json::to_vec(&disputed_balance)?,
        };

        // Update channel (drop mutex guard before await)
        {
            let mut channels = self.channels.lock().unwrap();
            let channel = channels.get_mut(channel_id).unwrap();
            channel.status = ChannelStatus::Disputed;
            channel.updated_at = chrono::Utc::now().timestamp();
        }

        // Create dispute request
        let dispute_request = DisputeChannelRequest {
            channel_id: channel_id.to_string(),
            disputed_state,
            evidence,
        };

        // Send dispute message (ignore errors for demo)
        let _ = self.message_sender.send(ChannelMessage::DisputeChannel(dispute_request)).await;

        info!("State channel disputed: {}", channel_id);
        Ok(())
    }

    /// Get channel information
    pub fn get_channel(&self, channel_id: &str) -> Result<StateChannel> {
        let channels = self.channels.lock().unwrap();
        channels.get(channel_id)
            .cloned()
            .ok_or_else(|| BlockchainError::NotFound("Channel not found".to_string()))
    }

    /// Get all channels for a participant
    pub fn get_participant_channels(&self, participant: &str) -> Vec<StateChannel> {
        let channels = self.channels.lock().unwrap();
        channels.values()
            .filter(|channel| channel.participants.contains(&participant.to_string()))
            .cloned()
            .collect()
    }

    /// Get channel updates
    pub fn get_channel_updates(&self, channel_id: &str) -> Result<Vec<ChannelUpdate>> {
        let updates = self.updates.lock().unwrap();
        Ok(updates.get(channel_id)
            .cloned()
            .unwrap_or_default())
    }

    /// Get channel statistics
    pub fn get_stats(&self) -> StateChannelStats {
        let channels = self.channels.lock().unwrap();
        let updates = self.updates.lock().unwrap();

        let total_channels = channels.len();
        let open_channels = channels.values()
            .filter(|c| c.status == ChannelStatus::Open)
            .count();
        let total_updates = updates.values()
            .map(|u| u.len())
            .sum();

        StateChannelStats {
            total_channels,
            open_channels,
            total_updates,
        }
    }

    /// Generate channel ID
    fn generate_channel_id(&self, participants: &[String]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(participants.join(":").as_bytes());
        hasher.update(chrono::Utc::now().timestamp().to_le_bytes());
        hex::encode(hasher.finalize())
    }

    /// Compute state hash
    fn compute_state_hash(&self, balance: &HashMap<String, f64>) -> Vec<u8> {
        let mut hasher = Sha256::new();
        let balance_data = serde_json::to_string(balance).unwrap();
        hasher.update(balance_data.as_bytes());
        hasher.finalize().to_vec()
    }

    /// Verify update signatures
    fn verify_update_signatures(
        &self,
        channel: &StateChannel,
        _balance: &HashMap<String, f64>,
        signatures: &HashMap<String, Vec<u8>>,
    ) -> Result<()> {
        // In a real implementation, this would verify actual signatures
        // For now, we'll just check that all participants have signed
        for participant in &channel.participants {
            if !signatures.contains_key(participant) {
                return Err(BlockchainError::InvalidSignature("Missing participant signature".to_string()));
            }
        }
        Ok(())
    }
}

/// State channel statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChannelStats {
    pub total_channels: usize,
    pub open_channels: usize,
    pub total_updates: usize,
}

impl Default for StateChannelManager {
    fn default() -> Self {
        let (manager, _) = Self::new();
        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_channel_lifecycle() {
        let (manager, _) = StateChannelManager::new();
        
        let participants = vec!["alice".to_string(), "bob".to_string()];
        let initial_balance = HashMap::from([
            ("alice".to_string(), 100.0),
            ("bob".to_string(), 100.0),
        ]);

        // Open channel
        let channel_id = manager.open_channel(participants.clone(), initial_balance.clone(), 3600).await.unwrap();
        
        // Get channel
        let channel = manager.get_channel(&channel_id).unwrap();
        assert_eq!(channel.status, ChannelStatus::Open);
        assert_eq!(channel.participants, participants);

        // Update channel
        let new_balance = HashMap::from([
            ("alice".to_string(), 80.0),
            ("bob".to_string(), 120.0),
        ]);
        let signatures = HashMap::from([
            ("alice".to_string(), vec![1, 2, 3]),
            ("bob".to_string(), vec![4, 5, 6]),
        ]);

        manager.update_channel(&channel_id, new_balance.clone(), signatures).await.unwrap();

        // Verify update
        let updated_channel = manager.get_channel(&channel_id).unwrap();
        assert_eq!(updated_channel.balance, new_balance);
        assert_eq!(updated_channel.state.version, 1);

        // Close channel
        let final_balance = HashMap::from([
            ("alice".to_string(), 70.0),
            ("bob".to_string(), 130.0),
        ]);
        let final_signatures = HashMap::from([
            ("alice".to_string(), vec![7, 8, 9]),
            ("bob".to_string(), vec![10, 11, 12]),
        ]);

        manager.close_channel(&channel_id, final_balance, final_signatures).await.unwrap();

        // Verify closure
        let closed_channel = manager.get_channel(&channel_id).unwrap();
        assert_eq!(closed_channel.status, ChannelStatus::Closing);
    }

    #[tokio::test]
    async fn test_channel_dispute() {
        let (manager, _) = StateChannelManager::new();
        
        let participants = vec!["alice".to_string(), "bob".to_string()];
        let initial_balance = HashMap::from([
            ("alice".to_string(), 100.0),
            ("bob".to_string(), 100.0),
        ]);

        let channel_id = manager.open_channel(participants, initial_balance, 3600).await.unwrap();

        // Dispute channel
        let disputed_balance = HashMap::from([
            ("alice".to_string(), 90.0),
            ("bob".to_string(), 110.0),
        ]);
        let evidence = vec![1, 2, 3, 4, 5];

        manager.dispute_channel(&channel_id, disputed_balance, evidence).await.unwrap();

        // Verify dispute
        let disputed_channel = manager.get_channel(&channel_id).unwrap();
        assert_eq!(disputed_channel.status, ChannelStatus::Disputed);
    }
}
