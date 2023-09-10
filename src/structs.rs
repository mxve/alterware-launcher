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
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub update_only: bool,
    pub skip_self_update: bool,
    pub bonus_content: bool,
}

// impl Default for Config {
//     fn default() -> Self {
//         Self {
//             update_only: false,
//             skip_self_update: false,
//             bonus_content: false,
//         }
//     }
// }
