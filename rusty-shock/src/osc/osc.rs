use rosc::{OscMessage, OscPacket};
use std::net::{SocketAddr,IpAddr};
use std::str::FromStr;
use tokio::net::UdpSocket;
use crate::config;
use crate::osc::touchpoints;

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

pub async fn start_osc_server() -> Result<(), Box<dyn std::error::Error>> {
    let osc_config = &config::get_config().osc;
    
    let ip_address = IpAddr::from_str(&osc_config.ip_address)?;
    
    let listen_addr = SocketAddr::from((ip_address, osc_config.listen_port));
    let send_addr = SocketAddr::from((ip_address, osc_config.send_port));
    
    log::debug!("\nOSC Config\n Listen Port: {}\n Send Port: {}\n IP Address: {}\n Listening on {}\n Sending on {}", osc_config.listen_port,osc_config.send_port,osc_config.ip_address,listen_addr, send_addr);

    let socket = UdpSocket::bind(listen_addr).await?;
    let mut buf = [0u8; rosc::decoder::MTU];
    loop {
        let (size, addr) = socket.recv_from(&mut buf).await?;
        //println!("Received packet from {}", addr);

        match rosc::decoder::decode_udp(&buf) {
            Ok(ref pkt ) => {
                log::debug!("Received packet with size {} from: {}", size, addr);
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                handle_packet(packet).await;
            }
            Err(_) => todo!(),
        }
    }
}

async fn handle_packet(packet: OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            log::debug!("OSC Address: {}", msg.addr);
            let touchpoint_id = msg.addr.split("/").last().unwrap();
            log::debug!("Touchpoint ID: {}", touchpoint_id);
            touchpoints::touchpoint_router(touchpoint_id.to_string(),msg.args.clone()).await;
            for arg in msg.args {
                log::debug!("OSC Argument: {:?}", arg);
            }
        },
        OscPacket::Bundle(bundle) => {
            log::debug!("OSC Bundle: {:?}", bundle);
        },
    }
}