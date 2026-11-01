//! Role: A small code editor built from the public native editing contract.
//!
//! The editor gives its text to the native editor once and keeps its own
//! copy by the range changes each edit reports, so a keystroke costs the
//! size of the change, not of the document. Its key policy is a closure
//! over the editor's own state: Tab and Shift+Tab indent and dedent the
//! line, Enter keeps the line's indentation, Home and End stay on the line,
//! the vertical arrows move to the nearest caret a line up or down from
//! where the moves started (the editor's laid-out lines, read through the
//! behavior's context), and Ctrl+Z undoes one edit (an IME composition
//! counts as one); every other key goes to the standard policy. Keywords
//! and block comments are colored line by line: a line hands whether a
//! comment is still open to the next. The caret's line and column follow
//! the session. Because this editor keeps Tab, a keyboard user leaves it
//! with Escape and then Tab (state the policy keeps between keys).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use kivact::control::{Node, NodeBuilder, node};
use kivact::interaction::text_edit_policy;
use kivact::primitive::{
    EditContext, EditOutcome, EditRequest, EditState, NodeRefExt, TextEditEvent, TextEditing,
    TextHighlight, TextSession, part,
};
use kivact::style::{ColorToken, rgb, style};
use kivact::view::{IntoText, IntoValue, Key, NodeRef, Role};
use kivact::{Signal, signal};

/// How many spaces Tab inserts.
const INDENT: &str = "    ";

/// The color of a keyword.
pub const KEYWORD: u32 = 0xD01010;

/// The color of a block comment.
pub const COMMENT: u32 = 0x10A040;

/// A code editor's document as its owner sees it.
#[derive(Clone)]
pub struct CodeEditor {
    doc: Rc<Document>,
    lines: Signal<usize>,
    caret: Signal<(usize, usize)>,
    node: NodeRef,
}

impl CodeEditor {
    /// The editor's text, preedit included, as its edits left it.
    pub fn text(&self) -> String {
        self.doc.text.borrow().clone()
    }

    /// The number of lines, counted from the edits' changes.
    pub fn lines(&self) -> Signal<usize> {
        self.lines
    }

    /// The caret's line and column (in characters), both from 1.
    pub fn caret(&self) -> Signal<(usize, usize)> {
        self.caret
    }

    /// The editor's node.
    pub fn node_ref(&self) -> NodeRef {
        self.node
    }
}

/// One change as it can be undone: at byte `at`, `inserted` replaced
/// `removed`.
struct Undo {
    at: usize,
    removed: String,
    inserted: String,
}

/// The editor's own state, shared by its key policy and its edit handler.
#[derive(Default)]
struct Document {
    text: RefCell<String>,
    /// Undo steps, each the changes of one edit or one composition.
    undo: RefCell<Vec<Vec<Undo>>>,
    /// A composition is still adding to the last step.
    composing: Cell<bool>,
    /// The next edit is an undo, which records nothing.
    undoing: Cell<bool>,
    /// The last key was Escape: the next Tab moves focus.
    escaped: Cell<bool>,
    /// Where vertical moves aim across, kept while they follow each other.
    goal: Cell<Option<f32>>,
    /// The editor's node, to sync the copy again from.
    node: Cell<Option<NodeRef>>,
    /// A line start and its line number (from 1), where the caret's line
    /// was last counted from: the next count goes from there.
    mark: Cell<Option<(usize, usize)>>,
}

impl Document {
    /// Keep the copy and the undo steps in step with one edit; returns how
    /// many line breaks it added (negative: removed). A change that does not
    /// fit the copy (it is not the editor's text) syncs the copy again from
    /// the editor and forgets the undo steps: `None`.
    fn edited(&self, event: &TextEditEvent) -> Option<isize> {
        let mut text = self.text.borrow_mut();
        let mut undo = self.undo.borrow_mut();
        let record = !self.undoing.replace(false);
        if record && !self.composing.get() && !event.changes.is_empty() {
            undo.push(Vec::new());
        }
        let mut breaks = 0;
        for change in &event.changes {
            let Some(removed) = text.get(change.range.clone()).map(str::to_string) else {
                drop((text, undo));
                self.resync(event);
                return None;
            };
            let added =
                change.text.matches('\n').count() as isize - removed.matches('\n').count() as isize;
            breaks += added;
            // A change before the mark moves it; one over it loses it.
            self.mark.set(match self.mark.get() {
                Some((at, line)) if change.range.end < at => Some((
                    at + change.text.len() - removed.len(),
                    line.saturating_add_signed(added),
                )),
                Some((at, line)) if change.range.start >= at => Some((at, line)),
                _ => None,
            });
            if record && let Some(step) = undo.last_mut() {
                step.push(Undo {
                    at: change.range.start,
                    removed,
                    inserted: change.text.clone(),
                });
            }
            change.apply(&mut text);
        }
        self.composing.set(event.composition().is_some());
        Some(breaks)
    }

