use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{Instant, timeout};

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
        socket.join_multicast_v4("239.42.42.42".parse()?, "0.0.0.0".parse()?)?;
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

    pub async fn send_broadcast(&mut self, broadcast_address: &str, port: u16) -> Result<()> {
        self.socket.set_broadcast(true)?;
        let request = UDPRequest::Discovery;
        let message = serde_json::to_vec(&request)?;

        self.socket
            .send_to(&message, (broadcast_address, port))
            .await?;

        Ok(())
    }

    pub async fn receive_responses(
        &mut self,
        timeout_dur: Duration,
    ) -> Result<Vec<(SocketAddr, UDPResponse)>> {
        let mut buffer = [0; 1024];
        let mut responses = Vec::new();

        let deadline = Instant::now() + timeout_dur;

        loop {
            let remaining = match deadline.checked_duration_since(Instant::now()) {
                Some(d) => d,
                None => break,
            };

            match timeout(remaining, self.socket.recv_from(&mut buffer)).await {
                Ok(Ok((size, src))) => {
                    let response = serde_json::from_slice(&buffer[..size])?;
                    responses.push((src, response));
                }
                Ok(Err(_)) => break,
                Err(_) => break, // timeout reached
            }
        }

        Ok(responses)
    }
}
