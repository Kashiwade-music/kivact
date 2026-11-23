//! Role: Lyra recipes of the radio, slider, splitter, toggle, progress and disclosure.
//!
//! The looks of radio-group.tsx, slider.tsx, resizable.tsx's handle and grip,
//! toggle.tsx, progress.tsx and accordion.tsx. The button, check and field
//! recipes are in the `recipes_controls_*` modules, installed from here.

use crate::color::{
    BORDER, FOREGROUND, MUTED, MUTED_FOREGROUND, PRIMARY, PRIMARY_FOREGROUND, RING, SWITCH_THUMB,
};
use crate::recipe_parts::{field_fill, toggle_style, wide_focus_ring, with_focus};
use crate::variants::{DisclosureVariant, ToggleVariant};
use kivact::geometry::Rect;
use kivact::style::{
    Align, Cursor, FlexDirection, Justify, Position, Style, TextDecorationLine, Theme, pct, style,
};
use kivact::ui::ProgressVariant;
use kivact::ui::theme::{
    AxisState, CheckableState, DisclosureRecipeState, DisclosureTheme, Look, ProgressState,
    ProgressTheme, RadioGroupTheme, SliderTheme, SplitTheme, SplitterTheme, ToggleButtonState,
    ToggleButtonTheme,
};
use kivact::view::Orientation;

/// `grid w-full gap-2`, or a row with `gap-4` when horizontal.
fn radio_list(_: &Theme, orientation: &Look<Orientation>, _base: Style) -> Style {
    match **orientation {
        Orientation::Horizontal => style()
            .flex_direction(FlexDirection::Row)
            .gap(16.0)
            .flex_shrink(0.0),
        _ => style()
            .flex_direction(FlexDirection::Column)
            .gap(8.0)
            .width(pct(100.0))
            .flex_shrink(0.0),
    }
}

/// A row of the indicator and the label (`text-xs/snug`); the indicator
/// shows the focus, so the row draws no ring of its own.
fn radio_item(_: &Theme, _: &Look<CheckableState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Start)
        .gap(8.0)
        .font_size(12.0)
        .line_height(1.375)
        .color(FOREGROUND)
        .focus_visible(style().outline_none())
}

/// `size-4 rounded-full border border-input dark:bg-input/30`, filled with
/// `primary` while checked around the theme's 8 px `dot` in
/// `primary-foreground`.
///
/// The ring is the item's focus: it is drawn on the indicator, a part
/// inside the focused item, so the recipe reads the item's state.
fn radio_indicator(t: &Theme, s: &Look<CheckableState>, _base: Style) -> Style {
    let dark = t.mode() == kivact::style::ThemeMode::Dark;
    let checked = s.is_checked();
    let base = style()
        .position(Position::Relative)
        .size(16.0, 16.0)
        .flex_shrink(0.0)
        .border_radius(8.0)
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border(
            1.0,
            if checked {
                PRIMARY
            } else {
                crate::color::INPUT
            },
        )
        .color(PRIMARY_FOREGROUND)
        .background(if checked { PRIMARY } else { field_fill(dark) });
    if s.input.is_focus_visible() {
        base.merge(&wide_focus_ring())
    } else {
        base
    }
}

/// `text-xs/relaxed text-muted-foreground` under the label.
fn radio_description(_: &Theme, _: &Look<CheckableState>, _base: Style) -> Style {
    style()
        .margin_top(2.0)
        .line_height(1.625)
        .color(MUTED_FOREGROUND)
}

/// The thumb (`size-3`), the track's bar (`h-1`) and the slider's box
/// across the track.
const THUMB: f64 = 12.0;
const BAR: f64 = 4.0;
const ACROSS: f64 = 16.0;

/// The bar of a track or range along its axis, centered across it.
fn bar_rect(vertical: bool, width: f64, height: f64) -> Rect {
    if vertical {
        Rect::new((width - BAR) / 2.0, 0.0, BAR, height)
    } else {
        Rect::new(0.0, (height - BAR) / 2.0, width, BAR)
    }
}

