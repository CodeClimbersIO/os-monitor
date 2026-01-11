//! WebSocket server for browser extension communication

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::browser::{BrowserError, handle_url_change};

/// Messages received from browser extension
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ExtensionMessage {
    #[serde(rename = "url_change")]
    UrlChange {
        url: String,
        title: String,
        #[serde(rename = "tabId")]
        tab_id: u32,
        timestamp: u64,
    },
    #[serde(rename = "status_check")]
    StatusCheck,
    #[serde(rename = "connection_init")]
    ConnectionInit,
}

/// Messages sent to browser extension
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum HostMessage {
    #[serde(rename = "status_response")]
    StatusResponse {
        blocking_active: bool,
        blocked_count: u32,
        server_version: String,
    },
    #[serde(rename = "block_url")]
    BlockUrl {
        url: String,
        #[serde(rename = "redirectTo")]
        redirect_to: String,
    },
    #[serde(rename = "redirect_tab")]
    RedirectTab {
        #[serde(rename = "tabId")]
        tab_id: u32,
        #[serde(rename = "redirectTo")]
        redirect_to: String,
    },
    #[serde(rename = "update_rules")]
    UpdateRules {
        rules: Vec<BlockingRule>,
    },
    #[serde(rename = "connection_ack")]
    ConnectionAck {
        message: String,
    },
}

/// Website blocking rule for extension
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockingRule {
    pub id: u32,
    pub priority: u32,
    pub url_pattern: String,
    pub redirect_to: String,
}

/// WebSocket connection manager
pub struct WebSocketServer {
    /// Active connections to browser extensions
    connections: Arc<Mutex<HashMap<SocketAddr, mpsc::UnboundedSender<HostMessage>>>>,
    /// Channel to send messages to all connected extensions
    broadcast_sender: mpsc::UnboundedSender<HostMessage>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        let (broadcast_sender, _broadcast_receiver) = mpsc::unbounded_channel();
        
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            broadcast_sender,
        }
    }
    
    /// Find the first available port starting from the base port
    async fn find_available_port(start_port: u16, max_attempts: u16) -> Result<u16, BrowserError> {
        for port in start_port..(start_port + max_attempts) {
            let addr = format!("127.0.0.1:{}", port);
            match TcpListener::bind(&addr).await {
                Ok(_) => {
                    println!("🔍 Found available port: {}", port);
                    return Ok(port);
                }
                Err(_) => {
                    println!("⚠️  Port {} is busy, trying next...", port);
                    continue;
                }
            }
        }
        Err(BrowserError::ConnectionFailed(format!(
            "No available ports found in range {}-{}",
            start_port,
            start_port + max_attempts - 1
        )))
    }
    
    /// Start the WebSocket server on an available port
    pub async fn start(self) -> Result<u16, BrowserError> {
        const BASE_PORT: u16 = 8080;
        const MAX_ATTEMPTS: u16 = 10;
        
        // Find available port
        let port = Self::find_available_port(BASE_PORT, MAX_ATTEMPTS).await?;
        let addr = format!("127.0.0.1:{}", port);
        
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| BrowserError::ConnectionFailed(format!("Failed to bind to {}: {}", addr, e)))?;
            
        println!("🚀 Ebb WebSocket server started on ws://{}", addr);
        println!("📡 Browser extensions can now connect on port {}", port);
        
        let connections = self.connections;
        let broadcast_sender = self.broadcast_sender;
        
        // Start accepting connections in background task
        tokio::spawn(async move {
            // Accept connections
            while let Ok((stream, addr)) = listener.accept().await {
                let connections_clone = Arc::clone(&connections);
                let broadcast_sender_clone = broadcast_sender.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, addr, connections_clone, broadcast_sender_clone).await {
                        eprintln!("Error handling connection from {}: {}", addr, e);
                    }
                });
            }
        });
        
        Ok(port)
    }
    
    /// Send message to all connected browser extensions
    pub async fn broadcast(&self, message: HostMessage) -> Result<(), BrowserError> {
        let connections = self.connections.lock()
            .map_err(|e| BrowserError::ConnectionFailed(format!("Mutex poisoned: {}", e)))?;
            
        let mut disconnected_addrs = Vec::new();
        
        for (addr, sender) in connections.iter() {
            if let Err(_) = sender.send(message.clone()) {
                disconnected_addrs.push(*addr);
            }
        }
        
        // Clean up disconnected connections
        drop(connections);
        if !disconnected_addrs.is_empty() {
            let mut connections = self.connections.lock().unwrap();
            for addr in disconnected_addrs {
                connections.remove(&addr);
                println!("🔌 Removed disconnected extension: {}", addr);
            }
        }
        
        Ok(())
    }
    
    /// Get the sender for broadcasting messages
    pub fn get_broadcast_sender(&self) -> mpsc::UnboundedSender<HostMessage> {
        self.broadcast_sender.clone()
    }
}

