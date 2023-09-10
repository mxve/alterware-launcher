use crate::structs::Config;

use std::{fs, path::PathBuf};

const DEFAULT: Config = Config {
    update_only: false,
    skip_self_update: false,
    bonus_content: false,
};

pub fn load(config_path: PathBuf) -> Config {
    if config_path.exists() {
        let cfg = fs::read_to_string(&config_path).unwrap();
        let cfg: Config = serde_json::from_str(&cfg).unwrap_or(DEFAULT);
        return cfg;
    }
    DEFAULT
}

pub fn save(config_path: PathBuf, config: Config) {
    fs::write(config_path, serde_json::to_string(&config).unwrap()).unwrap();
}
