use rosc::OscType;
use async_tungstenite::{tokio::connect_async, tungstenite::Message,WebSocketStream};
use crate::{config,osc::touchpoints::{self, Device, CommandState},openshock_legacy::websocket::WebSocketClient};
use chrono::{Utc, DateTime};
use tokio::sync::Mutex;
use std::sync::Arc;
use futures_util::SinkExt;
use tokio::net::TcpStream;
use async_throttle::RateLimiter;
use tokio::time::{Instant,self,Duration};
use std::collections::HashMap;
use serde_json::json;
use std::fmt::Write;

// For reference
/*pub struct CommandState {
    pub id: String,
    pub duration: u64,
    pub intensity: f32,
    pub last_issued: Instant,
    pub expiry: Instant,
} */

pub async fn handler(websocket_url: String,commandmap: Arc<Mutex<HashMap<String, CommandState>>>){

    log::debug!("Legacy Touchpoint Handler");

    let websocket_url = format!("ws://{}:8080/ws",websocket_url);

    log::debug!("WebSocket URL: {}",websocket_url);

    let touchpoint_config = &touchpoints::get_config().touchpoints;

    // Initialize the WebSocket client and store it in a Mutex 
    let ws_client = match WebSocketClient::get_or_init_websocket_client(&websocket_url).await {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to create WebSocket client: {}", e);
            return;
        },
    };

        // WebSocket client mutex
        let ws_client_mutex = Arc::new(Mutex::new(ws_client));

    // Command loop duration
    //let mut interval = time::interval(Duration::from_millis(50));
    let interval = Duration::from_millis(150);
    loop {
        let now = Instant::now();
    
        // First, clone the command map so we can iterate without holding the lock.
        // This step assumes that the command map doesn't get too large to clone each iteration.
        // If the command map is very large, this might not be the most efficient approach.
        let cloned_commandmap = {
            let commandmap_lock = commandmap.lock().await;
            commandmap_lock.clone() // We clone here to avoid holding the lock while sending messages.
        };

        let mut batch_commands: HashMap<(u8, u8), Vec<u16>> = HashMap::new();

        // Iterate over cloned command map and send commands for non-expired IDs

        for (key, command_state) in cloned_commandmap.iter() {
            if command_state.expiry > now && command_state.intensity > 0.0 {
                // Parse and prepare your keys
                let (id_str, method_str) = key.split_once('_').expect("Invalid key format");
                let id = id_str.parse::<u16>().expect("Invalid ID format");
                let method = method_str.parse::<u8>().expect("Invalid method format");
                let intensity = (command_state.intensity * 100.0).round() as u8;
        
                // Group commands by method and intensity
                batch_commands
                    .entry((method, intensity))
                    .or_default()
                    .push(id);
            }
        }

        for ((method, intensity), ids) in batch_commands {
            let json_payload = json!({
                "method": method,
                "intensity": intensity,
                "duration": 50, // fixed duration for all commands
                "ids": ids,
                "timestamp": Utc::now().timestamp_millis(),
            }).to_string();
        
            log::debug!("Sending JSON payload: {}", json_payload);
            // Acquire lock and send
            let client_lock = ws_client_mutex.lock().await;
            client_lock.send(json_payload).await.unwrap(); // Properly handle errors
        }
    
        // Sleep until the next loop iteration to maintain the loop interval
        time::sleep(interval).await;
    }

}
