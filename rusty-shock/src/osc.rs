/*/
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