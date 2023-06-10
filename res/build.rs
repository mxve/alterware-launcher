#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("res/icon.ico").set_language(0x0409);

    if let Err(e) = res.compile() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

#[cfg(unix)]
fn main() {}