/// Handle individual WebSocket connection
async fn handle_connection(
    stream: TcpStream, 
    addr: SocketAddr,
    connections: Arc<Mutex<HashMap<SocketAddr, mpsc::UnboundedSender<HostMessage>>>>,
    _broadcast_sender: mpsc::UnboundedSender<HostMessage>,
) -> Result<(), BrowserError> {
    println!("🔗 New browser extension connected: {}", addr);
    
    let ws_stream = accept_async(stream).await
        .map_err(|e| BrowserError::ConnectionFailed(format!("WebSocket handshake failed: {}", e)))?;
        
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Create channel for sending messages to this specific connection
    let (tx, mut rx) = mpsc::unbounded_channel::<HostMessage>();
    
    // Register connection in local map
    {
        let mut conns = connections.lock().unwrap();
        conns.insert(addr, tx.clone());
    }
    
    // Register connection in global map for broadcasting
    {
        let mut global_conns = GLOBAL_CONNECTIONS.lock().unwrap();
        global_conns.insert(addr, tx);
    }
    
    // Update connection count
    {
        let mut count = CONNECTION_COUNT.lock().unwrap();
        *count += 1;
        println!("📊 Active connections: {}", *count);
    }
    
    // Send connection acknowledgment
    let ack_message = HostMessage::ConnectionAck {
        message: "Connected to Ebb OS Monitor".to_string(),
    };
    
    let ack_json = serde_json::to_string(&ack_message).unwrap();
    if let Err(e) = ws_sender.send(Message::Text(ack_json)).await {
        eprintln!("Failed to send connection ack: {}", e);
    }
    
    // Handle messages in both directions
    loop {
        tokio::select! {
            // Messages from extension
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_extension_message(text).await {
                            eprintln!("Error handling message from {}: {}", addr, e);
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("🔌 Extension {} disconnected", addr);
                        break;
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error from {}: {}", addr, e);
                        break;
                    }
                    None => {
                        println!("🔌 Extension {} connection closed", addr);
                        break;
                    }
                    _ => {
                        // Ignore other message types (binary, ping, pong)
                    }
                }
            }
            
            // Messages to extension
            msg = rx.recv() => {
                match msg {
                    Some(host_msg) => {
                        let json = serde_json::to_string(&host_msg).unwrap();
                        if let Err(e) = ws_sender.send(Message::Text(json)).await {
                            eprintln!("Failed to send message to {}: {}", addr, e);
                            break;
                        }
                    }
                    None => {
                        println!("🔌 Message channel closed for {}", addr);
                        break;
                    }
                }
            }
        }
    }
    
    // Clean up connection from local map
    {
        let mut conns = connections.lock().unwrap();
        conns.remove(&addr);
    }
    
    // Clean up connection from global map
    {
        let mut global_conns = GLOBAL_CONNECTIONS.lock().unwrap();
        global_conns.remove(&addr);
    }
    
    // Update connection count
    {
        let mut count = CONNECTION_COUNT.lock().unwrap();
        if *count > 0 {
            *count -= 1;
        }
        println!("📊 Active connections: {}", *count);
    }
    
    println!("🔌 Cleaned up connection for {}", addr);
    Ok(())
}

