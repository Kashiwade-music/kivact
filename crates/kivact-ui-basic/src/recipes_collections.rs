//! Role: Lyra recipes of the collections: the carousel.
//!
//! The looks of carousel.tsx: a column of slides that glide on a 300 ms
//! cubic-bezier, with 28 px arrows beside the track. The list and data table
//! recipes join this module later.

use crate::color::{BACKGROUND, FOREGROUND, INPUT, MUTED};
use crate::recipe_parts::theme_ring;
use kivact::motion::{Easing, Transition};
use kivact::style::{FlexDirection, Position, Style, Theme, TransitionProp, pct, style};
use kivact::ui::theme::{CarouselSurfaceState, CarouselTheme, Look};
use kivact::view::{Orientation, ReadingDirection};

/// `relative flex-col`, with the theme's focus ring.
fn carousel_root(t: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style()
        .position(Position::Relative)
        .flex_direction(FlexDirection::Column)
        .focus(theme_ring(t))
}

/// `gap-4` between the slides, gliding on a 300 ms cubic-bezier.
fn carousel_track(_: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style().gap(16.0).transition(
        TransitionProp::All,
        Transition::tween(300.0, Easing::CubicBezier(0.4, 0.0, 0.2, 1.0)),
    )
}

fn carousel_viewport(_: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style().flex_direction(FlexDirection::Column)
}

fn carousel_slide(_: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style()
}

/// `absolute size-7 border bg-background`, beside the track.
fn carousel_control(_: &Theme, s: &Look<CarouselSurfaceState>, previous: bool) -> Style {
    let base = style()
        .position(Position::Absolute)
        .size(28.0, 28.0)
        .min_width(28.0)
        .padding(0.0)
        .background(BACKGROUND)
        .color(FOREGROUND)
        .border(1.0, INPUT)
        .hover(style().background(MUTED).color(FOREGROUND));
    if s.orientation == Orientation::Vertical {
        if previous {
            base.top(-48.0).left(pct(50.0) - 14.0)
        } else {
            base.bottom(-48.0).left(pct(50.0) - 14.0)
        }
    } else if previous != (s.direction == ReadingDirection::Rtl) {
        base.left(-48.0).top(pct(50.0) - 14.0)
    } else {
        base.right(-48.0).top(pct(50.0) - 14.0)
    }
}

fn carousel_previous(t: &Theme, s: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    carousel_control(t, s, true)
}

fn carousel_next(t: &Theme, s: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    carousel_control(t, s, false)
}

/// The collection recipes: the carousel.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        CarouselTheme::default()
            .root(carousel_root)
            .viewport(carousel_viewport)
            .track(carousel_track)
            .slide(carousel_slide)
            .previous(carousel_previous)
            .next(carousel_next),
    )
}
