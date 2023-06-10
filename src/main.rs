mod http;
use std::path::PathBuf;

const MASTER: &str = "https://master.alterware.dev";

fn download_and_launch(url: &str, file_path: &PathBuf) {
    http::download_file(url, file_path);
    std::process::Command::new(file_path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let game: String;
    if args.len() > 1 {
        game = String::from(&args[1]);
    } else {
        // check if iw4sp.exe or iw4mp.exe exists
        if std::path::Path::new("iw4sp.exe").exists() || std::path::Path::new("iw4mp.exe").exists()
        {
            game = String::from("iw4-sp");
        } else if std::path::Path::new("iw5sp.exe").exists()
            || std::path::Path::new("iw5mp.exe").exists()
            || std::path::Path::new("iw5mp_server.exe").exists()
        {
            game = String::from("iw5-mod");
        } else {
            println!("No game specified and no game found in current directory");
            return;
        }
    }

    if game == "iw4-sp" {
        download_and_launch(
            &format!("{}/iw4/iw4-sp.exe", MASTER),
            &PathBuf::from("iw4-sp.exe"),
        );
    } else if game == "iw5-mod" {
        download_and_launch(
            &format!("{}/iw5/iw5-mod.exe", MASTER),
            &PathBuf::from("iw5-mod.exe"),
        );
    } else {
        println!("Invalid game");
    }
}
