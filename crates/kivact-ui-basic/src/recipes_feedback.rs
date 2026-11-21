//! Role: Lyra recipes of the feedback parts: toasts, skeletons, spinners and keyboard keys.
//!
//! The looks of sonner.tsx (the stacked toasts), skeleton.tsx, spinner.tsx,
//! kbd.tsx, badge.tsx, empty.tsx and alert.tsx. The avatar recipes are in
//! `recipes_feedback_avatar`.

use crate::color::{
    BORDER, CARD, CARD_FOREGROUND, DESTRUCTIVE, FOREGROUND, MUTED, MUTED_FOREGROUND, POPOVER,
    POPOVER_FOREGROUND, PRIMARY, PRIMARY_FOREGROUND, SECONDARY, SECONDARY_FOREGROUND,
};
use crate::recipe_parts::{shadow_lg, stroke_loader, wide_focus_ring};
use crate::variants::{AlertTone, BadgeVariant, EmptyMediaVariant};
use kivact::custom::PartState;
use kivact::geometry::Point;
use kivact::interaction::{ToastState, Tone};
use kivact::style::{
    Align, Color, Cursor, Display, FlexDirection, Justify, Overflow, Position, Style, TextAlign,
    TextDecorationLine, Theme, ThemeMode, Track, auto, color, pct, style,
};
use kivact::ui::ControlSize;
use kivact::ui::SkeletonShape;
use kivact::ui::theme::{
    AlertState, AlertTheme, BadgeState, BadgeTheme, EmptyStateState, EmptyStateTheme, KbdKeyState,
    KbdTheme, Look, SkeletonState, SkeletonTheme, SpinnerState, SpinnerTheme, ToastGroupState,
    ToastTheme,
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

/// The colors of a badge's `variant`, at rest or while a pressable badge is
/// hovered: the fill, the text and the border.
fn badge_colors(dark: bool, variant: BadgeVariant, hovered: bool) -> (Color, Color, Color) {
    let clear = Color::TRANSPARENT;
    match variant {
        BadgeVariant::Default => (
            PRIMARY.with_alpha(if hovered { 0.8 } else { 1.0 }),
            PRIMARY_FOREGROUND,
            clear,
        ),
        BadgeVariant::Secondary => (
            SECONDARY.with_alpha(if hovered { 0.8 } else { 1.0 }),
            SECONDARY_FOREGROUND,
            clear,
        ),
        BadgeVariant::Destructive => {
            let (base, over) = if dark { (0.2, 0.3) } else { (0.1, 0.2) };
            let alpha = if hovered { over } else { base };
            (DESTRUCTIVE.with_alpha(alpha), DESTRUCTIVE, clear)
        }
        BadgeVariant::Outline => {
            if hovered {
                (MUTED, MUTED_FOREGROUND, BORDER)
            } else {
                (clear, FOREGROUND, BORDER)
            }
        }
        BadgeVariant::Ghost => {
            if hovered {
                (
                    MUTED.with_alpha(if dark { 0.5 } else { 1.0 }),
                    MUTED_FOREGROUND,
                    clear,
                )
            } else {
                (clear, FOREGROUND, clear)
            }
        }
        BadgeVariant::Link => (clear, PRIMARY, clear),
    }
}

/// badge.tsx: the variant's colors, and while pressable the hover colors,
/// the underline of a hovered link (a text decoration, which a state style
/// cannot change, so it follows `input`) and the wide focus ring.
fn badge_style(
    dark: bool,
    variant: BadgeVariant,
    pressable: bool,
    input: &PartState,
    padding: (f64, f64),
) -> Style {
    let (fill, fg, line) = badge_colors(dark, variant, false);
    let mut s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0.0)
        .gap(4.0)
        .height(20.0)
        .padding_left(padding.0)
        .padding_right(padding.1)
        .border_radius(0.0)
        .border(1.0, line)
        .background(fill)
        .color(fg)
        .font_size(12.0)
        .font_weight(500)
        .text_wrap(false)
        .clip(true);
    if variant == BadgeVariant::Link && input.is_hovered() {
        s = s
            .text_decoration(TextDecorationLine::UNDERLINE)
            .text_underline_offset(4.0);
    }
    if !pressable {
        return s;
    }
    let (fill, fg, line) = badge_colors(dark, variant, true);
    let focus = if variant == BadgeVariant::Destructive {
        style()
            .border_color(DESTRUCTIVE.with_alpha(0.4))
            .outline(3.0, DESTRUCTIVE.with_alpha(if dark { 0.4 } else { 0.2 }))
    } else {
        wide_focus_ring()
    };
    s.cursor(Cursor::Pointer)
        .hover(style().background(fill).color(fg).border_color(line))
        .focus_visible(focus)
}

/// The badge's root, with room for a leading or a trailing icon.
fn badge_root(t: &Theme, s: &Look<BadgeState>, _base: Style) -> Style {
    badge_style(
        t.mode() == ThemeMode::Dark,
        s.variant::<BadgeVariant>().unwrap_or_default(),
        s.actionable,
        &s.input,
        (
            if s.leading_icon { 6.0 } else { 8.0 },
            if s.trailing_icon { 6.0 } else { 8.0 },
        ),
    )
}

