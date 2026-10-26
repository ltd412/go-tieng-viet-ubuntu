//! Identifies the focused application on X11 so each app can get its own delivery mode.
//! Wayland offers no equivalent, so detection simply yields `None` there.

use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};
use x11rb::rust_connection::RustConnection;

pub struct AppDetector {
    conn: RustConnection,
}

impl AppDetector {
    pub fn connect() -> Option<Self> {
        let (conn, _) = RustConnection::connect(None).ok()?;
        Some(AppDetector { conn })
    }

    /// `"instance class"` of the toplevel owning the input focus, lowercase.
    pub fn focused_class(&self) -> Option<String> {
        let mut win = self.conn.get_input_focus().ok()?.reply().ok()?.focus;
        // 0 = None, 1 = PointerRoot: nothing useful to inspect.
        for _ in 0..32 {
            if win <= 1 {
                return None;
            }
            if let Some(class) = self.wm_class(win) {
                return Some(class);
            }
            let tree = self.conn.query_tree(win).ok()?.reply().ok()?;
            if tree.parent == tree.root || tree.parent == 0 {
                return None;
            }
            win = tree.parent;
        }
        None
    }

    /// Screen y of the top edge of the focused toplevel's client area.
    pub fn focused_top(&self) -> Option<i32> {
        let mut win = self.conn.get_input_focus().ok()?.reply().ok()?.focus;
        for _ in 0..32 {
            if win <= 1 {
                return None;
            }
            if self.wm_class(win).is_some() {
                let root = self.conn.get_geometry(win).ok()?.reply().ok()?.root;
                let pos = self.conn.translate_coordinates(win, root, 0, 0).ok()?.reply().ok()?;
                return Some(i32::from(pos.dst_y));
            }
            let tree = self.conn.query_tree(win).ok()?.reply().ok()?;
            if tree.parent == tree.root || tree.parent == 0 {
                return None;
            }
            win = tree.parent;
        }
        None
    }

    fn wm_class(&self, win: u32) -> Option<String> {
        let reply = self
            .conn
            .get_property(false, win, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 256)
            .ok()?
            .reply()
            .ok()?;
        if reply.value.is_empty() {
            return None;
        }
        let parts: Vec<_> = reply
            .value
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .map(String::from_utf8_lossy)
            .collect();
        Some(parts.join(" ").to_lowercase())
    }
}
