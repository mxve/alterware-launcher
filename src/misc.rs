use std::{fs, path::Path};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{global, structs};

pub fn stdin() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn rev_to_int(rev: &str) -> u16 {
    rev.strip_prefix('r')
        .unwrap_or("0")
        .parse::<u16>()
        .unwrap_or(0)
}

pub fn human_readable_bytes(bytes: u64) -> String {
    let mut bytes = bytes as f64;
    let mut i = 0;
    const UNITS: [&str; 9] = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    while bytes >= 1024.0 {
        bytes /= 1024.0;
        i += 1;
    }
    format!("{bytes:.2}{}", UNITS[i])
}

pub fn pb_style_download(pb: &ProgressBar, state: bool) {
    let style = if state {
        ProgressStyle::with_template(
            "{spinner:.magenta} {msg:.magenta} > {bytes}/{total_bytes} | {bytes_per_sec} | {eta}",
        )
    } else {
        ProgressStyle::with_template("{spinner:.magenta} {msg}")
    };
    pb.set_style(style.unwrap());
}

#[cfg(unix)]
pub fn is_program_in_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .and_then(|paths| {
            paths.to_str().map(|paths| {
                paths
                    .split(':')
                    .any(|dir| fs::metadata(format!("{}/{}", dir, program)).is_ok())
            })
        })
        .unwrap_or(false)
}

#[macro_export]
macro_rules! println_info {
    ($($arg:tt)*) => {{
        println!("{}", format!("{}{}", $crate::misc::prefix("info"), format!($($arg)*)));
        info!($($arg)*);
    }}
}

#[macro_export]
macro_rules! println_error {
    ($($arg:tt)*) => {{
        eprintln!("{}", format!("{}{}", $crate::misc::prefix("error"), format!($($arg)*)));
        error!($($arg)*);
    }}
}

#[cfg(windows)]
fn install_dependency(path: &Path, args: &[&str]) {
    if path.exists() {
        match runas::Command::new(path).args(args).status() {
            Ok(status) if status.success() || matches!(status.code(), Some(1638) | Some(3010)) => {
                info!("{} installed successfully", path.display());
            }
            Ok(status) => {
                println_error!("Error installing dependency {}, {status}", path.display());
            }
            Err(e) if e.raw_os_error() == Some(740) => {
                println_error!(
                    "Error: Process requires elevation. Please run the launcher as administrator or install {} manually",
                    path.display()
                );
            }
            Err(e) => {
                println_error!("Error running file {}: {e}", path.display());
            }
        }
    } else {
        println_error!("Installer not found: {}", path.display());
    }
}

#[cfg(windows)]
async fn download_and_install_dependency(url: &str, path: &Path, args: &[&str]) {
    if !path.exists() {
        info!("Downloading {} from {url}", path.display());
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                println_error!("Error creating directory {}: {e}", parent.display());
                return;
            }
        }
        match crate::http_async::download_file(url, &std::path::PathBuf::from(path)).await {
            Ok(_) => info!("Downloaded {}", path.display()),
            Err(e) => {
                println_error!("Error downloading {}: {e}", path.display());
                return;
            }
        }
    }
    install_dependency(path, args);
}

#[cfg(windows)]
pub async fn install_dependencies(install_path: &Path) {
    println!("If you run into issues during dependency installation, open alterware-launcher.json and set \"skip_redist\" to true");

    let redist_dir = install_path.join("redist\\alterware");
    let redists = [
        ("VC++ 2005", "https://download.microsoft.com/download/8/B/4/8B42259F-5D70-43F4-AC2E-4B208FD8D66A/vcredist_x86.EXE", "vcredist_2005_x86.exe", &["/Q"] as &[&str]),
        ("VC++ 2008", "https://download.microsoft.com/download/5/D/8/5D8C65CB-C849-4025-8E95-C3966CAFD8AE/vcredist_x86.exe", "vcredist_2008_x86.exe", &["/Q"] as &[&str]),
        ("VC++ 2010", "https://download.microsoft.com/download/1/6/5/165255E7-1014-4D0A-B094-B6A430A6BFFC/vcredist_x86.exe", "vcredist_2010_x86.exe", &["/Q"] as &[&str]),
        ("VC++ 2015", "https://download.microsoft.com/download/9/3/F/93FCF1E7-E6A4-478B-96E7-D4B285925B00/vc_redist.x86.exe", "vcredist_2015_x86.exe", &["/install", "/passive", "/norestart"] as &[&str]),
        ("DirectX End User Runtime", "https://download.microsoft.com/download/1/7/1/1718CCC4-6315-4D8E-9543-8E28A4E18C4C/dxwebsetup.exe", "dxwebsetup.exe", &["/Q"] as &[&str]),
    ];

    for (name, url, file, args) in redists.iter() {
        let path = redist_dir.join(file);
        println_info!("Installing {name}");
        download_and_install_dependency(url, &path, args).await;
    }
}

pub fn prefix(tag_name: &str) -> String {
    global::PREFIXES
        .get(tag_name)
        .map_or_else(|| tag_name.to_string(), |tag| tag.formatted())
}

pub fn get_cache(dir: &Path) -> structs::Cache {
    let cache_path = dir.join("awcache.json");
    let cache_content = fs::read_to_string(cache_path).unwrap_or_default();
    if cache_content.trim().is_empty() {
        structs::Cache::default()
    } else {
        serde_json::from_str(&cache_content).unwrap_or_default()
    }
}

pub fn save_cache(dir: &Path, cache: structs::Cache) {
    let cache_path = dir.join("awcache.json");
    let cache_serialized = serde_json::to_string_pretty(&cache).unwrap();
    fs::write(cache_path, cache_serialized).unwrap_or_else(|e| {
        println_error!("Failed to save cache: {}", e);
    });
}

pub fn random_string(length: u32) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut result = String::new();
    for _ in 0..length {
        let random: u8 = rng.gen_range(33..127);
        result.push(random as char);
    }
    result
}
