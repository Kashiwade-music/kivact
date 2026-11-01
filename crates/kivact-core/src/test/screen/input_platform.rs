//! Role: Portable platform event injection through the real test window.
use super::model::Screen;
use crate::draw::Point;
use crate::view::{FilePhase, Key, KeyCode, Modifiers};
use kivact_engine::testing::ShellEvent;
impl Screen {
    /// Inject a key press at a physical keyboard position.
    pub fn key_down(&self, key: Key, physical_code: KeyCode, modifiers: Modifiers) {
        self.send(ShellEvent::Modifiers(modifiers));
        let physical = match physical_code {
            KeyCode::KeyA => Some('a'),
            KeyCode::KeyB => Some('b'),
            KeyCode::KeyC => Some('c'),
            KeyCode::KeyD => Some('d'),
            KeyCode::KeyE => Some('e'),
            KeyCode::KeyF => Some('f'),
            KeyCode::KeyG => Some('g'),
            KeyCode::KeyH => Some('h'),
            KeyCode::KeyI => Some('i'),
            KeyCode::KeyJ => Some('j'),
            KeyCode::KeyK => Some('k'),
            KeyCode::KeyL => Some('l'),
            KeyCode::KeyM => Some('m'),
            KeyCode::KeyN => Some('n'),
            KeyCode::KeyO => Some('o'),
            KeyCode::KeyP => Some('p'),
            KeyCode::KeyQ => Some('q'),
            KeyCode::KeyR => Some('r'),
            KeyCode::KeyS => Some('s'),
            KeyCode::KeyT => Some('t'),
            KeyCode::KeyU => Some('u'),
            KeyCode::KeyV => Some('v'),
            KeyCode::KeyW => Some('w'),
            KeyCode::KeyX => Some('x'),
            KeyCode::KeyY => Some('y'),
            KeyCode::KeyZ => Some('z'),
            KeyCode::Unidentified => match key {
                Key::Char(c) if c.is_ascii_alphabetic() => Some(c.to_ascii_lowercase()),
                _ => None,
            },
            _ => None,
        };
        self.send(ShellEvent::Key {
            key,
            physical,
            code: physical_code,
            repeat: false,
            text: None,
        });
        self.run_until_idle();
    }
    /// Inject a release; it cannot type text or activate a control.
    pub fn key_up(&self, key: Key, physical_code: KeyCode, modifiers: Modifiers) {
        self.send(ShellEvent::Modifiers(modifiers));
        self.send(ShellEvent::KeyUp {
            key,
            code: physical_code,
        });
        self.run_until_idle();
    }
    /// Inject one OS file drag event with a window position.
    pub fn file_drag(&self, phase: FilePhase, files: Vec<std::path::PathBuf>, point: Point) {
        self.send(ShellEvent::File {
            phase,
            files,
            position: kivact_engine::testing::Point {
                x: point.x,
                y: point.y,
            },
        });
        self.run_until_idle();
    }
}
