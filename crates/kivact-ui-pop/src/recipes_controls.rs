//! Role: The Pop looks of the standard controls: buttons, toggles, progress, disclosure
//! and sliders.
//!
//! The check, switch and radio rows are in [`recipes_controls_check`](crate::recipes_controls_check),
//! the text fields in [`recipes_controls_field`](crate::recipes_controls_field).

use crate::color;
use crate::recipe_parts::{button_style, check_style};
use crate::variants::{ButtonTone, Hue, Segments};
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::style::color as core;
use kivact::style::linear_gradient;
use kivact::style::{Align, Color, FlexDirection, Justify, Style, Theme, pct, style};
use kivact::ui::theme::{
    AxisState, ButtonState, ButtonTheme, DisclosureRecipeState, DisclosureTheme, Look,
    ProgressState, ProgressTheme, SliderTheme, SplitTheme, SplitterTheme, ToggleButtonState,
    ToggleButtonTheme,
};
use kivact::ui::{ButtonVariant, ControlSize, ProgressVariant, Slant, Striped};

/// Install the kit's recipes of the controls.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(
            ButtonTheme::default()
                .root(button)
                .content(button_content)
                .busy(button_busy)
                .icon(button_icon),
        )
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
        .with(
            SliderTheme::default()
                .track(slider_track)
                .range(slider_range)
                .thumb(slider_thumb),
        )
        .with(SplitterTheme::default().handle(splitter_handle))
        .with(SplitTheme::default().grip(split_grip));
    let theme = crate::recipes_controls_check::install(theme);
    crate::recipes_controls_field::install(theme)
}

/// The button: the look of its variant, tone and size; an icon button is a square.
fn button(_theme: &Theme, state: &Look<ButtonState>, _base: Style) -> Style {
    let variant = state.variant::<ButtonVariant>().unwrap_or_default();
    let hue = state.variant::<ButtonTone>();
    let s = button_style(variant, hue, state.size, state.variant::<Slant>().is_some());
    if !state.is_icon_only() {
        return s;
    }
    // An icon button is a square of the button's height with a quiet icon.
    let side = match state.size {
        ControlSize::ExtraSmall => 26,
        ControlSize::Small => 32,
        ControlSize::Medium => 36,
        ControlSize::Large => 40,
    };
    let s = s.width(side).padding_x(0).border_radius(8);
    if hue.is_none() && matches!(variant, ButtonVariant::Ghost | ButtonVariant::Link) {
        s.color(color::TEXT_MUTED)
            .hover(style().background(color::SURFACE_ACTIVE).color(color::TEXT))
            .active(style().opacity(0.7))
    } else {
        s
    }
}

/// The content row of a button: the icon and the label, slanted with the button.
fn button_content(_theme: &Theme, s: &Look<ButtonState>, _base: Style) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(8);
    if s.variant::<Slant>().is_some() {
        base.transform(kivact::style::Transform::new().skew_x(12))
    } else {
        base
    }
}

/// The spinner the button turns while busy.
fn button_busy(_theme: &Theme, _: &Look<ButtonState>, _base: Style) -> Style {
    let phase = 0.0;
    let (color, background) = (color::BLUE, color::SURFACE_ACTIVE);
    style().size(16, 16).flex_shrink(0).paint(move |d, size| {
        let arc = |fraction: f64| {
            let mut path = Path::new();
            for i in 0..=64 {
                let a = -std::f64::consts::FRAC_PI_2
                    + std::f64::consts::TAU * (phase + fraction * i as f64 / 64.0);
                let p = Point::new(
                    size.width / 2.0 + 7.0 * a.cos(),
                    size.height / 2.0 + 7.0 * a.sin(),
                );
                path = if i == 0 {
                    path.move_to(p)
                } else {
                    path.line_to(p)
                };
            }
            path
        };
        let stroke = Stroke::new(2.0).cap(LineCap::Round).join(LineJoin::Round);
        d.stroke_path(&arc(1.0), &stroke, background);
        d.stroke_path(&arc(0.25), &stroke, color);
    })
}

