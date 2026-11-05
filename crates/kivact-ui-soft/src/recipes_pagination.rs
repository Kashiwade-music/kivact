//! Role: Style canonical page controls and present actual pagination facts.
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Color, Direction, Justify, Position, Style, Theme, auto, style};
use kivact::ui::{
    ControlSize, Look, PageItem, PaginationItemState, PaginationMode, PaginationRecipes,
    PaginationSummary,
};
use kivact::view::{IntoView, LiveRegion, Node, Role, node, text};
/// Reactive visual sizing of standard pagination; mode remains canonical.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaginationAppearance {
    size: Value<ControlSize>,
}
impl PaginationAppearance {
    /// Create a pagination appearance with a reactive standard control size.
    pub fn new(size: impl IntoValue<ControlSize>) -> Self {
        Self {
            size: size.into_value(),
        }
    }
}
impl kivact::ui::Variant for PaginationAppearance {}
fn small<S>(s: &Look<S>) -> bool {
    s.variant::<PaginationAppearance>()
        .is_some_and(|v| v.size.get() == ControlSize::Small)
}
fn direction() -> Direction {
    if crate::stack::rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    }
}
pub(crate) fn pagination(r: PaginationRecipes) -> PaginationRecipes {
    r.root(|_| {
        |_: &Theme, _: &Look<()>| {
            style()
                .direction(direction())
                .align_items(Align::Center)
                .justify_content(Justify::End)
                .gap(16)
                .width(kivact::style::pct(100))
                .min_width(0)
                .padding(0)
        }
    })
    .controls(|_| {
        |_: &Theme, _: &Look<PaginationSummary>| {
            style()
                .direction(direction())
                .align_items(Align::Center)
                .justify_content(Justify::End)
                .gap(4)
        }
    })
    .dots(|_| {
        |_: &Theme, _: &Look<()>| {
            style()
                .direction(direction())
                .align_items(Align::Center)
                .gap(4)
        }
    })
    .sizes(|_| {
        |_: &Theme, _: &Look<PaginationSummary>| {
            let base = style().direction(direction()).width(80).min_width(0);
            if crate::stack::rtl() {
                base.margin_left(auto())
            } else {
                base.margin_right(auto())
            }
        }
    })
    .input(|_| {
        |_: &Theme, _: &Look<PaginationSummary>| {
            style().direction(direction()).width(80).min_width(0)
        }
    })
    .item(|_| item)
    .ellipsis(|_| {
        |t: &Theme, s: &Look<()>| {
            let t = t.ext::<crate::Tokens>();
            style()
                .min_width(if small(s) {
                    t.sizes.control_sm
                } else {
                    t.sizes.control_md
                })
                .height(if small(s) {
                    t.sizes.control_sm
                } else {
                    t.sizes.control_md
                })
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .font_size(if small(s) { 12 } else { 14 })
                .color(t.colors.text_secondary)
        }
    })
    .summary(|_| {
        |t: &Theme, _: &Look<PaginationSummary>| {
            style()
                .direction(direction())
                .align_items(Align::Center)
                .gap(8)
                .font_size(t.ext::<crate::Tokens>().typography.supporting.size_px)
                .line_height(1.4286)
                .color(t.ext::<crate::Tokens>().colors.text_secondary)
                .text_wrap(false)
        }
    })
}
fn item(theme: &Theme, s: &Look<PaginationItemState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let small = small(s);
    let base = style()
        .padding(0)
        .border(0, Color::TRANSPARENT)
        .background(Color::TRANSPARENT)
        .radius(t.radius.element)
        .color(t.colors.text_primary)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
        .disabled(style().opacity(t.sizes.disabled_opacity));
    if s.mode == PaginationMode::Dots && matches!(s.item, PageItem::Page(_)) {
        let side = if small { t.spacing.s1_5 } else { t.spacing.s2 };
        return base
            .size(side, side)
            .min_width(0)
            .font_size(0)
            .radius(t.radius.full)
            .background(if s.current {
                t.colors.accent
            } else {
                t.colors.neutral
            })
            .transition(
                kivact::style::TransitionProp::Paint,
                kivact::motion::Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
            );
    }
    let side = if small {
        t.sizes.control_sm
    } else {
        t.sizes.control_md
    };
    let base = base
        .size(side, side)
        .min_width(side)
        .font_size(if small { 12 } else { 14 })
        .background(if s.current {
            t.colors.neutral
        } else {
            Color::TRANSPARENT
        })
        .hover(style().background(t.colors.overlay_hover))
        .pressed(style().background(t.colors.overlay_pressed));
    if matches!(s.item, PageItem::Page(_)) {
        return base;
    }
    arrow(base, s.item, t.colors.text_primary)
}
fn arrow(base: Style, item: PageItem, color: Color) -> Style {
    let leading = matches!(item, PageItem::First | PageItem::Previous);
    let double = matches!(item, PageItem::First | PageItem::Last);
    let left = leading != crate::stack::rtl();
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
fn page_messages(facts: Value<PaginationSummary>) -> Value<String> {
    let known = kivact::i18n::message_with(
        "pagination.page_of_total",
        "Page {current} of {total}",
        move || {
            [
                ("current", facts.get().page.saturating_add(1).to_string()),
                ("total", facts.get().count.to_string()),
            ]
        },
    );
    let current =
        kivact::i18n::message_with("pagination.page_announce", "Page {current}", move || {
            [("current", facts.get().page.saturating_add(1).to_string())]
        });
    kivact::state::memo(move || {
        if facts.get().count == 0 && facts.get().has_more {
            current.get()
        } else {
            known.get()
        }
    })
}
/// Present a page number and known total from the canonical actual source.
pub fn pagination_page_summary(facts: Value<PaginationSummary>) -> impl IntoView {
    text(page_messages(facts))
}
/// Present the item range when configured, otherwise the actual page summary.
pub fn pagination_count(facts: Value<PaginationSummary>) -> impl IntoView {
    let page = page_messages(facts);
    let range = kivact::i18n::message_with(
        "pagination.item_count",
        "{from}–{to} of {total}",
        move || {
            let facts = facts.get();
            let total = facts.total_items.unwrap_or(0);
            let width = u64::from(facts.page_size);
            let from = (facts.page as u64)
                .saturating_mul(width)
                .saturating_add(1)
                .min(total);
            let to = (facts.page as u64)
                .saturating_add(1)
                .saturating_mul(width)
                .min(total);
            [
                ("from", from.to_string()),
                ("to", to.to_string()),
                ("total", total.to_string()),
            ]
        },
    );
    text(move || {
        if facts.get().total_items.is_some() {
            range.get()
        } else {
            page.get()
        }
    })
}
/// Keep the original polite page notice without another navigation or announcement owner.
pub fn pagination_status(facts: Value<PaginationSummary>) -> Node {
    node()
        .role(Role::Status)
        .live_region(LiveRegion::Polite)
        .style(
            style()
                .position(Position::Absolute)
                .size(1, 1)
                .opacity(0.0)
                .clip(true),
        )
        .child(text(page_messages(facts)))
}
