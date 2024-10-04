mod config;
mod extend;
mod github;
mod global;
mod http_async;
mod iw4x;
mod misc;
mod self_update;
mod structs;

#[cfg(test)]
mod tests;

use extend::*;
use global::*;
use structs::*;

#[macro_use]
extern crate simple_log;

use colored::Colorize;
use indicatif::ProgressBar;
#[cfg(windows)]
use mslnk::ShellLink;
use simple_log::LogConfigBuilder;
use std::{
    borrow::Cow,
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};
#[cfg(windows)]
use steamlocate::SteamDir;

#[cfg(windows)]
fn get_installed_games(games: &Vec<Game>) -> Vec<(u32, PathBuf)> {
    let mut installed_games = Vec::new();
    let steamdir_result = SteamDir::locate();

    let steamdir = match steamdir_result {
        Ok(steamdir) => steamdir,
        Err(error) => {
            crate::println_error!("Error locating Steam: {error}");
            return installed_games;
        }
    };

    for game in games {
        if let Ok(Some((app, library))) = steamdir.find_app(game.app_id) {
            let game_path = library
                .path()
                .join("steamapps")
                .join("common")
                .join(&app.install_dir);
            installed_games.push((game.app_id, game_path));
        }
    }

    installed_games
}

#[cfg(windows)]
fn create_shortcut(path: &Path, target: &Path, icon: String, args: String) {
    if let Ok(mut sl) = ShellLink::new(target) {
        sl.set_arguments(Some(args));
        sl.set_icon_location(Some(icon));
        sl.create_lnk(path).unwrap_or_else(|error| {
            crate::println_error!("Error creating shortcut.\n{error}");
        });
    } else {
        crate::println_error!("Error creating shortcut.");
    }
}

#[cfg(windows)]
fn setup_client_links(game: &Game, game_dir: &Path) {
    if game.client.len() > 1 {
        println!("Multiple clients installed, use the shortcuts (launch-<client>.lnk in the game directory or on the desktop) to launch a specific client.");
    }

    for c in game.client.iter() {
        create_shortcut(
            &game_dir.join(format!("launch-{c}.lnk")),
            &game_dir.join("alterware-launcher.exe"),
            game_dir
                .join(format!("{c}.exe"))
                .to_string_lossy()
                .into_owned(),
            c.to_string(),
        );
    }
}

#[cfg(windows)]
fn setup_desktop_links(path: &Path, game: &Game) {
    println!("Create Desktop shortcut? (Y/n)");
    let input = misc::stdin().to_ascii_lowercase();

    if input != "n" {
        let desktop = PathBuf::from(&format!("{}\\Desktop", env::var("USERPROFILE").unwrap()));

        for c in game.client.iter() {
            create_shortcut(
                &desktop.join(format!("{c}.lnk")),
                &path.join("alterware-launcher.exe"),
                path.join(format!("{c}.exe")).to_string_lossy().into_owned(),
                c.to_string(),
            );
        }
    }
}

#[cfg(windows)]
async fn auto_install(path: &Path, game: &Game<'_>) {
    setup_client_links(game, path);
    setup_desktop_links(path, game);
    update(game, path, false, false, None, None).await;
}

