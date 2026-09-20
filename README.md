<div align="center">

# BleedOut

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/223a8d72-4556-45c6-8716-4ff18a817852" />


**Educational ransomware. No bitcoin. No mercy for your lab VM.**

[![Rust](https://img.shields.io/badge/Rust-Windows-orange)](#)
[![Platform](https://img.shields.io/badge/platform-Windows-blue)](#)

</div>

AES at runtime. RSA wraps the session key. Files get a new extension, a note hits the folder, wallpaper if you let it. Optional loot staging over FTP or a local ZIP.

Relative `--dirs` are resolved against the user profile (`Desktop\lab` → `%USERPROFILE%\Desktop\lab`).

**Environments with consent only.**

## Lock

```powershell
cargo build --release
.\target\release\bleedouter.exe lock --dirs "Desktop\lab" --dry-run
.\target\release\bleedouter.exe lock --dirs "Desktop\lab" -w -v
```

`-n` / `--dry-run` walks the tree and lists hits. Nothing is written.

`-P` / `--paranoid` runs the anti-reversing checks (debugger / noisy processes). If it doesn't like the box, it bails.

`-x` / `--exfil` ships copies out: `ftp://user:pass@host/base`, `file:///C:/out/loot.zip`, or a folder URL.

`-E` / `--exclude` adds globs. Built-in skips (`$RECYCLE.BIN`, `.git`, `node_modules`, `target`, …) stay on unless `--no-exclude-defaults`. `-s` caps file size (default 100MB, `0` = no cap). `-i` overrides the extension list.

Public key is baked in. Override with `-k`.

## Unlock

```powershell
.\target\release\bleedouter.exe unlock --dirs "Desktop\lab" --private-key .\keys\private_key.pem -f C:\Users\you\Desktop\lab\README.txt
```

`-f` reads `KEY = ...` from the note. `-k` if you already have the blob. Private key is required; there is no default.

## Flow

1. Session AES key
2. Walk + encrypt (or dry-run)
3. RSA-wrap the key, drop `README.txt`
4. Optional exfil + wallpaper
5. Unlock unwraps the key and restores

## Fine print

Do not run this on a machine you do not own or do not have written permission to test. BleedOut is a teaching / report PoC, not a product.
