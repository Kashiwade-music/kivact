//! Role: Width-bounded keyed overflow lists.

use kivact::state::ItemKey;
use kivact::state::PropValue;
use std::collections::HashMap;
use std::rc::Rc;

use kivact::view::Element;
use kivact::view::Node;
use kivact::view::node;
use kivact::view::text;

use kivact::Prop;
use kivact::component;
use kivact::state::Callback;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::effect_on;
use kivact::state::memo;
use kivact::state::{Machine, MachineHandle, Step, machine};
use kivact::style::Align;
use kivact::style::FlexDirection;
use kivact::style::Overflow;
use kivact::style::style;
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
/// Logical edge from which items move into the overflow indicator.
pub enum CollapseFrom {
    /// Hide items from the logical start first.
    Start,
    #[default]
    /// Hide items from the logical end first.
    End,
}

#[derive(Clone, Default, PartialEq, Prop)]
/// Element whose width determines the available overflow space.
pub enum OverflowMeasure {
    #[default]
    /// Use the overflow list own measured width.
    OwnWidth,
    /// Measure against the caller's reference to the containing element.
    ParentWidth(NodeRef),
}

type ItemRenderer<T> = Rc<dyn Fn(Value<T>) -> View>;
type ItemWidth<T> = Rc<dyn Fn(&T) -> f64>;
type OverflowRenderer<T> = Rc<dyn Fn(Value<Vec<T>>) -> View>;

/// A keyed list that fits visible items and exposes the remaining items through an indicator.
///
/// `item_width` gives the widths up front; by default, mounted items are measured and
/// a provisional width of 80 px is used until each measurement arrives.
/// `overflow` renders the indicator from the reactive collection of hidden items.
#[component]
pub fn overflow_list<T: PropValue, K: ItemKey>(
    items: Value<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    item: Rc<dyn Fn(Value<T>) -> View>,
    item_width: Option<Rc<dyn Fn(&T) -> f64>>,
    #[prop(default = |count: usize| 18.0 + count.to_string().len() as f64 * 8.0)]
    indicator_width: Rc<dyn Fn(usize) -> f64>,
    #[prop(default = |hidden: Value<Vec<T>>| {
        text(move || format!("+{}", hidden.with(Vec::len)))
            .style(|| {
                style()
                    .padding_x(5)
                    .font_size(12)
                    .color(crate::color::TEXT_SECONDARY)
            })
            .into_view()
    })]
    overflow: Rc<dyn Fn(Value<Vec<T>>) -> View>,
    on_overflow_change: Option<Callback<Vec<K>>>,
    #[prop(default = OverflowMeasure::OwnWidth)] measure: Value<OverflowMeasure>,
    #[prop(default = 8.0)] gap: Value<f64>,
    #[prop(default = 0)] min_visible: Value<usize>,
    #[prop(default = usize::MAX)] max_visible: Value<usize>,
    #[prop(default = 1)] max_rows: Value<usize>,
    #[prop(default = CollapseFrom::End)] collapse_from: Value<CollapseFrom>,
) -> impl Element {
    let gap = (move || gap.get().max(0.0)).into_value();
    let max_rows = (move || max_rows.get().max(1)).into_value();
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
    let rows = item_rows(visible, key, item, widths, measure_items);
    let marker = overflow_marker(hidden, overflow);
    let root = overflow_root(node(), own, gap, max_rows);
    match collapse_from.get() {
        CollapseFrom::Start => root.child((marker, rows)),
        CollapseFrom::End => root.child((rows, marker)),
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
    indicator_width: Rc<dyn Fn(usize) -> f64>,
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
                Some(width) => items.iter().map(|item| width(item)).collect(),
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
        .style(style().flex_shrink(0))
        .child(render(item))
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
                node().style(style().flex_shrink(0)).child(render(item))
            }
        },
    )
}
fn overflow_marker<T: PropValue>(hidden: Value<Vec<T>>, indicator: OverflowRenderer<T>) -> View {
    show(
        move || hidden.with(|items| !items.is_empty()),
        move || {
            node()
                .style(style().flex_shrink(0))
                .child(indicator(hidden))
        },
    )
    .into_view()
}
fn overflow_root(root: Node, own: NodeRef, gap: Value<f64>, rows: Value<usize>) -> Node {
    root.node_ref(own).with_node(move |node| {
        node.base_style(move || {
            style()
                .flex_direction(FlexDirection::Row)
                .gap(gap.get())
                .flex_wrap(rows.get() > 1)
                .align_items(Align::Center)
                .min_width(0)
                .overflow(Overflow::Hidden)
        })
    })
}
