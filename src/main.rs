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
const GAMES: [Game; 3] = [
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
    Game {
        engine: "iw6",
        client: "iw6-mod",
        references: &["iw6sp64_ship.exe", "iw6mp64_ship.exe"],
    },
];

fn file_get_sha1(path: &PathBuf) -> String {
    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(&fs::read(path).unwrap());
    sha1.digest().to_string()
}

fn update(game: &Game) {
    let cdn_info: Vec<CdnFile> = serde_json::from_str(&http::get_body_string(
        format!(
            "{}/files.json?{}",
            MASTER,
            rand::Rng::gen_range(&mut rand::thread_rng(), 0..1000)
        )
        .as_str(),
    ))
    .unwrap();

    let mut files_to_update: Vec<CdnFile> = Vec::new();
    for file in cdn_info {
        if file.name.starts_with(game.engine) {
            files_to_update.push(file);
        }
    }

    for file in files_to_update {
        let file_path = PathBuf::from(&file.name.replace(&format!("{}/", game.engine), ""));
        if file_path.exists() {
            let sha1_local = file_get_sha1(&file_path).to_lowercase();
            let sha1_remote = file.hash.to_lowercase();
            if sha1_local != sha1_remote {
                println!(
                    "Updating {}...\nLocal hash: {}\nRemote hash: {}",
                    file_path.display(),
                    sha1_local,
                    sha1_remote
                );
                http::download_file(
                    &format!(
                        "{}/{}?{}",
                        MASTER,
                        file.name,
                        rand::Rng::gen_range(&mut rand::thread_rng(), 0..1000)
                    ),
                    &file_path,
                );
            }
        } else {
            println!("Downloading {}...", file_path.display());
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).unwrap();
                }
            }
            http::download_file(
                &format!(
                    "{}/{}?{}",
                    MASTER,
                    file.name,
                    rand::Rng::gen_range(&mut rand::thread_rng(), 0..1000)
                ),
                &file_path,
            );
        }
    }
}

fn launch(file_path: &PathBuf) {
    println!("Launching {}...", file_path.display());
    std::process::Command::new(file_path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
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
            update(&g);
            launch(&PathBuf::from(format!("{}.exe", g.client)));
            return;
        }
    }

    println!("Game not found!");
    println!("Place the launcher in the game folder, if that doesn't work specify the client on the command line (ex. alterware-launcher.exe iw4-sp)");
    println!("Press enter to exit...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
