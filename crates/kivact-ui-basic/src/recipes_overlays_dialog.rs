//! Role: Lyra recipes of the dialog: dialog, alert dialog, sheet and popover.
//!
//! One table draws every kind of `kivact::ui::dialog`: a `Modal` is
//! dialog.tsx, an `AlertModal` alert-dialog.tsx (its width class is the
//! [`AlertDialogSize`] given with `.variant(..)`), a `Drawer(side)`
//! sheet.tsx attached to that edge and a `Popover` popover.tsx.

use crate::color::{BORDER, FOREGROUND, MUTED};
use crate::recipe_parts::{
    close_box, description_style, popover_surface, scrim_style, shadow_lg, shadow_md, title_style,
};
use crate::variants::{AlertDialogSize, DatePickerVariant, DrawerVariant};
use kivact::geometry::Edge;
use kivact::style::{
    Align, FlexDirection, Justify, Position, Style, TextAlign, Theme, auto, pct, style,
};
use kivact::ui::DialogKind;
use kivact::ui::theme::{DialogState, DialogTheme, Look};

/// The width class of an alert dialog; `Default` when it has none.
fn alert_size(s: &Look<DialogState>) -> AlertDialogSize {
    s.variant::<AlertDialogSize>().unwrap_or_default()
}

/// Whether `s` is an alert dialog drawn centered (`data-size=sm`).
fn centered(s: &Look<DialogState>) -> bool {
    s.kind == DialogKind::AlertModal && alert_size(s) == AlertDialogSize::Sm
}

/// Whether `s` is the swipe drawer with its visible grab handle.
fn swipe_drawer(s: &Look<DialogState>) -> bool {
    s.variant::<DrawerVariant>() == Some(DrawerVariant::Drawer)
}

fn surface(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    let base = popover_surface()
        .flex_direction(FlexDirection::Column)
        .line_height(1.625);
    match s.kind {
        // popover.tsx: `w-72 p-2.5 gap-2.5 shadow-md`; `p-0` for a flush one.
        DialogKind::Popover => {
            let flush = s.variant::<DatePickerVariant>().is_some();
            let pad = if flush { 0.0 } else { 10.0 };
            let base = if flush {
                base.width(auto())
            } else {
                base.width(288.0)
            };
            shadow_md(base).max_width(pct(100.0)).padding(pad).gap(pad)
        }
        // sheet.tsx: `shadow-lg` and the border of the inner edge, no ring.
        DialogKind::Drawer(side) => {
            if swipe_drawer(s) {
                let base = sheet(base.outline_none(), side).position(Position::Relative);
                match side {
                    Edge::Top => base.padding_bottom(12.0).max_height(pct(100.0) - 96.0),
                    Edge::Bottom => base.padding_top(12.0).max_height(pct(100.0) - 96.0),
                    Edge::Left => base.padding_right(12.0),
                    Edge::Right => base.padding_left(12.0),
                }
            } else {
                sheet(shadow_lg(base.outline_none()), side)
            }
        }
        // alert-dialog.tsx: `max-w-sm` or `max-w-xs`.
        DialogKind::AlertModal => base
            .width(match alert_size(s) {
                AlertDialogSize::Default => 384.0,
                AlertDialogSize::Sm => 320.0,
            })
            .max_width(pct(100.0) - 32.0)
            .padding(16.0)
            .gap(16.0),
        _ => base
            .width(384.0)
            .max_width(pct(100.0) - 32.0)
            .padding(16.0)
            .gap(16.0),
    }
}

/// A sheet on `side`: left and right ones take 3/4 of the window's width, at
/// most 384 px (`w-3/4 sm:max-w-sm`); top and bottom ones span it.
fn sheet(base: Style, side: Edge) -> Style {
    match side {
        Edge::Right => base
            .width(pct(75.0))
            .max_width(384.0)
            .border_left(1.0, BORDER),
        Edge::Left => base
            .width(pct(75.0))
            .max_width(384.0)
            .border_right(1.0, BORDER),
        Edge::Top => base.max_height(pct(100.0)).border_bottom(1.0, BORDER),
        Edge::Bottom => base.max_height(pct(100.0)).border_top(1.0, BORDER),
    }
}

fn title(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    let base = title_style();
    if centered(s) {
        base.text_align(TextAlign::Center)
    } else {
        base
    }
}

fn description(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    let base = description_style();
    if centered(s) {
        base.text_align(TextAlign::Center)
    } else {
        base
    }
}

