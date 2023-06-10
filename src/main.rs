mod http;
use std::{fs, path::PathBuf};

#[derive(serde::Deserialize, serde::Serialize)]
struct CdnFile {
    name: String,
    size: u32,
    hash: String,
}

struct Game<'a> {
    engine: &'a str,
    client: &'a str,
    references: &'a [&'a str],
}

const MASTER: &str = "https://master.alterware.dev";
const GAMES: [Game; 2] = [
    Game {
        engine: "iw4",
        client: "iw4-sp",
        references: &["iw4sp.exe", "iw4mp.exe"],
    },
    Game {
        engine: "iw5",
        client: "iw5-mod",
        references: &["iw5sp.exe", "iw5mp.exe", "iw5mp_server.exe"],
    },
];

fn file_get_sha1(path: &PathBuf) -> String {
    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(&fs::read(path).unwrap());
    sha1.digest().to_string()
}

fn download_and_launch(url: &str, file_path: &PathBuf, hash: Option<String>) {
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
    }

    // clippy will suggest using if let or match, but i prefer it being somewhat readable
    #[allow(clippy::unnecessary_unwrap)]
    if file_path.exists() && hash.is_some() {
        let sha1_local = file_get_sha1(file_path).to_lowercase();
        let sha1_remote = hash.unwrap().to_lowercase();
        if sha1_local != sha1_remote {
            println!(
                "Updating {}...\nLocal hash: {}\nRemote hash: {}",
                file_path.display(),
                sha1_local,
                sha1_remote
            );
            http::download_file(url, file_path);
        }
    } else {
        println!("Downloading {}...", file_path.display());
        http::download_file(url, file_path);
    }

    println!("Launching {}...", file_path.display());
    std::process::Command::new(file_path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn get_hash(game: &Game) -> Option<String> {
    let cdn_info: Vec<CdnFile> = serde_json::from_str(&http::get_body_string(
        format!(
            "{}/files.json?{}",
            MASTER,
            rand::Rng::gen_range(&mut rand::thread_rng(), 0..1000)
        )
        .as_str(),
    ))
    .unwrap();

    for file in cdn_info {
        if file.name == format!("{}/{}.exe", game.engine, game.client) {
            return Some(file.hash);
        }
    }

    None
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut game: String = String::new();
    if args.len() > 1 {
        game = String::from(&args[1]);
    } else {
        'main: for g in GAMES.iter() {
            for r in g.references.iter() {
                if std::path::Path::new(r).exists() {
                    game = String::from(g.client);
                    break 'main;
                }
            }
        }
    }

    for g in GAMES.iter() {
        if g.client == game {
            download_and_launch(
                &format!(
                    "{}/{}/{}.exe?{}",
                    MASTER,
                    g.engine,
                    g.client,
                    rand::Rng::gen_range(&mut rand::thread_rng(), 0..1000)
                ),
                &PathBuf::from(format!("{}.exe", g.client)),
                get_hash(g),
            );
            return;
        }
    }
}
