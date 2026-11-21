//! Role: The Pop looks of the metrics: headings, stat cards, trend charts and term-detail rows.

use crate::color;
use crate::variants::Hue;
use kivact::style::Shadow;
use kivact::style::color as core;
use kivact::style::{Align, Display, FlexDirection, Style, Theme, Track, pct, style};
use kivact::ui::theme::{
    DescriptionListTheme, HeadingState, HeadingTheme, Look, SparklineState, SparklineTheme,
    StatState, StatTheme,
};
use kivact::ui::{SparklineFill, Trend};

/// Install the kit's recipes of the metrics.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(HeadingTheme::default().root(heading))
        .with(
            StatTheme::default()
                .root(stat_root)
                .icon(stat_icon)
                .label(stat_label)
                .value(stat_value)
                .delta(stat_delta)
                .chart(stat_chart),
        )
        .with(
            SparklineTheme::default()
                .root(sparkline)
                .round(true)
                .fill(SparklineFill::Fade(color::SPARKLINE_FILL)),
        );
    theme.with(
        DescriptionListTheme::default()
            .root(description_root)
            .item(description_item)
            .term(description_term)
            .detail(description_detail),
    )
}

fn heading(_theme: &Theme, s: &Look<HeadingState>, _base: Style) -> Style {
    style()
        .font_size(match s.level {
            1 => 28,
            2 => 20,
            3 => 18,
            4 => 16,
            _ => 14,
        })
        .font_weight(600)
        .line_height(1.2)
        .color(core::TEXT)
}

/// A stat card: a two-column grid on a panel surface, 185 px tall at least.
fn stat_root(_theme: &Theme, _: &Look<StatState>, _base: Style) -> Style {
    style()
        .display(Display::Grid)
        .grid_template_columns((Track::fr(1), Track::fr(1)))
        .gap(4)
        .padding_top(20)
        .min_height(185)
        .clip(true)
        .background(color::SURFACE)
        .border(1, color::LINE)
        .border_radius(crate::size::PANEL_RADIUS)
        .shadow(Shadow::new(0, 1, 3, core::SHADOW))
}

fn stat_icon(_theme: &Theme, _: &Look<StatState>, _base: Style) -> Style {
    style()
        .grid_column(1)
        .grid_row(1)
        .margin_left(20)
        .margin_bottom(8)
        .size(32, 32)
        .border_radius(8)
        .background(Hue::Blue.colors().0)
        .color(color::TEXT_MUTED)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(kivact::style::Justify::Center)
}

fn stat_label(_theme: &Theme, _: &Look<StatState>, _base: Style) -> Style {
    style()
        .grid_column((1, 3))
        .grid_row(2)
        .margin_x(20)
        .font_size(13)
        .font_weight(500)
        .color(core::TEXT_MUTED)
}

fn stat_value(_theme: &Theme, _: &Look<StatState>, _base: Style) -> Style {
    style()
        .grid_column((1, 3))
        .grid_row(3)
        .margin_x(20)
        .font_size(26)
        .font_weight(600)
        .line_height(1.2)
}

/// The delta badge: the hue of the variant, or the trend's hue unless one is given.
fn stat_delta(_theme: &Theme, s: &Look<StatState>, _base: Style) -> Style {
    let hue = s.variant::<Hue>().unwrap_or(match s.trend {
        Trend::Up => Hue::Mint,
        Trend::Down => Hue::Red,
        Trend::Neutral => Hue::Gray,
    });
    style()
        .grid_column(2)
        .grid_row(1)
        .margin_right(20)
        .align_self(Align::Center)
        .justify_self(Align::End)
        .font_size(11)
        .padding_x(6)
        .padding_y(1)
        .border_radius(4)
        .border(1, core::BORDER)
        .background(core::SURFACE)
        .color(hue.colors().1)
}

fn stat_chart(_theme: &Theme, _: &Look<StatState>, _base: Style) -> Style {
    style()
        .grid_column((1, 3))
        .grid_row(4)
        .margin_top(8)
        .min_width(1)
}

/// The trend chart: the full width of its slot and a 52 px height.
fn sparkline(_theme: &Theme, _: &Look<SparklineState>, _base: Style) -> Style {
    style().height(52).width(pct(100)).min_width(1)
}

fn description_root(_theme: &Theme, _: &Look<()>, base: Style) -> Style {
    base.gap(0)
}

fn description_item(_theme: &Theme, _: &Look<()>, base: Style) -> Style {
    base.gap(12)
        .align_items(Align::Center)
        .min_height(44)
        .border_bottom(1, color::LINE)
}

fn description_term(_theme: &Theme, _: &Look<()>, _base: Style) -> Style {
    style().width(110).font_size(13).color(color::TEXT_MUTED)
}

fn description_detail(_theme: &Theme, _: &Look<()>, base: Style) -> Style {
    base.flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
}
