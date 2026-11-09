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

/// What the platform is offered for the IME: either not allowed, or allowed
/// at an area with the surrounding text.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ImeOffer {
    /// No text input has the focus: the platform shows no IME.
    Disallowed,
    /// The IME is allowed at `area`, with the surrounding `text`.
    Allowed {
        /// The caret area, in logical window pixels.
        area: Rect,
        /// The text around the caret, when the platform asks for it.
        text: Option<ShellImeText>,
    },
}

/// Output cache: each offer reaches the platform only when it changes (S-9).
#[derive(Default)]
pub struct ImeDriver {
    offer: Sent<ImeOffer>,
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
        let next = match (allowed, self.offer.get()) {
            (true, Some(ImeOffer::Allowed { area, text })) => ImeOffer::Allowed {
                area: *area,
                text: text.clone(),
            },
            (true, _) => ImeOffer::Allowed {
                area: Rect::default(),
                text: None,
            },
            (false, _) => ImeOffer::Disallowed,
        };
        if self.offer.update(next).is_some() {
            port.allow(allowed);
        }
    }
    pub fn set_cursor_area(&mut self, port: &mut dyn ImePort, area: Rect) {
        let Some(ImeOffer::Allowed { text, .. }) = self.offer.get() else {
            // The IME is not allowed: nothing is shown, so there is no area to offer.
            return;
        };
        let next = ImeOffer::Allowed {
            area,
            text: text.clone(),
        };
        if self.offer.update(next).is_some() {
            port.cursor_area(area);
        }
    }
    pub fn set_text(&mut self, port: &mut dyn ImePort, text: Option<ShellImeText>) {
        let Some(ImeOffer::Allowed { area, .. }) = self.offer.get() else {
            return;
        };
        let area = *area;
        let next = ImeOffer::Allowed {
            area,
            text: text.clone(),
        };
        if self.offer.update(next).is_some() {
            port.surrounding(text.as_ref());
        }
    }
}
