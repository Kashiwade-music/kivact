//! Role: Pop recipes for the standard toggle, progress bar and disclosure.
//!
//! The Pop look of `kivact::ui::toggle_button` (filled in the variant's [`Hue`]
//! while pressed), `progress` (a gradient fill, optionally striped or cut into
//! [`Segments`]) and `disclosure` (a ruled row with a turning chevron).

use crate::choice::check_style;
use crate::tokens::{Hue, Tokens};
use kivact::draw::{LineCap, Path, Stroke, linear_gradient};
use kivact::geometry::{Point, Rect};
use kivact::state::Value;
use kivact::style::{Align, ColorToken, Direction, Justify, Style, Theme, pct, style};
use kivact::ui::{
    DisclosureRecipeState, DisclosureRecipes, Look, ProgressRecipes, ProgressState,
    ProgressVariant, ToggleButtonState, Variant,
};

/// Draw a standard progress bar with diagonal stripes on its fill:
/// `kivact::ui::progress(..).variant(Striped)`. Another theme draws the
/// plain bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Striped;

impl Variant for Striped {}

/// Cut a standard progress bar into this many segments:
/// `kivact::ui::progress(..).variant(Segments(10))`. Another theme draws the
/// plain bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Segments(pub u16);

impl Variant for Segments {}

impl Variant for Hue {}
/// Reactive Pop progress hue, including ring and gauge appearances.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressHue(pub Value<Hue>);
impl Variant for ProgressHue {}

/// A toggle: the outline row, filled in the variant's hue while pressed
/// (default [`Hue::Blue`]).
pub(crate) fn toggle(t: &Theme, look: &Look<ToggleButtonState>) -> Style {
    let p = t.ext::<Tokens>();
    let (fill, text) = p.hue(look.variant::<Hue>().copied().unwrap_or_default());
    let s = check_style(t)
        .justify_content(Justify::Center)
        .gap(8)
        .height(32)
        .padding_x(12)
        .radius(8)
        .border(1, p.line)
        .font_size(13)
        .weight(500)
        .hover(style().background(p.surface_active));
    if look.pressed {
        s.background(fill).color(text)
    } else {
        s
    }
}

pub(crate) fn progress(r: ProgressRecipes) -> ProgressRecipes {
    r.track(|_| progress_track)
        .fill(|_| progress_fill)
        .label(|_| |_: &Theme, _: &Look<ProgressState>| style().font_size(16).weight(600))
}

fn progress_track(t: &Theme, look: &Look<ProgressState>) -> Style {
    if look.variant::<ProgressVariant>() == Some(&ProgressVariant::Ring) {
        let color = t.ext::<Tokens>().surface_active;
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
        .radius(999)
        .clip(true)
        .background(t.ext::<Tokens>().surface_active)
}

/// The fill: a gradient from the hue's accent, with stripes and segment
/// gaps over it; an indeterminate bar's segment moves.
fn progress_fill(t: &Theme, look: &Look<ProgressState>) -> Style {
    let hue = look.variant::<ProgressHue>().map_or_else(
        || look.variant::<Hue>().copied().unwrap_or_default(),
        |hue| hue.0.get(),
    );
    if look.variant::<ProgressVariant>() == Some(&ProgressVariant::Ring) {
        return ring_fill(t, look, hue);
    }
    let striped = look.variant::<Striped>().is_some();
    let segments = look.variant::<Segments>().map_or(
        if look.variant::<ProgressVariant>() == Some(&ProgressVariant::Gauge) {
            20
        } else {
            0
        },
        |s| s.0,
    );
    let state = **look;
    let p = t.ext::<Tokens>();
    let accent = p.accent(hue);
    let end = if hue == Hue::Blue { p.cyan } else { accent };
    let surface = p.surface;
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
                        crate::tokens::progress_stripe(),
                    );
                }
            }
            for i in 1..segments {
                let x = size.width * f64::from(i) / f64::from(segments);
                d.fill_rect(Rect::new(x - 1.0, 0.0, 2.0, size.height), surface);
            }
        })
}

pub(crate) fn disclosure(r: DisclosureRecipes) -> DisclosureRecipes {
    r.root(|_| disclosure_root)
        .trigger(|_| disclosure_trigger)
        .indicator(|_| disclosure_indicator)
        .content(|_| disclosure_content)
}

fn disclosure_root(_: &Theme, _: &Look<DisclosureRecipeState>) -> Style {
    style()
        .direction(Direction::Column)
        .border_bottom(1, ColorToken::Border)
}

fn disclosure_trigger(_: &Theme, _: &Look<DisclosureRecipeState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(12)
        .padding_y(14)
        .weight(500)
}

fn disclosure_indicator(_: &Theme, state: &Look<DisclosureRecipeState>) -> Style {
    style()
        .size(16, 16)
        .shrink(0)
        .rotate(if state.open { 180 } else { 0 })
}

fn disclosure_content(_: &Theme, _: &Look<DisclosureRecipeState>) -> Style {
    style()
        .enter_size(kivact::view::Orientation::Vertical)
        .exit_size(kivact::view::Orientation::Vertical)
        .direction(Direction::Column)
        .padding_bottom(14)
        .color(ColorToken::TextMuted)
}

fn ring_fill(t: &Theme, look: &Look<ProgressState>, hue: Hue) -> Style {
    let state = **look;
    let color = t.ext::<Tokens>().accent(hue);
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
