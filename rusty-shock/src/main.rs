use swing::Logger;
use tokio::sync::mpsc;
use tokio::sync::Notify;
use std::sync::Arc;
use rosc::OscMessage;
use async_throttle::RateLimiter;
use tokio::time::Duration;

// MODULES BABBBBBYYYYYY
mod config;
mod osc;
mod openshock_legacy;
mod openshock;
mod pishock;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Initialize the logging
    let logging_config = config::get_logging_config();
    Logger::with_config(logging_config).init().unwrap();

    // Initialize the command map and pass it to the OSC handler task
    let command_states = osc::touchpoints::initialize_commandmap().await;

    log::info!("Rusty Shock has started");

    // Create a channel for OSC messages
    let (tx, mut rx) = mpsc::channel::<OscMessage>(1); // buffer size of 100

    // Spawn the OSC server task, passing it the sender part of the channel
    let server_handle = tokio::spawn(async move {
        //osc::osc::start_osc_server(tx).await.expect("OSC server failed");
        osc::osc::start_osc_server(tx).await.expect("OSC server failed");
    });

    // Spawn the API handler task
    let touchpoints_handle = tokio::spawn(async {
            // Pass the desired delay in milliseconds here
    let delay_ms = 50; // for a 100 ms delay
        // Pass the receiver `rx` into the function
        osc::touchpoints::display_and_handle_touchpoints(rx,delay_ms).await.unwrap();
    });

    // Display the touchpoints
    //osc::touchpoints::display_touchpoints().await?;

    // Wait for the server and API handler to complete. This will never return unless those tasks panic or are otherwise stopped
    let _ = tokio::try_join!(server_handle, touchpoints_handle);

    Ok(())
}
