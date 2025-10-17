use anyhow::Result;
use eframe::{egui, epi};
use rfd::FileDialog;
use rust_wiper::{interactive_confirm, simulate_command, validate_device};
use std::env;

struct WiperApp {
    device: String,
    method: String,
    passes: u8,
    dry_run: bool,
    execute_requested: bool,
    log: Vec<String>,
}

impl Default for WiperApp {
    fn default() -> Self {
        Self {
            device: String::new(),
            method: "dd".to_string(),
            passes: 1,
            dry_run: true,
            execute_requested: false,
            log: vec![],
        }
    }
}

impl epi::App for WiperApp {
    fn name(&self) -> &str {
        "rust-wiper GUI"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("rust-wiper (prototype) — GUI");
            ui.horizontal(|ui|{
                ui.label("Device:");
                if ui.button("Choose...").clicked() {
                    if let Some(p) = FileDialog::new().pick_file() {
                        self.device = p.to_string_lossy().to_string();
                    }
                }
                ui.text_edit_singleline(&mut self.device);
            });
            ui.horizontal(|ui| {
                ui.label("Method:");
                egui::ComboBox::from_id_source("method")
                    .selected_text(&self.method)
                    .show_ui(ui, |ui|{
                        ui.selectable_value(&mut self.method, "dd".to_string(), "dd");
                        ui.selectable_value(&mut self.method, "blkdiscard".to_string(), "blkdiscard");
                        ui.selectable_value(&mut self.method, "hdparm".to_string(), "hdparm");
                        ui.selectable_value(&mut self.method, "nvme".to_string(), "nvme");
                    });
                ui.label("Passes:");
                ui.add(egui::DragValue::new(&mut self.passes).clamp_range(1..=10));
            });
            ui.checkbox(&mut self.dry_run, "Dry-run (safe)");
            ui.label("To perform destructive action: set env WIPER_ALLOW_EXECUTE=1, toggle Execute and confirm.");
            if ui.button("Simulate / Show commands").clicked() {
                self.log.clear();
                let simulated = format!("Simulate: method={} passes={} device={}", self.method, self.passes, self.device);
                self.log.push(simulated);
            }
            ui.separator();
            if ui.checkbox(&mut self.execute_requested, "Execute (destructive) - REQUIRES ENV VAR").clicked() {
                // toggled
            }
            if ui.button("Run").clicked() {
                // Run flow: validate device, check env, prompt confirm via console (note: GUI can't read stdin easily)
                // We'll only run interactive confirm if env var present; otherwise simulate
                let dry = self.dry_run || !self.execute_requested;
                if let Err(e) = validate_device(&self.device) {
                    self.log.push(format!("Validation error: {:?}", e));
                } else {
                    if !dry {
                        let allow = env::var("WIPER_ALLOW_EXECUTE").unwrap_or_default();
                        if allow != "1" {
                            self.log.push("Missing WIPER_ALLOW_EXECUTE env var - refusing to execute.".to_string());
                            return;
                        }
                        // interactive_confirm in CLI requires stdin; here we simulate by adding a log entry and refusing to proceed automatically.
                        self.log.push("Interactive confirmation required - in GUI this prototype will ask you to confirm in terminal.".to_string());
                        // For safety, do not perform destructive steps in GUI automatically.
                    }
                    // Build command strings and simulate
                    match self.method.as_str() {
                        "dd" => {
                            for p in 1..=self.passes {
                                let cmd = format!("dd if=/dev/urandom of={} bs=4M status=progress (pass {}/{})", self.device, p, self.passes);
                                if dry {
                                    self.log.push(format!("[DRY-RUN] {}", cmd));
                                } else {
                                    self.log.push(format!("[EXEC] {}", cmd));
                                }
                            }
                            let final_cmd = format!("dd if=/dev/zero of={} bs=4M status=progress (final)", self.device);
                            if dry { self.log.push(format!("[DRY-RUN] {}", final_cmd)); } else { self.log.push(format!("[EXEC] {}", final_cmd)); }
                        }
                        "blkdiscard" => {
                            let cmd = format!("blkdiscard {}", self.device);
                            if dry { self.log.push(format!("[DRY-RUN] {}", cmd)); } else { self.log.push(format!("[EXEC] {}", cmd)); }
                        }
                        "hdparm" => {
                            let cmd = format!("hdparm -I {} && hdparm --user-master u --security-set-pass p {}", self.device, self.device);
                            self.log.push(format!("[NOTE] hdparm flow printed: {}", cmd));
                        }
                        "nvme" => {
                            let cmd = format!("nvme sanitize {} --ses 1", self.device);
                            if dry { self.log.push(format!("[DRY-RUN] {}", cmd)); } else { self.log.push(format!("[EXEC] {}", cmd)); }
                        }
                        other => self.log.push(format!("Unknown method: {}", other)),
                    }
                }
            }

            ui.separator();
            ui.label("Log:");
            for line in self.log.iter().rev().take(30) {
                ui.label(line);
            }
        });
    }
}

fn main() -> Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "rust-wiper GUI",
        options,
        Box::new(|_cc| Box::new(WiperApp::default())),
    );
    Ok(())
}
