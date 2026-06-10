use crate::ipc::{IPCClient, IPCRequest, IPCResponse};
use anyhow::{Result, anyhow};

pub fn stop(key: Option<&String>) -> Result<()> {
    if let Err(_) = IPCClient::check_connection() {
        println!("No server currently running");
        return Ok(());
    };
    let response = if let Some(key) = key {
        println!("Stopping serving file with key: {key}");
        IPCClient::request(IPCRequest::RemoveFile {
            key: key.to_owned(),
        })?
    } else {
        println!("Stopping serving all files");
        IPCClient::request(IPCRequest::RemoveAllFiles)?
    };
    match response {
        IPCResponse::RemovedLastFile => {
            println!("Last file removed, daemon stopped");
        }
        IPCResponse::Ok => {
            println!("File removed successfully")
        }
        IPCResponse::Error(error) => {
            return Err(anyhow!("{}", error));
        }
        _ => {
            return Err(anyhow!("Unexpected response: {:?}", response));
        }
    }
    Ok(())
}
