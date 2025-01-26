use crate::structs::PrintPrefix;
use colored::Colorize;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::pin::Pin;
use std::future::Future;

pub const GH_OWNER: &str = "mxve";
pub const GH_REPO: &str = "alterware-launcher";
pub const GH_IW4X_OWNER: &str = "iw4x";
pub const GH_IW4X_REPO: &str = "iw4x-client";
pub const DEFAULT_MASTER: &str = "https://cdn.alterware.ovh";
pub const BACKUP_MASTER: &str = "https://cdn.iw4x.getserve.rs";

pub static MASTER_URL: Lazy<Mutex<String>> = Lazy::new(|| {
    Mutex::new(String::from(DEFAULT_MASTER))
});

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

pub fn check_connectivity(master_url: Option<String>) -> Pin<Box<dyn Future<Output = bool> + Send>> {
    Box::pin(async move {
        let retry = master_url.is_some();
        if !retry {
            *MASTER_URL.lock().unwrap() = String::from(DEFAULT_MASTER);
            println!("Running connectivity check on default CDN");
        } else {
            *MASTER_URL.lock().unwrap() = String::from(BACKUP_MASTER);
            println!("Running connectivity check on backup CDN");
        }

        let master_url = MASTER_URL.lock().unwrap().clone();

        let result = match crate::http_async::get_body_string(&master_url).await {
            Ok(_) => true,
            Err(_) => {
                *IS_OFFLINE.lock().unwrap() = true;
                false
            }
        };

        if !result {
            println!("Failed to connect to CDN {}", master_url);
        }

        if !retry && !result {
            check_connectivity(Some(String::from(BACKUP_MASTER))).await
        } else {
            result
        }
    })
}
