//! Role: Routing of key presses and text input to focus, editing and the client.
//!
//! Keys and text input: routing a key press (Tab, editing keys of the
//! focused text input, typed text, everything else reported as `Key`) and
//! the text input entry points a platform IME or a test driver calls.
//! Key and paste policy comes from the attached pure EditBehavior.
//!
//! The engine holds no editing policy of its own. A focused editor's
//! behavior is given each routed key (Tab and Shift+Tab too) and each
//! paste, synchronously with the model borrowed, after the editor's lines
//! are brought up to its text, so the behavior's `EditContext` lays the
//! request's text out. Without a behavior an editing key has no default
//! (it is routed as any other key) and a paste does nothing. Typed text,
//! the IME, pointer selection, shaping, the caret and its reveal, the
//! clipboard's IO and assistive technology's `SetValue` stay on the native
//! path, behavior or not.
//!
//! Invariants:
//! - While the focused text input composes, every key is the IME's.
//! - Tab and Shift+Tab move focus unless the focused editor's behavior
//!   consumes them. A native editor can explicitly forward selected control
//!   keys to the client; they skip focus and editing. Ctrl/Alt/Meta and
//!   Shift (except Shift+Tab) never use this opt-in path.
//! - An explicit spin control's unmodified Home/End reach its handler. A focusable
//!   spin ancestor's nested text editor keeps its native boundary keys.
//! - A key the focused text input consumes is never reported; a key that
//!   types text (the platform says which) without Ctrl, Alt or Meta, other
//!   than Enter, is text for it (the platform text input inserts it), not
//!   a `Key`.
//! - Every change of a text input's value, selection or composition made
//!   through these entry points that changes the value queues one
//!   `TextChanged` with the range changes it made (no whole value);
//!   caret-only moves queue none. The editing state is the text's one
//!   store: the `InputValue` prop keeps the last value kivact wrote. The
//!   whole value rides along only for an editor that asks for it
//!   (`reports_text`), as it copies the text every time.
//! - A disabled text input takes no edit. A read-only one takes no change
//!   of its text and no composition (they are taken back), only of its
//!   selection.
//! - A change of the text an entry point does not report is taken back
//!   (a behavior that returns `changed: false`): the edit's `Journal` takes
//!   it back, and the text the core copied by its reported changes stays the
//!   editor's.
//! - Every entry point ([`Model::edit_input_with`]), a behavior's
//!   included, holds the editor to its maximum length in scalars of
//!   committed text: after it the value is cut to the larger of the limit
//!   and the length it had before, so a value the application set over a
//!   lowered limit is not cut (a caret move cuts nothing, a deletion goes
//!   on step by step) but does not grow, and new text takes the limit as
//!   it is. A composition is not cut. A whole state kivact gives back is
//!   cut to the limit itself first (`Model::replace_edit_state`). Without a
//!   limit nothing is counted.
//! - The edit that ends a composition takes the controlled value held
//!   while composing, reported in its `TextChanged`.
//! - With Ctrl or Cmd held, the key is what the keyboard layout makes of
//!   the keystroke when that is ASCII (AZERTY and Dvorak keep their
//!   letters), else the US-layout letter of the physical key (a Russian
//!   or Greek layout's Ctrl+С is Ctrl+C), like the platforms' shortcut
//!   handling. Editing chords (Ctrl or Cmd with A, C, X, V) and the
//!   reported `Key` both use it.

use std::ops::Range;

use crate::contract::HostEvent;
use kivact_types::{A11yAction, Key, KeyCode, KeyPhase, Modifiers, NodeId, Role};

use crate::measure::editor::EditorNode;
use crate::model::Model;
use crate::model::changes::Dirty;
use crate::text_input::{EditLines, Shaping};
use kivact_editing::Journal;
use kivact_editing::{EditContext, EditLayout, EditOutcome, EditRequest, EditState};

