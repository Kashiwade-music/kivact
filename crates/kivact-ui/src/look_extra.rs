//! Role: Default recipes for standard multi-part components.
use kivact_core::style::{Align, ColorToken, Direction, Style, Theme, style};
use kivact_core::view::Orientation;
use kivact_interaction::{TabListState, TabState};

pub(super) fn tabs_list(_: &Theme, s: &TabListState) -> Style {
    if s.orientation == Orientation::Vertical {
        style()
            .direction(Direction::Column)
            .gap(4)
            .border_right(1, ColorToken::Border)
    } else {
        style()
            .direction(Direction::Row)
            .gap(4)
            .border_bottom(1, ColorToken::Border)
    }
}
pub(super) fn no_tab_indicator(_: &Theme, _: &TabState) -> Style {
    style()
}

pub(super) fn tab(t: &Theme, s: &TabState) -> Style {
    let mark = if s.orientation == Orientation::Vertical {
        style().border_right(2, ColorToken::Accent)
    } else {
        style().border_bottom(2, ColorToken::Accent)
    };
    style()
        .padding_x(t.space().md)
        .height(t.control_height())
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().sm)
        .justify_content(kivact_core::style::Justify::Center)
        .selected(mark)
}
pub(super) fn panel(t: &Theme, _: &TabState) -> Style {
    style().padding(t.space().md)
}
