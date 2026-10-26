//! IBus wire protocol: the Factory the daemon asks for engines, and the Engine object that
//! receives key events and answers with CommitText / ForwardKeyEvent / DeleteSurroundingText.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::time::{Duration, Instant};

use gtv_core::Method;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedObjectPath, Structure, StructureBuilder, Value};
use zbus::{interface, Connection, ObjectServer};

use crate::appdetect::AppDetector;
use crate::config::FileConfig;
use crate::session::{is_modifier, KeyEvent, Mode, Out, Session, RELEASE_MASK};

const CAP_SURROUNDING_TEXT: u32 = 1 << 5;
/// GTK resets the input context whenever it handles a BackSpace, including ones we forwarded.
/// A Reset within this window of our last forwarded key is that echo, not a real cursor move.
const RESET_ECHO_WINDOW_MS: u64 = 200;
/// Height of a browser's tab strip + toolbar; a caret above this line is in the address bar.
const BROWSER_TOOLBAR_PX: i32 = 80;
/// Extra wait after a settle pause, so keys that arrive right behind it join the same commit.
const COALESCE_GRACE_MS: u64 = 2;
const ATTR_TYPE_UNDERLINE: u32 = 1;
const ATTR_UNDERLINE_SINGLE: u32 = 1;

/// Process-wide state shared by every engine instance.
pub struct Shared {
    cfg: Mutex<(FileConfig, Option<SystemTime>)>,
    detector: Option<AppDetector>,
}

impl Shared {
    pub fn new() -> Arc<Self> {
        let detector = AppDetector::connect();
        if detector.is_none() {
            log::info!("no X11 display: per-app modes disabled");
        }
        Arc::new(Shared { cfg: Mutex::new((FileConfig::load(), config_mtime())), detector })
    }

    /// Re-reads the config file when it changed since the last look.
    fn with_config<R>(&self, f: impl FnOnce(&FileConfig) -> R) -> R {
        let mut guard = self.cfg.lock().unwrap();
        let mtime = config_mtime();
        if mtime != guard.1 {
            *guard = (FileConfig::load(), mtime);
            log::info!("config reloaded");
        }
        f(&guard.0)
    }
}

fn config_mtime() -> Option<SystemTime> {
    std::fs::metadata(FileConfig::path()).and_then(|m| m.modified()).ok()
}

fn now_ms() -> u64 {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}

pub struct Factory {
    shared: Arc<Shared>,
    next_id: AtomicU32,
}

impl Factory {
    pub fn new(shared: Arc<Shared>) -> Self {
        Factory { shared, next_id: AtomicU32::new(0) }
    }
}

#[interface(name = "org.freedesktop.IBus.Factory")]
impl Factory {
    async fn create_engine(
        &self,
        name: &str,
        #[zbus(object_server)] server: &ObjectServer,
        #[zbus(connection)] conn: &Connection,
    ) -> zbus::fdo::Result<OwnedObjectPath> {
        let method = match name {
            "gtv-telex" => Method::Telex,
            "gtv-vni" => Method::Vni,
            other => return Err(zbus::fdo::Error::Failed(format!("unknown engine {other}"))),
        };
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let path = OwnedObjectPath::try_from(format!("/org/freedesktop/IBus/Engine/GTV/{id}"))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        let cfg = self.shared.with_config(|c| c.composer_config(method));
        let clocks = Clocks {
            pending: Arc::new(AtomicUsize::new(0)),
            last_forward: Arc::new(AtomicU64::new(0)),
            last_mixed: Arc::new(AtomicU64::new(0)),
        };
        let queue = spawn_emitter(conn.clone(), path.clone(), clocks.clone());
        let engine = Engine {
            session: Session::new(cfg, Mode::Surrounding),
            shared: self.shared.clone(),
            caps: 0,
            path: path.clone(),
            underline: false,
            browser: false,
            pause: Duration::from_millis(15),
            queue,
            clocks,
        };
        server.at(&path, engine).await?;
        server.at(&path, Service { path: path.clone() }).await?;
        log::info!("created engine {name} at {path}");
        Ok(path)
    }
}

/// `org.freedesktop.IBus.Service`: the daemon calls Destroy when it is done with an engine.
struct Service {
    path: OwnedObjectPath,
}

#[interface(name = "org.freedesktop.IBus.Service")]
impl Service {
    async fn destroy(&self, #[zbus(object_server)] server: &ObjectServer) {
        let _ = server.remove::<Engine, _>(&self.path).await;
        let _ = server.remove::<Service, _>(&self.path).await;
    }
}

