//! Role: Width-bounded keyed overflow lists.

use kivact::state::ItemKey;
use kivact::state::PropValue;
use std::collections::HashMap;
use std::rc::Rc;

use crate::tokens;
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
use kivact::state::signal;
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CollapseFrom {
    Start,
    #[default]
    End,
}

#[derive(Clone, Default, PartialEq)]
pub enum OverflowMeasure {
    #[default]
    OwnWidth,
    /// Measure against the caller's reference to the containing element.
    ParentWidth(NodeRef),
}

type ItemRenderer<T> = kivact::view::ViewFn<Value<T>>;
type ItemWidth<T> = Box<dyn Fn(&T) -> f64>;
type OverflowRenderer<T> = kivact::view::ViewFn<Value<Vec<T>>>;

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
                    style()
                        .padding_x(5)
                        .font_size(12)
                        .color(tokens().colors.text_secondary)
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

    pub fn overflow<V: IntoView>(mut self, render: impl Fn(Value<Vec<T>>) -> V + 'static) -> Self {
        self.indicator = render.into();
        self
    }

    pub fn gap(mut self, gap: impl IntoValue<f64>) -> Self {
        let gap = gap.into_value();
        self.gap = (move || gap.get().max(0.0)).into_value();
        self
    }

    pub fn min_visible(mut self, count: impl IntoValue<usize>) -> Self {
        self.min_visible = count.into_value();
        self
    }

    pub fn max_visible(mut self, count: impl IntoValue<usize>) -> Self {
        self.max_visible = count.into_value();
        self
    }

    pub fn max_rows(mut self, rows: impl IntoValue<usize>) -> Self {
        let rows = rows.into_value();
        self.max_rows = (move || rows.get().max(1)).into_value();
        self
    }

    pub fn collapse_from(mut self, edge: impl IntoValue<CollapseFrom>) -> Self {
        self.collapse_from = edge.into_value();
        self
    }

    pub fn measure(mut self, measure: impl IntoValue<OverflowMeasure>) -> Self {
        self.measure = measure.into_value();
        self
    }

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
        let widths_cache = signal(HashMap::<K, f64>::new());
        let widths_key = key.clone();
        let split = memo(move || {
            items.with(|items| {
                let floor = min_visible.get().min(items.len());
                let ceiling = max_visible.get().max(floor).min(items.len());
                let Some(rect) = rect.get() else {
                    return ceiling;
                };
                let widths: Vec<_> = match &item_width {
                    Some(width) => items.iter().map(width).collect(),
                    None => widths_cache.with(|cache| {
                        items
                            .iter()
                            .map(|item| cache.get(&widths_key(item)).copied().unwrap_or(80.0))
                            .collect()
                    }),
                };
                for count in (floor..=ceiling).rev() {
                    let hidden = items.len() - count;
                    let marker = (hidden > 0).then(|| indicator_width(hidden));
                    let fit = match collapse_from.get() {
                        CollapseFrom::Start => fits(
                            marker.into_iter().chain(widths[hidden..].iter().copied()),
                            rect.width,
                            gap.get(),
                            max_rows.get(),
                        ),
                        CollapseFrom::End => fits(
                            widths[..count].iter().copied().chain(marker),
                            rect.width,
                            gap.get(),
                            max_rows.get(),
                        ),
                    };
                    if fit {
                        return count;
                    }
                }
                floor
            })
        });
        let visible = memo(move || {
            items.with(|items| {
                let count = split.get().min(items.len());
                match collapse_from.get() {
                    CollapseFrom::Start => items[items.len() - count..].to_vec(),
                    CollapseFrom::End => items[..count].to_vec(),
                }
            })
        });
        let hidden = memo(move || {
            items.with(|items| {
                let count = split.get().min(items.len());
                match collapse_from.get() {
                    CollapseFrom::Start => items[..items.len() - count].to_vec(),
                    CollapseFrom::End => items[count..].to_vec(),
                }
            })
        });
        if let Some(notify) = on_overflow_change {
            let key = key.clone();
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
        if measure_items {
            let active_key = key.clone();
            effect_on(items, move |items| {
                let active: std::collections::HashSet<_> =
                    items.iter().map(|item| active_key(item)).collect();
                widths_cache.update(|cache| cache.retain(|key, _| active.contains(key)));
            });
        }
        let row_key = key.clone();
        let rows = for_each(
            visible,
            move |item| key(item),
            move |item| {
                let key = item.with_untracked(|item| row_key(item));
                let root = node().style(style().shrink(0)).child(render.call(item));
                if measure_items {
                    let item_ref = node_ref();
                    let rect = item_ref.rect();
                    let width = memo(move || rect.get().map(|rect| rect.width));
                    effect_on(width, move |width| {
                        if let Some(width) = *width {
                            widths_cache.update(|cache| {
                                cache.insert(key.clone(), width);
                            });
                        }
                    });
                    root.node_ref(item_ref)
                } else {
                    root
                }
            },
        );
        let marker = show(
            move || hidden.with(|items| !items.is_empty()),
            move || {
                node()
                    .style(style().shrink(0))
                    .child(indicator.call(hidden))
            },
        );
        let root = root.node_ref(own).base_style(move || {
            style()
                .direction(Direction::Row)
                .gap(gap.get())
                .wrap(max_rows.get() > 1)
                .align_items(Align::Center)
                .min_width(0)
                .overflow(Overflow::Hidden)
        });
        match collapse_from.get() {
            CollapseFrom::Start => root.child((marker, rows)),
            CollapseFrom::End => root.child((rows, marker)),
        }
        .into_view()
    }
}

kivact::state::constant_value!(CollapseFrom, OverflowMeasure);
