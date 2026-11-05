//! Role: Normalize literal and typed shortcuts to localized keyboard key caps.
use crate::kbd_values::KbdKeyState;
use kivact_core::platform::CommandKey;
use kivact_core::state::Value;
use kivact_core::view::Key;
use std::collections::HashMap;

pub(super) struct KeyNames {
    words: HashMap<&'static str, Value<String>>,
}
impl KeyNames {
    pub(super) fn new() -> Self {
        Self {
            words: [
                ("command", "kbd.key.command", "Command"),
                ("control", "kbd.key.control", "Control"),
                ("control_short", "kbd.key.control_short", "Ctrl"),
                ("alt", "kbd.key.alt", "Alt"),
                ("shift", "kbd.key.shift", "Shift"),
                ("enter", "kbd.key.enter", "Enter"),
                ("backspace", "kbd.key.backspace", "Backspace"),
                ("tab", "kbd.key.tab", "Tab"),
                ("up", "kbd.key.up", "Up arrow"),
                ("down", "kbd.key.down", "Down arrow"),
                ("left", "kbd.key.left", "Left arrow"),
                ("right", "kbd.key.right", "Right arrow"),
                ("plus", "kbd.key.plus", "Plus"),
                ("escape", "kbd.key.escape", "Escape"),
                ("escape_short", "kbd.key.escape_short", "Esc"),
                ("space", "kbd.key.space", "Space"),
            ]
            .into_iter()
            .map(|(name, key, fallback)| (name, kivact_core::i18n::message(key, fallback)))
            .collect(),
        }
    }
    fn word(&self, name: &str) -> String {
        self.words
            .get(name)
            .expect("invariant: normalized key has a localized word")
            .get()
    }
    pub(super) fn literal(&self, token: &str, command: CommandKey) -> KbdKeyState {
        let token = token.trim();
        let lower = token.to_ascii_lowercase();
        let (glyph, name) = match lower.as_str() {
            "mod" if command == CommandKey::Command => ("⌘", "command"),
            "mod" => {
                return KbdKeyState {
                    glyph: self.word("control_short"),
                    label: self.word("control"),
                };
            }
            "cmd" | "command" => ("⌘", "command"),
            "ctrl" | "control" => ("⌃", "control"),
            "alt" => ("⌥", "alt"),
            "shift" => ("⇧", "shift"),
            "enter" | "return" => ("↵", "enter"),
            "backspace" => ("⌫", "backspace"),
            "tab" => ("⇥", "tab"),
            "up" | "arrowup" => ("↑", "up"),
            "down" | "arrowdown" => ("↓", "down"),
            "left" | "arrowleft" => ("←", "left"),
            "right" | "arrowright" => ("→", "right"),
            "plus" => ("+", "plus"),
            "escape" | "esc" => {
                return KbdKeyState {
                    glyph: self.word("escape_short"),
                    label: self.word("escape"),
                };
            }
            "space" => {
                return KbdKeyState {
                    glyph: self.word("space"),
                    label: self.word("space"),
                };
            }
            _ => {
                return KbdKeyState {
                    glyph: token.to_uppercase(),
                    label: token.to_uppercase(),
                };
            }
        };
        KbdKeyState {
            glyph: glyph.into(),
            label: self.word(name),
        }
    }
    pub(super) fn typed(&self, key: Key, command: CommandKey) -> KbdKeyState {
        let token = match key {
            Key::Char('+') => "plus".into(),
            Key::Char(c) => c.to_string(),
            Key::Enter => "enter".into(),
            Key::Tab => "tab".into(),
            Key::Space => "space".into(),
            Key::Backspace => "backspace".into(),
            Key::Escape => "escape".into(),
            Key::ArrowUp => "up".into(),
            Key::ArrowDown => "down".into(),
            Key::ArrowLeft => "left".into(),
            Key::ArrowRight => "right".into(),
            Key::F(n) => format!("F{n}"),
            other => format!("{other:?}"),
        };
        self.literal(&token, command)
    }
}
