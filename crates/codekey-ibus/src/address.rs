//! Discover and connect to the private IBus D-Bus daemon.

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use zbus::connection::Builder;
use zbus::{AuthMechanism, Connection};

/// Connect to the IBus private bus (not the session bus).
#[allow(dead_code)]
pub async fn connect() -> Result<Connection> {
    let addr = discover_address().context("discover IBUS_ADDRESS")?;
    Builder::address(addr.as_str())?
        .auth_mechanism(AuthMechanism::External)
        .build()
        .await
        .with_context(|| format!("connect to IBus at {addr}"))
}

/// Resolve the private IBus bus address string.
pub fn discover_address() -> Result<String> {
    if let Ok(addr) = env::var("IBUS_ADDRESS") {
        if !addr.is_empty() {
            return Ok(addr);
        }
    }
    if let Ok(path) = env::var("IBUS_ADDRESS_FILE") {
        if !path.is_empty() {
            return read_address_file(PathBuf::from(path));
        }
    }

    let machine_id = read_machine_id()?;
    let display = display_token();
    // IBus names files: `{machine-id}-unix-{display}` (X11 and Wayland).
    let path = config_dir()
        .join("ibus")
        .join("bus")
        .join(format!("{machine_id}-unix-{display}"));

    read_address_file(path)
}

fn read_address_file(path: PathBuf) -> Result<String> {
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("read IBus bus file {}", path.display()))?;
    contents
        .lines()
        .find_map(|line| line.strip_prefix("IBUS_ADDRESS=").map(str::to_string))
        .ok_or_else(|| anyhow!("IBUS_ADDRESS= not found in {}", path.display()))
}

fn read_machine_id() -> Result<String> {
    for p in ["/var/lib/dbus/machine-id", "/etc/machine-id"] {
        if let Ok(id) = fs::read_to_string(p) {
            return Ok(id.trim().to_string());
        }
    }
    bail!("no machine-id found")
}

fn display_token() -> String {
    if let Ok(wd) = env::var("WAYLAND_DISPLAY") {
        if !wd.is_empty() {
            return wd;
        }
    }
    let d = env::var("DISPLAY").unwrap_or_else(|_| ":0".into());
    d.trim_start_matches(':')
        .split('.')
        .next()
        .unwrap_or("0")
        .to_string()
}

fn config_dir() -> PathBuf {
    env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".into())).join(".config")
        })
}
