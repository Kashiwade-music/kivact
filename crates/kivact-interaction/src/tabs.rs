//! Role: Tabs attach keyed semantics and navigation to caller-owned elements.
//!
//! [`use_tabs`] is a Copy handle over one [`ListApi`] with the tabs policy;
//! [`TabsApi::list`], [`TabsApi::item`] and [`TabsApi::panel`] return parts. The
//! list publishes the `TabList` orientation, a tab its name, `selected`,
//! disabled state and the panel it `controls`, and a panel is
//! `labelled_by` its tab and focusable; the key's node references are shared
//! under the owner. A panel shows only while its key is selected and still
//! among the options. An empty selection offers the first enabled key to the
//! binding, and a selected key that is missing stays selected. Activation
//! and the layout direction may be set before or after the parts. The caller
//! decides the panels' lifetime with a keyed `for_each` or `show`; keeping
//! panels mounted is the renderer's choice. A segmented control is a radio
//! group (the list's radio policy), not tabs, and a kit's domain tab bar with
//! close, reorder, icons or counters moves and selects through these parts
//! too.

use crate::Selection;
use crate::options_cell::OptionsCell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use kivact_core::custom::{Owned, Owner, Part, PartState, part};
use kivact_core::state::IntoValue;
use kivact_core::state::{Binding, Value, memo, var};
use kivact_core::view::{NodeRef, Orientation, Role};

use super::axis::Axis;
use super::{ListActivation, ListApi, ListItem, ListOptions, use_list};

/// What the setters of [`TabsApi`] store; read where the parts use it.
#[derive(Clone, Copy)]
struct TabsOptions {
    activation: ListActivation,
    axis: Axis,
}

struct Refs<K> {
    map: RefCell<HashMap<K, (NodeRef, NodeRef)>>,
}

/// The selected state and actual input state of a tab or panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabState {
    pub input: PartState,
    pub selected: bool,
    /// How the tab list is laid out ([`TabsApi::orientation`]); a look lays a
    /// tab out by it (full height in a row, full width in a column).
    pub orientation: Orientation,
}

/// The actual input state and the orientation of a tab list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TabListState {
    pub input: PartState,
    /// How the tabs are laid out ([`TabsApi::orientation`]).
    pub orientation: Orientation,
}

/// Copy controller for caller-rendered tabs and panels.
/// Render keyed rows with `for_each`, attaching `item(key)` to each row.
/// Render panels with `show` to recreate them, or `for_each` to retain them.
/// A tab list always has a selected key: the caller's binding holds one. A
/// selected key missing from the options stays selected; no panel is shown
/// for it.
///
/// ```no_run
/// use kivact_core::{state::signal, view::{Element, column, node, text, for_each}};
/// use kivact_interaction::{use_tabs, ListItem};
/// let options = signal(vec![ListItem::new("general", "General"),
///     ListItem::new("keys", "Keys")]);
/// let selected = signal("general");
/// let tabs = use_tabs(options, selected);
/// let list = node().part(tabs.list()).child(for_each(options, |row| *row.key(), move |row| {
///     let key = row.with(|row| *row.key());
///     node().part(tabs.item(key)).child(text(move || row.with(|row| row.label())))
/// }));
/// // Keeping every panel mounted is a caller rendering decision.
/// let panels = for_each(options, |row| *row.key(), move |item| {
///     let key = item.with(|item| *item.key());
///     node().part(tabs.panel(key)).child(text(format!("Settings: {key}")))
/// });
/// let view = column((list, panels));
/// ```
pub struct TabsApi<K: kivact_core::state::ItemKey> {
    group: ListApi<K>,
    options: Value<Vec<ListItem<K>>>,
    selected: Binding<K>,
    edit: crate::EditSession<K>,
    owner: Owner,
    refs: Owned<Rc<Refs<K>>>,
    settings: OptionsCell<TabsOptions>,
    /// The layout direction, derived from the settings.
    orientation: Value<Orientation>,
}

impl<K: kivact_core::state::ItemKey> Clone for TabsApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for TabsApi<K> {}

/// Create keyed tabs whose selected key is `value`. Setup only.
#[track_caller]
pub fn use_tabs<K: kivact_core::state::ItemKey>(
    items: impl IntoValue<Vec<ListItem<K>>>,
    value: impl kivact_core::state::IntoBinding<K>,
) -> TabsApi<K> {
    let owner = Owner::current();
    let options = items.into_value();
    let original: Binding<K> = value.into_binding();
    let edit = crate::EditSession::new(move |value| original.set(value));
    let selected = kivact_core::state::binding(original.into_value(), move |value: K| {
        edit.once(original.get_untracked(), || original.set(value))
    });
    // The list's single selection is optional; the tabs always hold a key,
    // so clearing it is ignored.
    let group = use_list(
        options,
        Selection::Single(selected.optional()),
        ListOptions::tabs(),
    );
    let settings = OptionsCell::new(TabsOptions {
        activation: ListActivation::Automatic,
        axis: Axis::Horizontal,
    });
    let settings_value = settings.value();
    let orientation = memo(move || settings_value.with(|settings| settings.axis.orientation()));
    let refs = owner.store(
        "Tabs",
        Rc::new(Refs::<K> {
            map: RefCell::new(HashMap::new()),
        }),
    );
    TabsApi {
        edit,
        group,
        options,
        selected,
        owner,
        refs,
        settings,
        orientation,
    }
}

