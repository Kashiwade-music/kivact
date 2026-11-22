//! Role: Style Astryx Neutral's empty states: a centered column of icon, title,
//! description and actions, in regular or compact spacing.
use crate::color as c;
use crate::space as s;
use crate::type_scale as t;
use crate::variants::EmptyStateVariant;
use kivact::style::{Align, FlexDirection, Justify, Style, TextAlign, Theme, pct, style};
use kivact::ui::theme::{EmptyStateState, EmptyStateTheme, Look};
use kivact::view::{ReadingDirection, use_direction};

/// Whether the empty state uses the compact spacing scale.
fn compact(look: &Look<EmptyStateState>) -> bool {
    look.variant::<EmptyStateVariant>()
        .is_some_and(|variant| variant.is_compact())
}

/// The root: a centered column; compact spacing when the variant asks for it.
fn root(_theme: &Theme, look: &Look<EmptyStateState>, _base: Style) -> Style {
    let (gap, x, y) = if compact(look) {
        (s::SPACE_S2, s::SPACE_S4, s::SPACE_S4)
    } else {
        (s::SPACE_S4, s::SPACE_S6, s::SPACE_S8)
    };
    style()
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .width(pct(100))
        .min_width(0.0)
        .gap(gap)
        .padding_x(x)
        .padding_y(y)
}

/// The title: the large text in the primary color, centered.
fn title(_theme: &Theme, look: &Look<EmptyStateState>, _base: Style) -> Style {
    let size = if compact(look) {
        t::SIZE_MD
    } else {
        t::LARGE.size_px
    };
    style()
        .max_width(360.0)
        .min_width(0.0)
        .text_align(TextAlign::Center)
        .font_size(size)
        .line_height(t::LARGE.line_height_px / t::LARGE.size_px)
        .font_weight(t::LARGE.weight)
        .color(c::TEXT_PRIMARY)
}

/// The description: the body text in the secondary color, pulled up under the title.
fn description(theme: &Theme, look: &Look<EmptyStateState>, _base: Style) -> Style {
    let (offset, size) = if compact(look) {
        (theme.get(&s::SPACE_S2_VAR), t::SIZE_SM)
    } else {
        (theme.get(&s::SPACE_S4_VAR), t::BODY.size_px)
    };
    style()
        .max_width(360.0)
        .min_width(0.0)
        .text_align(TextAlign::Center)
        .margin_top(-offset)
        .font_size(size)
        .line_height(t::BODY.line_height_px / t::BODY.size_px)
        .color(c::TEXT_SECONDARY)
}

/// The icon: a centered box.
fn icon(_theme: &Theme, _look: &Look<EmptyStateState>, _base: Style) -> Style {
    style()
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

/// The actions: a centered row, or a column in compact spacing.
fn actions(_theme: &Theme, look: &Look<EmptyStateState>, _base: Style) -> Style {
    let direction = if compact(look) {
        FlexDirection::Column
    } else if use_direction() == ReadingDirection::Rtl {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    };
    style()
        .flex_direction(direction)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(s::SPACE_S2)
        .margin_top(s::SPACE_S1)
}

/// Install the kit's recipes of the empty state.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        EmptyStateTheme::default()
            .root(root)
            .title(title)
            .description(description)
            .icon(icon)
            .actions(actions),
    )
}