/// The icon box of a button: the icon at its own size, centered.
fn button_icon(_theme: &Theme, _: &Look<ButtonState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0)
}

/// A toggle: the outline row, filled in the variant's hue while pressed
/// (default [`Hue::Blue`]).
fn toggle(_theme: &Theme, look: &Look<ToggleButtonState>, _base: Style) -> Style {
    let (fill, text) = look.variant::<Hue>().unwrap_or_default().colors();
    let s = check_style()
        .justify_content(Justify::Center)
        .gap(8)
        .height(32)
        .padding_x(12)
        .border_radius(8)
        .border(1, color::LINE)
        .font_size(13)
        .font_weight(500)
        .hover(style().background(color::SURFACE_ACTIVE));
    if look.pressed {
        s.background(fill).color(text)
    } else {
        s
    }
}

fn progress_label(_theme: &Theme, _: &Look<ProgressState>, _base: Style) -> Style {
    style().font_size(16).font_weight(600)
}

fn progress_track(_theme: &Theme, look: &Look<ProgressState>, _base: Style) -> Style {
    if matches!(
        look.variant::<ProgressVariant>(),
        Some(ProgressVariant::Ring)
    ) {
        let color = color::SURFACE_ACTIVE;
        return style()
            .size(72, 72)
            .position(kivact::style::Position::Relative)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .paint(move |d, size| {
                d.stroke_path(
                    &Path::new().arc(size.to_rect().center(), 27.0, -90.0, 269.99, false),
                    &Stroke::new(6.0).cap(LineCap::Round),
                    color,
                )
            });
    }
    style()
        .width(pct(100))
        .height(8)
        .min_width(0)
        .border_radius(999)
        .clip(true)
        .background(color::SURFACE_ACTIVE)
}

/// The fill: a gradient from the hue's accent, with stripes and segment
/// gaps over it; an indeterminate bar's segment moves.
fn progress_fill(_theme: &Theme, look: &Look<ProgressState>, _base: Style) -> Style {
    let hue = look.variant::<Hue>().unwrap_or_default();
    if matches!(
        look.variant::<ProgressVariant>(),
        Some(ProgressVariant::Ring)
    ) {
        return ring_fill(look, hue);
    }
    let striped = look.variant::<Striped>().is_some();
    let segments: u16 = look.variant::<Segments>().map_or(
        if matches!(
            look.variant::<ProgressVariant>(),
            Some(ProgressVariant::Gauge)
        ) {
            20
        } else {
            0
        },
        |s| s.0,
    );
    let state = **look;
    let accent = hue.accent();
    let end = if hue == Hue::Blue {
        color::CYAN
    } else {
        accent
    };
    let surface = color::SURFACE;
    style()
        .width(pct(100))
        .height(pct(100))
        .paint(move |d, size| {
            let (x, width) = if state.value.is_some() {
                (0.0, size.width * state.shown.get().clamp(0.0, 1.0))
            } else {
                let (start, length) = state.sweep();
                (size.width * start, size.width * length)
            };
            d.rounded_rect(
                Rect::new(x, 0.0, width, size.height),
                size.height / 2.0,
                linear_gradient(90).colors([accent, end]),
            );
            if striped && state.value.is_some() {
                for i in 0..((width / 16.0) as usize + 1) {
                    let x = i as f64 * 16.0;
                    d.fill_path(
                        &Path::new()
                            .move_to(Point::new(x, 0.0))
                            .line_to(Point::new(x + 8.0, 0.0))
                            .line_to(Point::new(x, size.height))
                            .line_to(Point::new(x - 8.0, size.height))
                            .close(),
                        color::PROGRESS_STRIPE,
                    );
                }
            }
            for i in 1..segments {
                let x = size.width * f64::from(i) / f64::from(segments);
                d.fill_rect(Rect::new(x - 1.0, 0.0, 2.0, size.height), surface);
            }
        })
}

fn disclosure_root(_theme: &Theme, _: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .border_bottom(1, core::BORDER)
}

fn disclosure_trigger(_theme: &Theme, _: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(12)
        .padding_y(14)
        .font_weight(500)
}

