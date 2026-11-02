//! Role: Tabs attach keyed semantics and navigation to caller-owned elements.
//!
//! [`use_tabs`] is a Copy handle over one [`List`] with the tabs policy;
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

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use kivact_core::primitive::{Owned, Owner, Part, PartState, part};
use kivact_core::view::{IntoValue, NodeRef, Orientation, Role};
use kivact_core::{Binding, Signal, Value, memo, signal, var};

use super::axis::Axis;
use super::{List, ListActivation, ListItem, ListPolicy, ListSelection, use_list};

/// What the setters of [`TabsApi`] store; read where the parts use it.
#[derive(Clone, Copy)]
struct TabsSettings {
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
/// use kivact_core::{signal, control::{NodeBuilder, column, node, text}, view::for_each};
/// use kivact_interaction::{use_tabs, ListItem};
/// let options = signal(vec![ListItem::new("general", "General"),
///     ListItem::new("keys", "Keys")]);
/// let selected = signal("general");
/// let tabs = use_tabs(options, selected);
/// let list = node().part(tabs.list()).child(for_each(options, |row| *row.key(),
///     move |key, row| node().part(tabs.item(key))
///         .child(text(move || row.with(|row| row.label())))));
/// // Keeping every panel mounted is a caller rendering decision.
/// let panels = for_each(options, |row| *row.key(), move |key, _| {
///     node().part(tabs.panel(key)).child(text(format!("Settings: {key}")))
/// });
/// let view = column((list, panels));
/// ```
pub struct TabsApi<K: Eq + Hash + Clone + 'static> {
    group: List<K>,
    options: Value<Vec<ListItem<K>>>,
    selected: Binding<K>,
    owner: Owner,
    refs: Owned<Refs<K>>,
    settings: Signal<TabsSettings>,
    /// The layout direction, derived from the settings.
    orientation: Value<Orientation>,
}

impl<K: Eq + Hash + Clone + 'static> Clone for TabsApi<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: Eq + Hash + Clone + 'static> Copy for TabsApi<K> {}

/// Create keyed tabs whose selected key is `selected`. Setup only.
#[track_caller]
pub fn use_tabs<K: Clone + Eq + Hash + 'static>(
    options: impl IntoValue<Vec<ListItem<K>>>,
    selected: impl Into<Binding<K>>,
) -> TabsApi<K> {
    let owner = Owner::current("tabs");
    let options = options.into_value();
    let selected: Binding<K> = selected.into();
    // The list's single selection is optional; the tabs always hold a key,
    // so clearing it is ignored.
    let group = use_list(
        options,
        ListSelection::Single(selected.optional()),
        ListPolicy::tabs(),
    );
    let settings = signal(TabsSettings {
        activation: ListActivation::Automatic,
        axis: Axis::Horizontal,
    });
    let orientation = memo(move || settings.with(|settings| settings.axis.orientation()));
    let refs = owner.store(
        "Tabs",
        Rc::new(Refs::<K> {
            map: RefCell::new(HashMap::new()),
        }),
    );
    TabsApi {
        group,
        options,
        selected,
        owner,
        refs,
        settings,
        orientation,
    }
}

impl<K: Clone + Eq + Hash + 'static> TabsApi<K> {
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
        let TabsSettings { activation, axis } = self.settings.get_untracked();
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
        self.group
            .list()
            .role(Role::TabList)
            .orientation(orientation)
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
        member
            .map_state(move |state| TabState {
                input: state.input,
                selected: state.selected,
                orientation: orientation.get(),
            })
            .role(Role::Tab)
            .name(label)
            .controls(panel_ref)
            .reference(tab_ref)
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
        part(move |input| TabState {
            input,
            selected: shown.get(),
            orientation: orientation.get(),
        })
        .role(Role::TabPanel)
        .hidden(move || !shown.get())
        .focusable(shown)
        .labelled_by(tab_ref)
        .reference(panel_ref)
    }
}
