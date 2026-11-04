//! Role: Render one-based pagination with Neutral controls and a moving page window.
use crate::stack::rtl;
use crate::tokens;
use kivact::i18n::{message, message_with};
use kivact::motion::Transition;
use kivact::state::IntoBinding;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

use kivact::interaction::ListItem;
use kivact::state::Binding;
use kivact::state::Callback;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::callback;
use kivact::state::effect_on;
use kivact::state::memo;
use kivact::state::provide;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::TransitionProp;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::ui::select;
use kivact::ui::text_input;
use kivact::view::Handled;
use kivact::view::HandlerResult;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::KeyEvent;
use kivact::view::LiveRegion;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::dynamic;
use kivact::view::for_each;
use kivact::view::node_ref;
use kivact::view::show;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaginationVariant {
    #[default]
    Pages,
    Count,
    Compact,
    Dots,
    Input,
    None,
}

pub struct Pagination {
    root: Node,
    page: Binding<u32>,
    page_size: Option<Binding<u32>>,
    size_options: Vec<u32>,
    total_items: Option<Value<u64>>,
    total_pages: Option<Value<u32>>,
    has_more: Value<bool>,
    disabled: Value<bool>,
    variant: Value<PaginationVariant>,
    page_label: Option<Value<String>>,
    size: Value<ControlSize>,
    first_last: Option<Value<bool>>,
    step: Value<u32>,
    siblings: Value<u32>,
    on_change: Option<Callback<u32>>,
}

pub fn pagination(page: impl IntoBinding<u32>) -> Pagination {
    Pagination {
        root: node()
            .role(Role::Navigation)
            .name(message("soft.pagination.label", "Pagination")),
        page: page.into_binding(),
        page_size: None,
        size_options: Vec::new(),
        total_items: None,
        total_pages: None,
        has_more: false.into_value(),
        disabled: false.into_value(),
        variant: (PaginationVariant::Pages).into_value(),
        page_label: None,
        size: (ControlSize::Medium).into_value(),
        first_last: None,
        step: 1.into_value(),
        siblings: 1.into_value(),
        on_change: None,
    }
}

impl Pagination {
    pub fn total_items(mut self, count: impl IntoValue<u64>) -> Self {
        self.total_items = Some(count.into_value());
        self
    }
    pub fn total_pages(mut self, count: impl IntoValue<u32>) -> Self {
        self.total_pages = Some(count.into_value());
        self
    }
    pub fn has_more(mut self, value: impl IntoValue<bool>) -> Self {
        self.has_more = value.into_value();
        self
    }
    pub fn page_size(mut self, size: impl IntoBinding<u32>) -> Self {
        self.page_size = Some(size.into_binding());
        self
    }
    pub fn page_size_options(mut self, values: Vec<u32>) -> Self {
        self.size_options = values;
        self
    }
    pub fn variant(mut self, variant: impl IntoValue<PaginationVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }
    pub fn page_label(mut self, label: impl IntoText) -> Self {
        self.page_label = Some(label.into_text_value());
        self
    }
    pub fn first_last(mut self, first_last: impl IntoValue<bool>) -> Self {
        self.first_last = Some(first_last.into_value());
        self
    }
    pub fn step(mut self, step: impl IntoValue<u32>) -> Self {
        let step = step.into_value();
        self.step = (move || step.get().max(1)).into_value();
        self
    }
    pub fn siblings(mut self, siblings: impl IntoValue<u32>) -> Self {
        self.siblings = siblings.into_value();
        self
    }
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
    pub fn on_change<R: HandlerResult>(mut self, change: impl Fn(u32) -> R + 'static) -> Self {
        self.on_change = Some(callback(change));
        self
    }
}

impl NodeBuilder for Pagination {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[derive(Clone, Copy)]
struct Model {
    page: Binding<u32>,
    total: Value<Option<u32>>,
    has_more: Value<bool>,
    disabled: Value<bool>,
    change: Option<Callback<u32>>,
}

impl Model {
    fn page(self) -> u32 {
        self.page
            .get()
            .clamp(1, self.total.get().unwrap_or(u32::MAX).max(1))
    }

