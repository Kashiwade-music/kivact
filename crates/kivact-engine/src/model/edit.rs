//! Role: The text inputs' editing state, its index, and the IME's view of it.
//!
//! The edit group: which nodes are text inputs with a native editing
//! state (`HostNode::edit`), and the IME text last given to the window;
//! with the writes that keep a node's editing state in step with its
//! `Editing` prop and controlled value, and the IME's entry points.
//!
//! Invariants:
//! - [`Model::set_edit`] is the only place a node gains or loses its
//!   editing state, and it keeps the index of text inputs with it: the
//!   index lists exactly the live nodes with one.
//! - A controlled value replaces the value but keeps the composition
//!   (`EditState::set_controlled`); one given while composing is held and
//!   taken, as a recorded change in the same `TextChanged`, by the edit
//!   that ends the composition (`Model::adopt_held`). A node's first value
//!   starts with the caret at the end.
//! - A controlled value is known by its `Rc`: the same one given again
//!   while the text keeps the revision it was given at changes nothing
//!   without comparing texts. Texts are compared only for another value,
//!   or for the same one after an edit. A controlled value queues no
//!   `TextChanged` (it is the application's own), and is taken by a
//!   disabled or read-only input too.
//! - A state kivact gives back changes the text by the changes it
//!   recorded since it was read; texts are compared only for a state read
//!   at another revision: another text is taken whole, as one range
//!   replacement, and the same text replaces only the session, keeping
//!   its revision (`Taken::Same`). It goes through the editing entry
//!   point, so a disabled input refuses it and a read-only one takes only
//!   its selection.
//! - A session kivact gives (`Model::replace_edit_session`) replaces the
//!   selection and the composition without reading or copying the text,
//!   clamped to its char boundaries; a disabled input refuses it, a
//!   read-only one takes only the selection, and a composition it starts
//!   or ends queues a `TextChanged`.
//! - A text input that reports its session queues one `SelectionChanged`
//!   whenever its session is not the one it last reported, after the
//!   entry point that changed it (`Model::sync`).
//! - The IME is given its text only when the text's revision, the session
//!   or the geometry changed since the last time. The offsets the IME
//!   answers with are in the text it was last given, which starts at
//!   `Edits::ime_start` in the value.
//! - The range the IME takes for reconversion (`ShellEvent::ImeSelection`,
//!   [`Model::ime_select`]) becomes the selection, which the composition
//!   that follows replaces. It is ignored while composing, and in a
//!   password or read-only input.

use std::cell::RefMut;
use std::rc::Rc;

use crate::measure::editor::EditorNode;
#[cfg(all(debug_assertions, feature = "testing"))]
use crate::measure::input::RefreshedLines;
use crate::model::Model;
use crate::mutation::Mutation;
use crate::tree::node::HostNode;
use kivact_editing::Editor;
use kivact_editing::{EditState, Taken, TextSession};
use kivact_text::EditLines;
#[cfg(all(debug_assertions, feature = "testing"))]
use kivact_text::{RefreshError, RefreshMode};
use kivact_wire::{HostEvent, ImeEvent, NodeId};

mod session;
pub(crate) mod state;

impl Model {
    /// Give `id` the editing state `edit` (`None`: it is no text input, or
    /// it was removed), keeping the index of text inputs with it.
    pub(crate) fn set_edit(
        &mut self,
        id: NodeId,
        edit: Option<EditState>,
    ) -> Option<Box<EditorNode>> {
        let slot = self.tree.nodes.index_of(id);
        let present = edit.is_some() && slot.is_some();
        let previous = slot.and_then(|slot| {
            std::mem::replace(
                &mut self.editors[slot.0],
                edit.map(|state| Box::new(EditorNode::new(state))),
            )
        });
        if present {
            self.edits.inputs.insert(id);
        } else {
            self.edits.inputs.remove(&id);
        }
        previous
    }

