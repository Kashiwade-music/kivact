//! Role: IME input and composition handling.
//!
//! The platform IME's side of text input beyond composing and committing:
//! what the host answers the IME's queries about the focused text input
//! (on Windows the window answers `WM_IME_REQUEST` from it: reconversion,
//! the text around the composition, character positions), and the range
//! an IME takes for reconversion, which the reconversion's composition
//! then replaces.
//!
//! The host speaks UTF-16 code units (what platform IMEs count); tests
//! speak chars of the value, converted here.

use std::ops::Range;

use kivact_engine::testing::ShellEvent;
use kivact_style::draw::Rect;

use super::model::{Element, Screen};
use super::results::ImeText;

impl Screen {
    /// What the host answers the platform IME about the focused text
    /// input: the text around its caret, the selection and composition in
    /// it, and where the composed characters and the caret are (`None`: no
    /// text input has the focus).
    pub fn ime_text(&self) -> Option<ImeText> {
        let text = self.host.borrow().ime_text()?;
        let value = self
            .focused()
            .and_then(|f| self.host.borrow().input_value(f.node))
            .unwrap_or_default();
        let start = value
            .chars()
            .scan(0, |units, c| {
                let at = *units;
                *units += c.len_utf16();
                Some(at)
            })
            .take_while(|at| *at < text.text_start)
            .count();
        Some(ImeText::from_host(text, start))
    }

    /// The platform ending its IME session without a commit
    /// (`ImeDisabled`): an open composition is gone, and a node with an IME
    /// session (`NodeExt::ime_session`) gets `ImeEvent::End`.
    pub fn ime_disabled(&self) {
        self.send(ShellEvent::ImeDisabled);
        self.run_until_idle();
    }

    /// Where the window puts the IME's candidate window: the caret of the
    /// focused text input in window coordinates, as drawn (`None`: no text
    /// input has the focus).
    pub fn ime_cursor_area(&self) -> Option<Rect> {
        self.host.borrow().ime_area()
    }
}

impl Element<'_> {
    /// Focus the element (unless focused) and have the IME take `range`
    /// (chars of the value, inside the text the IME is given:
    /// [`Screen::ime_text`]) for reconversion, as the Windows IME confirms
    /// the range it reconverts: it becomes the selection, which the next
    /// composition ([`Element::compose`], [`Element::commit`]) replaces.
    #[track_caller]
    pub fn reconvert(&self, range: Range<usize>) {
        self.ensure_focus();
        self.screen
            .log(format!("reconvert {range:?} in {}", self.short()));
        let mut value = self
            .screen
            .host
            .borrow()
            .input_value(self.node)
            .unwrap_or_default()
            .to_string();
        if value.is_empty() {
            // A node with an IME session (`NodeExt::ime_session`) has no
            // native value: the IME was given the session's text.
            value = self.screen.ime_text().map(|t| t.text).unwrap_or_default();
        }
        // The IME answers in the text it was given.
        let base = self
            .screen
            .host
            .borrow()
            .ime_text()
            .map_or(0, |t| t.text_start);
        let unit = |chars: usize| -> usize {
            let units: usize = value.chars().take(chars).map(char::len_utf16).sum();
            units.saturating_sub(base)
        };
        self.screen.send(ShellEvent::ImeSelection {
            start: unit(range.start),
            end: unit(range.end),
        });
        self.screen.run_until_idle();
    }
}
