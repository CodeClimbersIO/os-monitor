//! Test WebSocket server for browser extension communication
//! This example runs just the WebSocket server without X11 dependencies
//! for testing browser extension connectivity and website blocking

use std::time::Duration;
use tokio::time;
use os_monitor::browser::{start_websocket_server, broadcast_to_extensions, HostMessage, BlockingRule};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("🚀 Starting Ebb WebSocket Test Server");
    
    // Start WebSocket server in background task
    let server_handle = tokio::spawn(async {
        if let Err(e) = start_websocket_server().await {
            eprintln!("WebSocket server error: {}", e);
        }
    });
    
    // Wait a moment for server to start
    time::sleep(Duration::from_secs(2)).await;
    
    println!("✅ WebSocket server should be running on ws://localhost:8081");
    println!("📱 Load your browser extension and check the console logs");
    println!("");
    
    // Test website blocking after 10 seconds
    println!("⏳ Will block Facebook in 10 seconds...");
    time::sleep(Duration::from_secs(10)).await;
    
    println!("🚫 Blocking Facebook!");
    let block_message = HostMessage::BlockUrl {
        url: "*facebook.com*".to_string(),
        redirect_to: "https://www.google.com/search?q=stay+focused".to_string(),
    };
    
    if let Err(e) = broadcast_to_extensions(block_message).await {
        eprintln!("Failed to send block message: {}", e);
    } else {
        println!("✅ Sent block command to all connected extensions");
    }
    
    // Test blocking rules update
    time::sleep(Duration::from_secs(5)).await;
    println!("📋 Updating blocking rules...");
    
    let blocking_rules = vec![
        BlockingRule {
            id: 1,
            priority: 1,
            url_pattern: "*facebook.com*".to_string(),
            redirect_to: "https://www.google.com/search?q=stay+focused".to_string(),
        },
        BlockingRule {
            id: 2,
            priority: 1,
            url_pattern: "*twitter.com*".to_string(),
            redirect_to: "https://www.google.com/search?q=stay+productive".to_string(),
        },
        BlockingRule {
            id: 3,
            priority: 1,
            url_pattern: "*instagram.com*".to_string(),
            redirect_to: "https://www.google.com/search?q=focus+time".to_string(),
        },
    ];
    
    let rules_message = HostMessage::UpdateRules {
        rules: blocking_rules,
    };
    
    if let Err(e) = broadcast_to_extensions(rules_message).await {
        eprintln!("Failed to send rules update: {}", e);
    } else {
        println!("✅ Updated blocking rules for Facebook, Twitter, and Instagram");
    }
    
    println!("");
    println!("🔄 Server running... Try visiting Facebook/Twitter/Instagram in your browser!");
    println!("📊 Check browser console for connection and blocking logs");
    println!("⚠️  Press Ctrl+C to stop");
    
    // Keep running
    server_handle.await?;
    
    Ok(())
}