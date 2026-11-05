//! Role: Style canonical translated and scrolling carousels with Neutral edge controls.
use crate::Tokens;
use kivact::motion::Transition;
use kivact::style::{
    Direction, Position, ScrollbarWidth, Style, Theme, TransitionProp, pct, style,
};
use kivact::ui::{CarouselRecipes, CarouselSurfaceState, Look};
use kivact::view::{Orientation, ReadingDirection};
pub(crate) fn carousel(r: CarouselRecipes) -> CarouselRecipes {
    r.root(|_| {
        |_: &Theme, _: &Look<CarouselSurfaceState>| {
            style()
                .position(Position::Relative)
                .direction(Direction::Column)
                .min_width(0)
                .max_width(pct(100))
        }
    })
    .viewport(|_| {
        |_: &Theme, _: &Look<CarouselSurfaceState>| {
            style().scrollbar_width(ScrollbarWidth::None).min_width(0)
        }
    })
    .track(|_| {
        |theme: &Theme, s: &Look<CarouselSurfaceState>| {
            let t = theme.ext::<Tokens>();
            style()
                .direction(if s.orientation == Orientation::Vertical {
                    Direction::Column
                } else if s.direction == ReadingDirection::Rtl {
                    Direction::RowReverse
                } else {
                    Direction::Row
                })
                .gap(t.spacing.s1)
                .min_width(pct(100))
                .shrink(0)
        }
    })
    .slide(|_| |_: &Theme, _: &Look<CarouselSurfaceState>| style().shrink(0))
    .previous(|_| |theme: &Theme, s: &Look<CarouselSurfaceState>| control(theme, s, false))
    .next(|_| |theme: &Theme, s: &Look<CarouselSurfaceState>| control(theme, s, true))
}
fn control(theme: &Theme, s: &Look<CarouselSurfaceState>, next: bool) -> Style {
    let t = theme.ext::<Tokens>();
    let size = t.sizes.control_sm;
    let base = style()
        .position(Position::Absolute)
        .z_index(1)
        .size(size, size)
        .radius(t.radius.full)
        .background(t.colors.bg_popover)
        .disabled(style().opacity(0.0))
        .transition(
            TransitionProp::Paint,
            Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
        );
    let base = if s.orientation == Orientation::Vertical {
        let base = base.left(pct(50) - size / 2.0);
        if next {
            base.bottom(-size / 2.0)
        } else {
            base.top(-size / 2.0)
        }
    } else {
        let base = base.top(pct(50) - size / 2.0);
        if next != (s.direction == ReadingDirection::Rtl) {
            base.right(-size / 2.0)
        } else {
            base.left(-size / 2.0)
        }
    };
    t.shadows.med.iter().copied().fold(base, Style::shadow)
}
