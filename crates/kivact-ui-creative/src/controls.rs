//! Role: The toolbar button look shared by the kit's recipes.
use kivact::motion::{Easing, Transition};
use kivact::style::{Align, Direction, Justify, Style, Theme, TransitionProp, rgba, style};

pub(crate) fn button_style(t: &Theme) -> Style {
    let p = t.get::<crate::tokens::Tokens>();
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .height(p.row)
        .gap(6)
        .padding_x(11)
        .radius(7)
        .background(rgba(0xffffff, 0.065))
        .border(1, rgba(0xffffff, 0.04))
        .color(p.text)
        .weight(500)
        .shrink(0)
        .hover(style().background(rgba(0xffffff, 0.1)).color(p.text_high))
        .pressed(style().scale(0.97).background(rgba(0xffffff, 0.05)))
        .selected(
            style()
                .background(p.selection)
                .color(p.accent_high)
                .border(1, p.accent.with_alpha(0.35)),
        )
        .transition(
            TransitionProp::Paint,
            Transition::tween(150.0, Easing::CubicBezier(0.2, 0.8, 0.2, 1.0)),
        )
}
