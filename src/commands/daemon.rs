use crate::ipc::{IPCRequest, IPCResponse, IPCServer};
use anyhow::Result;
use std::collections::BTreeMap;
use tokio::task;

pub struct DaemonState {
    pub served_files: BTreeMap<String, String>,
}
pub async fn daemon(file: &str, key: &str) -> Result<()> {
    println!("Starting daemon to serve file: {file} with key: {key}");

    let mut state = DaemonState {
        served_files: BTreeMap::from_iter([(key.to_string(), file.to_string())].into_iter()),
    };
    // Spawn the IPC server on a separate task
    let server_task = task::spawn(async move {
        let mut server = IPCServer::start()?;
        server.handle_connections(|request| handle_request(request, &mut state))?;
        Ok::<(), anyhow::Error>(())
    });

    // Await the result if needed (not typically done for a long-running server)
    match server_task.await? {
        Ok(()) => println!("IPC server shut down gracefully"),
        Err(e) => eprintln!("Error in IPC server: {e}"),
    }
    Ok(())
}

fn handle_request(request: &IPCRequest, state: &mut DaemonState) -> IPCResponse {
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
