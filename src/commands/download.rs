use crate::commands::explore_files;
use crate::communication::tcp::{TCPClient, TCPRequest, TCPResponse};
use crate::config::Config;
use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub async fn download(
    config: &Config<'_>,
    key: &String,
    output: Option<&String>,
    print: bool,
) -> Result<()> {
    let shared_files = explore_files(config).await?;

    let Some((_, addr, (file_name, num_chunks))) = shared_files.get(key) else {
        println!("No shared file found for {}", key);
        return Ok(());
    };

    let destination = if print {
        None
    } else {
        match output {
            None => {
                println!("Downloading file with key: {key}");
                Some(Path::new(
                    Path::new(file_name).file_name().expect("No filename"),
                ))
            }
            Some(path) => {
                println!("Downloading file with key: {key} to {path}");
                Some(Path::new(path))
            }
        }
    };

    let mut file = match destination {
        Some(path) => Some(File::create_new(path).await?),
        None => None,
    };

    let mut client = TCPClient::new(*addr, config.port).await?;

    for chunk_index in 0usize..*num_chunks {
        let response = client
            .send_request(TCPRequest::Download(key.clone(), chunk_index))
            .await?;
        if let TCPResponse::FileChunk(chunk) = response {
            match &mut file {
                None => {
                    let contents = String::from_utf8(chunk)?;
                    print!("{}", contents);
                }
                Some(file) => {
                    file.write_all(chunk.as_slice()).await?;
                }
            };
        } else {
            return Err(anyhow!("File transfer failed at chunk {}", chunk_index));
        }
    }

    Ok(())
}
