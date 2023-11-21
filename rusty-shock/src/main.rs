use osc::touchpoints::CommandState;
use serde::de;
use swing::Logger;
use tokio::sync::mpsc;
use std::sync::Arc;
use rosc::OscMessage;
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

    // initialize the features config
    let features_config = config::get_features_config();

    // Tasks Vector for Tokio tasks
    let mut tasks = Vec::new();

    // Initialize the command map and pass it to the OSC handler task
    let command_states = osc::touchpoints::initialize_commandmap().await;
    let command_states_clone = Arc::clone(&command_states);
    log::info!("Rusty Shock has started");
    log::debug!("Disabled features: {:?}", features_config.disabled_features);


    // Check if osc_router is in the disabled features list
    let (_osc_tx, osc_rx) = if features_config.disabled_features.contains(&"osc_router".to_string()) {
        log::warn!("OSC Router is disabled in the config.toml file. Please remove it from the disabled_features list to enable it.");
        (None, None)
    } else {
        log::info!("OSC Router is enabled.");
        let (tx, rx) = mpsc::channel::<OscMessage>(1);
        let tx_clone = tx.clone(); // Clone the transmitter
        // Spawn the OSC server task, passing it the sender part of the channel
        let _server_handle = tasks.push(tokio::spawn(async move {
            osc::osc::start_osc_server(tx_clone).await.expect("OSC server failed");
        }));
        (Some(tx), Some(rx))
    };

    // Check if world_command is in the disabled features list
    let (_world_command_tx, world_command_rx) = if features_config.disabled_features.contains(&"world_command_router".to_string()) {
        log::warn!("World Command is disabled in the config.toml file. Please remove it from the disabled_features list to enable it.");
        (None, None)
    } else {
        log::info!("World Command is enabled.");
        
        // Create a channel for WorldCommand messages
        let (tx, rx) = mpsc::channel::<WorldCommandEvent>(1);
        let _tx_clone = tx.clone(); // Clone the transmitter
        let _world_command_handle = tasks.push(tokio::spawn(async move {
            //world_command::handler::start_world_command_server(tx_clone).await.unwrap();
        }));
        (Some(tx), Some(rx))
    };

    // Check if touchpoints is in the disabled features list
    log::debug!("Disabled features before touchpoints check: {:?}", features_config.disabled_features);
    if features_config.disabled_features.contains(&"touchpoint_router".to_string()) {
        log::warn!("Touchpoints is disabled in the config.toml file...");
        return Ok(())
    } else {
        // touchpoints enabled logic
    
        log::info!("Touchpoints is enabled.");
        
        // Pass the desired delay in milliseconds here
        let delay_ms = 50; // for a 100 ms delay
        // Spawn the API handler task
        let _touchpoints_handle = tasks.push(tokio::spawn(async move {
            // Directly pass osc_rx and world_command_rx as they are already Option types
            if let Err(e) = osc::touchpoints::display_and_handle_touchpoints(osc_rx, world_command_rx, command_states, delay_ms).await {
                log::error!("Error in display_and_handle_touchpoints: {:?}", e);
            }
        }));
        
    // Check if firmware is in the disabled features list
    if features_config.disabled_features.contains(&"api_router".to_string()) {
        log::warn!("Firmware is disabled in the config.toml file. Please remove it from the disabled_features list to enable it.");
        return Ok(())
    } else {
        log::info!("Firmware is enabled.");
        let _firmware_handle = tasks.push(tokio::spawn(async {
            identify_firmware_start_server(command_states_clone).await;
        }));
    }


    // Wait for all the spawned tasks to complete
    for task in tasks {
        let _ = task.await?;
    }

    Ok(())
}
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
            let commandmap_clone = Arc::clone(&commandmap);
            openshock::handler::handler(config.firmware.api_endpoint.clone(),commandmap_clone).await;
        },
        "pishock" => {
            log::debug!("PiShock Touchpoint");
            log::warn!("PiShock is not yet implemented");
            //pishock::websocket::start_websocket_server().await;
        }
        _ => log::error!("Unknown touchpoint firmware: {}", config.firmware.firmware),
    }
}