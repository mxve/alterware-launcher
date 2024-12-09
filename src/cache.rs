use crate::structs::{Cache, StoredGameData};
use std::{fs, path::Path};

pub fn get_cache(dir: &Path) -> Cache {
    let cache_path = dir.join("awcache.json");
    let cache_content = fs::read_to_string(cache_path).unwrap_or_default();
    if cache_content.trim().is_empty() {
        Cache::default()
    } else {
        serde_json::from_str(&cache_content).unwrap_or_default()
    }
}

pub fn save_cache(dir: &Path, cache: Cache) {
    let cache_path = dir.join("awcache.json");
    let cache_serialized = serde_json::to_string_pretty(&cache).unwrap();
    fs::write(cache_path, cache_serialized).unwrap_or_else(|e| {
        error!("Failed to save cache: {}", e);
    });
}

pub fn get_stored_data() -> Option<StoredGameData> {
    let dir = std::env::current_dir().ok()?;
    let cache = get_cache(&dir);
    cache.stored_data
}

pub fn store_game_data(data: &StoredGameData) -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::current_dir()?;
    let mut cache = get_cache(&dir);
    cache.stored_data = Some((*data).clone());
    save_cache(&dir, cache);
    Ok(())
}
