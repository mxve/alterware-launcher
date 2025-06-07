use std::fs;
#[cfg(windows)]
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};

use crate::global;

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

#[cfg(windows)]
pub fn is_program_in_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .and_then(|paths| {
            paths.to_str().map(|paths| {
                paths.split(';').any(|dir| {
                    fs::metadata(format!("{}\\{}.exe", dir, program)).is_ok()
                        || fs::metadata(format!("{}\\{}.cmd", dir, program)).is_ok()
                        || fs::metadata(format!("{}\\{}.bat", dir, program)).is_ok()
                })
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
pub async fn install_dependencies(_install_path: &Path, force_reinstall: bool) {
    if force_reinstall {
        crate::println_info!("Force reinstalling redistributables...");
    } else {
        crate::println_info!("Installing redistributables...");
    }

    if !is_program_in_path("winget") {
        crate::println_info!(
            "winget is not available. Unable to install redistributables automatically."
        );
        crate::println_info!(
            "Please install Visual C++ Redistributables and DirectX manually if needed."
        );
        return;
    }

    let packages = [
        "Microsoft.VCRedist.2005.x86",
        "Microsoft.VCRedist.2008.x86",
        "Microsoft.VCRedist.2010.x86",
        "Microsoft.VCRedist.2015+.x86",
        "Microsoft.DirectX",
    ];

    for package in packages.iter() {
        let mut args = vec![
            "install",
            "--id",
            package,
            "--silent",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ];

        if force_reinstall {
            args.push("--force");
        }

        let result = std::process::Command::new("winget").args(&args).output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    crate::println_info!("Successfully installed {}", package);
                } else {
                    crate::println_info!(
                        "Failed to install {} (may already be installed)",
                        package
                    );
                }
            }
            Err(_) => {
                crate::println_info!("Unable to install redistributables automatically.");
                crate::println_info!(
                    "Please install Visual C++ Redistributables and DirectX manually if needed."
                );
                break;
            }
        }
    }
}

pub fn prefix(tag_name: &str) -> String {
    global::PREFIXES
        .get(tag_name)
        .map_or_else(|| tag_name.to_string(), |tag| tag.formatted())
}

pub fn random_string(length: u32) -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let mut result = String::new();
    for _ in 0..length {
        let random: u8 = rng.random_range(33..127);
        result.push(random as char);
    }
    result
}
