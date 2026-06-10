use crate::communication::ipc::{IPCClient, IPCRequest, IPCResponse};
use crate::config::Config;
use anyhow::Result;
use rand::RngExt;
use std::path;
use std::process::{Command, Stdio};

pub fn upload(config: &Config, file: &str) -> Result<()> {
    // TODO: Implement collision handling
    let mut key = generate_key(config);

    let file_path = path::Path::new(file);
    if !file_path.exists() {
        println!("File not found: {file}");
        return Err(anyhow::anyhow!("File not found"));
    }

    if let Err(_) = IPCClient::check_connection() {
        println!("Starting daemon...");
        let current_binary_path = std::env::current_exe()
            .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {}", e))?;
        let _ = Command::new(current_binary_path)
            .arg("daemon")
            .arg(path::absolute(file_path)?)
            .arg(key.clone())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start daemon: {}", e))?;
        println!("Uploading file: {file} with key: {key}");
        return Ok(());
    };

    // Check if the daemon is running and if the file is already being served
    // If not, start serving the file and print the key
    let response = IPCClient::request(IPCRequest::ListFiles)?;
    match response {
        IPCResponse::Files(files) => {
            while files.contains_key(&key) {
                key = generate_key(config);
            }
        }
        _ => return Err(anyhow::anyhow!("Invalid IPC response")),
    }

    println!("Uploading file: {file} with key: {key}");
    let response = IPCClient::request(IPCRequest::AddFile {
        key,
        path: path::absolute(file_path)?.to_string_lossy().to_string(),
    })?;
    match response {
        IPCResponse::Ok => {
            println!("File uploaded successfully");
        }
        _ => {
            return Err(anyhow::anyhow!("Error uploading file: {:?}", response));
        }
    }
    Ok(())
}

pub fn generate_key(config: &Config) -> String {
    (0..3)
        .map(|_| {
            let word = &config.code_words[rand::rng().random_range(0..config.code_words.len())];
            word.to_string()
        })
        .collect::<Vec<String>>()
        .join("-")
}
