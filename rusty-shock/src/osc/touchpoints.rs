use rosc::{OscType,OscMessage};
use serde::Deserialize;
use std::fs;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration,self,Instant};
use tokio::sync::Mutex;

use crate::{openshock_legacy,openshock,pishock};

#[derive(Deserialize)]
pub struct Touchpoints {
    pub touchpoints: Vec<Device>,
}

#[derive(Deserialize)]
pub struct Device {
    pub address: String,
    pub method: Vec<u8>,
    pub intensity: f32,
    pub duration: u64,
    pub ids: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct CommandState {
    pub id: String,
    pub duration: u64,
    pub intensity: f32,
    pub last_issued: Instant,
    pub expiry: Instant,
}

pub static TOUCHPOINTS: Lazy<Touchpoints> = Lazy::new(|| {
    let touchpoints_path = "touchpoints.toml";
    let touchpoints_str = fs::read_to_string(touchpoints_path)
        .expect("Failed to read touchpoints.toml");
    toml::from_str(&touchpoints_str)
        .expect("Failed to parse touchpoints")
});

pub fn get_config() -> &'static Touchpoints {
    &TOUCHPOINTS
}

pub async fn initialize_commandmap() -> Arc<Mutex<HashMap<String, CommandState>>> {
    let command_states: Arc<Mutex<HashMap<String, CommandState>>> = Arc::new(Mutex::new(HashMap::new()));
    for device in TOUCHPOINTS.touchpoints.iter() {
        for &id in &device.ids {
            // Convert the u32 id to a String based on the expected format in the rest of the code
            let id_string = id.to_string(); // Ensure this is the format you want, possibly including method
            
            // Initialize with default values: 0 duration and intensity, expiry as now
            let command_state = CommandState {
                id: id_string.clone(), // Cloning the string
                duration: 0,           // Starting at 0
                intensity: 0.0,        // Starting at 0.0
                last_issued: Instant::now(), // You could also choose another sentinel value
                expiry: Instant::now(), // Expiry starts at the current time
            };
            // Insert the command state with a String key
            command_states.lock().await.insert(id_string, command_state);
        }
    }
    command_states
}


pub async fn display_and_handle_touchpoints(mut rx: Receiver<OscMessage>,delay_ms: u64,commandmap: Arc<Mutex<HashMap<String, CommandState>>>,) -> Result<(), Box<dyn std::error::Error>> {
    // Display the touchpoints as before
    for device in TOUCHPOINTS.touchpoints.iter() {
        log::debug!("\nTouchpoint: {}\n IDs: {}", device.address, device.ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", "));
    }

    // Interval duration
    let interval = Duration::from_millis(delay_ms);
    let mut interval_timer = time::interval(interval);
        // Tick immediately to align the start of the interval with the start of processing.
    interval_timer.tick().await; // Wait for the next interval tick

    // Main loop
    loop {
        tokio::select! {
            message = rx.recv() => {
                if let Some(msg) = message {
                    // Process the incoming message and update hashmap
                    process_message(&msg, Arc::clone(&commandmap)).await;
                } else {
                    // If the channel is closed, exit the loop.
                    break;
                }
            }
            _ = interval_timer.tick() => {
                // This is where you would handle the tick delay, 
                // potentially sending out commands based on the updated commandmap states.
            }
        }
    }

    Ok(())
}

// Helper function to process each message
async fn process_message(msg: &OscMessage,commandmap: Arc<Mutex<HashMap<String, CommandState>>>,) {
    // Get all the potential shocker IDs
    let shocker_ids = extract_shocker_ids(msg.addr.clone()).await;
    log::debug!("Shocker IDs: {:?}", shocker_ids);
    // Safely attempt to extract the float value from the first argument.
    // This is an unsafe assumption of how OSC will output from VRChat
    let intensity = match msg.args[0] {
        OscType::Float(val) => val,
        _ => {
            log::error!("First touchpoint argument is not a float; handler will not proceed.");
            return;
        }
    };
    log::debug!("Intensity: {}", intensity);

    // Assume the duration is 50 ms
    // We're assuming this because our loop occurs every 50 ms and we do not trust the remote client to process duration well
    // We may later add a match for firmware type and adjust the duration accordingly based on the firmware
    let duration = 50;
    log::debug!("Duration: {}", duration);


    // Process each shocker ID
    for shocker_id in shocker_ids {
        process_shocker_by_id(shocker_id, msg, commandmap.clone(), duration, intensity).await;
    }
}

