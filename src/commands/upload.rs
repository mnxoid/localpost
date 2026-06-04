use crate::config::Config;
use rand::RngExt;

pub fn upload(config: &Config, file: &str) {
    // TODO: Implement collision handling
    let key = generate_key(config);

    let file_path = std::path::Path::new(file);
    if !file_path.exists() {
        println!("File not found: {file}");
        return;
    }

    // Check if the daemon is running and if the file is already being served
    // If not, start serving the file and print the key

    println!("Uploading file: {file} with key: {key}");
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
