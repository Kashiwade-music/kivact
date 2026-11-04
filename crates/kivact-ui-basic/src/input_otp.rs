//! Role: One-time-code input: slots drawn over one hidden native text edit.

use crate::theme_access::{field_fill, tokens};
use kivact::interaction::use_text_edit;
use kivact::state::IntoBinding;
use kivact::view::icon;
use kivact::view::{Node, NodeBuilder, node, row, text};

use kivact::state::{Binding, Callback, Value, binding, memo};
use kivact::state::{IntoText, IntoValue, callback};
use kivact::style::{Align, Color, Direction, Justify, Position, style};
use kivact::view::HandlerResult;
use kivact::view::{IntoView, Role, View, node_ref};

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
/// Created by [`input_otp`]. A single hidden native text edit takes the
/// typing, so caret, selection, paste and IME stay native; the slots show its
/// characters and a caret in the active slot.
pub struct InputOtp {
    root: Node,
    label: Value<String>,
    value: Binding<String>,
    length: usize,
    groups: Vec<usize>,
    pattern: OtpPattern,
    complete: Option<Callback<String>>,
}

/// A code input of `length` characters named `label`.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn verify() -> impl IntoView {
///     let code = signal(String::new());
///     basic::input_otp("Verification code", code, 6)
///         .groups(&[3, 3])
///         .on_complete(move |code| println!("{code}"))
/// }
/// ```
#[track_caller]
pub fn input_otp(
    label: impl IntoText,
    value: impl kivact::state::IntoBinding<String>,
    length: usize,
) -> InputOtp {
    let length = length.max(1);
    InputOtp {
        root: node(),
        label: label.into_text_value(),
        value: value.into_binding(),
        length,
        groups: vec![length],
        pattern: OtpPattern::Digits,
        complete: None,
    }
}

impl InputOtp {
    /// Split the slots into groups of these sizes, with a separator between
    /// them (`InputOTPGroup`). Sizes beyond the length are cut off; slots
    /// left over form a last group.
    pub fn groups(mut self, groups: &[usize]) -> Self {
        let mut left = self.length;
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
        self.groups = out;
        self
    }

    /// The accepted characters (default [`OtpPattern::Digits`]).
    pub fn pattern(mut self, pattern: OtpPattern) -> Self {
        self.pattern = pattern;
        self
    }

    /// Called with the code when the user has typed all `length` characters.
    pub fn on_complete<R: HandlerResult>(mut self, f: impl Fn(String) -> R + 'static) -> Self {
        self.complete = Some(callback(f));
        self
    }
}

impl NodeBuilder for InputOtp {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for InputOtp {
    fn into_view(self) -> View {
        let InputOtp {
            mut root,
            label,
            value,
            length,
            groups,
            pattern,
            complete,
        } = self;
        // `.disabled(..)` ignores input and dims the slots.
        let disabled: Option<Value<bool>> = root.accessible_disabled();
        let current = value.into_value();
        // Keep only accepted characters, at most `length` of them.
        let filtered = binding(current, move |typed: String| {
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
        });
        let editor_ref = node_ref();
        let focused = editor_ref.focused();
        let active = memo(move || current.with(|v| v.chars().count()).min(length - 1));
        let mut editor = node()
            .node_ref(editor_ref)
            .name(label)
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
                                .direction(Direction::Row)
                                .align_items(Align::Center)
                                .padding_x(2)
                                .color(crate::Color::Foreground),
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
        root.base_style(move || {
            let s = style()
                .position(Position::Relative)
                .shrink(0)
                .direction(Direction::Row);
            if disabled.is_some_and(|d| d.get()) {
                s.opacity(0.5)
            } else {
                s
            }
        })
        .with_node(|root| root.role(Role::Group))
        .child(row(cells).style(style().align_items(Align::Center)))
        .child(editor)
        .into_view()
    }
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
            let p = tokens();
            // The caret of the active empty slot.
            let show = is_active.get() && glyph.get().is_none();
            style()
                .position(Position::Absolute)
                .left(15)
                .top(8)
                .width(1)
                .height(16)
                .background(if show {
                    p.foreground
                } else {
                    Color::TRANSPARENT
                })
        }))
        .style(move || {
            let p = tokens();
            let s = style()
                .position(Position::Relative)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .size(32, 32)
                .shrink(0)
                .margin_left(if first { 0 } else { -1 })
                .font_size(12)
                .color(p.foreground)
                .background(field_fill(&p))
                .border(1, p.input);
            if is_active.get() {
                s.z_index(1).merge(&crate::theme_access::focus_ring(&p))
            } else {
                s
            }
        })
}
