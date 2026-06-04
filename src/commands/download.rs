use anyhow::Result;
pub fn download(key: &String, output: Option<&String>) -> Result<()> {
    if let Some(output) = output {
        println!("Downloading file with key: {key} to {output}");
    } else {
        println!("Downloading file with key: {key}");
    }
    Ok(())
}
