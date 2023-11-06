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
    // Initialize the WebSocket client and store it in a Mutex 
    let ws_client = match WebSocketClient::get_or_init_websocket_client(&websocket_url).await {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to create WebSocket client: {}", e);
            return;
        },
    };

    // Command loop duration
    let mut interval = time::interval(Duration::from_millis(50));

    loop {
        interval.tick().await;
        let commandmap = commandmap.lock().await;
    }


}




// Saved for historical reference
/*
pub async fn handler(device: &Device,touchpoint: String,touchpoint_args: Vec<OscType>) {
    if touchpoint_args.is_empty() {
        // If there are no arguments, do not process further.
        log::warn!("No touchpoint arguments provided; handler will not proceed.");
        return;
    }

    // Safely attempt to extract the float value from the first argument.
    let first_arg_f32 = match touchpoint_args[0] {
        OscType::Float(val) => (device.intensity * val)*100.0,
        _ => {
            log::error!("First touchpoint argument is not a float; handler will not proceed.");
            return;
        }
    };

    // Ensure the value does not exceed 100 due to floating-point arithmetic quirks
    let first_arg_f32 = first_arg_f32.min(100.0);

    let legacy_config = &config::get_config().firmware;
    let touchpoint_config = &touchpoints::get_config().touchpoints;

    log::debug!("Legacy Touchpoint Handler");
    for arg in touchpoint_args {
        log::debug!("Touchpoint Argument: {:?}", arg);
    }


    // assume device.intensity is equivalent to 100% and scale it using the touchpoint intensity
    //let intensity = device.intensity * first_arg_f32;

    let ws_command = format!("{{\"method\":{},\"intensity\":{},\"duration\":{},\"ids\":[{}],\"timestamp\":{}}}",1,first_arg_f32,device.duration,device.ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", "),chrono::Utc::now().timestamp_millis());
    
    log::debug!("Sending WS Command: {}",ws_command);
    
    let ws_url = format!("ws://{}:8080/ws",legacy_config.api_endpoint);

    match WebSocketClient::get_or_init_websocket_client(ws_url.as_str()).await {
        Ok(ws_client) => {
                ws_client.send(ws_command).await.unwrap();
            // Use ws_client

        },
        Err(e) => {
            // Handle the error, possibly by logging or taking corrective action
            log::error!("Failed to get or initialize the WebSocket client: {}", e);
        }
    }

}
*/
