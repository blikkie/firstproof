use sha2::Digest;
use std::fs::File;
use std::io::{BufReader, Read, Result};
use std::path::Path;

#[derive(Clone, Default)]
pub struct Sha256 {
    inner: sha2::Sha256,
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            inner: sha2::Sha256::new(),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    pub fn finalize(self) -> [u8; 32] {
        self.inner.finalize().into()
    }

    pub fn digest(data: &[u8]) -> [u8; 32] {
        sha2::Sha256::digest(data).into()
    }
}

/// Computes hex-encoded SHA-256 hash of a file on disk using a 64KB streaming buffer.
pub fn compute_sha256_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(65536, file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let digest = hasher.finalize();
    Ok(format_hex(&digest))
}

/// Bit-for-bit file verification.
/// First checks file length, then computes SHA-256 of both files to guarantee bit integrity.
pub fn verify_files_identical<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2) -> Result<bool> {
    let src_meta = std::fs::metadata(&src)?;
    let dst_meta = std::fs::metadata(&dst)?;

    if src_meta.len() != dst_meta.len() {
        return Ok(false);
    }

    let src_hash = compute_sha256_file(&src)?;
    let dst_hash = compute_sha256_file(&dst)?;

    Ok(src_hash == dst_hash)
}

fn format_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty_string() {
        let digest = Sha256::digest(b"");
        assert_eq!(
            format_hex(&digest),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_abc() {
        let digest = Sha256::digest(b"abc");
        assert_eq!(
            format_hex(&digest),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_verify_files_identical() {
        let temp_dir = std::env::temp_dir().join("firstproof_test_checksum");
        let _ = std::fs::create_dir_all(&temp_dir);
        let f1 = temp_dir.join("file1.raw");
        let f2 = temp_dir.join("file2.raw");
        let f3 = temp_dir.join("file3.raw");

        std::fs::write(&f1, b"first proof honest color raw payload").unwrap();
        std::fs::write(&f2, b"first proof honest color raw payload").unwrap();
        std::fs::write(&f3, b"corrupted payload").unwrap();

        assert!(verify_files_identical(&f1, &f2).unwrap());
        assert!(!verify_files_identical(&f1, &f3).unwrap());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
