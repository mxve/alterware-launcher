use crate::github;
use crate::global::*;

use semver::Version;
#[cfg(not(windows))]
use std::{thread, time};

pub fn self_update_available() -> bool {
    let current_version: Version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let latest_version = github::latest_version(GH_OWNER, GH_REPO);

    current_version < latest_version
}

#[cfg(not(windows))]
pub fn run(_update_only: bool) {
    if self_update_available() {
        println!("A new version of the AlterWare launcher is available.");
        println!("Download it at {}", github::latest_release_url());
        println!("Launching in 10 seconds..");
        thread::sleep(time::Duration::from_secs(10));
    }
}

#[cfg(windows)]
pub fn run(update_only: bool) {
    use std::{fs, path::PathBuf};

    use crate::http;

    let working_dir = std::env::current_dir().unwrap();
    let files = fs::read_dir(&working_dir).unwrap();

    for file in files {
        let file = file.unwrap();
        let file_name = file.file_name().into_string().unwrap();

        if file_name.contains("alterware-launcher")
            && (file_name.contains(".__relocated__.exe")
                || file_name.contains(".__selfdelete__.exe"))
        {
            fs::remove_file(file.path()).unwrap();
        }
    }

    if self_update_available() {
        println!("Performing launcher self-update.");
        println!(
            "If you run into any issues, please download the latest version at {}",
            github::latest_release_url(GH_OWNER, GH_REPO)
        );

        let update_binary = PathBuf::from("alterware-launcher-update.exe");
        let file_path = working_dir.join(&update_binary);

        if update_binary.exists() {
            fs::remove_file(&update_binary).unwrap();
        }

        http::download_file(
            &format!(
                "{}/download/alterware-launcher.exe",
                github::latest_release_url(GH_OWNER, GH_REPO)
            ),
            &file_path,
        );

        if !file_path.exists() {
            println!("Failed to download launcher update.");
            return;
        }

        self_replace::self_replace("alterware-launcher-update.exe").unwrap();
        fs::remove_file(&file_path).unwrap();
        println!("Launcher updated. Please run it again.");
        if !update_only {
            std::io::stdin().read_line(&mut String::new()).unwrap();
        }
        std::process::exit(201);
    }
}