fn header(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    let column = style().flex_direction(FlexDirection::Column);
    match s.kind {
        // sheet.tsx: `flex flex-col gap-0.5 p-4`.
        DialogKind::Drawer(_) if swipe_drawer(s) => {
            column.gap(2.0).padding(16.0).padding_bottom(0.0)
        }
        DialogKind::Drawer(_) => column.gap(2.0).padding(16.0),
        DialogKind::AlertModal if centered(s) => column.gap(6.0).align_items(Align::Center),
        // alert-dialog.tsx: the media spans the title and the description
        // at their start (`grid-rows-[auto_1fr] gap-x-4`): the column of
        // the text leaves the media's 40 px and the gap at its start.
        DialogKind::AlertModal if s.media => column
            .gap(6.0)
            .align_items(Align::Start)
            .padding_left(56.0)
            .min_height(40.0),
        DialogKind::AlertModal => column.gap(6.0).align_items(Align::Start),
        // dialog.tsx and popover.tsx: `flex flex-col gap-1`.
        _ => column.gap(4.0),
    }
}

/// alert-dialog.tsx's media: a 40 px `bg-muted` square centering its icon.
fn media(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    let base = style()
        .size(40.0, 40.0)
        .flex_shrink(0.0)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(MUTED)
        .color(FOREGROUND);
    if centered(s) {
        // The header's `gap-1.5` and 8 px more under it.
        base.margin_bottom(8.0)
    } else {
        base.position(Position::Absolute).left(0.0).top(0.0)
    }
}

fn footer(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    match s.kind {
        // sheet.tsx: `mt-auto flex flex-col gap-2 p-4`.
        DialogKind::Drawer(_) if swipe_drawer(s) => style()
            .flex_direction(FlexDirection::Column)
            .margin_top(auto())
            .gap(8.0)
            .padding(16.0)
            .padding_top(0.0),
        DialogKind::Drawer(_) => style()
            .flex_direction(FlexDirection::Column)
            .margin_top(auto())
            .gap(8.0)
            .padding(16.0),
        // alert-dialog.tsx's `data-size=sm` is a two-column grid; a style
        // makes no grid of the footer's row, so the buttons share it when
        // each grows (`.style(style().grow(1).basis(0))`).
        DialogKind::AlertModal if centered(s) => {
            style().flex_direction(FlexDirection::Row).gap(8.0)
        }
        _ => style()
            .flex_direction(FlexDirection::Row)
            .justify_content(Justify::End)
            .gap(8.0),
    }
}

fn scrim(_: &Theme, _: &Look<DialogState>, _base: Style) -> Style {
    scrim_style()
}

fn close(t: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    // `top-3 right-3` on a sheet, `top-2 right-2` on a dialog.
    let inset = if matches!(s.kind, DialogKind::Drawer(_)) {
        12.0
    } else {
        8.0
    };
    close_box(t, &s.input, inset, inset).color(FOREGROUND)
}

fn drag_handle(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    if !swipe_drawer(s) {
        return style()
            .position(Position::Absolute)
            .size(0.0, 0.0)
            .opacity(0.0);
    }
    let DialogKind::Drawer(edge) = s.kind else {
        return style();
    };
    let horizontal = matches!(edge, Edge::Left | Edge::Right);
    let base = style()
        .position(Position::Absolute)
        .flex_shrink(0.0)
        .cursor(if s.input.is_pressed() {
            kivact::style::Cursor::Grabbing
        } else {
            kivact::style::Cursor::Grab
        })
        .paint(move |d, size| {
            let (w, h) = if horizontal { (4.0, 48.0) } else { (48.0, 4.0) };
            let (x, y) = match edge {
                Edge::Bottom => ((size.width - w) / 2.0, size.height - h),
                Edge::Top => ((size.width - w) / 2.0, 0.0),
                Edge::Left => (0.0, (size.height - h) / 2.0),
                Edge::Right => (size.width - w, (size.height - h) / 2.0),
            };
            d.fill_rect(kivact::geometry::Rect::new(x, y, w, h), MUTED);
        });
    match edge {
        Edge::Bottom => base.top(0.0).left(0.0).right(0.0).height(12.0),
        Edge::Top => base.bottom(0.0).left(0.0).right(0.0).height(12.0),
        Edge::Left => base.right(0.0).top(0.0).bottom(0.0).width(12.0),
        Edge::Right => base.left(0.0).top(0.0).bottom(0.0).width(12.0),
    }
}

/// The dialog recipes: the surface, the parts of the header, body and footer,
/// the scrim, the close box and the drag handle of a drawer.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        DialogTheme::default()
            .surface(surface)
            .title(title)
            .description(description)
            .header(header)
            .media(media)
            .body(body)
            .footer(footer)
            .scrim(scrim)
            .close(close)
            .drag_handle(drag_handle),
    )
}

/// The body: sheet.tsx has no body box, so the content sits between the
/// header's and the footer's `p-4`.
fn body(_: &Theme, s: &Look<DialogState>, _base: Style) -> Style {
    let base = style().flex_direction(FlexDirection::Column);
    match s.kind {
        DialogKind::Drawer(_) if swipe_drawer(s) => base.padding_x(16.0).padding_top(16.0),
        DialogKind::Drawer(_) => base.padding_x(16.0),
        _ => base.gap(8.0),
    }
}