/// A batch of outputs for the emitter thread, with the settings in force when it was made.
struct Batch {
    outs: Vec<Out>,
    underline: bool,
    pause: Duration,
    /// Text travels as forwarded keys too (`forward-keys`), so one ordered stream carries it all.
    one_channel: bool,
}

#[derive(Clone)]
struct Clocks {
    /// Batches handed to the emitter and not yet fully sent.
    pending: Arc<AtomicUsize>,
    /// `now_ms()` when keys were last forwarded to the client (0 = never).
    last_forward: Arc<AtomicU64>,
    /// `now_ms()` when forwarded keys and text were last sent together (a batch with a pause).
    /// The client may still be working through those keys, so what follows has to wait.
    last_mixed: Arc<AtomicU64>,
}

/// Browsers whose address bar inline-autocompletes while typing.
fn is_browser(class: &str) -> bool {
    ["chrome", "chromium", "brave", "vivaldi", "edge", "opera", "firefox", "navigator"]
        .iter()
        .any(|b| class.contains(b))
}

struct Engine {
    session: Session,
    shared: Arc<Shared>,
    caps: u32,
    path: OwnedObjectPath,
    underline: bool,
    /// The focused app is a browser (see `is_browser`).
    browser: bool,
    pause: Duration,
    /// The only road to the client: one thread emits every signal, in order, so two senders can
    /// never race on the connection and a delayed batch can never be overtaken.
    queue: Sender<Batch>,
    clocks: Clocks,
}

/// Emits batches in order on its own thread. Handlers return to the client immediately, which
/// matters in sync mode: the client cannot process the keys we forward until it has our reply.
fn spawn_emitter(conn: Connection, path: OwnedObjectPath, clocks: Clocks) -> Sender<Batch> {
    let (tx, rx) = mpsc::channel::<Batch>();
    std::thread::spawn(move || {
        let Ok(em) = SignalEmitter::new(&conn, path) else { return };
        let mut carry: Option<Batch> = None;
        loop {
            let Some(mut batch) = carry.take().or_else(|| rx.recv().ok()) else { return };
            // The client may still be working through keys it was handed together with text;
            // whatever follows waits until `pause` has passed since then.
            let mixed = clocks.last_mixed.load(Ordering::SeqCst);
            if mixed != 0 {
                let ready_at = mixed + batch.pause.as_millis() as u64;
                let wait = ready_at.saturating_sub(now_ms());
                if wait > 0 {
                    // Keys typed while we waited are about to arrive: let them join this batch.
                    std::thread::sleep(Duration::from_millis(wait + COALESCE_GRACE_MS));
                }
            }
            let mut merged = 1;
            if is_plain(&batch) {
                // Plain text that is already waiting goes out as one commit: two commits sent
                // back to back can be applied in the wrong order by some clients.
                while let Ok(next) = rx.try_recv() {
                    if is_plain(&next)
                        && next.underline == batch.underline
                        && next.one_channel == batch.one_channel
                    {
                        batch.outs.extend(next.outs);
                        merged += 1;
                    } else {
                        carry = Some(next);
                        break;
                    }
                }
            }
            batch.outs = coalesce_commits(batch.outs);
            let forwards = batch.outs.iter().any(|o| matches!(o, Out::Key(_)));
            // Forwarded keys and committed text travel separately and the client may handle the
            // keys later, so whatever follows has to wait (see `pause_ms`).
            let is_mixed = batch.outs.contains(&Out::Pause) || (forwards && !batch.one_channel);
            zbus::block_on(emit(&em, batch.outs, batch.underline, batch.pause));
            if forwards {
                clocks.last_forward.store(now_ms(), Ordering::SeqCst);
            }
            if is_mixed {
                clocks.last_mixed.store(now_ms(), Ordering::SeqCst);
            }
            clocks.pending.fetch_sub(merged, Ordering::SeqCst);
        }
    });
    tx
}

fn is_plain(batch: &Batch) -> bool {
    batch.outs.iter().all(|o| matches!(o, Out::Commit(_)))
}

/// Joins runs of adjacent `Commit`s; everything else keeps its place.
fn coalesce_commits(outs: Vec<Out>) -> Vec<Out> {
    let mut merged: Vec<Out> = Vec::with_capacity(outs.len());
    for out in outs {
        match (merged.last_mut(), out) {
            (Some(Out::Commit(prev)), Out::Commit(next)) => prev.push_str(&next),
            (_, out) => merged.push(out),
        }
    }
    merged
}

