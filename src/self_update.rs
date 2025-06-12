#[cfg(windows)]
use reqwest::Client;
#[cfg(windows)]
use std::{fs::File, io::Write, path::PathBuf};

use crate::Platform;

// GitHub constants
const GH_OWNER_IW4X: &str = "iw4x";
const GH_REPO_IW4X: &str = "launcher";
const GH_OWNER_ALTERWARE: &str = "alterware";
const GH_REPO_ALTERWARE: &str = "alterware-launcher";

pub async fn print_urls(platform: &Platform) {
    let (owner, repo) = match platform {
        Platform::IW4x => (GH_OWNER_IW4X, GH_REPO_IW4X),
        Platform::AlterWare => (GH_OWNER_ALTERWARE, GH_REPO_ALTERWARE),
    };

    println!("A new Launcher is available!");
    println!("Please download it from https://github.com/{owner}/{repo}");
}

pub fn stdin() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input
}

#[cfg(windows)]
pub async fn download_file(url: &str, path: &PathBuf) -> Result<(), String> {
    let body = get_body(url).await?;
    let mut file = File::create(path).or(Err("Failed to create file"))?;
    file.write_all(&body).or(Err("Failed to write to file"))?;
    Ok(())
}

#[cfg(windows)]
pub async fn get_body(url: &str) -> Result<Vec<u8>, String> {
    let client = Client::new();
    let res = client
        .get(url)
        .header(
            "User-Agent",
            "AlterWare Launcher migration tool | github.com/mxve/alterware-launcher",
        )
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    println!("{} {url}", res.status());

    res.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Failed to get body: {}", e))
}

#[cfg(windows)]
fn get_launcher_info(platform: &Platform) -> (String, String) {
    let arch_suffix = if cfg!(target_arch = "x86") {
        "-x86"
    } else {
        ""
    };

    let (name, owner, repo) = match platform {
        Platform::IW4x => (
            format!("iw4x-launcher{arch_suffix}.exe"),
            GH_OWNER_IW4X,
            GH_REPO_IW4X,
        ),
        Platform::AlterWare => (
            format!("alterware-launcher{arch_suffix}.exe"),
            GH_OWNER_ALTERWARE,
            GH_REPO_ALTERWARE,
        ),
    };

    let download_url = format!("https://github.com/{owner}/{repo}/releases/latest/download/{name}");
    (name, download_url)
}

#[cfg(windows)]
fn cleanup_old_files(working_dir: &std::path::Path, platform: &Platform) {
    use std::fs;

    let prefix = match platform {
        Platform::IW4x => "iw4x-launcher",
        Platform::AlterWare => "alterware-launcher",
    };

    if let Ok(files) = fs::read_dir(working_dir) {
        files
            .flatten()
            .filter_map(|file| file.file_name().into_string().ok().map(|name| (file, name)))
            .filter(|(_, name)| {
                name.contains(prefix)
                    && (name.contains(".__relocated__.exe") || name.contains(".__selfdelete__.exe"))
            })
            .for_each(|(file, _)| {
                fs::remove_file(file.path()).unwrap_or_else(|_| {
                    println!("Failed to remove old launcher file.");
                    stdin();
                });
            });
    }
}

#[cfg(windows)]
fn restart_launcher(exe_name: Option<&str>) -> std::io::Error {
    use std::os::windows::process::CommandExt;

    let exe_path = exe_name
        .map(|name| std::env::current_dir().unwrap().join(name))
        .unwrap_or_else(|| std::env::current_exe().unwrap());

    match std::process::Command::new(exe_path)
        .args(std::env::args().skip(1))
        .creation_flags(0x00000010)
        .spawn()
    {
        Ok(_) => std::process::exit(0),
        Err(err) => err,
    }
}

#[cfg(windows)]
fn handle_restart_failure(exe_name: Option<&str>) {
    let restart_error = restart_launcher(exe_name);
    println!("Failed to restart launcher: {restart_error}");
    println!("Please restart the launcher manually.");
    stdin();
    std::process::exit(201);
}

#[cfg(windows)]
async fn download_and_check(url: &str, path: &PathBuf) -> bool {
    download_file(url, path).await.unwrap_or_else(|e| {
        println!("Failed to download launcher update: {}", e);
    });

    if path.exists() {
        true
    } else {
        println!("Failed to download launcher update.");
        false
    }
}

#[cfg(not(windows))]
pub async fn run_with_platform(platform: Platform) {
    print_urls(&platform).await;
}

#[cfg(windows)]
pub async fn run_with_platform(platform: Platform) {
    use std::fs;

    let working_dir = std::env::current_dir().unwrap();
    cleanup_old_files(&working_dir, &platform);
    print_urls(&platform).await;

    let (launcher_name, download_url) = get_launcher_info(&platform);
    let target_exe = working_dir.join(&launcher_name);
    let current_exe = std::env::current_exe().unwrap();
    let diff_target_name = current_exe.file_name() != target_exe.file_name();

    if diff_target_name {
        if target_exe.exists() {
            fs::remove_file(&target_exe).unwrap_or_else(|_| {
                println!("Failed to remove existing target launcher.");
            });
        }

        if !download_and_check(&download_url, &target_exe).await {
            return;
        }

        let current_name = current_exe.file_name().unwrap().to_str().unwrap();
        let delete_name = format!(
            "{}.__selfdelete__.exe",
            current_name.trim_end_matches(".exe")
        );
        let delete_path = working_dir.join(&delete_name);

        fs::rename(&current_exe, &delete_path).unwrap_or_else(|e| {
            println!("Warning: Failed to mark old launcher for deletion: {}", e);
        });

        handle_restart_failure(Some(&launcher_name));
    } else {
        let update_binary = PathBuf::from(format!(
            "{}-update.exe",
            launcher_name.trim_end_matches(".exe")
        ));
        let file_path = working_dir.join(&update_binary);

        if update_binary.exists() {
            fs::remove_file(&update_binary).unwrap();
        }

        if !download_and_check(&download_url, &file_path).await {
            return;
        }

        let update_filename = update_binary.file_name().unwrap().to_str().unwrap();
        self_replace::self_replace(update_filename).unwrap();
        fs::remove_file(&file_path).unwrap();

        handle_restart_failure(None);
    }
}