/// Handle message from browser extension
async fn handle_extension_message(message: String) -> Result<(), BrowserError> {
    let extension_msg: ExtensionMessage = serde_json::from_str(&message)
        .map_err(|e| BrowserError::InvalidMessage(format!("Invalid JSON: {}", e)))?;
        
    match extension_msg {
        ExtensionMessage::UrlChange { url, title, tab_id, timestamp: _ } => {
            println!("📍 URL changed: {} - {}", title, url);
            handle_url_change(&url, &title, tab_id).await?;
        }
        
        ExtensionMessage::StatusCheck => {
            println!("❓ Status check from extension");
            // Status response is handled by the connection handler
        }
        
        ExtensionMessage::ConnectionInit => {
            println!("🤝 Extension initialized connection");
        }
    }
    
    Ok(())
}

/// Global WebSocket server state
pub static WS_SERVER: once_cell::sync::Lazy<Arc<Mutex<Option<mpsc::UnboundedSender<HostMessage>>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// Global WebSocket port storage
static WS_PORT: once_cell::sync::Lazy<Arc<Mutex<Option<u16>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// Global connection count
pub static CONNECTION_COUNT: once_cell::sync::Lazy<Arc<Mutex<u32>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(0)));

/// Global connections map for broadcasting
pub static GLOBAL_CONNECTIONS: once_cell::sync::Lazy<Arc<Mutex<HashMap<SocketAddr, mpsc::UnboundedSender<HostMessage>>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Start the WebSocket server and return the port it's running on
pub async fn start_websocket_server() -> Result<u16, BrowserError> {
    let server = WebSocketServer::new();
    let broadcast_sender = server.get_broadcast_sender();
    
    // Store broadcast sender globally
    {
        let mut global_sender = WS_SERVER.lock()
            .map_err(|e| BrowserError::ConnectionFailed(format!("Mutex poisoned: {}", e)))?;
        *global_sender = Some(broadcast_sender);
    }
    
    let port = server.start().await?;
    
    // Store port globally
    {
        let mut global_port = WS_PORT.lock()
            .map_err(|e| BrowserError::ConnectionFailed(format!("Mutex poisoned: {}", e)))?;
        *global_port = Some(port);
    }
    
    Ok(port)
}

/// Get the current WebSocket server port
pub async fn get_websocket_port() -> Option<u16> {
    if let Ok(global_port) = WS_PORT.lock() {
        *global_port
    } else {
        None
    }
}

/// Send message to all connected browser extensions
pub async fn broadcast_to_extensions(message: HostMessage) -> Result<(), BrowserError> {
    println!("🔍 Attempting to broadcast message: {:?}", message);
    
    let global_conns = GLOBAL_CONNECTIONS.lock()
        .map_err(|e| BrowserError::ConnectionFailed(format!("Mutex poisoned: {}", e)))?;
    
    if global_conns.is_empty() {
        println!("❌ No connections available for broadcasting");
        return Err(BrowserError::SendFailed("No browser extensions connected".to_string()));
    }
    
    println!("📡 Broadcasting to {} connections", global_conns.len());
    let mut failed_connections = Vec::new();
    
    for (addr, sender) in global_conns.iter() {
        match sender.send(message.clone()) {
            Ok(_) => {
                println!("✅ Message sent to connection {}", addr);
            }
            Err(e) => {
                println!("❌ Failed to send to connection {}: {}", addr, e);
                failed_connections.push(*addr);
            }
        }
    }
    
    // Clean up failed connections
    drop(global_conns);
    if !failed_connections.is_empty() {
        let mut global_conns = GLOBAL_CONNECTIONS.lock()
            .map_err(|e| BrowserError::ConnectionFailed(format!("Mutex poisoned: {}", e)))?;
        for addr in failed_connections {
            global_conns.remove(&addr);
        }
    }
    
    println!("✅ Broadcast completed");
    Ok(())
}