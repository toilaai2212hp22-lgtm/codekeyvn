//! CodeKey IBus engine binary.
//!
//! Registers `org.freedesktop.IBus.codekey` and serves a factory that creates
//! per-context engines composing Vietnamese (Telex/VNI) via preedit.

mod address;
mod engine;
mod factory;
mod handler;
mod keysyms;
mod types;

use anyhow::{Context, Result};
use tracing_subscriber::EnvFilter;
use zbus::connection::Builder;
use zbus::AuthMechanism;

use crate::factory::{Factory, FACTORY_PATH};

const BUS_NAME: &str = "org.freedesktop.IBus.codekey";

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .with_writer(std::io::stderr)
        .init();

    let addr = address::discover_address().context("discover IBUS_ADDRESS")?;
    let factory = Factory::new();

    // Register factory *before* claiming the name so early method calls are not lost.
    let _connection = Builder::address(addr.as_str())?
        .auth_mechanism(AuthMechanism::External)
        .serve_at(FACTORY_PATH, factory)?
        .name(BUS_NAME)?
        .build()
        .await
        .context("connect/register with IBus (is ibus-daemon running?)")?;

    tracing::info!("CodeKey IBus engine registered as {BUS_NAME}");
    std::future::pending::<()>().await;
    #[allow(unreachable_code)]
    Ok(())
}
