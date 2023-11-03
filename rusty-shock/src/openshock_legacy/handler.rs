use rosc::OscType;
use async_tungstenite::{tokio::connect_async, tungstenite::Message};
use crate::{config,osc::touchpoints::{self, Device}};
use chrono::{Utc, DateTime};

pub async fn handler(device: &Device,touchpoint: String,touchpoint_args: Vec<OscType>) {
    let legacy_config = &config::get_config().openshock_legacy;
    let touchpoint_config = &touchpoints::get_config().touchpoints;

    log::debug!("Legacy Touchpoint Handler");
    for arg in touchpoint_args {
        log::debug!("Touchpoint Argument: {:?}", arg);
    }

    let ws_url = format!("ws://{}:8080/ws",legacy_config.api_endpoint);

    // Connect to the WebSocket server asynchronously
    let (mut ws_stream, _) = match connect_async(&ws_url).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("Failed to connect to websocket: {}", e);
            return;
        }
    };


    //let (write, read) = ws_stream.split();
    //{ method: 0, intensity: 0, duration: 0, ids: ids, timestamp: Date.now() };

    let ws_command = format!("{{\"method\":{},\"intensity\":{},\"duration\":{},\"ids\":[{}],\"timestamp\":{}}}",device.method,device.intensity,device.duration,device.ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", "),chrono::Utc::now().timestamp_millis());
log::debug!("Sending WS Command: {}",ws_command);
    //write.send(ws_command).await.unwrap();

}