async fn emit(em: &SignalEmitter<'_>, outs: Vec<Out>, underline: bool, pause: Duration) {
    for out in outs {
        log::trace!("emit {out:?}");
        let sent = match out {
            Out::Commit(text) => Engine::commit_text(em, ibus_text(&text, false)).await,
            Out::DeleteSurrounding(n) => {
                Engine::delete_surrounding_text(em, -(n as i32), n as u32).await
            }
            Out::Key(k) => Engine::forward_key_event(em, k.keyval, k.keycode, k.state).await,
            Out::Pause => {
                std::thread::sleep(pause);
                Ok(())
            }
            Out::Preedit(Some(text)) => {
                let cursor = text.chars().count() as u32;
                Engine::update_preedit_text(em, ibus_text(&text, underline), cursor, true, 0).await
            }
            Out::Preedit(None) => Engine::hide_preedit_text(em).await,
            Out::RequireSurroundingText => Engine::require_surrounding_text(em).await,
        };
        if let Err(e) = sent {
            log::warn!("signal failed: {e}");
        }
    }
}

impl Engine {
    /// True while earlier output is queued or the client may still be digesting forwarded keys.
    fn busy(&self) -> bool {
        let mixed = self.clocks.last_mixed.load(Ordering::SeqCst);
        self.clocks.pending.load(Ordering::SeqCst) > 0
            || (mixed != 0 && now_ms().saturating_sub(mixed) < self.pause.as_millis() as u64)
    }

    fn deliver(&mut self, outs: Vec<Out>) {
        if outs.is_empty() {
            return;
        }
        self.clocks.pending.fetch_add(1, Ordering::SeqCst);
        let one_channel = self.session.mode() == Mode::ForwardKeys;
        let batch = Batch { outs, underline: self.underline, pause: self.pause, one_channel };
        if self.queue.send(batch).is_err() {
            self.clocks.pending.fetch_sub(1, Ordering::SeqCst);
        }
    }

    /// Chrome's address bar and GTK answer a forwarded BackSpace with Reset and/or a focus
    /// change; acting on those would drop the word being typed.
    fn is_echo(&self) -> bool {
        let last = self.clocks.last_forward.load(Ordering::SeqCst);
        self.clocks.pending.load(Ordering::SeqCst) > 0
            || (last != 0 && now_ms().saturating_sub(last) < RESET_ECHO_WINDOW_MS)
    }

    fn focus(&mut self) {
        if self.is_echo() {
            log::trace!("focus change ignored: echo of our forwarded keys");
            return;
        }
        let class = self.shared.detector.as_ref().and_then(AppDetector::focused_class);
        let has_surrounding = self.caps & CAP_SURROUNDING_TEXT != 0;
        let (mode, underline, pause_ms) = self
            .shared
            .with_config(|c| (c.mode_for(class.as_deref()), c.preedit_underline(), c.pause_ms()));
        self.underline = underline;
        self.browser = class.as_deref().is_some_and(is_browser);
        self.session.set_clear_suggestion(false);
        self.pause = Duration::from_millis(pause_ms);
        let mode =
            if mode == Mode::Surrounding && !has_surrounding { Mode::Backspace } else { mode };
        log::debug!("focus in {:?}: {:?} ({})", class, mode, self.path);
        let mut outs = self.session.set_mode(mode);
        if mode == Mode::Surrounding {
            outs.push(Out::RequireSurroundingText);
        }
        self.deliver(outs);
    }

    fn flush(&mut self) {
        if self.is_echo() {
            log::trace!("reset ignored: echo of our forwarded keys");
            return;
        }
        let outs = self.session.reset();
        self.deliver(outs);
    }
}

#[interface(name = "org.freedesktop.IBus.Engine")]
impl Engine {
    fn process_key_event(&mut self, keyval: u32, keycode: u32, state: u32) -> bool {
        let (mut handled, mut outs) = self.session.process_key(KeyEvent { keyval, keycode, state });
        let ahead = self.busy() || !outs.is_empty();
        if ahead && !handled && state & RELEASE_MASK == 0 && !is_modifier(keyval) {
            // Our output reaches the client after this reply, so a key the application would
            // handle itself (Enter, arrows, shortcuts) must travel the same ordered road.
            outs.push(Out::Key(KeyEvent { keyval, keycode, state }));
            outs.push(Out::Key(KeyEvent { keyval, keycode, state: state | RELEASE_MASK }));
            self.session.consume_release_of(keyval);
            handled = true;
        }
        log::trace!("key {keyval:#x} state {state:#x} -> handled={handled} {outs:?}");
        self.deliver(outs);
        handled
    }

