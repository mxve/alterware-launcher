use std::{fs, path::PathBuf};

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
    rev.strip_prefix('r').unwrap().parse::<u16>().unwrap_or(0)
}
