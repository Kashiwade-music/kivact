//! Role: Apply source-line emphasis and native read-only code navigation.
use kivact_core::custom::{EditContext, EditOutcome, EditRequest, EditState, TextEditing};
use kivact_core::state::Value;
use kivact_core::style::{Color, ColorToken, StyledRange};
use kivact_core::view::Key;
use kivact_interaction::text_edit_policy;
/// Fill syntax gaps and unset backgrounds without overlapping glyph-color ranges.
pub(super) fn line_background(
    ranges: &mut Vec<StyledRange>,
    line: &str,
    index: usize,
    highlighted: &[u32],
    diff: bool,
) {
    let color = if highlighted.contains(&((index + 1) as u32)) {
        Some(Color::token(ColorToken::SurfaceAlt))
    } else if diff && line.starts_with('+') {
        Some(Color::token(ColorToken::Success).with_alpha(0.15))
    } else if diff && line.starts_with('-') {
        Some(Color::token(ColorToken::Danger).with_alpha(0.15))
    } else {
        None
    };
    if let Some(background) = color {
        ranges.sort_by_key(|range| range.range.start);
        let mut gaps = Vec::new();
        let mut cursor = 0;
        for range in ranges.iter_mut() {
            let start = range.range.start.min(line.len());
            if cursor < start {
                gaps.push(StyledRange {
                    range: cursor..start,
                    color: None,
                    background: Some(background),
                    decoration: None,
                });
            }
            if range.background.is_none() {
                range.background = Some(background);
            }
            cursor = cursor.max(range.range.end.min(line.len()));
        }
        if cursor < line.len() {
            gaps.push(StyledRange {
                range: cursor..line.len(),
                color: None,
                background: Some(background),
                decoration: None,
            });
        }
        ranges.extend(gaps);
    }
}
pub(super) fn plain_highlight(
    editor: TextEditing,
    highlighted: Value<Vec<u32>>,
    diff: Value<bool>,
) -> TextEditing {
    editor.line_styles(
        move || (highlighted.get(), diff.get()),
        move |(highlighted, diff), line, index: &usize| {
            let mut ranges = Vec::new();
            line_background(&mut ranges, line, *index, highlighted, *diff);
            (ranges, index + 1)
        },
    )
}
pub(super) fn code_keys(
    state: &mut EditState,
    request: EditRequest<'_>,
    use_context: &EditContext<'_>,
) -> EditOutcome {
    let EditRequest::Key { key, modifiers } = request else {
        return text_edit_policy(state, request, use_context);
    };
    if modifiers.ctrl || modifiers.meta || modifiers.alt {
        return text_edit_policy(state, request, use_context);
    }
    let caret = state.caret();
    let to = use_context
        .line_of(state, caret)
        .and_then(|line| match key {
            Key::Home => Some(use_context.line_range(state, line)?.start),
            Key::End => Some(use_context.line_range(state, line)?.end),
            Key::ArrowUp if line == 0 => Some(0),
            Key::ArrowDown if line + 1 == use_context.line_count(state)? => {
                Some(state.value().len())
            }
            Key::ArrowUp | Key::ArrowDown => {
                let next = if key == Key::ArrowUp {
                    line - 1
                } else {
                    line + 1
                };
                use_context.offset_at(state, next, use_context.x_of(state, caret)?)
            }
            _ => None,
        });
    match to {
        Some(to) => {
            state.move_to(to, modifiers.shift);
            EditOutcome::Done { changed: false }
        }
        None => text_edit_policy(state, request, use_context),
    }
}
