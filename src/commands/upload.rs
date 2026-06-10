use crate::commands::explore_files;
use crate::communication::ipc::{IPCClient, IPCRequest, IPCResponse};
use crate::config::Config;
use anyhow::Result;
use rand::RngExt;
use std::collections::{BTreeMap, BTreeSet};
use std::path;
use std::process::{Command, Stdio};

pub async fn upload(config: &Config<'_>, file: &str) -> Result<()> {
    let file_path = path::Path::new(file);
    if !file_path.exists() {
        println!("File not found: {file}");
        return Err(anyhow::anyhow!("File not found"));
    }
    let absolute_path = path::absolute(file_path)?;
    let absolute_path_str = absolute_path.to_str().expect("file is not valid UTF-8");

    // Check for local file collision
    let local_files = match IPCClient::request(IPCRequest::ListFiles)
        .unwrap_or(IPCResponse::Files(BTreeMap::new()))
    {
        IPCResponse::Files(local_files) => local_files.values().cloned().collect(),
        _ => BTreeSet::new(),
    };
    if local_files.contains(absolute_path_str) {
        println!("File is already being served");
        return Ok(());
    }

    let shared_files = explore_files(config).await?;
    let used_keys = shared_files.keys().collect::<BTreeSet<_>>();
    let mut key = generate_key(config);
    // Regenerate if needed
    while used_keys.contains(&key) {
        key = generate_key(config);
    }

    if let Err(_) = IPCClient::check_connection() {
        println!("Starting daemon...");
        let current_binary_path = std::env::current_exe()
            .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {}", e))?;
        let _ = Command::new(current_binary_path)
            .arg("daemon")
            .arg(absolute_path)
            .arg(key.clone())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to start daemon: {}", e))?;
        println!("Uploading file: {file} with key: {key}");
        return Ok(());
    };

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
