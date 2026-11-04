//! Role: Pure native editing keys and clipboard normalization.

use kivact_core::custom::{EditContext, EditOutcome, EditRequest, EditState};
use kivact_core::view::Key;

/// The key and paste policy of [`use_text_edit`](crate::use_text_edit), for a
/// [`TextEditing::behavior`](kivact_core::custom::TextEditing::behavior)
/// that handles some keys itself and leaves the rest to it.
///
/// It moves and deletes by grapheme, selects with Shift, goes to the start
/// and end of the text with Home and End, breaks lines with Enter in a
/// multiline editor, and handles Ctrl/Cmd with A, C, X and V. A pasted
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
    let sel = state.selection();
    let caret = state.caret();
    let changed = match key {
        Key::Char(c) if command => match c.to_ascii_lowercase() {
            'a' => {
                state.move_to(0, false);
                state.move_to(state.value().len(), true);
                false
            }
            'c' | 'x' if sel.is_empty() => false,
            'c' => {
                return EditOutcome::Copy {
                    text: state.selected_text().to_string(),
                    changed: false,
                };
            }
            'x' => {
                let text = state.selected_text().to_string();
                state.replace(sel, "");
                return EditOutcome::Copy {
                    text,
                    changed: true,
                };
            }
            'v' => return EditOutcome::Paste,
            _ => return EditOutcome::NotEditing,
        },
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
