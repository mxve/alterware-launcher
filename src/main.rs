mod http;
#[cfg(windows)]
use mslnk::ShellLink;
use semver::Version;
use std::{fs, path::Path, path::PathBuf};
#[cfg(not(windows))]
use std::{thread, time};
#[cfg(windows)]
use steamlocate::SteamDir;

#[derive(serde::Deserialize, serde::Serialize)]
struct CdnFile {
    name: String,
    size: u32,
    hash: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Game<'a> {
    engine: &'a str,
    client: Vec<&'a str>,
    references: Vec<&'a str>,
    app_id: u32,
}

const MASTER: &str = "https://master.alterware.dev";
const REPO: &str = "mxve/alterware-launcher";

fn get_file_sha1(path: &PathBuf) -> String {
    let mut sha1 = sha1_smol::Sha1::new();
    sha1.update(&fs::read(path).unwrap());
    sha1.digest().to_string()
}

fn get_input() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn self_update_available() -> bool {
    let current_version: Version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let github_body = http::get_body_string(
        format!("https://api.github.com/repos/{}/releases/latest", REPO).as_str(),
    );
    let github_json: serde_json::Value = serde_json::from_str(&github_body).unwrap();
    let latest_version = github_json["tag_name"]
        .to_string()
        .replace(['v', '"'].as_ref(), "");
    let latest_version = Version::parse(&latest_version).unwrap();

    current_version < latest_version
}

#[cfg(not(windows))]
fn self_update(_update_only: bool) {
    if self_update_available() {
        println!("A new version of the AlterWare launcher is available.");
        println!("Download it at https://github.com/{}/releases/latest", REPO);
        println!("Launching in 10 seconds..");
        thread::sleep(time::Duration::from_secs(10));
    }
}

#[cfg(windows)]
fn self_update(update_only: bool) {
    let working_dir = std::env::current_dir().unwrap();
    let files = fs::read_dir(&working_dir).unwrap();

    for file in files {
        let file = file.unwrap();
        let file_name = file.file_name().into_string().unwrap();

        if file_name.contains("alterware-launcher")
            && (file_name.contains(".__relocated__.exe")
                || file_name.contains(".__selfdelete__.exe"))
        {
            fs::remove_file(file.path()).unwrap();
        }
    }

    if self_update_available() {
        println!("Performing launcher self-update.");
        println!("If you run into any issues, please download the latest version at https://github.com/{}/releases/latest", REPO);

        let update_binary = PathBuf::from("alterware-launcher-update.exe");
        let file_path = working_dir.join(&update_binary);

        if update_binary.exists() {
            fs::remove_file(&update_binary).unwrap();
        }

        http::download_file(
            &format!(
                "https://github.com/{}/releases/latest/download/alterware-launcher.exe",
                REPO
            ),
            &file_path,
        );

        if !file_path.exists() {
            println!("Failed to download launcher update.");
            return;
        }

        self_replace::self_replace("alterware-launcher-update.exe").unwrap();
        fs::remove_file(&file_path).unwrap();
        println!("Launcher updated. Please run it again.");
        if !update_only {
            std::io::stdin().read_line(&mut String::new()).unwrap();
        }
        std::process::exit(201);
    }
}

#[cfg(windows)]
fn get_installed_games(games: &Vec<Game>) -> Vec<(u32, PathBuf)> {
    let mut installed_games = Vec::new();
    let mut steamdir = match SteamDir::locate() {
        Some(steamdir) => steamdir,
        None => {
            println!("Steam not found.");
            return installed_games;
        }
    };

    for game in games {
        if let Some(app) = steamdir.app(&game.app_id) {
            installed_games.push((game.app_id, PathBuf::from(&app.path)));
        }
    }

    installed_games
}

#[cfg(windows)]
fn setup_client_links(game: &Game, game_dir: &Path) {
    if game.client.len() > 1 {
        println!("Multiple clients installed, use the shortcuts (launch-<client>.lnk in the game directory or desktop shortcuts) to launch a specific client.");
    }

    let target = game_dir.join("alterware-launcher.exe");

    for c in game.client.iter() {
        let lnk = game_dir.join(format!("launch-{}.lnk", c));

        let mut sl = ShellLink::new(target.clone()).unwrap();
        sl.set_arguments(Some(c.to_string()));
        sl.set_icon_location(Some(
            game_dir
                .join(format!("{}.exe", c))
                .to_string_lossy()
                .into_owned(),
        ));
        sl.create_lnk(&lnk).unwrap();
    }
}

