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
