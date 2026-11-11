//! Role: Style the standard dropdown, context and submenu parts.
use crate::Tokens;
use crate::recipes_dialog::{anchored, body_text, focus, motion, shadows, supporting_text};
use kivact::custom::PartState;
use kivact::interaction::MenuItemState;
use kivact::interaction::PopupState;
use kivact::interaction::PressOver;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Style;
use kivact::style::Theme;
use kivact::style::auto;
use kivact::style::style;
use kivact::ui::Look;
use kivact::ui::MenuRecipes;

const MENU_MIN_FLYOUT_WIDTH: f64 = 160.0;
const MENU_MAX_HEIGHT: f64 = 300.0;

pub(crate) fn menu(r: MenuRecipes) -> MenuRecipes {
    r.trigger(|_| trigger)
        .surface(|_| {
            |t: &Theme, _: &Look<PopupState>| {
                let p = t.ext::<Tokens>();
                anchored(surface(&p), &p, 0.0, -p.spacing.s2)
            }
        })
        .context_surface(|_| {
            |t: &Theme, _: &Look<PopupState>| {
                let p = t.ext::<Tokens>();
                anchored(
                    surface(&p).min_width(MENU_MIN_FLYOUT_WIDTH),
                    &p,
                    0.0,
                    -p.spacing.s2,
                )
            }
        })
        .sub_surface(|_| {
            |t: &Theme, _: &Look<PopupState>| {
                let p = t.ext::<Tokens>();
                anchored(
                    surface(&p).min_width(MENU_MIN_FLYOUT_WIDTH),
                    &p,
                    -p.spacing.s2,
                    0.0,
                )
            }
        })
        .item(|_| item)
        .separator(|_| {
            |t: &Theme, _: &Look<PartState>| {
                let p = t.ext::<Tokens>();
                style()
                    .height(p.sizes.border_width)
                    .shrink(0)
                    .margin_y(p.spacing.s1)
                    .background(p.colors.border)
            }
        })
        .label(|_| {
            |t: &Theme, _: &Look<PartState>| {
                let p = t.ext::<Tokens>();
                supporting_text(&p)
                    .padding_x(p.spacing.s2)
                    .padding_y(p.spacing.s1)
            }
        })
        .description(|_| {
            |t: &Theme, s: &Look<MenuItemState>| {
                let p = t.ext::<Tokens>();
                supporting_text(&p).color(if s.destructive {
                    p.colors.text_error
                } else {
                    p.colors.text_secondary
                })
            }
        })
        .shortcut(|_| {
            |t: &Theme, s: &Look<MenuItemState>| {
                let p = t.ext::<Tokens>();
                supporting_text(&p)
                    .margin_left(auto())
                    .shrink(0)
                    .color(if s.destructive {
                        p.colors.text_error
                    } else {
                        p.colors.text_secondary
                    })
            }
        })
        .check(|_| indicator)
        .submenu(|_| indicator)
        .inset(|_| {
            |t: &Theme, _: &Look<PartState>| {
                style().width(t.ext::<Tokens>().sizes.icon_sm).shrink(0)
            }
        })
}

fn surface(p: &Tokens) -> Style {
    shadows(body_text(p), &p.shadows.low)
        .direction(Direction::Column)
        .background(p.colors.bg_popover)
        .radius(p.radius.container)
        .padding(p.spacing.s1)
        .gap(p.spacing.s0_5)
        .max_height(MENU_MAX_HEIGHT)
}

fn trigger(t: &Theme, s: &Look<PopupState>) -> Style {
    let p = t.ext::<Tokens>();
    let hover = if s.open {
        p.colors.overlay_pressed
    } else {
        p.colors.overlay_hover
    };
    let fill = if s.open {
        p.colors.overlay_pressed
    } else {
        p.colors.neutral
    };
    let base = body_text(&p)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(p.spacing.s2)
        .min_height(p.sizes.control_md)
        .padding_x(p.spacing.s3)
        .weight(p.typography.weight_medium)
        .radius(p.radius.element)
        .background(fill)
        .hover(style().background(hover))
        .pressed(style().background(p.colors.overlay_pressed))
        .focus_visible(focus(&p));
    motion(base, &p, p.motion.fast)
}

fn item(t: &Theme, s: &Look<MenuItemState>) -> Style {
    let p = t.ext::<Tokens>();
    let highlight = style().background(p.colors.overlay_hover);
    let base = body_text(&p)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_height(p.sizes.control_md)
        .padding_x(p.spacing.s2)
        .padding_y(p.spacing.s1_5)
        .gap(p.spacing.s2)
        .radius(p.radius.container - p.spacing.s1)
        .color(if s.destructive {
            p.colors.text_error
        } else {
            p.colors.text_primary
        });
    let base = if s.input.is_disabled() {
        base.opacity(p.sizes.disabled_opacity)
    } else if s.pressed_over == PressOver::Over {
        base.background(p.colors.overlay_hover)
    } else if s.pressed_over == PressOver::Away {
        base
    } else {
        base.hover(highlight.clone())
            .focus(highlight.clone())
            .pressed(style().background(p.colors.overlay_pressed))
            .focus_visible(style().ring_with_offset(
                p.sizes.focus_ring_width,
                -p.sizes.focus_ring_width,
                p.colors.focus_ring,
            ))
    };
    let base = if s.is_expanded() {
        base.merge(&highlight)
    } else {
        base
    };
    motion(base, &p, p.motion.fast_min)
}

fn indicator(t: &Theme, s: &Look<MenuItemState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .size(p.sizes.icon_sm, p.sizes.icon_sm)
        .shrink(0)
        .color(if s.destructive {
            p.colors.text_error
        } else {
            p.colors.icon_secondary
        })
}
