//! Role: Lyra recipes for the radio group, slider, toasts, splitter, toggle and more.
//!
//! The looks of radio-group.tsx, slider.tsx, sonner.tsx (the stacked
//! toasts), resizable.tsx's handle and grip, toggle.tsx, progress.tsx and
//! accordion.tsx, drawn by `kivact::ui::radio_group`, `slider`,
//! `range_slider`, `toast_region`, `splitter`, `split`, `toggle`, `progress`
//! and `disclosure`.

use crate::choice::{ToggleVariant, toggle_style};
use crate::theme_access::{shadow_lg, wide_focus_ring, with_focus};
use crate::tokens::Tokens;
use kivact::custom::PartState;
use kivact::geometry::Rect;
use kivact::interaction::ToastState;
use kivact::style::{
    Align, Color, Cursor, Direction, Justify, Overflow, Position, Style, TextDecorationLine, Theme,
    pct, style,
};
use kivact::ui::DisclosureRecipeState;
use kivact::ui::{
    AxisState, CheckableState, DisclosureRecipes, Look, ProgressRecipes, ProgressState,
    RadioGroupRecipes, SliderRecipes, SplitRecipes, SplitterRecipes, ToastGroupState, ToastRecipes,
    ToggleButtonState,
};
use kivact::view::Orientation;

pub(crate) fn radio_group(r: RadioGroupRecipes) -> RadioGroupRecipes {
    r.group(|_| radio_list)
        .item(|_| radio_item)
        .indicator(|_| radio_indicator)
        .description(|_| radio_description)
}

/// `grid w-full gap-2`, or a row with `gap-4` when horizontal.
fn radio_list(_: &Theme, orientation: &kivact::ui::Look<Orientation>) -> Style {
    match **orientation {
        Orientation::Horizontal => style().direction(Direction::Row).gap(16).shrink(0),
        _ => style()
            .direction(Direction::Column)
            .gap(8)
            .width(pct(100))
            .shrink(0),
    }
}

/// A row of the indicator and the label (`text-xs/snug`); the indicator
/// shows the focus, so the row draws no ring of its own.
fn radio_item(t: &Theme, _: &kivact::ui::Look<CheckableState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Start)
        .gap(8)
        .font_size(12)
        .line_height(1.375)
        .color(t.ext::<Tokens>().foreground)
        .focus_visible(style().ring_none())
}

/// `size-4 rounded-full border border-input dark:bg-input/30`, filled with
/// `primary` while checked around the theme's 8 px `dot` in
/// `primary-foreground`.
///
/// The ring is the item's focus: it is drawn on the indicator, a part
/// inside the focused item, so the recipe reads the item's state.
fn radio_indicator(t: &Theme, s: &kivact::ui::Look<CheckableState>) -> Style {
    let p = t.ext::<Tokens>();
    let checked = s.checked.is_checked();
    let base = style()
        .position(Position::Relative)
        .size(16, 16)
        .shrink(0)
        .radius(8)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border(1, if checked { p.primary } else { p.input })
        .color(p.primary_foreground)
        .background(if checked {
            p.primary
        } else if p.is_dark() {
            p.input.with_alpha(0.3)
        } else {
            Color::TRANSPARENT
        });
    if s.input.focus_visible {
        base.merge(&wide_focus_ring(&p))
    } else {
        base
    }
}

/// `text-xs/relaxed text-muted-foreground` under the label.
fn radio_description(t: &Theme, _: &kivact::ui::Look<CheckableState>) -> Style {
    style()
        .margin_top(2)
        .line_height(1.625)
        .color(t.ext::<Tokens>().muted_foreground)
}

/// The thumb (`size-3`), the track's bar (`h-1`) and the slider's box
/// across the track.
const THUMB: f64 = 12.0;
const BAR: f64 = 4.0;
const ACROSS: f64 = 16.0;

pub(crate) fn slider(r: SliderRecipes) -> SliderRecipes {
    r.track(|_| slider_track)
        .range(|_| slider_range)
        .thumb(|_| slider_thumb)
}

