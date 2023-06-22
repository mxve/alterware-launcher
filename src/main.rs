mod http;
use std::{fs, path::PathBuf};

#[derive(serde::Deserialize, serde::Serialize)]
struct CdnFile {
    name: String,
    size: u32,
    hash: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Game<'a> {
    engine: &'a str,
    client: &'a str,
    references: Vec<&'a str>,
}

const MASTER: &str = "https://master.alterware.dev";
const REPO: &str = "mxve/alterware-launcher";

fn version_str_to_int(version: &str) -> u16 {
    version.replace(['v', '.'], "").parse::<u16>().unwrap()
}

#[cfg(windows)]
fn extract_archive() {
    let mut archive = zip::ZipArchive::new(fs::File::open("alterware_update").unwrap()).unwrap();
    archive.extract("alterware-update").unwrap();
}

#[cfg(windows)]
fn update_binary() {
    let update_script = "
        @echo off
        del alterware-launcher.exe
        move alterware-update\\alterware-launcher.exe alterware-launcher.exe
        rmdir /s /q alterware-update
        start alterware-launcher.exe
        exit"
        .replace("        ", "");

    fs::write("update.bat", update_script).unwrap();
    std::process::Command::new("update.bat")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    std::process::exit(0);
}

#[cfg(not(windows))]
fn extract_archive() {}

#[cfg(not(windows))]
fn update_binary() {}

// allow dead code, function is only called if not debug env
#[allow(dead_code)]
fn update_self() {
    let platform = if cfg!(windows) {
        "alterware-launcher-x86_64-pc-windows-msvc.zip"
    } else {
        "unknown"
    };

    if platform == "unknown" {
        println!("Unsupported platform, can't perform self-update!");
        return;
    }

    let current_version = version_str_to_int(env!("CARGO_PKG_VERSION"));

    let github_body = http::get_body_string(
        format!("https://api.github.com/repos/{}/releases/latest", REPO).as_str(),
    );
    let github_json: serde_json::Value = serde_json::from_str(&github_body).unwrap();
    let latest_version = version_str_to_int(github_json["tag_name"].as_str().unwrap());

    if latest_version > current_version {
        println!("Updating launcher..");
        let download_url = format!(
            "https://github.com/{}/releases/download/{}/{}",
            REPO,
            github_json["tag_name"].as_str().unwrap(),
            platform
        );
        println!("Downloading {}...", download_url);
        http::download_file(&download_url, &PathBuf::from("alterware_update"));
        println!("Extracting update...");
        extract_archive();
        fs::remove_file("alterware_update").unwrap();
        update_binary();
    }
}

fn file_get_sha1(path: &PathBuf) -> String {
    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(&fs::read(path).unwrap());
    sha1.digest().to_string()
}

fn update(game: &Game) {
    let cdn_info: Vec<CdnFile> = serde_json::from_str(&http::get_body_string(
        format!(
            "{}/files.json?{}",
            MASTER,
            rand::Rng::gen_range(&mut rand::thread_rng(), 0..1000)
        )
        .as_str(),
    ))
    .unwrap();

    let mut files_to_update: Vec<CdnFile> = Vec::new();
    for file in cdn_info {
        if file.name.starts_with(game.engine) {
            files_to_update.push(file);
        }
    }

    for file in files_to_update {
        let file_path = PathBuf::from(&file.name.replace(&format!("{}/", game.engine), ""));
        if file_path.exists() {
            let sha1_local = file_get_sha1(&file_path).to_lowercase();
            let sha1_remote = file.hash.to_lowercase();
            if sha1_local != sha1_remote {
                println!(
                    "Updating {}...\nLocal hash: {}\nRemote hash: {}",
                    file_path.display(),
                    sha1_local,
                    sha1_remote
                );
                http::download_file(
                    &format!(
                        "{}/{}?{}",
                        MASTER,
                        file.name,
                        rand::Rng::gen_range(&mut rand::thread_rng(), 0..1000)
                    ),
                    &file_path,
                );
            }
        } else {
            println!("Downloading {}...", file_path.display());
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).unwrap();
                }
            }
            http::download_file(
                &format!(
                    "{}/{}?{}",
                    MASTER,
                    file.name,
                    rand::Rng::gen_range(&mut rand::thread_rng(), 0..1000)
                ),
                &file_path,
            );
        }
    }
}

fn launch(file_path: &PathBuf) {
    println!("Launching {}...", file_path.display());
    std::process::Command::new(file_path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn main() {
    #[cfg(not(debug_assertions))]
    #[cfg(windows)]
    update_self();

    let mut args: Vec<String> = std::env::args().collect();

    let games_json = http::get_body_string(format!("{}/games.json", MASTER).as_str());
    let games: Vec<Game> = serde_json::from_str(&games_json).unwrap();

    let mut update_only = false;
    if args.contains(&String::from("update")) {
        update_only = true;
        args.iter()
            .position(|r| r == "update")
            .map(|e| args.remove(e));
    }

    let mut game: String = String::new();
    if args.len() > 1 {
        game = String::from(&args[1]);
    } else {
        'main: for g in games.iter() {
            for r in g.references.iter() {
                if std::path::Path::new(r).exists() {
                    game = String::from(g.client);
                    break 'main;
                }
            }
        }
    }

    for g in games.iter() {
        if g.client == game {
            update(g);
            if update_only {
                return;
            }
            launch(&PathBuf::from(format!("{}.exe", g.client)));
            return;
        }
    }

    println!("Game not found!");
    println!("Place the launcher in the game folder, if that doesn't work specify the client on the command line (ex. alterware-launcher.exe iw4-sp)");
    println!("Press enter to exit...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
