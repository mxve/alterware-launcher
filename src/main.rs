mod cdn;
mod extend;
mod game;
mod global;
mod hash;
mod http;

use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("I:\\SteamLibrary\\steamapps\\common\\Call of Duty Modern Warfare 2");
    let game = game::detect_game(path);

    if let Some(game) = game {
        println!("Game: {:?}", game);
        let clients = game.clients();
        println!("Clients: {:?}", clients);

        let info = cdn::get_info().await?;
        let files = cdn::filter_files(info.files.clone(), game);
        println!("Files: {:?}", files);
        for file in files {
            println!("File: {:?}", file);
            println!("Size: {:?}", file.size_human());
            println!("URL: {:?}", file.url());
            println!("Cache name: {:?}", file.cache_name());
            println!("Cache path: {:?}", file.cache_path());
        }
    } else {
        println!("No game detected");
    }

    Ok(())
}
