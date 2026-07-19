//! `org.freedesktop.IBus.Factory` — creates engine instances.

use std::sync::atomic::{AtomicU64, Ordering};

use zbus::object_server::SignalEmitter;
use zbus::{fdo, interface, Connection, ObjectServer};
use zbus::zvariant::OwnedObjectPath;

use crate::engine::EngineInterface;
use crate::handler::CodeKeyHandler;

pub const FACTORY_PATH: &str = "/org/freedesktop/IBus/Factory";

pub struct Factory {
    counter: AtomicU64,
}

impl Factory {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }
}

#[interface(name = "org.freedesktop.IBus.Factory")]
impl Factory {
    async fn create_engine(
        &self,
        engine_name: &str,
        #[zbus(object_server)] server: &ObjectServer,
        #[zbus(connection)] conn: &Connection,
    ) -> fdo::Result<OwnedObjectPath> {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        let path = OwnedObjectPath::try_from(format!(
            "/org/freedesktop/IBus/Engine/codekey/{n}"
        ))
        .map_err(|e| fdo::Error::Failed(e.to_string()))?;

        let handler = CodeKeyHandler::from_engine_name(engine_name);
        let emitter =
            SignalEmitter::new(conn, &path).map_err(|e| fdo::Error::Failed(e.to_string()))?;
        let iface = EngineInterface::spawn(handler, emitter.to_owned(), conn.clone(), path.clone());
        server.at(&path, iface).await?;
        tracing::debug!("created engine {engine_name} at {path}");
        Ok(path)
    }

    async fn destroy(&self) {}
}