/// What became of a key press ([`Model::handle_key`]).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum KeyResult {
    /// Reported to the core as `Key`.
    Reported,
    /// Used by the host: a focus move, an edit, or the IME's while
    /// composing.
    Consumed,
    /// Text for the focused text input: the platform's text input inserts
    /// it; a host without one calls [`Model::insert_text`].
    Text(String),
    /// Copy or cut: the host writes this to its clipboard.
    Copy(String),
    /// Paste: the host reads its clipboard and calls [`Model::paste_input`].
    Paste(NodeId),
}

/// The key a press with `modifiers` stands for: with Ctrl or Cmd held,
/// a key the layout does not make ASCII is the physical key's US-layout
/// letter (upper case with Shift).
fn shortcut_key(key: Key, physical: Option<char>, modifiers: Modifiers) -> Key {
    let ascii = match key {
        Key::Char(c) => c.is_ascii(),
        Key::Unidentified => false,
        _ => true,
    };
    match physical {
        Some(letter) if (modifiers.ctrl || modifiers.meta) && !ascii => {
            Key::Char(if modifiers.shift {
                letter.to_ascii_uppercase()
            } else {
                letter.to_ascii_lowercase()
            })
        }
        _ => key,
    }
}

impl Model {
    /// A key event with the shell's input and the US-layout letter of the
    /// physical key (`physical`, for shortcuts on non-Latin layouts). Typed
    /// text goes into the focused text input; copy and paste are left to
    /// the shell.
    pub(crate) fn key(
        &mut self,
        input: kivact_types::KeyInput,
        physical: Option<char>,
        engine: &mut arvane::Engine,
    ) -> KeyResult {
        let kivact_types::KeyInput {
            key,
            code,
            phase,
            modifiers,
            pointer,
        } = input;
        self.modifiers = modifiers;
        let KeyPhase::Down { repeat, text } = phase else {
            self.key_up(key, code, pointer);
            return KeyResult::Reported;
        };
        let text = text.filter(|t| !t.is_empty() && !t.chars().any(char::is_control));
        let result = self.handle_key(
            key,
            physical,
            code,
            modifiers,
            repeat,
            text.as_deref(),
            pointer,
            engine,
        );
        if let KeyResult::Text(text) = &result {
            self.insert_text(text);
        }
        self.sync();
        result
    }

    /// Clipboard text pasted into the text input `id` (as is).
    pub(crate) fn paste(&mut self, id: NodeId, text: &str, engine: &mut arvane::Engine) {
        self.paste_input(id, text, engine);
        self.sync();
    }

    /// A key release, reported to the focused node.
    pub(crate) fn key_up(&mut self, key: Key, code: KeyCode, pointer: Option<kivact_types::Point>) {
        let node = self.focused();
        self.push(HostEvent::Key(crate::contract::Routed {
            target: node.map(|f| f.node),
            region: node.and_then(|f| f.region),
            local: pointer
                .zip(node.map(|f| f.node))
                .map(|(point, node)| self.to_local(node, point))
                .unwrap_or_default(),
            captured: false,
            activation: None,
            input: kivact_types::KeyInput {
                key,
                code,
                phase: KeyPhase::Up,
                modifiers: self.modifiers,
                pointer,
            },
        }));
    }

    /// The focused text input, if it takes edits (not disabled).
    pub(crate) fn focused_input(&self) -> Option<NodeId> {
        match self.edits.target {
            crate::model::edit::state::ImeTarget::Native(slot) => {
                Some(self.tree.nodes.id_at(slot.0))
            }
            crate::model::edit::state::ImeTarget::None
            | crate::model::edit::state::ImeTarget::Session(_) => None,
        }
    }

    /// Edit the text input `id` with `f`, which says whether the value
    /// changed (then `TextChanged` is queued). `None`: not a text input
    /// that takes edits.
    pub(crate) fn edit_input(
        &mut self,
        id: NodeId,
        f: impl FnOnce(&mut EditState) -> bool,
    ) -> Option<bool> {
        self.edit_editor_with(id, |journal, _| f(journal.state_mut()))
    }