#[cfg(windows)]
async fn windows_launcher_install(games: &Vec<Game<'_>>) {
    crate::println_info!(
        "{}",
        "No game specified/found. Checking for installed Steam games..".yellow()
    );
    let installed_games = get_installed_games(games);

    if !installed_games.is_empty() {
        let current_dir = env::current_dir().unwrap();
        for (id, path) in installed_games.iter() {
            if current_dir.starts_with(path) {
                crate::println_info!("Found game in current directory.");
                crate::println_info!("Installing AlterWare client for {}.", id);
                let game = games.iter().find(|&g| g.app_id == *id).unwrap();
                auto_install(path, game).await;
                crate::println_info!("Installation complete. Please run the launcher again or use a shortcut to launch the game.");
                std::io::stdin().read_line(&mut String::new()).unwrap();
                std::process::exit(0);
            }
        }

        println!("Installed games:");

        for (id, path) in installed_games.iter() {
            println!("{id}: {}", path.display());
        }

        println!("Enter the ID of the game you want to install the AlterWare client for:");
        let input: u32 = misc::stdin().parse().unwrap();

        for (id, path) in installed_games.iter() {
            if *id == input {
                let game = games.iter().find(|&g| g.app_id == input).unwrap();

                let launcher_path = env::current_exe().unwrap();
                let target_path = path.join("alterware-launcher.exe");

                if launcher_path != target_path {
                    fs::copy(launcher_path, &target_path).unwrap();
                    crate::println_info!("Launcher copied to {}", path.display());
                }
                auto_install(path, game).await;
                crate::println_info!("Installation complete.");
                crate::println_info!("Please use one of the shortcuts (on your Desktop or in the game folder) to play.");
                crate::println_info!(
                    "Alternatively run the launcher again from the game folder {}",
                    target_path.display()
                );
                std::io::stdin().read_line(&mut String::new()).unwrap();
                break;
            }
        }
        std::process::exit(0);
    } else {
        println!(
            "No installed games found. Make sure to place the launcher in the game directory."
        );
        std::io::stdin().read_line(&mut String::new()).unwrap();
        std::process::exit(0);
    }
}

fn total_download_size(cdn_info: &Vec<CdnFile>, remote_dir: &str) -> u64 {
    let remote_dir = format!("{remote_dir}/");
    let mut size: u64 = 0;
    for file in cdn_info {
        if !file.name.starts_with(&remote_dir) || file.name == "iw4/iw4x.dll" {
            continue;
        }
        size += file.size as u64;
    }
    size
}

async fn update_dir(
    cdn_info: &Vec<CdnFile>,
    remote_dir: &str,
    dir: &Path,
    hashes: &mut HashMap<String, String>,
    pb: &ProgressBar,
    skip_iw4x_sp: bool,
) {
    misc::pb_style_download(pb, false);

    let remote_dir_pre = format!("{remote_dir}/");

    let mut files_to_download: Vec<CdnFile> = vec![];

    for file in cdn_info {
        if !file.name.starts_with(&remote_dir_pre) || file.name == "iw4/iw4x.dll" {
            continue;
        }
        if skip_iw4x_sp && file.name == "iw4/iw4x-sp.exe" {
            continue;
        }

        let hash_remote = file.blake3.to_lowercase();
        let file_name = &file.name.replace(remote_dir_pre.as_str(), "");
        let file_path = dir.join(file_name);
        if file_path.exists() {
            let hash_local = hashes
                .get(file_name)
                .map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Owned(file_path.get_blake3().unwrap()))
                .to_string();

            if hash_local != hash_remote {
                files_to_download.push(file.clone());
            } else {
                let msg = format!("{}{}", misc::prefix("checked"), file_path.cute_path());
                pb.println(&msg);
                info!("{msg}");
                hashes.insert(file_name.to_owned(), file.blake3.to_lowercase());
            }
        } else {
            files_to_download.push(file.clone());
        }
    }

    if files_to_download.is_empty() {
        let msg = format!(
            "{}No files to download for {}",
            misc::prefix("info"),
            remote_dir
        );
        pb.println(&msg);
        info!("{msg}");
        return;
    }
    let msg = format!(
        "{}Downloading outdated or missing files for {remote_dir}, {}",
        misc::prefix("info"),
        misc::human_readable_bytes(total_download_size(&files_to_download, remote_dir))
    );
    pb.println(&msg);
    info!("{msg}");

    misc::pb_style_download(pb, true);
    let client = reqwest::Client::new();
    for file in files_to_download {
        let file_name = &file.name.replace(&remote_dir_pre, "");
        let file_path = dir.join(file_name);
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).unwrap();
            }
        }

        // Prompt user to retry downloads if they fail
        let mut download_complete = false;
        let mut bust_cache = false;
        let mut local_hash = String::default();
        while !download_complete {
            let url = format!("{}/{}", MASTER.lock().unwrap(), file.name);
            let url = if bust_cache {
                bust_cache = false;
                format!("{}?{}", url, misc::random_string(6))
            } else {
                url
            };

            if let Err(err) =
                http_async::download_file_progress(&client, pb, &url, &file_path, file.size as u64)
                    .await
            {
                let file_name = file_path.clone().cute_path();
                println_error!("{err}");
                println!("Failed to download file {file_name}, retry? (Y/n)");
                let input = misc::stdin().to_ascii_lowercase();
                if input == "n" {
                    error!("Download for file {file_name} failed with {err}");
                    panic!("{err}");
                } else {
                    warn!(
                        "Download for file {file_name} failed with {err} user chose to retry download"
                    );
                }
            };

            local_hash = file_path.get_blake3().unwrap().to_lowercase();
            let remote = file.blake3.to_lowercase();
            if local_hash != remote {
                println_error!("Downloaded file hash does not match remote!\nRemote {remote}, local {local_hash}, {}\nIf this issue persists please try again in 15 minutes.", file_path.cute_path());
                println!("Retry download? (Y/n)");
                let input = misc::stdin().to_ascii_lowercase();
                if input != "n" {
                    println_info!(
                        "Retrying download for {} due to hash mismatch",
                        file_path.cute_path()
                    );
                    bust_cache = true;
                    continue;
                }
            }

            download_complete = true;
        }

        hashes.insert(file_name.to_owned(), local_hash);

        #[cfg(unix)]
        if file_name.ends_with(".exe") {
            let perms = std::os::unix::fs::PermissionsExt::from_mode(0o755);
            fs::set_permissions(&file_path, perms).unwrap_or_else(|error| {
                crate::println_error!("Error setting permissions for {file_name}: {error}");
            })
        }
    }
    misc::pb_style_download(pb, false);
}

