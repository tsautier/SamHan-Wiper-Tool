use anyhow::Result;
use chrono::Local;
use clap::Parser;
use std::env;
use std::process;

use rust_wiper::interactive_confirm;
use rust_wiper::simulate_command;
use rust_wiper::validate_device;

#[derive(Parser, Debug)]
#[command(author, version, about = "rust-wiper CLI (dry-run default)")]
struct Args {
    /// Device to wipe (e.g. /dev/sdb or C:\)
    #[arg(short, long)]
    device: Option<String>,

    /// Method: dd | blkdiscard | hdparm | nvme
    #[arg(short, long, default_value = "dd")]
    method: String,

    /// Number of passes (only for dd)
    #[arg(short = 'n', long, default_value_t = 1)]
    passes: u8,

    /// Dry-run default: shows commands but does not execute.
    #[arg(short, long)]
    dry_run: bool,

    /// Actually execute (requires WIPER_ALLOW_EXECUTE=1 env var)
    #[arg(short, long)]
    execute: bool,

    /// List block devices (uses lsblk on Unix)
    #[arg(long)]
    list: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // default to dry-run unless --execute provided
    let mut dry = true;
    if args.execute {
        dry = false;
    }
    if args.dry_run {
        dry = true;
    }

    // simple list
    if args.list {
        #[cfg(unix)]
        {
            let out = std::process::Command::new("lsblk")
                .arg("-o")
                .arg("NAME,SIZE,TYPE,MOUNTPOINT,MODEL")
                .arg("-J")
                .output()?;
            println!("{}", String::from_utf8_lossy(&out.stdout));
            return Ok(());
        }
        #[cfg(windows)]
        {
            println!("Listing devices on Windows is not implemented in this prototype. Use Disk Management or wmic diskdrive list brief");
            return Ok(());
        }
    }

    let device = args
        .device
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--device is required for wipe operations"))?;
    validate_device(device)?;

    // Safety: require env var for execution
    if !dry {
        let allow = env::var("WIPER_ALLOW_EXECUTE").unwrap_or_default();
        if allow != "1" {
            eprintln!("Refusing to execute: set environment variable WIPER_ALLOW_EXECUTE=1 to allow destructive operations.");
            process::exit(2);
        }
        // interactive confirm
        interactive_confirm(device)?;
    }

    // Build command (simulated)
    match args.method.as_str() {
        "dd" => {
            for p in 1..=args.passes {
                let cmd = format!(
                    "dd if=/dev/urandom of={} bs=4M status=progress (pass {}/{})",
                    device, p, args.passes
                );
                simulate_command(&cmd, dry)?;
            }
            simulate_command(
                &format!(
                    "dd if=/dev/zero of={} bs=4M status=progress (final)",
                    device
                ),
                dry,
            )?;
        }
        "blkdiscard" => {
            simulate_command(&format!("blkdiscard {}", device), dry)?;
        }
        "hdparm" => {
            simulate_command(
                &format!(
                    "hdparm -I {} && hdparm --user-master u --security-set-pass p {}",
                    device, device
                ),
                dry,
            )?;
            println!(
                "Note: hdparm flow is printed but not automatically executed by this prototype."
            );
        }
        "nvme" => {
            simulate_command(&format!("nvme sanitize {} --ses 1", device), dry)?;
        }
        other => anyhow::bail!("Unknown method '{}'", other),
    }

    // write a simple log
    let ts = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let logpath = format!("/var/log/rust-wiper-{}.log", ts);
    println!("Log path (example): {}", logpath);

    Ok(())
}