/// `w-full` (vertical: `h-full min-h-40`), 16 px across, with the 4 px
/// `bg-muted` bar in its middle; the whole slider is at half opacity
/// while disabled.
fn slider_track(t: &Theme, s: &kivact::ui::Look<AxisState>) -> Style {
    let fill = t.ext::<Tokens>().muted;
    let vertical = s.orientation == Orientation::Vertical;
    let base = style()
        .shrink(0)
        .radius(0)
        .disabled(style().opacity(0.5))
        .paint(move |d, size| {
            let rect = if vertical {
                Rect::new((size.width - BAR) / 2.0, 0.0, BAR, size.height)
            } else {
                Rect::new(0.0, (size.height - BAR) / 2.0, size.width, BAR)
            };
            d.fill_rect(rect, fill);
        });
    if vertical {
        base.width(ACROSS).height(pct(100)).min_height(160)
    } else {
        base.width(pct(100)).height(ACROSS)
    }
}

/// The `bg-primary` part of the bar.
fn slider_range(t: &Theme, s: &kivact::ui::Look<AxisState>) -> Style {
    let fill = t.ext::<Tokens>().primary;
    let vertical = s.orientation == Orientation::Vertical;
    style().paint(move |d, size| {
        // The range spans the track across its axis; its bar is the track's.
        let rect = if vertical {
            Rect::new((size.width - BAR) / 2.0, 0.0, BAR, size.height)
        } else {
            Rect::new(0.0, (size.height - BAR) / 2.0, size.width, BAR)
        };
        d.fill_rect(rect, fill);
    })
}

/// `size-3 rounded-none border border-ring bg-white`, with `ring-1
/// ring-ring/50` while hovered, pressed or focused. Inside the dimmed
/// track it keeps its own opacity.
fn slider_thumb(t: &Theme, _: &kivact::ui::Look<AxisState>) -> Style {
    let p = t.ext::<Tokens>();
    let halo = style().ring(1, p.ring.with_alpha(0.5));
    style()
        .size(THUMB, THUMB)
        .radius(0)
        .border(1, p.ring)
        .background(Color::WHITE)
        .hover(halo.clone())
        .pressed(halo.clone())
        .focus_visible(halo)
        .disabled(style().opacity(1.0))
}

/// The gap of an open stack and the offset of a toast behind the newest
/// (sonner's `--gap`).
const GAP: f64 = 14.0;
/// How much each toast behind the newest is smaller than the one in front.
const SCALE_STEP: f64 = 0.05;
/// How many toasts show behind the newest in a closed stack (sonner shows
/// three in all).
const BEHIND: usize = 2;

pub(crate) fn toast(r: ToastRecipes) -> ToastRecipes {
    r.region(|_| toast_region)
        .group(|_| toast_group)
        .toast(|_| toast_card)
        .title(|_| toast_title)
        .body(|_| toast_body)
        .icon(|_| toast_icon)
        .close(|_| toast_close)
}

/// The stack at the bottom right of the window, above the page.
fn toast_region(_: &Theme, _: &kivact::ui::Look<PartState>) -> Style {
    style()
        .position(Position::Fixed)
        .z_index(100)
        .right(16)
        .bottom(16)
        .width(384)
        .max_width(pct(100) - 32)
}

/// One live region's stack; the assertive one sits a gap below the polite
/// one while both show toasts.
fn toast_group(_: &Theme, s: &kivact::ui::Look<ToastGroupState>) -> Style {
    let base = style()
        .position(Position::Relative)
        .gap(GAP)
        .width(pct(100));
    if s.count > 0 && s.above > 0 {
        base.margin_top(GAP)
    } else {
        base
    }
}

/// Whether a toast is behind the newest of a closed stack: only a card
/// whose content does not show (`data-front=false`).
fn behind(s: &kivact::ui::Look<ToastState>) -> bool {
    s.depth > 0 && !s.engaged
}

/// `rounded-none border bg-popover shadow-lg p-4 gap-3`. A closed stack
/// (nothing in it hovered or focused) shows the newest toast in front and
/// up to two older ones peeking out above it, each one smaller; it opens
/// into a list while held.
fn toast_card(t: &Theme, s: &kivact::ui::Look<ToastState>) -> Style {
    let p = t.ext::<Tokens>();
    let card = shadow_lg(
        style()
            .direction(Direction::Row)
            .align_items(Align::Center)
            .gap(12)
            .padding(16)
            .radius(0)
            .border(1, p.border)
            .background(p.popover)
            .color(p.popover_foreground)
            .font_size(14),
    );
    if !behind(s) {
        return card.z_index(BEHIND as i32 + 1);
    }
    // Stretched over the newest toast, which sets the group's size, then
    // moved up by `k` gaps and shrunk toward its bottom edge.
    let k = s.depth;
    let scale = 1.0 - SCALE_STEP * k as f64;
    card.position(Position::Absolute)
        .top(0)
        .right(0)
        .bottom(0)
        .left(0)
        .overflow(Overflow::Hidden)
        .z_index((BEHIND + 1).saturating_sub(k) as i32)
        .transform_origin(pct(50), pct(100))
        .translate_y(-GAP * k as f64)
        .scale_xy(scale, scale)
        .opacity(if k > BEHIND { 0.0 } else { 1.0 })
}