    /// The `Editing` prop of `id` changed: it keeps its editing state with
    /// the new controlled value, gains one, or loses it with its text
    /// scroll.
    pub(crate) fn editing_changed(&mut self, id: NodeId) -> Option<Box<EditorNode>> {
        let node = self.tree.get_mut(id)?;
        let Some(data) = &node.properties.editing else {
            return self.set_edit(id, None);
        };
        let value = data.value.as_deref().unwrap_or("");
        let edit = match &mut self.editors[node.index.0] {
            Some(edit) => {
                if edit.editor.state().value() != value {
                    edit.held = !edit.editor.set_controlled(value);
                }
                return None;
            }
            None => EditState::new(value),
        };
        self.set_edit(id, Some(edit))
    }

    /// kivact gave `id` the controlled value `value`: its editing state
    /// takes it (the node's first value, with the caret at the end), and
    /// is measured again; whether its text changed.
    pub(crate) fn controlled_value(&mut self, id: NodeId, value: Rc<str>) -> bool {
        let created = self.writes.created(id);
        let Some(node) = self.tree.get_mut(id) else {
            return false;
        };
        let Some(data) = node.properties.editing.as_deref_mut() else {
            return false;
        };
        let initialize = created && data.value.is_none();
        let known = data
            .value
            .as_ref()
            .is_some_and(|old| Rc::ptr_eq(old, &value))
            && self.editors[node.index.0]
                .as_ref()
                .is_some_and(|edit| edit.editor.revision() == edit.controlled_at);
        if known {
            return false;
        }
        data.value = Some(value.clone());
        let changed = match self.editors[node.index.0].as_mut() {
            Some(edit) if initialize => {
                // Creation starts at the end; later controlled writes clamp selection.
                edit.editor = Editor::new(EditState::new(&value));
                true
            }
            // Another value, or the same one after an edit: compared once;
            // held while composing.
            Some(edit) if edit.editor.state().value() != &*value => {
                let taken = edit.editor.set_controlled(&value);
                edit.held = !taken;
                taken
            }
            Some(edit) => {
                edit.held = false;
                false
            }
            None => false,
        };
        if let Some(edit) = &mut self.editors[node.index.0] {
            edit.controlled_at = edit.editor.revision();
        }
        if changed {
            self.changed(
                id,
                if initialize {
                    Mutation::NONE
                } else {
                    Mutation::LOCAL_TEXT
                },
            );
            self.remeasure(id);
        }
        changed
    }

    /// Replace the editing state of the text input `id` with kivact's
    /// `state` (clamped to its maximum length); whether it changed.
    pub(crate) fn replace_edit_state(&mut self, id: NodeId, state: EditState) -> bool {
        let mut state = Editor::new(state);
        state.set_max_length(self.node(id).and_then(|node| node.max_length()));
        if !state.state().is_composing()
            && let Some(max) = state.max_length()
            && let Some((end, _)) = state.state().value().char_indices().nth(max)
        {
            state.truncate(end);
        }
        let Some(current) = self.editor_node(id) else {
            return false;
        };
        let revision = current.editor.revision();
        let current = current.editor.state();
        let how = if state.revision() == revision {
            Taken::Same
        } else if state.recorded_since(revision) {
            Taken::Recorded
        } else if state.state().value() == current.value() {
            Taken::Same
        } else {
            Taken::Whole
        };
        if matches!(how, Taken::Same) && state.state().session() == current.session() {
            return false;
        }
        let before = (revision, current.session().clone());
        self.edit_editor_with(id, |edit, _| {
            let reported =
                !matches!(how, Taken::Same) || edit.state().marked() != state.state().marked();
            edit.take(state.into_state(), how);
            reported
        });
        self.editor_node(id).is_some_and(|node| {
            (node.editor.revision(), node.editor.state().session()) != (before.0, &before.1)
        })
    }

