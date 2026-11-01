//! Role: Text inputs' native editors: edit states, controlled values, sessions and lines.

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
use kivact_wire::{HostEvent, NodeId};
use std::cell::RefMut;
use std::rc::Rc;

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
    pub(in crate::model) fn adopt_held(
        n: &HostNode,
        node: &mut crate::measure::editor::EditorNode,
    ) -> bool {
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
