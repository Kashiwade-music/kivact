//! Role: Group already sorted and paged data without adding synthetic data keys.
use super::group_state::DataTableGroupState;
use super::rows::{self, Grid};
use super::types::Grouping;
use crate::layer::with_recipe_style;
use crate::{ButtonVariant, button};
use kivact_core::i18n::message_with;
use kivact_core::motion::{Easing, Transition};
use kivact_core::state::{Binding, Value, memo};
use kivact_core::style::{
    Align, ColorToken, Cursor, Justify, Position, TransitionProp, auto, pct, style,
};
use kivact_core::view::IntoView;
use kivact_core::view::NodeBuilder;
use kivact_core::view::{ReadingDirection, Role, View, for_each, use_direction};
use kivact_core::view::{icon, node, row, text};
use kivact_interaction::TableFocus;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, PartialEq)]
enum Entry<R> {
    Group(String, DataTableGroupState),
    Data(R),
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum EntryKey<K> {
    Group(String),
    Data(K),
}

pub(super) struct Plan<R: 'static> {
    entries: Value<Vec<Entry<R>>>,
    collapsed: Binding<Vec<String>>,
}

impl<R> Clone for Plan<R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> Copy for Plan<R> {}

pub(super) fn plan<R: kivact_core::state::PropValue>(
    rows: Value<Vec<R>>,
    grouping: Grouping<R>,
) -> Plan<R> {
    let Grouping { key, collapsed } = grouping;
    let entries = memo(move || {
        let mut positions = HashMap::new();
        let mut groups = Vec::<(String, Vec<R>)>::new();
        rows.with(|rows| {
            for row in rows {
                let label = key(row);
                let index = *positions.entry(label.clone()).or_insert_with(|| {
                    let index = groups.len();
                    groups.push((label, Vec::new()));
                    index
                });
                groups[index].1.push(row.clone());
            }
        });
        collapsed.with(|closed| {
            let mut entries = Vec::new();
            for (label, members) in groups {
                let state = DataTableGroupState {
                    count: members.len(),
                    collapsed: closed.contains(&label),
                };
                entries.push(Entry::Group(label, state));
                if !state.collapsed {
                    entries.extend(members.into_iter().map(Entry::Data));
                }
            }
            entries
        })
    });
    Plan { entries, collapsed }
}

impl<R: kivact_core::state::PropValue> Plan<R> {
    pub(super) fn rows(self) -> Value<Vec<R>> {
        memo(move || {
            self.entries.with(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| match entry {
                        Entry::Data(row) => Some(row.clone()),
                        Entry::Group(..) => None,
                    })
                    .collect()
            })
        })
    }

    pub(super) fn view<K: kivact_core::state::ItemKey>(
        self,
        grid: Grid<R, K>,
        key: Rc<dyn Fn(&R) -> K>,
    ) -> View {
        for_each(
            self.entries,
            move |entry| match entry {
                Entry::Group(label, _) => EntryKey::Group(label.clone()),
                Entry::Data(row) => EntryKey::Data(key(row)),
            },
            move |entry| {
                let entry_key = entry.with_untracked(|entry| match entry {
                    Entry::Group(label, _) => EntryKey::Group(label.clone()),
                    Entry::Data(row) => EntryKey::Data((grid.key)(row)),
                });
                match entry_key {
                    EntryKey::Group(label) => {
                        let state = memo(move || {
                            entry.with(|entry| match entry {
                                Entry::Group(_, state) => *state,
                                Entry::Data(_) => unreachable!("group key refers to a group"),
                            })
                        });
                        header(&grid, label, state, self.collapsed)
                    }
                    EntryKey::Data(key) => {
                        let row = memo(move || {
                            entry.with(|entry| match entry {
                                Entry::Data(row) => row.clone(),
                                Entry::Group(..) => unreachable!("data key refers to a row"),
                            })
                        });
                        rows::body(&grid, key, row)
                    }
                }
            },
        )
    }
}

fn header<R: 'static, K: kivact_core::state::ItemKey>(
    grid: &Grid<R, K>,
    label: String,
    state: Value<DataTableGroupState>,
    collapsed: Binding<Vec<String>>,
) -> View {
    let row_name = label.clone();
    let metadata = grid.row_metadata;
    let coordinate_label = label.clone();
    let row_index = memo(move || {
        metadata.with(|metadata| metadata.groups.get(&coordinate_label).copied().unwrap_or(0))
    });
    let name = {
        let group = label.clone();
        let expand = message_with("data_table.group.expand", "Expand group {group}", {
            let group = group.clone();
            move || [("group", group.clone())]
        });
        let collapse = message_with(
            "data_table.group.collapse",
            "Collapse group {group}",
            move || [("group", group.clone())],
        );
        memo(move || {
            if state.get().collapsed {
                expand.get()
            } else {
                collapse.get()
            }
        })
    };
    let rtl = use_direction().get() == ReadingDirection::Rtl;
    let mark = icon("chevron-right").style(move || {
        let angle = if !state.get().collapsed {
            90
        } else if rtl {
            180
        } else {
            0
        };
        style()
            .size(16, 16)
            .color(ColorToken::TextMuted)
            .rotate(angle)
            .transition(
                TransitionProp::Paint,
                Transition::tween(150, Easing::CubicBezier(0.25, 0.1, 0.25, 1.0)),
            )
    });
    let content = row((
        mark,
        text(label.clone()).style(style().weight(600)),
        text(move || format!("({})", state.get().count))
            .style(style().weight(400).color(ColorToken::TextMuted)),
    ))
    .style({
        let base = style()
            .position(Position::Sticky)
            .gap(4)
            .align_items(Align::Center);
        if rtl {
            base.right(0).padding_right(4)
        } else {
            base.left(0).padding_left(4)
        }
    });
    let disclosure = button(name)
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .with_node(|node| node.expanded(move || !state.get().collapsed))
        .on_press(move || {
            let mut groups = collapsed.get_untracked();
            if groups.contains(&label) {
                groups.retain(|key| key != &label);
            } else {
                groups.push(label.clone());
            }
            collapsed.set(groups);
        })
        .style(
            style()
                .width(pct(100))
                .height(auto())
                .min_height(0)
                .padding(0)
                .padding_y(8)
                .padding_right(12)
                .justify_content(Justify::Start)
                .radius(0)
                .cursor(Cursor::Pointer),
        )
        .child(content);
    let cell_role = if matches!(grid.focus, TableFocus::None) {
        Role::Cell
    } else {
        Role::GridCell
    };
    with_recipe_style(
        row(node()
            .role(cell_role)
            .column_index(1)
            .style(style().grow(1))
            .child(disclosure))
        .role(Role::Row)
        .row_index(row_index)
        .name(row_name),
        |recipes| recipes.data_table_recipes().group_header_recipe(),
        &grid.variants,
        move || state.get(),
    )
    .into_view()
}
