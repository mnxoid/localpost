pub fn download(key: &String, output: Option<&String>) {
    if let Some(output) = output {
        println!("Downloading file with key: {key} to {output}");
    } else {
        println!("Downloading file with key: {key}");
    }
}
