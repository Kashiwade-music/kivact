//! Role: Install Astryx Neutral's pagination recipes: the bar, its controls, its page
//! items, dots, arrows and summary.
use crate::color as c;
use crate::metrics as m;
use crate::motion::FAST;
use crate::radius as r;
use crate::recipe_parts::{ring, tween};
use crate::space as s;
use crate::type_scale as t;
use crate::variants::PaginationSize;
use kivact::style::{Align, Color, FlexDirection, Justify, Style, Theme, auto, pct, style};
use kivact::ui::theme::{Look, PaginationItemState, PaginationSummary, PaginationTheme};
use kivact::ui::{ControlSize, PageItem, PaginationMode};
use kivact::view::{ReadingDirection, use_direction};

/// The flow of the pagination parts: reversed in a right-to-left reading direction.
fn flow() -> FlexDirection {
    if use_direction() == ReadingDirection::Rtl {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    }
}

/// Whether the pagination is at the small control size.
fn small<S>(look: &Look<S>) -> bool {
    look.variant::<PaginationSize>() == Some(PaginationSize(ControlSize::Small))
}

/// The bar: its parts at the end of the row, filling the width.
fn root(_theme: &Theme, _s: &Look<()>, _base: Style) -> Style {
    style()
        .flex_direction(flow())
        .align_items(Align::Center)
        .justify_content(Justify::End)
        .gap(16)
        .width(pct(100))
        .min_width(0)
        .padding(0)
}

/// The controls: the previous, the items and the next, at the end of the row.
fn controls(_theme: &Theme, _s: &Look<PaginationSummary>, _base: Style) -> Style {
    style()
        .flex_direction(flow())
        .align_items(Align::Center)
        .justify_content(Justify::End)
        .gap(4)
}

/// The dots: a centered row of small dots.
fn dots(_theme: &Theme, _s: &Look<()>, _base: Style) -> Style {
    style()
        .flex_direction(flow())
        .align_items(Align::Center)
        .gap(4)
}

/// The page size: a fixed width, pushed to the start of the bar.
fn sizes(_theme: &Theme, _s: &Look<PaginationSummary>, _base: Style) -> Style {
    let base = style().flex_direction(flow()).width(80).min_width(0);
    if use_direction() == ReadingDirection::Rtl {
        base.margin_left(auto())
    } else {
        base.margin_right(auto())
    }
}

/// The page input: a fixed width.
fn input(_theme: &Theme, _s: &Look<PaginationSummary>, _base: Style) -> Style {
    style().flex_direction(flow()).width(80).min_width(0)
}

/// One item: a centered control with the hover and pressed fills. A dot is a small round control, and an
/// arrow draws its chevron, doubled for the first and last.
fn item(_theme: &Theme, s: &Look<PaginationItemState>, _base: Style) -> Style {
    let small = small(s);
    let base = style()
        .padding(0)
        .border(0.0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .border_radius(r::RADIUS_ELEMENT)
        .color(c::TEXT_PRIMARY)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .focus_visible(ring(c::ACCENT))
        .disabled(style().opacity(m::DISABLED_OPACITY));
    if s.mode == PaginationMode::Dots && matches!(s.item, PageItem::Page(_)) {
        let side = if small { s::SPACE_S1_5 } else { s::SPACE_S2 };
        let dot = base
            .size(side, side)
            .min_width(0)
            .font_size(0)
            .border_radius(r::RADIUS_FULL)
            .background(if s.current { c::ACCENT } else { c::NEUTRAL });
        return tween(dot, FAST);
    }
    let side = if small { m::CONTROL_SM } else { m::CONTROL_MD };
    let base = base
        .size(side, side)
        .min_width(side)
        .font_size(if small { t::SIZE_SM } else { t::SIZE_MD })
        .background(if s.current {
            c::NEUTRAL
        } else {
            Color::TRANSPARENT
        })
        .hover(style().background(c::OVERLAY_HOVER))
        .active(style().background(c::OVERLAY_PRESSED));
    if matches!(s.item, PageItem::Page(_)) {
        return base;
    }
    arrow(base, s.item, c::TEXT_PRIMARY)
}

/// An arrow item: a chevron pointing back or forward in the reading direction, doubled for the ends.
fn arrow(base: Style, item: PageItem, color: Color) -> Style {
    let leading = matches!(item, PageItem::First | PageItem::Previous);
    let double = matches!(item, PageItem::First | PageItem::Last);
    let left = leading != (use_direction() == ReadingDirection::Rtl);
    base.font_size(0).paint(move |draw, size| {
        use kivact::draw::{Path, Stroke};
        use kivact::geometry::Point;
        let x = size.width / 2.0;
        let y = size.height / 2.0;
        let a = if left { 3.0 } else { -3.0 };
        let b = -a;
        let path = Path::new()
            .move_to(Point::new(x + a, y - 6.0))
            .line_to(Point::new(x + b, y))
            .line_to(Point::new(x + a, y + 6.0));
        draw.stroke_path(&path, &Stroke::new(1.5), color);
        if double {
            let shift = if left { -4.0 } else { 4.0 };
            draw.stroke_path(
                &Path::new()
                    .move_to(Point::new(x + a + shift, y - 6.0))
                    .line_to(Point::new(x + b + shift, y))
                    .line_to(Point::new(x + a + shift, y + 6.0)),
                &Stroke::new(1.5),
                color,
            );
        }
    })
}

/// An ellipsis: a centered placeholder at the control size, in the secondary text color.
fn ellipsis(_theme: &Theme, s: &Look<()>, _base: Style) -> Style {
    let small = small(s);
    let side = if small { m::CONTROL_SM } else { m::CONTROL_MD };
    style()
        .min_width(side)
        .height(side)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .font_size(if small { t::SIZE_SM } else { t::SIZE_MD })
        .color(c::TEXT_SECONDARY)
}

/// The summary: the page text in the supporting size, in the secondary text color.
fn summary(_theme: &Theme, _s: &Look<PaginationSummary>, _base: Style) -> Style {
    style()
        .flex_direction(flow())
        .align_items(Align::Center)
        .gap(8)
        .font_size(t::SUPPORTING.size_px)
        .line_height(1.4286)
        .color(c::TEXT_SECONDARY)
        .text_wrap(false)
}

/// Install the kit's recipes of the pagination.
pub(crate) fn install(theme: Theme) -> Theme {
    theme.with(
        PaginationTheme::default()
            .root(root)
            .controls(controls)
            .dots(dots)
            .sizes(sizes)
            .input(input)
            .item(item)
            .ellipsis(ellipsis)
            .summary(summary),
    )
}
