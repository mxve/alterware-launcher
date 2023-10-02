use std::{fs, path::PathBuf};

use colored::Colorize;

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
    println!("\n\n{}:\n{}", "Error".bright_red(), error);
    stdin();
    std::process::exit(1);
}
