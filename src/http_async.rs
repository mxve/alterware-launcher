use serde::de::DeserializeOwned;
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use colored::*;
use futures_util::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;

use crate::misc;

async fn make_request(client: &Client, url: &str) -> Result<reqwest::Response, reqwest::Error> {
    client
        .get(url)
        .header(
            "User-Agent",
            &format!(
                "AlterWare Launcher | github.com/{}/{}",
                crate::global::GH_OWNER,
                crate::global::GH_REPO
            ),
        )
        .send()
        .await
}

pub async fn download_file_progress(
    client: &Client,
    pb: &ProgressBar,
    url: &str,
    path: &PathBuf,
    size: u64,
) -> Result<(), String> {
    let res = make_request(client, url)
        .await
        .or(Err(format!("Failed to GET from '{}'", url)))?;
    // Fix for CF shenanigans
    let total_size = res.content_length().unwrap_or(size);
    pb.set_length(total_size);
    let msg = format!(
        "[{}] {} ({})",
        "Downloading".bright_yellow(),
        misc::cute_path(path),
        misc::human_readable_bytes(total_size)
    );
    pb.println(&msg);
    info!("{}", msg);
    pb.set_message(path.file_name().unwrap().to_str().unwrap().to_string());

    let mut file =
        File::create(path).or(Err(format!("Failed to create file '{}'", path.display())))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Error while downloading file: {e}"))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Error while writing to file: {e}"))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new); // This should be done in a separate thread if it's a spinner
    }

    pb.set_message(String::default());
    Ok(())
}

pub async fn download_file(url: &str, path: &PathBuf) -> Result<(), String> {
    let client = Client::new();
    match make_request(&client, url).await {
        Ok(res) => {
            let body = res.bytes().await.or(Err("Failed to download file"))?;
            let mut file = File::create(path).or(Err("Failed to create file"))?;
            file.write_all(&body).or(Err("Failed to write to file"))?;
            Ok(())
        }
        Err(e) => {
            misc::fatal_error(&e.to_string());
            Err("Could not download file".to_string())
        }
    }
}

pub async fn get_body(url: &str) -> Result<Vec<u8>, String> {
    let client = Client::new();
    match make_request(&client, url).await {
        Ok(res) => {
            debug!("{} {}", res.status().to_string(), url);
            let body = res.bytes().await.or(Err("Failed to get body"))?;
            Ok(body.to_vec())
        }
        Err(e) => {
            misc::fatal_error(&e.to_string());
            Err("Could not get body".to_string())
        }
    }
}

pub async fn get_body_string(url: &str) -> Result<String, String> {
    let body = get_body(url).await?;
    Ok(String::from_utf8(body).unwrap())
}

pub async fn get_body_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    // basically analogous to Reqwest::Response::json()
    serde_json::from_slice::<T>(&get_body(url).await?).map_err(|err| err.to_string())
}
