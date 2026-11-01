//! Role: Lyra recipes for the standard dialog: dialog, alert dialog, sheet and popover.
//!
//! One table draws every kind of `kivact::ui::dialog`: a `Modal` is
//! dialog.tsx, an `AlertModal` alert-dialog.tsx (its width class is the
//! [`AlertDialogSize`] given with `.variant(..)`), a `Drawer(side)`
//! sheet.tsx attached to that edge and a `Popover` popover.tsx.

use crate::alert_dialog::AlertDialogSize;
use crate::layers::{close_box, description_style, scrim_style, title_style};
use crate::theme_access::{popover_surface, shadow_lg, shadow_md};
use crate::tokens::Tokens;
use kivact::style::{
    Align, Direction, Justify, Position, Style, TextAlign, Theme, auto, pct, style,
};
use kivact::ui::{DialogKind, DialogRecipes, DialogState, Look, Side};

/// A popover drawn without its padding (`p-0`), for a panel whose content
/// draws its own, such as the date picker's calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Flush;

impl kivact::ui::Variant for Flush {}

/// The width class of an alert dialog; `Default` when it has none.
fn alert_size(s: &Look<DialogState>) -> AlertDialogSize {
    s.variant::<AlertDialogSize>().copied().unwrap_or_default()
}

/// Whether `s` is an alert dialog drawn centered (`data-size=sm`).
fn centered(s: &Look<DialogState>) -> bool {
    s.kind == DialogKind::AlertModal && alert_size(s) == AlertDialogSize::Sm
}

pub(crate) fn dialog(r: DialogRecipes) -> DialogRecipes {
    r.surface(|_| surface)
        .title(|_| {
            |_: &Theme, s: &Look<DialogState>| {
                let base = title_style();
                if centered(s) {
                    base.text_align(TextAlign::Center)
                } else {
                    base
                }
            }
        })
        .description(|_| {
            |_: &Theme, s: &Look<DialogState>| {
                let base = description_style();
                if centered(s) {
                    base.text_align(TextAlign::Center)
                } else {
                    base
                }
            }
        })
        .header(|_| header)
        .media(|_| media)
        .body(|_| {
            |_: &Theme, s: &Look<DialogState>| {
                let base = style().direction(Direction::Column);
                match s.kind {
                    // sheet.tsx has no body box: the content sits between
                    // the header's and the footer's `p-4`.
                    DialogKind::Drawer(_) => base.padding_x(16),
                    _ => base.gap(8),
                }
            }
        })
        .footer(|_| footer)
        .scrim(|_| |_: &Theme, _: &Look<DialogState>| scrim_style())
        .close(|_| {
            |t: &Theme, s: &Look<DialogState>| {
                // `top-3 right-3` on a sheet, `top-2 right-2` on a dialog.
                let inset = if matches!(s.kind, DialogKind::Drawer(_)) {
                    12.0
                } else {
                    8.0
                };
                close_box(&s.input, inset, inset).color(t.get::<Tokens>().foreground)
            }
        })
}

fn surface(t: &Theme, s: &Look<DialogState>) -> Style {
    let p = t.get::<Tokens>();
    let base = popover_surface(&p)
        .direction(Direction::Column)
        .line_height(1.625);
    match s.kind {
        // popover.tsx: `w-72 p-2.5 gap-2.5 shadow-md`; `p-0` for a flush one.
        DialogKind::Popover => {
            let pad = if s.variant::<Flush>().is_some() {
                0
            } else {
                10
            };
            shadow_md(base)
                .width(288)
                .max_width(pct(100))
                .padding(pad)
                .gap(pad)
        }
        // sheet.tsx: `shadow-lg` and the border of the inner edge, no ring.
        DialogKind::Drawer(side) => sheet(shadow_lg(base.ring_none()), side, &p),
        // alert-dialog.tsx: `max-w-sm` or `max-w-xs`.
        DialogKind::AlertModal => base
            .width(match alert_size(s) {
                AlertDialogSize::Default => 384,
                AlertDialogSize::Sm => 320,
            })
            .max_width(pct(100) - 32)
            .padding(16)
            .gap(16),
        _ => base.width(384).max_width(pct(100) - 32).padding(16).gap(16),
    }
}

/// A sheet on `side`: left and right ones take 3/4 of the window's width, at
/// most 384 px (`w-3/4 sm:max-w-sm`); top and bottom ones span it.
fn sheet(base: Style, side: Side, p: &Tokens) -> Style {
    match side {
        Side::Right => base.width(pct(75)).max_width(384).border_left(1, p.border),
        Side::Left => base.width(pct(75)).max_width(384).border_right(1, p.border),
        Side::Top => base.max_height(pct(100)).border_bottom(1, p.border),
        Side::Bottom => base.max_height(pct(100)).border_top(1, p.border),
    }
}

fn header(_: &Theme, s: &Look<DialogState>) -> Style {
    let column = style().direction(Direction::Column);
    match s.kind {
        // sheet.tsx: `flex flex-col gap-0.5 p-4`.
        DialogKind::Drawer(_) => column.gap(2).padding(16),
        DialogKind::AlertModal if centered(s) => column.gap(6).align_items(Align::Center),
        // alert-dialog.tsx: the media spans the title and the description
        // at their start (`grid-rows-[auto_1fr] gap-x-4`): the column of
        // the text leaves the media's 40 px and the gap at its start.
        DialogKind::AlertModal if s.media => column
            .gap(6)
            .align_items(Align::Start)
            .padding_left(56)
            .min_height(40),
        DialogKind::AlertModal => column.gap(6).align_items(Align::Start),
        // dialog.tsx and popover.tsx: `flex flex-col gap-1`.
        _ => column.gap(4),
    }
}

/// alert-dialog.tsx's media: a 40 px `bg-muted` square centering its icon.
fn media(t: &Theme, s: &Look<DialogState>) -> Style {
    let p = t.get::<Tokens>();
    let base = style()
        .size(40, 40)
        .shrink(0)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(p.muted)
        .color(p.foreground);
    if centered(s) {
        // The header's `gap-1.5` and 8 px more under it.
        base.margin_bottom(8)
    } else {
        base.position(Position::Absolute).left(0).top(0)
    }
}

fn footer(_: &Theme, s: &Look<DialogState>) -> Style {
    match s.kind {
        // sheet.tsx: `mt-auto flex flex-col gap-2 p-4`.
        DialogKind::Drawer(_) => style()
            .direction(Direction::Column)
            .margin_top(auto())
            .gap(8)
            .padding(16),
        // alert-dialog.tsx's `data-size=sm` is a two-column grid; a style
        // makes no grid of the footer's row, so the buttons share it when
        // each grows (`.style(style().grow(1).basis(0))`).
        DialogKind::AlertModal if centered(s) => style().direction(Direction::Row).gap(8),
        _ => style()
            .direction(Direction::Row)
            .justify_content(Justify::End)
            .gap(8),
    }
}
