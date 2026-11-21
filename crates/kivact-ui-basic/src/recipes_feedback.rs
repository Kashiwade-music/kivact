//! Role: Lyra recipes of the feedback parts: toasts, skeletons, spinners and keyboard keys.
//!
//! The looks of sonner.tsx (the stacked toasts), skeleton.tsx, spinner.tsx
//! and kbd.tsx. The badge, avatar, alert and empty state recipes join this
//! module later.

use crate::color::{
    BORDER, DESTRUCTIVE, FOREGROUND, MUTED, MUTED_FOREGROUND, POPOVER, POPOVER_FOREGROUND,
};
use crate::recipe_parts::{shadow_lg, stroke_loader};
use kivact::custom::PartState;
use kivact::geometry::Point;
use kivact::interaction::{ToastState, Tone};
use kivact::style::{Align, FlexDirection, Justify, Overflow, Position, Style, Theme, pct, style};
use kivact::ui::ControlSize;
use kivact::ui::SkeletonShape;
use kivact::ui::theme::{
    KbdKeyState, KbdTheme, Look, SkeletonState, SkeletonTheme, SpinnerState, SpinnerTheme,
    ToastGroupState, ToastTheme,
};

/// The gap of an open stack and the offset of a toast behind the newest
/// (sonner's `--gap`).
const GAP: f64 = 14.0;
/// How much each toast behind the newest is smaller than the one in front.
const SCALE_STEP: f64 = 0.05;
/// How many toasts show behind the newest in a closed stack (sonner shows
/// three in all).
const BEHIND: usize = 2;

/// The stack at the bottom right of the window, above the page.
fn toast_region(_: &Theme, _: &Look<PartState>, _base: Style) -> Style {
    style()
        .position(Position::Fixed)
        .z_index(100)
        .right(16.0)
        .bottom(16.0)
        .width(384.0)
        .max_width(pct(100.0) - 32.0)
}

/// One live region's stack; the assertive one sits a gap below the polite
/// one while both show toasts.
fn toast_group(_: &Theme, s: &Look<ToastGroupState>, _base: Style) -> Style {
    let base = style()
        .position(Position::Relative)
        .gap(GAP)
        .width(pct(100.0));
    if s.count > 0 && s.above > 0 {
        base.margin_top(GAP)
    } else {
        base
    }
}

/// Whether a toast is behind the newest of a closed stack: only a card
/// whose content does not show (`data-front=false`).
fn behind(s: &Look<ToastState>) -> bool {
    s.depth > 0 && !s.engaged
}

/// `rounded-none border bg-popover shadow-lg p-4 gap-3`. A closed stack
/// (nothing in it hovered or focused) shows the newest toast in front and
/// up to two older ones peeking out above it, each one smaller; it opens
/// into a list while held.
fn toast_card(_: &Theme, s: &Look<ToastState>, _base: Style) -> Style {
    let card = shadow_lg(
        style()
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .gap(12.0)
            .padding(16.0)
            .border_radius(0.0)
            .border(1.0, BORDER)
            .background(POPOVER)
            .color(POPOVER_FOREGROUND)
            .font_size(14.0),
    );
    if !behind(s) {
        return card.z_index(BEHIND as i32 + 1);
    }
    // Stretched over the newest toast, which sets the group's size, then
    // moved up by `k` gaps and shrunk toward its bottom edge.
    let k = s.depth;
    let scale = 1.0 - SCALE_STEP * k as f64;
    card.position(Position::Absolute)
        .top(0.0)
        .right(0.0)
        .bottom(0.0)
        .left(0.0)
        .overflow(Overflow::Hidden)
        .z_index((BEHIND + 1).saturating_sub(k) as i32)
        .transform_origin(pct(50.0), pct(100.0))
        .translate_y(-GAP * k as f64)
        .scale_xy(scale, scale)
        .opacity(if k > BEHIND { 0.0 } else { 1.0 })
}

