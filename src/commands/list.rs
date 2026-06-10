use crate::ipc::{IPCClient, IPCRequest, IPCResponse};
pub use anyhow::Result;
use anyhow::anyhow;

pub fn list() -> Result<()> {
    println!("Listing currently served files");
    let mut ipc = match IPCClient::new() {
        Ok(ipc) => ipc,
        Err(_) => {
            println!("No server currently running");
            return Ok(());
        }
    };
    println!("Currently served files");
    let response = ipc.request(IPCRequest::ListFiles)?;
    match response {
        IPCResponse::Files(files) => {
            for (k, v) in files {
                println!("{} : {}", k, v);
            }
        }
        _ => {
            return Err(anyhow!(
                "Invalid IPC response: {}",
                serde_json::to_string(&response)?
            ));
        }
    }

    Ok(())
}
