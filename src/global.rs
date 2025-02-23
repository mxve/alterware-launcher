use crate::http_async;
use crate::structs::PrintPrefix;
use colored::Colorize;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

pub const GH_OWNER: &str = "mxve";
pub const GH_REPO: &str = "alterware-launcher";
pub const GH_IW4X_OWNER: &str = "iw4x";
pub const GH_IW4X_REPO: &str = "iw4x-client";
pub const DEFAULT_MASTER: &str = "https://cdn.alterware.ovh";
pub const BACKUP_MASTER: &str = "https://cdn.iw4x.getserve.rs";

pub static MASTER_URL: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::from(DEFAULT_MASTER)));

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

pub fn check_connectivity(
    master_url: Option<String>,
) -> Pin<Box<dyn Future<Output = bool> + Send>> {
    Box::pin(async move {
        let retry = master_url.is_some();
        if !retry {
            crate::println_info!("Running connectivity check on {}", DEFAULT_MASTER);
        } else {
            let master = master_url.unwrap();
            *MASTER_URL.lock().unwrap() = master.clone();
            crate::println_info!("Running connectivity check on {}", master);
        }

        let master_url = MASTER_URL.lock().unwrap().clone();

        // Check ASN number using the new get_json function
        let asn_response: Result<Value, String> =
            http_async::get_json("https://ip2asn.getserve.rs/v1/as/ip/self").await;

        let mut switched_to_backup = false;

        if let Ok(asn_data) = asn_response {
            if let Some(as_number) = asn_data.get("as_number").and_then(|v| v.as_i64()) {
                if as_number == 3320 && master_url == DEFAULT_MASTER {
                    *MASTER_URL.lock().unwrap() = String::from(BACKUP_MASTER);
                    crate::println_info!(
                        "Detected DTAG as ISP, switched to backup master URL: {}",
                        BACKUP_MASTER
                    );
                    switched_to_backup = true;
                }
            }
        }

        // Run connectivity check regardless of ASN switch
        let result = match crate::http_async::get_body_string(&master_url).await {
            Ok(_) => true,
            Err(_) => {
                *IS_OFFLINE.lock().unwrap() = true;
                false
            }
        };

        if !result {
            crate::println_error!("Failed to connect to CDN {}", master_url);
        }

        // If we switched to backup, do not retry
        if !retry && !result && !switched_to_backup {
            check_connectivity(Some(String::from(BACKUP_MASTER))).await
        } else {
            result
        }
    })
}
