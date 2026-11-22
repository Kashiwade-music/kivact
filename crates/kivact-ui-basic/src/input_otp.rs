//! Role: One-time-code input: slots drawn over one hidden native text edit.

use crate::recipe_parts::{field_fill, focus_ring};
use crate::{FOREGROUND, INPUT};
use kivact::component;
use kivact::custom::Caller;
use kivact::interaction::use_text_edit;
use kivact::state::{Callback, IntoValue, Signal, Value, binding, memo};
use kivact::style::{Align, Color, FlexDirection, Justify, Position, ThemeMode, style, use_theme};
use kivact::view::{Element, Node, Role, View, icon, node, node_ref, row, text};

/// The characters an [`InputOtp`] accepts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OtpPattern {
    /// `0-9` only.
    #[default]
    Digits,
    /// ASCII letters and digits.
    Alphanumeric,
}

impl OtpPattern {
    fn accepts(self, c: char) -> bool {
        match self {
            OtpPattern::Digits => c.is_ascii_digit(),
            OtpPattern::Alphanumeric => c.is_ascii_alphanumeric(),
        }
    }
}

/// A code input with one slot per character (`InputOTP`).
///
/// A code input of `length` characters bound to `value`. A single hidden
/// native text edit takes the typing, so caret, selection, paste and IME stay
/// native; the slots show its characters and a caret in the active slot.
/// Name it with `.name(..)`. Split the slots with `.groups(..)`, accept
/// letters with `.pattern(..)`, and react to a full code with
/// `.on_complete(..)`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn verify() -> impl IntoView {
///     let code = signal(String::new());
///     basic::input_otp(code, 6usize)
///         .name("Verification code")
///         .groups([3, 3])
///         .on_complete(move |code| println!("{code}"))
/// }
/// ```
#[component]
pub fn input_otp(
    value: Signal<String>,
    length: usize,
    groups: Option<Vec<usize>>,
    #[prop(default = OtpPattern::Digits)] pattern: OtpPattern,
    on_complete: Option<Callback<String>>,
    #[caller] caller: Caller,
) -> impl Element {
    let length = length.max(1);
    let groups = match groups {
        Some(groups) => split_groups(&groups, length),
        None => vec![length],
    };
    // `.disabled(..)` ignores input and dims the slots.
    let disabled: Option<Value<bool>> = caller.disabled();
    let name = caller.name().unwrap_or_else(|| String::new().into_value());
    let current = value.into_value();
    let filtered = accepted_value(value, current, pattern, length, on_complete);
    let editor_ref = node_ref();
    let focused = editor_ref.focused();
    let active = memo(move || current.with(|v| v.chars().count()).min(length - 1));
    let mut editor = node()
        .node_ref(editor_ref)
        .name(name)
        .with_node(|root| root.part(use_text_edit(filtered).max_length(length).control()))
        .style(
            style()
                .position(Position::Absolute)
                .inset(0)
                .opacity(0.0)
                .caret_color(Color::TRANSPARENT),
        );
    if let Some(disabled) = disabled {
        editor = editor.disabled(disabled);
    }
    let mut cells: Vec<View> = Vec::new();
    let mut index = 0;
    for (n, size) in groups.iter().enumerate() {
        if n > 0 {
            cells.push(
                node()
                    .with_node(|root| root.role(Role::Separator))
                    .child(icon("minus"))
                    .style(
                        style()
                            .flex_direction(FlexDirection::Row)
                            .align_items(Align::Center)
                            .padding_x(2)
                            .color(FOREGROUND),
                    )
                    .into_view(),
            );
        }
        let slots = (0..*size)
            .map(|local| {
                let at = index;
                index += 1;
                slot(at, current, active, focused, local == 0)
            })
            .collect::<Vec<_>>();
        cells.push(row(slots).into_view());
    }
    node()
        .base_style(move || {
            let s = style()
                .position(Position::Relative)
                .flex_shrink(0.0)
                .flex_direction(FlexDirection::Row);
            if disabled.is_some_and(|d| d.get()) {
                s.opacity(0.5)
            } else {
                s
            }
        })
        .with_node(|root| root.role(Role::Group))
        .child(row(cells).style(style().align_items(Align::Center)))
        .child(editor)
}

/// Split `length` slots into the sizes of `groups`, the last group taking
/// the rest. Sizes beyond the length are cut off.
fn split_groups(groups: &[usize], length: usize) -> Vec<usize> {
    let mut left = length;
    let mut out = Vec::new();
    for &size in groups.iter().filter(|s| **s > 0) {
        let size = size.min(left);
        if size == 0 {
            break;
        }
        out.push(size);
        left -= size;
    }
    if left > 0 {
        out.push(left);
    }
    out
}

/// One character cell: borders on top, bottom and right, and the left one on
/// the first slot of a group, so neighbours share their edges.
fn slot(
    at: usize,
    current: Value<String>,
    active: Value<usize>,
    focused: Value<bool>,
    first: bool,
) -> Node {
    let glyph = memo(move || current.with(|v| v.chars().nth(at)).map(String::from));
    let is_active = memo(move || focused.get() && active.get() == at);
    node()
        .child(text(move || glyph.get().unwrap_or_default()))
        .child(node().style(move || {
            // The caret of the active empty slot.
            let show = is_active.get() && glyph.get().is_none();
            style()
                .position(Position::Absolute)
                .left(15)
                .top(8)
                .width(1)
                .height(16)
                .background(if show { FOREGROUND } else { Color::TRANSPARENT })
        }))
        .style(move || {
            let theme = use_theme().get();
            let dark = theme.mode() == ThemeMode::Dark;
            let s = style()
                .position(Position::Relative)
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .size(32, 32)
                .flex_shrink(0.0)
                .margin_left(if first { 0 } else { -1 })
                .font_size(12)
                .color(FOREGROUND)
                .background(field_fill(dark))
                .border(1, INPUT);
            if is_active.get() {
                s.z_index(1).merge(&focus_ring(&theme))
            } else {
                s
            }
        })
}

fn accepted_value(
    value: Signal<String>,
    current: Value<String>,
    pattern: OtpPattern,
    length: usize,
    complete: Option<Callback<String>>,
) -> Signal<String> {
    binding(current, move |typed: String| {
        let kept: String = typed
            .chars()
            .filter(|c| pattern.accepts(*c))
            .take(length)
            .collect();
        value.set(kept.clone());
        if kept.chars().count() == length
            && let Some(complete) = complete
        {
            complete.call(kept);
        }
    })
}
