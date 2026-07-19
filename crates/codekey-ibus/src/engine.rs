//! `org.freedesktop.IBus.Engine` D-Bus interface.

use tokio::sync::{mpsc, oneshot};
use zbus::object_server::SignalEmitter;
use zbus::{interface, Connection};
use zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::handler::{commit_text, preedit_text, Action, CodeKeyHandler};

const PREEDIT_COMMIT: u32 = 1;

enum Cmd {
    ProcessKey {
        keyval: u32,
        keycode: u32,
        state: u32,
        reply: oneshot::Sender<bool>,
    },
    FocusIn,
    FocusOut,
    Reset,
    Enable,
    Disable,
    Destroy,
}

struct Actor {
    handler: CodeKeyHandler,
    emitter: SignalEmitter<'static>,
    conn: Connection,
    path: OwnedObjectPath,
}

impl Actor {
    async fn run(mut self, mut rx: mpsc::Receiver<Cmd>) {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Cmd::ProcessKey {
                    keyval,
                    keycode,
                    state,
                    reply,
                } => {
                    let (handled, actions) =
                        self.handler.process_key_event(keyval, keycode, state);
                    self.emit_all(actions).await;
                    let _ = reply.send(handled);
                }
                Cmd::FocusIn => {}
                Cmd::FocusOut => {
                    let actions = self.handler.focus_out();
                    self.emit_all(actions).await;
                }
                Cmd::Reset | Cmd::Disable => {
                    let actions = self.handler.reset();
                    self.emit_all(actions).await;
                }
                Cmd::Enable => {}
                Cmd::Destroy => break,
            }
        }
        let _ = self
            .conn
            .object_server()
            .remove::<EngineInterface, _>(self.path)
            .await;
    }

    async fn emit_all(&self, actions: Vec<Action>) {
        for a in actions {
            let _ = self.emit(a).await;
        }
    }

    async fn emit(&self, action: Action) -> zbus::Result<()> {
        let e = &self.emitter;
        match action {
            Action::CommitText(text) => {
                let t = commit_text(&text);
                EngineInterface::commit_text(e, t.into()).await
            }
            Action::UpdatePreedit {
                text,
                cursor,
                visible,
            } => {
                let t = preedit_text(&text);
                EngineInterface::update_preedit_text(e, t.into(), cursor, visible, PREEDIT_COMMIT)
                    .await
            }
            Action::HidePreedit => EngineInterface::hide_preedit_text(e).await,
            Action::ForwardKey {
                keyval,
                keycode,
                state,
            } => EngineInterface::forward_key_event(e, keyval, keycode, state).await,
        }
    }
}

pub struct EngineInterface {
    cmds: mpsc::Sender<Cmd>,
}

impl EngineInterface {
    pub fn spawn(
        handler: CodeKeyHandler,
        emitter: SignalEmitter<'static>,
        conn: Connection,
        path: OwnedObjectPath,
    ) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let actor = Actor {
            handler,
            emitter,
            conn,
            path,
        };
        tokio::spawn(actor.run(rx));
        Self { cmds: tx }
    }

    async fn send(&self, cmd: Cmd) {
        let _ = self.cmds.send(cmd).await;
    }
}

#[interface(name = "org.freedesktop.IBus.Engine")]
impl EngineInterface {
    async fn process_key_event(&self, keyval: u32, keycode: u32, state: u32) -> bool {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .cmds
            .send(Cmd::ProcessKey {
                keyval,
                keycode,
                state,
                reply: reply_tx,
            })
            .await
            .is_err()
        {
            return false;
        }
        reply_rx.await.unwrap_or(false)
    }

    async fn focus_in(&self) {
        self.send(Cmd::FocusIn).await;
    }

    async fn focus_out(&self) {
        self.send(Cmd::FocusOut).await;
    }

    async fn reset(&self) {
        self.send(Cmd::Reset).await;
    }

    async fn enable(&self) {
        self.send(Cmd::Enable).await;
    }

    async fn disable(&self) {
        self.send(Cmd::Disable).await;
    }

    async fn set_cursor_location(&self, _x: i32, _y: i32, _w: i32, _h: i32) {}

    async fn set_capabilities(&self, _caps: u32) {}

    async fn set_content_type(&self, _purpose: u32, _hints: u32) {}

    async fn page_up(&self) {}
    async fn page_down(&self) {}
    async fn cursor_up(&self) {}
    async fn cursor_down(&self) {}

    async fn candidate_clicked(&self, _index: u32, _button: u32, _state: u32) {}

    async fn property_activate(&self, _name: &str, _state: u32) {}
    async fn property_show(&self, _name: &str) {}
    async fn property_hide(&self, _name: &str) {}

    async fn destroy(&self) {
        self.send(Cmd::Destroy).await;
    }

    #[zbus(signal)]
    async fn commit_text(emitter: &SignalEmitter<'_>, text: Value<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn update_preedit_text(
        emitter: &SignalEmitter<'_>,
        text: Value<'_>,
        cursor_pos: u32,
        visible: bool,
        mode: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn hide_preedit_text(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn forward_key_event(
        emitter: &SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> zbus::Result<()>;
}

// silence unused import if OwnedValue only used via into_variant path
#[allow(dead_code)]
fn _owned_value_link(v: OwnedValue) -> OwnedValue {
    v
}
