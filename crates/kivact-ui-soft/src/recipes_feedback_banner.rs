//! Role: Style Astryx Neutral's standard alerts: the banner surface, its header, its
//! content and its controls.
use crate::color as c;
use crate::radius as r;
use crate::recipe_parts::ring;
use crate::shadows::elevate;
use crate::type_scale as t;
use crate::variants::{BannerVariant, Elevation};
use kivact::draw::{Path, Stroke};
use kivact::geometry::Point;
use kivact::interaction::Tone;
use kivact::style::{
    Align, Color, FlexDirection, Justify, Length, Style, Theme, ThemeMode, pct, style,
};
use kivact::ui::theme::{AlertState, AlertTheme, Expansion, Look};
use kivact::view::{ReadingDirection, use_direction};

/// The muted surface and its ink for an alert of `tone`.
fn colors(tone: Tone) -> (Color, Color) {
    match tone {
        Tone::Warning => (c::WARNING_MUTED, c::ON_WARNING_MUTED),
        Tone::Danger => (c::ERROR_MUTED, c::ON_ERROR_MUTED),
        Tone::Success => (c::SUCCESS_MUTED, c::ON_SUCCESS_MUTED),
        _ => (c::INFO_MUTED, c::ON_INFO_MUTED),
    }
}

/// The corner radius of an alert: rounded as a card, square as a section.
fn radius(look: &Look<AlertState>) -> Length {
    if look.variant::<BannerVariant>().unwrap_or_default() == BannerVariant::Card {
        r::RADIUS_CONTAINER
    } else {
        r::RADIUS_NONE
    }
}

/// The row direction of the alert's parts: reversed in RTL.
fn direction() -> FlexDirection {
    if use_direction() == ReadingDirection::Rtl {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    }
}

/// The root: a column with no padding or border, its surface shaped by the elevation.
fn root(theme: &Theme, look: &Look<AlertState>, _base: Style) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Column)
        .gap(0.0)
        .padding(0.0)
        .border(0.0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .border_radius(radius(look))
        .min_width(0.0);
    elevate(
        base,
        look.variant::<Elevation>().unwrap_or_default(),
        theme.mode() == ThemeMode::Dark,
    )
}

/// The header: the icon, the title and the controls on one wrapping row, in the tone's surface.
fn header(_theme: &Theme, look: &Look<AlertState>, _base: Style) -> Style {
    let (bg, ink) = colors(look.tone);
    let radius = radius(look);
    let bottom = if look.has_content() && look.expansion() != Expansion::Collapsed {
        r::RADIUS_NONE
    } else {
        radius
    };
    style()
        .flex_direction(direction())
        .flex_wrap(true)
        .column_gap(8.0)
        .row_gap(12.0)
        .padding_y(12.0)
        .padding_x(16.0)
        .align_items(
            if (look.has_actions
                || look.dismissible
                || look.expansion() != Expansion::NotExpandable)
                && !look.has_description
            {
                Align::Center
            } else {
                Align::Start
            },
        )
        .border_radius_corners(radius, radius, bottom, bottom)
        .background(bg)
        .color(ink)
}

/// The message: the title and the description, which grows and gives way to the actions.
fn message(_theme: &Theme, look: &Look<AlertState>, _base: Style) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .min_width(0.0);
    if look.has_actions {
        base.flex_basis(128.0)
    } else {
        base
    }
}

/// The controls: the dismiss and the toggle, at the end of the header.
fn controls(_theme: &Theme, _look: &Look<AlertState>, _base: Style) -> Style {
    style()
        .flex_direction(direction())
        .flex_wrap(true)
        .align_items(Align::Center)
        .justify_content(Justify::End)
        .gap(8.0)
        .flex_shrink(0.0)
        .max_width(pct(100))
        .margin_y(-4.0)
}

/// The title: the semibold label of the alert.
fn title(_theme: &Theme, _look: &Look<AlertState>, _base: Style) -> Style {
    style()
        .font_size(t::SIZE_MD)
        .line_height(20.0 / t::SIZE_MD)
        .font_weight(t::WEIGHT_SEMIBOLD)
}

/// The description: the small body text of the alert.
fn description(_theme: &Theme, _look: &Look<AlertState>, _base: Style) -> Style {
    style()
        .font_size(t::SIZE_SM)
        .line_height(20.0 / t::SIZE_SM)
        .font_weight(t::WEIGHT_NORMAL)
}

/// The icon: a 20 px box in the header.
fn icon(_theme: &Theme, _look: &Look<AlertState>, _base: Style) -> Style {
    style()
        .size(20.0, 20.0)
        .flex_shrink(0.0)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

/// The actions: a row of buttons in the text color.
fn actions(_theme: &Theme, _look: &Look<AlertState>, _base: Style) -> Style {
    style()
        .flex_direction(direction())
        .gap(8.0)
        .align_items(Align::Center)
        .color(c::TEXT_PRIMARY)
}

/// The content: the body under the header, framed by the alert's border.
fn content(_theme: &Theme, look: &Look<AlertState>, _base: Style) -> Style {
    let radius = radius(look);
    style()
        .flex_direction(FlexDirection::Column)
        .padding_y(12.0)
        .padding_x(16.0)
        .background(c::BG_CARD)
        .border_radius_corners(r::RADIUS_NONE, r::RADIUS_NONE, radius, radius)
        .border_left(1.0, c::BORDER)
        .border_right(1.0, c::BORDER)
        .border_bottom(1.0, c::BORDER)
}

/// The close or the expand button: a 24 px box that paints its `x` or chevron glyph.
fn control(look: &Look<AlertState>, dismiss: bool) -> Style {
    let (_, ink) = colors(look.tone);
    let expanded = look.expansion() == Expansion::Expanded;
    style()
        .size(24.0, 24.0)
        .padding(0.0)
        .font_size(0.0)
        .border(0.0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .focus_visible(ring(c::ACCENT))
        .paint(move |draw, _| {
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

/// The dismiss button of an alert.
fn dismiss(_theme: &Theme, look: &Look<AlertState>, _base: Style) -> Style {
    control(look, true)
}

/// The expand and collapse button of an alert with content.
fn toggle(_theme: &Theme, look: &Look<AlertState>, _base: Style) -> Style {
    control(look, false)
}

/// Install the kit's recipes of the alert.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        AlertTheme::default()
            .root(root)
            .header(header)
            .message(message)
            .controls(controls)
            .title(title)
            .description(description)
            .icon(icon)
            .actions(actions)
            .content(content)
            .dismiss(dismiss)
            .toggle(toggle),
    )
}