    fn focus_in(&mut self) {
        self.focus();
    }

    #[zbus(name = "FocusInId")]
    fn focus_in_id(&mut self, _object_path: &str, _client: &str) {
        self.focus();
    }

    fn focus_out(&mut self) {
        self.flush();
    }

    #[zbus(name = "FocusOutId")]
    fn focus_out_id(&mut self, _object_path: &str) {
        self.flush();
    }

    fn reset(&mut self) {
        self.flush();
    }

    fn enable(&mut self) {}

    fn disable(&mut self) {
        self.flush();
    }

    fn set_capabilities(&mut self, caps: u32) {
        log::debug!("capabilities {caps:#x}");
        self.caps = caps;
    }

    fn set_cursor_location(&mut self, x: i32, y: i32, w: i32, h: i32) {
        log::trace!("cursor location {x},{y} {w}x{h}");
        if !self.browser || (w == 0 && h == 0) {
            return;
        }
        let top = self.shared.detector.as_ref().and_then(AppDetector::focused_top);
        let in_toolbar = top.is_some_and(|t| (0..BROWSER_TOOLBAR_PX).contains(&(y - t)));
        self.session.set_clear_suggestion(in_toolbar);
    }

    fn set_surrounding_text(&self, text: Value<'_>, cursor_pos: u32, anchor_pos: u32) {
        log::trace!("surrounding {text:?} cursor={cursor_pos} anchor={anchor_pos}");
    }

    fn property_activate(&self, _name: &str, _state: u32) {}

    fn property_show(&self, _name: &str) {}

    fn property_hide(&self, _name: &str) {}

    fn candidate_clicked(&self, _index: u32, _button: u32, _state: u32) {}

    fn page_up(&self) {}

    fn page_down(&self) {}

    fn cursor_up(&self) {}

    fn cursor_down(&self) {}

    fn process_hand_writing_event(&self, _coordinates: Vec<f64>) {}

    fn cancel_hand_writing(&self, _n_strokes: u32) {}

    fn panel_extension_received(&self, _event: Value<'_>) {}

    fn panel_extension_register_keys(&self, _keys: Value<'_>) {}

    #[zbus(signal)]
    async fn commit_text(em: &SignalEmitter<'_>, text: Value<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn forward_key_event(
        em: &SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn delete_surrounding_text(
        em: &SignalEmitter<'_>,
        offset: i32,
        nchars: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn update_preedit_text(
        em: &SignalEmitter<'_>,
        text: Value<'_>,
        cursor_pos: u32,
        visible: bool,
        mode: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn hide_preedit_text(em: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn require_surrounding_text(em: &SignalEmitter<'_>) -> zbus::Result<()>;
}

fn serializable<'a>(
    name: &str,
    fields: impl FnOnce(StructureBuilder<'a>) -> StructureBuilder<'a>,
) -> Structure<'a> {
    let empty: HashMap<String, Value<'static>> = HashMap::new();
    let builder = StructureBuilder::new().add_field(name.to_string()).add_field(empty);
    fields(builder).build().expect("serializable has fields")
}

/// `IBusText`: `(sa{sv}sv)` wrapped in a variant, with an optional single underline.
fn ibus_text(text: &str, underline: bool) -> Value<'static> {
    let mut attrs: Vec<Value<'static>> = Vec::new();
    if underline {
        let end = text.chars().count() as u32;
        attrs.push(Value::new(serializable("IBusAttribute", |b| {
            b.add_field(ATTR_TYPE_UNDERLINE)
                .add_field(ATTR_UNDERLINE_SINGLE)
                .add_field(0u32)
                .add_field(end)
        })));
    }
    let attr_list = Value::new(serializable("IBusAttrList", |b| b.add_field(attrs)));
    Value::new(serializable("IBusText", |b| b.add_field(text.to_string()).add_field(attr_list)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_commits_become_one() {
        let outs = vec![
            Out::Commit("a".into()),
            Out::Commit("b".into()),
            Out::Pause,
            Out::Commit("c".into()),
            Out::Commit("d".into()),
        ];
        assert_eq!(
            coalesce_commits(outs),
            vec![Out::Commit("ab".into()), Out::Pause, Out::Commit("cd".into())]
        );
    }

    #[test]
    fn keys_and_deletes_are_never_merged_across() {
        let outs =
            vec![Out::Commit("a".into()), Out::DeleteSurrounding(1), Out::Commit("b".into())];
        assert_eq!(coalesce_commits(outs.clone()), outs);
    }
}
