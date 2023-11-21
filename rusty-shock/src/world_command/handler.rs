use notify::{RecursiveMode, Watcher};
use std::sync::mpsc as std_mpsc; // Use the standard library's MPSC for notify
use tokio::sync::mpsc;
use crate::WorldCommandEvent;
use std::env;
use std::path::PathBuf;

pub async fn start_world_command_server(mut tx: mpsc::Sender<WorldCommandEvent>) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("Starting WorldCommand server");
    
    // Set up the non-async std::sync::mpsc channel for notify
    let (notify_tx, notify_rx) = std_mpsc::channel::<notify::Result<notify::Event>>();

    // Set up the watcher
        // Automatically select the best implementation for your platform.
        let mut watcher = notify::recommended_watcher(|res| {
            match res {
               Ok(event) => println!("event: {:?}", event),
               Err(e) => println!("watch error: {:?}", e),
            }
        })?;

    // Specify the path to watch
    //let path_to_watch = r"%appdata%\..\LocalLow\VRChat\VRChat\"; // This should be the directory where the files are located
    let appdata_var = env::var("APPDATA").map_err(|_| "Could not find APPDATA environment variable")?;
    let vrchat_path_buf = PathBuf::from(appdata_var).join(r"..\LocalLow\VRChat\VRChat\");
    let vrchat_path = vrchat_path_buf.as_path(); // Convert PathBuf to &Path
    watcher.watch(vrchat_path, RecursiveMode::Recursive)?;


    
    // This task will run on a separate thread as it's a blocking operation
    tokio::spawn(async move {
        loop {
            match notify_rx.recv() {
                Ok(event) => {
                    println!("Received filesystem event: {:?}", event);
                    // Here you would implement your logic to handle the event,
                    // such as reading the latest file, parsing the event, and creating a WorldCommandEvent.
                    // You'll then send this event through the tx channel.
                    
                    // Example event handling:
                    // let world_event = parse_event_to_world_command(event);
                    // tx.send(world_event).await.expect("Failed to send world command event");
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    Ok(())
}
