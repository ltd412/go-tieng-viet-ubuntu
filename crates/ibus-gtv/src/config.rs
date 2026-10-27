//! `~/.config/gtv/config.toml`, all keys optional:
//!
//! ```toml
//! tone_style = "modern"      # or "old"  (hòa/thủy vs hoà/thuỷ)
//! free_marking = true
//! spell_check = true
//! pause_ms = 30              # wait after forwarded keys, before committing replacement text
//! preedit_underline = false  # underline the word while composing in `preedit` mode
//!
//! [apps]                     # WM_CLASS substring (case-insensitive) -> delivery mode
//! "gnome-text-editor" = "surrounding"
//! "some-app" = "preedit"
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

use gtv_core::{Config, Method, ToneStyle};
use serde::Deserialize;

use crate::session::Mode;

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct FileConfig {
    tone_style: Option<ToneStyleName>,
    free_marking: Option<bool>,
    spell_check: Option<bool>,
    /// Underline the word while it is composed in `preedit` mode. Off by default: the text
    /// then looks like ordinary typed text but is still replaced atomically.
    preedit_underline: Option<bool>,
    /// Milliseconds to wait between forwarded keys and the text that replaces them.
    pause_ms: Option<u64>,
    /// Mode for apps that match nothing below.
    default_mode: Option<Mode>,
    /// Mode while the caret is in a browser address bar. Chrome answers the engine's
    /// BackSpace + commit with its inline autocomplete a moment later and can overwrite the
    /// text, so the address bar defaults to `preedit`, which edits nothing behind its back. Measured
    /// through ibus-portal: `backspace` still lost characters in most omnibox runs even with the
    /// longer address-bar pause, `preedit` did not.
    address_bar_mode: Option<Mode>,
    apps: HashMap<String, Mode>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ToneStyleName {
    Modern,
    Old,
}

/// Apps that deserve something other than the default (`Mode::Backspace`).
///
/// `Backspace` (forwarded BackSpace, a short pause, then CommitText) is the only mode that was
/// correct in every toolkit tried (GTK3, VTE, Chromium/Electron). `Surrounding` is cleaner where
/// the client really implements it, but Chromium advertises the capability and then ignores
/// DeleteSurroundingText, so it is only listed for apps verified to honour it.
const BUILTIN_APPS: &[(&str, Mode)] =
    &[("gnome-text-editor", Mode::Surrounding), ("gedit", Mode::Surrounding)];

impl FileConfig {
    pub fn path() -> PathBuf {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_default();
        base.join("gtv/config.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
                log::warn!("ignoring {}: {e}", path.display());
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    pub fn composer_config(&self, method: Method) -> Config {
        let mut cfg = Config { method, ..Config::default() };
        if let Some(style) = self.tone_style {
            cfg.tone_style = match style {
                ToneStyleName::Modern => ToneStyle::Modern,
                ToneStyleName::Old => ToneStyle::Old,
            };
        }
        cfg.free_marking = self.free_marking.unwrap_or(cfg.free_marking);
        cfg.spell_check = self.spell_check.unwrap_or(cfg.spell_check);
        cfg
    }

    pub fn pause_ms(&self) -> u64 {
        self.pause_ms.unwrap_or(30)
    }

    pub fn address_bar_mode(&self) -> Mode {
        self.address_bar_mode.unwrap_or(Mode::Preedit)
    }

    pub fn preedit_underline(&self) -> bool {
        self.preedit_underline.unwrap_or(false)
    }

    /// Delivery mode for the app identified by `wm_class` ("instance class", lowercase).
    /// User entries win over built-ins; the longest matching key wins within each list.
    /// `gtk4` marks a GTK4 app: GTK4 ignores the BackSpace the engine forwards (the old text
    /// stays and `ee` becomes `eê`) but implements DeleteSurroundingText, so apps that match no
    /// entry and have no user-chosen default get `Surrounding`.
    pub fn mode_for(&self, wm_class: Option<&str>, gtk4: bool) -> Mode {
        let fallback =
            self.default_mode.unwrap_or(if gtk4 { Mode::Surrounding } else { Mode::Backspace });
        let Some(class) = wm_class.map(str::to_lowercase) else { return fallback };
        let best = |entries: &mut dyn Iterator<Item = (&str, Mode)>| {
            entries
                .filter(|(k, _)| class.contains(&k.to_lowercase()))
                .max_by_key(|(k, _)| k.len())
                .map(|(_, m)| m)
        };
        best(&mut self.apps.iter().map(|(k, m)| (k.as_str(), *m)))
            .or_else(|| best(&mut BUILTIN_APPS.iter().copied()))
            .unwrap_or(fallback)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gtk4_apps_default_to_surrounding_unless_configured() {
        let cfg = FileConfig::default();
        assert_eq!(cfg.mode_for(Some("nautilus org.gnome.nautilus"), true), Mode::Surrounding);
        assert_eq!(cfg.mode_for(None, true), Mode::Surrounding);
        assert_eq!(cfg.mode_for(Some("nautilus org.gnome.nautilus"), false), Mode::Backspace);
        let cfg: FileConfig = toml::from_str(
            r#"
            default_mode = "preedit"
            [apps]
            "nautilus" = "off"
            "#,
        )
        .unwrap();
        assert_eq!(cfg.mode_for(Some("some-gtk4-app"), true), Mode::Preedit);
        assert_eq!(cfg.mode_for(Some("nautilus org.gnome.nautilus"), true), Mode::Off);
    }

    #[test]
    fn terminals_can_be_switched_off() {
        let cfg: FileConfig = toml::from_str(
            r#"
            [apps]
            "gnome-terminal" = "off"
            "#,
        )
        .unwrap();
        assert_eq!(
            cfg.mode_for(Some("gnome-terminal-server gnome-terminal-server"), false),
            Mode::Off
        );
        assert_eq!(cfg.mode_for(Some("code code"), false), Mode::Backspace);
    }

    #[test]
    fn user_entry_beats_builtin_and_longest_wins() {
        let cfg: FileConfig = toml::from_str(
            r#"
            tone_style = "old"
            [apps]
            "gnome-terminal" = "preedit"
            "code-insiders" = "select"
            "#,
        )
        .unwrap();
        assert_eq!(
            cfg.mode_for(Some("gnome-terminal-server Gnome-terminal"), false),
            Mode::Preedit
        );
        assert_eq!(cfg.mode_for(Some("code-insiders Code - Insiders"), false), Mode::Select);
        assert_eq!(cfg.mode_for(Some("code Code"), false), Mode::Backspace);
        assert_eq!(cfg.composer_config(Method::Telex).tone_style, ToneStyle::Old);
    }

    #[test]
    fn unknown_app_gets_the_universal_mode() {
        let cfg = FileConfig::default();
        assert_eq!(cfg.mode_for(Some("gedit Gedit"), false), Mode::Surrounding);
        assert_eq!(cfg.mode_for(Some("google-chrome google-chrome"), false), Mode::Backspace);
        assert_eq!(
            cfg.mode_for(Some("gnome-terminal-server Gnome-terminal"), false),
            Mode::Backspace
        );
        assert_eq!(cfg.mode_for(None, false), Mode::Backspace);
    }

    #[test]
    fn address_bar_defaults_to_preedit_and_is_configurable() {
        assert_eq!(FileConfig::default().address_bar_mode(), Mode::Preedit);
        let cfg: FileConfig = toml::from_str("address_bar_mode = \"backspace\"").unwrap();
        assert_eq!(cfg.address_bar_mode(), Mode::Backspace);
    }

    #[test]
    fn default_mode_is_configurable() {
        let cfg: FileConfig = toml::from_str("default_mode = \"preedit\"").unwrap();
        assert_eq!(cfg.mode_for(Some("anything"), false), Mode::Preedit);
    }
}
