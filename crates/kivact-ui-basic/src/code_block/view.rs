//! Role: The CodeBlock builder and its highlighted lines, gutter and copy button.

use super::lines::{CodeLine, CodeTone};
use crate::slot::{Slot, slot};
use crate::theme_access::tokens;
use crate::tokens::CodeColors;
use crate::typography::MONO;
use kivact::control::icon;
use kivact::control::{Node, NodeBuilder, ScrollAxes, column, node, row, scroll, text};
use kivact::interaction::{Delays, text_edit_policy, use_delayed};
use kivact::primitive::{
    EditContext, EditOutcome, EditRequest, EditState, TextEditing, TextHighlight, part,
};
use kivact::style::{Align, Color, Direction, Length, LineHeight, style};
use kivact::ui::{ButtonVariant, ControlSize};
use kivact::view::HandlerResult;
use kivact::view::{IntoText, IntoValue, IntoView, Key, Role, View, callback, show};
use kivact::{Callback, Value};
use std::rc::Rc;
use std::time::Duration;

/// The text size and padding of a code block (code-block.tsx `size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CodeBlockSize {
    /// 14 px code on 23 px lines with 12 px padding.
    #[default]
    Default,
    /// 12 px code on 20 px lines with 8 px padding.
    Sm,
}

impl CodeBlockSize {
    fn font(self) -> f64 {
        match self {
            CodeBlockSize::Default => 14.0,
            CodeBlockSize::Sm => 12.0,
        }
    }
    fn pad(self) -> f64 {
        match self {
            CodeBlockSize::Default => 12.0,
            CodeBlockSize::Sm => 8.0,
        }
    }
    /// The line height in pixels (`leading-relaxed`, 1.625, rounded), so
    /// the code's lines and the gutter's numbers lie the same distance apart.
    fn line(self) -> f64 {
        match self {
            CodeBlockSize::Default => 23.0,
            CodeBlockSize::Sm => 20.0,
        }
    }
}

/// A code listing (code-block.tsx). Created by [`code_block`].
pub struct CodeBlock {
    root: Node,
    lines: Value<Vec<CodeLine>>,
    title: Option<Value<String>>,
    copy: Option<Callback<()>>,
    line_numbers: Value<bool>,
    size: CodeBlockSize,
    footer: Option<Slot>,
    max_height: Option<Value<Length>>,
}

/// A bordered block showing `lines` in the monospace family.
///
/// The lines come highlighted: [`rust_lines`] tokenizes Rust, a caller
/// builds [`CodeSpan`]s with tones for other languages, and [`code_lines`]
/// leaves text plain. Each tone is drawn in its color of the theme's
/// [`CodeColors`] (`Tokens::code`).
///
/// The code is a read-only, multiline text box (`Role::TextBox`, named by
/// the title, else "Code", its value the code) in the Tab order: the
/// pointer selects by dragging; the arrows move a caret (up and down by
/// line), Home and End go to the line's start and end, Ctrl+Home and
/// Ctrl+End to the code's, Shift selects, Ctrl+A selects all and Ctrl+C
/// (Cmd+C) copies the selection to the clipboard. The content scrolls horizontally and, with
/// [`CodeBlock::max_height`], vertically.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn snippet() -> impl IntoView {
///     basic::code_block(basic::rust_lines("fn main() {}\n"))
///         .title("main.rs")
///         .on_copy(|| {
///             // Write the text to the clipboard through the application.
///         })
/// }
/// ```
#[track_caller]
pub fn code_block(lines: impl IntoValue<Vec<CodeLine>>) -> CodeBlock {
    CodeBlock {
        root: column(()).style(move || {
            let p = tokens();
            style()
                .direction(Direction::Column)
                .min_width(0)
                .radius(0)
                .border(1, p.input)
                .background(if p.is_dark() {
                    p.input.with_alpha(0.3)
                } else {
                    Color::TRANSPARENT
                })
                .color(p.foreground)
        }),
        lines: lines.into_value(),
        title: None,
        copy: None,
        line_numbers: false.into_value(),
        size: CodeBlockSize::Default,
        footer: None,
        max_height: None,
    }
}

impl CodeBlock {
    /// Show `title` in the header; it also names the code's text box.
    pub fn title(mut self, title: impl IntoText) -> Self {
        self.title = Some(title.into_text_value());
        self
    }