    /// Sync the copy again from the editor, forgetting the undo steps.
    fn resync(&self, event: &TextEditEvent) {
        let state = self.node.get().and_then(|node| node.edit_state());
        *self.text.borrow_mut() = state.map_or_else(String::new, EditState::into_value);
        self.undo.borrow_mut().clear();
        self.mark.set(None);
        self.composing.set(event.composition().is_some());
    }

    /// The caret's line and column, both from 1, in the copy: the line
    /// counted from the mark, so a caret move costs the distance it moved.
    fn position(&self, session: &TextSession) -> (usize, usize) {
        let text = self.text.borrow();
        let caret = session.caret().min(text.len());
        let start = line_start(&text, caret);
        let line = match self.mark.get() {
            Some((at, line)) if at <= start => line + text[at..start].matches('\n').count(),
            Some((at, line)) => line - text[start..at].matches('\n').count(),
            None => text[..start].matches('\n').count() + 1,
        };
        self.mark.set(Some((start, line)));
        (line, text[start..caret].chars().count() + 1)
    }

    /// The key policy: the editor's own keys, then the standard ones.
    fn keys(
        &self,
        state: &mut EditState,
        request: EditRequest<'_>,
        context: &EditContext<'_>,
    ) -> EditOutcome {
        let EditRequest::Key { key, modifiers } = request else {
            return text_edit_policy(state, request, context);
        };
        let command = modifiers.ctrl || modifiers.meta;
        let escaped = self.escaped.replace(key == Key::Escape);
        let vertical = matches!(key, Key::ArrowUp | Key::ArrowDown) && !command;
        if !vertical {
            self.goal.set(None);
        }
        if context.read_only || (escaped && key == Key::Tab) {
            return text_edit_policy(state, request, context);
        }
        match key {
            Key::Char('z') if command => self.undo(state),
            Key::Tab if !command && !modifiers.alt => {
                if modifiers.shift {
                    dedent(state)
                } else {
                    state.replace(state.selection(), INDENT);
                    EditOutcome::Done { changed: true }
                }
            }
            Key::Enter if !command => {
                let text = state.value();
                let start = line_start(text, state.selection().start);
                let indent: String = text[start..].chars().take_while(|c| *c == ' ').collect();
                state.replace(state.selection(), &format!("\n{indent}"));
                EditOutcome::Done { changed: true }
            }
            Key::Home | Key::End if !command => {
                let text = state.value();
                let caret = state.caret();
                let to = if key == Key::Home {
                    line_start(text, caret)
                } else {
                    line_end(text, caret)
                };
                state.move_to(to, modifiers.shift);
                EditOutcome::Done { changed: false }
            }
            _ if vertical => match self.vertical(state, context, key == Key::ArrowUp) {
                Some(to) => {
                    state.move_to(to, modifiers.shift);
                    EditOutcome::Done { changed: false }
                }
                None => text_edit_policy(state, request, context),
            },
            _ => text_edit_policy(state, request, context),
        }
    }

    /// The caret one line up or down, at the caret nearest where the
    /// vertical moves started across; the start or end of the text past
    /// the first or last line (`None`: the editor is not laid out).
    fn vertical(&self, state: &EditState, context: &EditContext<'_>, up: bool) -> Option<usize> {
        let caret = state.caret();
        let line = context.line_of(state, caret)?;
        let x = match self.goal.get() {
            Some(x) => x,
            None => context.x_of(state, caret)?,
        };
        self.goal.set(Some(x));
        let last = context.line_count(state)? - 1;
        match (up, line) {
            (true, 0) => Some(0),
            (false, line) if line == last => Some(state.value().len()),
            (true, line) => context.offset_at(state, line - 1, x),
            (false, line) => context.offset_at(state, line + 1, x),
        }
    }

    /// Undo the last step in `state`.
    fn undo(&self, state: &mut EditState) -> EditOutcome {
        let Some(step) = self.undo.borrow_mut().pop() else {
            return EditOutcome::Done { changed: false };
        };
        for change in step.iter().rev() {
            state.replace(
                change.at..change.at + change.inserted.len(),
                &change.removed,
            );
        }
        self.undoing.set(true);
        EditOutcome::Done { changed: true }
    }
}

