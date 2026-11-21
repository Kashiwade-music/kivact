//! Role: Lyra recipes of the code block: its frame, header, body, gutter and footer.
//!
//! The looks of the code block of the kit: a bordered frame, a muted header
//! and footer, and the monospace code with its line numbers in the gutter.

use crate::color::{BACKGROUND, FOREGROUND, INPUT, MUTED_FOREGROUND};
use crate::recipe_parts::MONO;
use crate::variants::CodeBlockSize;
use kivact::style::{Align, FlexDirection, LineHeight, Style, TextAlign, Theme, pct, style};
use kivact::ui::theme::{CodeBlockState, CodeBlockTheme, Look};

/// The font size, line height and padding of the code block's size.
fn metrics(s: &Look<CodeBlockState>) -> (f64, f64, f64) {
    match s.variant::<CodeBlockSize>().unwrap_or_default() {
        CodeBlockSize::Default => (14.0, 23.0, 12.0),
        CodeBlockSize::Sm => (12.0, 20.0, 8.0),
    }
}

/// `flex-col rounded-none border bg-background`: the frame of the block.
fn code_block_root(_: &Theme, _: &Look<CodeBlockState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .min_width(0.0)
        .border_radius(0.0)
        .border(1.0, INPUT)
        .background(BACKGROUND)
        .color(FOREGROUND)
}

/// The header: a row of the title and the copy button, in the muted color.
fn code_block_header(_: &Theme, s: &Look<CodeBlockState>, _base: Style) -> Style {
    let pad = metrics(s).2;
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(8.0)
        .padding_x(pad)
        .padding_top(pad)
        .color(MUTED_FOREGROUND)
}

fn code_block_title(_: &Theme, _: &Look<CodeBlockState>, _base: Style) -> Style {
    style()
        .flex_grow(1.0)
        .single_line()
        .font_size(12.0)
        .font_weight(500)
}

fn code_block_copy(_: &Theme, _: &Look<CodeBlockState>, _base: Style) -> Style {
    style().size(24.0, 24.0).padding(0.0).margin_y(-4.0)
}

/// The body: the gutter after the code, reversed, with 12 px between them.
fn code_block_body(_: &Theme, _: &Look<CodeBlockState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::RowReverse)
        .gap(12.0)
        .align_items(Align::Start)
        .min_width(pct(100.0))
}

/// The viewport of the code: the block's padding, less a little under a header or a footer.
fn code_block_viewport(_: &Theme, s: &Look<CodeBlockState>, _base: Style) -> Style {
    let pad = metrics(s).2;
    style()
        .min_height(0.0)
        .padding_x(pad)
        .padding_top(if s.has_header() { pad - 4.0 } else { pad })
        .padding_bottom(if s.has_footer { pad - 4.0 } else { pad })
}

/// The monospace text of the code at the size's font and line height.
fn code_text(s: &Look<CodeBlockState>) -> Style {
    let (font, line, _) = metrics(s);
    style()
        .font_family(MONO)
        .font_size(font)
        .line_height(LineHeight::px(line))
        .flex_grow(1.0)
        .flex_shrink(0.0)
        .text_wrap(s.wrapped)
}

fn code_block_code(_: &Theme, s: &Look<CodeBlockState>, _base: Style) -> Style {
    code_text(s)
}

/// The line numbers: right-aligned in the muted color, at least 24 px wide.
fn code_block_gutter(_: &Theme, s: &Look<CodeBlockState>, _base: Style) -> Style {
    code_text(s)
        .flex_grow(0.0)
        .min_width(24.0)
        .text_align(TextAlign::End)
        .text_wrap(false)
        .color(MUTED_FOREGROUND)
}

fn code_block_footer(_: &Theme, s: &Look<CodeBlockState>, _base: Style) -> Style {
    let pad = metrics(s).2;
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(8.0)
        .padding_x(pad)
        .padding_bottom(pad)
        .color(MUTED_FOREGROUND)
}

/// Installs the code block's parts.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        CodeBlockTheme::default()
            .root(code_block_root)
            .header(code_block_header)
            .title(code_block_title)
            .copy(code_block_copy)
            .body(code_block_body)
            .viewport(code_block_viewport)
            .code(code_block_code)
            .gutter(code_block_gutter)
            .footer(code_block_footer),
    )
}
