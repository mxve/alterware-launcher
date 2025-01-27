use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Helper function to read file and apply a given update closure for the hasher
fn hash_file<F>(file: &Path, mut update_fn: F) -> std::io::Result<()>
where
    F: FnMut(&[u8]),
{
    let file = File::open(file)?;

    // We explicitly set BufReaders buffer size to 8KB to match our buffers size
    // This ensures consistency in case BufReaders default size changes in future versions
    let mut reader = BufReader::with_capacity(8192, file);
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        update_fn(&buffer[..bytes_read]);
    }

    Ok(())
}

// /// Calculate the SHA1 hash of a file
// pub fn sha1_file(file: &Path) -> std::io::Result<String> {
//     let mut hasher = sha1_smol::Sha1::new();
//     hash_file(file, |data| {
//         hasher.update(data);
//     })?;
//     Ok(hasher.digest().to_string().to_lowercase())
// }

/// Calculate the BLAKE3 hash of a file
pub fn blake3_file(file: &Path) -> std::io::Result<String> {
    let mut hasher = blake3::Hasher::new();
    hash_file(file, |data| {
        hasher.update(data);
    })?;
    Ok(hasher.finalize().to_hex().to_string().to_lowercase())
}
