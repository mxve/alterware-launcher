use colored::ColoredString;
use std::{collections::HashMap, path::Path};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct CdnFile {
    pub name: String,
    pub size: u32,
    pub blake3: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Game<'a> {
    pub engine: &'a str,
    pub client: Vec<&'a str>,
    pub references: Vec<&'a str>,
    pub app_id: u32,
    pub bonus: Vec<&'a str>,
    pub delete: Vec<&'a str>,
    pub required: Vec<&'a str>,
}

impl<'a> Game<'a> {
    pub fn required_files_exist(&self, dir: &Path) -> bool {
        for required_file in &self.required {
            let file_path = dir.join(required_file);
            if !file_path.exists() {
                crate::println_error!("Required file {} does not exist", file_path.display());
                return false;
            }
        }
        true
    }
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
pub struct Config {
    pub update_only: bool,
    pub skip_self_update: bool,
    pub download_bonus_content: bool,
    pub ask_bonus_content: bool,
    pub force_update: bool,
    pub args: String,
    #[serde(default)]
    pub engine: String,
    #[serde(default)]
    pub use_https: bool,
    #[serde(default)]
    pub skip_redist: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_only: false,
            skip_self_update: false,
            download_bonus_content: true,
            ask_bonus_content: false,
            force_update: false,
            args: String::default(),
            engine: String::default(),
            use_https: true,
            skip_redist: false,
        }
    }
}

pub struct PrintPrefix {
    pub text: ColoredString,
    pub padding: usize,
}

impl PrintPrefix {
    pub fn formatted(&self) -> String {
        format!("[{}]{:width$}", self.text, "", width = self.padding).to_string()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, PartialEq, Debug, Clone)]
pub struct Cache {
    pub iw4x_revision: String,
    pub hashes: HashMap<String, String>,
}
