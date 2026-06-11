use crate::communication::ipc::{IPCClient, IPCRequest, IPCResponse};
pub use anyhow::Result;
use anyhow::anyhow;

pub fn list() -> Result<()> {
    if IPCClient::check_connection().is_err() {
        println!("No server currently running");
        return Ok(());
    };
    println!("Currently served files:\n");
    let response = IPCClient::request(IPCRequest::ListFiles)?;
    match response {
        IPCResponse::Files(files) => {
            for (k, (filename, chunks)) in files {
                println!("{} : {} ({} chunks)", k, filename, chunks);
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