async fn process_shocker_by_id(shocker_id: String,message: &OscMessage,command_map: Arc<Mutex<HashMap<String, CommandState>>>, duration: u64,intensity: f32,) {
    log::debug!("Processing shocker ID: {}", shocker_id);
    let mut command_states = command_map.lock().await;
    // If the command state for this ID exists, update it
    if let Some(command_state) = command_states.get_mut(&shocker_id) {
        command_state.last_issued = Instant::now();
        command_state.expiry = calculate_expiry(command_state.duration.into()).await;
        command_state.intensity = intensity;
        command_state.duration = duration;
    } else {
        // If the command state for this ID does not exist, create it
        let new_command_state = CommandState {
            id: shocker_id.clone(), // Clone the shocker_id since you're inserting it into the HashMap
            duration,
            intensity,
            last_issued: Instant::now(),
            expiry: calculate_expiry(duration).await,
        };
        command_states.insert(shocker_id, new_command_state); // Now it's expecting a String key
    }
    // Release the lock automatically when it goes out of scope
}

async fn extract_shocker_ids(message_addr: String) -> Vec<String> {
    let message_addr = match message_addr.split("/").last() {
        Some(addr) => addr,
        None => { 
            log::error!("Failed to extract shocker IDs from message address: {}", message_addr);
            return Vec::new();
        },
    };
    log::debug!("Extracting shocker IDs from message address: {}", message_addr);

    if let Some(device) = TOUCHPOINTS.touchpoints.iter().find(|device| device.address == message_addr) {
        device.ids.iter().flat_map(|id| {
            device.method.iter().map(move |&method| format!("{}_{}", id, method))
        }).collect()
    } else {
        log::error!("Unknown touchpoint: {}", message_addr);
        Vec::new()
    }
}



async fn calculate_expiry(duration: u64) -> Instant {
    let duration =Duration::from_millis(duration);
    Instant::now() + duration
}

//takes in a given touchpoint and routes it to the appropriate function for it's firmware
// Retained for historical reference, invalid now that we've moved firmware to the config
/*
pub async fn touchpoint_router(touchpoint: String, touchpoint_args: Vec<OscType>) {
    log::debug!("Touchpoint Router: {}", touchpoint);

    //split touchpoint at / to the last element
    let touchpoint = touchpoint.split("/").last().unwrap().to_string();

    if let Some(device) = TOUCHPOINTS.touchpoints.iter().find(|device| device.address == touchpoint) {
        log::debug!("Touchpoint Firmware: {}", device.firmware);
        match config.firmware.to_ascii_lowercase().as_str() {
            "legacy" => {
                log::debug!("Legacy Touchpoint");
                openshock_legacy::handler::handler(device,touchpoint,touchpoint_args.clone()).await;
            },
            "openshock" => {
                log::debug!("OpenShock Touchpoint");
                openshock::handler::handler(touchpoint,touchpoint_args.clone()).await;
            },
            "pishock" => {
                log::debug!("PiShock Touchpoint");
                pishock::handler::handler(touchpoint,touchpoint_args.clone()).await;
            }
            _ => log::error!("Unknown touchpoint firmware: {}", device.firmware),
        }
    } else {
        log::error!("Unknown touchpoint: {}", touchpoint);
    }

    for arg in touchpoint_args {
        log::debug!("Touchpoint Argument: {:?}", arg);
    }
}
*/
