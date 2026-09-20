extern crate winapi;

use crate::decypher::decypher;
use crate::ui::tag;
use std::ffi::CString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use winapi::um::fileapi::DeleteFileA;

/// Walk each target directory and decrypt files whose extension matches
/// `encrypted_ext`. Returns the number of files restored.
pub fn walk_decrypt(
    target_dirs: &[PathBuf],
    aes_key: &[u8],
    encrypted_ext: &str,
    verbose: bool,
) -> usize {
    let mut restored = 0usize;
    for dir in target_dirs {
        if verbose {
            println!("{} walk    {}", tag("*"), dir.display());
        }

        restored += traverse_and_decrypt_path(dir, aes_key, encrypted_ext, verbose).unwrap_or(0);
    }
    restored
}

pub fn traverse_and_decrypt_path(
    directory_path: &Path,
    aes_key: &[u8],
    encrypted_ext: &str,
    verbose: bool,
) -> io::Result<usize> {
    if !directory_path.is_dir() {
        return Ok(0);
    }

    let mut restored = 0usize;

    for entry in fs::read_dir(directory_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            restored +=
                traverse_and_decrypt_path(&path, aes_key, encrypted_ext, verbose).unwrap_or(0);
            continue;
        }

        let matches = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e == encrypted_ext)
            .unwrap_or(false);

        if !matches {
            continue;
        }

        // Strip the ransomware extension to recover the original name.
        let mut decrypted_path = path.clone();
        decrypted_path.set_extension("");

        if verbose {
            println!(
                "{} decrypt {} -> {}",
                tag("*"),
                path.display(),
                decrypted_path.display()
            );
        }

        let c_original =
            CString::new(path.to_string_lossy().as_bytes()).expect("CString from source path");
        let c_decrypted = CString::new(decrypted_path.to_string_lossy().as_bytes())
            .expect("CString from destination path");

        let ok = decypher::decrypt(c_original.clone(), c_decrypted, aes_key.to_vec());
        if ok {
            unsafe { DeleteFileA(c_original.as_ptr()) };
            restored += 1;
        } else if verbose {
            println!("{} fail    {}", tag("!"), path.display());
        }
    }
    Ok(restored)
}
