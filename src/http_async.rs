use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use colored::*;
use futures_util::StreamExt;
use indicatif::ProgressBar;
use reqwest::Client;

use crate::misc;

pub async fn download_file(
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
    pb.set_message(format!("{}", path.file_name().unwrap().to_str().unwrap()));

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
