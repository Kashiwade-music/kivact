//! Role: Style canonical persistent alerts with Neutral banner surfaces.
use crate::Elevation;
use kivact::interaction::Tone;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Color, Direction, Justify, Style, Theme, pct, style};
use kivact::ui::{AlertRecipes, AlertState, Expansion, Look};
/// The visual container treatment of a standard alert.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BannerContainer {
    /// Rounded card surface, the default.
    #[default]
    Card,
    /// Section surface without the card radius.
    Section,
}
impl kivact::ui::Variant for BannerContainer {}
/// Independent reactive surface and elevation choices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BannerAppearance {
    /// Read-only reactive container treatment.
    pub container: Value<BannerContainer>,
    /// Read-only reactive surface elevation.
    pub elevation: Value<Elevation>,
}
impl BannerAppearance {
    /// Combine independent container and elevation inputs for a canonical alert.
    pub fn new(
        container: impl IntoValue<BannerContainer>,
        elevation: impl IntoValue<Elevation>,
    ) -> Self {
        Self {
            container: container.into_value(),
            elevation: elevation.into_value(),
        }
    }
}
impl kivact::ui::Variant for BannerAppearance {}
fn container(s: &Look<AlertState>) -> BannerContainer {
    s.variant::<BannerAppearance>().map_or_else(
        || s.variant::<BannerContainer>().copied().unwrap_or_default(),
        |v| v.container.get(),
    )
}
fn radius(t: &crate::Tokens, s: &Look<AlertState>) -> f64 {
    if container(s) == BannerContainer::Card {
        t.radius.container
    } else {
        0.0
    }
}
fn colors(t: &crate::Tokens, tone: Tone) -> (Color, Color) {
    let c = t.colors;
    match tone {
        Tone::Warning => (c.warning_muted, c.on_warning_muted),
        Tone::Danger => (c.error_muted, c.on_error_muted),
        Tone::Success => (c.success_muted, c.on_success_muted),
        _ => (c.info_muted, c.on_info_muted),
    }
}
fn direction() -> Direction {
    if crate::stack::rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    }
}
pub(crate) fn alert(r: AlertRecipes) -> AlertRecipes {
    r.root(|_| root)
        .header(|_| header)
        .message(|_| message)
        .controls(|_| controls)
        .title(|_| {
            |_: &Theme, _: &Look<AlertState>| {
                style().font_size(14).line_height(20.0 / 14.0).weight(600)
            }
        })
        .description(|_| {
            |_: &Theme, _: &Look<AlertState>| {
                style().font_size(12).line_height(20.0 / 12.0).weight(400)
            }
        })
        .icon(|_| {
            |_: &Theme, _: &Look<AlertState>| {
                style()
                    .size(20, 20)
                    .shrink(0)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
            }
        })
        .actions(|_| {
            |t: &Theme, _: &Look<AlertState>| {
                style()
                    .direction(direction())
                    .gap(8)
                    .align_items(Align::Center)
                    .color(t.ext::<crate::Tokens>().colors.text_primary)
            }
        })
        .content(|_| content)
        .dismiss(|_| |t: &Theme, s: &Look<AlertState>| control(t, s, true))
        .toggle(|_| |t: &Theme, s: &Look<AlertState>| control(t, s, false))
}
fn root(theme: &Theme, s: &Look<AlertState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let base = style()
        .direction(Direction::Column)
        .gap(0)
        .padding(0)
        .border(0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .radius(radius(&t, s))
        .min_width(0);
    s.variant::<BannerAppearance>()
        .map_or(Elevation::None, |v| v.elevation.get())
        .apply(base, &t)
}
fn header(theme: &Theme, s: &Look<AlertState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let (bg, ink) = colors(&t, s.tone);
    let radius = radius(&t, s);
    let bottom = if s.has_content() && s.expansion() != Expansion::Collapsed {
        0.0
    } else {
        radius
    };
    style()
        .direction(direction())
        .wrap(true)
        .column_gap(8)
        .row_gap(12)
        .padding_y(12)
        .padding_x(16)
        .align_items(
            if (s.has_actions || s.dismissible || s.expansion() != Expansion::NotExpandable)
                && !s.has_description
            {
                Align::Center
            } else {
                Align::Start
            },
        )
        .radius_corners(radius, radius, bottom, bottom)
        .background(bg)
        .color(ink)
}
fn message(_: &Theme, s: &Look<AlertState>) -> Style {
    let base = style().direction(Direction::Column).grow(1).min_width(0);
    if s.has_actions { base.basis(128) } else { base }
}
fn controls(_: &Theme, _: &Look<AlertState>) -> Style {
    style()
        .direction(direction())
        .wrap(true)
        .align_items(Align::Center)
        .justify_content(Justify::End)
        .gap(8)
        .shrink(0)
        .max_width(pct(100))
        .margin_y(-4)
}
fn content(theme: &Theme, s: &Look<AlertState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let radius = radius(&t, s);
    style()
        .direction(Direction::Column)
        .padding_y(12)
        .padding_x(16)
        .background(t.colors.bg_card)
        .radius_corners(0, 0, radius, radius)
        .border_left(1, t.colors.border)
        .border_right(1, t.colors.border)
        .border_bottom(1, t.colors.border)
}
fn control(theme: &Theme, s: &Look<AlertState>, dismiss: bool) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let (_, ink) = colors(&t, s.tone);
    let expanded = s.expansion() == Expansion::Expanded;
    style()
        .size(24, 24)
        .padding(0)
        .font_size(0)
        .border(0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
        .paint(move |draw, _| {
            use kivact::draw::{Path, Stroke};
            use kivact::geometry::Point;
            let path = if dismiss {
                Path::new()
                    .move_to(Point::new(8.0, 8.0))
                    .line_to(Point::new(16.0, 16.0))
                    .move_to(Point::new(16.0, 8.0))
                    .line_to(Point::new(8.0, 16.0))
            } else if expanded {
                Path::new()
                    .move_to(Point::new(8.0, 14.0))
                    .line_to(Point::new(12.0, 10.0))
                    .line_to(Point::new(16.0, 14.0))
            } else {
                Path::new()
                    .move_to(Point::new(8.0, 10.0))
                    .line_to(Point::new(12.0, 14.0))
                    .line_to(Point::new(16.0, 10.0))
            };
            draw.stroke_path(&path, &Stroke::new(1.5), ink);
        })
}
/// Decorative tone icon for a standard alert's explicit icon slot.
pub fn banner_icon(tone: Tone) -> impl kivact::view::IntoView {
    crate::icon_view(match tone {
        Tone::Warning => "triangle-alert",
        Tone::Danger => "circle-x",
        Tone::Success => "circle-check",
        _ => "info",
    })
}
kivact::state::constant_value!(BannerContainer);