    /// Show a copy button in the header that reports presses to `on_copy`.
    ///
    /// Writing the whole code to the clipboard is the application's (a
    /// selection copies itself with Ctrl+C). After a press the button
    /// shows a check and is named "Copied" for two seconds.
    pub fn on_copy<R: HandlerResult>(mut self, on_copy: impl Fn() -> R + 'static) -> Self {
        self.copy = Some(callback(move |()| on_copy()));
        self
    }

    /// Number the lines from 1 in a muted gutter while true (constant or
    /// reactive).
    #[track_caller]
    pub fn line_numbers(mut self, on: impl IntoValue<bool>) -> Self {
        self.line_numbers = on.into_value();
        self
    }

    /// Choose the text size and padding.
    pub fn size(mut self, size: CodeBlockSize) -> Self {
        self.size = size;
        self
    }

    /// Show `footer` under the code.
    pub fn footer<V: IntoView>(mut self, footer: impl FnOnce() -> V + 'static) -> Self {
        self.footer = Some(slot(footer));
        self
    }

    /// Scroll vertically beyond `height` of code, constant or reactive.
    #[track_caller]
    pub fn max_height(mut self, height: impl IntoValue<Length>) -> Self {
        self.max_height = Some(height.into_value());
        self
    }
}

impl NodeBuilder for CodeBlock {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// The color of `tone`; plain text and punctuation keep the foreground.
fn tone_color(colors: &CodeColors, tone: CodeTone) -> Option<Color> {
    match tone {
        CodeTone::Plain | CodeTone::Punctuation => None,
        CodeTone::Keyword => Some(colors.keyword),
        CodeTone::String => Some(colors.string),
        CodeTone::Number => Some(colors.number),
        CodeTone::Comment => Some(colors.comment),
        CodeTone::Function => Some(colors.function),
        CodeTone::Type => Some(colors.type_name),
        CodeTone::Macro => Some(colors.macro_call),
        CodeTone::Lifetime => Some(colors.lifetime),
        CodeTone::Attribute => Some(colors.attribute),
    }
}

/// The colored ranges of `text`, the editor's line `index`, from the
/// spans of that [`CodeLine`] (the text is theirs, joined); and the index
/// of the next line, the state the editor passes on.
fn line_colors(
    (lines, colors): &(Vec<CodeLine>, CodeColors),
    text: &str,
    index: &usize,
) -> (Vec<TextHighlight>, usize) {
    let mut out = Vec::new();
    let mut at = 0;
    for span in lines.get(*index).map_or(&[][..], |line| &line.0) {
        let range = at.min(text.len())..(at + span.text.len()).min(text.len());
        at += span.text.len();
        if let Some(color) = tone_color(colors, span.tone)
            && !range.is_empty()
        {
            out.push(TextHighlight { range, color });
        }
    }
    (out, index + 1)
}

/// The keys of the code's text box: the standard policy, with Home and End
/// going to the start and end of the caret's line and ArrowUp and
/// ArrowDown moving a line (Shift selects; Ctrl+Home and Ctrl+End still go
/// to the start and end of the code).
fn code_keys(
    state: &mut EditState,
    request: EditRequest<'_>,
    context: &EditContext<'_>,
) -> EditOutcome {
    let EditRequest::Key { key, modifiers } = request else {
        return text_edit_policy(state, request, context);
    };
    if modifiers.ctrl || modifiers.meta || modifiers.alt {
        return text_edit_policy(state, request, context);
    }
    let caret = state.caret();
    let to = context.line_of(state, caret).and_then(|line| match key {
        Key::Home => Some(context.line_range(state, line)?.start),
        Key::End => Some(context.line_range(state, line)?.end),
        Key::ArrowUp if line == 0 => Some(0),
        Key::ArrowDown if line + 1 == context.line_count(state)? => Some(state.value().len()),
        Key::ArrowUp | Key::ArrowDown => {
            let next = if key == Key::ArrowUp {
                line - 1
            } else {
                line + 1
            };
            context.offset_at(state, next, context.x_of(state, caret)?)
        }
        _ => None,
    });
    match to {
        Some(to) => {
            state.move_to(to, modifiers.shift);
            EditOutcome::Done { changed: false }
        }
        None => text_edit_policy(state, request, context),
    }
}

/// The code as one read-only, multiline text box, colored line by line
/// and as high as its lines (a text box never scrolls vertically itself).
fn code_view(lines: Value<Vec<CodeLine>>, name: Value<String>, size: CodeBlockSize) -> Node {
    let value = move || {
        lines.with(|lines| {
            let text = lines.iter().map(CodeLine::text).collect::<Vec<_>>();
            Rc::<str>::from(text.join("\n"))
        })
    };
    let editing = TextEditing::new(value)
        .multiline(true)
        .read_only(true)
        .behavior(code_keys)
        .highlight(move || (lines.get(), tokens().code.clone()), line_colors);
    let control = part(|state| state)
        .role(Role::TextBox)
        .focusable(true)
        .name(name)
        .text_editing(editing);
    let count = move || lines.with(|lines| lines.len().max(1));
    node().part(control).style(move || {
        style()
            .grow(1)
            .shrink(0)
            .height(count() as f64 * size.line())
    })
}

/// The line numbers from 1, one per line, right-aligned in a muted gutter.
fn gutter(lines: Value<Vec<CodeLine>>) -> Node {
    let numbers = move || {
        let count = lines.with(|lines| lines.len().max(1));
        (1..=count)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    };
    text(numbers).style(
        style()
            .min_width(24)
            .shrink(0)
            .text_align(kivact::style::TextAlign::End)
            .color(crate::Color::MutedForeground),
    )
}

/// The header copy button: `copy` until pressed, then `check` for two seconds.
fn copy_button(on_copy: Callback<()>) -> View {
    let feedback = use_delayed(
        false,
        Delays {
            open: Duration::ZERO,
            close: Duration::from_secs(2),
        },
    );
    let copied = feedback.shown();
    let copy_name = kivact::i18n::message("basic.code_block.copy", "Copy");
    let copied_name = kivact::i18n::message("basic.code_block.copied", "Copied");
    let name = move || {
        if copied.get() {
            copied_name.get()
        } else {
            copy_name.get()
        }
    };
    let button = kivact::ui::button(name)
        .variant(ButtonVariant::Ghost)
        .size(ControlSize::Small)
        .hide_label(true)
        .child(show(copied, || icon("check")))
        .child(show(move || !copied.get(), || icon("copy")))
        .style(style().size(24, 24).padding(0).margin_y(-4))
        .on_press(move || {
            on_copy.call(());
            feedback.pulse();
        });
    let status = node()
        .role(Role::Status)
        .name(move || {
            if copied.get() {
                copied_name.get()
            } else {
                String::new()
            }
        })
        .style(style().size(0, 0));
    (button, status).into_view()
}

impl IntoView for CodeBlock {
    fn into_view(self) -> View {
        let size = self.size;
        let pad = size.pad();
        let has_header = self.title.is_some() || self.copy.is_some();
        let has_footer = self.footer.is_some();
        let name = self
            .title
            .unwrap_or_else(|| kivact::i18n::message("basic.code_block.name", "Code"));
        let header = has_header.then(|| {
            let title = self.title.map(|title| {
                text(title).style(style().grow(1).single_line().font_size(12).weight(500))
            });
            row((
                title.unwrap_or_else(|| node().style(style().grow(1))),
                self.copy.map(copy_button),
            ))
            .style(
                style()
                    .align_items(Align::Center)
                    .gap(8)
                    .padding_x(pad)
                    .padding_top(pad)
                    .color(crate::Color::MutedForeground),
            )
        });
        let lines = self.lines;
        let numbered = self.line_numbers;
        let body = row((
            show(numbered, move || gutter(lines)),
            code_view(lines, name, size),
        ))
        .style(
            style()
                .gap(12)
                .align_items(Align::Start)
                .min_width(kivact::style::pct(100)),
        );
        let max_height = self.max_height;
        let content = scroll(body).axes(ScrollAxes::Both).style(move || {
            let s = style()
                .min_height(0)
                .padding_x(pad)
                .padding_top(if has_header { pad - 4.0 } else { pad })
                .padding_bottom(if has_footer { pad - 4.0 } else { pad })
                .font_family(MONO)
                .font_size(size.font())
                .line_height(LineHeight::px(size.line()));
            match max_height {
                Some(height) => s.max_height(height.get()),
                None => s,
            }
        });
        let footer = self.footer.map(|footer| {
            row(footer).style(
                style()
                    .align_items(Align::Center)
                    .gap(8)
                    .padding_x(pad)
                    .padding_bottom(pad)
                    .color(crate::Color::MutedForeground),
            )
        });
        self.root
            .child(header)
            .child(content)
            .child(footer)
            .into_view()
    }
}
