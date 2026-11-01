//! Role: The Lyra `line` variant of the standard tabs, given with `.variant(..)`.

use crate::theme_access::wide_focus_ring;
use crate::tokens::Tokens;
use kivact::draw::Rect;
use kivact::interaction::{TabListState, TabState};
use kivact::style::{Align, Color, Direction, Justify, Position, Style, Theme, pct, style};
use kivact::ui::Look;
use kivact::view::Orientation;

/// A look of `kivact::ui::tabs` beside the theme's filled one, given with
/// `.variant(..)`.
///
/// `Line` is tabs.tsx's `variant="line"`: no filled list, and a 2 px
/// underline in place of the selected tab's fill (beside the tab when
/// vertical). The tabs' keys, semantics and panels do not change:
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn reports() -> impl IntoView {
///     let current = signal("overview");
///     tabs(
///         "Report sections",
///         vec![ListItem::new("overview", "Overview"), ListItem::new("reports", "Reports")],
///         current,
///     )
///     .variant(basic::TabsVariant::Line)
///     .content(|key| text(key))
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TabsVariant {
    /// The underlined tabs of `variant="line"`.
    Line,
}

impl kivact::ui::Variant for TabsVariant {}

/// Whether `look` belongs to line tabs.
pub(crate) fn is_line<S>(look: &Look<S>) -> bool {
    look.variant::<TabsVariant>() == Some(&TabsVariant::Line)
}

/// `inline-flex w-fit items-center gap-1 p-[3px] h-8 bg-transparent text-muted-foreground`.
pub(crate) fn list(t: &Theme, s: &Look<TabListState>) -> Style {
    let p = t.get::<Tokens>();
    let base = style()
        .align_self(Align::Start)
        .align_items(Align::Center)
        .gap(4)
        .padding(3)
        .radius(0)
        .background(Color::TRANSPARENT)
        .color(p.muted_foreground);
    if s.orientation == Orientation::Vertical {
        base.direction(Direction::Column)
            .align_items(Align::Stretch)
            .justify_content(Justify::Start)
    } else {
        base.direction(Direction::Row)
            .justify_content(Justify::Center)
            .height(32)
    }
}

/// The tab, with the `after:` 2 px bar in `--foreground` while selected:
/// under the tab (5 px below its box), or 4 px beside it when vertical.
pub(crate) fn tab(t: &Theme, s: &Look<TabState>) -> Style {
    let p = t.get::<Tokens>();
    let rest = if p.is_dark() {
        p.muted_foreground
    } else {
        p.foreground.with_alpha(0.6)
    };
    let vertical = s.orientation == Orientation::Vertical;
    let base = style()
        .position(Position::Relative)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(6)
        .padding_x(6)
        .radius(0)
        .border(1, Color::TRANSPARENT)
        .font_size(12)
        .weight(500)
        .text_wrap(false)
        .color(if s.selected { p.foreground } else { rest })
        .hover(style().color(p.foreground))
        // `focus-visible:ring-[3px] ring-ring/50 outline-1 outline-ring`.
        .focus_visible(wide_focus_ring(&p));
    let base = if vertical {
        base.justify_content(Justify::Start).padding_y(5)
    } else {
        base.justify_content(Justify::Center)
            .grow(1)
            .height(pct(100) - 1)
            .padding_y(2)
    };
    if !s.selected {
        return base;
    }
    let bar = p.foreground;
    base.paint(move |d, size| {
        let rect = if vertical {
            Rect::new(size.width + 2.0, 0.0, 2.0, size.height)
        } else {
            Rect::new(0.0, size.height + 3.0, size.width, 2.0)
        };
        d.fill_rect(rect, bar);
    })
}

/// `flex-1 text-xs/relaxed`, after the root's `gap-2`; the theme's focus
/// ring shows while the focus is visible.
pub(crate) fn panel(t: &Theme, s: &Look<TabState>) -> Style {
    let base = style()
        .grow(1)
        .font_size(12)
        .line_height(1.625)
        .color(t.get::<Tokens>().foreground);
    if s.orientation == Orientation::Vertical {
        base.padding_left(8)
    } else {
        base.padding_top(8)
    }
}
