use osc::touchpoints::CommandState;
use swing::Logger;
use tokio::sync::mpsc;
use tokio::sync::Notify;
use std::sync::Arc;
use rosc::OscMessage;
use async_throttle::RateLimiter;
use tokio::time::{Duration,Instant};
use std::collections::HashMap;
use tokio::sync::Mutex;

// MODULES BABBBBBYYYYYY
mod config;
mod osc;
mod openshock_legacy;
mod openshock;
mod pishock;
mod world_command;

// New type for WorldCommand that defines the data we want to extract from the log file
#[derive(Debug)]
pub struct WorldCommandEvent {
    pub address: String,
    pub method: Vec<u8>,
    pub intensity: f32,
    pub duration: u64,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize the logging
    let logging_config = config::get_logging_config();
    Logger::with_config(logging_config).init().unwrap();

    // Initialize the command map and pass it to the OSC handler task
    let command_states = osc::touchpoints::initialize_commandmap().await;
    let command_states_clone = Arc::clone(&command_states);
    log::info!("Rusty Shock has started");

    // Create a channel for OSC messages
    let (tx, mut rx) = mpsc::channel::<OscMessage>(1); // buffer size of 100

    // Spawn the OSC server task, passing it the sender part of the channel
    let server_handle = tokio::spawn(async move {
        //osc::osc::start_osc_server(tx).await.expect("OSC server failed");
        osc::osc::start_osc_server(tx).await.expect("OSC server failed");
    });

    // Create a channel for WorldCommand messages
    let (world_command_tx, mut world_command_rx) = mpsc::channel::<WorldCommandEvent>(100); // buffer size as needed
    let world_command_handle = tokio::spawn(async move {
        world_command::handler::start_world_command_server(world_command_tx).await.unwrap();
    });
    
    // Pass the desired delay in milliseconds here
    let delay_ms = 50; // for a 100 ms delay
    // Spawn the API handler task
    let touchpoints_handle = tokio::spawn(async move {
        // Pass the receiver `rx` into the function
        osc::touchpoints::display_and_handle_touchpoints(rx,delay_ms,command_states).await.unwrap();
    });

    let websocket_handle = tokio::spawn(async {
        identify_firmware_start_server(command_states_clone).await;
    });

    // Wait for the server and API handler to complete. This will never return unless those tasks panic or are otherwise stopped
    let _ = tokio::try_join!(server_handle, touchpoints_handle,websocket_handle);

    Ok(())
}


async fn identify_firmware_start_server(commandmap: Arc<Mutex<HashMap<String, CommandState>>>){
    log::debug!("Firmware Router: Starting Web Server");

    let config = config::get_config();

    match config.firmware.firmware.to_ascii_lowercase().as_str(){
        "legacy" => {
            log::debug!("Legacy Touchpoint");
            let commandmap_clone = Arc::clone(&commandmap);
            openshock_legacy::handler::handler(config.firmware.api_endpoint.clone(),commandmap_clone).await;
        },
        "openshock" => {
            log::debug!("OpenShock Touchpoint");
            log::warn!("OpenShock is not yet implemented");
            //openshock::websocket::start_websocket_server().await;
        },
        "pishock" => {
            log::debug!("PiShock Touchpoint");
            log::warn!("PiShock is not yet implemented");
            //pishock::websocket::start_websocket_server().await;
        }
        _ => log::error!("Unknown touchpoint firmware: {}", config.firmware.firmware),
    }
}