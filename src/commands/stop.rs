use anyhow::Result;
pub fn stop(key: Option<&String>) -> Result<()> {
    if let Some(key) = key {
        println!("Stopping serving file with key: {key}");
    } else {
        println!("Stopping serving all files");
    }
    Ok(())
}
