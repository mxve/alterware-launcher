#[derive(serde::Deserialize, serde::Serialize)]
pub struct CdnFile {
    pub name: String,
    pub size: u32,
    pub hash: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Game<'a> {
    pub engine: &'a str,
    pub client: Vec<&'a str>,
    pub references: Vec<&'a str>,
    pub app_id: u32,
    pub bonus: Vec<&'a str>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub update_only: bool,
    pub skip_self_update: bool,
    pub download_bonus_content: bool,
    pub ask_bonus_content: bool,
    pub force_update: bool,
    pub args: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_only: false,
            skip_self_update: false,
            download_bonus_content: false,
            ask_bonus_content: true,
            force_update: false,
            args: String::from(""),
        }
    }
}