/// The content of a toast behind the newest does not show.
fn content(s: &kivact::ui::Look<ToastState>, base: Style) -> Style {
    base.opacity(if behind(s) { 0.0 } else { 1.0 })
}

fn toast_title(t: &Theme, s: &kivact::ui::Look<ToastState>) -> Style {
    content(
        s,
        style()
            .font_size(14)
            .weight(500)
            .line_height(20.0 / 14.0)
            .color(t.ext::<Tokens>().popover_foreground),
    )
}

fn toast_body(t: &Theme, s: &kivact::ui::Look<ToastState>) -> Style {
    content(
        s,
        style()
            .margin_top(4)
            .font_size(14)
            .line_height(20.0 / 14.0)
            .color(t.ext::<Tokens>().muted_foreground),
    )
}

/// The icon of the toast's tone: 16 px in the text color, in the destructive
/// color for a danger.
fn toast_icon(t: &Theme, s: &kivact::ui::Look<ToastState>) -> Style {
    let p = t.ext::<Tokens>();
    let base = style().size(16, 16).shrink(0);
    content(
        s,
        if s.tone == kivact::interaction::Tone::Danger {
            base.color(p.destructive)
        } else {
            base
        },
    )
}

/// `shrink-0 text-muted-foreground after:-inset-2 hover:text-foreground`: a
/// bare 16 px `x` whose hit area reaches 8 px beyond it.
fn toast_close(t: &Theme, s: &kivact::ui::Look<ToastState>) -> Style {
    let p = t.ext::<Tokens>();
    content(
        s,
        style()
            .size(32, 32)
            .margin(-8)
            .direction(Direction::Row)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .shrink(0)
            .radius(0)
            .color(p.muted_foreground)
            .hover(style().color(p.foreground)),
    )
}

/// The footprint of a handle in the flow (its 1 px line) and the hit area
/// around it (the handle's `after:` strip).
const LINE: f64 = 1.0;
const HIT: f64 = 5.0;

pub(crate) fn splitter(r: SplitterRecipes) -> SplitterRecipes {
    r.handle(|_| splitter_handle)
}

/// A 1 px `bg-border` line in a 5 px hit area, in `--ring` while the focus
/// is visible.
///
/// The line is local paint, which a state style cannot carry, so the
/// recipe reads the handle's focus.
fn splitter_handle(t: &Theme, s: &kivact::ui::Look<AxisState>) -> Style {
    let p = t.ext::<Tokens>();
    // Panes in a row are split by a vertical line.
    let stacked = s.orientation == Orientation::Vertical;
    let line = if s.input.focus_visible {
        p.ring
    } else {
        p.border
    };
    let base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .shrink(0)
        .position(Position::Relative)
        .z_index(10)
        .focus_visible(style().ring_none())
        .paint(move |d, size| {
            let rect = if stacked {
                Rect::new(0.0, (size.height - LINE) / 2.0, size.width, LINE)
            } else {
                Rect::new((size.width - LINE) / 2.0, 0.0, LINE, size.height)
            };
            d.fill_rect(rect, line);
        });
    if stacked {
        base.width(pct(100))
            .height(HIT)
            .margin_y(-(HIT - LINE) / 2.0)
            .cursor(Cursor::NsResize)
    } else {
        base.height(pct(100))
            .width(HIT)
            .margin_x(-(HIT - LINE) / 2.0)
            .cursor(Cursor::EwResize)
    }
}

/// A grip on a split handle (`withHandle`): a 4 x 24 bar across the line.
pub(crate) fn split(r: SplitRecipes) -> SplitRecipes {
    r.grip(|_| split_grip)
}

fn split_grip(t: &Theme, s: &Look<AxisState>) -> Style {
    let base = style()
        .shrink(0)
        .radius(0)
        .background(t.ext::<Tokens>().border);
    if s.orientation == Orientation::Vertical {
        base.width(24).height(4)
    } else {
        base.width(4).height(24)
    }
}

