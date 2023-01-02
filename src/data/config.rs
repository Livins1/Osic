use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;


const CONFIG_PATH: &str = "./Osic.toml";

#[derive(Serialize, Default, Deserialize)]
pub struct AppConfig {
    wp_dirs: Vec<String>,
}

impl AppConfig {
    pub fn add_wp_dirs(&mut self, dir: String) {
        self.wp_dirs.push(dir);
        self.wp_dirs.dedup();
    }

    pub fn get_wp_dirs(&self) -> &Vec<String> {
        return &self.wp_dirs;
    }

    pub fn load_from_file() -> AppConfig {
        if Path::new(CONFIG_PATH).exists() {
            println!("Config file exists, read config.");
            // let f = File::open(CONFIG_PATH).unwrap();
            let mut buf = String::new();
            if let Ok(mut f) = File::open(CONFIG_PATH) {
                if let Ok(_) = f.read_to_string(&mut buf) {
                    match toml::from_str::<AppConfig>(&buf) {
                        Ok(config) => return config,
                        Err(_) => {
                            println!("Config file read error.")
                        }
                    }
                }
            }
        }
        AppConfig::default()
    }

    // write configs to a toml file
    pub fn save_to_toml(&self) {
        let toml = toml::to_string(&self).unwrap();
        let mut f = File::create(CONFIG_PATH).unwrap();
        if let Ok(_) = f.write_all(toml.as_bytes()) {
            println!("Config write successfully.");
        }
    }
}
