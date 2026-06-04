use std::path::PathBuf;

use directories::ProjectDirs;

pub struct Config<'a> {
    pub port: u16,
    pub code_words: &'a [&'a str],
}

impl<'a> Config<'a> {
    pub fn default() -> Self {
        Self {
            port: 9057,
            code_words: &[
                "apple", "bounty", "cheese", "drum", "eerie", "fancy", "gloom", "honey", "island",
                "jungle",
            ],
        }
    }

    pub fn load() -> Self {
        let path = config_path();
        let pathstr = path.to_str().unwrap_or("unknown");
        if path.exists() {
            println!("Config file found at: {pathstr}");
            // In the future, this will read the config file and populate the fields accordingly
            Self::default()
        } else {
            println!("Config file not found at: {pathstr}");
            Self::default()
        }
    }
}

fn config_path() -> PathBuf {
    let config_dir = ProjectDirs::from("com", "mnxoid", "localpost")
        .map_or(PathBuf::from("."), |project_dirs| {
            project_dirs.config_dir().to_path_buf()
        });
    config_dir.join("config.json")
}