async fn update(
    game: &Game<'_>,
    dir: &Path,
    bonus_content: bool,
    force: bool,
    skip_iw4x_sp: Option<bool>,
    ignore_required_files: Option<bool>,
) {
    let skip_iw4x_sp = skip_iw4x_sp.unwrap_or(false);
    let ignore_required_files = ignore_required_files.unwrap_or(false);

    let res =
        http_async::get_body_string(format!("{}/files.json", MASTER.lock().unwrap()).as_str())
            .await
            .unwrap();
    let cdn_info: Vec<CdnFile> = serde_json::from_str(&res).unwrap();

    if !ignore_required_files && !game.required_files_exist(dir) {
        println!(
            "{}\nVerify game file integrity on Steam or reinstall the game.",
            "Critical game files missing.".bright_red()
        );
        std::io::stdin().read_line(&mut String::new()).unwrap();
        std::process::exit(0);
    }

    let old_files = [".sha-sums", ".iw4xrevision"];
    for f in old_files {
        if dir.join(f).exists() {
            match fs::remove_file(dir.join(f)) {
                Ok(_) => {}
                Err(error) => {
                    crate::println_error!("Error removing {f}: {error}");
                }
            }
        }
    }

    let mut cache = if force {
        structs::Cache::default()
    } else {
        misc::get_cache(dir)
    };

    if game.engine == "iw4" {
        iw4x::update(dir, &mut cache).await;

        let iw4x_dirs = vec!["iw4x", "zone/patch"];
        for d in &iw4x_dirs {
            if let Ok(dir_iter) = dir.join(d).read_dir() {
                'outer: for file in dir_iter.filter_map(|entry| entry.ok()) {
                    let file_path = file.path();

                    if file_path.is_dir() {
                        continue;
                    }

                    let file_path_rel = match file_path.strip_prefix(dir) {
                        Ok(rel) => rel.to_path_buf(),
                        Err(_) => continue,
                    };

                    if iw4x_dirs
                        .iter()
                        .any(|prefix| file_path_rel.starts_with(Path::new(prefix)))
                    {
                        if !cdn_info
                            .iter()
                            .any(|cdn_file| cdn_file.name.starts_with("iw4"))
                        {
                            continue;
                        }

                        let should_continue = cdn_info.iter().any(|cdn_file| {
                            let path_rem = Path::new(&cdn_file.name)
                                .strip_prefix(Path::new("iw4"))
                                .unwrap_or_else(|_| Path::new(&cdn_file.name));
                            path_rem == file_path_rel
                        });

                        if should_continue {
                            continue 'outer;
                        }

                        crate::println_info!(
                            "{}{}",
                            misc::prefix("removed"),
                            file_path.cute_path()
                        );

                        if fs::remove_file(&file_path).is_err() {
                            crate::println_error!(
                                "{}Couldn't delete {}",
                                misc::prefix("error"),
                                file_path.cute_path()
                            );
                        }
                    }
                }
            }
        }
    }

    let pb = ProgressBar::new(0);
    update_dir(
        &cdn_info,
        game.engine,
        dir,
        &mut cache.hashes,
        &pb,
        skip_iw4x_sp,
    )
    .await;

    if bonus_content && !game.bonus.is_empty() {
        for bonus in game.bonus.iter() {
            update_dir(&cdn_info, bonus, dir, &mut cache.hashes, &pb, skip_iw4x_sp).await;
        }
    }

    pb.finish();

    for f in game.delete.iter() {
        let file_path = dir.join(f);
        if file_path.is_file() {
            if fs::remove_file(&file_path).is_err() {
                println!(
                    "{}Couldn't delete {}",
                    misc::prefix("error"),
                    file_path.cute_path()
                );
            } else {
                println!("{}{}", misc::prefix("removed"), file_path.cute_path());
            }
        } else if file_path.is_dir() {
            if fs::remove_dir_all(&file_path).is_err() {
                println!(
                    "{}Couldn't delete {}",
                    misc::prefix("error"),
                    file_path.cute_path()
                );
            } else {
                println!("{}{}", misc::prefix("removed"), file_path.cute_path());
            }
        }
    }

    misc::save_cache(dir, cache);
}

