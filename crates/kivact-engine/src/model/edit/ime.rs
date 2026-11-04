//! Role: IME composition, commit and selection applied to the focused text input.

use super::state::{ImeGiven, ImeUpdate, InputSource};
use crate::model::Model;
use kivact_editing::EditState;
use kivact_types::ImeEvent;

impl Model {
    /// The IME's composition (`selection` in bytes of `text`); an empty one
    /// ends a composition, removing its text.
    pub(crate) fn preedit(&mut self, text: &str, selection: Option<(usize, usize)>) {
        if self.session_preedit(text, selection) {
            return;
        }
        if text.is_empty() {
            let composing = self
                .focused_input()
                .and_then(|i| self.node(i))
                .and_then(|n| {
                    self.editors[n.index.0]
                        .as_deref()
                        .map(|editor| editor.editor.state())
                })
                .is_some_and(EditState::is_composing);
            if composing {
                self.compose_text(None, "", None);
            }
        } else {
            let selected = selection.map(|(a, b)| a.min(b)..a.max(b));
            self.compose_text(None, text, selected);
        }
        self.sync();
    }

    /// Text the IME committed.
    pub(crate) fn commit(&mut self, text: &str) {
        if self.session_event(ImeEvent::Commit(text.to_string())) {
            return;
        }
        self.insert_text(text);
        self.sync();
    }

    /// The IME ended its session: the composition is gone.
    pub(crate) fn ime_disabled(&mut self) {
        if !self.session_event(ImeEvent::End) {
            self.preedit("", None);
        }
    }

    /// The IME took `start..end` (UTF-16 code units of the text it was
    /// last given, which starts at byte `start` of `ImeGiven::Input` in the
    /// focused input's value) for reconversion: it becomes the selection, which the
    /// reconversion's composition replaces. Ignored while composing.
    pub(crate) fn ime_select(&mut self, start: usize, end: usize) {
        if self.session_select(start, end) {
            return;
        }
        let Some(id) = self.focused_input() else {
            return;
        };
        if self.node(id).is_some_and(|n| n.password() || n.read_only()) {
            return;
        }
        let ImeGiven::Input {
            source,
            start: Some(base),
        } = &self.edits.given
        else {
            return;
        };
        if source.node != id {
            return;
        }
        let base = *base;
        let Some((start, end)) = self
            .node(id)
            .and_then(|n| {
                self.editors[n.index.0]
                    .as_deref()
                    .map(|editor| editor.editor.state())
            })
            .filter(|e| !e.is_composing())
            .and_then(|e| e.value().get(base..))
            .map(|text| {
                let at = |units| {
                    base + crate::model::interact::text_geometry::byte_of_utf16(text, units)
                };
                (at(start), at(end))
            })
        else {
            return;
        };
        self.place_caret(id, start.min(end), false);
        self.place_caret(id, start.max(end), true);
        self.sync();
    }

    /// The IME text to give the window, when it changed since the last one
    /// given.
    pub(crate) fn ime_changed(&mut self, engine: &mut arvane::Engine) -> ImeUpdate {
        if let Some(id) = self.focused_session() {
            return match self.session_ime_changed(id) {
                Some(text) => ImeUpdate::Set(Some(text)),
                None => ImeUpdate::Unchanged,
            };
        }
        let now = self.focused_input().and_then(|id| {
            let n = self.node(id)?;
            let area = crate::model::interact::text_geometry::text_line(self, id, engine);
            let transform = self.transform_between(id, None);
            let editor = self.editors[n.index.0].as_deref()?;
            Some(InputSource {
                node: id,
                revision: editor.editor.revision(),
                session: editor.editor.state().session().clone(),
                area,
                scroll: editor.scroll_x,
                transform,
                password: n.password(),
                read_only: n.read_only(),
            })
        });
        let same = match (&self.edits.given, &now) {
            (ImeGiven::Nothing, None) => true,
            (ImeGiven::Input { source, .. }, Some(now)) => source == now,
            _ => false,
        };
        if same {
            return ImeUpdate::Unchanged;
        }
        let text = crate::model::interact::text_geometry::ime_text(self, engine);
        self.edits.given = match now {
            None => ImeGiven::Nothing,
            Some(source) => ImeGiven::Input {
                source,
                start: text.as_ref().map(|(_, start)| *start),
            },
        };
        ImeUpdate::Set(text.map(|(text, _)| text))
    }
}
