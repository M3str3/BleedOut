use super::Exfiltrator;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use url::Url;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

enum FileMode {
    Zip,
    Dir,
}

pub struct FileExfiltrator {
    mode: FileMode,
    /// Absolute path where we will write (zip file or directory).
    base: PathBuf,
    zip: Option<ZipWriter<BufWriter<File>>>,
}

impl FileExfiltrator {
    pub fn from_url(url: &Url) -> Result<Self, String> {
        // Reject `file://host/...` forms. `file://test.zip` is almost always a
        // typo for `file:///test.zip` (three slashes), and silently writing to
        // cwd is confusing.
        if let Some(host) = url.host_str() {
            if !host.is_empty() && !host.eq_ignore_ascii_case("localhost") {
                return Err(format!(
                    "file:// URL has host {host:?}, which is not supported.\n\
                     For a local file use `file:///absolute/path.zip` (three slashes) \
                     or `file:relative/path.zip` (relative to cwd)."
                ));
            }
        }

        let raw_path = url.path();
        let decoded = decode_file_url_path(raw_path)?;

        if decoded.is_empty() {
            return Err(
                "file:// URL has an empty path. Use `file:///path/to/output.zip` \
                 or `file:relative/path.zip`."
                    .into(),
            );
        }

        let path = PathBuf::from(decoded);
        // Resolve relative paths against the current working directory so that
        // we always print and use an absolute path.
        let absolute = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()
                .map_err(|e| format!("cannot resolve cwd: {e}"))?
                .join(path)
        };

        let is_zip = absolute
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("zip"))
            .unwrap_or(false);

        if is_zip {
            if let Some(parent) = absolute.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("create zip parent {}: {e}", parent.display()))?;
                }
            }
            let file = File::create(&absolute)
                .map_err(|e| format!("create zip {}: {e}", absolute.display()))?;
            let writer = ZipWriter::new(BufWriter::new(file));
            Ok(Self {
                mode: FileMode::Zip,
                base: absolute,
                zip: Some(writer),
            })
        } else {
            fs::create_dir_all(&absolute)
                .map_err(|e| format!("create dir {}: {e}", absolute.display()))?;
            Ok(Self {
                mode: FileMode::Dir,
                base: absolute,
                zip: None,
            })
        }
    }
}

/// Turn a `file://` path component into a Windows-friendly local path.
fn decode_file_url_path(p: &str) -> Result<String, String> {
    // Strip the leading '/' but keep it if it's just a drive letter prefix
    // like "/C:/Users/..." → "C:/Users/..." (on Windows).
    let stripped = p.strip_prefix('/').unwrap_or(p);
    Ok(percent_decode(stripped))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn relative_path(root_dir: &Path, local_path: &Path) -> PathBuf {
    let root_name = root_dir
        .file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("root"));

    let relative = local_path
        .strip_prefix(root_dir)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(local_path.file_name().unwrap_or_default()));

    root_name.join(relative)
}

impl Exfiltrator for FileExfiltrator {
    fn upload_file(&mut self, local_path: &Path, root_dir: &Path) -> Result<(), Box<dyn Error>> {
        let rel = relative_path(root_dir, local_path);

        match self.mode {
            FileMode::Zip => {
                let zip = self.zip.as_mut().ok_or("zip writer missing")?;
                let zip_name = rel.to_string_lossy().replace('\\', "/");
                zip.start_file(
                    zip_name,
                    FileOptions::default().compression_method(CompressionMethod::Deflated),
                )?;
                let mut f = File::open(local_path)?;
                std::io::copy(&mut f, zip)?;
                Ok(())
            }
            FileMode::Dir => {
                let dest = self.base.join(&rel);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(local_path, &dest)?;
                Ok(())
            }
        }
    }

    fn finalize(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(mut zip) = self.zip.take() {
            let mut inner = zip.finish()?;
            inner.flush()?;
        }
        Ok(())
    }

    fn describe(&self) -> String {
        match self.mode {
            FileMode::Zip => format!("file:// → {} (ZIP archive)", self.base.display()),
            FileMode::Dir => format!("file:// → {} (directory)", self.base.display()),
        }
    }
}
