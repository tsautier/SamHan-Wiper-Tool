use anyhow::Result;
use regex::Regex;
use std::io::{self, Write};

pub fn validate_device(device: &str) -> Result<()> {
    let re = Regex::new(r"^(/dev/|\\\\\\\\.\\\\PhysicalDrive|[A-Za-z]:)").unwrap();
    if !re.is_match(device) {
        anyhow::bail!("Device validation failed: '{}' does not look like a valid device path", device);
    }
    Ok(())
}

pub fn interactive_confirm(device: &str) -> Result<()> {
    println!("\nATTENTION: vous êtes sur le point d'effacer définitivement : {}", device);
    println!("Ce processus est irréversible. Sauvegardez vos données avant de continuer.");
    print!("Tapez le chemin EXACT du périphérique pour confirmer: ");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    if buf.trim() != device {
        anyhow::bail!("Chemin incorrect — annulation.");
    }

    print!(\"Tapez 'ERASE' en MAJUSCULE pour confirmer (1/2): \");
    io::stdout().flush()?;
    buf.clear();
    io::stdin().read_line(&mut buf)?;
    if buf.trim() != \"ERASE\" {
        anyhow::bail!(\"Confirmation manquante — annulation.\");
    }

    print!(\"Retapez 'ERASE' pour confirmer (2/2): \");
    io::stdout().flush()?;
    buf.clear();
    io::stdin().read_line(&mut buf)?;
    if buf.trim() != \"ERASE\" {
        anyhow::bail!(\"Confirmation manquante — annulation.\");
    }

    Ok(())
}

// Runner skeleton: does not execute destructive commands unless dry=false and all checks passed.
pub fn simulate_command(cmd: &str, dry: bool) -> Result<()> {
    println!("{}", if dry { format!("[DRY-RUN] {}", cmd) } else { format!("[EXEC] {}", cmd) });
    Ok(())
}