    /// Run `f` (a behavior's request) on the text input `id`, its lines
    /// brought up to its text first and shaped as the behavior asks,
    /// aligned in a box the given width wide.
    pub(crate) fn edit_input_with(
        &mut self,
        id: NodeId,
        engine: &mut arvane::Engine,
        f: impl FnOnce(&mut EditState, &Shaping<'_>, f64) -> bool,
    ) -> Option<bool> {
        let width = self.lay_out_lines(id, engine);
        self.edit_editor_with(id, |journal, lines| {
            let lines = lines.expect("invariant: lay_out_lines made the input's lines");
            let lines = Shaping::new(lines, engine);
            f(journal.state_mut(), &lines, width)
        })
    }

    pub(super) fn edit_editor_with(
        &mut self,
        id: NodeId,
        f: impl FnOnce(&mut Journal<'_>, Option<&mut EditLines>) -> bool,
    ) -> Option<bool> {
        let n = self.tree.get_mut(id)?;
        if n.disabled() {
            return None;
        }
        let read_only = n.read_only();
        let max_length = n.max_length();
        let report = n.reports_text();
        let EditorNode {
            editor,
            lines,
            controlled,
            ..
        } = self.editors[n.index.0].as_deref_mut()?;
        let mut journal = editor.journal(max_length);
        let limit = max_length.map(|max| {
            if journal.state().is_composing() {
                max
            } else {
                max.max(journal.state().value().chars().count())
            }
        });
        let mut reported = f(&mut journal, lines.get_mut().as_mut());
        // A change of the text must be reported: an unreported one (a
        // behavior that changed the text and said it did not) is taken back.
        if !reported && journal.text_changed() {
            journal.revert();
        }
        if !journal.state().is_composing()
            && let Some(max) = limit
            && let Some((end, _)) = journal.state().value().char_indices().nth(max)
        {
            journal.truncate(end);
            reported = true;
        }
        if read_only && (journal.text_changed() || journal.state().is_composing()) {
            journal.revert();
        }
        if EditorNode::adopt_pending_controlled(controlled, &mut journal) {
            reported = true;
        }
        let changed = !read_only && reported;
        let changes = journal.finish();
        let event = changed.then(|| HostEvent::TextChanged {
            node: id,
            changes,
            session: editor.state().session().clone(),
            text: report.then(|| editor.state().value().to_string()),
        });
        if let Some(editor) = self.editor_node_mut(id) {
            editor.flush_damage();
        }
        self.repaint_text(id);
        if let Some(e) = event {
            self.changes.mark(id, Dirty::MEASURE);
            self.changes.needs_layout = true;
            self.push(e);
        }
        Some(changed)
    }

    /// A key press on the focused node. `text` is what the platform types
    /// for it (`None` for a key that types nothing, or with Ctrl held):
    /// text for a focused text input, and the `text` of a reported
    /// character key. `physical` is the US-layout letter of the physical
    /// key (`a`..=`z`), if it is a letter key: with Ctrl or Cmd held, a
    /// key that is not ASCII is that letter (see the module invariants).
    /// Editing chords (Ctrl or Cmd with A, C, X, V) match the result.
    #[expect(
        clippy::too_many_arguments,
        reason = "a key press is these values; the engine shapes the editor's lines"
    )]
    pub(crate) fn handle_key(
        &mut self,
        key: Key,
        physical: Option<char>,
        code: KeyCode,
        modifiers: Modifiers,
        repeat: bool,
        text: Option<&str>,
        pointer: Option<kivact_types::Point>,
        engine: &mut arvane::Engine,
    ) -> KeyResult {
        let key = shortcut_key(key, physical, modifiers);
        let input = self.focused_input();
        let editor = input.and_then(|i| self.tree.get(i)).and_then(|n| {
            self.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
        });
        if editor.is_some_and(EditState::is_composing) {
            return KeyResult::Consumed;
        }
        let roving_boundary = matches!(
            key,
            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown
        ) && input.zip(editor).is_some_and(|(input, editor)| {
            let selection = editor.selection();
            if !selection.is_empty() {
                return false;
            }
            let Some(group) = self.roving_focus_for(input) else {
                return false;
            };
            match (group.orientation, &key) {
                (kivact_types::Orientation::Horizontal, Key::ArrowLeft | Key::ArrowRight) => {
                    let end = (key == Key::ArrowRight) != group.rtl;
                    selection.start == if end { editor.value().len() } else { 0 }
                }
                (kivact_types::Orientation::Vertical, Key::ArrowUp | Key::ArrowDown) => {
                    self.tree.get(input).is_some_and(|node| !node.multiline())
                }
                _ => false,
            }
        });
        let forward = !modifiers.ctrl
            && !modifiers.alt
            && !modifiers.meta
            && (!modifiers.shift || key == Key::Tab)
            && (roving_boundary
                || (matches!(
                    key,
                    Key::Tab
                        | Key::ArrowLeft
                        | Key::ArrowRight
                        | Key::ArrowUp
                        | Key::ArrowDown
                        | Key::Home
                        | Key::End
                        | Key::PageUp
                        | Key::PageDown
                        | Key::Escape
                        | Key::Enter
                        | Key::Backspace
                ) && input
                    .and_then(|id| self.tree.get(id))
                    .and_then(|node| node.input_data())
                    .and_then(|data| data.forward_keys.as_ref())
                    .is_some_and(|keys| keys.contains(&key))));
        let focus_key = !forward && key == Key::Tab && !modifiers.ctrl && !modifiers.alt;
        if let Some(i) = input.filter(|_| !forward) {
            let node = self.tree.get(i).expect("invariant: focused input exists");
            let flags = (node.multiline(), node.read_only(), node.password());
            let behavior = node.input_data().and_then(|data| data.behavior.clone());
            // A spin button owns its unmodified boundary keys, including
            // when its native editor is a child of the semantic control.
            let spin_boundary = modifiers == Modifiers::default()
                && matches!(key, Key::Home | Key::End)
                && self
                    .ancestor_where(i, |ancestor| {
                        ancestor.role() == Some(Role::SpinButton)
                            && ancestor.actions().contains(&A11yAction::SetValue)
                            && (ancestor.index == node.index
                                || !ancestor.focusable(self.editors[ancestor.index.0].is_some()))
                    })
                    .is_some();
            let read_only = flags.1;
            let mut outcome = EditOutcome::NotEditing;
            if let Some(behavior) = behavior.filter(|_| !spin_boundary) {
                self.edit_input_with(i, engine, |state, lines, width| {
                    let context = context(flags, lines, width);
                    let request = EditRequest::Key { key, modifiers };
                    outcome = behavior.invoke(state, request, &context);
                    matches!(
                        outcome,
                        EditOutcome::Done { changed: true }
                            | EditOutcome::Copy { changed: true, .. }
                    )
                });
            }
            match outcome {
                EditOutcome::NotEditing => {}
                EditOutcome::Done { .. } => return KeyResult::Consumed,
                EditOutcome::Copy { text, .. } => return KeyResult::Copy(text),
                EditOutcome::Paste => return KeyResult::Paste(i),
            }
            let command = modifiers.ctrl || modifiers.alt || modifiers.meta;
            if let Some(t) = text
                && !command
                && !read_only
                && !focus_key
                && key != Key::Enter
            {
                return KeyResult::Text(t.to_string());
            }
        }
        if focus_key {
            self.move_focus(modifiers.shift);
            return KeyResult::Consumed;
        }
        let text = match key {
            Key::Char(_) | Key::Space => text.map(str::to_string),
            _ => None,
        };
        self.push(HostEvent::Key(crate::contract::Routed {
            target: self.focused().map(|f| f.node),
            region: self.focused().and_then(|f| f.region),
            local: pointer
                .zip(self.focused().map(|f| f.node))
                .map(|(point, node)| self.to_local(node, point))
                .unwrap_or_default(),
            captured: false,
            activation: None,
            input: kivact_types::KeyInput {
                key,
                code,
                phase: KeyPhase::Down { repeat, text },
                modifiers,
                pointer,
            },
        }));
        KeyResult::Reported
    }

