use rosc::{OscMessage, OscPacket};
use std::net::{SocketAddr,IpAddr};
use std::str::FromStr;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{Sender,Receiver};
use crate::config;
use async_throttle::RateLimiter;
use std::sync::Arc;

/*
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
*/

pub async fn start_osc_server(sender: Sender<OscMessage>) -> Result<(), Box<dyn std::error::Error>> {
    let osc_config = &config::get_config().osc;
    
    let ip_address = IpAddr::from_str(&osc_config.ip_address)?;
    
    let listen_addr = SocketAddr::from((ip_address, osc_config.listen_port));
    let send_addr = SocketAddr::from((ip_address, osc_config.send_port));

    //https://docs.rs/async-throttle/0.3.2/async_throttle/struct.RateLimiter.html
    
    log::debug!("\nOSC Config\n Listen Port: {}\n Send Port: {}\n IP Address: {}\n Listening on {}\n Sending on {}", osc_config.listen_port,osc_config.send_port,osc_config.ip_address,listen_addr, send_addr);

    let socket = UdpSocket::bind(listen_addr).await?;
    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
        let (size, addr) = socket.recv_from(&mut buf).await?;
        match rosc::decoder::decode_udp(&buf[..size]) {
            Ok((_, packet)) => {
                log::debug!("Received packet with size {} from: {}", size, addr);
                if let Err(e) = handle_packet(packet, sender.clone()).await {
                    log::error!("Failed to handle OSC packet: {}", e);
                    // Decide what to do here - stop the server or just log the error
                }
            }
            Err(e) => {
                log::error!("Failed to decode OSC packet: {}", e);
                // Handle the error as needed
            },
        }
    }

    Ok(())
}

async fn handle_packet(packet: OscPacket, sender: Sender<OscMessage>) -> Result<(), String> {
    match packet {
        OscPacket::Message(msg) => {
            // Instead of breaking, return an error if the message can't be sent
            sender.send(msg).await.map_err(|e| e.to_string())
        }
        OscPacket::Bundle(bundle) => {
            // If you want to handle bundles, you would do it here.
            // For now, we can just ignore them or log a message
            log::debug!("OSC Bundle: {:?}", bundle);
            Ok(())
        },
    }
}
