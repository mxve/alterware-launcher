mod config;
mod github;
mod global;
mod http;
mod iw4x;
mod misc;
mod self_update;
mod structs;

use global::*;
use structs::*;

use colored::*;
#[cfg(windows)]
use mslnk::ShellLink;
use std::{borrow::Cow, collections::HashMap, fs, path::Path, path::PathBuf};
#[cfg(windows)]
use steamlocate::SteamDir;

#[cfg(windows)]
fn get_installed_games(games: &Vec<Game>) -> Vec<(u32, PathBuf)> {
    let mut installed_games = Vec::new();
    let mut steamdir = match SteamDir::locate() {
        Some(steamdir) => steamdir,
        None => {
            println!("{}", "Steam not found!".yellow());
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
        println!("Multiple clients installed, use the shortcuts (launch-<client>.lnk in the game directory or on the desktop) to launch a specific client.");
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
fn setup_desktop_links(path: &Path, game: &Game) {
    println!("Create Desktop shortcut? (Y/n)");
    let input = misc::stdin().to_ascii_lowercase();

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
}

#[cfg(windows)]
fn auto_install(path: &Path, game: &Game) {
    setup_client_links(game, path);
    setup_desktop_links(path, game);
    update(game, path, false, false);
}

#[cfg(windows)]
fn windows_launcher_install(games: &Vec<Game>) {
    println!(
        "{}",
        "No game specified/found. Checking for installed Steam games..".yellow()
    );
    let installed_games = get_installed_games(games);

    if !installed_games.is_empty() {
        let current_dir = std::env::current_dir().unwrap();
        for (id, path) in installed_games.iter() {
            if current_dir.starts_with(path) {
                println!("Found game in current directory.");
                println!("Installing AlterWare client for {}.", id);
                let game = games.iter().find(|&g| g.app_id == *id).unwrap();
                auto_install(path, game);
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
        let input: u32 = misc::stdin().parse().unwrap();

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
                auto_install(path, game);
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
    let input: usize = misc::stdin().parse().unwrap();
    String::from(games[input].client[0])
}

fn manual_install(games: &[Game]) {
    let selection = prompt_client_selection(games);
    let game = games.iter().find(|&g| g.client[0] == selection).unwrap();
    update(game, &std::env::current_dir().unwrap(), false, false);
    println!("Installation complete. Please run the launcher again or use a shortcut to launch the game.");
    std::io::stdin().read_line(&mut String::new()).unwrap();
    std::process::exit(0);
}

fn update_dir(
    cdn_info: &Vec<CdnFile>,
    remote_dir: &str,
    dir: &Path,
    hashes: &mut HashMap<String, String>,
) {
    let remote_dir = format!("{}/", remote_dir);

    for file in cdn_info {
        if !file.name.starts_with(&remote_dir) || file.name == "iw4/iw4x.dll" {
            continue;
        }

        let sha1_remote = file.hash.to_lowercase();
        let file_name = &file.name.replace(remote_dir.as_str(), "");
        let file_path = dir.join(file_name);
        if file_path.exists() {
            let sha1_local = hashes
                .get(file_name)
                .map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Owned(misc::get_file_sha1(&file_path)))
                .to_string();

            if sha1_local != sha1_remote {
                println!(
                    "[{}]    {}",
                    "Updating".bright_yellow(),
                    file_path.display()
                );
                http::download_file(&format!("{}/{}", MASTER, file.name), &file_path);
            } else {
                println!("[{}]     {}", "Checked".bright_blue(), file_path.display());
            }
            hashes.insert(file_name.to_owned(), sha1_remote.to_owned());
        } else {
            println!(
                "[{}] {}",
                "Downloading".bright_yellow(),
                file_path.display()
            );
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).unwrap();
                }
            }
            http::download_file(&format!("{}/{}", MASTER, file.name), &file_path);
            hashes.insert(file_name.to_owned(), sha1_remote.to_owned());
        }
    }
}

fn update(game: &Game, dir: &Path, bonus_content: bool, force: bool) {
    let cdn_info: Vec<CdnFile> = serde_json::from_str(&http::get_body_string(
        format!("{}/files.json", MASTER).as_str(),
    ))
    .unwrap();

    let mut hashes = HashMap::new();
    let hash_file = dir.join(".sha-sums");
    if hash_file.exists() && !force {
        let hash_file = fs::read_to_string(hash_file).unwrap();
        for line in hash_file.lines() {
            let mut split = line.split_whitespace();
            let hash = split.next().unwrap();
            let file = split.next().unwrap();
            hashes.insert(file.to_owned(), hash.to_owned());
        }
    }

    update_dir(&cdn_info, game.engine, dir, &mut hashes);

    if game.engine == "iw4" {
        iw4x::update(dir);
    }

    if bonus_content && !game.bonus.is_empty() {
        for bonus in game.bonus.iter() {
            update_dir(&cdn_info, bonus, dir, &mut hashes);
        }
    }

    let mut hash_file_content = String::new();
    for (file, hash) in hashes.iter() {
        hash_file_content.push_str(&format!("{} {}\n", hash, file));
    }
    fs::write(dir.join(".sha-sums"), hash_file_content).unwrap();
}

fn launch(file_path: &PathBuf) {
    println!("Launching {}...", file_path.display());
    std::process::Command::new(file_path)
        .spawn()
        .expect("Failed to launch the game")
        .wait()
        .expect("Failed to wait for the game process to finish");
}

#[cfg(windows)]
fn setup_env() {
    colored::control::set_virtual_terminal(true).unwrap_or_else(|error| {
        println!("{:#?}", error);
        colored::control::SHOULD_COLORIZE.set_override(false);
    });
}

fn arg_value(args: &[String], arg: &str) -> Option<String> {
    let val = args
        .iter()
        .position(|r| r == arg)
        .map(|e| args[e + 1].clone());
    if let Some(ref val) = val {
        if val.starts_with('-') {
            return None;
        }
    }
    val
}

fn arg_bool(args: &[String], arg: &str) -> bool {
    args.iter().any(|r| r == arg)
}

fn arg_remove(args: &mut Vec<String>, arg: &str) {
    args.iter().position(|r| r == arg).map(|e| args.remove(e));
}

fn main() {
    #[cfg(windows)]
    setup_env();

    let mut args: Vec<String> = std::env::args().collect();
    let mut cfg = config::load(PathBuf::from("alterware-launcher.json"));

    if !arg_bool(&args, "--skip-launcher-update") && !cfg.skip_self_update {
        self_update::run(cfg.update_only);
    } else {
        arg_remove(&mut args, "--skip-launcher-update");
    }

    if arg_bool(&args, "--update") || arg_bool(&args, "-u") {
        cfg.update_only = true;
        arg_remove(&mut args, "--update");
        arg_remove(&mut args, "-u");
    }

    if arg_bool(&args, "--bonus") {
        cfg.download_bonus_content = true;
        arg_remove(&mut args, "--bonus");
    }

    if arg_bool(&args, "--force") || arg_bool(&args, "-f") {
        cfg.force_update = true;
        arg_remove(&mut args, "--force");
        arg_remove(&mut args, "-f");
    }

    let install_path: PathBuf;
    if let Some(path) = arg_value(&args, "--path") {
        install_path = PathBuf::from(path);
        arg_remove(&mut args, "--path");
    } else if let Some(path) = arg_value(&args, "-p") {
        install_path = PathBuf::from(path);
        arg_remove(&mut args, "-p");
    } else {
        install_path = std::env::current_dir().unwrap();
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
                        if cfg.update_only {
                            game = String::from(g.client[0]);
                            break 'main;
                        }

                        #[cfg(windows)]
                        setup_client_links(g, &std::env::current_dir().unwrap());

                        #[cfg(not(windows))]
                        println!("Multiple clients installed, set the client as the first argument to launch a specific client.");
                        println!("Select a client to launch:");
                        for (i, c) in g.client.iter().enumerate() {
                            println!("{}: {}", i, c);
                        }
                        game = String::from(g.client[misc::stdin().parse::<usize>().unwrap()]);
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
                if cfg.ask_bonus_content && !g.bonus.is_empty() {
                    println!("Download bonus content? (Y/n)");
                    let input = misc::stdin().to_ascii_lowercase();
                    cfg.download_bonus_content = input != "n";
                    config::save_value(
                        install_path.join("alterware-launcher.json"),
                        "download_bonus_content",
                        cfg.download_bonus_content,
                    );
                    config::save_value(
                        install_path.join("alterware-launcher.json"),
                        "ask_bonus_content",
                        false,
                    );
                }

                update(
                    g,
                    install_path.as_path(),
                    cfg.download_bonus_content,
                    cfg.force_update,
                );
                if !cfg.update_only {
                    launch(&install_path.join(format!("{}.exe", c)));
                }
                return;
            }
        }
    }

    #[cfg(windows)]
    windows_launcher_install(&games);

    #[cfg(not(windows))]
    manual_install(&games);

    println!("{}", "Game not found!".bright_red());
    println!("Place the launcher in the game folder, if that doesn't work specify the client on the command line (ex. alterware-launcher.exe iw4-sp)");
    println!("Press enter to exit...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
