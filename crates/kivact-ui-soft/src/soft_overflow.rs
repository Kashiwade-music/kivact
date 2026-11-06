//! Role: Width-bounded keyed overflow lists.

use kivact::state::ItemKey;
use kivact::state::PropValue;
use std::collections::HashMap;
use std::rc::Rc;

use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::node;
use kivact::view::text;

use kivact::state::Callback;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::callback;
use kivact::state::effect_on;
use kivact::state::memo;
use kivact::state::{Machine, MachineHandle, Step, machine};
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Overflow;
use kivact::style::style;
use kivact::view::HandlerResult;
use kivact::view::IntoView;
use kivact::view::NodeRef;
use kivact::view::View;
use kivact::view::for_each;
use kivact::view::node_ref;
use kivact::view::show;
use std::collections::HashSet;
use std::marker::PhantomData;

/// The cache holds only actual measured layout facts.
struct WidthMeasurements<K>(PhantomData<K>);
enum Measurement<K> {
    Measured(K, f64),
    Retain(HashSet<K>),
}
impl<K: ItemKey> Machine for WidthMeasurements<K> {
    type State = HashMap<K, f64>;
    type Event = Measurement<K>;
    type Effect = ();
    fn transition(
        &self,
        state: &Self::State,
        event: Self::Event,
    ) -> Step<Self::State, Self::Event, ()> {
        let mut next = state.clone();
        match event {
            Measurement::Measured(key, width) => {
                if state.get(&key) == Some(&width) {
                    return Step::stay();
                }
                next.insert(key, width);
            }
            Measurement::Retain(keys) => {
                next.retain(|key, _| keys.contains(key));
                if next.len() == state.len() {
                    return Step::stay();
                }
            }
        }
        Step::to(next)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverflowFit {
    Measuring,
    Fitted { visible: usize },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Logical edge from which items move into the overflow indicator.
pub enum CollapseFrom {
    /// Hide items from the logical start first.
    Start,
    #[default]
    /// Hide items from the logical end first.
    End,
}

#[derive(Clone, Default, PartialEq)]
/// Element whose width determines the available overflow space.
pub enum OverflowMeasure {
    #[default]
    /// Use the overflow list own measured width.
    OwnWidth,
    /// Measure against the caller's reference to the containing element.
    ParentWidth(NodeRef),
}

type ItemRenderer<T> = kivact::view::ViewFn<Value<T>>;
type ItemWidth<T> = Box<dyn Fn(&T) -> f64>;
type OverflowRenderer<T> = kivact::view::ViewFn<Value<Vec<T>>>;

/// A keyed list that fits visible items and exposes the remaining items through an indicator.
pub struct OverflowList<T: PropValue, K: ItemKey> {
    root: Node,
    items: Value<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    render: ItemRenderer<T>,
    item_width: Option<ItemWidth<T>>,
    indicator_width: Box<dyn Fn(usize) -> f64>,
    indicator: OverflowRenderer<T>,
    on_overflow_change: Option<Callback<Vec<K>>>,
    measure: Value<OverflowMeasure>,
    gap: Value<f64>,
    min_visible: Value<usize>,
    max_visible: Value<usize>,
    max_rows: Value<usize>,
    collapse_from: Value<CollapseFrom>,
}

/// Build an overflow-aware list with stable keys and a reactive renderer for each item.
pub fn overflow_list<T, K, V>(
    items: impl IntoValue<Vec<T>>,
    key: impl Fn(&T) -> K + 'static,
    item: impl Fn(Value<T>) -> V + 'static,
) -> OverflowList<T, K>
where
    T: PropValue,
    K: ItemKey,
    V: IntoView,
{
    OverflowList {
        root: node(),
        items: items.into_value(),
        key: Rc::new(key),
        render: item.into(),
        item_width: None,
        indicator_width: Box::new(|count| 18.0 + count.to_string().len() as f64 * 8.0),
        indicator: kivact::view::ViewFn::from(|hidden: Value<Vec<T>>| {
            text(move || format!("+{}", hidden.with(Vec::len)))
                .style(move || {
                    style().padding_x(5).font_size(12).color(
                        kivact::style::use_tokens::<crate::Tokens>()
                            .colors
                            .text_secondary,
                    )
                })
                .into_view()
        }),
        on_overflow_change: None,
        measure: (OverflowMeasure::OwnWidth).into_value(),
        gap: (8.0).into_value(),
        min_visible: 0.into_value(),
        max_visible: usize::MAX.into_value(),
        max_rows: 1.into_value(),
        collapse_from: (CollapseFrom::End).into_value(),
    }
}

impl<T: PropValue, K: ItemKey> NodeBuilder for OverflowList<T, K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<T: PropValue, K: ItemKey> OverflowList<T, K> {
    /// Provide item widths up front. By default, mounted items are measured and
    /// a provisional width of 80 px is used until each measurement arrives.
    pub fn item_width(mut self, width: impl Fn(&T) -> f64 + 'static) -> Self {
        self.item_width = Some(Box::new(width));
        self
    }

    /// Reserve this much horizontal space for a custom indicator.
    pub fn indicator_width(mut self, width: impl Fn(usize) -> f64 + 'static) -> Self {
        self.indicator_width = Box::new(width);
        self
    }

    /// Render the overflow indicator from the reactive collection of hidden items.
    pub fn overflow<V: IntoView>(mut self, render: impl Fn(Value<Vec<T>>) -> V + 'static) -> Self {
        self.indicator = render.into();
        self
    }

    /// Set spacing between visible items and the overflow indicator.
    pub fn gap(mut self, gap: impl IntoValue<f64>) -> Self {
        let gap = gap.into_value();
        self.gap = (move || gap.get().max(0.0)).into_value();
        self
    }

    /// Keep at least this many items visible when calculating overflow.
    pub fn min_visible(mut self, count: impl IntoValue<usize>) -> Self {
        self.min_visible = count.into_value();
        self
    }

    /// Limit the number of visible items independently of available width.
    pub fn max_visible(mut self, count: impl IntoValue<usize>) -> Self {
        self.max_visible = count.into_value();
        self
    }

    /// Limit the rows of items allowed before overflow is applied.
    pub fn max_rows(mut self, rows: impl IntoValue<usize>) -> Self {
        let rows = rows.into_value();
        self.max_rows = (move || rows.get().max(1)).into_value();
        self
    }

    /// Choose the logical edge from which items collapse.
    pub fn collapse_from(mut self, edge: impl IntoValue<CollapseFrom>) -> Self {
        self.collapse_from = edge.into_value();
        self
    }

    /// Choose the element used to measure available width.
    pub fn measure(mut self, measure: impl IntoValue<OverflowMeasure>) -> Self {
        self.measure = measure.into_value();
        self
    }

    /// Report the hidden item keys when the overflow result changes.
    pub fn on_overflow_change<R: HandlerResult>(
        mut self,
        callback_fn: impl Fn(Vec<K>) -> R + 'static,
    ) -> Self {
        self.on_overflow_change = Some(callback(callback_fn));
        self
    }
}

fn fits(widths: impl IntoIterator<Item = f64>, available: f64, gap: f64, max_rows: usize) -> bool {
    let mut rows = 1;
    let mut used = 0.0;
    let mut started = false;
    for width in widths {
        let width = width.max(0.0);
        if width > available {
            return false;
        }
        let required = if started { used + gap + width } else { width };
        if required > available {
            rows += 1;
            if rows > max_rows {
                return false;
            }
            used = width;
        } else {
            used = required;
        }
        started = true;
    }
    true
}

// Immutable fitting inputs and the one measurement-owner handle; no cached fit, selection, lifecycle or duplicated source is stored.
struct FitInput<T: PropValue, K: ItemKey> {
    items: Value<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    item_width: Option<ItemWidth<T>>,
    indicator_width: Box<dyn Fn(usize) -> f64>,
    rect: Value<Option<kivact::geometry::Rect>>,
    widths: MachineHandle<WidthMeasurements<K>>,
    min: Value<usize>,
    max: Value<usize>,
    gap: Value<f64>,
    rows: Value<usize>,
    edge: Value<CollapseFrom>,
}
impl<T: PropValue, K: ItemKey> FitInput<T, K> {
    fn fit(&self) -> OverflowFit {
        self.items.with(|items| {
            let floor = self.min.get().min(items.len());
            let ceiling = self.max.get().max(floor).min(items.len());
            let Some(rect) = self.rect.get() else {
                return OverflowFit::Measuring;
            };
            let widths: Vec<_> = match &self.item_width {
                Some(width) => items.iter().map(width).collect(),
                None => {
                    let cache = self.widths.get();
                    items
                        .iter()
                        .map(|item| cache.get(&(self.key)(item)).copied().unwrap_or(80.0))
                        .collect()
                }
            };
            for count in (floor..=ceiling).rev() {
                let hidden = items.len() - count;
                let marker = (hidden > 0).then(|| (self.indicator_width)(hidden));
                let fit = match self.edge.get() {
                    CollapseFrom::Start => fits(
                        marker.into_iter().chain(widths[hidden..].iter().copied()),
                        rect.width,
                        self.gap.get(),
                        self.rows.get(),
                    ),
                    CollapseFrom::End => fits(
                        widths[..count].iter().copied().chain(marker),
                        rect.width,
                        self.gap.get(),
                        self.rows.get(),
                    ),
                };
                if fit {
                    return OverflowFit::Fitted { visible: count };
                }
            }
            OverflowFit::Fitted { visible: floor }
        })
    }
}
fn partition<T: PropValue>(
    items: Value<Vec<T>>,
    split: Value<OverflowFit>,
    min: Value<usize>,
    max: Value<usize>,
    edge: Value<CollapseFrom>,
) -> Value<(Vec<T>, Vec<T>)> {
    memo(move || {
        items.with(|items| {
            let count = match split.get() {
                OverflowFit::Measuring => max.get().max(min.get()).min(items.len()),
                OverflowFit::Fitted { visible } => visible.min(items.len()),
            };
            match edge.get() {
                CollapseFrom::Start => (
                    items[items.len() - count..].to_vec(),
                    items[..items.len() - count].to_vec(),
                ),
                CollapseFrom::End => (items[..count].to_vec(), items[count..].to_vec()),
            }
        })
    })
}
fn observe_hidden<T: PropValue, K: ItemKey>(
    hidden: Value<Vec<T>>,
    rect: Value<Option<kivact::geometry::Rect>>,
    key: Rc<dyn Fn(&T) -> K>,
    notify: Callback<Vec<K>>,
) {
    let hidden_keys = memo(move || {
        rect.get()?;
        Some(hidden.with(|items| items.iter().map(|item| key(item)).collect()))
    });
    effect_on(hidden_keys, move |keys: &Option<Vec<K>>| {
        if let Some(keys) = keys {
            notify.call(keys.clone());
        }
    });
}
fn observe_items<T: PropValue, K: ItemKey>(
    items: Value<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    widths: MachineHandle<WidthMeasurements<K>>,
) {
    effect_on(items, move |items| {
        widths.send(Measurement::Retain(
            items.iter().map(|item| key(item)).collect(),
        ));
    });
}
fn measured_row<T: PropValue, K: ItemKey>(
    item: Value<T>,
    key: K,
    render: ItemRenderer<T>,
    widths: MachineHandle<WidthMeasurements<K>>,
) -> Node {
    let reference = node_ref();
    let rect = reference.rect();
    let width = memo(move || rect.get().map(|rect| rect.width));
    effect_on(width, move |width| {
        if let Some(width) = *width {
            widths.send(Measurement::Measured(key.clone(), width));
        }
    });
    node()
        .node_ref(reference)
        .style(style().shrink(0))
        .child(render.call(item))
}
fn item_rows<T: PropValue, K: ItemKey>(
    visible: Value<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    render: ItemRenderer<T>,
    widths: MachineHandle<WidthMeasurements<K>>,
    measure: bool,
) -> View {
    let row_key = key.clone();
    for_each(
        visible,
        move |item| key(item),
        move |item| {
            if measure {
                let key = item.with_untracked(|item| row_key(item));
                measured_row(item, key, render.clone(), widths)
            } else {
                node().style(style().shrink(0)).child(render.call(item))
            }
        },
    )
}
fn overflow_marker<T: PropValue>(hidden: Value<Vec<T>>, indicator: OverflowRenderer<T>) -> View {
    show(
        move || hidden.with(|items| !items.is_empty()),
        move || {
            node()
                .style(style().shrink(0))
                .child(indicator.call(hidden))
        },
    )
    .into_view()
}
fn overflow_root(root: Node, own: NodeRef, gap: Value<f64>, rows: Value<usize>) -> Node {
    root.node_ref(own).with_node(move |node| {
        node.base_style(move || {
            style()
                .direction(Direction::Row)
                .gap(gap.get())
                .wrap(rows.get() > 1)
                .align_items(Align::Center)
                .min_width(0)
                .overflow(Overflow::Hidden)
        })
    })
}
impl<T: PropValue, K: ItemKey> IntoView for OverflowList<T, K> {
    fn into_view(self) -> View {
        let Self {
            root,
            items,
            key,
            render,
            item_width,
            indicator_width,
            indicator,
            on_overflow_change,
            measure,
            gap,
            min_visible,
            max_visible,
            max_rows,
            collapse_from,
        } = self;
        let own = node_ref();
        let rect = match measure.get() {
            OverflowMeasure::OwnWidth => own.rect(),
            OverflowMeasure::ParentWidth(parent) => parent.rect(),
        };
        let measure_items = item_width.is_none();
        let widths = machine(
            HashMap::new(),
            || WidthMeasurements(PhantomData),
            |_, ()| {},
        );
        let fitting = FitInput {
            items,
            key: key.clone(),
            item_width,
            indicator_width,
            rect,
            widths,
            min: min_visible,
            max: max_visible,
            gap,
            rows: max_rows,
            edge: collapse_from,
        };
        let split = memo(move || fitting.fit());
        let parts = partition(items, split, min_visible, max_visible, collapse_from);
        let visible = memo(move || parts.get().0);
        let hidden = memo(move || parts.get().1);
        if let Some(notify) = on_overflow_change {
            observe_hidden(hidden, rect, key.clone(), notify);
        }
        if measure_items {
            observe_items(items, key.clone(), widths);
        }
        let rows = item_rows(visible, key, render, widths, measure_items);
        let marker = overflow_marker(hidden, indicator);
        let root = overflow_root(root, own, gap, max_rows);
        match collapse_from.get() {
            CollapseFrom::Start => root.child((marker, rows)),
            CollapseFrom::End => root.child((rows, marker)),
        }
        .into_view()
    }
}

kivact::state::constant_value!(CollapseFrom, OverflowMeasure);
