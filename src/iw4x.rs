use crate::github;
use crate::global::*;
use crate::http_async;
use crate::misc;

use colored::*;
use std::{fs, path::Path};

pub fn local_revision(dir: &Path) -> u16 {
    if let Ok(revision) = fs::read_to_string(dir.join(".iw4xrevision")) {
        misc::rev_to_int(&revision)
    } else {
        0
    }
}

pub async fn remote_revision() -> u16 {
    misc::rev_to_int(&github::latest_tag(GH_IW4X_OWNER, GH_IW4X_REPO).await)
}

pub async fn update(dir: &Path) {
    let remote = remote_revision().await;
    let local = local_revision(dir);

    if remote <= local && dir.join("iw4x.dll").exists() {
        crate::println_info!(
            "[{}]        No files to download for IW4x",
            "Info".bright_magenta(),
        );
        return;
    }

    crate::println_info!(
        "[{}]        Downloading outdated or missing files for IW4x",
        "Info".bright_magenta()
    );
    crate::println_info!(
        "[{}] {}",
        "Downloading".bright_yellow(),
        misc::cute_path(&dir.join("iw4x.dll"))
    );
    http_async::download_file(
        &format!(
            "{}/download/iw4x.dll",
            github::latest_release_url(GH_IW4X_OWNER, GH_IW4X_REPO)
        ),
        &dir.join("iw4x.dll"),
    )
    .await
    .unwrap();
    fs::write(dir.join(".iw4xrevision"), format!("r{}", remote)).unwrap();
}
