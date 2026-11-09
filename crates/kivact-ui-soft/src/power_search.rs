//! Role: Compose named search filters with an explicit editor and remove controls.

use kivact::i18n::message;
use kivact::interaction::DialogMode;
use kivact::interaction::ListItem;
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::ItemKey;
use kivact::state::Machine;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::machine;
use kivact::style::Align;
use kivact::style::Length;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::button;
use kivact::ui::dialog;
use kivact::ui::select;
use kivact::ui::text_input;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::View;
use kivact::view::for_each;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use std::hash::Hash;

/// A filter over a named field. Values are stored as text for caller-defined search backends.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchFilter<K> {
    /// Key of the field to which this filter applies.
    pub field: K,
    /// Comparison performed against the field value.
    pub operator: SearchOperator,
    /// Text operand interpreted by the caller search backend.
    pub value: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Comparison choices offered by the structured search editor.
pub enum SearchOperator {
    /// Match an equal field value.
    Is,
    /// Exclude an equal field value.
    IsNot,
    /// Match a field value containing the operand.
    Contains,
}

/// A structured search editor bound to a collection of field filters.
pub struct PowerSearch<K: ItemKey> {
    root: Node,
    label: Value<String>,
    fields: Value<Vec<ListItem<K>>>,
    filters: Binding<Vec<SearchFilter<K>>>,
}

/// Build an editor for adding, changing and removing filters over the supplied keyed fields.
pub fn power_search<K: ItemKey>(
    label: impl IntoText,
    fields: impl IntoValue<Vec<ListItem<K>>>,
    filters: impl IntoBinding<Vec<SearchFilter<K>>>,
) -> PowerSearch<K> {
    PowerSearch {
        root: node(),
        label: label.into_text_value(),
        fields: fields.into_value(),
        filters: filters.into_binding(),
    }
}

impl<K: ItemKey> Element for PowerSearch<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<K: ItemKey> IntoView for PowerSearch<K> {
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
        let tokens_view = filter_tokens(fields, filters, editor);
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
            .mode(DialogMode::Popover)
            .hide_title(true)
            .width(Length::Auto)
            .content(move || filter_form(fields, field, operator, value, editor))
            .attach(trigger)
    }
}

#[derive(Clone, PartialEq)]
struct FilterDraft<K> {
    // xtask-allow(state-fields): passive record - field
    field: Option<K>,
    // xtask-allow(state-fields): passive record - operator
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

fn filter_tokens<K: ItemKey>(
    fields: Value<Vec<ListItem<K>>>,
    filters: Binding<Vec<SearchFilter<K>>>,
    editor: kivact::state::MachineHandle<SearchMachine<K>>,
) -> impl IntoView {
    let entries =
        kivact::state::memo(move || filters.with(|filters| (0..filters.len()).collect::<Vec<_>>()));
    for_each(
        entries,
        |index| *index,
        move |entry| {
            let index = entry.get_untracked();
            let name = kivact::state::memo(move || {
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
                let t = kivact::style::use_tokens::<crate::Tokens>();
                style()
                    .height(28)
                    .align_items(Align::Center)
                    .radius(t.radius.element)
                    .background(t.colors.accent_muted)
            })
        },
    )
}

fn filter_form<K: ItemKey>(
    fields: Value<Vec<ListItem<K>>>,
    field: Binding<Option<K>>,
    operator: Binding<Option<SearchOperator>>,
    value: Binding<String>,
    editor: kivact::state::MachineHandle<SearchMachine<K>>,
) -> Node {
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
            field.get().is_none() || operator.get().is_none() || value.get().trim().is_empty()
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
}