#[cfg(windows)]
fn launch(file_path: &PathBuf, args: &str) {
    println!("\n\nJoin the AlterWare Discord server:\nhttps://discord.gg/2ETE8engZM\n\n");
    crate::println_info!("Launching {} {args}", file_path.display());
    let exit_status = std::process::Command::new(file_path)
        .args(args.trim().split(' '))
        .current_dir(file_path.parent().unwrap())
        .spawn()
        .expect("Failed to launch the game")
        .wait()
        .expect("Failed to wait for the game process to finish");

    crate::println_error!("Game exited with {exit_status}");
    if !exit_status.success() {
        misc::stdin();
    }
}

#[cfg(unix)]
fn launch(file_path: &PathBuf, args: &str) {
    println!("\n\nJoin the AlterWare Discord server:\nhttps://discord.gg/2ETE8engZM\n\n");
    crate::println_info!("Launching {} {args}", file_path.display());
    let exit_status = if misc::is_program_in_path("wine") {
        println!("Found wine, launching game using wine.\nIf you run into issues or want to launch a different way, run {} manually.", file_path.display());
        std::process::Command::new("wine")
            .args([file_path.to_str().unwrap(), args.trim()])
            .current_dir(file_path.parent().unwrap())
            .spawn()
            .expect("Failed to launch the game")
            .wait()
            .expect("Failed to wait for the game process to finish")
    } else {
        std::process::Command::new(file_path)
            .args(args.trim().split(' '))
            .current_dir(file_path.parent().unwrap())
            .spawn()
            .expect("Failed to launch the game")
            .wait()
            .expect("Failed to wait for the game process to finish")
    };

    crate::println_error!("Game exited with {exit_status}");
    if !exit_status.success() {
        misc::stdin();
    }
}

