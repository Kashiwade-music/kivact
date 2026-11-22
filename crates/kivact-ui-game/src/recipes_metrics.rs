//! Role: Style semantic statistics, headings and description rows as console displays.
use crate::fonts::{DISPLAY_FAMILY_VAR, MONO_FAMILY_VAR};
use crate::tone::ToneExt;
use crate::variants::{HeadingVariant, StatVariant};
use kivact::interaction::Tone;
use kivact::style::color::{BORDER, SURFACE, TEXT_MUTED};
use kivact::style::{
    Align, Display, FlexDirection, FontStyle, Length, Style, Theme, Track, auto, font, pct, style,
};
use kivact::ui::theme::{
    DescriptionListTheme, HeadingState, HeadingTheme, Look, StatState, StatTheme,
};

/// Install the stat, heading, description list and sparkline recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            StatTheme::default()
                .root(stat_root)
                .label(stat_label)
                .value(stat_value)
                .delta(stat_delta)
                .chart(stat_chart),
        )
        .with(HeadingTheme::default().root(heading))
        .with(
            DescriptionListTheme::default()
                .root(description_root)
                .item(description_item)
                .term(description_term)
                .detail(description_detail),
        )
        .with(crate::sparkline::sparkline_table())
}

pub(crate) fn stat_root(t: &Theme, s: &Look<StatState>, base: Style) -> Style {
    let kpi = s.variant::<StatVariant>() == Some(StatVariant::Kpi);
    let tone = s.variant::<Tone>().unwrap_or_default();
    base.display(Display::Grid)
        .grid_template_columns((Track::fr(1), auto()))
        .padding_x(16)
        .padding_top(14)
        .padding_bottom(12)
        .gap(6)
        .background(SURFACE)
        .border(1, BORDER)
        .border_top(if kpi { 3 } else { 1 }, tone.color(t))
}

pub(crate) fn stat_label(t: &Theme, _: &Look<StatState>, _base: Style) -> Style {
    style()
        .grid_column((1, 3))
        .grid_row(1)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(11.5)
        .color(TEXT_MUTED)
}

pub(crate) fn stat_value(t: &Theme, _: &Look<StatState>, _base: Style) -> Style {
    style()
        .grid_column((1, 3))
        .grid_row(2)
        .font_family(t.get(&DISPLAY_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(36)
        .font_weight(700)
        .font_style(FontStyle::Italic)
        .line_height(1.)
}

pub(crate) fn stat_delta(t: &Theme, s: &Look<StatState>, _base: Style) -> Style {
    style()
        .grid_column(1)
        .grid_row(3)
        .align_self(Align::Center)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(11.5)
        .color(s.variant::<Tone>().unwrap_or_default().color(t))
}

pub(crate) fn stat_chart(_: &Theme, _: &Look<StatState>, _base: Style) -> Style {
    style().grid_column(2).grid_row(3).align_self(Align::Center)
}

pub(crate) fn heading(t: &Theme, s: &Look<HeadingState>, _base: Style) -> Style {
    let size: Length = if s.variant::<HeadingVariant>().is_some() {
        Length::from(24.)
    } else {
        font::SIZE_LARGE
    };
    style()
        .font_family(t.get(&DISPLAY_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(size)
        .line_height(1.)
        .letter_spacing(0.96)
        .font_weight(700)
        .font_style(FontStyle::Italic)
}

pub(crate) fn description_root(_: &Theme, _: &Look<()>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .border_top(1, BORDER)
}

pub(crate) fn description_item(_: &Theme, _: &Look<()>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .border_bottom(1, BORDER)
}

pub(crate) fn description_term(t: &Theme, _: &Look<()>, _base: Style) -> Style {
    style()
        .width(120)
        .padding_x(10)
        .padding_y(8)
        .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(11.5)
        .background(kivact::style::color::SURFACE_ALT)
}

pub(crate) fn description_detail(_: &Theme, _: &Look<()>, _base: Style) -> Style {
    style()
        .flex_grow(1.)
        .padding_x(10)
        .padding_y(8)
        .font_size(12.5)
}
