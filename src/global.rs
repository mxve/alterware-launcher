use crate::structs::PrintPrefix;
use colored::Colorize;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

use crate::cdn::{Hosts, Region, Server};

pub const GH_OWNER: &str = "mxve";
pub const GH_REPO: &str = "alterware-launcher";
pub const GH_IW4X_OWNER: &str = "iw4x";
pub const GH_IW4X_REPO: &str = "iw4x-client";
pub const DEFAULT_MASTER: &str = "https://cdn.alterware.ovh";

pub const CDN_HOSTS: [Server; 3] = [
    Server::new("cdn.alterware.ovh", Region::Global),
    Server::new("us-cdn.alterware.ovh", Region::NorthAmerica),
    Server::new("cdn.iw4x.dev", Region::Europe),
];

pub const IP2ASN: &str = "https://ip2asn.getserve.rs/v1/as/ip/self";

pub static USER_AGENT: Lazy<String> = Lazy::new(|| {
    format!(
        "AlterWare Launcher v{} on {} | github.com/{}/{}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        GH_OWNER,
        GH_REPO
    )
});

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
        (
            "renamed",
            PrintPrefix {
                text: "Renamed".bright_blue(),
                padding: 5,
            },
        ),
    ])
});

pub async fn check_connectivity_and_rate_cdns() -> Pin<Box<dyn Future<Output = bool> + Send>> {
    Box::pin(async move {
        crate::println_info!("Initializing CDN rating system...");
        let hosts = Hosts::new().await;
        let best_cdn = hosts.get_master_url();

        if let Some(cdn_url) = best_cdn {
            let cdn_url = cdn_url.trim_end_matches('/');
            *MASTER_URL.lock().unwrap() = cdn_url.to_string();
            crate::println_info!("Selected CDN: {}", cdn_url);

            match crate::http_async::get_body_string(cdn_url).await {
                Ok(_) => {
                    info!("Successfully connected to CDN: {}", cdn_url);
                    true
                }
                Err(e) => {
                    error!("Failed to connect to selected CDN {}: {}", cdn_url, e);
                    *IS_OFFLINE.lock().unwrap() = true;
                    false
                }
            }
        } else {
            crate::println_error!("No CDN hosts are available");
            *IS_OFFLINE.lock().unwrap() = true;
            false
        }
    })
}

pub fn check_connectivity(
    master_url: Option<String>,
) -> Pin<Box<dyn Future<Output = bool> + Send>> {
    Box::pin(async move {
        if let Some(url) = master_url {
            *MASTER_URL.lock().unwrap() = url.clone();
            crate::println_info!("Using fallback connectivity check on {}", url);

            match crate::http_async::get_body_string(&url).await {
                Ok(_) => true,
                Err(_) => {
                    *IS_OFFLINE.lock().unwrap() = true;
                    false
                }
            }
        } else {
            check_connectivity_and_rate_cdns().await.await
        }
    })
}