#[cfg(windows)]
fn setup_env() {
    colored::control::set_virtual_terminal(true).unwrap_or_else(|error| {
        crate::println_error!("{:#?}", error);
        colored::control::SHOULD_COLORIZE.set_override(false);
    });

    if let Ok(system_root) = env::var("SystemRoot") {
        if let Ok(current_dir) = env::current_dir() {
            if current_dir.starts_with(system_root) {
                if let Ok(current_exe) = env::current_exe() {
                    if let Some(parent) = current_exe.parent() {
                        if let Err(error) = env::set_current_dir(parent) {
                            crate::println_error!("{:#?}", error);
                        } else {
                            crate::println_info!("Running from the system directory. Changed working directory to the executable location.");
                        }
                    }
                }
            }
        }
    }
}

fn arg_value(args: &[String], arg: &str) -> Option<String> {
    if let Some(e) = args.iter().position(|r| r == arg) {
        if e + 1 < args.len() {
            return Some(args[e + 1].clone());
        }
    }
    None
}

fn arg_bool(args: &[String], arg: &str) -> bool {
    args.iter().any(|r| r == arg)
}

fn arg_remove(args: &mut Vec<String>, arg: &str) {
    args.iter().position(|r| r == arg).map(|e| args.remove(e));
}

fn arg_remove_value(args: &mut Vec<String>, arg: &str) {
    if let Some(e) = args.iter().position(|r| r == arg) {
        args.remove(e);
        args.remove(e);
    };
}