/// `flex-col` centred in its region, with 24 px of room around it.
fn empty_state_root(_: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(8.0)
        .width(pct(100.0))
        .min_width(0.0)
        .padding(24.0)
        .border_radius(0.0)
        .color(FOREGROUND)
        .font_size(12.0)
        .text_align(TextAlign::Center)
}

/// `text-sm font-medium`, 384 px at most.
fn empty_state_title(_: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style()
        .font_size(14.0)
        .line_height(20.0 / 14.0)
        .font_weight(500)
        .max_width(384.0)
}

/// `text-sm`, muted, 384 px at most.
fn empty_state_description(_: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style()
        .line_height(1.625)
        .color(MUTED_FOREGROUND)
        .max_width(384.0)
}

/// The icon row: 32 px squared on a muted box when the media is an icon.
fn empty_state_icon(_: &Theme, s: &Look<EmptyStateState>, _base: Style) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0.0)
        .margin_bottom(8.0);
    if s.variant::<EmptyMediaVariant>() == Some(EmptyMediaVariant::Icon) {
        base.size(32.0, 32.0).background(MUTED).color(FOREGROUND)
    } else {
        base
    }
}

/// The actions under the message: a column of `gap-2.5`, 384 px at most.
fn empty_state_actions(_: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .align_items(Align::Center)
        .gap(10.0)
        .width(pct(100.0))
        .max_width(384.0)
        .min_width(0.0)
        .font_size(12.0)
        .margin_top(8.0)
}

/// Whether an alert is the destructive one.
fn alert_destructive(s: &Look<AlertState>) -> bool {
    s.variant::<AlertTone>() == Some(AlertTone::Destructive)
}

/// `relative grid` with the icon in its own column, and the tone's color.
fn alert_root(_: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    let columns = if s.has_icon {
        vec![Track::from(auto()), Track::minmax(0, Track::fr(1))]
    } else {
        vec![Track::minmax(0, Track::fr(1))]
    };
    let tint = if alert_destructive(s) {
        DESTRUCTIVE
    } else {
        match s.tone {
            Tone::Neutral => CARD_FOREGROUND,
            Tone::Info | Tone::Accent => color::ACCENT,
            Tone::Success => color::SUCCESS,
            Tone::Warning => color::WARNING,
            Tone::Danger => DESTRUCTIVE,
        }
    };
    style()
        .display(Display::Grid)
        .grid_template_columns(columns)
        .align_items(Align::Start)
        .position(Position::Relative)
        .width(pct(100.0))
        .gap(2.0)
        .padding_left(10.0)
        .padding_y(8.0)
        .padding_right(if s.has_actions { 72.0 } else { 10.0 })
        .border(1.0, BORDER)
        .border_radius(0.0)
        .background(CARD)
        .font_size(12.0)
        .color(tint)
}

fn alert_title(_: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    style()
        .font_weight(500)
        .grid_row(1)
        .grid_column(if s.has_icon { 2 } else { 1 })
}

fn alert_description(_: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    style()
        .line_height(1.625)
        .color(if alert_destructive(s) {
            DESTRUCTIVE.with_alpha(0.9)
        } else {
            MUTED_FOREGROUND
        })
        .grid_row(2)
        .grid_column(if s.has_icon { 2 } else { 1 })
}

fn alert_icon(_: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    style()
        .flex_shrink(0.0)
        .margin_right(6.0)
        .grid_column(1)
        .grid_row((1, if s.has_description { 3 } else { 2 }))
}

/// The actions at the top right, beside the message.
fn alert_actions(_: &Theme, _: &Look<AlertState>, _base: Style) -> Style {
    style()
        .position(Position::Absolute)
        .top(0.0)
        .bottom(0.0)
        .right(5.0)
        .align_items(Align::Center)
}

fn alert_content(_: &Theme, _: &Look<AlertState>, _base: Style) -> Style {
    style().grid_column((1, -1)).grid_row(4)
}

/// Installs the feedback parts of this module, and the avatar parts.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = install_toast(theme);
    // The reduced period of the busy spinner stays with its recipe (`with`
    // replaces the whole table).
    let theme = theme
        .with(SkeletonTheme::default().root(skeleton))
        .with(
            SpinnerTheme::default()
                .root(spinner)
                .reduced_period(Some(1000.0)),
        )
        .with(KbdTheme::default().key(kbd_key))
        .with(BadgeTheme::default().root(badge_root))
        .with(
            EmptyStateTheme::default()
                .root(empty_state_root)
                .title(empty_state_title)
                .description(empty_state_description)
                .icon(empty_state_icon)
                .actions(empty_state_actions),
        )
        .with(
            AlertTheme::default()
                .root(alert_root)
                .title(alert_title)
                .description(alert_description)
                .icon(alert_icon)
                .actions(alert_actions)
                .content(alert_content),
        );
    crate::recipes_feedback_avatar::install(theme)
}
