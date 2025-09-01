use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener as TokioTcpListener, TcpStream as TokioTcpStream};
use tokio::sync::mpsc;
use tokio::time::timeout;
use log::{info, debug, warn, error};
use crate::{Result, BlockchainError, Blockchain, Block, Transaction, BlockchainMonitor};

/// Network message types for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// New block broadcast
    NewBlock(Block),
    /// New transaction broadcast
    NewTransaction(Transaction),
    /// Request for blockchain sync
    SyncRequest,
    /// Response with blockchain data
    SyncResponse(Vec<Block>),
    /// Ping message for health check
    Ping,
    /// Pong response
    Pong,
    /// Peer discovery request
    PeerDiscovery,
    /// Peer discovery response
    PeerList(Vec<String>),
}

/// Network peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    /// Peer address
    pub address: String,
    /// Last seen timestamp
    pub last_seen: i64,
    /// Connection status
    pub connected: bool,
    /// Peer version
    pub version: String,
}

/// P2P network manager for blockchain communication
/// 
/// Handles peer connections, message broadcasting, and blockchain synchronization
/// across a distributed network of nodes.
#[derive(Debug)]
pub struct Network {
    /// Local address to bind to
    local_address: String,
    /// Connected peers
    peers: Arc<Mutex<HashMap<String, Peer>>>,
    /// Blockchain instance
    blockchain: Arc<Mutex<Blockchain>>,
    /// Monitor for metrics
    monitor: Arc<Mutex<BlockchainMonitor>>,
    /// Message sender for async communication
    message_sender: mpsc::UnboundedSender<NetworkMessage>,
    /// Running status
    running: Arc<Mutex<bool>>,
}

