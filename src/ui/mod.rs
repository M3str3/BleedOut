mod ansi;
mod banner;

pub use ansi::{enable_ansi, BOLD, DIM, GREEN, RED, RED_BRIGHT, RESET, YELLOW};
pub use banner::{banner_text, ASCII_ART, BANNER};

pub fn tag(inner: &str) -> String {
    format!("[{RED_BRIGHT}{inner}{RESET}]")
}

pub fn info(msg: &str) {
    println!("{} {msg}", tag("*"));
}

pub fn ok(msg: &str) {
    println!("{} {RED_BRIGHT}{msg}{RESET}", tag("+"));
}

pub fn warn(msg: &str) {
    eprintln!("{} {msg}", tag("!"));
}

pub fn fail(msg: &str) {
    eprintln!("{} {RED_BRIGHT}{msg}{RESET}", tag("-"));
}

pub fn vinfo(verbose: bool, msg: &str) {
    if verbose {
        info(msg);
    }
}

pub fn print_banner(cmd: &str) {
    for line in banner_text().lines() {
        println!("{RED_BRIGHT}{line}{RESET}");
    }
    info(&format!("BleedOut/{cmd}"));
}
