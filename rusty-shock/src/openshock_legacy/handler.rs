use rosc::OscType;
use async_tungstenite::{tokio::connect_async, tungstenite::Message,WebSocketStream};
use crate::{config,osc::touchpoints::{self, Device},openshock_legacy::websocket::WebSocketClient};
use chrono::{Utc, DateTime};
use tokio::sync::Mutex;
use std::sync::Arc;
use futures_util::SinkExt;
use tokio::net::TcpStream;

pub async fn handler(device: &Device,touchpoint: String,touchpoint_args: Vec<OscType>) {
    let legacy_config = &config::get_config().openshock_legacy;
    let touchpoint_config = &touchpoints::get_config().touchpoints;

    log::debug!("Legacy Touchpoint Handler");
    for arg in touchpoint_args {
        log::debug!("Touchpoint Argument: {:?}", arg);
    }
    //let (write, read) = ws_stream.split();
    //{ method: 0, intensity: 0, duration: 0, ids: ids, timestamp: Date.now() };

    let ws_client = Arc::new(WebSocketClient {
        ws_stream: Mutex::new(None),
        url: Mutex::new(String::new()),
    });


    let ws_command = format!("{{\"method\":{},\"intensity\":{},\"duration\":{},\"ids\":[{}],\"timestamp\":{}}}",device.method,device.intensity,device.duration,device.ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", "),chrono::Utc::now().timestamp_millis());
    
    log::debug!("Sending WS Command: {}",ws_command);
    
    let ws_url = format!("ws://{}:8080/ws",legacy_config.api_endpoint);
    
    // Create a new WebSocket connection if necessary
    match WebSocketClient::create_new(ws_url.as_str(), ws_client.clone()).await {
        Ok(client) => {
            client.send(ws_command).await.unwrap();
        }
        Err(e) => {
            log::error!("Failed to create or retrieve the WebSocket client: {:?}", e);
        }
    }

    //write.send(ws_command).await.unwrap();

}