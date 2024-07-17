use semver::Version;

use crate::misc;

#[derive(serde::Deserialize)]
struct GithubJSON {
    tag_name: String,
}

pub async fn latest_tag(owner: &str, repo: &str) -> String {
    match crate::http_async::get_body_json::<GithubJSON>(
        format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        )
        .as_str(),
    )
    .await
    {
        Ok(v) => v.tag_name, // There used to be a replace here, that serves no purpose.
        Err(e) => {
            misc::fatal_error(&e);
            panic!()
            // This panic shouldn't ever trigger, but compiler fucks up because of misc::fatal_error() exits the program.
            //
            // This error branc could and probably should be a recursive
            // function call, so that it doesn't just exit but it needs
            // converting from async syntax to Box<Future> syntax.
            //
            // This originally just returned v0.0.0 if it errored out.
        }
    }
}

pub async fn latest_version(owner: &str, repo: &str) -> Version {
    let tag = latest_tag(owner, repo).await.replace('v', "");
    Version::parse(&tag).unwrap()
}

pub fn latest_release_url(owner: &str, repo: &str) -> String {
    format!("https://github.com/{}/{}/releases/latest", owner, repo)
}
