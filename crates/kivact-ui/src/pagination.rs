//! Role: Render stable keyed page controls and an atomic editable page draft.
use kivact_core::state::{Binding, IntoBinding, IntoText, IntoValue, Value, memo};
use kivact_core::view::{
    CurrentItem, IntoView, Node, NodeBuilder, Orientation, Role, View, for_each, node, text,
};
use std::rc::Rc;

use crate::{PageItem, PaginationMode, PaginationSummary};
/// Page controls backed by the application's zero-based page binding.
pub struct Pagination {
    root: Node,
    page: Binding<usize>,
    count: PageCount,
    siblings: Value<usize>,
    // xtask-allow(state-fields): Independent reactive construction option controlling endpoint visibility.
    first_last: Value<bool>,
    step: Value<usize>,
    // xtask-allow(state-fields): Caller-owned information about an unfinished remote page count.
    has_more: Value<bool>,
    // xtask-allow(state-fields): Optional caller binding for page-size selection, independent of page draft state.
    page_size: Option<Binding<u32>>,
    size_options: Value<Vec<u32>>,
    page_label: Value<String>,
    mode: PaginationMode,
    // xtask-allow(state-fields): Optional caller-provided presentation slot selected only during construction.
    item: Option<PageSlot>,
    format: Rc<dyn Fn(usize) -> String>,
    // xtask-allow(state-fields): Immutable optional presentation factory over the canonical derived page facts.
    summary: Option<SummarySlot>,
    variants: crate::variants::Variants,
}
enum PageCount {
    Pages(Value<usize>),
    Items(Value<u64>),
}
/// Construct pagination for `count` pages and a caller-owned page index.
#[track_caller]
pub fn pagination(count: impl IntoValue<usize>, page: impl IntoBinding<usize>) -> Pagination {
    Pagination {
        root: node()
            .role(Role::Navigation)
            .name(kivact_core::i18n::message("pagination.label", "Pagination")),
        page: page.into_binding(),
        count: PageCount::Pages(count.into_value()),
        siblings: 1usize.into_value(),
        first_last: false.into_value(),
        step: 1usize.into_value(),
        has_more: false.into_value(),
        page_size: None,
        size_options: Vec::<u32>::new().into_value(),
        page_label: kivact_core::i18n::message("pagination.page", "Page"),
        mode: PaginationMode::Pages,
        item: None,
        format: Rc::new(|n| n.to_string()),
        summary: None,
        variants: crate::variants::Variants::default(),
    }
}
impl Pagination {
    /// Neighbours shown on either side of the current page.
    pub fn siblings(mut self, value: impl IntoValue<usize>) -> Self {
        self.siblings = value.into_value();
        self
    }
    /// Show explicit first and last controls.
    pub fn first_last(mut self, value: impl IntoValue<bool>) -> Self {
        self.first_last = value.into_value();
        self
    }
    /// Pages moved by previous and next controls.
    pub fn step(mut self, value: impl IntoValue<usize>) -> Self {
        self.step = value.into_value();
        self
    }
    /// Permit forward navigation when the total is not yet known.
    pub fn has_more(mut self, value: impl IntoValue<bool>) -> Self {
        self.has_more = value.into_value();
        self
    }
    /// Derive page count from an item total and the configured page size.
    pub fn total_items(mut self, value: impl IntoValue<u64>) -> Self {
        self.count = PageCount::Items(value.into_value());
        self
    }
    /// Bind the number of items per page.
    pub fn page_size(mut self, value: impl IntoBinding<u32>) -> Self {
        self.page_size = Some(value.into_binding());
        self
    }
    /// Offer these page sizes in a select control.
    pub fn page_size_options(mut self, values: impl IntoValue<Vec<u32>>) -> Self {
        self.size_options = values.into_value();
        self
    }
    /// Name the editable page field.
    pub fn page_label(mut self, value: impl IntoText) -> Self {
        self.page_label = value.into_text_value();
        self
    }
    /// Select the structural presentation at construction time.
    pub fn mode(mut self, mode: PaginationMode) -> Self {
        self.mode = mode;
        self
    }
    /// Replace the visible contents of every cell; behavior stays on the cell.
    pub fn item<V: IntoView>(mut self, f: impl Fn(PageItem) -> V + 'static) -> Self {
        self.item = Some(Rc::new(move |item| f(item).into_view()));
        self
    }
    /// Format one-based visible page numbers.
    pub fn label_format(mut self, f: impl Fn(usize) -> String + 'static) -> Self {
        self.format = Rc::new(f);
        self
    }
    /// Replace summary presentation using actual zero-based page and total inputs.
    pub fn summary<V: IntoView>(
        mut self,
        summary: impl Fn(Value<PaginationSummary>) -> V + 'static,
    ) -> Self {
        self.summary = Some(Rc::new(move |facts| summary(facts).into_view()));
        self
    }
    /// Add a kit-defined visual variant.
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for Pagination {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
fn cells(page: usize, count: usize, siblings: usize, ends: bool) -> Vec<(usize, PageItem)> {
    let mut out = Vec::new();
    if ends {
        out.push(PageItem::First);
    }
    out.push(PageItem::Previous);
    let mut shown = vec![0, count.saturating_sub(1)];
    shown.extend(
        page.saturating_sub(siblings)..=page.saturating_add(siblings).min(count.saturating_sub(1)),
    );
    shown.retain(|n| *n < count);
    shown.sort_unstable();
    shown.dedup();
    let mut last = None;
    for n in shown {
        if last.is_some_and(|before| n > before + 1) {
            out.push(PageItem::Ellipsis);
        }
        out.push(PageItem::Page(n));
        last = Some(n);
    }
    out.push(PageItem::Next);
    if ends {
        out.push(PageItem::Last);
    }
    out.into_iter()
        .enumerate()
        .map(|(at, item)| (if item == PageItem::Ellipsis { at } else { 0 }, item))
        .collect()
}
impl IntoView for Pagination {
    fn into_view(self) -> View {
        let Self {
            mut root,
            page,
            count,
            siblings,
            first_last,
            step,
            has_more,
            page_size,
            size_options,
            page_label,
            mode,
            item,
            format,
            summary,
            variants,
        } = self;
        let disabled = root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let total_items = match &count {
            PageCount::Items(value) => Some(*value),
            PageCount::Pages(_) => None,
        };
        let count = page_count(count, page_size);
        let current = memo(move || {
            if has_more.get() {
                page.get()
            } else {
                page.get().min(count.get().saturating_sub(1))
            }
        });
        let items = visible_cells(current, count, siblings, first_last, mode);
        let options = PageOptions {
            step,
            has_more,
            item,
            format,
            mode,
            variants: variants.clone(),
        };
        let controls = for_each(
            items,
            |cell| *cell,
            move |cell| page_control(cell.get().1, page, current, count, disabled, &options),
        );
        let facts = summary_source(total_items, page_size, current, count, has_more);
        let summary = render_summary(summary, mode, facts, &variants);
        let input = (mode == PaginationMode::Input)
            .then(|| crate::pagination_draft::input(page, count, page_label, disabled));
        let sizes = page_size.map(|size| size_selector(size, size_options, page));
        crate::layer::with_base_recipe_style(
            root.roving_focus(Orientation::Horizontal),
            |r| r.pagination_recipes().root_recipe(),
            &variants,
            || (),
        )
        .child(controls)
        .child(summary)
        .child(input)
        .child(sizes)
        .into_view()
    }
}

/// Application-provided presentation for a pagination cell.
type PageSlot = Rc<dyn Fn(PageItem) -> View>;
type SummarySlot = Rc<dyn Fn(Value<PaginationSummary>) -> View>;

/// Immutable presentation and navigation inputs shared by mounted page cells.
struct PageOptions {
    step: Value<usize>,
    has_more: Value<bool>,
    item: Option<PageSlot>,
    format: Rc<dyn Fn(usize) -> String>,
    mode: PaginationMode,
    variants: crate::variants::Variants,
}
fn page_control(
    cell: PageItem,
    page: Binding<usize>,
    current: Value<usize>,
    count: Value<usize>,
    disabled: Value<bool>,
    options: &PageOptions,
) -> View {
    let (label, body) = page_content(cell, options);
    if matches!(cell, PageItem::Ellipsis) {
        return crate::layer::with_recipe_style(
            node().name(label),
            |r| r.pagination_recipes().ellipsis_recipe(),
            &options.variants,
            || (),
        )
        .child(body)
        .into_view();
    }
    let has_more = options.has_more;
    let step = options.step;
    let selected = memo(move || matches!(cell, PageItem::Page(n) if n == current.get()));
    let mode = options.mode;
    crate::layer::with_recipe_style(
        crate::button("")
            .name(label)
            .with_node(|n| n.role(Role::Button))
            .with_node(|n| n.selected(selected))
            .with_node(|n| n.current_item(move || selected.get().then_some(CurrentItem::Page)))
            .disabled(move || {
                disabled.get()
                    || match cell {
                        PageItem::First | PageItem::Previous => {
                            current.get() == 0 || count.get() == 0
                        }
                        PageItem::Next | PageItem::Last => {
                            !has_more.get() && current.get().saturating_add(1) >= count.get()
                        }
                        _ => false,
                    }
            })
            .on_press(move || {
                let at = current.get_untracked();
                let last = count.get_untracked().saturating_sub(1);
                let delta = step.get_untracked().max(1);
                let next = target_page(cell, at, last, delta);
                page.set(if has_more.get_untracked() {
                    next
                } else {
                    next.min(last)
                });
            }),
        |r| r.pagination_recipes().item_recipe(),
        &options.variants,
        move || crate::PaginationItemState {
            current: selected.get(),
            item: cell,
            mode,
        },
    )
    .child(body)
    .into_view()
}
fn target_page(cell: PageItem, at: usize, last: usize, delta: usize) -> usize {
    match cell {
        PageItem::First => 0,
        PageItem::Previous => at.saturating_sub(delta),
        PageItem::Next => at.saturating_add(delta),
        PageItem::Last => last,
        PageItem::Page(n) => n,
        PageItem::Ellipsis => at,
    }
}
fn visible_cells(
    current: Value<usize>,
    count: Value<usize>,
    siblings: Value<usize>,
    first_last: Value<bool>,
    mode: PaginationMode,
) -> Value<Vec<(usize, PageItem)>> {
    memo(move || {
        let mut cells = cells(current.get(), count.get(), siblings.get(), first_last.get());
        if mode != PaginationMode::Pages && mode != PaginationMode::Dots {
            cells.retain(|(_, c)| !matches!(c, PageItem::Page(_) | PageItem::Ellipsis));
        }
        if mode == PaginationMode::None {
            cells.clear();
        }
        cells
    })
}
fn page_count(count: PageCount, page_size: Option<Binding<u32>>) -> Value<usize> {
    memo(move || match &count {
        PageCount::Items(total) => total
            .get()
            .div_ceil(u64::from(page_size.map_or(1, |size| size.get().max(1))))
            .min(usize::MAX as u64) as usize,
        PageCount::Pages(count) => count.get(),
    })
}
fn size_selector(size: Binding<u32>, size_options: Value<Vec<u32>>, page: Binding<usize>) -> View {
    crate::select(
        "Items per page",
        move || {
            size_options
                .get()
                .into_iter()
                .filter(|n| *n > 0)
                .map(|n| kivact_interaction::ListItem::new(n, n.to_string()))
                .collect::<Vec<_>>()
        },
        kivact_core::state::binding(
            move || Some(size.get()),
            move |next| {
                if let Some(next) = next {
                    size.set(next);
                    page.set(0);
                }
            },
        ),
    )
    .into_view()
}

fn page_content(cell: PageItem, options: &PageOptions) -> (String, View) {
    let label = match cell {
        PageItem::First => "First page".to_owned(),
        PageItem::Previous => "Previous page".to_owned(),
        PageItem::Next => "Next page".to_owned(),
        PageItem::Last => "Last page".to_owned(),
        PageItem::Page(n) => (options.format)(n + 1),
        PageItem::Ellipsis => "More pages".to_owned(),
    };
    let body = options.item.as_ref().map_or_else(
        || {
            text(
                if options.mode == PaginationMode::Dots && matches!(cell, PageItem::Page(_)) {
                    "•".to_owned()
                } else {
                    label.clone()
                },
            )
            .into_view()
        },
        |slot| slot(cell),
    );
    (label, body)
}

fn summary_source(
    total: Option<Value<u64>>,
    size: Option<Binding<u32>>,
    page: Value<usize>,
    count: Value<usize>,
    has_more: Value<bool>,
) -> Value<PaginationSummary> {
    memo(move || PaginationSummary {
        page: page.get(),
        count: count.get(),
        total_items: total.map(|total| total.get()),
        page_size: size.map_or(1, |size| size.get().max(1)),
        has_more: has_more.get(),
    })
}
fn render_summary(
    slot: Option<SummarySlot>,
    mode: PaginationMode,
    facts: Value<PaginationSummary>,
    variants: &crate::variants::Variants,
) -> Option<View> {
    let content = if let Some(slot) = slot {
        slot(facts)
    } else if matches!(mode, PaginationMode::Count | PaginationMode::Compact) {
        text(move || format!("{} / {}", facts.get().page + 1, facts.get().count)).into_view()
    } else {
        return None;
    };
    Some(
        crate::layer::with_recipe_style(
            node().child(content),
            |r| r.pagination_recipes().summary_recipe(),
            variants,
            move || facts.get(),
        )
        .into_view(),
    )
}