fn disclosure_indicator(
    _theme: &Theme,
    state: &Look<DisclosureRecipeState>,
    _base: Style,
) -> Style {
    style()
        .size(16, 16)
        .flex_shrink(0)
        .rotate(if state.open { 180 } else { 0 })
}

fn disclosure_content(_theme: &Theme, _: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .enter_size(kivact::view::Orientation::Vertical)
        .exit_size(kivact::view::Orientation::Vertical)
        .flex_direction(FlexDirection::Column)
        .padding_bottom(14)
        .color(core::TEXT_MUTED)
}

fn ring_fill(look: &Look<ProgressState>, hue: Hue) -> Style {
    let state = **look;
    let color = hue.accent();
    style()
        .position(kivact::style::Position::Absolute)
        .left(0)
        .top(0)
        .size(pct(100), pct(100))
        .paint(move |d, size| {
            let (start, length) = if state.value.is_some() {
                (0.0, state.shown.get().clamp(0.0, 1.0))
            } else {
                state.sweep()
            };
            if length > 0.0 {
                d.stroke_path(
                    &Path::new().arc(
                        size.to_rect().center(),
                        27.0,
                        -90.0 + start * 360.0,
                        -90.0 + (start + length.min(0.9999)) * 360.0,
                        false,
                    ),
                    &Stroke::new(6.0).cap(LineCap::Round),
                    color,
                );
            }
        })
}

/// The bar of a track or its filled range: 6 px across the middle of `size`.
fn bar(size: kivact::geometry::Size) -> Rect {
    if size.width >= size.height {
        Rect::new(0.0, (size.height - BAR) / 2.0, size.width, BAR)
    } else {
        Rect::new((size.width - BAR) / 2.0, 0.0, BAR, size.height)
    }
}

/// The height of a slider's track and the side of its thumb.
const TRACK: f64 = 20.0;
const BAR: f64 = 6.0;

fn slider_track(_theme: &Theme, s: &Look<AxisState>, _base: Style) -> Style {
    let color = color::SURFACE_ACTIVE;
    let base = style()
        .flex_shrink(0)
        .cursor(kivact::style::Cursor::Pointer)
        .paint(move |d, size| d.rounded_rect(bar(size), BAR / 2.0, color));
    match s.orientation {
        kivact::view::Orientation::Vertical => base.width(TRACK).height(160),
        _ => base.height(TRACK).min_width(120),
    }
}

fn slider_range(_theme: &Theme, _: &Look<AxisState>, _base: Style) -> Style {
    let color = color::BLUE;
    style().paint(move |d, size| {
        if size.width > 0.0 && size.height > 0.0 {
            d.rounded_rect(bar(size), BAR / 2.0, color);
        }
    })
}

/// A white 16 px thumb ringed in the accent.
fn slider_thumb(_theme: &Theme, _: &Look<AxisState>, _base: Style) -> Style {
    style()
        .size(16, 16)
        .border_radius(8)
        .background(Color::WHITE)
        .border(1, color::BLUE)
}

/// The bar of a splitter: 6 px thick, in the accent while hovered or pressed.
fn splitter_handle(_theme: &Theme, look: &Look<AxisState>, _base: Style) -> Style {
    let s = style()
        .flex_shrink(0)
        .flex_direction(FlexDirection::Row)
        .justify_content(Justify::Center)
        .background(color::LINE)
        .hover(style().background(color::BLUE))
        .active(style().background(color::BLUE));
    match look.orientation {
        kivact::view::Orientation::Vertical => s.height(6).cursor(kivact::style::Cursor::NsResize),
        _ => s.width(6).cursor(kivact::style::Cursor::EwResize),
    }
}

/// The grip of a split: a short rounded bar across the divider.
fn split_grip(_theme: &Theme, look: &Look<AxisState>, _base: Style) -> Style {
    let s = style()
        .flex_shrink(0)
        .align_self(Align::Center)
        .border_radius(2)
        .background(color::LINE_STRONG);
    match look.orientation {
        kivact::view::Orientation::Vertical => s.size(24, 4),
        _ => s.size(4, 24),
    }
}
