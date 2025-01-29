mod cdn;
mod extend;
mod game;
mod global;
mod hash;
mod http;
mod utils;

use std::path::PathBuf;
use strum::IntoEnumIterator;

fn arg_value(args: &[String], possible_args: &[&str]) -> Option<String> {
    for arg in possible_args {
        if let Some(e) = args.iter().position(|r| r == *arg) {
            if e + 1 < args.len() {
                let value = args[e + 1].clone();
                if value.starts_with('-') {
                    continue;
                }

                arg_value_remove(&mut args.to_vec(), arg);
                return Some(value);
            }
        }
    }
    None
}

fn arg_value_remove(args: &mut Vec<String>, arg: &str) {
    if let Some(e) = args.iter().position(|r| r == arg) {
        args.remove(e);
        args.remove(e);
    };
}

fn arg_bool(args: &[String], possible_args: &[&str]) -> bool {
    possible_args
        .iter()
        .any(|arg| args.iter().any(|r| r == *arg))
}

fn arg_path(args: &[String], possible_args: &[&str]) -> Option<PathBuf> {
    arg_value(args, possible_args).map(PathBuf::from)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();

    utils::set_mutex(
        &global::CDN_PROTOCOL,
        arg_value(&args, &["--protocol"]).unwrap_or("https".to_owned()),
    );
    utils::set_mutex(
        &global::CDN_BRANCH,
        arg_value(&args, &["--branch"]).unwrap_or("stable".to_owned()),
    );
    utils::set_mutex(
        &global::GAME_DIR,
        arg_path(&args, &["-p", "--path", "--game-path"])
            .unwrap_or(std::env::current_dir().unwrap()),
    );

    let client = args
        .iter()
        .find_map(|arg| game::Client::iter().find(|&client| client.internal_name() == arg));

    if let Some(client) = client {
        utils::set_mutex(&global::GAME_CLIENT, Some(client));
        utils::set_mutex(&global::GAME, client.game());
    } else {
        if let Some(game) = game::detect_game(&utils::get_mutex(&global::GAME_DIR)) {
            utils::set_mutex(&global::GAME, game);
            println!("Game: {:?}", game);

            match game.clients().len() {
                0 => {
                    println!("No clients found for game");
                    return Ok(());
                }
                1 => {
                    utils::set_mutex(&global::GAME_CLIENT, Some(game.clients()[0]));
                }
                _ => {
                    println!("Multiple clients found for game, please specify one");
                    return Ok(());
                }
            }
        } else {
            println!("No game detected");
            return Ok(());
        }
    }

    let game = utils::get_mutex(&global::GAME);
    let clients = game.clients();
    println!("Clients: {:?}", clients);

    let info = cdn::get_info().await?;
    let files = cdn::filter_files(info.files.clone(), game);
    //println!("Files: {:?}", files);
    //for file in files {
    //println!("File: {:?}", file);
    // println!("Size: {:?}", file.size_human());
    // println!("URL: {:?}", file.url());
    // println!("Cache name: {:?}", file.cache_name());
    // println!("Cache path: {:?}", file.cache_path());
    //}

    Ok(())
}