/// The content of a toast behind the newest does not show.
fn content(s: &Look<ToastState>, base: Style) -> Style {
    base.opacity(if behind(s) { 0.0 } else { 1.0 })
}

fn toast_title(_: &Theme, s: &Look<ToastState>, _base: Style) -> Style {
    content(
        s,
        style()
            .font_size(14.0)
            .font_weight(500)
            .line_height(20.0 / 14.0)
            .color(POPOVER_FOREGROUND),
    )
}

fn toast_body(_: &Theme, s: &Look<ToastState>, _base: Style) -> Style {
    content(
        s,
        style()
            .margin_top(4.0)
            .font_size(14.0)
            .line_height(20.0 / 14.0)
            .color(MUTED_FOREGROUND),
    )
}

/// The icon of the toast's tone: 16 px in the text color, in the destructive
/// color for a danger.
fn toast_icon(_: &Theme, s: &Look<ToastState>, _base: Style) -> Style {
    let base = style().width(16.0).height(16.0).flex_shrink(0.0);
    content(
        s,
        if s.tone == Tone::Danger {
            base.color(DESTRUCTIVE)
        } else {
            base
        },
    )
}

/// `shrink-0 text-muted-foreground after:-inset-2 hover:text-foreground`: a
/// bare 16 px `x` whose hit area reaches 8 px beyond it.
fn toast_close(_: &Theme, s: &Look<ToastState>, _base: Style) -> Style {
    content(
        s,
        style()
            .width(32.0)
            .height(32.0)
            .margin(-8.0)
            .flex_direction(FlexDirection::Row)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .flex_shrink(0.0)
            .border_radius(0.0)
            .color(MUTED_FOREGROUND)
            .hover(style().color(FOREGROUND)),
    )
}

/// Installs the toast parts.
fn install_toast(theme: Theme) -> Theme {
    theme.with(
        ToastTheme::default()
            .region(toast_region)
            .group(toast_group)
            .toast(toast_card)
            .title(toast_title)
            .body(toast_body)
            .icon(toast_icon)
            .close(toast_close),
    )
}

/// A skeleton: a `bg-muted` block that pulses, a circle when it is one.
fn skeleton(_: &Theme, s: &Look<SkeletonState>, _base: Style) -> Style {
    let base = style()
        .flex_grow(1.0)
        .min_height(16.0)
        .width(pct(100.0))
        .background(MUTED)
        .opacity(s.pulse);
    match s.shape {
        SkeletonShape::Circle => base.width(36.0).height(36.0).border_radius(999.0),
        _ => base.border_radius(0.0),
    }
}

/// A spinner: a ring of the text color turning with the state.
fn spinner(_: &Theme, s: &Look<SpinnerState>, _base: Style) -> Style {
    let side = match s.size {
        ControlSize::ExtraSmall => 12.0,
        ControlSize::Small => 14.0,
        ControlSize::Medium => 16.0,
        ControlSize::Large => 20.0,
    };
    let turn = s.turn / 360.0;
    style()
        .width(side)
        .height(side)
        .flex_shrink(0.0)
        .paint(move |d, _| stroke_loader(d, Point::new(0.0, 0.0), side, turn, FOREGROUND))
}

/// A keyboard key: a small `bg-muted` chip with its label.
fn kbd_key(_: &Theme, _: &Look<KbdKeyState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(4.0)
        .height(20.0)
        .min_width(20.0)
        .padding_x(4.0)
        .border_radius(0.0)
        .background(MUTED)
        .color(MUTED_FOREGROUND)
        .font_size(12.0)
        .font_weight(500)
        .text_wrap(false)
}

/// Installs the feedback parts of this module.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = install_toast(theme);
    // The reduced period of the busy spinner stays with its recipe (`with`
    // replaces the whole table).
    theme
        .with(SkeletonTheme::default().root(skeleton))
        .with(
            SpinnerTheme::default()
                .root(spinner)
                .reduced_period(Some(1000.0)),
        )
        .with(KbdTheme::default().key(kbd_key))
}
