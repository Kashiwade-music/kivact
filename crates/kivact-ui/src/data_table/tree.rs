//! Role: Arrange hierarchical table rows and render their disclosure controls.
use super::types::{DataTableTreeIndent, Key, TreeConfig};
use crate::{Button, ButtonVariant, button};
use kivact_core::custom::NodeExt;
use kivact_core::i18n::message;
use kivact_core::motion::{Easing, Transition};
use kivact_core::state::{Binding, Value, memo};
use kivact_core::style::{Align, ColorToken, Direction, TransitionProp, style};
use kivact_core::view::{IntoView, ReadingDirection, View, current_direction, dynamic};
use kivact_core::view::{NodeBuilder, icon, node, row};
use kivact_interaction::{TreeNode, TreeRow, flatten_tree_rows};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

impl DataTableTreeIndent {
    fn pixels(self) -> f64 {
        match self {
            Self::Small => 12.0,
            Self::Medium => 16.0,
            Self::Large => 24.0,
        }
    }
}

pub(super) struct Plan<K: 'static> {
    pub(super) column: usize,
    pub(super) show_all: bool,
    indent: DataTableTreeIndent,
    rows: Value<HashMap<K, TreeRow<K>>>,
    parents: Value<Vec<K>>,
    expanded: Binding<Vec<K>>,
}

impl<K> Clone for Plan<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for Plan<K> {}

pub(super) fn arrange<R, K: Clone + Eq + Hash + 'static>(
    rows: &[R],
    order: Vec<usize>,
    key: &Key<R, K>,
    parent: &Key<R, Option<K>>,
    expanded: Binding<Vec<K>>,
) -> Vec<usize> {
    let mut indices = HashMap::with_capacity(order.len());
    let nodes = order
        .into_iter()
        .map(|index| {
            let row = &rows[index];
            let id = key(row);
            indices.entry(id.clone()).or_insert(index);
            TreeNode::new(id, parent(row), "")
        })
        .collect::<Vec<_>>();
    let open = expanded.with(|keys| keys.iter().cloned().collect());
    flatten_tree_rows(&nodes, &open)
        .into_iter()
        .map(|row| indices[&row.key])
        .collect()
}

impl<K: Clone + Eq + Hash + 'static> Plan<K> {
    pub(super) fn new<R: 'static>(
        source: Value<Vec<R>>,
        key: Key<R, K>,
        config: TreeConfig<R, K>,
        column: usize,
        indent: DataTableTreeIndent,
        show_all: bool,
    ) -> Self {
        let TreeConfig { parent, expanded } = config;
        let all = memo(move || {
            source.with(|rows| {
                let nodes = rows
                    .iter()
                    .map(|row| TreeNode::new(key(row), parent(row), ""))
                    .collect::<Vec<_>>();
                let keys = nodes
                    .iter()
                    .map(|node| node.key().clone())
                    .collect::<HashSet<_>>();
                flatten_tree_rows(&nodes, &keys)
            })
        });
        let rows = memo(move || {
            all.with(|all| {
                all.iter()
                    .map(|row| (row.key.clone(), row.clone()))
                    .collect()
            })
        });
        let parents = memo(move || {
            all.with(|all| {
                all.iter()
                    .filter(|row| row.branch.has_children())
                    .map(|row| row.key.clone())
                    .collect()
            })
        });
        Self {
            column,
            show_all,
            indent,
            rows,
            parents,
            expanded,
        }
    }

    pub(super) fn is_parent(self, key: K) -> Value<bool> {
        memo(move || {
            self.rows
                .with(|rows| rows.get(&key).is_some_and(|row| row.branch.has_children()))
        })
    }

    pub(super) fn level(self, key: K) -> Value<u32> {
        memo(move || {
            self.rows
                .with(|rows| rows.get(&key).map_or(1, |row| row.level))
        })
    }

    pub(super) fn open(self, key: K) -> Value<bool> {
        memo(move || self.expanded.with(|keys| keys.contains(&key)))
    }

    pub(super) fn cell(self, key: K, content: View) -> View {
        let level = self.level(key.clone());
        let open = self.open(key.clone());
        let trigger = dynamic(self.is_parent(key.clone()), move |branch| {
            if !branch {
                return node().style(style().size(24, 24).shrink(0)).into_view();
            }
            let key = key.clone();
            control(open, false)
                .on_press(move || {
                    let mut keys = self.expanded.get_untracked();
                    if keys.contains(&key) {
                        keys.retain(|id| id != &key);
                    } else {
                        keys.push(key.clone());
                    }
                    self.expanded.set(keys);
                })
                .into_view()
        });
        let rtl = current_direction() == ReadingDirection::Rtl;
        row((trigger, content))
            .style(move || {
                let inset = level.get().saturating_sub(1) as f64 * self.indent.pixels();
                let style = style().gap(4).align_items(Align::Center).min_width(0);
                if rtl {
                    style.direction(Direction::RowReverse).padding_right(inset)
                } else {
                    style.padding_left(inset)
                }
            })
            .into_view()
    }

    pub(super) fn header(self) -> View {
        let aggregate = memo(move || {
            self.parents.with(|parents| {
                self.expanded.with(|open| {
                    let count = parents.iter().filter(|key| open.contains(key)).count();
                    (
                        parents.is_empty(),
                        count != 0 && count == parents.len(),
                        count != 0 && count < parents.len(),
                    )
                })
            })
        });
        dynamic(
            move || aggregate.get().0,
            move |empty| {
                if empty {
                    return ().into_view();
                }
                let open = memo(move || aggregate.get().1);
                control(open, true)
                    .indeterminate(move || aggregate.get().2)
                    .on_press(move || {
                        self.expanded.set(if open.get_untracked() {
                            Vec::new()
                        } else {
                            self.parents.get_untracked()
                        })
                    })
                    .into_view()
            },
        )
    }
}

fn control(open: Value<bool>, all: bool) -> Button {
    let (expand, collapse) = if all {
        (
            message("soft.table.tree.expand_all", "Expand all rows"),
            message("soft.table.tree.collapse_all", "Collapse all rows"),
        )
    } else {
        (
            message("soft.table.tree.expand", "Expand row"),
            message("soft.table.tree.collapse", "Collapse row"),
        )
    };
    let rtl = current_direction() == ReadingDirection::Rtl;
    button(move || {
        if open.get() {
            collapse.get()
        } else {
            expand.get()
        }
    })
    .variant(ButtonVariant::Ghost)
    .hide_label(true)
    .expanded(open)
    .style(
        style()
            .size(24, 24)
            .min_width(24)
            .min_height(24)
            .shrink(0)
            .padding(0)
            .radius(6),
    )
    .child(
        icon(if rtl { "chevron-left" } else { "chevron-right" }).style(move || {
            style()
                .size(12, 12)
                .color(ColorToken::TextMuted)
                .rotate(if open.get() {
                    if rtl { -90.0 } else { 90.0 }
                } else {
                    0.0
                })
                .transition(
                    TransitionProp::Paint,
                    Transition::tween(150, Easing::CubicBezier(0.25, 0.1, 0.25, 1.0)),
                )
        }),
    )
}
