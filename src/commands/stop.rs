pub fn stop(key: Option<&String>) {
    if let Some(key) = key {
        println!("Stopping serving file with key: {key}");
    } else {
        println!("Stopping serving all files");
    }
}
