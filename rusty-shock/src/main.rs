use std::net::{IpAddr,SocketAddr};
use std::str::FromStr;
use tokio::net::UdpSocket;
use async_std::stream::StreamExt;
use async_std::task;
use swing::Logger;

//load config global module
mod config;
//load osc global module
mod osc;

//using CONFIG.osc.listen_port
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let logging_config = config::get_logging_config(); // Do not take a reference here
    Logger::with_config(logging_config).init().unwrap();

    let osc_config = &config::get_config().osc;
    
    let ip_address = IpAddr::from_str(&osc_config.ip_address)?;
    
    let listen_addr = SocketAddr::from((ip_address, osc_config.listen_port));
    let send_addr = SocketAddr::from((ip_address, osc_config.send_port));

    log::info!("Rusty Shock has started");
    log::debug!("\nOSC Config\n Listen Port: {}\n Send Port: {}\n IP Address: {}\n Listening on {}\n Sending on {}", osc_config.listen_port,osc_config.send_port,osc_config.ip_address,listen_addr, send_addr);
    //listen_to_osc(&listen_addr).await?;
    //send_to_osc(&send_addr).await?;
    
    Ok(())
}

/*

*/