    /// Text committed into the focused text input (typing, or an IME
    /// commit): replaces the composition, else the selection.
    pub(crate) fn insert_text(&mut self, text: &str) {
        if let Some(i) = self.focused_input() {
            self.edit_input(i, |e| {
                e.commit(None, text);
                true
            });
        }
    }

    /// An IME composition in the focused text input: `text` replaces
    /// `range` (else the composition, else the selection) and is marked;
    /// `selected` is the selection inside `text` in bytes.
    pub(crate) fn compose_text(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
    ) {
        if let Some(i) = self.focused_input() {
            self.edit_input(i, |e| {
                e.compose(range, text, selected);
                true
            });
        }
    }

    /// End the composition of the text input `id`, keeping its text (a
    /// commit of what it shows), and report it. Unlike an edit, it also
    /// ends the composition of a disabled input: the IME's text stays.
    pub(crate) fn commit_composition(&mut self, id: NodeId) {
        let Some(n) = self.tree.get_mut(id) else {
            return;
        };
        let report = n.reports_text();
        let Some(EditorNode {
            editor, controlled, ..
        }) = self.editors[n.index.0].as_deref_mut()
        else {
            return;
        };
        let mut journal = editor.journal(n.max_length());
        if !journal.state_mut().unmark() {
            return;
        }
        EditorNode::adopt_pending_controlled(controlled, &mut journal);
        let changes = journal.finish();
        let event = HostEvent::TextChanged {
            node: id,
            changes,
            session: editor.state().session().clone(),
            text: report.then(|| editor.state().value().to_string()),
        };
        if let Some(editor) = self.editor_node_mut(id) {
            editor.flush_damage();
        }
        self.repaint_text(id);
        self.push(event);
    }

