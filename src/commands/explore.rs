use crate::communication::udp::{UDPResponse, UDPServer};
use crate::config::Config;
use anyhow::Result;
use std::time::Duration;

pub async fn explore(config: Config<'_>) -> Result<()> {
    println!("Exploring currently served files on the local network");
    let mut udp_server = UDPServer::new(0).await?;
    udp_server
        .send_broadcast("255.255.255.255", config.port)
        .await?;
    let responses = udp_server.receive_responses(Duration::from_secs(5)).await?;

    for (addr, response) in responses {
        match response {
            UDPResponse::Discovery => println!("Server found: {addr}"),
            UDPResponse::Other => println!("Received other response: {addr}"),
        }
    }
    Ok(())
}