#[cfg(windows)]
fn windows_launcher_install(games: &Vec<Game>) {
    println!("No game specified/found. Checking for installed Steam games..");
    let installed_games = get_installed_games(games);

    if !installed_games.is_empty() {
        // if current directory is in the steamapps/common folder of a game, use that game
        let current_dir = std::env::current_dir().unwrap();
        for (id, path) in installed_games.iter() {
            if current_dir.starts_with(path) {
                println!("Found game in current directory.");
                println!("Installing AlterWare client for {}.", id);
                let game = games.iter().find(|&g| g.app_id == *id).unwrap();
                setup_client_links(game, path);
                update(game, path);
                println!("Installation complete. Please run the launcher again or use a shortcut to launch the game.");
                std::io::stdin().read_line(&mut String::new()).unwrap();
                std::process::exit(0);
            }
        }

        println!("Installed games:");

        for (id, path) in installed_games.iter() {
            println!("{}: {}", id, path.display());
        }

        println!("Enter the ID of the game you want to install the AlterWare client for, enter 0 for manual selection:");
        let input: u32 = get_input().parse().unwrap();

        if input == 0 {
            return manual_install(games);
        }

        for (id, path) in installed_games.iter() {
            if *id == input {
                let game = games.iter().find(|&g| g.app_id == input).unwrap();

                let launcher_path = std::env::current_exe().unwrap();
                let target_path = path.join("alterware-launcher.exe");

                if launcher_path != target_path {
                    fs::copy(launcher_path, target_path).unwrap();
                    println!("Launcher copied to {}", path.display());
                }
                setup_client_links(game, path);

                println!("Create Desktop shortcut? (Y/n)");
                let input = get_input().to_ascii_lowercase();

                if input == "y" || input.is_empty() {
                    let desktop = PathBuf::from(&format!(
                        "{}\\Desktop",
                        std::env::var("USERPROFILE").unwrap()
                    ));

                    let target = path.join("alterware-launcher.exe");

                    for c in game.client.iter() {
                        let lnk = desktop.join(format!("{}.lnk", c));

                        let mut sl = ShellLink::new(target.clone()).unwrap();
                        sl.set_arguments(Some(c.to_string()));
                        sl.set_icon_location(Some(
                            path.join(format!("{}.exe", c))
                                .to_string_lossy()
                                .into_owned(),
                        ));
                        sl.create_lnk(lnk).unwrap();
                    }
                }

                update(game, path);
                println!("Installation complete. Please run the launcher again or use a shortcut to launch the game.");
                std::io::stdin().read_line(&mut String::new()).unwrap();
                break;
            }
        }
        std::process::exit(0);
    } else {
        manual_install(games);
    }
}

fn prompt_client_selection(games: &[Game]) -> String {
    println!(
        "Couldn't detect any games, please select a client to install in the current directory:"
    );
    for (i, g) in games.iter().enumerate() {
        for c in g.client.iter() {
            println!("{}: {}", i, c);
        }
    }
    let input: usize = get_input().parse().unwrap();
    String::from(games[input].client[0])
}

fn manual_install(games: &[Game]) {
    let selection = prompt_client_selection(games);
    let game = games.iter().find(|&g| g.client[0] == selection).unwrap();
    update(game, &std::env::current_dir().unwrap());
    println!("Installation complete. Please run the launcher again or use a shortcut to launch the game.");
    std::io::stdin().read_line(&mut String::new()).unwrap();
    std::process::exit(0);
}

fn update(game: &Game, dir: &Path) {
    let cdn_info: Vec<CdnFile> = serde_json::from_str(&http::get_body_string(
        format!("{}/files.json", MASTER).as_str(),
    ))
    .unwrap();

    for file in cdn_info {
        if !file.name.starts_with(game.engine) {
            continue;
        }

        let file_path = dir.join(&file.name.replace(&format!("{}/", game.engine), ""));
        if file_path.exists() {
            let sha1_local = get_file_sha1(&file_path).to_lowercase();
            let sha1_remote = file.hash.to_lowercase();
            if sha1_local != sha1_remote {
                println!(
                    "Updating {}...\nLocal hash: {}\nRemote hash: {}",
                    file_path.display(),
                    sha1_local,
                    sha1_remote
                );
                http::download_file(&format!("{}/{}", MASTER, file.name), &file_path);
            }
        } else {
            println!("Downloading {}...", file_path.display());
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).unwrap();
                }
            }
            http::download_file(&format!("{}/{}", MASTER, file.name), &file_path);
        }
    }
}

fn launch(file_path: &PathBuf) {
    println!("Launching {}...", file_path.display());
    std::process::Command::new(file_path)
        .spawn()
        .expect("Failed to launch the game")
        .wait()
        .expect("Failed to wait for the game process to finish");
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();

    let mut update_only = false;
    if args.contains(&String::from("update")) {
        update_only = true;
        args.iter()
            .position(|r| r == "update")
            .map(|e| args.remove(e));
    }

    if !args.contains(&String::from("skip-launcher-update")) {
        self_update(update_only);
    } else {
        args.iter()
            .position(|r| r == "skip-launcher-update")
            .map(|e| args.remove(e));
    }

    let games_json = http::get_body_string(format!("{}/games.json", MASTER).as_str());
    let games: Vec<Game> = serde_json::from_str(&games_json).unwrap();

    let mut game: String = String::new();
    if args.len() > 1 {
        game = String::from(&args[1]);
    } else {
        'main: for g in games.iter() {
            for r in g.references.iter() {
                if std::path::Path::new(r).exists() {
                    if g.client.len() > 1 {
                        if update_only {
                            game = String::from(g.client[0]);
                            break 'main;
                        }

                        #[cfg(windows)]
                        setup_client_links(g, &std::env::current_dir().unwrap());

                        #[cfg(not(windows))]
                        println!("Multiple clients installed, set the client as the first argument to launch a specific client.");

                        for (i, c) in g.client.iter().enumerate() {
                            println!("{}: {}", i, c);
                        }
                        game = String::from(g.client[get_input().parse::<usize>().unwrap()]);
                        break 'main;
                    }
                    game = String::from(g.client[0]);
                    break 'main;
                }
            }
        }
    }

    for g in games.iter() {
        for c in g.client.iter() {
            if c == &game {
                update(g, &std::env::current_dir().unwrap());
                if !update_only {
                    launch(&PathBuf::from(format!("{}.exe", c)));
                }
                return;
            }
        }
    }

    #[cfg(windows)]
    windows_launcher_install(&games);

    #[cfg(not(windows))]
    manual_install(&games);

    println!("Game not found!");
    println!("Place the launcher in the game folder, if that doesn't work specify the client on the command line (ex. alterware-launcher.exe iw4-sp)");
    println!("Press enter to exit...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
