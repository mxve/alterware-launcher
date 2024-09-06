use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

pub fn file_blake3(file: &Path) -> std::io::Result<String> {
    let mut blake3 = blake3::Hasher::new();
    let mut file = File::open(file)?;
    let mut buffer = [0; 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        blake3.update(&buffer[..n]);
    }
    Ok(blake3.finalize().to_string())
}

pub trait Blake3Path {
    fn get_blake3(&self) -> io::Result<String>;
}
impl Blake3Path for Path {
    fn get_blake3(&self) -> io::Result<String> {
        if self.is_dir() {
            // The default Error (PermissionDenied) is not very helpful when troubleshooting
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path is a directory ({})", self.cute_path()),
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
        self.to_str().unwrap().replace('\\', "/")
    }
}
impl CutePath for PathBuf {
    fn cute_path(&self) -> String {
        self.as_path().cute_path()
    }
}
