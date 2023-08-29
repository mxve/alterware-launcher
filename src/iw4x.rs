use crate::github;
use crate::http;
use crate::misc;
use crate::global::*;

use std::{fs, path::Path};

pub fn local_revision(dir: &Path) -> u16 {
    if let Ok(revision) = fs::read_to_string(dir.join(".iw4xrevision")) {
        misc::rev_to_int(&revision)
    } else {
        0
    }
}

pub fn remote_revision() -> u16 {
    misc::rev_to_int(&github::latest(GH_IW4X_OWNER, GH_IW4X_REPO))
}

pub fn update_available(dir: &Path) -> bool {
    local_revision(dir) < remote_revision()
}

pub fn update(dir: &Path) {
    if update_available(dir) {
        println!("Updating IW4x...");
        http::download_file(
            &format!(
                "{}/download/iw4x.dll",
                github::latest_release_url(GH_IW4X_OWNER, GH_IW4X_REPO)
            ),
            &dir.join("iw4x.dll"),
        );
        fs::write(
            dir.join(".iw4xrevision"),
            github::latest(GH_IW4X_OWNER, GH_IW4X_REPO),
        )
        .unwrap();
    }
}
