//! Role: Install Astryx Neutral's recipes of the navigation group: the navigation menu's bar, triggers, panels and links.
use crate::color as c;
use crate::metrics as m;
use crate::motion::FAST_MAX;
use crate::radius as r;
use crate::recipe_parts::{ring, shadowed, text, tween};
use crate::shadows;
use crate::space as s;
use crate::type_scale as t;
use kivact::interaction::NavOpen;
use kivact::style::{Align, Color, FlexDirection, Justify, Style, Theme, ThemeMode, style};
use kivact::ui::theme::{Look, NavigationLinkState, NavigationMenuTheme, NavigationTriggerState};
use kivact::view::{ReadingDirection, use_direction};

/// The flow of a row of navigation parts: reversed in a right-to-left reading direction.
fn flow() -> FlexDirection {
    if use_direction() == ReadingDirection::Rtl {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    }
}

/// The bar of the navigation menu: its items in a centered row.
fn root(_theme: &Theme, _s: &Look<NavOpen>, _base: Style) -> Style {
    style()
        .flex_direction(flow())
        .align_items(Align::Center)
        .gap(4)
        .padding(0)
}

/// One item of the bar: the label in the label role, with the hover and pressed fills, and the open
/// fill when it is open.
fn item(open: bool) -> Style {
    let fill = if open {
        c::OVERLAY_HOVER
    } else {
        Color::TRANSPARENT
    };
    style()
        .flex_direction(flow())
        .align_items(Align::Center)
        .gap(8)
        .padding_x(12)
        .padding_y(6)
        .height(kivact::style::auto())
        .min_height(0)
        .min_width(0)
        .border(0.0, Color::TRANSPARENT)
        .border_radius(r::RADIUS_ELEMENT)
        .font_size(t::LABEL.size_px)
        .line_height(t::LABEL.line_height_px / t::LABEL.size_px)
        .font_weight(t::LABEL.weight)
        .color(if open {
            c::TEXT_PRIMARY
        } else {
            c::TEXT_SECONDARY
        })
        .background(fill)
        .hover(style().background(c::OVERLAY_HOVER))
        .active(style().background(c::OVERLAY_PRESSED))
        .disabled(style().opacity(m::DISABLED_OPACITY))
        .focus_visible(ring(c::ACCENT))
}

/// The action item of the bar: the item in its closed state.
fn action(_theme: &Theme, _s: &Look<NavOpen>, _base: Style) -> Style {
    item(false)
}

/// The trigger of a menu item: the item with room for a chevron, which turns when the menu is open.
fn trigger(_theme: &Theme, s: &Look<NavigationTriggerState>, _base: Style) -> Style {
    let open = s.open;
    let rtl = use_direction() == ReadingDirection::Rtl;
    let base = item(open);
    let base = if rtl {
        base.padding_left(36)
    } else {
        base.padding_right(36)
    };
    base.paint(move |draw, size| {
        use kivact::draw::{Path, Stroke};
        use kivact::geometry::Point;
        let x = if rtl { 20.0 } else { size.width - 20.0 };
        let y = size.height / 2.0;
        let flip = if open { -1.0 } else { 1.0 };
        draw.stroke_path(
            &Path::new()
                .move_to(Point::new(x - 4.0, y - 2.0 * flip))
                .line_to(Point::new(x, y + 2.0 * flip))
                .line_to(Point::new(x + 4.0, y - 2.0 * flip)),
            &Stroke::new(1.5),
            c::TEXT_SECONDARY,
        );
    })
}

/// The panel of a menu item: a popover column that fades and slides in, on the low shadow.
fn panel(theme: &Theme, _s: &Look<NavOpen>, _base: Style) -> Style {
    let hidden = style()
        .opacity(0.0)
        .translate_y(-theme.get(&s::SPACE_S2_VAR))
        .scale_xy(0.95, 0.95);
    let base = style()
        .flex_direction(FlexDirection::Column)
        .padding(s::SPACE_S1)
        .gap(s::SPACE_S1)
        .min_width(280)
        .background(c::BG_POPOVER)
        .border_radius(r::RADIUS_CONTAINER)
        .border(0.0, Color::TRANSPARENT)
        .enter(hidden);
    shadowed(
        tween(base, FAST_MAX),
        shadows::low(theme.mode() == ThemeMode::Dark),
    )
}

/// A link in a panel: the row of its icon, title and description, with the hover fill.
fn link(_theme: &Theme, look: &Look<NavigationLinkState>, _base: Style) -> Style {
    let hovered = !look.input.is_disabled() && (look.input.is_hovered() || look.input.is_focused());
    let fill = if hovered {
        c::OVERLAY_HOVER
    } else {
        Color::TRANSPARENT
    };
    style()
        .flex_direction(flow())
        .align_items(if look.has_description {
            Align::Start
        } else {
            Align::Center
        })
        .gap(s::SPACE_S3)
        .padding(s::SPACE_S3)
        .min_width(0)
        .border_radius(r::RADIUS_ELEMENT)
        .background(fill)
        .disabled(style().opacity(m::DISABLED_OPACITY))
        .focus_visible(ring(c::ACCENT))
}

/// A link's title: the body text in the semibold weight.
fn link_title(_theme: &Theme, _s: &Look<NavigationLinkState>, _base: Style) -> Style {
    text(style(), t::BODY, c::TEXT_PRIMARY).font_weight(t::WEIGHT_SEMIBOLD)
}

/// A link's description: the supporting text.
fn link_description(_theme: &Theme, _s: &Look<NavigationLinkState>, _base: Style) -> Style {
    text(style(), t::SUPPORTING, c::TEXT_SECONDARY)
}

/// A link's icon: a neutral square tile with the secondary icon color.
fn link_icon(_theme: &Theme, _s: &Look<NavigationLinkState>, _base: Style) -> Style {
    style()
        .size(40.0, 40.0)
        .flex_shrink(0.0)
        .border_radius(r::RADIUS_ELEMENT)
        .background(c::NEUTRAL)
        .color(c::ICON_SECONDARY)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

/// Install the kit's recipes of the navigation group.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme.with(
        NavigationMenuTheme::default()
            .root(root)
            .action(action)
            .trigger(trigger)
            .panel(panel)
            .link(link)
            .link_title(link_title)
            .link_description(link_description)
            .link_icon(link_icon),
    );
    crate::recipes_navigation_pagination::install(theme)
}
