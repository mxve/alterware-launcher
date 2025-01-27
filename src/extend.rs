use crate::hash;
use simple_log::*;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// Cross-platform path separator
pub trait CutePath {
    fn cute_path(&self) -> String;
}
impl CutePath for Path {
    fn cute_path(&self) -> String {
        let path = self.to_str().unwrap().replace('\\', "/");
        path
    }
}
impl CutePath for PathBuf {
    fn cute_path(&self) -> String {
        self.as_path().cute_path()
    }
}

/// Get the BLAKE3 hash of a file. Returns io::ErrorKind::InvalidInput if the path is a directory.
pub trait Blake3Path {
    fn get_blake3(&self) -> io::Result<String>;
}
impl Blake3Path for Path {
    fn get_blake3(&self) -> io::Result<String> {
        if self.is_dir() {
            // The default Error (PermissionDenied) is not very helpful when troubleshooting
            let error_msg = format!("Path is a directory: {}", self.cute_path());
            error!("{}", error_msg);
            return Err(io::Error::new(io::ErrorKind::InvalidInput, error_msg));
        }
        info!("Getting BLAKE3 hash for file: {}", self.cute_path());
        hash::blake3_file(self)
    }
}
impl Blake3Path for PathBuf {
    fn get_blake3(&self) -> io::Result<String> {
        self.as_path().get_blake3()
    }
}

/// Convert int to human readable file size
pub trait HumanReadableSize {
    fn human_readable_size(self) -> String;
}

/// Implement the HumanReadableSize trait for multiple types
macro_rules! impl_human_readable_size {
    ($($t:ty)*) => {
        $(
            impl HumanReadableSize for $t {
                fn human_readable_size(self) -> String {
                    let mut bytes = self as f64;
                    let mut i = 0;
                    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
                    while bytes >= 1024.0 && i < units.len() - 1 {
                        bytes /= 1024.0;
                        i += 1;
                    }
                    let result = format!("{:.2} {}", bytes, units[i]);
                    result
                }
            }
        )*
    }
}
impl_human_readable_size!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 usize isize);
