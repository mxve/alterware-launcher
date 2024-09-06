use semver::Version;

pub async fn latest_tag(owner: &str, repo: &str) -> Result<String, Box<dyn std::error::Error>> {
    let github_body = crate::http_async::get_body_string(
        format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        )
        .as_str(),
    )
    .await?;

    let github_json: serde_json::Value = serde_json::from_str(&github_body)?;

    if let Some(tag_name) = github_json.get("tag_name") {
        if let Some(tag_name_str) = tag_name.as_str() {
            return Ok(tag_name_str.to_string().replace('"', ""));
        }
    }

    Ok("0.0.0".to_string())
}

pub async fn latest_version(owner: &str, repo: &str) -> Version {
    match latest_tag(owner, repo).await {
        Ok(tag) => {
            let cleaned_tag = tag.replace('v', "");
            Version::parse(&cleaned_tag).unwrap_or_else(|_| Version::new(0, 0, 0))
        }
        Err(_) => {
            crate::println_error!(
                "Failed to get latest version for {owner}/{repo}, assuming we are up to date."
            );
            Version::new(0, 0, 0)
        }
    }
}

pub fn latest_release_url(owner: &str, repo: &str) -> String {
    format!("https://github.com/{owner}/{repo}/releases/latest")
}
