use crate::extend::*;
use crate::github;
use crate::global::*;
use crate::http_async;
use crate::misc;
use crate::structs;

use std::path::Path;

pub async fn remote_revision() -> u16 {
    match github::latest_tag(GH_IW4X_OWNER, GH_IW4X_REPO).await {
        Ok(tag) => misc::rev_to_int(&tag),
        Err(_) => {
            crate::println_error!("Failed to get latest version for {GH_IW4X_OWNER}/{GH_IW4X_REPO}, assuming we are up to date.");
            0
        }
    }
}

pub async fn update(dir: &Path, cache: &mut structs::Cache) {
    let remote = remote_revision().await;
    let local = misc::rev_to_int(&cache.iw4x_revision);

    if remote <= local && dir.join("iw4x.dll").exists() {
        crate::println_info!("No files to download for IW4x");
        return;
    }

    crate::println_info!("Downloading outdated or missing files for IW4x",);
    println!(
        "{}{}",
        misc::prefix("downloading"),
        dir.join("iw4x.dll").cute_path()
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

    cache.iw4x_revision = format!("r{remote}");
}
