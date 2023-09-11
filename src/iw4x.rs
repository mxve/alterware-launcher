use crate::github;
use crate::global::*;
use crate::http;
use crate::misc;

use std::{fs, path::Path};
use colored::*;

pub fn local_revision(dir: &Path) -> u16 {
    if let Ok(revision) = fs::read_to_string(dir.join(".iw4xrevision")) {
        misc::rev_to_int(&revision)
    } else {
        0
    }
}

pub fn remote_revision() -> u16 {
    misc::rev_to_int(&github::latest_tag(GH_IW4X_OWNER, GH_IW4X_REPO))
}

pub fn update(dir: &Path) {
    let remote = remote_revision();
    let local = local_revision(dir);

    if remote <= local && dir.join("iw4x.dll").exists() {
        return;
    }

    println!("[{}] {}", "Downloading".bright_yellow(), dir.join("iw4x.dll").display());
    http::download_file(
        &format!(
            "{}/download/iw4x.dll",
            github::latest_release_url(GH_IW4X_OWNER, GH_IW4X_REPO)
        ),
        &dir.join("iw4x.dll"),
    );
    fs::write(dir.join(".iw4xrevision"), format!("r{}", remote)).unwrap();
}