    fn set(self, page: u32) {
        let page = page.clamp(1, self.total.get().unwrap_or(u32::MAX).max(1));
        if !self.disabled.get() && page != self.page() {
            self.page.set(page);
            if let Some(change) = self.change {
                change.call(page);
            }
        }
    }
}

impl IntoView for Pagination {
    fn into_view(self) -> View {
        let Self {
            root,
            page,
            page_size,
            size_options,
            total_items,
            total_pages,
            has_more,
            disabled,
            variant,
            page_label,
            size,
            first_last,
            step,
            siblings,
            on_change,
        } = self;
        provide((), move || {
            let page_size = page_size.unwrap_or_else(|| signal(10).into_binding());
            let total = memo(move || {
                if let Some(items) = total_items {
                    let pages = items.get().div_ceil(u64::from(page_size.get().max(1)));
                    Some(u32::try_from(pages).unwrap_or(u32::MAX))
                } else {
                    total_pages.map(|pages| pages.get())
                }
            });
            let model = Model {
                page,
                total,
                has_more,
                disabled,
                change: on_change,
            };
            let summary = dynamic(
                move || total.get().is_some(),
                move |known| {
                    if known {
                        text(message_with(
                            "soft.pagination.page_of_total",
                            "Page {current} of {total}",
                            move || {
                                [
                                    ("current", model.page().to_string()),
                                    ("total", model.total.get().unwrap_or_default().to_string()),
                                ]
                            },
                        ))
                    } else {
                        text(message_with(
                            "soft.pagination.page_announce",
                            "Page {current}",
                            move || [("current", model.page().to_string())],
                        ))
                    }
                },
            );
            let live = node()
                .role(Role::Status)
                .live_region(LiveRegion::Polite)
                .style(
                    style()
                        .position(Position::Absolute)
                        .size(1, 1)
                        .opacity(0.0)
                        .clip(true),
                )
                .child(summary);
            let controls = show(
                move || total.get() != Some(0),
                move || {
                    let size_selector = if size_options.is_empty() {
                        None
                    } else {
                        let options = size_options
                            .iter()
                            .copied()
                            .filter(|size| *size > 0)
                            .map(|size| ListItem::new(size, size.to_string()))
                            .collect::<Vec<_>>();
                        let selected = binding(
                            move || Some(page_size.get()),
                            move |next: Option<u32>| {
                                if let Some(next) = next.filter(|next| *next != page_size.get()) {
                                    page_size.set(next);
                                    model.set(1);
                                }
                            },
                        );
                        Some(
                            select(
                                message("soft.pagination.items_per_page", "Items per page"),
                                options,
                                selected,
                            )
                            .size(size.get())
                            .disabled(disabled)
                            .style(style().width(80)),
                        )
                    };
                    let show_edges = first_last
                        .map_or(variant.get() == PaginationVariant::Input, |value| {
                            value.get()
                        });
                    let first = show(
                        move || show_edges && total.get().is_some(),
                        move || arrow(model, Arrow::First, size.get(), step),
                    );
                    let last = show(
                        move || show_edges && total.get().is_some(),
                        move || arrow(model, Arrow::Last, size.get(), step),
                    );
                    let center = match variant.get() {
                        PaginationVariant::Pages => for_each(
                            move || {
                                model.total.get().map_or_else(Vec::new, |total| {
                                    page_window(model.page(), total, siblings.get())
                                })
                            },
                            |cell| *cell,
                            move |cell| {
                                let cell = cell.get();
                                match cell {
                                    Cell::Page(page) => {
                                        page_button(model, page, size.get()).into_view()
                                    }
                                    Cell::LeftGap | Cell::RightGap => row(text("…"))
                                        .style(move || {
                                            let t = tokens();
                                            let dimension = dimension(size.get());
                                            style()
                                                .min_width(dimension)
                                                .height(dimension)
                                                .justify_content(Justify::Center)
                                                .align_items(Align::Center)
                                                .font_size(if size.get() == ControlSize::Small {
                                                    12
                                                } else {
                                                    14
                                                })
                                                .color(t.colors.text_secondary)
                                        })
                                        .into_view(),
                                }
                            },
                        ),
                        PaginationVariant::Count if total_items.is_some() => {
                            let label = message_with(
                                "soft.pagination.count",
                                "{from}–{to} of {total}",
                                move || {
                                    let count = total_items.map_or(0, |items| items.get());
                                    let n = u64::from(page_size.get().max(1));
                                    let from = u64::from(model.page() - 1) * n + 1;
                                    let to = u64::from(model.page()) * n;
                                    [
                                        ("from", from.min(count).to_string()),
                                        ("to", to.min(count).to_string()),
                                        ("total", count.to_string()),
                                    ]
                                },
                            );
                            text(label).style(secondary).into_view()
                        }
                        PaginationVariant::Count | PaginationVariant::Compact => dynamic(
                            move || total.get().is_some(),
                            move |known| {
                                let label = if known {
                                    message_with(
                                        "soft.pagination.page_of_total",
                                        "Page {current} of {total}",
                                        move || {
                                            [
                                                ("current", model.page().to_string()),
                                                (
                                                    "total",
                                                    total.get().unwrap_or_default().to_string(),
                                                ),
                                            ]
                                        },
                                    )
                                } else {
                                    message_with(
                                        "soft.pagination.page_announce",
                                        "Page {current}",
                                        move || [("current", model.page().to_string())],
                                    )
                                };
                                text(label).style(secondary)
                            },
                        ),
                        PaginationVariant::Dots => dynamic(
                            move || total.get().unwrap_or_default(),
                            move |total| dots(model, total, size.get()),
                        ),
                        PaginationVariant::Input => dynamic(
                            move || total.get().unwrap_or(0),
                            move |total| {
                                let label = page_label.unwrap_or_else(|| {
                                    message("soft.pagination.page_label", "Page")
                                });
                                let suffix = (total > 0).then(|| {
                                    text(message_with(
                                        "soft.pagination.of_total_pages",
                                        "/ {total}",
                                        move || [("total", total.to_string())],
                                    ))
                                    .style(secondary)
                                });
                                row((
                                    text(label).style(secondary),
                                    page_input(model, total, size.get()),
                                    suffix,
                                ))
                                .style(controls_style)
                            },
                        ),
                        PaginationVariant::None => View::empty(),
                    };
                    let has_selector = size_selector.is_some();
                    row((
                        size_selector,
                        row((
                            first,
                            arrow(model, Arrow::Previous, size.get(), step),
                            center,
                            arrow(model, Arrow::Next, size.get(), step),
                            last,
                        ))
                        .style(controls_style),
                    ))
                    .style(move || {
                        let t = tokens();
                        style()
                            .direction(logical_row())
                            .align_items(Align::Center)
                            .justify_content(if has_selector {
                                Justify::SpaceBetween
                            } else {
                                Justify::End
                            })
                            .gap(t.spacing.s4)
                    })
                },
            );
            root.child((controls, live))
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Cell {
    Page(u32),
    LeftGap,
    RightGap,
}

fn page_window(current: u32, total: u32, siblings: u32) -> Vec<Cell> {
    let adjacent = siblings.saturating_mul(2);
    if total <= 5_u32.saturating_add(adjacent) {
        return (1..=total).map(Cell::Page).collect();
    }
    let left = current.saturating_sub(siblings).max(1);
    let right = current.saturating_add(siblings).min(total);
    let gap_left = left > 2;
    let gap_right = right < total - 1;
    let mut cells = Vec::new();
    if gap_left {
        cells.extend([Cell::Page(1), Cell::LeftGap]);
    }
    let (start, end) = match (gap_left, gap_right) {
        (false, true) => (1, 3 + adjacent),
        (true, false) => (total - 2 - adjacent, total),
        (true, true) => (left, right),
        (false, false) => (1, total),
    };
    cells.extend((start..=end).map(Cell::Page));
    if gap_right {
        cells.extend([Cell::RightGap, Cell::Page(total)]);
    }
    cells
}

fn logical_row() -> Direction {
    if rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    }
}

fn controls_style() -> kivact::style::Style {
    style()
        .direction(logical_row())
        .align_items(Align::Center)
        .gap(tokens().spacing.s1)
}

fn dimension(size: ControlSize) -> f64 {
    let t = tokens();
    if size == ControlSize::Small {
        t.sizes.control_sm
    } else {
        t.sizes.control_md
    }
}

fn secondary() -> kivact::style::Style {
    let t = tokens();
    style()
        .font_size(t.typography.supporting.size_px)
        .line_height(1.4286)
        .color(t.colors.text_secondary)
        .text_wrap(false)
}

fn page_button(model: Model, page: u32, size: ControlSize) -> impl IntoView {
    button(page.to_string())
        .name(message_with(
            "soft.pagination.go_to_page",
            "Go to page {page}",
            move || [("page", page.to_string())],
        ))
        .variant(ButtonVariant::Ghost)
        .size(size)
        .disabled(model.disabled)
        .with_node(move |node| node.selected(move || model.page() == page))
        .current_item(move || (model.page() == page).then_some(kivact::view::CurrentItem::Page))
        .style(move || {
            if model.page() == page {
                style().background(tokens().colors.neutral)
            } else {
                style()
            }
        })
        .on_press(move || model.set(page))
}

#[derive(Clone, Copy)]
enum Arrow {
    First,
    Previous,
    Next,
    Last,
}

fn arrow(model: Model, which: Arrow, size: ControlSize, step: Value<u32>) -> impl IntoView {
    kivact::view::dynamic(step, move |step| arrow_step(model, which, size, step))
}

fn arrow_step(model: Model, which: Arrow, size: ControlSize, step: u32) -> impl IntoView {
    let leading = matches!(which, Arrow::First | Arrow::Previous);
    let edges = matches!(which, Arrow::First | Arrow::Last);
    let label = match which {
        Arrow::First => message("soft.pagination.first", "Go to first page"),
        Arrow::Previous if step == 1 => message("soft.pagination.previous", "Go to previous page"),
        Arrow::Previous => message_with(
            "soft.pagination.previous_by",
            "Go back {step} pages",
            move || [("step", step.to_string())],
        ),
        Arrow::Next if step == 1 => message("soft.pagination.next", "Go to next page"),
        Arrow::Next => message_with(
            "soft.pagination.next_by",
            "Go forward {step} pages",
            move || [("step", step.to_string())],
        ),
        Arrow::Last => message("soft.pagination.last", "Go to last page"),
    };
    let glyph = match (leading != rtl(), edges) {
        (true, true) => "chevrons-left",
        (false, true) => "chevrons-right",
        (true, false) => "chevron-left",
        (false, false) => "chevron-right",
    };
    button(label)
        .variant(ButtonVariant::Ghost)
        .size(size)
        .icon(icon(glyph))
        .hide_label(true)
        .disabled(move || {
            model.disabled.get()
                || if leading {
                    model.page() <= 1
                } else {
                    model
                        .total
                        .get()
                        .map_or(!model.has_more.get(), |total| model.page() >= total)
                }
        })
        .on_press(move || {
            let current = model.page();
            let page = match which {
                Arrow::First => 1,
                Arrow::Previous => current.saturating_sub(step),
                Arrow::Next => current.saturating_add(step),
                Arrow::Last => model.total.get().unwrap_or(current),
            };
            model.set(page);
        })
}

fn dots(model: Model, total: u32, size: ControlSize) -> Node {
    let refs: Rc<[_]> = (0..total).map(|_| node_ref()).collect::<Vec<_>>().into();
    let children = (1..=total)
        .map(|page| {
            let refs = refs.clone();
            let reference = refs[(page - 1) as usize];
            node()
                .role(Role::Button)
                .node_ref(reference)
                .name(message_with(
                    "soft.pagination.go_to_page",
                    "Go to page {page}",
                    move || [("page", page.to_string())],
                ))
                .with_node(move |node| node.selected(move || model.page() == page))
                .current_item(move || {
                    (model.page() == page).then_some(kivact::view::CurrentItem::Page)
                })
                .focusable(move || !model.disabled.get() && model.page() == page)
                .disabled(model.disabled)
                .on_press(move || model.set(page))
                .on_key(move |event: KeyEvent| {
                    if model.disabled.get() {
                        return Handled::No;
                    }
                    let forward = match event.key() {
                        Key::ArrowLeft => rtl(),
                        Key::ArrowRight => !rtl(),
                        _ => return Handled::No,
                    };
                    let target = if forward {
                        if page == total { 1 } else { page + 1 }
                    } else if page == 1 {
                        total
                    } else {
                        page - 1
                    };
                    model.set(target);
                    refs[(target - 1) as usize].focus();
                    Handled::Yes
                })
                .style(move || {
                    let t = tokens();
                    let dot = if size == ControlSize::Small {
                        t.spacing.s1_5
                    } else {
                        t.spacing.s2
                    };
                    style()
                        .size(dot, dot)
                        .radius(t.radius.full)
                        .background(if model.page() == page {
                            t.colors.accent
                        } else {
                            t.colors.neutral
                        })
                        .disabled(style().opacity(t.sizes.disabled_opacity))
                        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
                        .transition(
                            TransitionProp::Paint,
                            Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
                        )
                })
        })
        .collect::<Vec<_>>();
    row(children)
        .role(Role::Group)
        .name(message(
            "soft.pagination.page_indicators",
            "Page indicators",
        ))
        .style(controls_style)
}

fn page_input(model: Model, total: u32, size: ControlSize) -> impl IntoView {
    let reference = node_ref();
    let focused = reference.focused();
    let page = memo(move || model.page());
    let draft = signal(page.get().to_string());
    let commit = move || {
        if let Ok(value) = draft.get().trim().parse::<i128>() {
            model.set(value.clamp(1, i128::from(total.max(1))) as u32);
        }
        draft.set(model.page().to_string());
    };
    effect_on(page, move |page| draft.set(page.to_string()));
    effect_on(focused, move |focused| {
        if !focused {
            commit();
        }
    });
    text_input(
        message("soft.pagination.go_to_page_input", "Go to page"),
        draft,
    )
    .node_ref(reference)
    .disabled(move || total == 0 || model.disabled.get())
    .on_key(move |event: KeyEvent| {
        if event.key() == Key::Enter && !model.disabled.get() && total > 0 {
            commit();
            Handled::Yes
        } else {
            Handled::No
        }
    })
    .style(move || {
        style()
            .width(dimension(size))
            .height(dimension(size))
            .min_width(0)
            .padding_x(3)
    })
}

#[cfg(test)]
mod tests {
    //! Role: Verify page-window boundary and gap rules.
    use super::Cell::{LeftGap, Page, RightGap};
    use super::page_window;

    #[test]
    fn page_window_reserves_gap_slots_and_keeps_edges() {
        assert_eq!(
            page_window(1, 20, 1),
            [
                Page(1),
                Page(2),
                Page(3),
                Page(4),
                Page(5),
                RightGap,
                Page(20)
            ]
        );
        assert_eq!(
            page_window(10, 20, 1),
            [
                Page(1),
                LeftGap,
                Page(9),
                Page(10),
                Page(11),
                RightGap,
                Page(20)
            ]
        );
        assert_eq!(
            page_window(20, 20, 1),
            [
                Page(1),
                LeftGap,
                Page(16),
                Page(17),
                Page(18),
                Page(19),
                Page(20)
            ]
        );
        assert_eq!(page_window(1, 3, 1), [Page(1), Page(2), Page(3)]);
    }
}

kivact::state::constant_value!(PaginationVariant);
