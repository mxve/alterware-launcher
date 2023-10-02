use crate::misc;
use std::{fs, io::Write, path::Path, str};

pub fn get_body(url: &str) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    match http_req::request::Request::new(&url.try_into().unwrap())
        .header(
            "User-Agent",
            "AlterWare Launcher | github.com/mxve/alterware-launcher",
        )
        .send(&mut res)
    {
        Ok(req) => {
            if req.status_code() == http_req::response::StatusCode::new(302)
                || req.status_code() == http_req::response::StatusCode::new(301)
            {
                let location = req.headers().get("Location").unwrap().as_str();
                return get_body(location);
            }

            if req.status_code() != http_req::response::StatusCode::new(200) {
                misc::fatal_error(&format!(
                    "Could not get body from {}, got {}",
                    url,
                    req.status_code()
                ));
            }
        }
        Err(e) => {
            misc::fatal_error(&format!("Could not get body from {}, got:\n{}", url, e));
        }
    }

    res
}

pub fn get_body_string(url: &str) -> String {
    String::from_utf8(get_body(url)).unwrap()
}

pub fn download_file(url: &str, file_path: &Path) {
    let body = get_body(url);

    match fs::File::create(file_path) {
        Ok(mut file) => match file.write_all(&body) {
            Ok(_) => (),
            Err(e) => {
                misc::fatal_error(&format!(
                    "Could not write to file {}, got:\n{}",
                    file_path.to_str().unwrap(),
                    e
                ));
            }
        },
        Err(e) => {
            misc::fatal_error(&format!(
                "Could not create file {}, got:\n{}",
                file_path.to_str().unwrap(),
                e
            ));
        }
    }
}