/// `w-full` (vertical: `h-full min-h-40`), 16 px across, with the 4 px
/// `bg-muted` bar in its middle; the whole slider is at half opacity
/// while disabled.
fn slider_track(_: &Theme, s: &Look<AxisState>, _base: Style) -> Style {
    let vertical = s.orientation == Orientation::Vertical;
    let base = style()
        .flex_shrink(0.0)
        .border_radius(0.0)
        .disabled(style().opacity(0.5))
        .paint(move |d, size| {
            d.fill_rect(bar_rect(vertical, size.width, size.height), MUTED);
        });
    if vertical {
        base.width(ACROSS).height(pct(100.0)).min_height(160.0)
    } else {
        base.width(pct(100.0)).height(ACROSS)
    }
}

/// The `bg-primary` part of the bar.
fn slider_range(_: &Theme, s: &Look<AxisState>, _base: Style) -> Style {
    let vertical = s.orientation == Orientation::Vertical;
    style().paint(move |d, size| {
        // The range spans the track across its axis; its bar is the track's.
        d.fill_rect(bar_rect(vertical, size.width, size.height), PRIMARY);
    })
}

/// `size-3 rounded-none border border-ring bg-white`, with `ring-1
/// ring-ring/50` while hovered, pressed or focused. Inside the dimmed
/// track it keeps its own opacity.
fn slider_thumb(_: &Theme, _: &Look<AxisState>, _base: Style) -> Style {
    let halo = style().outline(1.0, RING.with_alpha(0.5));
    style()
        .width(THUMB)
        .height(THUMB)
        .border_radius(0.0)
        .border(1.0, RING)
        .background(SWITCH_THUMB)
        .hover(halo.clone())
        .active(halo.clone())
        .focus_visible(halo)
        .disabled(style().opacity(1.0))
}

/// A 1 px `bg-border` line in a 5 px hit area, in `--ring` while the focus
/// is visible.
///
/// The line is local paint, which a state style cannot carry, so the
/// recipe reads the handle's focus.
fn splitter_handle(_: &Theme, s: &Look<AxisState>, _base: Style) -> Style {
    /// The footprint of a handle in the flow (its 1 px line) and the hit
    /// area around it (the handle's `after:` strip).
    const LINE: f64 = 1.0;
    const HIT: f64 = 5.0;
    // Panes in a row are split by a vertical line.
    let stacked = s.orientation == Orientation::Vertical;
    let line = if s.input.is_focus_visible() {
        RING
    } else {
        BORDER
    };
    let base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0.0)
        .position(Position::Relative)
        .z_index(10)
        .focus_visible(style().outline_none())
        .paint(move |d, size| {
            let rect = if stacked {
                Rect::new(0.0, (size.height - LINE) / 2.0, size.width, LINE)
            } else {
                Rect::new((size.width - LINE) / 2.0, 0.0, LINE, size.height)
            };
            d.fill_rect(rect, line);
        });
    if stacked {
        base.width(pct(100.0))
            .height(HIT)
            .margin_y(-(HIT - LINE) / 2.0)
            .cursor(Cursor::NsResize)
    } else {
        base.height(pct(100.0))
            .width(HIT)
            .margin_x(-(HIT - LINE) / 2.0)
            .cursor(Cursor::EwResize)
    }
}

/// A grip on a split handle (`withHandle`): a 4 x 24 bar across the line.
fn split_grip(_: &Theme, s: &Look<AxisState>, _base: Style) -> Style {
    let base = style()
        .flex_shrink(0.0)
        .border_radius(0.0)
        .background(BORDER);
    if s.orientation == Orientation::Vertical {
        base.width(24.0).height(4.0)
    } else {
        base.width(4.0).height(24.0)
    }
}

/// toggle.tsx: the toggle button of its variant, muted while pressed.
fn toggle(_: &Theme, look: &Look<ToggleButtonState>, _base: Style) -> Style {
    let variant = look.variant::<ToggleVariant>().unwrap_or_default();
    toggle_style(
        variant == ToggleVariant::Outline,
        look.size,
        look.pressed,
        look.leading_icon,
        false,
    )
}

/// progress.tsx: a 4 px `bg-muted` track that clips its `bg-primary` fill.
/// A ring or another non-bar variant draws its own track.
fn progress_track(t: &Theme, state: &Look<ProgressState>, base: Style) -> Style {
    if state
        .variant::<ProgressVariant>()
        .is_some_and(|v| v != ProgressVariant::Bar)
    {
        return crate::recipes_metrics::track(t, state, base);
    }
    style()
        .width(pct(100.0))
        .height(4.0)
        .min_width(0.0)
        .background(MUTED)
        .clip(true)
}

