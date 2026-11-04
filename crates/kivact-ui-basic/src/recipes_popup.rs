//! Role: Lyra recipes for select, menu, tooltip and tabs parts.
//!
//! The looks of select.tsx, dropdown-menu.tsx, tooltip.tsx and
//! the `default` variant of tabs.tsx.

use crate::button::{Look, trigger_style};
use crate::list_popup::{option_row, surface_style};
use crate::theme_access::{CHEVRON_DOWN, field_fill, stroke_glyph, wide_focus_ring, with_focus};
use crate::tokens::Tokens;
use kivact::draw::Point;
use kivact::interaction::{ListItemState, PopupState, TabListState, TabState};
use kivact::style::{Align, Color, Direction, Justify, Position, Style, Theme, pct, style};
use kivact::ui::{ControlSize, MenuRecipes, SelectRecipes, Side, TabsRecipes, TooltipArrowState};
use kivact::view::Orientation;

pub(crate) fn select(r: SelectRecipes) -> SelectRecipes {
    r.trigger(|_| select_trigger)
        .surface(|_| {
            |t: &Theme, _: &kivact::ui::Look<PopupState>| {
                surface_style(&t.get::<Tokens>())
                    .min_width(144)
                    .max_height(320)
            }
        })
        .option(|_| {
            |t: &Theme, s: &kivact::ui::Look<ListItemState>| option_row(&t.get::<Tokens>(), s)
        })
        .check(|_| |_: &Theme, _: &kivact::ui::Look<ListItemState>| check_box())
        .indicator(|_| select_indicator)
        .description(|_| {
            |_: &Theme, _: &kivact::ui::Look<ListItemState>| {
                style().color(crate::Color::MutedForeground)
            }
        })
}

fn select_trigger(t: &Theme, s: &kivact::ui::Look<kivact::ui::SelectTriggerState>) -> Style {
    let p = t.get::<Tokens>();
    let base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(6)
        // `data-[size=default]:h-8 data-[size=sm]:h-7`, like input and button.
        .height(match s.size {
            ControlSize::ExtraSmall => 24,
            ControlSize::Small => 28,
            ControlSize::Medium => 32,
            ControlSize::Large => 36,
        })
        .padding_left(10)
        .padding_right(8)
        .radius(0)
        .border(1, p.input)
        .background(field_fill(&p))
        // `data-placeholder:text-muted-foreground`.
        .color(if s.placeholder {
            p.muted_foreground
        } else {
            p.foreground
        })
        .font_size(12)
        .text_wrap(false);
    let base = if p.is_dark() {
        base.hover(style().background(p.input.with_alpha(0.5)))
    } else {
        base
    };
    with_focus(base, &p)
}

/// The box of the trigger's `chevron-down` icon.
fn select_indicator(t: &Theme, _: &kivact::ui::Look<PopupState>) -> Style {
    style()
        .size(16, 16)
        .shrink(0)
        .color(t.get::<Tokens>().muted_foreground)
}

pub(crate) fn menu(r: MenuRecipes) -> MenuRecipes {
    crate::recipes_menu::menu(r.trigger(|_| menu_trigger))
}

/// An outline button with a trailing `chevron-down`.
fn menu_trigger(t: &Theme, s: &kivact::ui::Look<PopupState>) -> Style {
    let p = t.get::<Tokens>();
    let color = p.muted_foreground;
    trigger_style(
        &p,
        Look::Outline,
        ControlSize::Medium,
        &s.input,
        Some(s.open),
    )
    .padding_right(30)
    .paint(move |d, size| {
        let at = Point::new(size.width - 24.0, (size.height - 16.0) / 2.0);
        stroke_glyph(d, at, 16.0, CHEVRON_DOWN, color);
    })
}

/// The 16 px box of a `check` at a row's right, in the row's color.
fn check_box() -> Style {
    style()
        .position(Position::Absolute)
        .right(8)
        .top(pct(50) - 8)
        .size(16, 16)
}

pub(crate) fn tooltip(t: &Theme, _: &kivact::ui::Look<PopupState>) -> Style {
    let p = t.get::<Tokens>();
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(6)
        .max_width(320)
        .padding_x(12)
        .padding_y(6)
        .radius(0)
        .background(p.foreground)
        .color(p.background)
        .font_size(12)
}

