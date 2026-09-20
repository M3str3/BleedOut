pub mod file;
pub mod ftp;

use std::error::Error;
use std::path::Path;

/// Anything that can ship a local file to a remote location.
///
/// `root_dir` is the top-level `--dirs` entry the file belongs to. Backends
/// use it to compute a relative remote path (e.g. to preserve directory
/// structure inside a ZIP).
pub trait Exfiltrator {
    fn upload_file(&mut self, local_path: &Path, root_dir: &Path) -> Result<(), Box<dyn Error>>;

    /// Called once after all `upload_file` calls. Default is a no-op;
    /// used by the ZIP backend to flush and close the archive.
    fn finalize(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Human-readable description of where files will land.
    /// Printed by the CLI right after `from_url` succeeds.
    fn describe(&self) -> String;
}

/// Build an exfiltrator from a URL.
///
/// Supported schemes:
///   - ftp://[user[:pass]@]host[:port][/base/path]
///   - file:///absolute/path/output.zip   (single ZIP, structure preserved)
///   - file:///absolute/path/folder/      (individual files)
///   - file:relative/path/output.zip      (relative to cwd)
pub fn from_url(url_str: &str) -> Result<Box<dyn Exfiltrator>, String> {
    let url = url::Url::parse(url_str).map_err(|e| format!("Invalid exfil URL: {e}"))?;
    match url.scheme() {
        "ftp" => Ok(Box::new(ftp::FtpExfiltrator::from_url(&url)?)),
        "file" => Ok(Box::new(file::FileExfiltrator::from_url(&url)?)),
        other => Err(format!("Unsupported exfil scheme: {other}")),
    }
}
