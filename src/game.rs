#![allow(unused)]

use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// The base games that are supported by the launcher.
#[derive(Debug, EnumIter, Clone, Copy)]
pub enum Game {
    IW4,
    IW5,
    IW6,
    S1,
}

impl Game {
    /// Get the clients that support the base game.
    pub fn clients(&self) -> Vec<Client> {
        match self {
            Game::IW4 => vec![Client::IW4x, Client::IW4xSP],
            Game::IW5 => vec![Client::IW5Mod],
            Game::IW6 => vec![Client::IW6Mod],
            Game::S1 => vec![Client::S1Mod],
        }
    }

    /// CDN base directory for the game.
    pub fn cdn_base_dirs(&self) -> Vec<&str> {
        match self {
            Game::IW4 => vec!["iw4", "iw4-dlc"],
            Game::IW5 => vec!["iw5"],
            Game::IW6 => vec!["iw6"],
            Game::S1 => vec!["s1"],
        }
    }

    /// Get the name of the base game.
    pub fn name(&self) -> &str {
        match self {
            Game::IW4 => "Call of Duty: Modern Warfare 2",
            Game::IW5 => "Call of Duty: Modern Warfare 3",
            Game::IW6 => "Call of Duty: Ghosts",
            Game::S1 => "Call of Duty: Advanced Warfare",
        }
    }

    /// Get the Steam app ID of the base game.
    pub fn steam_app_id(&self) -> u32 {
        match self {
            Game::IW4 => 10180,
            Game::IW5 => 115300,
            Game::IW6 => 209160,
            Game::S1 => 209650,
        }
    }

    /// Get the required files for the game, if these don't exist the game files are definetly invalid.
    pub fn required_files(&self) -> Vec<&str> {
        match self {
            Game::IW4 => vec!["binkw32.dll", "mss32.dll"],
            Game::IW5 => vec!["binkw32.dll", "mss32.dll"],
            Game::IW6 => vec![],
            Game::S1 => vec![],
        }
    }

    /// Get the reference files for the game, these files are unique to each base game and are used to identify it.
    pub fn reference_files(&self) -> Vec<&str> {
        match self {
            Game::IW4 => vec!["iw4sp.exe", "iw4mp.exe", "iw4x.exe"],
            Game::IW5 => vec!["iw5sp.exe", "iw5mp.exe", "iw5mp_server.exe"],
            Game::IW6 => vec!["iw6sp64_ship.exe", "iw6mp64_ship.exe"],
            Game::S1 => vec!["s1_sp64_ship.exe", "s1_mp64_ship.exe"],
        }
    }

    /// Check if any of the reference files exists in the given path
    pub fn reference_file_exist(&self, path: &Path) -> bool {
        self.reference_files()
            .iter()
            .any(|file| path.join(file).exists())
    }

    /// Check if all of the required files exist in the given path
    pub fn required_files_exist(&self, path: &Path) -> bool {
        self.required_files()
            .iter()
            .all(|file| path.join(file).exists())
    }
}

/// The clients that are supported by the launcher
#[derive(Debug, EnumIter, Clone, Copy)]
pub enum Client {
    IW4x,
    IW4xSP,
    IW5Mod,
    IW6Mod,
    S1Mod,
}

impl Client {
    /// Get the base game that the client supports
    pub fn game(&self) -> Game {
        match self {
            Client::IW4x => Game::IW4,
            Client::IW4xSP => Game::IW4,
            Client::IW5Mod => Game::IW5,
            Client::IW6Mod => Game::IW6,
            Client::S1Mod => Game::S1,
        }
    }

    /// Get the executable name of the client.
    pub fn executable(&self) -> &str {
        match self {
            Client::IW4x => "iw4x.exe",
            Client::IW4xSP => "iw4x-sp.exe",
            Client::IW5Mod => "iw5-mod.exe",
            Client::IW6Mod => "iw6-mod.exe",
            Client::S1Mod => "s1-mod.exe",
        }
    }

    /// Get the name of the client
    pub fn name(&self) -> &str {
        match self {
            Client::IW4x => "IW4x Multiplayer",
            Client::IW4xSP => "IW4x Singleplayer",
            Client::IW5Mod => "IW5 Mod",
            Client::IW6Mod => "IW6 Mod",
            Client::S1Mod => "S1 Mod",
        }
    }

    pub fn internal_name(&self) -> &str {
        match self {
            Client::IW4x => "iw4x",
            Client::IW4xSP => "iw4x-sp",
            Client::IW5Mod => "iw5-mod",
            Client::IW6Mod => "iw6-mod",
            Client::S1Mod => "s1-mod",
        }
    }
}

/// Detect game in the given path
pub fn detect_game(path: &Path) -> Option<Game> {
    Game::iter().find(|&game| game.reference_file_exist(path))
}
