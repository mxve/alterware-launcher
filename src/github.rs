use semver::Version;

pub fn latest(owner: &str, repo: &str) -> String {
    let github_body = crate::http::get_body_string(
        format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        )
        .as_str(),
    );
    let github_json: serde_json::Value = serde_json::from_str(&github_body).unwrap();
    github_json["tag_name"]
        .to_string()
        .replace(['v', '"'].as_ref(), "")
}

pub fn latest_version(owner: &str, repo: &str) -> Version {
    Version::parse(&latest(owner, repo)).unwrap()
}

pub fn latest_release_url(owner: &str, repo: &str) -> String {
    format!("https://github.com/{}/{}/releases/latest", owner, repo)
}
