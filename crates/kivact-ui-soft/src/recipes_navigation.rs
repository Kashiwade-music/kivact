//! Role: Install Astryx Neutral's recipes of the navigation group: the navigation menu's bar, triggers, panels and links.
use crate::color as c;
use crate::metrics as m;
use crate::motion::FAST_MAX;
use crate::motion::{EASE_STANDARD, FAST};
use crate::radius as r;
use crate::recipe_parts::{ring, shadowed, text, tween};
use crate::shadows;
use crate::space as s;
use crate::type_scale as t;
use crate::variants::BreadcrumbsVariant;
use kivact::interaction::{NavOpen, TabListState, TabState};
use kivact::motion::Transition;
use kivact::style::{
    Align, Color, Cursor, FlexDirection, Justify, Position, Style, TextDecorationLine, Theme,
    ThemeMode, TransitionProp, style,
};
use kivact::ui::theme::{
    BreadcrumbItemState, BreadcrumbTheme, Look, NavigationLinkState, NavigationMenuTheme,
    NavigationTriggerState, TabsTheme,
};
use kivact::view::{Orientation, ReadingDirection, use_direction};

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

/// The breadcrumb trail: a wrapping row in the body role, or the supporting role when the variant asks.
fn breadcrumb_list(_theme: &Theme, s: &Look<()>, _base: Style) -> Style {
    let font = if s.variant::<BreadcrumbsVariant>().unwrap_or_default()
        == BreadcrumbsVariant::Supporting
    {
        t::SUPPORTING
    } else {
        t::BODY
    };
    style()
        .flex_direction(flow())
        .flex_wrap(true)
        .align_items(Align::Center)
        .gap(s::SPACE_S1)
        .font_size(font.size_px)
        .line_height(font.line_height_px / font.size_px)
        .color(c::TEXT_SECONDARY)
}

/// One breadcrumb: semibold for the current page, a pointer and an underline on hover for a link.
fn breadcrumb_item(_theme: &Theme, s: &Look<BreadcrumbItemState>, _base: Style) -> Style {
    let supporting = s.variant::<BreadcrumbsVariant>() == Some(BreadcrumbsVariant::Supporting);
    let base = style()
        .flex_direction(flow())
        .align_items(Align::Center)
        .gap(s::SPACE_S1)
        .padding(0)
        .height(kivact::style::auto())
        .min_width(0)
        .border(0.0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .color(if s.input().is_none() && !supporting {
            c::TEXT_PRIMARY
        } else {
            c::TEXT_SECONDARY
        });
    if s.input().is_none() {
        base.font_weight(t::WEIGHT_SEMIBOLD)
    } else {
        let base = if s.input().is_some_and(|input| input.is_hovered()) {
            base.text_decoration(TextDecorationLine::UNDERLINE)
        } else {
            base
        };
        base.padding_y(s::SPACE_S1)
            .cursor(Cursor::Pointer)
            .focus_visible(ring(c::ACCENT))
    }
}

/// The separator between breadcrumbs: the chevron, mirrored in a right-to-left reading direction.
fn breadcrumb_separator(_theme: &Theme, _s: &Look<()>, _base: Style) -> Style {
    style().scale_xy(
        if use_direction() == ReadingDirection::Rtl {
            -1.0
        } else {
            1.0
        },
        1.0,
    )
}

/// The list of tabs: its tabs in a row, or a column when the list is vertical, with the spacing step between them.
fn tabs_list(_: &Theme, look: &Look<TabListState>, _base: Style) -> Style {
    style()
        .flex_direction(if look.orientation == Orientation::Vertical {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        })
        .gap(s::SPACE_S0_5)
        .min_width(0)
}

/// A tab: its label in the label role, centered, with the hover and pressed fills unless it is disabled.
fn tab(_: &Theme, look: &Look<TabState>, _base: Style) -> Style {
    let ink = if look.selected {
        c::TEXT_PRIMARY
    } else {
        c::TEXT_SECONDARY
    };
    let tab = text(style(), t::LABEL, ink)
        .position(Position::Relative)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .height(m::CONTROL_MD)
        .flex_shrink(0.0)
        .padding_x(s::SPACE_S3)
        .gap(s::SPACE_S1)
        .border_radius(r::RADIUS_ELEMENT)
        .font_weight(if look.selected {
            t::WEIGHT_SEMIBOLD
        } else {
            t::WEIGHT_NORMAL
        })
        .single_line()
        .focus_visible(ring(c::FOCUS_RING))
        .transition(
            TransitionProp::All,
            Transition::tween(FAST as f64, EASE_STANDARD),
        );
    if look.input.is_disabled() {
        tab
    } else {
        tab.hover(style().background(c::OVERLAY_HOVER))
            .active(style().background(c::OVERLAY_PRESSED))
    }
}

/// The indicator of a tab: a bar on the tab's edge, faded in when the tab is selected.
fn tab_indicator(_: &Theme, look: &Look<TabState>, _base: Style) -> Style {
    let inset = s::SPACE_S3;
    let indicator = style()
        .position(Position::Absolute)
        .border_radius(1.0)
        .background(c::ACCENT)
        .opacity(if look.selected { 1.0 } else { 0.0 })
        .transition(
            TransitionProp::All,
            Transition::tween(FAST as f64, EASE_STANDARD).essential(),
        );
    if look.orientation == Orientation::Vertical {
        let indicator = indicator.top(inset).bottom(inset).width(2);
        if use_direction() == ReadingDirection::Rtl {
            indicator.right(0)
        } else {
            indicator.left(0)
        }
    } else {
        indicator.left(inset).right(inset).bottom(-1).height(2)
    }
}

/// Install the kit's recipes of the navigation group.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme.with(
        TabsTheme::default()
            .list(tabs_list)
            .tab(tab)
            .indicator(tab_indicator),
    );
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
    let theme = theme.with(
        BreadcrumbTheme::default()
            .root(breadcrumb_root)
            .list(breadcrumb_list)
            .item(breadcrumb_item)
            .separator(breadcrumb_separator),
    );
    let theme = crate::recipes_navigation_pagination::install(theme);
    crate::recipes_navigation_stepper::install(theme)
}

/// A breadcrumb trail's root: it shrinks to its content.
fn breadcrumb_root(_theme: &Theme, _s: &Look<()>, _base: Style) -> Style {
    style().min_width(0)
}
