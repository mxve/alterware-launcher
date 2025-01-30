use std::path::PathBuf;

use crate::{extend::*, global, http, utils};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct File {
    pub name: String,
    pub size: u64,
    pub hash: String,
}

impl File {
    /// CDN URL for the file
    pub fn url(&self) -> String {
        format!("{}/{}", get_cdn_url(), self.hash)
    }

    /// Temporary file name on disk, truncated to 24 characters to prevent MAX_PATH issues
    pub fn cache_name(&self) -> String {
        self.hash[..24].to_string()
    }

    /// Temporary (full) file path for downloading
    pub fn cache_path(&self) -> PathBuf {
        format!(
            "{}/{}",
            utils::get_mutex(&global::CACHE_DIR).display(),
            self.cache_name()
        )
        .into()
    }

    /// Human-readable file size
    pub fn size_human(&self) -> String {
        self.size.human_readable_size()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Info {
    pub delete_list: Vec<String>,
    pub files: Vec<File>,
}

pub fn get_cdn_url() -> String {
    format!(
        "{}://{}/{}",
        utils::get_mutex(&global::CDN_PROTOCOL),
        utils::get_mutex(&global::CDN_HOST),
        utils::get_mutex(&global::CDN_BRANCH)
    )
}

/// Try each CDN host and set the first working one
async fn find_working_host() -> Result<(), Box<dyn std::error::Error>> {
    for host in global::CDN_HOSTS {
        println!("Checking {}", host);
        utils::set_mutex(&global::CDN_HOST, host.to_owned());

        let url = format!("{}/info.json", get_cdn_url());
        match http::quick_request(&url).await {
            Ok(response) => match serde_json::from_str::<Info>(&response) {
                Ok(_) => {
                    println!("Successfully connected to {}", host);
                    return Ok(());
                }
                Err(e) => {
                    println!("Invalid JSON from {}: {}", host, e);
                    continue;
                }
            },
            Err(e) => {
                println!("Failed to get info from {}: {}", host, e);
                continue;
            }
        }
    }
    Err("No CDN host is reachable or returned valid info".into())
}

/// Get info from CDN
pub async fn get_info() -> Result<Info, Box<dyn std::error::Error>> {
    async fn get_info_inner() -> Result<Info, Box<dyn std::error::Error>> {
        if let Some(host) = utils::get_mutex_opt(&global::CDN_HOST) {
            let url = format!("{}/info.json", get_cdn_url());
            match http::quick_request(&url).await {
                Ok(response) => match serde_json::from_str::<Info>(&response) {
                    Ok(info) => return Ok(info),
                    Err(e) => println!("Invalid JSON from {}: {}", host, e),
                },
                Err(e) => println!("Failed to get info from {}: {}", host, e),
            }
        }

        find_working_host().await?;
        Box::pin(get_info()).await
    }

    get_info_inner().await
}

/// Filter files by game
pub fn filter_files(files: Vec<File>, game: crate::game::Game) -> Vec<File> {
    files
        .into_iter()
        .filter(|file| {
            game.cdn_base_dirs()
                .contains(&file.name.split('/').next().unwrap())
        })
        .collect()
}
