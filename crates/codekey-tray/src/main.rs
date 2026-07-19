//! CodeKey system tray (StatusNotifierItem via ksni/zbus).
//!
//! - Left-click: toggle Vietnamese / English
//! - Menu: Telex / VNI / EN / Quit

use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use codekey_engine::InputMethod;
use ksni::blocking::TrayMethods; // .spawn() for blocking API
use ksni::menu::*;
use ksni::Tray;

const ENG_ENGINE: &str = "xkb:us::eng";
const TELEX_ENGINE: &str = "codekey";
const VNI_ENGINE: &str = "codekey-vni";

#[derive(Debug)]
struct CodeKeyTray {
    vn_on: bool,
    method: InputMethod,
}

impl CodeKeyTray {
    fn apply_engine(&self) {
        let engine = if self.vn_on {
            match self.method {
                InputMethod::Telex => TELEX_ENGINE,
                InputMethod::Vni => VNI_ENGINE,
            }
        } else {
            ENG_ENGINE
        };
        if let Err(e) = set_ibus_engine(engine) {
            eprintln!("codekey-tray: {e}");
        }
    }

    fn sync_from_ibus(&mut self) {
        if let Some(eng) = current_ibus_engine() {
            match eng.as_str() {
                TELEX_ENGINE => {
                    self.vn_on = true;
                    self.method = InputMethod::Telex;
                }
                VNI_ENGINE => {
                    self.vn_on = true;
                    self.method = InputMethod::Vni;
                }
                _ => {
                    self.vn_on = false;
                }
            }
        }
    }
}

impl Tray for CodeKeyTray {
    fn id(&self) -> String {
        "codekey".into()
    }

    fn title(&self) -> String {
        if self.vn_on {
            format!("CodeKey · {}", self.method.as_str().to_uppercase())
        } else {
            "CodeKey · EN".into()
        }
    }

    fn icon_name(&self) -> String {
        if self.vn_on {
            "preferences-desktop-locale".into()
        } else {
            "input-keyboard".into()
        }
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: self.title(),
            description: "Bộ gõ tiếng Việt — click trái: VN/EN · menu: Telex/VNI".into(),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![
            StandardItem {
                label: "CodeKey".into(),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: if self.vn_on {
                    "● Tiếng Việt".into()
                } else {
                    "○ Tiếng Việt".into()
                },
                activate: Box::new(|this: &mut Self| {
                    this.vn_on = true;
                    this.apply_engine();
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: if !self.vn_on {
                    "● English".into()
                } else {
                    "○ English".into()
                },
                activate: Box::new(|this: &mut Self| {
                    this.vn_on = false;
                    this.apply_engine();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: if matches!(self.method, InputMethod::Telex) {
                    "● Telex".into()
                } else {
                    "○ Telex".into()
                },
                activate: Box::new(|this: &mut Self| {
                    this.method = InputMethod::Telex;
                    this.vn_on = true;
                    this.apply_engine();
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: if matches!(self.method, InputMethod::Vni) {
                    "● VNI".into()
                } else {
                    "○ VNI".into()
                },
                activate: Box::new(|this: &mut Self| {
                    this.method = InputMethod::Vni;
                    this.vn_on = true;
                    this.apply_engine();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Thoát".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.vn_on = !self.vn_on;
        self.apply_engine();
    }

    fn menu_about_to_show(&mut self) {
        self.sync_from_ibus();
    }
}

fn set_ibus_engine(name: &str) -> Result<()> {
    let status = Command::new("ibus")
        .args(["engine", name])
        .status()
        .with_context(|| format!("run ibus engine {name}"))?;
    if !status.success() {
        anyhow::bail!("ibus engine {name} exited with {status}");
    }
    Ok(())
}

fn current_ibus_engine() -> Option<String> {
    let out = Command::new("ibus").arg("engine").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn main() -> Result<()> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("codekey-tray {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let engine = current_ibus_engine().unwrap_or_else(|| ENG_ENGINE.into());
    let (vn_on, method) = match engine.as_str() {
        TELEX_ENGINE => (true, InputMethod::Telex),
        VNI_ENGINE => (true, InputMethod::Vni),
        _ => (false, InputMethod::Telex),
    };

    let tray = CodeKeyTray { vn_on, method };
    let handle = tray.spawn().context("spawn StatusNotifierItem (tray)")?;

    eprintln!("codekey-tray: ready (ibus={engine}). Left-click toggles VN/EN.");

    // Keep process alive; refresh title if external engine changes.
    loop {
        std::thread::sleep(Duration::from_secs(2));
        let _ = handle.update(|t: &mut CodeKeyTray| {
            t.sync_from_ibus();
        });
    }
}
