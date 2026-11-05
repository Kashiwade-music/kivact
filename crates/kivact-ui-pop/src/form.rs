//! Role: Share the Pop text-input surface style between standard input recipes.

use crate::Tokens;
use kivact::draw::Shadow;
use kivact::style::{ColorToken, Theme, style};
pub(crate) fn input_style(t: &Theme) -> kivact::style::Style {
    let p = t.ext::<Tokens>();
    style()
        .height(36)
        .min_height(0)
        .padding_x(12)
        .padding_y(8)
        .radius(8)
        .border(1, p.line)
        .background(p.surface)
        .color(p.text)
        .font_size(14)
        .line_height(1.2)
        .shadow(Shadow::new(0, 1, 2, crate::tokens::subtle_shadow()))
        .focus(style().border_color(p.blue).ring(3, ColorToken::Focus))
}