    /// The value the text input `id` held back while it composed, once the
    /// composition ended: taken as a recorded change (`Editor::adopt`);
    /// whether its text changed.
    pub(super) fn adopt_held(n: &HostNode, node: &mut crate::measure::editor::EditorNode) -> bool {
        if !node.held || node.editor.state().is_composing() {
            return false;
        }
        node.held = false;
        let value = n
            .properties
            .editing
            .as_ref()
            .and_then(|data| data.value.clone())
            .unwrap_or_default();
        let changed = node.editor.adopt(&value);
        node.controlled_at = node.editor.revision();
        changed
    }

    /// Replace the session of the text input `id` with kivact's (clamped
    /// into its text), without its text; whether it changed. A new or
    /// ended composition is an edit (`TextChanged`).
    pub(crate) fn replace_edit_session(&mut self, id: NodeId, session: TextSession) -> bool {
        let Some(before) = self.edit_session(id) else {
            return false;
        };
        if before == session {
            return false;
        }
        self.edit_input(id, |edit| {
            edit.set_session(session);
            edit.marked() != before.marked()
        });
        self.edit_session(id).is_some_and(|now| now != before)
    }

    /// Queue `SelectionChanged` for every text input that reports its
    /// session (`ReportSession`) whose session is not the one it last
    /// reported: caret moves, selections and edits alike.
    pub(crate) fn report_sessions(&mut self) {
        let mut events = Vec::new();
        for id in &self.edits.inputs {
            let Some(n) = self.tree.get_mut(*id) else {
                continue;
            };
            if !n.reports_session() {
                continue;
            }
            let Some(session) = self.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .map(|e| e.session().clone())
            else {
                continue;
            };
            if self.editors[n.index.0]
                .as_ref()
                .and_then(|node| node.reported_session.as_ref())
                != Some(&session)
            {
                if let Some(node) = &mut self.editors[n.index.0] {
                    node.reported_session = Some(session.clone());
                }
                events.push(HostEvent::SelectionChanged { node: *id, session });
            }
        }
        for event in events {
            self.push(event);
        }
    }

    /// The session of the text input `id`, read without its text.
    pub(crate) fn edit_session(&self, id: NodeId) -> Option<TextSession> {
        self.edit(id).map(|edit| edit.session().clone())
    }

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

impl Model {
    pub(crate) fn editor_node(&self, id: NodeId) -> Option<&EditorNode> {
        let slot = self.tree.nodes.index_of(id)?;
        self.editors[slot.0].as_deref()
    }
    pub(crate) fn editor_node_mut(&mut self, id: NodeId) -> Option<&mut EditorNode> {
        let slot = self.tree.nodes.index_of(id)?;
        self.editors[slot.0].as_deref_mut()
    }
    pub(crate) fn edit(&self, id: NodeId) -> Option<&EditState> {
        Some(self.editor_node(id)?.editor.state())
    }
    /// The text of the input `id` (`""` when it is not one); read by the
    /// testing host only.
    #[cfg(feature = "testing")]
    pub(crate) fn input_value(&self, id: NodeId) -> &str {
        self.edit(id).map_or("", EditState::value)
    }
}

impl Model {
    pub(crate) fn edit_lines(
        &self,
        id: NodeId,
        engine: &mut arvane::Engine,
    ) -> Option<(RefMut<'_, EditLines>, &str)> {
        crate::measure::input::edit_lines(&self.tree, &self.measure, &self.editors, id, engine)
    }
    /// Refresh the lines of the input `id` in `mode`; the testing host's
    /// self-check verifies them with it.
    #[cfg(all(debug_assertions, feature = "testing"))]
    pub(crate) fn refresh_lines(
        &self,
        id: NodeId,
        engine: &mut arvane::Engine,
        mode: RefreshMode,
    ) -> Result<Option<RefreshedLines<'_>>, RefreshError> {
        crate::measure::input::refresh_lines(
            &self.tree,
            &self.measure,
            &self.editors,
            id,
            engine,
            mode,
        )
    }
}
