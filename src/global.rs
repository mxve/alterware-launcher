use crate::structs::PrintPrefix;
use colored::Colorize;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub const GH_OWNER: &str = "mxve";
pub const GH_REPO: &str = "alterware-launcher";
pub const GH_IW4X_OWNER: &str = "iw4x";
pub const GH_IW4X_REPO: &str = "iw4x-client";

pub static MASTER: Lazy<Mutex<String>> =
    Lazy::new(|| Mutex::new("https://cdn.alterware.ovh".to_owned()));

pub static PREFIXES: Lazy<HashMap<&'static str, PrintPrefix>> = Lazy::new(|| {
    HashMap::from([
        (
            "info",
            PrintPrefix {
                text: "Info".bright_magenta(),
                padding: 8,
            },
        ),
        (
            "downloading",
            PrintPrefix {
                text: "Downloading".bright_yellow(),
                padding: 1,
            },
        ),
        (
            "checked",
            PrintPrefix {
                text: "Checked".bright_blue(),
                padding: 5,
            },
        ),
        (
            "removed",
            PrintPrefix {
                text: "Removed".bright_red(),
                padding: 5,
            },
        ),
        (
            "error",
            PrintPrefix {
                text: "Error".red(),
                padding: 7,
            },
        ),
    ])
});
