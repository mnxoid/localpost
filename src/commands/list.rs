pub use anyhow::Result;
pub fn list() -> Result<()> {
    println!("Listing currently served files");
    Ok(())
}
