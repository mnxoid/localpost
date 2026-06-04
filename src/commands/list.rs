use crate::constants::SOCKET_NAME;
pub use anyhow::Result;
use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{GenericNamespaced, ToNsName};
use std::io::{Read, Write};

pub fn list() -> Result<()> {
    println!("Listing currently served files");
    let name = SOCKET_NAME.to_ns_name::<GenericNamespaced>()?;
    let mut stream = match LocalSocketStream::connect(name) {
        Ok(stream) => stream,
        Err(e) => {
            println!("No server currently running");
            return Ok(());
        }
    };
    println!("Connected to server");
    let mut buffer = String::new();
    stream.write_all(b"Hello from client!\n")?;
    stream
        .read_to_string(&mut buffer)
        .expect("Should have something to read");
    println!("{}", buffer);
    Ok(())
}
