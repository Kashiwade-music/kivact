//! Role: Compose named search filters with an explicit editor and remove controls.
use crate::theme_access::tokens;
use kivact::control::{Node, NodeBuilder, icon, node, row};
use kivact::i18n::message;
use kivact::state::{Machine, Step, machine};
use kivact::style::{Align, Length, style};
use kivact::ui::{ButtonVariant, DialogKind, ListItem, button, dialog, select, text_input};
use kivact::view::{IntoText, IntoValue, IntoView, View, for_each};
use kivact::{Binding, Value, binding};
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
        let editor = machine(
            PowerSearchState::Closed,
            move || SearchMachine {
                filters: filters.get_untracked(),
            },
            move |_, entries| filters.set(entries),
        );
        let open = binding(
            move || !matches!(editor.get(), PowerSearchState::Closed),
            move |open| {
                if !open {
                    editor.send(SearchEvent::Close);
                } else if matches!(editor.get(), PowerSearchState::Closed) {
                    editor.send(SearchEvent::Add);
                }
            },
        );
        let field = binding(
            move || editor.get().draft().field,
            move |field| editor.send(SearchEvent::Field(field)),
        );
        let operator = binding(
            move || editor.get().draft().operator,
            move |operator| editor.send(SearchEvent::Operator(operator)),
        );
        let value = binding(
            move || editor.get().draft().value,
            move |value| editor.send(SearchEvent::Value(value)),
        );
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
                            editor.send(SearchEvent::Edit(index, filter));
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
                editor.send(SearchEvent::Add);
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
                        editor.send(SearchEvent::Apply);
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

// xtask-allow(state-fields): Field and operator are independent incomplete form inputs.
#[derive(Clone, PartialEq)]
struct FilterDraft<K> {
    field: Option<K>,
    operator: Option<SearchOperator>,
    value: String,
}
impl<K> Default for FilterDraft<K> {
    fn default() -> Self {
        Self {
            field: None,
            operator: Some(SearchOperator::Is),
            value: String::new(),
        }
    }
}
#[derive(Clone, PartialEq)]
enum PowerSearchState<K> {
    Closed,
    Composing {
        draft: FilterDraft<K>,
    },
    Editing {
        index: usize,
        draft: FilterDraft<K>,
        original: SearchFilter<K>,
    },
}
impl<K: Clone> PowerSearchState<K> {
    fn draft(&self) -> FilterDraft<K> {
        match self {
            Self::Closed => FilterDraft::default(),
            Self::Composing { draft } | Self::Editing { draft, .. } => draft.clone(),
        }
    }
}
enum SearchEvent<K> {
    Close,
    Add,
    Edit(usize, SearchFilter<K>),
    Field(Option<K>),
    Operator(Option<SearchOperator>),
    Value(String),
    Apply,
}
struct SearchMachine<K> {
    filters: Vec<SearchFilter<K>>,
}
impl<K: Clone + Eq + 'static> Machine for SearchMachine<K> {
    type State = PowerSearchState<K>;
    type Event = SearchEvent<K>;
    type Effect = Vec<SearchFilter<K>>;
    fn transition(
        &self,
        state: &PowerSearchState<K>,
        event: SearchEvent<K>,
    ) -> Step<PowerSearchState<K>, SearchEvent<K>, Vec<SearchFilter<K>>> {
        let mut next = state.clone();
        match event {
            SearchEvent::Close => return Step::to(PowerSearchState::Closed),
            SearchEvent::Add => {
                return Step::to(PowerSearchState::Composing {
                    draft: FilterDraft::default(),
                });
            }
            SearchEvent::Edit(index, original) => {
                return Step::to(PowerSearchState::Editing {
                    index,
                    draft: FilterDraft {
                        field: Some(original.field.clone()),
                        operator: Some(original.operator),
                        value: original.value.clone(),
                    },
                    original,
                });
            }
            SearchEvent::Apply => {
                let draft = state.draft();
                let (Some(field), Some(operator)) = (draft.field, draft.operator) else {
                    return Step::stay();
                };
                if draft.value.trim().is_empty() {
                    return Step::stay();
                }
                let filter = SearchFilter {
                    field,
                    operator,
                    value: draft.value,
                };
                let mut entries = self.filters.clone();
                match state {
                    PowerSearchState::Closed => return Step::stay(),
                    PowerSearchState::Composing { .. } => entries.push(filter),
                    PowerSearchState::Editing {
                        index, original, ..
                    } => {
                        let Some(entry) = entries.get_mut(*index) else {
                            return Step::stay();
                        };
                        if entry != original {
                            return Step::stay();
                        }
                        *entry = filter;
                    }
                }
                return Step::to(PowerSearchState::Closed).with(entries);
            }
            event => {
                let draft = match &mut next {
                    PowerSearchState::Closed => return Step::stay(),
                    PowerSearchState::Composing { draft }
                    | PowerSearchState::Editing { draft, .. } => draft,
                };
                match event {
                    SearchEvent::Field(field) => draft.field = field,
                    SearchEvent::Operator(operator) => draft.operator = operator,
                    SearchEvent::Value(value) => draft.value = value,
                    SearchEvent::Close
                    | SearchEvent::Add
                    | SearchEvent::Edit(..)
                    | SearchEvent::Apply => {
                        unreachable!("invariant: structural events return before draft updates")
                    }
                }
            }
        }
        Step::to(next)
    }
}
