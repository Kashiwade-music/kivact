//! Role: Install Astryx Neutral's carousel recipes: the translated track, the scrolling
//! viewport and the edge controls.
use crate::color as c;
use crate::metrics as m;
use crate::radius as r;
use crate::recipe_parts::{fast, shadowed};
use crate::shadows;
use crate::space as s;
use kivact::style::{FlexDirection, Position, ScrollbarWidth, Style, Theme, ThemeMode, pct, style};
use kivact::ui::theme::{CarouselSurfaceState, CarouselTheme, Look};
use kivact::view::{Orientation, ReadingDirection};

/// The root of a carousel: a relative column that shrinks to its content.
fn root(_: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style()
        .position(Position::Relative)
        .flex_direction(FlexDirection::Column)
        .min_width(0)
        .max_width(pct(100))
}

/// The viewport of a carousel: its scrollbar hidden.
fn viewport(_: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style().scrollbar_width(ScrollbarWidth::None).min_width(0)
}

/// The track of a carousel: its slides in a row, or a column when vertical, mirrored in a right-to-left reading direction.
fn track(_: &Theme, look: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style()
        .flex_direction(if look.orientation == Orientation::Vertical {
            FlexDirection::Column
        } else if look.direction == ReadingDirection::Rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .gap(s::SPACE_S1)
        .min_width(pct(100))
        .flex_shrink(0.0)
}

/// A slide of a carousel: it keeps its size in the track.
fn slide(_: &Theme, _: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    style().flex_shrink(0.0)
}

/// The previous control of a carousel, on the leading edge.
fn previous(theme: &Theme, look: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    control(theme, look, false)
}

/// The next control of a carousel, on the trailing edge.
fn next(theme: &Theme, look: &Look<CarouselSurfaceState>, _base: Style) -> Style {
    control(theme, look, true)
}

/// An edge control: a round popover button, centered on the edge, that fades out when disabled.
fn control(theme: &Theme, look: &Look<CarouselSurfaceState>, next: bool) -> Style {
    let size = m::CONTROL_SM;
    let base = fast(
        style()
            .position(Position::Absolute)
            .z_index(1)
            .size(size, size)
            .border_radius(r::RADIUS_FULL)
            .background(c::BG_POPOVER)
            .disabled(style().opacity(0.0)),
    );
    let base = if look.orientation == Orientation::Vertical {
        let base = base.left(pct(50) - size / 2.0);
        if next {
            base.bottom(-size / 2.0)
        } else {
            base.top(-size / 2.0)
        }
    } else {
        let base = base.top(pct(50) - size / 2.0);
        if next != (look.direction == ReadingDirection::Rtl) {
            base.right(-size / 2.0)
        } else {
            base.left(-size / 2.0)
        }
    };
    shadowed(base, shadows::med(theme.mode() == ThemeMode::Dark))
}

/// Install the kit's recipes of the carousel.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        CarouselTheme::default()
            .root(root)
            .viewport(viewport)
            .track(track)
            .slide(slide)
            .previous(previous)
            .next(next),
    )
}
