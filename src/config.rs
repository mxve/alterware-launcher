use crate::structs::Config;

use std::{fs, path::PathBuf};

pub fn load(config_path: PathBuf) -> Config {
    if config_path.exists() {
        let cfg = fs::read_to_string(&config_path).unwrap();
        let cfg: Config = serde_json::from_str(&cfg).unwrap_or(Config::default());
        return cfg;
    }
    save(config_path.clone(), Config::default());
    Config::default()
}

pub fn save(config_path: PathBuf, config: Config) {
    fs::write(config_path, serde_json::to_string(&config).unwrap()).unwrap();
}