    /// Paste `text` from the clipboard into the text input `id`.
    pub(crate) fn paste_input(&mut self, id: NodeId, text: &str, engine: &mut arvane::Engine) {
        let Some(node) = self.tree.get(id) else {
            return;
        };
        let flags = (node.multiline(), node.read_only(), node.password());
        let Some(behavior) = node.input_data().and_then(|data| data.behavior.clone()) else {
            return;
        };
        self.edit_input_with(id, engine, |state, lines, width| {
            let context = context(flags, lines, width);
            matches!(
                behavior.invoke(state, EditRequest::Paste(text), &context),
                EditOutcome::Done { changed: true } | EditOutcome::Copy { changed: true, .. }
            )
        });
    }

    /// Put the caret of the text input `id` at byte offset `at` (`extend`:
    /// select from the anchor). Changes no value.
    pub(crate) fn place_caret(&mut self, id: NodeId, at: usize, extend: bool) {
        self.edit_input(id, |e| {
            e.move_to(at, extend);
            false
        });
    }

    /// Replace the whole value of the text input `id` (assistive
    /// technology's `SetValue`).
    pub(crate) fn set_input_value(&mut self, id: NodeId, value: &str) {
        self.edit_input(id, |e| {
            let len = e.value().len();
            e.replace(0..len, value);
            true
        });
    }
}

impl Model {
    /// Bring the lines of the text input `id` up to its text for a
    /// behavior to read; the width of the box they are aligned in.
    fn lay_out_lines(&self, id: NodeId, engine: &mut arvane::Engine) -> f64 {
        drop(self.edit_lines(id, engine));
        crate::model::interact::visual::text_box(self, id).width
    }
}

/// What a behavior is given: the input's `(multiline, read_only,
/// password)` and its `lines`, aligned in a box `width` wide.
fn context<'a>(flags: (bool, bool, bool), lines: &'a Shaping<'_>, width: f64) -> EditContext<'a> {
    let (multiline, read_only, password) = flags;
    let layout = EditLayout {
        lines,
        revision: lines.revision(),
        width,
    };
    EditContext::new(multiline, read_only, password, Some(layout))
}
