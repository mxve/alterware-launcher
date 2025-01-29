#![allow(unused)]

use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;

/// The owner of the launcher repository
pub const GITHUB_OWNER: &str = "mxve";
/// The repository of the launcher
pub const GITHUB_REPO: &str = "alterware-launcher";


// TODO: Make this configurable
/// Base URL for file downloads
pub const CDN_URL: &str = "https://cdn.getserve.rs/stable";

// TODO: Make this configurable
/// The path to the download cache
pub const CACHE_DIR: &str = "awtmp";

/// User-agent for HTTP requests
pub static USER_AGENT: Lazy<String> = Lazy::new(|| {
    format!(
        "AlterWare Launcher v{} on {} | github.com/{}/{}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        GITHUB_OWNER,
        GITHUB_REPO
    )
});
