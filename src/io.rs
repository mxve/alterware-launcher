pub fn unzip(zip_path: &str, out_path: &str) {
    let mut archive = zip::ZipArchive::new(fs::File::open(&temp_file).unwrap()).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = iw4x_path.join(file.name());

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!("Unpacking {}", file.name());
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }
}