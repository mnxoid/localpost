use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

#[derive(Serialize, Deserialize, Debug)]
pub enum UDPRequest {
    Discovery,
    Other,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UDPResponse {
    Discovery,
    Other,
}

pub struct UDPServer {
    socket: UdpSocket,
}

impl UDPServer {
    pub async fn new(port: u16) -> Result<UDPServer> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
        Ok(Self { socket })
    }

    pub async fn start<F, Fut>(&mut self, mut handler: F) -> Result<()>
    where
        F: FnMut(UDPRequest) -> Fut,
        Fut: Future<Output = UDPResponse>,
    {
        loop {
            let mut buffer = [0; 1024];
            let (size, src) = self
                .socket
                .recv_from(&mut buffer)
                .await
                .map_err(|e| anyhow!("Error receiving UDP packet: {e}"))?;
            let request = serde_json::from_slice(&buffer[..size])?;
            let response = handler(request).await;
            self.socket
                .send_to(&serde_json::to_vec(&response)?, &src)
                .await?;
        }
    }
}