/// The fill is as wide as the value; an indeterminate bar's segment moves.
/// A ring or another non-bar variant draws its own fill.
fn progress_fill(t: &Theme, look: &Look<ProgressState>, base: Style) -> Style {
    if look
        .variant::<ProgressVariant>()
        .is_some_and(|v| v != ProgressVariant::Bar)
    {
        return crate::recipes_metrics::fill(t, look, base);
    }
    let state = **look;
    match state.value {
        Some(value) => style()
            .height(pct(100.0))
            .width(pct(value * 100.0))
            .background(PRIMARY),
        None => style()
            .width(pct(100.0))
            .height(pct(100.0))
            .paint(move |d, size| {
                let (start, length) = state.sweep();
                d.fill_rect(
                    Rect::new(size.width * start, 0.0, size.width * length, size.height),
                    PRIMARY,
                );
            }),
    }
}

fn progress_label(_: &Theme, _: &Look<ProgressState>, _base: Style) -> Style {
    style().font_size(12.0).font_weight(500).color(FOREGROUND)
}

/// accordion.tsx: each section ends in a rule, its trigger is
/// `flex items-start justify-between py-2.5 text-xs font-medium hover:underline`
/// and its content `text-xs` with a `pb-2.5` inset.
fn disclosure_root(_: &Theme, _: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100.0))
        .border_bottom(1.0, BORDER)
}

fn disclosure_trigger(t: &Theme, state: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    let s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Start)
        .justify_content(Justify::SpaceBetween)
        .gap(8.0)
        .padding_y(10.0)
        .border(1.0, kivact::style::Color::TRANSPARENT)
        .border_radius(0.0)
        .font_size(12.0)
        .font_weight(500)
        .color(FOREGROUND);
    // `hover:underline`: a text decoration, which a state style cannot
    // change, so it follows the trigger's state.
    let s = if state.input.is_hovered() && !state.input.is_disabled() {
        s.text_decoration(TextDecorationLine::UNDERLINE)
    } else {
        s
    };
    with_focus(s, t)
}

/// The chevron: `chevron-down`, turned up while the section is open.
fn disclosure_indicator(_: &Theme, state: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .size(16.0, 16.0)
        .flex_shrink(0.0)
        .color(MUTED_FOREGROUND)
        .rotate(if state.open { 180.0 } else { 0.0 })
}

/// The detail of a trace: a muted column with a rule at its left edge, under
/// the compact trigger of a [`DisclosureVariant::Trace`] disclosure.
fn disclosure_trace(_: &Theme) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .min_width(0.0)
        .margin_top(8.0)
        .margin_bottom(4.0)
        .margin_left(6.0)
        .padding_left(15.0)
        .padding_right(4.0)
        .border_left(1.0, BORDER)
        .line_height(1.625)
        .color(MUTED_FOREGROUND)
        .font_size(12.0)
}

fn disclosure_content(t: &Theme, state: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    match state.variant::<DisclosureVariant>() {
        Some(DisclosureVariant::Plain) => return style(),
        Some(DisclosureVariant::Trace) => return disclosure_trace(t),
        None => (),
    }
    style()
        .enter_size(Orientation::Vertical)
        .exit_size(Orientation::Vertical)
        .flex_direction(FlexDirection::Column)
        .font_size(12.0)
        .color(FOREGROUND)
        .padding_bottom(10.0)
}

/// Installs the controls: the button, the checkbox, the switch, the text
/// fields and the parts of this module.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = crate::recipes_controls_button::install(theme);
    let theme = crate::recipes_controls_check::install(theme);
    let theme = crate::recipes_controls_field::install(theme);
    theme
        .with(
            RadioGroupTheme::default()
                .group(radio_list)
                .item(radio_item)
                .indicator(radio_indicator)
                .description(radio_description),
        )
        .with(
            SliderTheme::default()
                .track(slider_track)
                .range(slider_range)
                .thumb(slider_thumb),
        )
        .with(SplitterTheme::default().handle(splitter_handle))
        .with(SplitTheme::default().grip(split_grip))
        .with(ToggleButtonTheme::default().root(toggle))
        .with(
            ProgressTheme::default()
                .track(progress_track)
                .fill(progress_fill)
                .label(progress_label),
        )
        .with(
            DisclosureTheme::default()
                .root(disclosure_root)
                .trigger(disclosure_trigger)
                .indicator(disclosure_indicator)
                .content(disclosure_content),
        )
}
