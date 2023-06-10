use std::{fs, io::Write, path::Path, str};

pub fn get_body(url: &str) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();
    http_req::request::get(url, &mut res).unwrap_or_else(|error| {
        panic!("\n\n{}:\n{:?}", "Error", error);
    });

    res
}

pub fn _get_body_string(url: &str) -> String {
    String::from_utf8(get_body(url)).unwrap()
}

pub fn download_file(url: &str, file_path: &Path) {
    let body = get_body(url);

    let mut f = fs::File::create(file_path).unwrap_or_else(|error| {
        panic!("\n\n{}:\n{:?}", "Error", error);
    });
    f.write_all(&body).unwrap_or_else(|error| {
        panic!("\n\n{}:\n{:?}", "Error", error);
    });
}
