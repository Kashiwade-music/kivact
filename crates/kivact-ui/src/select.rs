//! Role: Render a themed select using the shared keyed selection parts.
use super::layer::with_recipe_style;
use super::values::{ControlSize, SelectTriggerState};
use super::variants::{Variant, Variants};
use kivact_core::control::{Node, NodeBuilder, column, node, scroll, text};
use kivact_core::primitive::NodeExt;
use kivact_core::style::style;
use kivact_core::view::HandlerResult;
use kivact_core::view::{
    IntoText, IntoValue, IntoView, Placement, View, callback, for_each, overlay, show,
};
use kivact_core::{Binding, Callback, Value};
use kivact_interaction::{ListItem, ListItemState, PopupState};
use std::cell::Cell;
use std::hash::Hash;
use std::rc::Rc;

/// What an option row shows, from its item and its state.
type OptionContent<K> = Rc<dyn Fn(Value<ListItem<K>>, Value<ListItemState>) -> View>;

pub struct Select<K: Clone + Eq + Hash + 'static> {
    root: Node,
    label: Value<String>,
    options: Value<Vec<ListItem<K>>>,
    selected: Binding<Option<K>>,
    placeholder: Option<Value<String>>,
    preview: Option<Callback<Option<K>>>,
    icon: Option<View>,
    size: ControlSize,
    state: Rc<Cell<Option<Value<SelectTriggerState>>>>,
    option: Option<OptionContent<K>>,
    variants: Variants,
}
/// A themed keyed select whose value, focus and keys belong to interaction.
///
/// The trigger shows the selected option's label and the theme's
/// `chevron-down` icon; an option row shows its icon, its label over its
/// description and, while it is the selected one, the theme's `check` icon.
/// [`option`](Select::option) replaces what an option row shows, the check
/// included.
pub fn select<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    selected: impl Into<Binding<Option<K>>>,
) -> Select<K> {
    let label = label.into_text_value();
    let state = Rc::new(Cell::new(None::<Value<SelectTriggerState>>));
    let read = state.clone();
    let variants = Variants::default();
    Select {
        root: with_recipe_style(
            node().name(label),
            |r| r.select_recipes().trigger_recipe(),
            &variants,
            move || {
                read.get()
                    .expect("invariant: select trigger prepared")
                    .get()
            },
        ),
        label,
        options: options.into_value(),
        selected: selected.into(),
        placeholder: None,
        preview: None,
        icon: None,
        size: ControlSize::Medium,
        state,
        option: None,
        variants,
    }
}
impl<K: Clone + Eq + Hash + 'static> Select<K> {
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
}
impl<K: Clone + Eq + Hash + 'static> Select<K> {
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.placeholder = Some(text.into_text_value());
        self
    }
    pub fn on_preview<R: HandlerResult>(
        mut self,
        preview: impl Fn(Option<K>) -> R + 'static,
    ) -> Self {
        self.preview = Some(callback(preview));
        self
    }
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    /// The size (default [`ControlSize::Medium`]), given to the trigger
    /// recipe in [`SelectTriggerState::size`]; the recipe sizes the trigger
    /// by it.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// What each option row shows, built once per row from its item and
    /// its state (both reactive). The row itself, its keys, its semantics
    /// and its recipe stay the component's.
    ///
    /// ```no_run
    /// use kivact::prelude::*;
    ///
    /// fn fruit() -> impl IntoView {
    ///     let chosen = signal(None::<&str>);
    ///     let options = vec![ListItem::new("apple", "Apple"), ListItem::new("pear", "Pear")];
    ///     select("Fruit", options, chosen).option(|item, state| {
    ///         text(move || {
    ///             let mark = if state.get().selected { "* " } else { "" };
    ///             format!("{mark}{}", item.get().label())
    ///         })
    ///     })
    /// }
    /// ```
    pub fn option<V: IntoView>(
        mut self,
        content: impl Fn(Value<ListItem<K>>, Value<ListItemState>) -> V + 'static,
    ) -> Self {
        self.option = Some(Rc::new(move |item, state| content(item, state).into_view()));
        self
    }
}
impl<K: Clone + Eq + Hash + 'static> NodeBuilder for Select<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<K: Clone + Eq + Hash + 'static> IntoView for Select<K> {
    fn into_view(mut self) -> View {
        let mut behavior = kivact_interaction::use_select(self.options, self.selected);
        if let Some(text) = self.placeholder {
            behavior = behavior.placeholder(text);
        }
        if let Some(preview) = self.preview {
            behavior = behavior.on_preview(move |key| preview.try_call(key));
        }
        let trigger = behavior.trigger();
        let state = trigger.state();
        let (options, selected, size) = (self.options, self.selected.into_value(), self.size);
        self.state.set(Some(kivact_core::memo(move || {
            let PopupState { input, open, .. } = state.get();
            let placeholder = selected.with(|key| {
                key.as_ref()
                    .is_none_or(|key| options.with(|rows| rows.iter().all(|row| row.key() != key)))
            });
            SelectTriggerState {
                input,
                open,
                placeholder,
                size,
            }
        })));
        let anchor = trigger.node_ref();
        let name = self.root.accessible_name();
        let fallback = self.label;
        let label = kivact_core::memo(move || {
            name.as_ref()
                .map(|read| read())
                .unwrap_or_else(|| fallback.get())
        });
        let variants = self.variants;
        let indicator = with_recipe_style(
            node(),
            |r| r.select_recipes().indicator_recipe(),
            &variants,
            move || state.get(),
        )
        .child(kivact_core::control::icon("chevron-down"));
        let trigger = self
            .root
            .part(trigger)
            .child(self.icon)
            .child(text(behavior.label()).style(style().grow(1).single_line()))
            .child(indicator);
        let options = self.options;
        let slot = self.option;
        let surface = show(behavior.shown(), move || {
            let slot = slot.clone();
            let part = behavior.content();
            let state = part.state();
            let rows = variants.clone();
            let body = for_each(
                options,
                |item| item.key().clone(),
                move |key, item| {
                    let variants = &rows;
                    let lines = rows.clone();
                    let part = behavior.item(key);
                    let state = part.state();
                    let content = match &slot {
                        Some(slot) => slot(item, state),
                        None => (
                            super::helpers::item_icon(move || item.get().icon_value().cloned()),
                            super::helpers::label_and_description(
                                move || item.get().label(),
                                move || item.get().description_value(),
                                move |line| {
                                    with_recipe_style(
                                        line,
                                        |r| r.select_recipes().description_recipe(),
                                        &lines,
                                        move || state.get(),
                                    )
                                },
                            ),
                            with_recipe_style(
                                node(),
                                |r| r.select_recipes().check_recipe(),
                                variants,
                                move || state.get(),
                            )
                            .child(kivact_core::view::dynamic(
                                move || state.get().selected,
                                |selected| selected.then(|| kivact_core::control::icon("check")),
                            )),
                        )
                            .into_view(),
                    };
                    with_recipe_style(
                        node().part(part),
                        |r| r.select_recipes().option_recipe(),
                        variants,
                        move || state.get(),
                    )
                    .child(content)
                },
            );
            overlay(
                anchor,
                Placement::Below,
                with_recipe_style(
                    scroll(column(body)),
                    |r| r.select_recipes().surface_recipe(),
                    &variants,
                    move || state.get(),
                ),
            )
            .part(part)
            .name(label)
            .offset(4.0)
            .at_least_anchor_width(true)
        });
        (trigger, surface).into_view()
    }
}
