use async_recursion::async_recursion;
use colored::*;
use futures_util::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;

use crate::global;
use crate::misc;
use std::{cmp::min, error::Error, fs::File, io::Write, path::PathBuf, str};

#[async_recursion]
async fn get_body(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let client = Client::new();

    let response = client
        .get(url)
        .header(
            "User-Agent",
            format!(
                "AlterWare Launcher | github.com/{}/{}",
                global::GH_OWNER,
                global::GH_REPO
            ),
        )
        .send()
        .await?;

    if response.status().is_redirection() {
        if let Some(location) = response.headers().get("Location") {
            if let Ok(location) = location.to_str() {
                return get_body(location).await;
            }
        }
    }

    if response.status() != reqwest::StatusCode::OK {
        return Err(format!(
            "Could not get body from {}, got {}",
            url,
            response.status()
        ))?;
    }

    let bytes = response.bytes().await?;
    Ok(bytes.to_vec())
}

pub async fn get_body_string(url: &str) -> Result<String, Box<dyn Error>> {
    let body_bytes = get_body(url).await?;
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    Ok(body_str)
}

pub async fn download_file_shared_client(
    client: &Client,
    pb: &ProgressBar,
    url: &str,
    path: &PathBuf,
    size: u64,
) -> Result<(), String> {
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    // Fix for CF shenanigans
    let total_size = res.content_length().unwrap_or(size);
    pb.set_length(total_size);
    pb.println(format!(
        "[{}] {} ({})",
        "Downloading".bright_yellow(),
        misc::cute_path(path),
        misc::human_readable_bytes(total_size)
    ));
    pb.set_message(path.file_name().unwrap().to_str().unwrap().to_string());

    let mut file =
        File::create(path).or(Err(format!("Failed to create file '{}'", path.display())))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err("Error while downloading file"))?;
        file.write_all(&chunk)
            .or(Err("Error while writing to file"))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.set_message(String::default());
    Ok(())
}

pub async fn download_file(url: &str, path: &PathBuf) -> Result<(), String> {
    let client = Client::new();
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))
        .unwrap();
    let mut file = File::create(path)
        .or(Err(format!("Failed to create file '{}'", path.display())))
        .unwrap();
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err("Error while downloading file"))?;
        file.write_all(&chunk)
            .or(Err("Error while writing to file"))?;
    }

    Ok(())
}
