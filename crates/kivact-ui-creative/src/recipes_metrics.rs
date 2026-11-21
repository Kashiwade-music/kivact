//! Role: Creative typography and read-only description styling for the standard heading, stat and description list.
use crate::color;
use crate::fonts::mono_family;
use crate::size::ROW;
use kivact::style::{Style, Theme};
use kivact::ui::theme::{
    DescriptionListTheme, HeadingState, HeadingTheme, Look, StatState, StatTheme,
};

/// Install the metrics' recipes into `theme`.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(heading()).with(stat()).with(description_list())
}

/// The theme of a heading: a semibold text in the high text color.
fn heading() -> HeadingTheme {
    HeadingTheme::default().root(heading_root)
}

fn heading_root(_t: &Theme, _s: &Look<HeadingState>, base: Style) -> Style {
    base.font_weight(600).color(color::TEXT_HIGH)
}

/// The theme of a stat: a card with a label, a large mono value and a delta.
fn stat() -> StatTheme {
    StatTheme::default()
        .root(stat_root)
        .label(stat_label)
        .value(stat_value)
        .delta(stat_delta)
        .icon(stat_icon)
        .chart(stat_chart)
}

fn stat_root(_t: &Theme, _s: &Look<StatState>, base: Style) -> Style {
    base.padding(12)
        .gap(4)
        .border_radius(9)
        .background(color::ISLAND)
        .border(1, color::HAIR)
}

fn stat_label(_t: &Theme, _s: &Look<StatState>, base: Style) -> Style {
    base.font_size(10.5).color(color::MUTED)
}

fn stat_value(t: &Theme, _s: &Look<StatState>, base: Style) -> Style {
    base.font_family(mono_family(t))
        .font_size(24)
        .font_weight(600)
        .color(color::TEXT_HIGH)
}

fn stat_delta(_t: &Theme, _s: &Look<StatState>, base: Style) -> Style {
    base.font_size(10.5).color(color::MUTED)
}

fn stat_icon(_t: &Theme, _s: &Look<StatState>, base: Style) -> Style {
    base.color(color::DIM)
}

fn stat_chart(_t: &Theme, _s: &Look<StatState>, base: Style) -> Style {
    base.min_width(0)
}

/// The theme of a description list: rows of muted terms and text details.
fn description_list() -> DescriptionListTheme {
    DescriptionListTheme::default()
        .root(description_root)
        .item(description_item)
        .term(description_term)
        .detail(description_detail)
}

fn description_root(_t: &Theme, _s: &Look<()>, base: Style) -> Style {
    base.gap(4)
}

fn description_item(_t: &Theme, _s: &Look<()>, base: Style) -> Style {
    base.gap(8).min_height(ROW)
}

fn description_term(_t: &Theme, _s: &Look<()>, base: Style) -> Style {
    base.font_size(11.5).color(color::MUTED)
}

fn description_detail(_t: &Theme, _s: &Look<()>, base: Style) -> Style {
    base.font_size(11.5).color(color::TEXT)
}
