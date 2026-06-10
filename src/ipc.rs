use crate::constants::SOCKET_NAME;
use anyhow::{Result, anyhow};
use interprocess::local_socket::traits::{ListenerExt, Stream};
use interprocess::local_socket::{
    GenericNamespaced, Listener, ListenerOptions, Stream as LocalSocketStream, ToNsName,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub enum IPCRequest {
    Ping,
    AddFile { key: String, path: String },
    RemoveFile { key: String },
    RemoveAllFiles,
    ListFiles,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IPCResponse {
    Ok,
    Files(BTreeMap<String, String>), // (key, path)
    Error(String),
    RemovedLastFile,
}

pub struct IPCServer {
    listener: Listener,
}

impl IPCServer {
    pub fn new() -> Result<IPCServer> {
        let name = SOCKET_NAME.to_ns_name::<GenericNamespaced>()?;

        // I have decided to use try_overwrite here because it's the cli part that's going to verify
        // that the daemon is running and a new one will only be started if the socket file is not in use
        // or the existing daemon is not responsive.
        let listener = match ListenerOptions::new()
            .name(name)
            .try_overwrite(true)
            .create_sync()
        {
            Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
                eprintln!(
                    "Error: could not start server because the socket file is \
           occupied. Please check if {SOCKET_NAME} is in use by another \
           process and try again."
                );
                return Err(e.into());
            }
            x => x?,
        };

        eprintln!("Server running at {SOCKET_NAME}");
        Ok(Self { listener })
    }

    pub fn start(&mut self, mut handler: impl FnMut(&IPCRequest) -> IPCResponse) -> Result<()> {
        // This buffer will be reused between clients.
        let mut buffer = String::with_capacity(512);

        for mut conn in self
            .listener
            .incoming()
            .filter_map(|conn| {
                conn.map_err(|e| eprintln!("Incoming connection failed: {e}"))
                    .ok()
            })
            .map(BufReader::new)
        {
            conn.read_line(&mut buffer)?;
            print!("Got request: {buffer}");

            let request = serde_json::from_str::<IPCRequest>(&buffer)?;

            let response = handler(&request);

            println!("Handler response: {response:?}");

            conn.get_mut()
                .write_all((serde_json::to_string(&response)? + "\n").as_bytes())?;

            // Avoid holding up resources.
            drop(conn);

            // Clear the buffer so that the next iteration will display new data
            // instead of messages stacking on top of one another.
            buffer.clear();

            if matches!(response, IPCResponse::RemovedLastFile) {
                println!("Removed last file");
                break;
            }
        }
        println!("Shutting down IPC server");
        Ok(())
    }
}

pub struct IPCClient {}

impl IPCClient {
    pub fn check_connection() -> Result<()> {
        let response = Self::request(IPCRequest::Ping)?;
        if let IPCResponse::Ok = response {
            Ok(())
        } else {
            Err(anyhow!("Invalid IPC response: {response:?}"))
        }
    }
    pub fn request(request: IPCRequest) -> Result<IPCResponse> {
        let name = SOCKET_NAME.to_ns_name::<GenericNamespaced>()?;
        let mut stream = LocalSocketStream::connect(name)?;
        let payload = serde_json::to_string(&request)? + "\n";
        stream.write_all(payload.as_bytes())?;
        let mut buffer = String::new();
        stream.read_to_string(&mut buffer)?;
        serde_json::from_str::<IPCResponse>(&buffer)
            .map_err(|e| anyhow!("Failed to parse IPC response: {e}"))
    }
}
