# SamHan-Wiper-Tool (cross-platform CLI + GUI)

Prototype Rust project providing both a CLI and a minimal GUI for safely *simulating* disk wipe operations.
**THIS IS A PROTOTYPE - BY DEFAULT IT DOES NOT DESTROY DATA (dry-run).**

## Structure

- `src/lib.rs` - shared logic (validation, confirmations, runner skeleton)
- `src/bin/cli.rs` - CLI binary (uses clap)
- `src/bin/gui.rs` - GUI binary (eframe/egui simple interface)
- `README.md`, `LICENSE`

## Safety model (must follow all 3 to execute destructive actions)
1. pass `--execute` on the CLI or press "Execute (destructive)" in GUI
2. set environment variable `WIPER_ALLOW_EXECUTE=1`
3. confirm interactively (retype device & ERASE twice)

If any of these are missing the program will only perform a dry-run showing the commands.

## Build (Linux / macOS / Windows)
Install Rust toolchain (rustup). For GUI builds eframe requires a native linker and a GUI toolchain:

### macOS
```bash
rustup default stable
# build CLI
cargo build --release --bin rust-wiper-cli
# build GUI (requires macOS system libs)
cargo build --release --bin rust-wiper-gui
```

### Windows (MSVC)
Install Visual Studio Build Tools (C++) and rustup toolchain with msvc. Then:
```powershell
cargo build --release --bin rust-wiper-cli
cargo build --release --bin rust-wiper-gui
```

### Notes
- The GUI uses `eframe` (egui). Packaging into `.app` / `.exe` installers is out of scope here but standard Rust packaging applies.
- Always test on non-production media or loopback files before using on real drives.

## Running examples

### CLI (dry-run default)
```bash
# show devices
cargo run --bin rust-wiper-cli -- --list

# simulate wipe:
sudo env WIPER_ALLOW_EXECUTE=0 cargo run --bin rust-wiper-cli -- --device /dev/sdX --method dd --passes 1
```

### CLI (actual execution - DANGEROUS)
```bash
export WIPER_ALLOW_EXECUTE=1
sudo cargo run --bin rust-wiper-cli -- --device /dev/sdX --method dd --passes 1 --execute
```

### GUI
```bash
cargo run --bin rust-wiper-gui
```

## Limitations
- This prototype is educational. It does not attempt low-level ATA secure erase automation (hdparm/nvme flows are printed but not auto-run).
- RAID, LVM, multipath, Windows physical device handling, and SED crypto-erase are out-of-scope for automatic handling here.
