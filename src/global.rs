use crate::misc;
use crate::structs::{PrintPrefix, StoredGameData};
use colored::Colorize;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub const GH_OWNER: &str = "mxve";
pub const GH_REPO: &str = "alterware-launcher";
pub const GH_IW4X_OWNER: &str = "iw4x";
pub const GH_IW4X_REPO: &str = "iw4x-client";

pub static MASTER: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("https://cdn.alterware.ovh".to_owned()));

pub static IS_OFFLINE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

pub static PREFIXES: Lazy<HashMap<&'static str, PrintPrefix>> = Lazy::new(|| {
    HashMap::from([
        (
            "info",
            PrintPrefix {
                text: "Info".bright_magenta(),
                padding: 8,
            },
        ),
        (
            "downloading",
            PrintPrefix {
                text: "Downloading".bright_yellow(),
                padding: 1,
            },
        ),
        (
            "checked",
            PrintPrefix {
                text: "Checked".bright_blue(),
                padding: 5,
            },
        ),
        (
            "removed",
            PrintPrefix {
                text: "Removed".bright_red(),
                padding: 5,
            },
        ),
        (
            "error",
            PrintPrefix {
                text: "Error".red(),
                padding: 7,
            },
        ),
    ])
});

pub async fn check_connectivity() -> bool {
    let master_url = MASTER.lock().unwrap().clone();

    match crate::http_async::get_body_string(&master_url).await {
        Ok(_) => true,
        Err(_) => {
            *IS_OFFLINE.lock().unwrap() = true;
            false
        }
    }
}

pub fn get_stored_data() -> Option<StoredGameData> {
    let dir = std::env::current_dir().ok()?;
    let cache = misc::get_cache(&dir);
    cache.stored_data
}

pub fn store_game_data(data: &StoredGameData) -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::current_dir()?;
    let mut cache = misc::get_cache(&dir);
    cache.stored_data = Some((*data).clone());
    misc::save_cache(&dir, cache);
    Ok(())
}
