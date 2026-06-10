use crate::config::Config;
use crate::ipc::{IPCRequest, IPCResponse, IPCServer};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::net::UdpSocket;
use tokio::task;

pub struct DaemonState {
    pub served_files: BTreeMap<String, String>,
}

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

    pub async fn start(&mut self) -> Result<()> {
        loop {
            let mut buffer = [0; 1024];
            let (size, src) = self
                .socket
                .recv_from(&mut buffer)
                .await
                .map_err(|e| anyhow!("Error receiving UDP packet: {e}"))?;
            let request = serde_json::from_slice(&buffer[..size])?;
            let response = handle_udp_request(request);
            self.socket
                .send_to(&serde_json::to_vec(&response)?, &src)
                .await?;
        }
    }
}
pub async fn daemon(file: &str, key: &str, config: Config<'_>) -> Result<()> {
    println!("Starting daemon to serve file: {file} with key: {key}");

    let mut state = DaemonState {
        served_files: BTreeMap::from_iter([(key.to_string(), file.to_string())].into_iter()),
    };
    // Spawn the IPC server on a separate task
    let ipc_server_task = task::spawn(async move {
        let mut server = IPCServer::new()?;
        server.start(|request| handle_ipc_request(request, &mut state))?;
        Ok::<(), anyhow::Error>(())
    });

    // Spawn the UDP server task on a separate task
    let udp_server_task = task::spawn(async move {
        let mut server = UDPServer::new(config.port).await?;
        server.start().await
    });

    // Await the result of either task
    tokio::select! {
        ipc_result = ipc_server_task => {
            match ipc_result {
                Ok(_) => println!("IPC server shut down gracefully"),
                Err(e) => println!("IPC server failed to shut down: {e}"),
            }
        },
        udp_result = udp_server_task => {
            match udp_result {
                Ok(_) => println!("UDP server shut down gracefully"),
                Err(e) => println!("UDP server failed to shut down: {e}"),
            }
        },
    }

    Ok(())
}

fn handle_ipc_request(request: &IPCRequest, state: &mut DaemonState) -> IPCResponse {
    match request {
        IPCRequest::AddFile { key, path } => {
            if state.served_files.contains_key(key) {
                return IPCResponse::Error(format!("File already exists: {path} (key: {key})"));
            }
            state.served_files.insert(key.to_string(), path.to_string());
            IPCResponse::Ok
        }
        IPCRequest::RemoveFile { key } => {
            println!("Removing file in handler: {key}");
            if state.served_files.contains_key(key) {
                state.served_files.remove(key);
                if state.served_files.is_empty() {
                    IPCResponse::RemovedLastFile
                } else {
                    IPCResponse::Ok
                }
            } else {
                IPCResponse::Error(format!("File does not exist: {key}"))
            }
        }
        IPCRequest::ListFiles => IPCResponse::Files(state.served_files.clone()),
        IPCRequest::RemoveAllFiles => {
            state.served_files.clear();
            IPCResponse::RemovedLastFile
        }
        IPCRequest::Ping => IPCResponse::Ok,
    }
}

fn handle_udp_request(packet: UDPRequest) -> UDPResponse {
    match packet {
        UDPRequest::Discovery => {
            println!("Received discovery packet");
            UDPResponse::Discovery
        }
        UDPRequest::Other => {
            println!("Received unknown packet type");
            UDPResponse::Other
        }
    }
}