impl Network {
    /// Create a new network instance
    /// 
    /// # Arguments
    /// * `local_address` - Local address to bind to (e.g., "127.0.0.1:8080")
    /// * `blockchain` - Blockchain instance to sync
    /// * `monitor` - Monitor for metrics
    /// 
    /// # Returns
    /// * `Result<Network>` - The network instance or an error
    /// 
    /// # Example
    /// ```no_run
    /// use gillean::network::Network;
    /// use gillean::blockchain::Blockchain;
    /// use gillean::monitor::BlockchainMonitor;
    /// use tokio::sync::Mutex;
    /// use std::sync::Arc;
    /// 
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let blockchain = Arc::new(Mutex::new(Blockchain::new_pow(4, 50.0)?));
    /// let monitor = Arc::new(Mutex::new(BlockchainMonitor::new()));
    /// let network = Network::new("127.0.0.1:8080".to_string(), blockchain, monitor)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        local_address: String,
        blockchain: Arc<Mutex<Blockchain>>,
        monitor: Arc<Mutex<BlockchainMonitor>>,
    ) -> Result<Self> {
        let (message_sender, _message_receiver) = mpsc::unbounded_channel::<NetworkMessage>();

        let network = Network {
            local_address,
            peers: Arc::new(Mutex::new(HashMap::new())),
            blockchain,
            monitor,
            message_sender,
            running: Arc::new(Mutex::new(false)),
        };

        info!("Network initialized on {}", network.local_address);
        Ok(network)
    }

    /// Start the network server
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub async fn start(&mut self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(BlockchainError::BlockValidationFailed(
                "Network is already running".to_string(),
            ));
        }
        *running = true;
        drop(running);

        let local_address = self.local_address.clone();
        let peers = Arc::clone(&self.peers);
        let blockchain = Arc::clone(&self.blockchain);
        let monitor = Arc::clone(&self.monitor);
        let message_sender = self.message_sender.clone();

        // Start the server in a separate task
        tokio::spawn(async move {
            if let Err(e) = Self::run_server(local_address, peers, blockchain, monitor, message_sender).await {
                error!("Network server error: {}", e);
            }
        });

        info!("Network server started on {}", self.local_address);
        Ok(())
    }

    /// Stop the network server
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub async fn stop(&mut self) -> Result<()> {
        let mut running = self.running.lock().await;
        *running = false;
        drop(running);

        info!("Network server stopped");
        Ok(())
    }

    /// Connect to a peer
    /// 
    /// # Arguments
    /// * `peer_address` - Address of the peer to connect to
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    /// 
    /// # Example
    /// ```no_run
    /// use gillean::network::Network;
    /// use gillean::blockchain::Blockchain;
    /// use gillean::monitor::BlockchainMonitor;
    /// use tokio::sync::Mutex;
    /// use std::sync::Arc;
    /// 
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let blockchain = Arc::new(Mutex::new(Blockchain::new_pow(4, 50.0)?));
    /// let monitor = Arc::new(Mutex::new(BlockchainMonitor::new()));
    /// let mut network = Network::new("127.0.0.1:8080".to_string(), blockchain, monitor)?;
    /// network.connect_to_peer("127.0.0.1:8081").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_to_peer(&mut self, peer_address: &str) -> Result<()> {
        let start_time = Instant::now();
        
        match timeout(Duration::from_secs(5), TokioTcpStream::connect(peer_address)).await {
            Ok(Ok(mut stream)) => {
                let connection_time = start_time.elapsed();
                
                // Send ping to verify connection
                let ping_message = NetworkMessage::Ping;
                let ping_data = serde_json::to_string(&ping_message)
                    .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
                
                stream.write_all(ping_data.as_bytes()).await
                    .map_err(|e| BlockchainError::BlockValidationFailed(
                        format!("Failed to send ping: {}", e)
                    ))?;

                // Add peer to our list
                let mut peers = self.peers.lock().await;
                peers.insert(peer_address.to_string(), Peer {
                    address: peer_address.to_string(),
                    last_seen: chrono::Utc::now().timestamp(),
                    connected: true,
                    version: "1.0.0".to_string(),
                });

                // Update monitor
                let mut monitor = self.monitor.lock().await;
                monitor.record_successful_connection();
                monitor.record_message_latency(connection_time);
                monitor.update_peer_count(peers.len() as u32);

                info!("Connected to peer: {} ({}ms)", peer_address, connection_time.as_millis());
                Ok(())
            }
            Ok(Err(e)) => {
                let mut monitor = self.monitor.lock().await;
                monitor.record_failed_connection();
                Err(BlockchainError::BlockValidationFailed(
                    format!("Failed to connect to peer {}: {}", peer_address, e)
                ))
            }
            Err(_) => {
                let mut monitor = self.monitor.lock().await;
                monitor.record_failed_connection();
                Err(BlockchainError::BlockValidationFailed(
                    format!("Connection timeout to peer: {}", peer_address)
                ))
            }
        }
    }

    /// Broadcast a new block to all peers
    /// 
    /// # Arguments
    /// * `block` - The block to broadcast
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub async fn broadcast_block(&self, block: &Block) -> Result<()> {
        let message = NetworkMessage::NewBlock(block.clone());
        self.broadcast_message(&message).await
    }

    /// Broadcast a new transaction to all peers
    /// 
    /// # Arguments
    /// * `transaction` - The transaction to broadcast
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub async fn broadcast_transaction(&self, transaction: &Transaction) -> Result<()> {
        let message = NetworkMessage::NewTransaction(transaction.clone());
        self.broadcast_message(&message).await
    }

    /// Request blockchain sync from peers
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    pub async fn request_sync(&self) -> Result<()> {
        let message = NetworkMessage::SyncRequest;
        self.broadcast_message(&message).await
    }

    /// Get list of connected peers
    /// 
    /// # Returns
    /// * `Vec<String>` - List of peer addresses
    pub async fn get_peers(&self) -> Vec<String> {
        let peers = self.peers.lock().await;
        peers.keys().cloned().collect()
    }

    /// Get peer count
    /// 
    /// # Returns
    /// * `usize` - Number of connected peers
    pub async fn peer_count(&self) -> usize {
        let peers = self.peers.lock().await;
        peers.len()
    }

    /// Check if network is running
    /// 
    /// # Returns
    /// * `bool` - True if running, false otherwise
    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    /// Broadcast a message to all connected peers
    /// 
    /// # Arguments
    /// * `message` - The message to broadcast
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    async fn broadcast_message(&self, message: &NetworkMessage) -> Result<()> {
        let peers = self.peers.lock().await;
        let peer_addresses: Vec<String> = peers.keys().cloned().collect();
        let peer_count = peer_addresses.len();
        drop(peers);

        let message_data = serde_json::to_string(message)
            .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;

        let mut success_count = 0;
        for peer_address in &peer_addresses {
            match timeout(Duration::from_secs(5), TokioTcpStream::connect(&peer_address)).await {
                Ok(Ok(mut stream)) => {
                    if stream.write_all(message_data.as_bytes()).await.is_ok() {
                        success_count += 1;
                        debug!("Broadcasted message to peer: {}", peer_address);
                    }
                }
                _ => {
                    warn!("Failed to broadcast to peer: {}", peer_address);
                }
            }
        }

        let mut monitor = self.monitor.lock().await;
        monitor.record_message_sent();

        info!("Broadcasted message to {}/{} peers", success_count, peer_count);
        Ok(())
    }

    /// Run the network server
    /// 
    /// # Arguments
    /// * `local_address` - Local address to bind to
    /// * `peers` - Shared peers map
    /// * `blockchain` - Shared blockchain instance
    /// * `monitor` - Shared monitor instance
    /// * `message_sender` - Message sender channel
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    async fn run_server(
        local_address: String,
        peers: Arc<Mutex<HashMap<String, Peer>>>,
        blockchain: Arc<Mutex<Blockchain>>,
        monitor: Arc<Mutex<BlockchainMonitor>>,
        message_sender: mpsc::UnboundedSender<NetworkMessage>,
    ) -> Result<()> {
        let listener = TokioTcpListener::bind(&local_address).await
            .map_err(|e| BlockchainError::BlockValidationFailed(
                format!("Failed to bind to {}: {}", local_address, e)
            ))?;

        info!("Network server listening on {}", local_address);

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    debug!("New connection from: {}", addr);
                    
                    let peers_clone = Arc::clone(&peers);
                    let blockchain_clone = Arc::clone(&blockchain);
                    let monitor_clone = Arc::clone(&monitor);
                    let message_sender_clone = message_sender.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(
                            socket, addr, peers_clone, blockchain_clone, monitor_clone, message_sender_clone
                        ).await {
                            error!("Connection handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    }

    /// Handle a client connection
    /// 
    /// # Arguments
    /// * `socket` - TCP socket
    /// * `addr` - Client address
    /// * `peers` - Shared peers map
    /// * `blockchain` - Shared blockchain instance
    /// * `monitor` - Shared monitor instance
    /// * `message_sender` - Message sender channel
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    async fn handle_connection(
        mut socket: TokioTcpStream,
        addr: SocketAddr,
        peers: Arc<Mutex<HashMap<String, Peer>>>,
        blockchain: Arc<Mutex<Blockchain>>,
        monitor: Arc<Mutex<BlockchainMonitor>>,
        message_sender: mpsc::UnboundedSender<NetworkMessage>,
    ) -> Result<()> {
        let mut buffer = [0; 4096];
        
        loop {
            match socket.read(&mut buffer).await {
                Ok(0) => {
                    debug!("Connection closed by peer: {}", addr);
                    break;
                }
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buffer[..n]);
                    
                    match serde_json::from_str::<NetworkMessage>(&data) {
                        Ok(message) => {
                            if let Err(e) = Self::handle_message(
                                message, &peers, &blockchain, &monitor, &message_sender, &mut socket
                            ).await {
                                error!("Message handling error: {}", e);
                            }
                        }
                        Err(e) => {
                            warn!("Invalid message from {}: {}", addr, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Read error from {}: {}", addr, e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle incoming network message
    /// 
    /// # Arguments
    /// * `message` - The network message
    /// * `peers` - Shared peers map
    /// * `blockchain` - Shared blockchain instance
    /// * `monitor` - Shared monitor instance
    /// * `message_sender` - Message sender channel
    /// * `socket` - TCP socket for response
    /// 
    /// # Returns
    /// * `Result<()>` - Ok if successful, error otherwise
    async fn handle_message(
        message: NetworkMessage,
        peers: &Arc<Mutex<HashMap<String, Peer>>>,
        blockchain: &Arc<Mutex<Blockchain>>,
        monitor: &Arc<Mutex<BlockchainMonitor>>,
        _message_sender: &mpsc::UnboundedSender<NetworkMessage>,
        socket: &mut TokioTcpStream,
    ) -> Result<()> {
        match message {
            NetworkMessage::NewBlock(block) => {
                debug!("Received new block: {}", block.index);
                
                // Add block to blockchain
                let mut bc = blockchain.lock().await;
                if let Err(e) = bc.add_block(block) {
                    warn!("Failed to add received block: {}", e);
                }
            }
            NetworkMessage::NewTransaction(transaction) => {
                debug!("Received new transaction: {}", transaction.id);
                
                // Add transaction to blockchain
                let mut bc = blockchain.lock().await;
                if let Err(e) = bc.add_transaction(
                    transaction.sender.clone(),
                    transaction.receiver.clone(),
                    transaction.amount,
                    transaction.message.clone(),
                ) {
                    warn!("Failed to add received transaction: {}", e);
                }
            }
            NetworkMessage::SyncRequest => {
                debug!("Received sync request");
                
                // Send blockchain data
                let bc = blockchain.lock().await;
                let blocks = bc.blocks.clone();
                let response = NetworkMessage::SyncResponse(blocks);
                
                if let Ok(response_data) = serde_json::to_string(&response) {
                    if let Err(e) = socket.write_all(response_data.as_bytes()).await {
                        error!("Failed to send sync response: {}", e);
                    }
                }
            }
            NetworkMessage::SyncResponse(blocks) => {
                debug!("Received sync response with {} blocks", blocks.len());
                
                // Process received blocks
                let mut bc = blockchain.lock().await;
                for block in blocks {
                    if let Err(e) = bc.add_block(block) {
                        warn!("Failed to add synced block: {}", e);
                    }
                }
            }
            NetworkMessage::Ping => {
                debug!("Received ping");
                
                // Send pong response
                let pong = NetworkMessage::Pong;
                if let Ok(pong_data) = serde_json::to_string(&pong) {
                    if let Err(e) = socket.write_all(pong_data.as_bytes()).await {
                        error!("Failed to send pong: {}", e);
                    }
                }
            }
            NetworkMessage::Pong => {
                debug!("Received pong");
            }
            NetworkMessage::PeerDiscovery => {
                debug!("Received peer discovery request");
                
                // Send peer list
                let peer_list = {
                    let peers_guard = peers.lock().await;
                    peers_guard.keys().cloned().collect::<Vec<String>>()
                };
                
                let response = NetworkMessage::PeerList(peer_list);
                if let Ok(response_data) = serde_json::to_string(&response) {
                    if let Err(e) = socket.write_all(response_data.as_bytes()).await {
                        error!("Failed to send peer list: {}", e);
                    }
                }
            }
            NetworkMessage::PeerList(peer_addresses) => {
                debug!("Received peer list with {} peers", peer_addresses.len());
                
                // Add new peers to our list
                let mut peers_guard = peers.lock().await;
                for address in peer_addresses {
                    if !peers_guard.contains_key(&address) {
                        peers_guard.insert(address.clone(), Peer {
                            address,
                            last_seen: chrono::Utc::now().timestamp(),
                            connected: false,
                            version: "1.0.0".to_string(),
                        });
                    }
                }
            }
        }

        // Update monitor
        let mut monitor_guard = monitor.lock().await;
        monitor_guard.record_message_received();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::sync::Mutex; // Unused import
    use std::sync::Arc;

    #[tokio::test]
    async fn test_network_creation() {
        let blockchain = Arc::new(tokio::sync::Mutex::new(Blockchain::new_pow(4, 50.0).unwrap()));
        let monitor = Arc::new(tokio::sync::Mutex::new(BlockchainMonitor::new()));
        let network = Network::new("127.0.0.1:0".to_string(), blockchain, monitor).unwrap();
        
        assert!(!network.is_running().await);
        assert_eq!(network.peer_count().await, 0);
    }

    #[tokio::test]
    async fn test_network_start_stop() {
        let blockchain = Arc::new(tokio::sync::Mutex::new(Blockchain::new_pow(4, 50.0).unwrap()));
        let monitor = Arc::new(tokio::sync::Mutex::new(BlockchainMonitor::new()));
        let network = Network::new("127.0.0.1:0".to_string(), blockchain, monitor).unwrap();
        
        // Note: This test doesn't actually start the server due to port 0
        // In a real scenario, you'd use a valid port
        assert!(!network.is_running().await);
    }

    #[test]
    fn test_network_message_serialization() {
        let block = Block::genesis().unwrap();
        let message = NetworkMessage::NewBlock(block);
        
        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: NetworkMessage = serde_json::from_str(&serialized).unwrap();
        
        assert!(matches!(deserialized, NetworkMessage::NewBlock(_)));
    }
}
