use anyhow::Result;
use serde::{Deserialize, Serialize};
use {
    interprocess::local_socket::{GenericNamespaced, ListenerOptions, prelude::*},
    std::io::{self, BufReader, prelude::*},
};

#[derive(Serialize, Deserialize, Debug)]
pub enum IPCRequest {
    AddFile { key: String, path: String },
    RemoveFile { key: String },
    ListFiles,
    GetFile { key: String },
    Ping,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IPCResponse {
    Ok,
    File(String),
    Files(Vec<(String, String)>), // (key, path)
    Pong,
    Error(String),
}

const SOCKET_NAME: &str = "localpost.sock";

pub fn daemon(file: &String, key: &String) -> Result<()> {
    println!("Starting daemon to serve file: {file} with key: {key}");

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

    // This is a good place to inform clients that the server is ready.
    eprintln!("Server running at {SOCKET_NAME}");

    // This buffer will be reused between clients.
    let mut buffer = String::with_capacity(512);

    for mut conn in listener
        .incoming()
        .filter_map(|conn| {
            conn.map_err(|e| eprintln!("Incoming connection failed: {e}"))
                .ok()
        })
        .map(BufReader::new)
    {
        // Since our client example sends first, the server should receive a
        // line and only then send a response. Otherwise, because receiving
        // from and sending to a connection cannot be simultaneous without
        // threads or async, we can deadlock the two processes by having both
        // sides wait for the send buffer to be emptied by the other.
        conn.read_line(&mut buffer)?;

        // Now that the receive has come through and the client is waiting
        // on the server's send, do it. (`.get_mut()` is to get the sender,
        // `BufReader` doesn't implement a pass-through `Write`.)
        conn.get_mut().write_all(b"Hello from server!\n")?;

        // Avoid holding up resources.
        drop(conn);

        // read_line keeps the line feed at the end.
        print!("Client answered: {buffer}");

        // Clear the buffer so that the next iteration will display new data
        // instead of messages stacking on top of one another.
        buffer.clear();
    }

    Ok(())
}