#[tokio::main]
async fn main() {
    #[cfg(windows)]
    let log_file = env::current_exe()
        .unwrap_or(PathBuf::from("alterware-launcher"))
        .with_extension("log");
    #[cfg(unix)]
    let log_file = PathBuf::from("/var/log/alterware-launcher.log");

    if log_file.exists() && fs::remove_file(&log_file).is_err() {
        println!("Couldn't clear log file, make sure target directory is writable.");
    }
    let logger_config = LogConfigBuilder::builder()
        .path(log_file.to_str().unwrap())
        .time_format("%Y-%m-%d %H:%M:%S.%f")
        .level("debug").unwrap()
        .output_file()
        .build();
    let _ = simple_log::new(logger_config);

    #[cfg(windows)]
    setup_env();

    let mut args: Vec<String> = env::args().collect();

    if arg_bool(&args, "--help") {
        println!("CLI Args:");
        println!("    <client>: Specify the client to launch");
        println!("    --help: Display this help message");
        println!("    --version: Display the launcher version");
        println!("    --path/-p <path>: Specify the game directory");
        println!("    --update/-u: Update only, don't launch the game");
        println!("    --bonus: Download bonus content");
        println!("    --skip-bonus: Don't download bonus content");
        println!("    --force/-f: Force file hash recheck");
        println!("    --pass <args>: Pass arguments to the game");
        println!("    --skip-launcher-update: Skip launcher self-update");
        println!("    --ignore-required-files: Skip required files check");
        println!("    --skip-redist: Skip redistributable installation");
        println!("    --redist: (Re-)Install redistributables");
        println!(
            "\nExample:\n    alterware-launcher.exe iw4x --bonus --pass \"-console -nointro\""
        );
        return;
    }

    if arg_bool(&args, "--version") || arg_bool(&args, "-v") {
        println!(
            "{} v{}",
            "AlterWare Launcher".bright_green(),
            env!("CARGO_PKG_VERSION")
        );
        println!("https://github.com/{GH_OWNER}/{GH_REPO}");
        println!(
            "\n{}{}{}{}{}{}{}",
            "For ".on_black(),
            "Alter".bright_blue().on_black().underline(),
            "Ware".white().on_black().underline(),
            ".dev".on_black().underline(),
            " by ".on_black(),
            "mxve".bright_magenta().on_black().underline(),
            ".de".on_black().underline()
        );
        return;
    }

    let install_path: PathBuf;
    if let Some(path) = arg_value(&args, "--path") {
        install_path = PathBuf::from(path);
        arg_remove_value(&mut args, "--path");
    } else if let Some(path) = arg_value(&args, "-p") {
        install_path = PathBuf::from(path);
        arg_remove_value(&mut args, "-p");
    } else {
        install_path = env::current_dir().unwrap();
    }

    let mut cfg = config::load(install_path.join("alterware-launcher.json"));

    if !cfg.use_https {
        let mut master_url = MASTER.lock().unwrap();
        *master_url = master_url.replace("https://", "http://");
    };

    if !arg_bool(&args, "--skip-launcher-update") && !cfg.skip_self_update {
        self_update::run(cfg.update_only).await;
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
        cfg.ask_bonus_content = false;
        arg_remove(&mut args, "--bonus");
    } else if arg_bool(&args, "--skip-bonus") {
        cfg.download_bonus_content = false;
        cfg.ask_bonus_content = false;
        arg_remove(&mut args, "--skip-bonus")
    }

    if arg_bool(&args, "--force") || arg_bool(&args, "-f") {
        cfg.force_update = true;
        arg_remove(&mut args, "--force");
        arg_remove(&mut args, "-f");
    }

    let ignore_required_files = arg_bool(&args, "--ignore-required-files");
    if ignore_required_files {
        arg_remove(&mut args, "--ignore-required-files");
    }

    if let Some(pass) = arg_value(&args, "--pass") {
        cfg.args = pass;
        arg_remove_value(&mut args, "--pass");
    } else if cfg.args.is_empty() {
        cfg.args = String::default();
    }

    if arg_bool(&args, "--skip-redist") {
        cfg.skip_redist = true;
        arg_remove(&mut args, "--skip-redist");
    }

    #[cfg(windows)]
    if arg_bool(&args, "--redist") {
        arg_remove(&mut args, "--redist");
        misc::install_dependencies(&install_path).await;
        println_info!("Redistributables installation finished. Press enter to exit...");
        misc::stdin();
        std::process::exit(0);
    }

    let games_json =
        http_async::get_body_string(format!("{}/games.json", MASTER.lock().unwrap()).as_str())
            .await
            .unwrap_or_else(|error| {
                crate::println_error!("Failed to get games.json: {:#?}", error);
                misc::stdin();
                std::process::exit(1);
            });
    let games: Vec<Game> = serde_json::from_str(&games_json).unwrap_or_else(|error| {
        crate::println_error!("Error parsing games.json: {:#?}", error);
        misc::stdin();
        std::process::exit(1);
    });

    let mut game: String = String::new();
    if args.len() > 1 {
        game = String::from(&args[1]);
    } else {
        'main: for g in games.iter() {
            for r in g.references.iter() {
                if install_path.join(r).exists() {
                    if g.client.len() > 1 {
                        if cfg.update_only {
                            game = String::from(g.client[0]);
                            break 'main;
                        }

                        #[cfg(windows)]
                        setup_client_links(g, &env::current_dir().unwrap());

                        #[cfg(not(windows))]
                        println!("Multiple clients installed, set the client as the first argument to launch a specific client.");
                        println!("Select a client to launch:");
                        for (i, c) in g.client.iter().enumerate() {
                            println!("{i}: {c}");
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
                if cfg.engine.is_empty() {
                    cfg.engine = String::from(g.engine);
                    config::save_value_s(
                        install_path.join("alterware-launcher.json"),
                        "engine",
                        cfg.engine.clone(),
                    );
                    if cfg.engine == "iw4" && cfg.args.is_empty() {
                        cfg.args = String::from("-stdout");
                        config::save_value_s(
                            install_path.join("alterware-launcher.json"),
                            "args",
                            cfg.args.clone(),
                        );
                    }

                    #[cfg(windows)]
                    if !cfg.skip_redist {
                        misc::install_dependencies(&install_path).await;
                    }
                }

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
                    Some(&game != "iw4x-sp"),
                    Some(ignore_required_files),
                )
                .await;
                if !cfg.update_only {
                    launch(&install_path.join(format!("{c}.exe")), &cfg.args);
                }
                return;
            }
        }
    }

    #[cfg(windows)]
    windows_launcher_install(&games).await;

    crate::println_error!("Game not found!");
    println!("Place the launcher in the game folder, if that doesn't work specify the client on the command line (ex. alterware-launcher.exe iw4-sp)");
    println!("Press enter to exit...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
