//! Role: Render stable keyed page controls and an atomic editable page draft.
use kivact_core::state::{Binding, IntoBinding, IntoText, IntoValue, Value, memo};
use kivact_core::view::{
    CurrentItem, IntoView, Node, NodeBuilder, Orientation, Role, View, for_each, node, text,
};
use std::rc::Rc;

/// A cell in a pagination bar; page indices are zero based.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PageItem {
    First,
    Previous,
    Page(usize),
    Ellipsis(usize),
    Next,
    Last,
}
/// Structural pagination presentations, including an editable page number.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaginationMode {
    #[default]
    Pages,
    Count,
    Compact,
    Dots,
    Input,
    None,
}
/// Page controls backed by the application's zero-based page binding.
pub struct Pagination {
    root: Node,
    page: Binding<usize>,
    count: Value<usize>,
    siblings: Value<usize>,
    first_last: Value<bool>,
    step: Value<usize>,
    has_more: Value<bool>,
    total_items: Option<Value<u64>>,
    page_size: Option<Binding<u32>>,
    size_options: Vec<u32>,
    page_label: Value<String>,
    mode: PaginationMode,
    item: Option<Rc<dyn Fn(PageItem) -> View>>,
    format: Rc<dyn Fn(usize) -> String>,
    variants: crate::variants::Variants,
}
/// Construct pagination for `count` pages and a caller-owned page index.
#[track_caller]
pub fn pagination(count: impl IntoValue<usize>, page: impl IntoBinding<usize>) -> Pagination {
    Pagination {
        root: node()
            .role(Role::Navigation)
            .name(kivact_core::i18n::message("pagination.label", "Pagination")),
        page: page.into_binding(),
        count: count.into_value(),
        siblings: 1usize.into_value(),
        first_last: false.into_value(),
        step: 1usize.into_value(),
        has_more: false.into_value(),
        total_items: None,
        page_size: None,
        size_options: Vec::new(),
        page_label: kivact_core::i18n::message("pagination.page", "Page"),
        mode: PaginationMode::Pages,
        item: None,
        format: Rc::new(|n| n.to_string()),
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
        self.total_items = Some(value.into_value());
        self
    }
    /// Bind the number of items per page.
    pub fn page_size(mut self, value: impl IntoBinding<u32>) -> Self {
        self.page_size = Some(value.into_binding());
        self
    }
    /// Offer these page sizes in a select control.
    pub fn page_size_options(mut self, values: Vec<u32>) -> Self {
        self.size_options = values;
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
fn cells(page: usize, count: usize, siblings: usize, ends: bool) -> Vec<PageItem> {
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
            out.push(PageItem::Ellipsis(n));
        }
        out.push(PageItem::Page(n));
        last = Some(n);
    }
    out.push(PageItem::Next);
    if ends {
        out.push(PageItem::Last);
    }
    out
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
            total_items,
            page_size,
            size_options,
            page_label,
            mode,
            item,
            format,
            variants,
        } = self;
        let disabled = root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let count = memo(move || match (total_items, page_size) {
            (Some(total), Some(size)) => total
                .get()
                .div_ceil(u64::from(size.get().max(1)))
                .min(usize::MAX as u64) as usize,
            _ => count.get(),
        });
        let current = memo(move || {
            if has_more.get() {
                page.get()
            } else {
                page.get().min(count.get().saturating_sub(1))
            }
        });
        let child_variants = variants.clone();
        let items = memo(move || {
            let mut cells = cells(current.get(), count.get(), siblings.get(), first_last.get());
            if mode != PaginationMode::Pages && mode != PaginationMode::Dots {
                cells.retain(|c| !matches!(c, PageItem::Page(_) | PageItem::Ellipsis(_)));
            }
            if mode == PaginationMode::None {
                cells.clear();
            }
            cells
        });
        let controls = for_each(
            items,
            |cell| *cell,
            move |cell| {
                let cell = cell.get();
                let label = match cell {
                    PageItem::First => "First page".to_owned(),
                    PageItem::Previous => "Previous page".to_owned(),
                    PageItem::Next => "Next page".to_owned(),
                    PageItem::Last => "Last page".to_owned(),
                    PageItem::Page(n) => format(n + 1),
                    PageItem::Ellipsis(_) => "More pages".to_owned(),
                };
                let body = item.as_ref().map_or_else(
                    || {
                        text(
                            if mode == PaginationMode::Dots && matches!(cell, PageItem::Page(_)) {
                                "•".to_owned()
                            } else {
                                label.clone()
                            },
                        )
                        .into_view()
                    },
                    |slot| slot(cell),
                );
                if matches!(cell, PageItem::Ellipsis(_)) {
                    return crate::layer::with_recipe_style(
                        node().name(label),
                        |r| r.pagination_recipes().ellipsis_recipe(),
                        &child_variants,
                        || (),
                    )
                    .child(body)
                    .into_view();
                }
                let selected =
                    memo(move || matches!(cell, PageItem::Page(n) if n == current.get()));
                crate::layer::with_recipe_style(
                    crate::button("")
                        .name(label)
                        .with_node(|n| n.role(Role::Link))
                        .with_node(|n| n.selected(selected))
                        .with_node(|n| {
                            n.current_item(move || selected.get().then_some(CurrentItem::Page))
                        })
                        .disabled(move || {
                            disabled.get()
                                || match cell {
                                    PageItem::First | PageItem::Previous => {
                                        current.get() == 0 || count.get() == 0
                                    }
                                    PageItem::Next | PageItem::Last => {
                                        !has_more.get()
                                            && current.get().saturating_add(1) >= count.get()
                                    }
                                    _ => false,
                                }
                        })
                        .on_press(move || {
                            let at = current.get_untracked();
                            let last = count.get_untracked().saturating_sub(1);
                            let delta = step.get_untracked().max(1);
                            let next = match cell {
                                PageItem::First => 0,
                                PageItem::Previous => at.saturating_sub(delta),
                                PageItem::Next => at.saturating_add(delta),
                                PageItem::Last => last,
                                PageItem::Page(n) => n,
                                PageItem::Ellipsis(_) => at,
                            };
                            page.set(if has_more.get_untracked() {
                                next
                            } else {
                                next.min(last)
                            });
                        }),
                    |r| r.pagination_recipes().item_recipe(),
                    &child_variants,
                    move || selected.get(),
                )
                .child(body)
                .into_view()
            },
        );
        let summary = if matches!(mode, PaginationMode::Count | PaginationMode::Compact) {
            Some(text(move || format!("{} / {}", current.get() + 1, count.get())).into_view())
        } else {
            None
        };
        let input = (mode == PaginationMode::Input)
            .then(|| crate::pagination_draft::input(page, count, page_label, disabled));
        let sizes = page_size.filter(|_| !size_options.is_empty()).map(|size| {
            crate::select(
                "Items per page",
                size_options
                    .into_iter()
                    .map(|n| kivact_interaction::ListItem::new(n, n.to_string()))
                    .collect::<Vec<_>>(),
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
        });
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
