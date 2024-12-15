use semver::Version;

pub async fn latest_tag(
    owner: &str,
    repo: &str,
    prerelease: Option<bool>,
) -> Result<String, Box<dyn std::error::Error>> {
    if prerelease.unwrap_or(false) {
        latest_tag_prerelease(owner, repo).await
    } else {
        latest_tag_full(owner, repo).await
    }
}

pub async fn latest_tag_full(
    owner: &str,
    repo: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let github_body = crate::http_async::get_body_string(
        format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        )
        .as_str(),
    )
    .await
    .map_err(|e| format!("Failed to fetch GitHub API: {}", e))?;

    let github_json: serde_json::Value = serde_json::from_str(&github_body)
        .map_err(|e| format!("Failed to parse GitHub API response: {}", e))?;

    let tag_name = github_json
        .get("tag_name")
        .ok_or("Missing tag_name field in GitHub response")?
        .as_str()
        .ok_or("tag_name is not a string")?;

    Ok(tag_name.replace('"', ""))
}

pub async fn latest_tag_prerelease(
    owner: &str,
    repo: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let github_body = crate::http_async::get_body_string(
        format!("https://api.github.com/repos/{}/{}/releases", owner, repo).as_str(),
    )
    .await
    .map_err(|e| format!("Failed to fetch GitHub API: {}", e))?;

    let github_json: serde_json::Value = serde_json::from_str(&github_body)
        .map_err(|e| format!("Failed to parse GitHub API response: {}", e))?;

    let latest_release = github_json.get(0).ok_or("No releases found")?;

    let tag_name = latest_release
        .get("tag_name")
        .ok_or("Release missing tag_name")?
        .as_str()
        .ok_or("tag_name is not a string")?;

    Ok(tag_name.replace('"', ""))
}

pub async fn latest_version(
    owner: &str,
    repo: &str,
    prerelease: Option<bool>,
) -> Result<Version, Box<dyn std::error::Error>> {
    let tag = latest_tag(owner, repo, prerelease).await?;
    let cleaned_tag = tag.replace('v', "");
    Version::parse(&cleaned_tag)
        .map_err(|e| format!("Failed to parse version '{}': {}", cleaned_tag, e).into())
}

pub fn download_url(owner: &str, repo: &str, tag: Option<&str>) -> String {
    if let Some(tag) = tag {
        format!("https://github.com/{owner}/{repo}/releases/download/{tag}")
    } else {
        format!("https://github.com/{owner}/{repo}/releases/latest")
    }
}