impl<K: kivact_core::state::ItemKey> TabsApi<K> {
    /// Observe a discrete selection edit.
    pub fn on_edit<R: kivact_core::view::HandlerResult>(
        self,
        f: impl Fn(kivact_core::view::EditPhase) -> R + 'static,
    ) -> Self {
        self.edit.on_edit(f);
        self
    }
    /// Drawing state of the mounted tab list.
    pub fn list_state(self) -> Value<TabListState> {
        let input = self.group.list_state();
        let orientation = self.orientation;
        memo(move || TabListState {
            input: input.get(),
            orientation: orientation.get(),
        })
    }
    /// Drawing state of an individual tab.
    pub fn item_state(self, key: K) -> Value<TabState> {
        let input = self.group.item_state(key);
        let orientation = self.orientation;
        memo(move || {
            let input = input.get();
            TabState {
                input: input.input,
                selected: input.selected,
                orientation: orientation.get(),
            }
        })
    }

    /// Whether moving to a tab with the arrow keys also selects it.
    /// [`Automatic`](ListActivation::Automatic) is the default;
    /// [`Manual`](ListActivation::Manual) only moves focus and Enter or
    /// Space selects. Setup only.
    #[track_caller]
    pub fn activation(self, activation: ListActivation) -> Self {
        self.settings
            .update(|settings| settings.activation = activation);
        self.tune()
    }

    /// The direction the tabs are laid out in (default
    /// [`Orientation::Horizontal`]): in a vertical list Up and Down move
    /// between them, and the list publishes the orientation.
    /// `Orientation::Unspecified` is a usage error. Setup only.
    #[track_caller]
    pub fn orientation(self, orientation: Orientation) -> Self {
        let axis = Axis::of(orientation, "TabsApi::orientation");
        self.settings.update(|settings| settings.axis = axis);
        self.tune()
    }

    /// Give the list the settings' activation and axis.
    #[track_caller]
    fn tune(self) -> Self {
        let TabsOptions { activation, axis } = self.settings.get();
        self.group.retune(|policy| {
            policy.activation = activation;
            policy.axis = Some(axis.orientation());
        });
        self
    }

    /// Caller-owned selected key, including a key missing from the options.
    pub fn selected(self) -> Value<K> {
        self.selected.into_value()
    }

    #[track_caller]
    fn refs(&self, key: &K) -> (NodeRef, NodeRef) {
        let state = self.refs.get();
        if let Some(pair) = state.map.borrow().get(key) {
            return *pair;
        }
        let pair = (self.owner.node_ref(), self.owner.node_ref());
        state.map.borrow_mut().insert(key.clone(), pair);
        pair
    }

    /// Tab-list semantics and keyboard orientation without layout or children.
    #[track_caller]
    pub fn list(self) -> Part<TabListState> {
        let orientation = self.orientation;
        let base = self.group.list();
        let part_state = base.state();
        Part::new(part_state, move |host| {
            host.part(base).role(Role::TabList).orientation(orientation)
        })
        .map_state(move |input| TabListState {
            input,
            orientation: orientation.get(),
        })
    }

    /// A tab with reactive label, selection, constraints and roving navigation.
    #[track_caller]
    pub fn item(self, key: K) -> Part<TabState> {
        let member = self.group.item(key.clone());
        let (tab_ref, panel_ref) = self.refs(&key);
        let options = self.options;
        let key = var(key);
        let label = memo(move || {
            options.with(|rows| {
                key.with(|key| {
                    rows.iter()
                        .find(|row| &row.key == key)
                        .map(|row| row.label.get())
                        .unwrap_or_default()
                })
            })
        });
        let orientation = self.orientation;
        let base = member.map_state(move |state| TabState {
            input: state.input,
            selected: state.selected,
            orientation: orientation.get(),
        });
        let part_state = base.state();
        Part::new(part_state, move |host| {
            host.part(base)
                .role(Role::Tab)
                .name(label)
                .controls(panel_ref)
                .node_ref(tab_ref)
        })
    }

    /// A panel hidden unless its key is selected and still exists in options.
    #[track_caller]
    pub fn panel(self, key: K) -> Part<TabState> {
        let (tab_ref, panel_ref) = self.refs(&key);
        let key = var(key);
        let selected = self.selected();
        let options = self.options;
        let orientation = self.orientation;
        let shown = memo(move || {
            key.with(|key| {
                selected.with(|selected| selected == key)
                    && options.with(|rows| rows.iter().any(|row| &row.key == key))
            })
        });
        let input = part(move |input| TabState {
            input,
            selected: shown.get(),
            orientation: orientation.get(),
        });
        let part_state = input.state();
        Part::new(part_state, move |host| {
            host.part(input)
                .role(Role::TabPanel)
                .hidden(move || !shown.get())
                .focusable(shown)
                .labelled_by(tab_ref)
                .node_ref(panel_ref)
        })
    }
}
