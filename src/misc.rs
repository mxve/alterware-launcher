use std::{
    fs,
    path::{Path},
};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub fn file_blake3(file: &std::path::Path) -> std::io::Result<String> {
    let mut blake3 = blake3::Hasher::new();
    let mut file = std::fs::File::open(file)?;
    let mut buffer = [0; 1024];
    loop {
        let n = std::io::Read::read(&mut file, &mut buffer)?;
        if n == 0 {
            break;
        }
        blake3.update(&buffer[..n]);
    }
    Ok(blake3.finalize().to_string())
}

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

pub fn fatal_error(error: &str) {
    crate::println_error!("{}: {}", "Error".bright_red(), error);
    stdin();
    std::process::exit(1);
}

pub fn human_readable_bytes(bytes: u64) -> String {
    let mut bytes = bytes as f64;
    let mut i = 0;
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    while bytes > 1024.0 {
        bytes /= 1024.0;
        i += 1;
    }
    format!("{:.2}{}", bytes, units[i])
}

pub fn pb_style_download(pb: &ProgressBar, state: bool) {
    if state {
        pb.set_style(
            ProgressStyle::with_template("{spinner:.magenta} {msg:.magenta} > {bytes}/{total_bytes} | {bytes_per_sec} | {eta}")
                .unwrap(),
        );
    } else {
        pb.set_style(ProgressStyle::with_template("{spinner:.magenta} {msg}").unwrap());
    }
}

pub fn cute_path(path: &Path) -> String {
    path.to_str().unwrap().replace('\\', "/")
}

#[cfg(unix)]
pub fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for p in path.split(':') {
            let p_str = format!("{}/{}", p, program);
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

#[macro_export]
macro_rules! println_info {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        info!($($arg)*);
    }}
}

#[macro_export]
macro_rules! println_error {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        error!($($arg)*);
    }}
}

#[cfg(windows)]
fn install_dependency(path: &Path, args: &[&str]) {
    if path.exists() {
        match runas::Command::new(path).args(args).status() {
            Ok(status) => {
                if !status.success() && !matches!(status.code(), Some(1638) | Some(3010)) {
                    println_error!("Error installing dependency {}, {}", path.display(), status);
                } else {
                    info!("{} installed successfully", path.display());
                }
            }
            Err(e) => {
                if let Some(740) = e.raw_os_error() {
                    println_error!(
                        "Error: Process requires elevation. Please run the launcher as administrator or install {} manually",
                        path.display()
                    );
                } else {
                    println_error!("Error running file {}: {}", path.display(), e);
                }
            }
        }
    } else {
        println_error!("Installer not found: {}", path.display());
    }
}

#[cfg(windows)]
async fn download_and_install_dependency(url: &str, path: &Path, args: &[&str]) {
    if !path.exists() {
        info!("Downloading {} from {}", path.display(), url);
        if let Some(parent) = path.parent() {
            match fs::create_dir_all(parent) {
                Ok(_) => (),
                Err(e) => {
                    println_error!("Error creating directory {}: {}", parent.display(), e);
                    return;
                }
            }
        }
        match crate::http_async::download_file(url, &PathBuf::from(path)).await {
            Ok(_) => info!("Downloaded {}", path.display()),
            Err(e) => println_error!("Error downloading {}: {}", path.display(), e),
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
        println_info!("Installing {}", name);
        download_and_install_dependency(url, &path, args).await;
    }
}
