use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use toml;
use async_osc::{prelude::*, OscPacket, OscSocket, OscType,Error,Result,OscMessage};
use std::net::{IpAddr,SocketAddr};
use std::str::FromStr;
use tokio::net::UdpSocket;
use async_std::stream::StreamExt;
use async_std::task;

//expect root Table and configure subtables, osc
#[derive(Deserialize)]
struct Config {
    osc: Osc
}

// Expected OSC config, listen_port,send_port,ip_address
#[derive(Deserialize)]
struct Osc {
    listen_port: u16,
    send_port: u16,
    ip_address: String,
}

// This will hold our configuration and only be initialized once using Lazy
static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config_file = fs::read_to_string("config.toml").expect("Unable to read config file");
    toml::from_str(&config_file).expect("Unable to parse config file")
});

//using CONFIG.osc.listen_port, open an async OSC socket from async-osc crate
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("OSC Config\n Listen Port: {}\n Send Port: {}\n IP Address: {}", CONFIG.osc.listen_port,CONFIG.osc.send_port,CONFIG.osc.ip_address);
    
    let ip_address = IpAddr::from_str(&CONFIG.osc.ip_address)?;
    let listen_addr = SocketAddr::from((ip_address, CONFIG.osc.listen_port));
    let send_addr = SocketAddr::from((ip_address, CONFIG.osc.send_port));
    //listen_to_osc(&listen_addr).await?;
    send_to_osc(&send_addr).await?;
    Ok(())
}

async fn send_to_osc(addr: &SocketAddr) -> async_osc::Result<()> {
    task::block_on(async {
        let socket = OscSocket::bind("localhost:0").await?;
        socket.connect("localhost:9000").await?;
        socket
            .send(("/volume", (0.9f32, "foo".to_string())))
            .await?;
        Ok(())
    })
    
}

async fn listen_to_osc(addr: &SocketAddr) -> async_osc::Result<()> {
    let mut socket = OscSocket::bind(addr).await?;
    println!("Listening on {}", addr);

    loop {
        while let Some(packet) = socket.next().await {
            let (packet, peer_addr) = packet?;
            eprintln!("Receive from {}: {:?}", peer_addr, packet);
        }
    }
}
