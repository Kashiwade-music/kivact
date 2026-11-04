//! Role: Render a themed selectable list over the shared keyed list parts.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::state::{Binding, Callback, Value, binding};
use kivact_core::state::{IntoText, IntoValue, callback};
use kivact_core::view::{HandlerResult, IntoView, View, for_each};
use kivact_core::view::{Node, NodeBuilder, node};
use kivact_interaction::{
    EditPhase, EditSession, ListItem, ListItemState, ListPolicy, ListSelection, use_list,
};
use std::hash::Hash;
use std::rc::Rc;

/// What an item shows, from its data and its state.
type ItemContent<K> = Rc<dyn Fn(Value<ListItem<K>>, Value<ListItemState>) -> View>;

/// A list of items the user picks from (`Role::ListBox`), created by
/// [`list`].
pub struct List<K: Clone + Eq + Hash + 'static> {
    root: Node,
    items: Value<Vec<ListItem<K>>>,
    selected: Binding<Option<K>>,
    multiple: Option<Binding<Vec<K>>>,
    on_edit: Option<Callback<EditPhase>>,
    item: Option<ItemContent<K>>,
    variants: Variants,
}

/// A themed list named `label` over `items` that writes the picked key to
/// `selected`.
///
/// Each item is a `Role::Option` named by its label, showing its icon, its
/// label and its description. The arrow keys move the focus and select
/// (Shift extends in a [`multiple`](List::multiple) list), Tab enters at the
/// selected item, a press or Space selects, typing a name jumps to the item
/// that starts with it, and a disabled item is skipped. The label is the
/// list's accessible name; it is not drawn. [`item`](List::item) replaces
/// what an item shows.
///
/// ```no_run
/// use kivact::prelude::*;
///
/// fn colors() -> impl IntoView {
///     let selected = signal(Some("red"));
///     list(
///         "Colors",
///         vec![ListItem::new("red", "Red"), ListItem::new("blue", "Blue")],
///         selected,
///     )
/// }
/// ```
#[track_caller]
pub fn list<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    items: impl IntoValue<Vec<ListItem<K>>>,
    selected: impl Into<Binding<Option<K>>>,
) -> List<K> {
    // The variants are shared: `.variant` adds to them before mount.
    let variants = Variants::default();
    List {
        root: node().name(label),
        items: items.into_value(),
        selected: selected.into(),
        multiple: None,
        on_edit: None,
        item: None,
        variants,
    }
}

impl<K: Clone + Eq + Hash + 'static> List<K> {
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }

    /// Let the user pick several items, keeping the picked keys in
    /// `selection`: a press selects one, Ctrl (Cmd) toggles one and Shift
    /// extends from the last one. The list then reads and writes
    /// `selection` only; the single `selected` binding given to [`list`] is
    /// not used.
    pub fn multiple(mut self, selection: impl Into<Binding<Vec<K>>>) -> Self {
        self.multiple = Some(selection.into());
        self
    }

    /// Report each change of the selection as one edit: `Begin`, the write,
    /// `Commit` ([`EditPhase`]). A change writes once, so there is nothing
    /// to cancel.
    pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.on_edit = Some(callback(f));
        self
    }

    /// What each item shows, built once per item from its data and its
    /// state (both reactive: a renamed item or a selected one updates the
    /// view). The item itself, its keys, its semantics and its recipe stay
    /// the component's; without a slot an item shows its icon, its label and
    /// its description.
    ///
    /// ```no_run
    /// use kivact::prelude::*;
    ///
    /// fn files() -> impl IntoView {
    ///     let selected = signal(Some(1_u32));
    ///     list(
    ///         "Files",
    ///         vec![ListItem::new(1, "notes.txt"), ListItem::new(2, "plan.md")],
    ///         selected,
    ///     )
    ///     .item(|item, state| {
    ///         text(move || {
    ///             let mark = if state.get().selected { "> " } else { "" };
    ///             format!("{mark}{}", item.get().label())
    ///         })
    ///     })
    /// }
    /// ```
    pub fn item<V: IntoView>(
        mut self,
        content: impl Fn(Value<ListItem<K>>, Value<ListItemState>) -> V + 'static,
    ) -> Self {
        self.item = Some(Rc::new(move |item, state| content(item, state).into_view()));
        self
    }
}

impl<K: Clone + Eq + Hash + 'static> NodeBuilder for List<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

/// `value` whose writes are each reported to `on_edit` as one edit.
fn edited<T: Clone + PartialEq + 'static>(
    value: Binding<T>,
    on_edit: Option<Callback<EditPhase>>,
) -> Binding<T> {
    let Some(on_edit) = on_edit else {
        return value;
    };
    let edit = EditSession::new(move |initial: T| value.set(initial));
    edit.on_edit(move |phase| on_edit.call(phase));
    binding(value.into_value(), move |next: T| {
        edit.once(value.get(), || value.set(next));
    })
}

impl<K: Clone + Eq + Hash + 'static> IntoView for List<K> {
    fn into_view(self) -> View {
        let List {
            root,
            items,
            selected,
            multiple,
            on_edit,
            item: slot,
            variants,
        } = self;
        let selection = match multiple {
            Some(keys) => ListSelection::Multiple(edited(keys, on_edit)),
            None => ListSelection::Single(edited(selected, on_edit)),
        };
        let behavior = use_list(
            items,
            selection,
            ListPolicy {
                typeahead: true,
                ..ListPolicy::default()
            },
        );
        let container = behavior.list();
        let list_state = container.state();
        let looks = variants.clone();
        let rows = for_each(
            items,
            |item| item.key().clone(),
            move |key, item| {
                let part = behavior.item(key);
                let state = part.state();
                let content = match &slot {
                    Some(slot) => slot(item, state),
                    None => (
                        super::helpers::item_icon(move || item.get().icon_value().cloned()),
                        super::helpers::label_and_description(
                            move || item.get().label(),
                            move || item.get().description_value(),
                            {
                                let looks = looks.clone();
                                move |line| {
                                    with_recipe_style(
                                        line,
                                        |r| r.list_recipes().description_recipe(),
                                        &looks,
                                        move || state.get(),
                                    )
                                }
                            },
                        ),
                    )
                        .into_view(),
                };
                with_recipe_style(
                    node().part(part),
                    |r| r.list_recipes().item_recipe(),
                    &looks,
                    move || state.get(),
                )
                .child(content)
            },
        );
        // The recipe is the base style, so the caller's styles win.
        with_base_recipe_style(
            root,
            |r| r.list_recipes().root_recipe(),
            &variants,
            move || list_state.get(),
        )
        .part(container)
        .child(rows)
        .into_view()
    }
}
