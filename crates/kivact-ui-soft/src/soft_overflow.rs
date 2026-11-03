//! Role: Width-bounded keyed overflow lists.

use std::hash::Hash;
use std::rc::Rc;

use crate::tokens;
use kivact::control::{Node, NodeBuilder, node, text};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, Overflow, style};
use kivact::view::{
    HandlerResult, IntoValue, IntoView, NodeRef, View, callback, for_each, node_ref, show,
};
use kivact::{Callback, Value, effect_on, memo};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CollapseFrom {
    Start,
    #[default]
    End,
}

#[derive(Clone, Default)]
pub enum OverflowMeasure {
    #[default]
    OwnWidth,
    /// Measure against the caller's reference to the containing element.
    ParentWidth(NodeRef),
}

type ItemRenderer<T> = Box<dyn Fn(Value<T>) -> View>;
type OverflowRenderer<T> = Box<dyn Fn(Value<Vec<T>>) -> View>;

pub struct OverflowList<T: Clone + PartialEq + 'static, K: Clone + Eq + Hash + 'static> {
    root: Node,
    items: Value<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    render: ItemRenderer<T>,
    item_width: Box<dyn Fn(&T) -> f32>,
    indicator_width: Box<dyn Fn(usize) -> f32>,
    indicator: OverflowRenderer<T>,
    on_overflow_change: Option<Callback<Vec<K>>>,
    measure: OverflowMeasure,
    gap: f32,
    min_visible: usize,
    max_visible: usize,
    max_rows: usize,
    collapse_from: CollapseFrom,
}

pub fn overflow_list<T, K, V>(
    items: impl IntoValue<Vec<T>>,
    key: impl Fn(&T) -> K + 'static,
    item: impl Fn(Value<T>) -> V + 'static,
) -> OverflowList<T, K>
where
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + 'static,
    V: IntoView,
{
    OverflowList {
        root: node(),
        items: items.into_value(),
        key: Rc::new(key),
        render: Box::new(move |value| item(value).into_view()),
        item_width: Box::new(|_| 80.0),
        indicator_width: Box::new(|count| 18.0 + count.to_string().len() as f32 * 8.0),
        indicator: Box::new(|hidden| {
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
        measure: OverflowMeasure::OwnWidth,
        gap: 8.0,
        min_visible: 0,
        max_visible: usize::MAX,
        max_rows: 1,
        collapse_from: CollapseFrom::End,
    }
}

impl<T: Clone + PartialEq + 'static, K: Clone + Eq + Hash + 'static> NodeBuilder
    for OverflowList<T, K>
{
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<T: Clone + PartialEq + 'static, K: Clone + Eq + Hash + 'static> OverflowList<T, K> {
    /// Provide each item's natural width in pixels. Defaults to 80 px.
    pub fn item_width(mut self, width: impl Fn(&T) -> f32 + 'static) -> Self {
        self.item_width = Box::new(width);
        self
    }

    /// Reserve this much horizontal space for a custom indicator.
    pub fn indicator_width(mut self, width: impl Fn(usize) -> f32 + 'static) -> Self {
        self.indicator_width = Box::new(width);
        self
    }

    pub fn overflow<V: IntoView>(mut self, render: impl Fn(Value<Vec<T>>) -> V + 'static) -> Self {
        self.indicator = Box::new(move |hidden| render(hidden).into_view());
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    pub fn min_visible(mut self, count: usize) -> Self {
        self.min_visible = count;
        self
    }

    pub fn max_visible(mut self, count: usize) -> Self {
        self.max_visible = count;
        self
    }

    pub fn max_rows(mut self, rows: usize) -> Self {
        self.max_rows = rows.max(1);
        self
    }

    pub fn collapse_from(mut self, edge: CollapseFrom) -> Self {
        self.collapse_from = edge;
        self
    }

    pub fn measure(mut self, measure: OverflowMeasure) -> Self {
        self.measure = measure;
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

fn fits(widths: impl IntoIterator<Item = f32>, available: f32, gap: f32, max_rows: usize) -> bool {
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

impl<T: Clone + PartialEq + 'static, K: Clone + Eq + Hash + 'static> IntoView
    for OverflowList<T, K>
{
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
        let rect = match measure {
            OverflowMeasure::OwnWidth => own.rect(),
            OverflowMeasure::ParentWidth(parent) => parent.rect(),
        };
        let split = memo(move || {
            items.with(|items| {
                let floor = min_visible.min(items.len());
                let ceiling = max_visible.max(floor).min(items.len());
                let Some(rect) = rect.get() else {
                    return ceiling;
                };
                let widths: Vec<_> = items.iter().map(&item_width).collect();
                for count in (floor..=ceiling).rev() {
                    let hidden = items.len() - count;
                    let marker = (hidden > 0).then(|| indicator_width(hidden));
                    let fit = match collapse_from {
                        CollapseFrom::Start => fits(
                            marker.into_iter().chain(widths[hidden..].iter().copied()),
                            rect.width,
                            gap,
                            max_rows,
                        ),
                        CollapseFrom::End => fits(
                            widths[..count].iter().copied().chain(marker),
                            rect.width,
                            gap,
                            max_rows,
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
                match collapse_from {
                    CollapseFrom::Start => items[items.len() - count..].to_vec(),
                    CollapseFrom::End => items[..count].to_vec(),
                }
            })
        });
        let hidden = memo(move || {
            items.with(|items| {
                let count = split.get().min(items.len());
                match collapse_from {
                    CollapseFrom::Start => items[..items.len() - count].to_vec(),
                    CollapseFrom::End => items[count..].to_vec(),
                }
            })
        });
        if let Some(notify) = on_overflow_change {
            let key = key.clone();
            let hidden_keys =
                memo(move || hidden.with(|items| items.iter().map(|item| key(item)).collect()));
            effect_on(hidden_keys, move |keys: &Vec<K>| notify.call(keys.clone()));
        }
        let rows = for_each(
            visible,
            move |item| key(item),
            move |_, item| node().style(style().shrink(0)).child(render(item)),
        );
        let marker = show(
            move || hidden.with(|items| !items.is_empty()),
            move || node().style(style().shrink(0)).child(indicator(hidden)),
        );
        let root = root.node_ref(own).base_style(
            style()
                .direction(Direction::Row)
                .gap(gap)
                .wrap(max_rows > 1)
                .align_items(Align::Center)
                .min_width(0)
                .overflow(Overflow::Hidden),
        );
        match collapse_from {
            CollapseFrom::Start => root.child((marker, rows)),
            CollapseFrom::End => root.child((rows, marker)),
        }
        .into_view()
    }
}
