fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("res/icon.ico").set_language(0x0409);
        res.compile().unwrap();
    }
}
