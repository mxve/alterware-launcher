use std::{
    env,
    io::{self, Write},
    path::{Path, PathBuf},
};

mod self_update;

#[derive(Debug)]
enum Game {
    IW2,
    IW4(u8), // 0 = iw4x, 1 = iw4x-sp
    IW5,
    IW6,
    S1,
}

impl Game {
    fn platform(&self) -> Platform {
        match self {
            Game::IW4(1) => Platform::IW4x, // iw4x belongs to IW4x platform
            _ => Platform::AlterWare,       // everything else belongs to AlterWare
        }
    }
}

#[derive(Debug, Clone)]
pub enum Platform {
    IW4x,
    AlterWare,
}

async fn run_update(game: Game) {
    let platform = game.platform();
    println!("Game: {:?}", game);
    println!("Platform: {:?}", platform);
    self_update::run_with_platform(platform).await;
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let game = if args.len() > 1 {
        if let Some(game) = detect_game_by_client(&args[1]) {
            Some(game)
        } else if let Some(path) = arg_value(&args, "--path").or_else(|| arg_value(&args, "-p")) {
            let install_path = PathBuf::from(path);
            detect_game(&install_path)
        } else {
            let install_path = env::current_dir().unwrap();
            detect_game(&install_path)
        }
    } else if let Some(path) = arg_value(&args, "--path").or_else(|| arg_value(&args, "-p")) {
        let install_path = PathBuf::from(path);
        detect_game(&install_path)
    } else {
        let install_path = env::current_dir().unwrap();
        detect_game(&install_path)
    };

    match game {
        Some(game) => {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(run_update(game));
        }
        None => println!("No supported game detected in the directory"),
    }
}

fn detect_game_by_client(client: &str) -> Option<Game> {
    match client {
        "iw2-mod" => Some(Game::IW2),
        "iw4x" => Some(Game::IW4(1)),
        "iw4x-sp" => Some(Game::IW4(0)),
        "iw5-mod" => Some(Game::IW5),
        "iw6-mod" => Some(Game::IW6),
        "s1-mod" => Some(Game::S1),
        _ => None,
    }
}

fn detect_game(dir: &Path) -> Option<Game> {
    println!("Checking directory: {}", dir.display());

    let games = [
        (Game::IW2, &["CoD2MP_s.exe", "CoD2SP_s.exe"][..]),
        (Game::IW4(0), &["iw4sp.exe", "iw4mp.exe", "iw4x.exe"]),
        (Game::IW5, &["iw5sp.exe", "iw5mp.exe", "iw5mp_server.exe"]),
        (Game::IW6, &["iw6sp64_ship.exe", "iw6mp64_ship.exe"]),
        (Game::S1, &["s1_sp64_ship.exe", "s1_mp64_ship.exe"]),
    ];

    for (game, refs) in games {
        println!("Checking for {:?} with files: {:?}", game, refs);
        if check_references(dir, refs) {
            return match game {
                Game::IW4(_) => Some(prompt_iw4_client()),
                _ => Some(game),
            };
        }
    }

    None
}

fn check_references(dir: &Path, references: &[&str]) -> bool {
    for reference in references {
        let file_path = dir.join(reference);
        if file_path.exists() {
            return true;
        }
    }
    false
}

fn prompt_iw4_client() -> Game {
    let clients = ["iw4x-sp", "iw4x"];

    println!("Multiple clients available, select one to launch:");
    for (i, c) in clients.iter().enumerate() {
        println!("{i}: {c}");
    }

    loop {
        print!("Enter selection: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            if let Ok(selection) = input.trim().parse::<usize>() {
                if selection < clients.len() {
                    return Game::IW4(selection as u8);
                }
            }
        }
        println!("Invalid selection, please try again.");
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