/// tooltip.tsx's arrow: a `size-2.5` square in `bg-foreground`, turned 45°,
/// half out of the edge toward the trigger.
pub(crate) fn tooltip_arrow(t: &Theme, s: &kivact::ui::Look<TooltipArrowState>) -> Style {
    const SIDE: f64 = 10.0;
    let p = t.get::<Tokens>();
    let base = style()
        .position(Position::Absolute)
        .size(SIDE, SIDE)
        .rotate(45)
        .background(p.foreground);
    let along = s.offset - SIDE / 2.0;
    match s.side {
        // `-bottom-2.5 translate-y-[calc(-50%-2px)]`.
        Side::Top => base.left(along).top(pct(100) - 7.0),
        // `top-1 translate-y-[calc(-50%-2px)]`.
        Side::Bottom => base.left(along).top(-3),
        // `top-1/2! -right-1 -translate-y-1/2`.
        Side::Left => base.top(pct(50) - SIDE / 2.0).left(pct(100) - 6.0),
        // `top-1/2! -left-1 -translate-y-1/2`.
        Side::Right => base.top(pct(50) - SIDE / 2.0).left(-4),
    }
}

/// tabs.tsx's `default` variant, and its `line` variant for tabs given
/// [`TabsVariant::Line`](crate::TabsVariant).
pub(crate) fn tabs(r: TabsRecipes) -> TabsRecipes {
    use crate::line_tabs::{self as line, is_line};
    r.list(|_| {
        |t: &Theme, s: &kivact::ui::Look<TabListState>| {
            if is_line(s) {
                line::list(t, s)
            } else {
                tabs_list(t, s)
            }
        }
    })
    .tab(|_| {
        |t: &Theme, s: &kivact::ui::Look<TabState>| {
            if is_line(s) {
                line::tab(t, s)
            } else {
                tab(t, s)
            }
        }
    })
    .panel(|_| {
        |t: &Theme, s: &kivact::ui::Look<TabState>| {
            if is_line(s) {
                line::panel(t, s)
            } else {
                tab_panel(t, s)
            }
        }
    })
}

/// `flex-1 text-xs/relaxed outline-none`: a panel draws no focus ring; the
/// root's `gap-2` is the panel's leading padding.
fn tab_panel(t: &Theme, s: &kivact::ui::Look<TabState>) -> Style {
    let p = t.get::<Tokens>();
    let base = style()
        .grow(1)
        .font_size(12)
        .line_height(1.625)
        .color(p.foreground)
        .focus_visible(style().ring_none());
    if s.orientation == Orientation::Vertical {
        base.padding_left(8)
    } else {
        base.padding_top(8)
    }
}

fn tabs_list(t: &Theme, s: &kivact::ui::Look<TabListState>) -> Style {
    let p = t.get::<Tokens>();
    let base = style()
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .align_self(Align::Start)
        .padding(3)
        .radius(0)
        .background(p.muted)
        .color(p.muted_foreground);
    if s.orientation == Orientation::Vertical {
        // `group-data-vertical/tabs:h-fit flex-col`.
        base.direction(Direction::Column)
    } else {
        base.direction(Direction::Row).height(32)
    }
}

fn tab(t: &Theme, s: &kivact::ui::Look<TabState>) -> Style {
    let p = t.get::<Tokens>();
    let dark = p.is_dark();
    let rest = if dark {
        p.muted_foreground
    } else {
        p.foreground.with_alpha(0.6)
    };
    let base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(6)
        .padding_x(6)
        .radius(0)
        .border(1, Color::TRANSPARENT)
        .font_size(12)
        .weight(500)
        .text_wrap(false)
        .color(rest)
        .hover(style().color(p.foreground));
    let base = if s.orientation == Orientation::Vertical {
        // `group-data-vertical/tabs:w-full justify-start py-[calc(--spacing(1.25))]`.
        base.width(pct(100))
            .justify_content(Justify::Start)
            .padding_y(5)
    } else {
        // `h-[calc(100%-1px)] flex-1 py-0.5`.
        base.grow(1).height(pct(100) - 1).padding_y(2)
    };
    let base = if s.selected {
        let (fill, border) = if dark {
            (p.input.with_alpha(0.3), p.input)
        } else {
            (p.background, Color::TRANSPARENT)
        };
        base.background(fill)
            .border_color(border)
            .color(p.foreground)
    } else {
        base
    };
    // `focus-visible:ring-[3px] ring-ring/50 outline-1 outline-ring`.
    base.focus_visible(wide_focus_ring(&p))
}
