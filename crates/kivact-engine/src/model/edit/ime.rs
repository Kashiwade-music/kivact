//! Role: IME composition, commit and selection applied to the focused text input.

use crate::model::Model;
use kivact_editing::EditState;
use kivact_wire::ImeEvent;

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
    /// last given, which starts at byte `ime_start` of the focused input's
    /// value) for reconversion: it becomes the selection, which the
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
        let base = self.edits.ime_start;
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

    /// The IME text to give the window, when it changed since the last
    /// one given (`Some(None)`: no text input has the focus any more).
    pub(crate) fn ime_changed(
        &mut self,
        engine: &mut arvane::Engine,
    ) -> Option<Option<arvane::ShellImeText>> {
        if let Some(id) = self.focused_session() {
            // A native editor gives its text again when it gets the focus back.
            self.edits.ime_from = None;
            return self.session_ime_changed(id).map(Some);
        }
        if self.edits.session_from.take().is_some() {
            self.edits.ime_from = None;
        }
        let now = self.focused_input().and_then(|id| {
            let n = self.node(id)?;
            let area = crate::model::interact::text_geometry::text_line(self, id, engine);
            let transform = self.transform_between(id, None);
            let edit = self.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())?;
            Some((
                id,
                (
                    self.editors[n.index.0].as_deref()?.editor.revision(),
                    edit.session(),
                ),
                area,
                self.editors[n.index.0]
                    .as_deref()
                    .map_or(0.0, |node| node.scroll_x),
                transform,
                (n.password(), n.read_only()),
            ))
        });
        let same = match (&self.edits.ime_from, &now) {
            (Some(None), None) => true,
            (
                Some(Some((id, edit, area, scroll, t, flags))),
                Some((id2, edit2, area2, scroll2, t2, flags2)),
            ) => {
                id == id2
                    && (edit.0, &edit.1) == *edit2
                    && area == area2
                    && scroll == scroll2
                    && t == t2
                    && flags == flags2
            }
            _ => false,
        };
        if same {
            return None;
        }
        self.edits.ime_from = Some(
            now.map(|(id, (revision, session), area, scroll, t, flags)| {
                (id, (revision, session.clone()), area, scroll, t, flags)
            }),
        );
        let text = crate::model::interact::text_geometry::ime_text(self, engine);
        self.edits.ime_start = text.as_ref().map_or(0, |(_, start)| *start);
        Some(text.map(|(text, _)| text))
    }
}