/// Remove up to one indentation from the start of the caret's line.
fn dedent(state: &mut EditState) -> EditOutcome {
    let start = line_start(state.value(), state.caret());
    let spaces = state.value()[start..]
        .chars()
        .take(INDENT.len())
        .take_while(|c| *c == ' ')
        .count();
    if spaces == 0 {
        return EditOutcome::Done { changed: false };
    }
    let caret = state.caret();
    state.replace(start..start + spaces, "");
    state.move_to(caret.saturating_sub(spaces).max(start), false);
    EditOutcome::Done { changed: true }
}

fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |i| i + 1)
}

fn line_end(text: &str, at: usize) -> usize {
    text[at..].find('\n').map_or(text.len(), |i| at + i)
}

/// The keyword ranges of `text`, which starts at byte `at` of its line.
fn keywords(keywords: &[String], text: &str, at: usize, out: &mut Vec<TextHighlight>) {
    let mut start = None;
    for (i, c) in text.char_indices().chain([(text.len(), ' ')]) {
        let word = c.is_alphanumeric() || c == '_';
        match (start, word) {
            (None, true) => start = Some(i),
            (Some(s), false) => {
                if keywords.iter().any(|k| *k == text[s..i]) {
                    out.push(TextHighlight {
                        range: at + s..at + i,
                        color: rgb(KEYWORD),
                    });
                }
                start = None;
            }
            _ => {}
        }
    }
}

/// The colors of one line: its keywords outside block comments, and its
/// block comments, from whether a comment is `open` where it starts; and
/// whether one is open where it ends.
fn colors(list: &[String], line: &str, open: &bool) -> (Vec<TextHighlight>, bool) {
    let mut out = Vec::new();
    let (mut open, mut at, mut from) = (*open, 0, 0);
    loop {
        if open {
            let end = line[at..].find("*/").map(|i| at + i + 2);
            let comment = from..end.unwrap_or(line.len());
            if !comment.is_empty() {
                out.push(TextHighlight {
                    range: comment,
                    color: rgb(COMMENT),
                });
            }
            let Some(end) = end else {
                return (out, true);
            };
            (open, at) = (false, end);
        } else {
            let opens = line[at..].find("/*").map(|i| at + i);
            keywords(list, &line[at..opens.unwrap_or(line.len())], at, &mut out);
            let Some(opens) = opens else {
                return (out, false);
            };
            (open, from, at) = (true, opens, opens + 2);
        }
    }
}

/// A multiline code editor holding `text`, coloring `keywords_list` and
/// block comments.
///
/// Call it during setup. The text is given to the native editor once; read
/// it back with [`CodeEditor::text`].
#[track_caller]
pub fn code_editor(
    label: impl IntoText,
    text: &str,
    keywords_list: impl IntoValue<Vec<String>>,
) -> (Node, CodeEditor) {
    let doc = Rc::new(Document {
        text: RefCell::new(text.to_string()),
        ..Document::default()
    });
    let lines = signal(text.matches('\n').count() + 1);
    let caret = signal((1, 1));
    let policy = doc.clone();
    let edits = doc.clone();
    let moves = doc.clone();
    let editing = TextEditing::new(text)
        .multiline(true)
        .behavior(move |state, request, context| policy.keys(state, request, context))
        .highlight(
            keywords_list,
            |list: &Vec<String>, line: &str, open: &bool| colors(list, line, open),
        )
        .on_edit(move |event| match edits.edited(&event) {
            Some(0) => {}
            Some(breaks) => lines.update(|n| *n = n.saturating_add_signed(breaks)),
            None => lines.set(edits.text.borrow().matches('\n').count() + 1),
        })
        .on_selection(move |session| caret.set(moves.position(&session)));
    let control = part(|state| state)
        .role(Role::TextBox)
        .focusable(true)
        .text_editing(editing);
    let reference = control.node_ref();
    doc.node.set(Some(reference));
    let view = node().name(label).part(control).style(
        style()
            .width(420)
            .height(220)
            .padding(8)
            .font_size(22)
            .border(1, ColorToken::Border)
            .background(ColorToken::Surface)
            .color(ColorToken::Text),
    );
    (
        view,
        CodeEditor {
            doc,
            lines,
            caret,
            node: reference,
        },
    )
}
