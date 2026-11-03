//! Role: Compose named search filters with an explicit editor and remove controls.
use crate::theme_access::tokens;
use kivact::control::{Node, NodeBuilder, icon, node, row};
use kivact::i18n::message;
use kivact::style::{Align, Length, style};
use kivact::ui::{ButtonVariant, DialogKind, ListItem, button, dialog, select, text_input};
use kivact::view::{IntoText, IntoValue, IntoView, View, for_each};
use kivact::{Binding, Value, signal};
use std::hash::Hash;

/// A filter over a named field. Values are stored as text for caller-defined search backends.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchFilter<K> {
    pub field: K,
    pub operator: SearchOperator,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SearchOperator {
    Is,
    IsNot,
    Contains,
}

pub struct PowerSearch<K: Clone + Eq + Hash + 'static> {
    root: Node,
    label: Value<String>,
    fields: Value<Vec<ListItem<K>>>,
    filters: Binding<Vec<SearchFilter<K>>>,
}

pub fn power_search<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    fields: impl IntoValue<Vec<ListItem<K>>>,
    filters: impl Into<Binding<Vec<SearchFilter<K>>>>,
) -> PowerSearch<K> {
    PowerSearch {
        root: node(),
        label: label.into_text_value(),
        fields: fields.into_value(),
        filters: filters.into(),
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for PowerSearch<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: Clone + Eq + Hash + 'static> IntoView for PowerSearch<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            fields,
            filters,
        } = self;
        let open = signal(false);
        let editing = signal(None::<(usize, SearchFilter<K>)>);
        let field = signal(None::<K>);
        let operator = signal(Some(SearchOperator::Is));
        let value = signal(String::new());
        let entries =
            kivact::memo(move || filters.with(|filters| (0..filters.len()).collect::<Vec<_>>()));
        let tokens_view = for_each(
            entries,
            |index| *index,
            move |index, _| {
                let name = kivact::memo(move || {
                    filters.with(|entries| {
                        entries
                            .get(index)
                            .map(|entry| {
                                let name = fields
                                    .with(|fields| {
                                        fields
                                            .iter()
                                            .find(|item| item.key() == &entry.field)
                                            .map(|item| item.label())
                                    })
                                    .unwrap_or_default();
                                format!("{} {}", name, entry.value)
                            })
                            .unwrap_or_default()
                    })
                });
                let edit = button(name)
                    .variant(ButtonVariant::Ghost)
                    .on_press(move || {
                        if let Some(filter) = filters.with(|entries| entries.get(index).cloned()) {
                            editing.set(Some((index, filter.clone())));
                            field.set(Some(filter.field));
                            operator.set(Some(filter.operator));
                            value.set(filter.value);
                            open.set(true);
                        }
                    });
                let remove = button(move || format!("Remove {}", name.get()))
                    .variant(ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(icon("x"))
                    .on_press(move || {
                        let mut entries = filters.get_untracked();
                        if index < entries.len() {
                            entries.remove(index);
                            filters.set(entries);
                        }
                    });
                row((edit, remove)).style(move || {
                    let t = tokens();
                    style()
                        .height(28)
                        .align_items(Align::Center)
                        .radius(t.radius.element)
                        .background(t.colors.accent_muted)
                })
            },
        );
        let add = button(message("soft.power_search.add", "Add filter"))
            .variant(ButtonVariant::Ghost)
            .icon(icon("plus"))
            .on_press(move || {
                editing.set(None);
                field.set(None);
                operator.set(Some(SearchOperator::Is));
                value.set(String::new());
                open.set(true);
            });
        let trigger = root
            .name(label)
            .child(row((tokens_view, add)).style(style().wrap(true).gap(4)));
        dialog(label, open)
            .kind(DialogKind::Popover)
            .hide_title(true)
            .width(Length::Auto)
            .content(move || {
                let operators = vec![
                    ListItem::new(SearchOperator::Is, message("soft.power_search.is", "Is")),
                    ListItem::new(
                        SearchOperator::IsNot,
                        message("soft.power_search.is_not", "Is not"),
                    ),
                    ListItem::new(
                        SearchOperator::Contains,
                        message("soft.power_search.contains", "Contains"),
                    ),
                ];
                let save = button(message("soft.power_search.apply", "Apply"))
                    .disabled(move || {
                        field.get().is_none()
                            || operator.get().is_none()
                            || value.get().trim().is_empty()
                    })
                    .on_press(move || {
                        let (Some(field), Some(operator)) =
                            (field.get_untracked(), operator.get_untracked())
                        else {
                            return;
                        };
                        let value = value.get_untracked();
                        if value.trim().is_empty() {
                            return;
                        }
                        let filter = SearchFilter {
                            field,
                            operator,
                            value,
                        };
                        let mut entries = filters.get_untracked();
                        match editing.get_untracked() {
                            Some((index, original)) => {
                                let Some(entry) = entries.get_mut(index) else {
                                    return;
                                };
                                if *entry != original {
                                    return;
                                }
                                *entry = filter;
                            }
                            None => entries.push(filter),
                        }
                        filters.set(entries);
                        open.set(false);
                    });
                row((
                    select(message("soft.power_search.field", "Field"), fields, field),
                    select(
                        message("soft.power_search.operator", "Operator"),
                        operators,
                        operator,
                    ),
                    text_input(message("soft.power_search.value", "Value"), value),
                    save,
                ))
                .style(style().wrap(true).gap(8).padding(12))
            })
            .attach(trigger)
    }
}
