use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::IpAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub struct TCPServer {
    listener: TcpListener,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TCPRequest {
    Discovery,
    Download(String, usize),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TCPResponse {
    Discovery {
        session_id: String,
        files: BTreeMap<String, (String, usize)>,
    },
    FileChunk(Vec<u8>),
    Error(String),
}

impl TCPServer {
    pub async fn new(port: u16) -> Result<TCPServer> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .with_context(|| format!("Failed to bind to port {}", port))?;

        Ok(Self { listener })
    }

    pub async fn start<F, Fut>(&mut self, mut handler: F) -> Result<()>
    where
        F: FnMut(TCPRequest) -> Fut,
        Fut: Future<Output = TCPResponse>,
    {
        loop {
            let (mut stream, _) = self
                .listener
                .accept()
                .await
                .with_context(|| "Failed to accept connection")?;

            let (mut reader, mut writer) = stream.split();

            // Read the request from the client
            let mut buffer = [0; 1024];
            let n = reader
                .read(&mut buffer)
                .await
                .with_context(|| "Failed to read from socket")?;
            let request: TCPRequest = serde_json::from_slice(&buffer[..n])?;

            // Handle the request using the provided handler function
            let response = handler(request).await;

            // Send the response back to the client
            let response_str = serde_json::to_string(&response)?;
            writer
                .write_all(response_str.as_bytes())
                .await
                .with_context(|| "Failed to write to socket")?;
        }
    }
}

pub struct TCPClient {
    stream: TcpStream,
}

impl TCPClient {
    pub async fn new(ip: IpAddr, port: u16) -> Result<Self> {
        let stream = TcpStream::connect(format!("{ip}:{port}")).await?;
        Ok(Self { stream })
    }
    pub async fn send_request(&mut self, request: TCPRequest) -> Result<TCPResponse> {
        self.stream
            .write_all(
                serde_json::to_string(&request)
                    .expect("Should be able to serialize")
                    .as_bytes(),
            )
            .await?;

        let mut buffer = String::with_capacity(1024);
        self.stream.read_to_string(&mut buffer).await?;
        let response = serde_json::from_str::<TCPResponse>(&buffer)?;
        Ok(response)
    }
}
