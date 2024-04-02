use std::{
    fs,
    path::{Path, PathBuf},
};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub fn get_file_sha1(path: &PathBuf) -> String {
    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(&fs::read(path).unwrap());
    sha1.digest().to_string()
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