/// toggle.tsx: the toggle button of `variant`, muted while pressed.
pub(crate) fn toggle(t: &Theme, look: &Look<ToggleButtonState>) -> Style {
    let variant = look.variant::<ToggleVariant>().copied().unwrap_or_default();
    toggle_style(
        &t.ext::<Tokens>(),
        variant,
        look.size,
        look.pressed,
        look.leading_icon,
        false,
    )
}

/// progress.tsx: a 4 px `bg-muted` track that clips its `bg-primary` fill.
pub(crate) fn progress(r: ProgressRecipes) -> ProgressRecipes {
    r.track(|_| progress_track)
        .fill(|_| progress_fill)
        .label(|_| progress_label)
}

fn progress_track(t: &Theme, state: &Look<ProgressState>) -> Style {
    if state
        .variant::<kivact::ui::ProgressVariant>()
        .is_some_and(|v| *v != kivact::ui::ProgressVariant::Bar)
    {
        return crate::recipes_metrics::track(t, state);
    }
    style()
        .width(pct(100))
        .height(4)
        .min_width(0)
        .background(t.ext::<Tokens>().muted)
        .clip(true)
}

/// The fill is as wide as the value; an indeterminate bar's segment moves.
fn progress_fill(t: &Theme, look: &Look<ProgressState>) -> Style {
    if look
        .variant::<kivact::ui::ProgressVariant>()
        .is_some_and(|v| *v != kivact::ui::ProgressVariant::Bar)
    {
        return crate::recipes_metrics::fill(t, look);
    }
    let (state, primary) = (**look, t.ext::<Tokens>().primary);
    match state.value {
        Some(value) => style()
            .height(pct(100))
            .width(pct(value * 100.0))
            .background(primary),
        None => style()
            .width(pct(100))
            .height(pct(100))
            .paint(move |d, size| {
                let (start, length) = state.sweep();
                d.fill_rect(
                    Rect::new(size.width * start, 0.0, size.width * length, size.height),
                    primary,
                );
            }),
    }
}

/// accordion.tsx: each section ends in a rule, its trigger is
/// `flex items-start justify-between py-2.5 text-xs font-medium hover:underline`
/// and its content `text-xs` with a `pb-2.5` inset.
pub(crate) fn disclosure(r: DisclosureRecipes) -> DisclosureRecipes {
    r.root(|_| disclosure_root)
        .trigger(|_| disclosure_trigger)
        .indicator(|_| disclosure_indicator)
        .content(|_| disclosure_content)
}

fn disclosure_root(t: &Theme, _: &Look<DisclosureRecipeState>) -> Style {
    style()
        .direction(Direction::Column)
        .width(pct(100))
        .border_bottom(1, t.ext::<Tokens>().border)
}

fn disclosure_trigger(t: &Theme, state: &Look<DisclosureRecipeState>) -> Style {
    let p = t.ext::<Tokens>();
    let s = style()
        .direction(Direction::Row)
        .align_items(Align::Start)
        .justify_content(Justify::SpaceBetween)
        .gap(8)
        .padding_y(10)
        .border(1, Color::TRANSPARENT)
        .radius(0)
        .font_size(12)
        .weight(500)
        .color(p.foreground);
    // `hover:underline`: a text decoration, which a state style cannot
    // change, so it follows the trigger's state.
    let s = if state.input.hovered && !state.input.disabled {
        s.text_decoration(TextDecorationLine::UNDERLINE)
    } else {
        s
    };
    with_focus(s, &p)
}

/// The chevron: `chevron-down`, turned up while the section is open.
fn disclosure_indicator(t: &Theme, state: &Look<DisclosureRecipeState>) -> Style {
    style()
        .size(16, 16)
        .shrink(0)
        .color(t.ext::<Tokens>().muted_foreground)
        .rotate(if state.open { 180 } else { 0 })
}

fn disclosure_content(t: &Theme, state: &Look<DisclosureRecipeState>) -> Style {
    if state.variant::<crate::DisclosureVariant>().is_some() {
        return style();
    }
    style()
        .enter_size(kivact::view::Orientation::Vertical)
        .exit_size(kivact::view::Orientation::Vertical)
        .direction(Direction::Column)
        .font_size(12)
        .color(t.ext::<Tokens>().foreground)
        .padding_bottom(10)
}

fn progress_label(t: &Theme, _: &Look<ProgressState>) -> Style {
    style()
        .font_size(12)
        .weight(500)
        .color(t.ext::<Tokens>().foreground)
}
