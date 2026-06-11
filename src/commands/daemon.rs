use crate::commands::generate_key;
use crate::communication::ipc::{IPCRequest, IPCResponse, IPCServer};
use crate::communication::tcp::{TCPRequest, TCPResponse, TCPServer};
use crate::config::Config;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;

pub struct DaemonState {
    pub served_files: BTreeMap<String, (String, usize)>,
    pub session_id: String,
}

pub async fn daemon(file: &str, key: &str, config: Config<'_>) -> Result<()> {
    println!("Starting daemon to serve file: {file} with key: {key}");

    let file_size_mb =
        (Path::new(file).metadata()?.len() as f64 / (1024.0 * 1024.0)).ceil() as usize;

    let state = Arc::new(Mutex::new(DaemonState {
        served_files: BTreeMap::from_iter(
            [(key.to_string(), (file.to_string(), file_size_mb))].into_iter(),
        ),
        session_id: generate_key(&config),
    }));
    // Spawn the IPC server on a separate task
    let ipc_state = state.clone();
    let ipc_server_task = task::spawn(async move {
        let mut server = IPCServer::new()?;
        server
            .start(async |request| {
                let state = ipc_state.clone();
                handle_ipc_request(request, state).await
            })
            .await
    });

    // Spawn the TCP server task on a separate task
    let mut server = TCPServer::new(config.port).await?;
    let tcp_state = state.clone();
    let tcp_server_task = task::spawn(async move {
        server
            .start(async |packet| {
                let state = tcp_state.clone();
                handle_tcp_request(packet, state).await
            })
            .await
    });

    // Await the result of either task
    tokio::select! {
        ipc_result = ipc_server_task => {
            match ipc_result {
                Ok(_) => println!("IPC server shut down gracefully"),
                Err(e) => println!("IPC server failed to shut down: {e}"),
            }
        },
        tcp_result = tcp_server_task => {
            match tcp_result {
                Ok(_) => println!("TCP server shut down gracefully"),
                Err(e) => println!("TCP server failed to shut down: {e}"),
            }
        },
    }

    Ok(())
}

async fn handle_ipc_request(request: IPCRequest, state: Arc<Mutex<DaemonState>>) -> IPCResponse {
    match request {
        IPCRequest::AddFile { key, path, chunks } => {
            let mut state = state.lock().await;
            if state.served_files.contains_key(&key) {
                return IPCResponse::Error(format!("File already exists: {path} (key: {key})"));
            }
            state
                .served_files
                .insert(key.to_string(), (path.to_string(), chunks));
            IPCResponse::Ok
        }
        IPCRequest::RemoveFile { key } => {
            let mut state = state.lock().await;
            println!("Removing file in handler: {key}");
            if state.served_files.contains_key(&key) {
                state.served_files.remove(&key);
                if state.served_files.is_empty() {
                    IPCResponse::RemovedLastFile
                } else {
                    IPCResponse::Ok
                }
            } else {
                IPCResponse::Error(format!("File does not exist: {key}"))
            }
        }
        IPCRequest::ListFiles => IPCResponse::Files(state.lock().await.served_files.clone()),
        IPCRequest::RemoveAllFiles => {
            state.lock().await.served_files.clear();
            IPCResponse::RemovedLastFile
        }
        IPCRequest::Ping => IPCResponse::Pong(state.lock().await.session_id.clone()),
    }
}

async fn handle_tcp_request(packet: TCPRequest, state: Arc<Mutex<DaemonState>>) -> TCPResponse {
    let state = state.lock().await;
    match packet {
        TCPRequest::Discovery => {
            println!("Received discovery packet");
            TCPResponse::Discovery {
                session_id: state.session_id.clone(),
                files: state.served_files.clone(),
            }
        }
        TCPRequest::Download(key) => {
            println!("Received unknown packet type");
            TCPResponse::Other
        }
    }
}
