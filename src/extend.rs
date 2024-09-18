use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
};

pub fn file_blake3(file: &Path) -> io::Result<String> {
    let file = File::open(file)?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

pub trait Blake3Path {
    fn get_blake3(&self) -> io::Result<String>;
}

impl Blake3Path for Path {
    fn get_blake3(&self) -> io::Result<String> {
        if self.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path is a directory: {}", self.cute_path()),
            ));
        }
        file_blake3(self)
    }
}

impl Blake3Path for PathBuf {
    fn get_blake3(&self) -> io::Result<String> {
        self.as_path().get_blake3()
    }
}

pub trait CutePath {
    fn cute_path(&self) -> String;
}

impl CutePath for Path {
    fn cute_path(&self) -> String {
        self.to_string_lossy().replace('\\', "/")
    }
}

impl CutePath for PathBuf {
    fn cute_path(&self) -> String {
        self.as_path().cute_path()
    }
}
