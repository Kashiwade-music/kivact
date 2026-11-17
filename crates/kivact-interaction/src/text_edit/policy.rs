//! Role: Pure native editing keys and clipboard normalization.

use kivact_core::custom::{EditContext, EditOutcome, EditRequest, EditState};
use kivact_core::view::Key;

/// The key and paste policy of [`use_text_edit`](crate::use_text_edit), for a
/// [`TextEditing::behavior`](kivact_core::custom::TextEditing::behavior)
/// that handles some keys itself and leaves the rest to it.
///
/// It moves and deletes by grapheme, selects with Shift, goes to the start
/// and end of the text with Home and End, moves a line up or down with the
/// arrow keys and breaks lines with Enter in a multiline editor, and handles Ctrl/Cmd with A, C, X and V. A pasted
/// text has its line breaks normalized (`\r\n` to `\n`, or to spaces in a
/// single-line editor). It keeps read-only values and hidden passwords,
/// and leaves every other key, Tab included, to routing.
pub fn text_edit_policy(
    state: &mut EditState,
    request: EditRequest<'_>,
    context: &EditContext<'_>,
) -> EditOutcome {
    let (mut key, modifiers) = match request {
        EditRequest::Key { key, modifiers } => (key, modifiers),
        EditRequest::Paste(text) => {
            if context.read_only {
                return EditOutcome::NotEditing;
            }
            let text = if context.multiline {
                text.replace("\r\n", "\n")
            } else {
                text.replace(['\r', '\n'], " ")
            };
            state.replace(state.selection(), &text);
            return EditOutcome::Done { changed: true };
        }
    };
    if let Key::Char(c) = key
        && (modifiers.ctrl || modifiers.meta)
        && ((context.password && matches!(c.to_ascii_lowercase(), 'c' | 'x'))
            || (context.read_only && matches!(c.to_ascii_lowercase(), 'x' | 'v')))
    {
        key = Key::Unidentified;
    }
    if context.read_only && matches!(key, Key::Backspace | Key::Delete | Key::Enter) {
        key = Key::Unidentified;
    }
    let multiline = context.multiline;
    let command = modifiers.ctrl || modifiers.meta;
    let shift = modifiers.shift;
    if multiline && !command && matches!(key, Key::ArrowUp | Key::ArrowDown) {
        return vertical_key(state, key == Key::ArrowUp, shift, context);
    }
    let sel = state.selection();
    let caret = state.caret();
    let changed = match key {
        Key::Char(c) if command => return command_key(state, c),
        Key::Backspace | Key::Delete => {
            let r = if !sel.is_empty() {
                sel.clone()
            } else if key == Key::Backspace {
                state.prev_grapheme(caret)..caret
            } else {
                caret..state.next_grapheme(caret)
            };
            if r.is_empty() {
                false
            } else {
                state.replace(r, "");
                true
            }
        }
        Key::ArrowLeft | Key::ArrowRight => {
            let to = match (key == Key::ArrowLeft, sel.is_empty() || shift) {
                (true, true) => state.prev_grapheme(caret),
                (true, false) => sel.start,
                (false, true) => state.next_grapheme(caret),
                (false, false) => sel.end,
            };
            state.move_to(to, shift);
            false
        }
        Key::Home => {
            state.move_to(0, shift);
            false
        }
        Key::End => {
            state.move_to(state.value().len(), shift);
            false
        }
        Key::Enter if multiline => {
            let before_len = state.value().len();
            state.replace(sel.clone(), "\n");
            !sel.is_empty() || state.value().len() != before_len
        }
        _ => return EditOutcome::NotEditing,
    };
    if !changed && state.selection() == sel && state.caret() == caret {
        EditOutcome::NotEditing
    } else {
        EditOutcome::Done { changed }
    }
}

/// Up (`up`) or Down in a multiline editor: move the caret a line, extending
/// the selection with Shift. Up at the start or Down at the end goes on to
/// routing (a composer recalls its history there), even over a selection.
fn vertical_key(
    state: &mut EditState,
    up: bool,
    shift: bool,
    context: &EditContext<'_>,
) -> EditOutcome {
    match vertical_target(state, up, context) {
        Some(to) if to != state.caret() => {
            state.move_to(to, shift);
            EditOutcome::Done { changed: false }
        }
        _ => EditOutcome::NotEditing,
    }
}

/// Where Up (`up`) or Down moves the caret of a multiline editor: the offset
/// under the caret's x on the laid-out line above or below; the start of the
/// text from the first line and its end from the last. `None` without layout.
fn vertical_target(state: &EditState, up: bool, context: &EditContext<'_>) -> Option<usize> {
    let caret = state.caret();
    let line = context.line_of(state, caret)?;
    if up && line == 0 {
        return Some(0);
    }
    if !up && line + 1 >= context.line_count(state)? {
        return Some(state.value().len());
    }
    let next = if up { line - 1 } else { line + 1 };
    context.offset_at(state, next, context.x_of(state, caret)?)
}

fn command_key(state: &mut EditState, key: char) -> EditOutcome {
    match key.to_ascii_lowercase() {
        'a' => {
            let before = state.selection();
            let caret = state.caret();
            state.move_to(0, false);
            state.move_to(state.value().len(), true);
            if before == state.selection() && caret == state.caret() {
                EditOutcome::NotEditing
            } else {
                EditOutcome::Done { changed: false }
            }
        }
        'c' | 'x' if state.selection().is_empty() => EditOutcome::NotEditing,
        'c' => EditOutcome::Copy {
            text: state.selected_text().to_string(),
            changed: false,
        },
        'x' => {
            let text = state.selected_text().to_string();
            state.replace(state.selection(), "");
            EditOutcome::Copy {
                text,
                changed: true,
            }
        }
        'v' => EditOutcome::Paste,
        _ => EditOutcome::NotEditing,
    }
}
