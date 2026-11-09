//! Role: Normalize composition input and deduplicate IME output.
use super::RawIme;
use crate::{ShellEvent, ShellImeText};
use kivact_types::cell::Sent;
use kivact_types::{ImeInput, Rect};

pub trait ImePort {
    fn allow(&mut self, allowed: bool);
    fn cursor_area(&mut self, area: Rect);
    fn surrounding(&mut self, text: Option<&ShellImeText>);
}
/// Output caches: each value reaches the platform only when it changes (S-9).
#[derive(Default)]
pub struct ImeDriver {
    allowed: Sent<bool>,
    area: Sent<Rect>,
    text: Sent<Option<ShellImeText>>,
}
impl ImeDriver {
    pub fn input(&mut self, raw: RawIme) -> Vec<ShellEvent> {
        let input = match raw {
            RawIme::Preedit { text, caret } => ImeInput::Preedit {
                text,
                selection: caret.map(|(start, end)| start..end),
            },
            RawIme::Commit(text) => ImeInput::Commit(text),
            RawIme::End => ImeInput::End,
            RawIme::Reconvert {
                selection: (start, end),
            } => ImeInput::Selection(start..end),
        };
        vec![ShellEvent::Ime(input)]
    }
    pub fn set_allowed(&mut self, port: &mut dyn ImePort, allowed: bool) {
        if let Some(value) = self.allowed.update(allowed) {
            port.allow(*value);
        }
    }
    pub fn set_cursor_area(&mut self, port: &mut dyn ImePort, area: Rect) {
        if let Some(value) = self.area.update(area) {
            port.cursor_area(*value);
        }
    }
    pub fn set_text(&mut self, port: &mut dyn ImePort, text: Option<ShellImeText>) {
        if let Some(value) = self.text.update(text) {
            port.surrounding(value.as_ref());
        }
    }
}
