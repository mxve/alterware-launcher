#![allow(unused)]

use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::Mutex;

/// The owner of the launcher repository
pub const GITHUB_OWNER: &str = "mxve";
/// The repository of the launcher
pub const GITHUB_REPO: &str = "alterware-launcher";

/// CDN hosts to iterate over
pub const CDN_HOSTS: [&str; 2] = ["test.test", "cdn.getserve.rs"];

/// CDN url procotol (http/https)
pub static CDN_PROTOCOL: OnceCell<Mutex<String>> = OnceCell::new();

/// CDN branch (sub dir)
pub static CDN_BRANCH: OnceCell<Mutex<String>> = OnceCell::new();

/// Chosen CDN host
pub static CDN_HOST: OnceCell<Mutex<String>> = OnceCell::new();

/// Game
pub static GAME: OnceCell<Mutex<crate::game::Game>> = OnceCell::new();

/// Game directory
pub static GAME_DIR: OnceCell<Mutex<PathBuf>> = OnceCell::new();

/// Game client
pub static GAME_CLIENT: OnceCell<Mutex<Option<crate::game::Client>>> = OnceCell::new();

/// The path to the download cache
pub static CACHE_DIR: OnceCell<Mutex<PathBuf>> = OnceCell::new();

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
