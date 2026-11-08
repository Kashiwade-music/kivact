//! Role: Render a shared themed tab list with optional selected panel lifetime.
//!
//! As a plain view the builder draws only the tab list. With
//! `.content(factory)` it also draws the panel of the selected valid key in a
//! new owner each time the selection changes: beside the list when vertical,
//! under it otherwise, with the recipes laying the tabs out by orientation
//! (`TabListState` and `TabState`). With `keep_mounted()` each option's
//! panel is built once and the tab panel part hides it while it is not
//! selected. A selected key that is not among the options stays in the
//! binding, but the factory is not called for it. A kit's domain tab bar
//! (close, reorder, icons, counters) moves and selects with the same
//! interaction parts and draws its ordinary parts with `UiRecipes::tabs`.
use super::layer::{recipes, with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::state::{Binding, Value};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::style::use_theme;
use kivact_core::view::{IntoView, Orientation, View, dynamic, for_each, show};
use kivact_core::view::{Node, NodeBuilder, column, node, row, text};
use kivact_interaction::TabState;
use kivact_interaction::{ListActivation, ListItem, TabsApi as Behavior};
use std::rc::Rc;

/// What a tab shows, from its item and its state.
type TabContent<K> = kivact_core::view::ViewFn<(Value<ListItem<K>>, Value<TabState>)>;

// xtask-allow(state-fields): Optional panel, item and activation configuration; selection and focus belong to TabListApi.
pub struct Tabs<K: kivact_core::state::ItemKey> {
    root: Node,
    options: Value<Vec<ListItem<K>>>,
    binding: Binding<K>,
    on_edit: Option<kivact_core::state::Callback<kivact_core::view::EditPhase>>,
    activation: ListActivation,
    orientation: Orientation,
    item: Option<TabContent<K>>,
    content: Option<kivact_core::view::ViewFn<K>>,
    variants: Variants,
    keep_mounted: bool,
}
/// A themed tab list whose selected key is `binding`; give it
/// [`content`](Tabs::content) when the tabs also own the selected panel.
///
/// `label` is the tab list's accessible name; it is not drawn.
///
/// A tab list always has a selected tab, so the binding holds a key, not an
/// `Option`: start it at the first tab's key.
///
/// Each tab shows its item's icon and label; [`item`](Tabs::item) replaces
/// what a tab shows, keeping its keys, semantics and recipe.
pub fn tabs<K: kivact_core::state::ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    binding: impl kivact_core::state::IntoBinding<K>,
) -> Tabs<K> {
    let variants = Variants::default();
    Tabs {
        root: node().name(label),
        options: options.into_value(),
        binding: binding.into_binding(),
        on_edit: None,
        activation: ListActivation::Automatic,
        orientation: Orientation::Horizontal,
        item: None,
        content: None,
        variants,
        keep_mounted: false,
    }
}
impl<K: kivact_core::state::ItemKey> Tabs<K> {
    /// Observe the beginning and commit of each value edit.
    pub fn on_edit<R: kivact_core::view::HandlerResult>(
        mut self,
        f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
    ) -> Self {
        self.on_edit = Some(kivact_core::state::callback(f));
        self
    }

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
    pub fn variant(mut self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}
impl<K: kivact_core::state::ItemKey> Tabs<K> {
    pub fn activation(mut self, activation: ListActivation) -> Self {
        self.activation = activation;
        self
    }
    /// The direction the tabs are laid out in (default
    /// [`Orientation::Horizontal`]). [`Orientation::Vertical`] lays them out
    /// in a column (Up and Down move between them) and, with
    /// [`content`](Self::content), beside their panel. The recipes read the
    /// orientation from [`TabListState`] and
    /// [`TabState`](kivact_interaction::TabState).
    ///
    /// A constant: the orientation is the keyboard policy of the tab list
    /// and the shape of the layout, both fixed when the tabs are built.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
    /// What each tab shows, built once per tab from its item and its state
    /// (both reactive: a renamed item or a selected tab updates the view).
    /// The tab itself, its keys, its semantics and its recipe stay the
    /// component's; without a slot a tab shows its icon and label.
    ///
    /// ```no_run
    /// use kivact::prelude::*;
    ///
    /// fn inbox(unread: Signal<u32>) -> impl IntoView {
    ///     let selected = signal("inbox");
    ///     tabs("Mailbox", vec![ListItem::new("inbox", "Inbox"), ListItem::new("sent", "Sent")], selected)
    ///         .item(move |item, _state| {
    ///             row((
    ///                 text(move || item.get().label()),
    ///                 text(move || match item.get().key() {
    ///                     &"inbox" => unread.get().to_string(),
    ///                     _ => String::new(),
    ///                 }),
    ///             ))
    ///             .style(style().gap(6))
    ///         })
    /// }
    /// ```
    pub fn item<V: IntoView>(
        mut self,
        content: impl Fn(Value<ListItem<K>>, Value<TabState>) -> V + 'static,
    ) -> Self {
        self.item = Some(kivact_core::view::ViewFn::from(move |(item, state)| {
            content(item, state)
        }));
        self
    }
    fn into_list(self) -> (Node, Behavior<K>) {
        let behavior = kivact_interaction::use_tabs(self.options, self.binding)
            .activation(self.activation)
            .orientation(self.orientation);
        if let Some(observer) = self.on_edit {
            behavior.on_edit(move |phase| observer.call(phase));
        }
        let part = behavior.list();
        let list_state = part.state();
        let slot = self.item;
        let variants = self.variants.clone();
        let labels = for_each(
            self.options,
            |item| item.key().clone(),
            move |item| {
                let key = item.with_untracked(|item| item.key().clone());
                let part = behavior.item(key);
                let state = part.state();
                let content = match &slot {
                    Some(slot) => slot.call((item, state)),
                    None => (
                        super::helpers::item_icon(move || item.get().icon().cloned()),
                        text(move || item.get().label()),
                    )
                        .into_view(),
                };
                let indicator_variants = variants.clone();
                let indicator = show(
                    || {
                        recipes(&use_theme().get())
                            .tabs_recipes()
                            .has_custom_indicator()
                    },
                    move || {
                        with_recipe_style(
                            node().hit_testable(false),
                            |r| r.tabs_recipes().indicator_recipe(),
                            &indicator_variants,
                            move || state.get(),
                        )
                    },
                );
                with_recipe_style(
                    node().part(part),
                    |r| r.tabs_recipes().tab_recipe(),
                    &variants,
                    move || state.get(),
                )
                .child(content)
                .child(indicator)
            },
        );
        let root = with_base_recipe_style(
            self.root,
            |r| r.tabs_recipes().list_recipe(),
            &self.variants,
            move || list_state.get(),
        );
        let list = root.part(part).child(labels);
        let list = if self.orientation == Orientation::Vertical {
            list.style(kivact_core::style::style().direction(kivact_core::style::Direction::Column))
        } else {
            list
        };
        (list, behavior)
    }
    /// Show the panel `factory` builds for the selected key under the list
    /// (beside it when vertical), in a fresh owner each time the selection
    /// changes. A selected key that is not among the options builds nothing.
    pub fn content<V: IntoView>(mut self, factory: impl Fn(K) -> V + 'static) -> Self {
        self.content = Some(kivact_core::view::ViewFn::from(factory));
        self
    }
    /// Build each option's panel once and keep it mounted, hidden while its
    /// tab is not selected, so the state inside a panel (a draft, a scroll
    /// position) survives switching tabs (Radix's `forceMount`). A panel is
    /// built when its option appears and dropped when it leaves the options.
    ///
    /// A constant: it chooses whether a panel's owner lives with the
    /// selection or with the option, fixed when the tabs are built.
    pub fn keep_mounted(mut self) -> Self {
        self.keep_mounted = true;
        self
    }
    fn with_panels(mut self) -> View {
        let Some(factory) = self.content.take() else {
            return self.into_list().0.into_view();
        };
        let options = self.options;
        let vertical = self.orientation == Orientation::Vertical;
        let variants = self.variants.clone();
        let keep = self.keep_mounted;
        let (list, behavior) = self.into_list();
        let selected = behavior.selected();
        // One panel: the tab panel part, named by its tab, around the
        // factory's view; the part hides it while its tab is not selected.
        let panel = Rc::new(move |key: K, factory: &kivact_core::view::ViewFn<K>| {
            let named = key.clone();
            let name = move || {
                options.with(|options| {
                    options
                        .iter()
                        .find(|item| *item.key() == named)
                        .map(|item| item.label())
                        .unwrap_or_default()
                })
            };
            let part = behavior.panel(key.clone());
            let state = part.state();
            with_recipe_style(
                node().part(part).name(name).child(factory.call(key)),
                |r| r.tabs_recipes().panel_recipe(),
                &variants,
                move || state.get(),
            )
        });
        if keep {
            let panels = for_each(
                options,
                |item| item.key().clone(),
                move |item| panel(item.with_untracked(|item| item.key().clone()), &factory),
            );
            return if vertical {
                row((list, panels)).into_view()
            } else {
                column((list, panels)).into_view()
            };
        }
        let present = kivact_core::state::memo(move || {
            let key = selected.get();
            options
                .with(|rows| rows.iter().any(|row| *row.key() == key))
                .then_some(key)
        });
        let panels = dynamic(present, move |key| key.map(|key| panel(key, &factory)));
        // Vertical tabs sit beside their panel, horizontal ones above it.
        if vertical {
            row((list, panels)).into_view()
        } else {
            column((list, panels)).into_view()
        }
    }
}
impl<K: kivact_core::state::ItemKey> NodeBuilder for Tabs<K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl<K: kivact_core::state::ItemKey> IntoView for Tabs<K> {
    fn into_view(self) -> View {
        self.with_panels()
    }
}
