use crate::commands::explore_files;
use crate::config::Config;
use anyhow::Result;

pub async fn download(config: &Config<'_>, key: &String, output: Option<&String>) -> Result<()> {
    let shared_files = explore_files(config).await?;

    let Some((session_id, addr, file)) = shared_files.get(key) else {
        println!("No shared file found for {}", key);
        return Ok(());
    };

    if let Some(output) = output {
        println!("Downloading file with key: {key} to {output}");
    } else {
        println!("Downloading file with key: {key}");
    }
    Ok(())
}
