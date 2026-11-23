//! Role: Style Astryx Neutral's menu: the trigger, the top, submenu and context surfaces,
//! the item rows and their marks.
use crate::color as c;
use crate::metrics as m;
use crate::motion::{FAST, FAST_MIN};
use crate::radius as r;
use crate::recipe_parts::{anchored, body_text, ring, shadowed, supporting_text, tween};
use crate::shadows;
use crate::space as s;
use crate::type_scale as t;
use kivact::custom::PartState;
use kivact::interaction::{MenuItemState, PopupState, PressOver};
use kivact::style::{Align, FlexDirection, Style, Theme, ThemeMode, auto, style};
use kivact::ui::theme::{Look, MenuTheme};

const MENU_MIN_FLYOUT_WIDTH: f64 = 160.0;
const MENU_MAX_HEIGHT: f64 = 300.0;

/// The floating panel of a menu: the body text over the popover fill and the low shadow.
fn panel(theme: &Theme) -> Style {
    let dark = theme.mode() == ThemeMode::Dark;
    shadowed(body_text(style()), shadows::low(dark))
        .flex_direction(FlexDirection::Column)
        .background(c::BG_POPOVER)
        .border_radius(r::RADIUS_CONTAINER)
        .padding(s::SPACE_S1)
        .gap(s::SPACE_S0_5)
        .max_height(MENU_MAX_HEIGHT)
}

/// The top level's surface: it opens below its trigger.
fn surface(theme: &Theme, _look: &Look<PopupState>, _base: Style) -> Style {
    anchored(panel(theme), 0.0, -theme.get(&s::SPACE_S2_VAR))
}

/// The surface of a menu opened by a context gesture: at least the flyout width.
fn context_surface(theme: &Theme, _look: &Look<PopupState>, _base: Style) -> Style {
    anchored(
        panel(theme).min_width(MENU_MIN_FLYOUT_WIDTH),
        0.0,
        -theme.get(&s::SPACE_S2_VAR),
    )
}

/// A submenu's surface: it opens beside its item, at least the flyout width.
fn sub_surface(theme: &Theme, _look: &Look<PopupState>, _base: Style) -> Style {
    anchored(
        panel(theme).min_width(MENU_MIN_FLYOUT_WIDTH),
        -theme.get(&s::SPACE_S2_VAR),
        0.0,
    )
}

/// The menu's own trigger: a neutral field that darkens while the menu is open.
fn trigger(_theme: &Theme, look: &Look<PopupState>, _base: Style) -> Style {
    let (hover, fill) = if look.open {
        (c::OVERLAY_PRESSED, c::OVERLAY_PRESSED)
    } else {
        (c::OVERLAY_HOVER, c::NEUTRAL)
    };
    let base = body_text(style())
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(s::SPACE_S2)
        .min_height(m::CONTROL_MD)
        .padding_x(s::SPACE_S3)
        .font_weight(t::WEIGHT_MEDIUM)
        .border_radius(r::RADIUS_ELEMENT)
        .background(fill)
        .hover(style().background(hover))
        .active(style().background(c::OVERLAY_PRESSED))
        .focus_visible(ring(c::FOCUS_RING));
    tween(base, FAST)
}

/// An item row: its text, highlighted on hover and focus, and dimmed when disabled.
fn item(theme: &Theme, look: &Look<MenuItemState>, _base: Style) -> Style {
    let highlight = style().background(c::OVERLAY_HOVER);
    let base = body_text(style())
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .min_height(m::CONTROL_MD)
        .padding_x(s::SPACE_S2)
        .padding_y(s::SPACE_S1_5)
        .gap(s::SPACE_S2)
        .border_radius(theme.get(&r::RADIUS_CONTAINER_VAR) - theme.get(&s::SPACE_S1_VAR))
        .color(if look.destructive {
            c::TEXT_ERROR
        } else {
            c::TEXT_PRIMARY
        });
    let base = if look.input.is_disabled() {
        base.opacity(m::DISABLED_OPACITY)
    } else if look.pressed_over == PressOver::Over {
        base.background(c::OVERLAY_HOVER)
    } else if look.pressed_over == PressOver::Away {
        base
    } else {
        base.hover(highlight.clone())
            .focus(highlight.clone())
            .active(style().background(c::OVERLAY_PRESSED))
            .focus_visible(
                style()
                    .outline(m::FOCUS_RING_WIDTH, c::FOCUS_RING)
                    .outline_offset(-m::FOCUS_RING_WIDTH),
            )
    };
    let base = if look.is_expanded() {
        base.merge(&highlight)
    } else {
        base
    };
    tween(base, FAST_MIN)
}

/// The separator line between groups of items.
fn separator(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    style()
        .height(m::BORDER_WIDTH)
        .flex_shrink(0)
        .margin_y(s::SPACE_S1)
        .background(c::BORDER)
}

/// A heading of a menu or of a group: the supporting text.
fn label(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    supporting_text(style())
        .padding_x(s::SPACE_S2)
        .padding_y(s::SPACE_S1)
}

/// The secondary line of an item: the supporting text, in the error color when destructive.
fn description(_theme: &Theme, look: &Look<MenuItemState>, _base: Style) -> Style {
    supporting_text(style()).color(if look.destructive {
        c::TEXT_ERROR
    } else {
        c::TEXT_SECONDARY
    })
}

/// The keyboard shortcut at the item's end.
fn shortcut(_theme: &Theme, look: &Look<MenuItemState>, _base: Style) -> Style {
    supporting_text(style())
        .margin_left(auto())
        .flex_shrink(0)
        .color(if look.destructive {
            c::TEXT_ERROR
        } else {
            c::TEXT_SECONDARY
        })
}

/// The icon box of a check, radio or submenu mark.
fn mark(_theme: &Theme, look: &Look<MenuItemState>, _base: Style) -> Style {
    style()
        .size(m::ICON_SM, m::ICON_SM)
        .flex_shrink(0)
        .color(if look.destructive {
            c::TEXT_ERROR
        } else {
            c::ICON_SECONDARY
        })
}

/// The empty box before the content of an inset item: as wide as a mark.
fn inset(_theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    style().width(m::ICON_SM).flex_shrink(0)
}

/// Install the kit's recipes of the menu.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        MenuTheme::default()
            .trigger(trigger)
            .surface(surface)
            .sub_surface(sub_surface)
            .context_surface(context_surface)
            .item(item)
            .separator(separator)
            .label(label)
            .description(description)
            .shortcut(shortcut)
            .check(mark)
            .submenu(mark)
            .inset(inset),
    )
}
