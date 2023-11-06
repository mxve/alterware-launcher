use semver::Version;

pub async fn latest_tag(owner: &str, repo: &str) -> String {
    let github_body = crate::http_async::get_body_string(
        format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        )
        .as_str(),
    )
    .await
    .unwrap();
    let github_json: serde_json::Value = serde_json::from_str(&github_body).unwrap();
    github_json["tag_name"].to_string().replace('"', "")
}

pub async fn latest_version(owner: &str, repo: &str) -> Version {
    let tag = latest_tag(owner, repo).await.replace('v', "");
    Version::parse(&tag).unwrap()
}

pub fn latest_release_url(owner: &str, repo: &str) -> String {
    format!("https://github.com/{}/{}/releases/latest", owner, repo)
}
