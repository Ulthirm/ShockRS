use swing::Logger;

//load config global module
mod config;
//load osc global module
mod osc;
//load openshock-legacy global module
mod openshock_legacy;
//load openshock global module
mod openshock;
//load pishock global module
mod pishock;

//using CONFIG.osc.listen_port
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let logging_config = config::get_logging_config(); // Do not take a reference here
    Logger::with_config(logging_config).init().unwrap();

    log::info!("Rusty Shock has started");

    
    let server_handle = tokio::spawn(async {
        osc::osc::start_osc_server().await.expect("OSC server failed");
    });

    osc::touchpoints::display_touchpoints().await?;
    
    let _ = server_handle.await;

    Ok(())
}

