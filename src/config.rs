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
    match fs::write(
        config_path.clone(),
        serde_json::to_string_pretty(&config).unwrap(),
    ) {
        Ok(_) => (),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                fs::create_dir_all(config_path.parent().unwrap()).unwrap();
                save(config_path, config);
            }
            _ => crate::println_error!("Error while saving config {}", e.to_string()),
        },
    }
}

pub fn save_value(config_path: PathBuf, key: &str, value: bool) {
    let mut config = load(config_path.clone());
    match key {
        "update_only" => config.update_only = value,
        "skip_self_update" => config.skip_self_update = value,
        "download_bonus_content" => config.download_bonus_content = value,
        "ask_bonus_content" => config.ask_bonus_content = value,
        "force_update" => config.force_update = value,
        "use_https" => config.use_https = value,
        "skip_redist" => config.skip_redist = value,
        _ => (),
    }
    save(config_path, config);
}

pub fn save_value_s(config_path: PathBuf, key: &str, value: String) {
    let mut config = load(config_path.clone());
    match key {
        "args" => config.args = value.to_string(),
        "engine" => config.engine = value.to_string(),
        _ => (),
    }
    save(config_path, config);
}
