use crate::commands::generate_key;
use crate::communication::ipc::{IPCRequest, IPCResponse, IPCServer};
use crate::communication::tcp::{TCPRequest, TCPResponse, TCPServer};
use crate::config::Config;
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
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
        served_files: BTreeMap::from_iter([(key.to_string(), (file.to_string(), file_size_mb))]),
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
    match packet {
        TCPRequest::Discovery => {
            let state = state.lock().await;
            println!("Received discovery packet");
            TCPResponse::Discovery {
                session_id: state.session_id.clone(),
                files: state.served_files.clone(),
            }
        }
        TCPRequest::Download(key, chunk_index) => {
            let state = state.lock().await;
            let Some((file_path, num_chunks)) = state.served_files.get(&key).cloned() else {
                return TCPResponse::Error("Key not found".to_string());
            };
            drop(state);
            if chunk_index >= num_chunks {
                return TCPResponse::Error("Chunk index out of range".to_string());
            }
            let Ok(chunk_content) = load_nth_megabyte(&file_path, chunk_index) else {
                return TCPResponse::Error("Failed to read file".to_string());
            };
            println!("Received download request for file: {key}, chunk_index: {chunk_index}");
            TCPResponse::FileChunk(chunk_content)
        }
    }
}

fn load_nth_megabyte(file_path: &str, n: usize) -> Result<Vec<u8>> {
    // Open the file in binary read mode
    let mut file = File::open(file_path)?;

    // Seek to the start of the n-th megabyte
    let byte_offset = (n * 1024 * 1024) as u64;
    file.seek(SeekFrom::Start(byte_offset))?;

    // Create a buffer to store the data
    let mut buffer = vec![0u8; 1024 * 1024];

    // Read 1 MB of data from the file into the buffer
    let bytes_read = file.read(&mut buffer)?;

    // Resize the buffer to the actual number of bytes read
    buffer.resize(bytes_read, 0);

    Ok(buffer)
}
