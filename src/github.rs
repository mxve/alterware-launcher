use crate::global::*;

use semver::Version;

pub fn latest_version() -> Version {
    let github_body = crate::http::get_body_string(
        format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            GH_OWNER, GH_REPO
        )
        .as_str(),
    );
    let github_json: serde_json::Value = serde_json::from_str(&github_body).unwrap();
    let latest_version = github_json["tag_name"]
        .to_string()
        .replace(['v', '"'].as_ref(), "");
    Version::parse(&latest_version).unwrap()
}

pub fn latest_release_url() -> String {
    format!(
        "https://github.com/{}/{}/releases/latest",
        GH_OWNER, GH_REPO
    )
}
