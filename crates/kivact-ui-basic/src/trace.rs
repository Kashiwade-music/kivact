//! Role: Basic activity-detail appearance for the canonical disclosure content part.
use crate::Tokens;
use kivact::style::{Direction, Style, Theme, style};
pub(crate) fn content(t: &Theme) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Column)
        .min_width(0)
        .margin_top(8)
        .margin_bottom(4)
        .margin_left(6)
        .padding_left(15)
        .padding_right(4)
        .border_left(1, p.border)
        .line_height(1.625)
        .color(p.muted_foreground)
        .font_size(12)
}
