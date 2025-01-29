use crate::extend::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct File {
    pub name: String,
    pub size: u64,
    pub hash: String,
}

impl File {
    pub fn url(&self) -> String {
        format!("{}/{}", crate::global::CDN_URL, self.hash)
    }

    pub fn cache_name(&self) -> String {
        format!("{}", self.hash[..24].to_string())
    }

    pub fn cache_path(&self) -> String {
        format!("{}/{}", crate::global::CACHE_DIR, self.cache_name())
    }

    pub fn size_human(&self) -> String {
        self.size.human_readable_size()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Info {
    pub delete_list: Vec<String>,
    pub files: Vec<File>,
}

pub async fn get_info() -> Result<Info, Box<dyn std::error::Error>> {
    let info = crate::http::quick_request(&format!("{0}/info.json", crate::global::CDN_URL)).await?;
    Ok(serde_json::from_str(&info)?)
}

pub fn filter_files(files: Vec<File>, game: crate::game::Game) -> Vec<File> {
    files
        .into_iter()
        .filter(|file| {
            game.cdn_base_dirs()
                .contains(&file.name.split('/').next().unwrap())
        })
        .collect()
}
