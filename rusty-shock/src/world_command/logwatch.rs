use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use tokio::sync::mpsc as async_mpsc;

pub fn start_watching_fs(
    log_directory: PathBuf,
    log_update_tx: async_mpsc::Sender<PathBuf>,
) -> notify::Result<()> {
    let (notify_tx, notify_rx) = std_mpsc::channel::<notify::Result<notify::Event>>();
        // Set up the watcher
        // Automatically select the best implementation for your platform.
        let mut watcher = notify::recommended_watcher(|res| {
            match res {
               Ok(event) => println!("event: {:?}", event),
               Err(e) => println!("watch error: {:?}", e),
            }
        })?;
    watcher.watch(&log_directory, RecursiveMode::NonRecursive)?;

    // The thread where notify runs
    std::thread::spawn(move || {
        for event in notify_rx.iter() {
            match event {
                Ok(Event { kind: EventKind::Create(_), paths, .. }) | 
                Ok(Event { kind: EventKind::Modify(_), paths, .. }) => {
                    for path in paths {
                        // You may want to add some logic here to verify it's a log file, and if it's newer than the current one.
                        log_update_tx.blocking_send(path).expect("Failed to send update log path");
                    }
                },
                Err(e) => eprintln!("watch error: {:?}", e),
                _ => {}
            }
        }
    });

    Ok(())
}