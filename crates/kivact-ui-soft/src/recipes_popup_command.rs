//! Role: Style the standard command palette and its native search field.
use crate::Tokens;
use crate::recipes_dialog::{body_text, modal, scrim, shadows, supporting_text};
use kivact::custom::PartState;
use kivact::interaction::ListItemState;
use kivact::interaction::PopupState;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Direction;
use kivact::style::Overflow;
use kivact::style::Style;
use kivact::style::TextAlign;
use kivact::style::Theme;
use kivact::style::auto;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::CommandPaletteRecipes;
use kivact::ui::Look;

const PALETTE_WIDTH: f64 = 640.0;
const PALETTE_MAX_HEIGHT: f64 = 480.0;

pub(crate) fn command_palette(r: CommandPaletteRecipes) -> CommandPaletteRecipes {
    r.scrim(|_| |t: &Theme, _: &Look<PartState>| scrim(&t.ext::<Tokens>()))
        .panel(|_| panel)
        .search(|_| search)
        .search_icon(|_| |t: &Theme, _: &Look<PartState>| icon_box(&t.ext::<Tokens>()))
        .project(|_| field)
        .list(|_| {
            |t: &Theme, _: &Look<PartState>| {
                let p = t.ext::<Tokens>();
                style()
                    .direction(Direction::Column)
                    .min_height(0)
                    .padding(p.spacing.s1)
                    .gap(p.spacing.s0_5)
            }
        })
        .group(|_| {
            |t: &Theme, _: &Look<PartState>| {
                let p = t.ext::<Tokens>();
                supporting_text(&p)
                    .padding_x(p.spacing.s3)
                    .padding_y(p.spacing.s1)
            }
        })
        .row(|_| row)
        .icon(|_| |t: &Theme, _: &Look<ListItemState>| icon_box(&t.ext::<Tokens>()))
        .hint(|_| {
            |t: &Theme, _: &Look<ListItemState>| {
                supporting_text(&t.ext::<Tokens>())
                    .margin_left(auto())
                    .shrink(0)
            }
        })
        .empty(|_| {
            |t: &Theme, _: &Look<PartState>| {
                let p = t.ext::<Tokens>();
                supporting_text(&p)
                    .padding_x(p.spacing.s4)
                    .padding_y(p.spacing.s8)
                    .align_self(Align::Stretch)
                    .text_align(TextAlign::Center)
            }
        })
        .footer(|_| footer)
}

fn panel(t: &Theme, _: &Look<PopupState>) -> Style {
    let p = t.ext::<Tokens>();
    let gutter = p.spacing.layer_viewport_gutter;
    modal(
        shadows(body_text(&p), &p.shadows.high)
            .background(p.colors.bg_surface)
            .radius(p.radius.container)
            .direction(Direction::Column)
            .width(PALETTE_WIDTH)
            .max_width(pct(100) - gutter * 2.0)
            .max_height(PALETTE_MAX_HEIGHT)
            .overflow(Overflow::Hidden),
        &p,
    )
}

fn search(t: &Theme, _: &Look<PartState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .shrink(0)
        .padding_x(p.spacing.s4)
        .padding_y(p.spacing.s3)
        .gap(p.spacing.s2)
        .border_bottom(p.sizes.border_width, p.colors.border)
}

fn field(t: &Theme, _: &Look<PartState>) -> Style {
    let p = t.ext::<Tokens>();
    body_text(&p)
        .grow(1)
        .min_width(0)
        .height(p.typography.body.line_height_px)
        .background(Color::TRANSPARENT)
        .placeholder_color(p.colors.text_secondary)
        .caret_color(p.colors.text_primary)
}

fn row(t: &Theme, s: &Look<ListItemState>) -> Style {
    let p = t.ext::<Tokens>();
    let fill = if s.active {
        p.colors.overlay_hover
    } else if s.selected {
        p.colors.accent_muted
    } else {
        Color::TRANSPARENT
    };
    let base = body_text(&p)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(p.spacing.s2)
        .padding_x(p.spacing.s3)
        .padding_y(p.spacing.s2)
        .min_height(p.sizes.control_lg)
        .radius(p.radius.inner)
        .background(fill);
    if s.input.disabled {
        base.opacity(p.sizes.disabled_opacity)
    } else {
        base.hover(style().background(p.colors.overlay_hover))
            .pressed(style().background(p.colors.overlay_pressed))
    }
}

fn icon_box(p: &Tokens) -> Style {
    style()
        .size(p.sizes.icon_sm, p.sizes.icon_sm)
        .shrink(0)
        .color(p.colors.text_secondary)
}

fn footer(t: &Theme, _: &Look<PartState>) -> Style {
    let p = t.ext::<Tokens>();
    supporting_text(&p)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(p.spacing.s4)
        .padding_x(p.spacing.s4)
        .padding_y(p.spacing.s2)
        .border_top(p.sizes.border_width, p.colors.border)
        .shrink(0)
}
