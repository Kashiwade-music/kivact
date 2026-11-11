//! Role: Basic code-size variants and recipes for the Core source editor.
use crate::Tokens;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Direction, LineHeight, Style, TextAlign, Theme, pct, style};
use kivact::ui::{CodeBlockRecipes, CodeBlockState, Look};
/// Text size and padding of a standard code block.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CodeBlockSize {
    #[default]
    Default,
    Sm,
}
impl kivact::ui::Variant for CodeBlockSize {}
kivact::state::constant_value!(CodeBlockSize);
/// A constant or reactive code-size presentation, independent of source editing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CodeBlockLook {
    size: Value<CodeBlockSize>,
}
impl Default for CodeBlockLook {
    fn default() -> Self {
        Self {
            size: CodeBlockSize::Default.into_value(),
        }
    }
}
impl kivact::ui::Variant for CodeBlockLook {}
impl CodeBlockLook {
    pub fn size(mut self, size: impl IntoValue<CodeBlockSize>) -> Self {
        self.size = size.into_value();
        self
    }
}
fn metrics(s: &Look<CodeBlockState>) -> (f64, f64, f64) {
    match s
        .variant::<CodeBlockLook>()
        .map(|s| s.size.get())
        .unwrap_or_else(|| s.variant::<CodeBlockSize>().copied().unwrap_or_default())
    {
        CodeBlockSize::Default => (14.0, 23.0, 12.0),
        CodeBlockSize::Sm => (12.0, 20.0, 8.0),
    }
}
pub(crate) fn recipes(r: CodeBlockRecipes) -> CodeBlockRecipes {
    r.root(|_| root)
        .header(|_| header)
        .title(|_| title)
        .copy(|_| copy)
        .body(|_| body)
        .viewport(|_| viewport)
        .code(|_| code)
        .gutter(|_| gutter)
        .footer(|_| footer)
}
fn root(t: &Theme, _: &Look<CodeBlockState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Column)
        .min_width(0)
        .radius(0)
        .border(1, p.input)
        .background(p.background)
        .color(p.foreground)
}
fn header(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(8)
        .padding_x(metrics(s).2)
        .padding_top(metrics(s).2)
        .color(t.ext::<Tokens>().muted_foreground)
}
fn title(_: &Theme, _: &Look<CodeBlockState>) -> Style {
    style().grow(1).single_line().font_size(12).weight(500)
}
fn copy(_: &Theme, _: &Look<CodeBlockState>) -> Style {
    style().size(24, 24).padding(0).margin_y(-4)
}
fn body(_: &Theme, _: &Look<CodeBlockState>) -> Style {
    style()
        .direction(Direction::RowReverse)
        .gap(12)
        .align_items(Align::Start)
        .min_width(pct(100))
}
fn viewport(_: &Theme, s: &Look<CodeBlockState>) -> Style {
    let pad = metrics(s).2;
    style()
        .min_height(0)
        .padding_x(pad)
        .padding_top(if s.has_header() { pad - 4.0 } else { pad })
        .padding_bottom(if s.has_footer { pad - 4.0 } else { pad })
}
fn code(_: &Theme, s: &Look<CodeBlockState>) -> Style {
    let (font, line, _) = metrics(s);
    style()
        .font_family(crate::typography::MONO)
        .font_size(font)
        .line_height(LineHeight::px(line))
        .grow(1)
        .shrink(0)
        .text_wrap(s.wrapped)
}
fn gutter(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    code(t, s)
        .grow(0)
        .min_width(24)
        .text_align(TextAlign::End)
        .text_wrap(false)
        .color(t.ext::<Tokens>().muted_foreground)
}
fn footer(t: &Theme, s: &Look<CodeBlockState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(8)
        .padding_x(metrics(s).2)
        .padding_bottom(metrics(s).2)
        .color(t.ext::<Tokens>().muted_foreground)
}
