//! Role: A searchable, sectioned list of keyed items in a popup.

use crate::ColorName;
use kivact::state::IntoBinding;

use kivact::style::use_theme;
use std::rc::Rc;

use kivact::custom::Reactive;
use kivact::geometry::Point;
use kivact::i18n::message;
use kivact::interaction::ListItem;
use kivact::interaction::{
    ComboboxApi, PopupApi, PopupCloseReason, Selection, use_combobox, use_popup,
};
use kivact::state::{
    Callback, Machine, MachineHandle, Step, Value, Var, binding, effect, machine, memo, on_cleanup,
    signal, untrack, var,
};
use kivact::state::{IntoText, IntoValue, callback};
use kivact::style::{Align, Icon, pct, style};
use kivact::view::{
    Handled, HandlerResult, IntoView, Key, KeyEvent, NodeRef, Overlay, Placement, PointerEvent,
    PointerPhase, Role, View, dynamic, for_each, node_ref, overlay, overlay_at,
};
use kivact::view::{Node, NodeBuilder, column, icon, node, row, scroll, text};

use crate::feedback::glass_style;

/// One choosable row of a [`SearchPicker`].
///
/// The key is the application's identity for the item: the picker hands the
/// key of the picked or previewed item to its callbacks and keeps the active
/// row on the same key while the list changes. Keys must be unique in one
/// picker.
///
/// The texts are [`Reactive`], so a localized `message(..)` follows a locale
/// switch. The label is also the row's accessible name and the description is
/// its accessible description; the search reads the label, the description and
/// the section.
///
/// ```
/// use kivact_ui_creative as ui;
/// let items = vec![
///     ui::PickerItem::new("solid", "Solid").section("Generate").shortcut("Shift S"),
///     ui::PickerItem::new("noise", "Noise")
///         .section("Generate")
///         .description("Fractal noise")
///         .icon("click"),
///     ui::PickerItem::new("blur", "Blur").enabled(false),
/// ];
/// assert_eq!(items.len(), 3);
/// ```
#[derive(Clone, PartialEq)]
pub struct PickerItem<K> {
    /// The application's identity for the item.
    pub key: K,
    /// What the row says; also its accessible name.
    pub label: Reactive<String>,
    // xtask-allow(state-fields): Optional immutable item description.
    /// The heading of the run of rows this one belongs to. Rows with no
    /// section have no heading.
    pub section: Option<Reactive<String>>,
    // xtask-allow(state-fields): Optional immutable item description.
    /// A line of detail after the label; also the row's accessible description.
    pub description: Option<Reactive<String>>,
    // xtask-allow(state-fields): Optional immutable shortcut hint.
    /// A shortcut hint at the end of the row. It does not bind a key.
    pub shortcut: Option<Reactive<String>>,
    // xtask-allow(state-fields): Optional immutable icon.
    /// An icon before the label: a name in the theme's icon set or an image.
    pub icon: Option<Icon>,
    // xtask-allow(state-fields): Independent item availability input.
    /// A disabled item stays in the list but is never active and cannot be
    /// picked.
    pub enabled: bool,
}

impl<K> PickerItem<K> {
    /// An enabled item with this key and label.
    pub fn new(key: K, label: impl IntoText) -> Self {
        Self {
            key,
            label: Reactive::new(label.into_text()),
            section: None,
            description: None,
            shortcut: None,
            icon: None,
            enabled: true,
        }
    }

    /// Put the item under this heading. Neighbouring items with the same
    /// section share one heading.
    pub fn section(mut self, section: impl IntoText) -> Self {
        self.section = Some(Reactive::new(section.into_text()));
        self
    }

    /// Show this line of detail after the label.
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(Reactive::new(description.into_text()));
        self
    }

    /// Show this shortcut hint at the end of the row.
    pub fn shortcut(mut self, shortcut: impl IntoText) -> Self {
        self.shortcut = Some(Reactive::new(shortcut.into_text()));
        self
    }

    /// Show this icon before the label: a name in the theme's icon set
    /// (such as `"click"`) or an image.
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Whether the item can be active and picked. A constant: the picker
    /// hands it on as the core list item's `disabled`, which is plain data
    /// (like `ListItem::disabled`), so a changed flag is a new list.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// A searchable popup list of keyed [`PickerItem`]s, in Creative's look.
///
/// Build it with [`search_picker`] and open it from a button with
/// [`attach`](Self::attach), or at a window point with [`at`](Self::at). The
/// application owns what a pick means: [`on_pick`](Self::on_pick) receives the
/// key, and [`on_preview`](Self::on_preview) lets it try an item out before.
///
/// The popup is a non-modal `Role::Dialog` named like the search field. Its
/// content is built anew on every opening, with an empty query and the focus
/// in the field. The focus stays in the field: the list is a `Role::ListBox`
/// that the field `controls`, and the active row is shown to assistive
/// technology as the field's `active_descendant`; rows cannot be focused. A row
/// is a `Role::Option` named by its label and `selected` while active, and
/// neighbouring items with the same section sit in one `Role::Group` named by
/// the section, below a heading.
///
/// The list shows the items whose label, description or section contains the
/// query, ignoring case and the spaces around the query. The first enabled
/// candidate is active when the popup opens and whenever the query changes.
/// ArrowDown and ArrowUp move the active row to the
/// next and the previous enabled one and wrap at the ends (from no active row,
/// ArrowDown goes to the first and ArrowUp to the last). Moving the pointer
/// onto an enabled row makes it active. Enter or a click on a row picks it and
/// closes the popup; a disabled row can be neither active nor picked. If the
/// active row is filtered out or removed, the first enabled candidate is active.
///
/// Escape, an outside press, Tab and a second press on the button cancel: the
/// popup closes without a pick. After Escape and a pick the focus goes back to
/// the button (`attach`) or to the anchor (`at`); after an outside press or Tab
/// it stays where the user put it.
/// With additional [`column`](Self::column)s, Tab/Shift+Tab and Right/Left
/// cycle through columns with enabled matches instead of closing. The search
/// field keeps focus and all columns share its query.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_creative as ui;
/// fn add_layer(layers: Signal<Vec<&'static str>>) -> impl IntoView {
///     ui::search_picker(vec![
///         ui::PickerItem::new("solid", "Solid").section("Generate"),
///         ui::PickerItem::new("noise", "Noise").section("Generate"),
///         ui::PickerItem::new("blur", "Blur").section("Filter"),
///     ])
///     .placeholder("Type a layer name")
///     .on_pick(move |key| layers.update(|all| all.push(key)))
///     .attach(kivact::ui::button("Add layer").hide_label(true).icon(kivact::view::icon("plus")))
/// }
/// ```
pub struct SearchPicker<K: 'static> {
    parts: Parts<K>,
}

/// What a picker is made of; every opening uses the same reactive sources.
#[derive(Clone)]
struct Parts<K: 'static> {
    items: Value<Vec<PickerItem<K>>>,
    columns: Rc<Vec<Column<K>>>,
    placeholder: Value<String>,
    // xtask-allow(state-fields): Optional immutable application callback.
    pick: Option<Callback<K>>,
    // xtask-allow(state-fields): Optional immutable application callback.
    preview: Option<Callback<Option<K>>>,
    // xtask-allow(state-fields): Optional immutable application callback.
    cancel: Option<Callback<()>>,
}

#[derive(Clone)]
struct Column<K: 'static> {
    title: Value<String>,
    items: Value<Vec<PickerItem<K>>>,
}

/// A searchable picker over `items`. Setup only.
#[track_caller]
pub fn search_picker<K: kivact::state::ItemKey>(
    items: impl IntoValue<Vec<PickerItem<K>>>,
) -> SearchPicker<K> {
    SearchPicker {
        parts: Parts {
            items: items.into_value(),
            columns: Rc::default(),
            placeholder: message("creative.picker.placeholder", "Search"),
            pick: None,
            preview: None,
            cancel: None,
        },
    }
}

impl<K: kivact::state::ItemKey> SearchPicker<K> {
    /// Append a named column to the right of the initial column. Setup only.
    ///
    /// All columns share the search field and keys must be unique across the
    /// whole picker. Tab/Shift+Tab and Right/Left wrap between columns, skipping
    /// columns without enabled matches; entering a column activates its first
    /// enabled match. Up/Down and picking keep the usual combobox behavior.
    /// Titles and item lists follow their reactive values while open.
    #[track_caller]
    pub fn column(
        mut self,
        title: impl IntoText,
        items: impl IntoValue<Vec<PickerItem<K>>>,
    ) -> Self {
        Rc::make_mut(&mut self.parts.columns).push(Column {
            title: title.into_text_value(),
            items: items.into_value(),
        });
        self
    }

    /// The hint in the empty search field, also the accessible name of the
    /// field, the list and the popup. The default is the message
    /// `picker.placeholder` (`Search`). Setup only.
    #[track_caller]
    pub fn placeholder(mut self, text: impl IntoText) -> Self {
        self.parts.placeholder = text.into_text_value();
        self
    }

    /// Receive the key of the item the user picks (Enter, a click or an
    /// assistive technology's Activate).
    pub fn on_pick<R: HandlerResult>(mut self, f: impl Fn(K) -> R + 'static) -> Self {
        self.parts.pick = Some(callback(f));
        self
    }

    /// Try the active item out: `f` receives `Some(key)` each time the active
    /// row changes, and `None` when nothing is active any more and when the
    /// popup ends, so the application can restore what it showed. A pick calls
    /// [`on_pick`](Self::on_pick) first and then `f(None)` once; a cancel calls
    /// [`on_cancel`](Self::on_cancel) first and then `f(None)`.
    pub fn on_preview<R: HandlerResult>(mut self, f: impl Fn(Option<K>) -> R + 'static) -> Self {
        self.parts.preview = Some(callback(f));
        self
    }

    /// Called when the popup closes without a pick: Escape, an outside press,
    /// Tab in a single-column picker, a second press on the button, or the
    /// popup being removed. Setup only.
    #[track_caller]
    pub fn on_cancel(mut self, f: impl Into<kivact::state::Callback<()>>) -> Self {
        self.parts.cancel = Some(f.into());
        self
    }

    /// Open the picker below `trigger`, a button of the caller's such as a
    /// [`button`](kivact::ui::button).
    ///
    /// The trigger gets `expanded` and `controls` for the popup, and a press
    /// on it, or an assistive technology's Expand and Collapse, opens and
    /// closes it. A press handler the trigger already has runs before the
    /// popup toggles; add it before calling `attach`. Setup only.
    #[track_caller]
    pub fn attach<T: IntoView + NodeBuilder>(self, trigger: T) -> View {
        let field = node_ref();
        let popup = use_popup(signal(false)).initial_focus(field);
        let part = popup.trigger();
        let anchor = kivact::view::node_ref();
        let part =
            kivact::custom::Part::new(part.state(), move |node| node.part(part).node_ref(anchor));
        let layer = self.layer(popup, field, move |content| {
            overlay(anchor, Placement::Below, content).offset(5)
        });
        let trigger = trigger.with_node(move |root| root.part(part));
        (trigger, layer).into_view()
    }

    /// Open the picker at a window point, for a picker a shortcut starts.
    ///
    /// Setting `position` opens the picker with its top-left corner there
    /// (flipped and kept inside the window); clearing it closes it, and the
    /// picker clears it when it closes. After Escape and a pick the focus goes
    /// to `anchor`. To open it under the mouse from a shortcut, use the key
    /// event's [`KeyEvent::pointer`](kivact::view::KeyEvent::pointer), which
    /// is `None` while the pointer is outside the window; pick a fallback such
    /// as the centre of the anchor. Setup only.
    ///
    /// ```no_run
    /// use kivact::prelude::*;
    /// use kivact::geometry::{Point};
    /// use kivact::view::{Handled, KeyEvent};
    /// use kivact_ui_creative as ui;
    /// fn viewer() -> impl IntoView {
    ///     let viewer = node_ref();
    ///     let bounds = viewer.rect();
    ///     let at = signal(None::<Point>);
    ///     (
    ///         node()
    ///             .node_ref(viewer)
    ///             .role(Role::Group)
    ///             .name("Viewer")
    ///             .focusable(true)
    ///             .on_key(move |e: KeyEvent| {
    ///                 if e.key() == Key::Char('Q') && e.modifiers().shift {
    ///                     let center = bounds.get_untracked().map(|r| r.center());
    ///                     at.set(e.pointer().or(center));
    ///                     return Handled::Yes;
    ///                 }
    ///                 Handled::No
    ///             }),
    ///         ui::search_picker(vec![ui::PickerItem::new(1, "Solid")]).at(viewer, at),
    ///     )
    /// }
    /// ```
    #[track_caller]
    pub fn at(self, anchor: NodeRef, position: impl IntoBinding<Option<Point>>) -> View {
        let position = position.into_binding();
        let point = position.into_value();
        let open = binding(
            (move || point.get().is_some()).into_value(),
            move |open: bool| {
                if !open {
                    position.set(None);
                }
            },
        );
        let field = node_ref();
        let popup = use_popup(open).anchor(anchor).initial_focus(field);
        let location = (move || point.get().unwrap_or_default()).into_value();
        self.layer(popup, field, move |content| {
            overlay_at(location, Placement::Below, content)
        })
    }

    /// The popup layer: fresh content on every opening, placed by `place`.
    #[track_caller]
    fn layer(
        self,
        popup: PopupApi,
        field: NodeRef,
        place: impl Fn(Node) -> Overlay + 'static,
    ) -> View {
        let parts = self.parts;
        kivact::view::show(popup.is_open(), move || {
            let panel = parts.clone().panel(popup, field);
            place(panel)
                .with_node(move |node| node.part(popup.content()))
                .with_node(move |node| node.role(Role::Dialog))
                .with_node(move |node| node.name(parts.placeholder))
        })
        .into_view()
    }
}

/// The search: the query, trimmed and lowercased, is part of the lowercased
/// label, description or section. An empty query accepts everything.
fn accepts<K>(query: &str, item: &PickerItem<K>) -> bool {
    let has = |text: &Reactive<String>| text.get().to_lowercase().contains(query);
    has(&item.label)
        || item.description.as_ref().is_some_and(has)
        || item.section.as_ref().is_some_and(has)
}

fn available_column<K>(
    lists: &[Vec<PickerItem<K>>],
    query: &str,
    start: usize,
    direction: i32,
) -> Option<usize> {
    let count = lists.len();
    (0..count)
        .map(|offset| {
            if direction > 0 {
                (start + offset) % count
            } else {
                (start + count - offset) % count
            }
        })
        .find(|index| {
            lists[*index]
                .iter()
                .any(|item| item.enabled && accepts(query, item))
        })
}

fn column_availability<K>(lists: &[Vec<PickerItem<K>>], query: &str) -> Vec<bool> {
    lists
        .iter()
        .map(|list| list.iter().any(|item| item.enabled && accepts(query, item)))
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PickerColumn {
    Empty,
    Active(usize),
}
impl PickerColumn {
    fn index(self) -> Option<usize> {
        match self {
            Self::Empty => None,
            Self::Active(index) => Some(index),
        }
    }
}
enum ColumnEvent<K> {
    QueryChanged(Vec<bool>),
    ItemsChanged(Vec<bool>),
    Move(i32, Vec<bool>),
    Activate(usize),
    Preview(Option<(usize, K)>),
    Pick(usize, K),
}
enum ColumnEffect<K> {
    Preview(Option<K>),
    Pick(K),
}
struct ColumnMachine<K>(std::marker::PhantomData<K>);
impl<K: kivact::state::ItemKey> Machine for ColumnMachine<K> {
    type State = PickerColumn;
    type Event = ColumnEvent<K>;
    type Effect = ColumnEffect<K>;
    fn transition(
        &self,
        state: &PickerColumn,
        event: ColumnEvent<K>,
    ) -> Step<PickerColumn, ColumnEvent<K>, ColumnEffect<K>> {
        let (available, movement) = match event {
            ColumnEvent::Preview(item) => {
                return Step::stay().with(ColumnEffect::Preview(
                    item.and_then(|(index, key)| (state.index() == Some(index)).then_some(key)),
                ));
            }
            ColumnEvent::Pick(index, key) => {
                return if state.index() == Some(index) {
                    Step::stay().with(ColumnEffect::Pick(key))
                } else {
                    Step::stay()
                };
            }
            ColumnEvent::Activate(index) => return Step::to(PickerColumn::Active(index)),
            ColumnEvent::QueryChanged(available) | ColumnEvent::ItemsChanged(available) => {
                (available, None)
            }
            ColumnEvent::Move(direction, available) => (available, Some(direction)),
        };
        let count = available.len();
        let current = state.index().unwrap_or(0);
        let (start, direction) = movement.map_or((current, 1), |direction| {
            (
                (current + if direction > 0 { 1 } else { count - 1 }) % count,
                direction,
            )
        });
        let next = (0..count)
            .map(|offset| {
                if direction > 0 {
                    (start + offset) % count
                } else {
                    (start + count - offset) % count
                }
            })
            .find(|index| available[*index])
            .map_or(PickerColumn::Empty, PickerColumn::Active);
        Step::to(next)
    }
}

/// A run of neighbouring items with the same section.
#[derive(Clone, PartialEq)]
struct Run<K> {
    /// The position of the run, which keeps its rows while the list changes.
    index: usize,
    section: Option<Reactive<String>>,
    items: Vec<PickerItem<K>>,
}

/// How one opening of the picker ended.
enum PickerOutcome<K> {
    Picked(K),
    Canceled,
}

/// Where one opening of the picker is: open (with the key last reported to
/// `on_preview`), or settled by a pick or a cancel.
#[derive(Clone, PartialEq)]
enum PickerPhase<K> {
    Open { previewed: Option<K> },
    Settled,
}

/// What the picker reports to the application, once per opening.
struct Outcomes<K: kivact::state::PropValue> {
    phase: Var<PickerPhase<K>>,
    // xtask-allow(state-fields): Optional immutable application callback.
    pick: Option<Callback<K>>,
    // xtask-allow(state-fields): Optional immutable application callback.
    preview: Option<Callback<Option<K>>>,
    // xtask-allow(state-fields): Optional immutable application callback.
    cancel: Option<Callback<()>>,
}

impl<K: kivact::state::PropValue> Clone for Outcomes<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact::state::PropValue> Copy for Outcomes<K> {}

impl<K: kivact::state::PropValue> Outcomes<K> {
    /// While open: report `key` when it differs from the one last reported.
    fn preview(self, key: Option<K>) {
        let changed = self
            .phase
            .with(|phase| matches!(phase, PickerPhase::Open { previewed } if *previewed != key));
        if changed {
            self.phase.set(PickerPhase::Open {
                previewed: key.clone(),
            });
            if let Some(preview) = self.preview {
                preview.call(key);
            }
        }
    }

    /// Once: settle the opening. A pick reports `on_pick`, a cancel
    /// `on_cancel`; then a preview that is still showing ends.
    fn settle(self, outcome: PickerOutcome<K>) {
        let PickerPhase::Open { previewed } = self.phase.get() else {
            return;
        };
        self.phase.set(PickerPhase::Settled);
        match outcome {
            PickerOutcome::Picked(key) => {
                if let Some(pick) = self.pick {
                    pick.call(key);
                }
            }
            PickerOutcome::Canceled => {
                if let Some(cancel) = self.cancel {
                    cancel.call(());
                }
            }
        }
        if previewed.is_some()
            && let Some(preview) = self.preview
        {
            preview.call(None);
        }
    }
}

/// The handles one opening of the picker shares.
struct Session<K: kivact::state::ItemKey> {
    active: Value<Option<K>>,
    combo: ComboboxApi<K>,
    outcomes: Outcomes<K>,
    field: NodeRef,
    // xtask-allow(state-fields): Optional multi-column setup capability.
    column: Option<MachineHandle<ColumnMachine<K>>>,
}

impl<K: kivact::state::ItemKey> Clone for Session<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact::state::ItemKey> Copy for Session<K> {}

impl<K: kivact::state::ItemKey> Parts<K> {
    /// The content of one opening: the search field over the list.
    fn panel(self, popup: PopupApi, field: NodeRef) -> Node {
        let Parts {
            items,
            columns,
            placeholder,
            pick,
            preview,
            cancel,
        } = self;
        let multiple = !columns.is_empty();
        let mut sources = vec![Column {
            title: placeholder,
            items,
        }];
        sources.extend(columns.iter().cloned());
        let titles: Vec<_> = sources.iter().map(|column| column.title).collect();
        let lists = memo(move || {
            sources
                .iter()
                .map(|column| column.items.get())
                .collect::<Vec<_>>()
        });
        let items =
            memo(move || lists.with(|lists| lists.iter().flatten().cloned().collect::<Vec<_>>()));
        let empty = message("creative.picker.empty", "No matches");
        let outcomes = Outcomes {
            phase: var(PickerPhase::Open { previewed: None }),
            pick,
            preview,
            cancel,
        };
        let options = memo(move || {
            items.with(|all| {
                all.iter()
                    .map(|item| {
                        ListItem::new(item.key.clone(), item.label.clone()).disabled(!item.enabled)
                    })
                    .collect::<Vec<_>>()
            })
        });
        let columns = machine(
            PickerColumn::Active(0),
            || ColumnMachine(std::marker::PhantomData),
            move |_, effect| match effect {
                ColumnEffect::Preview(key) => outcomes.preview(key),
                ColumnEffect::Pick(key) => {
                    outcomes.settle(PickerOutcome::Picked(key));
                    popup.close(PopupCloseReason::Selection);
                }
            },
        );
        let combo = column_combobox(options, lists, columns);
        let query = combo.query();
        follow_columns(multiple, lists, query, columns);
        let shown = memo(move || {
            let query = query.get().trim().to_lowercase();
            lists.with(|all| {
                all.iter()
                    .map(|list| {
                        list.iter()
                            .filter(|item| accepts(&query, item))
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
        });
        let session = Session {
            active: combo.active(),
            combo,
            outcomes,
            field,
            column: multiple.then_some(columns),
        };
        follow_active(session.active, lists, columns);
        // Closing without a pick is a cancel.
        on_cleanup(move || session.outcomes.settle(PickerOutcome::Canceled));
        let search = search_field(session, placeholder, move |event| {
            column_key(event, multiple, lists, query, columns)
        });
        let list = picker_list(session, shown, titles, placeholder, multiple);
        picker_panel(search, list, shown, empty, field)
    }
}

fn follow_active<K: kivact::state::ItemKey>(
    active: Value<Option<K>>,
    lists: Value<Vec<Vec<PickerItem<K>>>>,
    columns: MachineHandle<ColumnMachine<K>>,
) {
    // The preview follows the active row.
    effect(move || {
        let key = active.get();
        let item = key.and_then(|key| {
            lists
                .with_untracked(|all| {
                    all.iter()
                        .position(|list| list.iter().any(|item| item.key == key))
                })
                .map(|index| (index, key))
        });
        untrack(|| columns.send(ColumnEvent::Preview(item)));
    });
}

fn column_combobox<K: kivact::state::ItemKey>(
    options: Value<Vec<ListItem<K>>>,
    lists: Value<Vec<Vec<PickerItem<K>>>>,
    columns: MachineHandle<ColumnMachine<K>>,
) -> ComboboxApi<K> {
    let active_column = columns.state();
    let chosen = signal(None::<K>);
    let value = binding(chosen, move |key: Option<K>| {
        let Some(key) = key else { return };
        if let Some(index) = lists.with_untracked(|all| {
            all.iter()
                .position(|list| list.iter().any(|item| item.key == key))
        }) {
            columns.send(ColumnEvent::Pick(index, key));
        }
    });
    use_combobox(options, Selection::Single(value)).filter(move |query, option| {
        let query = query.trim().to_lowercase();
        lists.with(|all| {
            available_column(all, &query, active_column.get().index().unwrap_or(0), 1)
                .and_then(|column| {
                    all[column]
                        .iter()
                        .find(|item| &item.key == option.key())
                        .filter(|item| accepts(&query, item))
                })
                .is_some()
        })
    })
}

fn follow_columns<K: kivact::state::ItemKey>(
    multiple: bool,
    lists: Value<Vec<Vec<PickerItem<K>>>>,
    query: kivact::state::Binding<String>,
    columns: MachineHandle<ColumnMachine<K>>,
) {
    if multiple {
        effect(move || {
            let query = query.get().trim().to_lowercase();
            let available = lists.with_untracked(|all| column_availability(all, &query));
            columns.send(ColumnEvent::QueryChanged(available));
        });
        effect(move || {
            let available = lists
                .with(|all| column_availability(all, &query.get_untracked().trim().to_lowercase()));
            columns.send(ColumnEvent::ItemsChanged(available));
        });
    }
}

fn column_key<K: kivact::state::ItemKey>(
    event: KeyEvent,
    multiple: bool,
    lists: Value<Vec<Vec<PickerItem<K>>>>,
    query: kivact::state::Binding<String>,
    columns: MachineHandle<ColumnMachine<K>>,
) -> Handled {
    if !multiple || event.modifiers().ctrl || event.modifiers().alt || event.modifiers().meta {
        return Handled::No;
    }
    let direction = match event.key() {
        Key::Tab if event.modifiers().shift => -1,
        Key::Tab | Key::ArrowRight => 1,
        Key::ArrowLeft => -1,
        _ => return Handled::No,
    };
    let available = lists.with_untracked(|all| {
        column_availability(all, &query.get_untracked().trim().to_lowercase())
    });
    columns.send(ColumnEvent::Move(direction, available));
    Handled::Yes
}
fn picker_list<K: kivact::state::ItemKey>(
    session: Session<K>,
    shown: Value<Vec<Vec<PickerItem<K>>>>,
    titles: Vec<Value<String>>,
    placeholder: Value<String>,
    multiple: bool,
) -> Node {
    let combo = session.combo;
    if multiple {
        let columns = titles
            .into_iter()
            .enumerate()
            .map(|(index, title)| {
                column((
                    text(title).role(Role::Heading),
                    column_rows(
                        session,
                        memo(move || shown.with(|lists| lists[index].clone())),
                        index,
                    )
                    .role(Role::ListBox)
                    .name(title),
                ))
                .style(style().min_width(240).grow(1))
                .into_view()
            })
            .collect::<Vec<_>>();
        row(columns)
            .part(combo.content())
            .role(Role::Group)
            .name(placeholder)
            .style(style().gap(10))
    } else {
        column_rows(
            session,
            memo(move || shown.with(|lists| lists[0].clone())),
            0,
        )
        .part(combo.content())
        .name(placeholder)
    }
}
fn picker_panel<K: kivact::state::ItemKey>(
    search: Node,
    list: Node,
    shown: Value<Vec<Vec<PickerItem<K>>>>,
    empty: Value<String>,
    field: NodeRef,
) -> Node {
    column((
        search,
        scroll(list).style(style().max_height(320).width(pct(100))),
        dynamic(
            memo(move || shown.with(|lists| lists.iter().all(Vec::is_empty))),
            move |none| {
                none.then(|| {
                    text(empty).style(move || {
                        let t = &use_theme().get();
                        let p = t.ext::<crate::tokens::Tokens>();
                        style().height(p.row).padding_x(7).color(p.dim)
                    })
                })
            },
        ),
    ))
    .style(move || {
        let t = &use_theme().get();
        glass_style(t)
            .gap(6)
            .align_items(Align::Stretch)
            .min_width(260)
    })
    // A press on a part that cannot take the focus (a disabled row, a
    // heading, the scroll bar) leaves the focus on the field.
    .on_pointer_default(move |event: PointerEvent| {
        if matches!(event.phase(), PointerPhase::Down(_)) {
            field.focus();
        }
        Handled::No
    })
}

fn column_rows<K: kivact::state::ItemKey>(
    session: Session<K>,
    shown: Value<Vec<PickerItem<K>>>,
    index: usize,
) -> Node {
    let runs = memo(move || {
        let mut out = Vec::<Run<K>>::new();
        let mut last: Option<Option<String>> = None;
        shown.with(|list| {
            for item in list {
                let section = item.section.as_ref().map(|s| s.get());
                if last.as_ref() != Some(&section) {
                    out.push(Run {
                        index: out.len(),
                        section: item.section.clone(),
                        items: Vec::new(),
                    });
                    last = Some(section);
                }
                if let Some(run) = out.last_mut() {
                    run.items.push(item.clone());
                }
            }
        });
        out
    });
    column(for_each(
        runs,
        |run| (run.index, run.section.is_some()),
        move |run| run_view(session, run, index),
    ))
    .style(style().gap(1))
}

/// The search field with its icon and its keys.
fn search_field<K: kivact::state::ItemKey>(
    session: Session<K>,
    placeholder: Value<String>,
    navigate: impl Fn(KeyEvent) -> Handled + 'static,
) -> Node {
    let search = kivact::ui::text_input(placeholder, session.combo.query());
    let search = if session.column.is_some() {
        search.forward_keys(vec![Key::Tab, Key::ArrowLeft, Key::ArrowRight])
    } else {
        search
    };
    let search = search
        .with_node(move |node| node.part(session.combo.input()))
        .placeholder(placeholder)
        .with_node(move |node| node.node_ref(session.field))
        .with_node(move |node| node.on_key(navigate))
        .with_node(move |node| node.style(style().grow(1).min_width(0)));
    row((
        icon("search")
            .style(style().shrink(0))
            .style(style().color(ColorName::Dim)),
        search,
    ))
    .style(style().gap(6).align_items(Align::Center).padding_left(6))
}

/// One run of items: its rows, in a named group below a heading if it has a
/// section.
fn run_view<K: kivact::state::ItemKey>(
    session: Session<K>,
    run: Value<Run<K>>,
    index: usize,
) -> View {
    let rows = for_each(
        memo(move || run.with(|run| run.items.clone())),
        |item| item.key.clone(),
        move |item: Value<PickerItem<K>>| item_row(session, item, index),
    );
    if run.with_untracked(|run| run.section.is_none()) {
        return rows;
    }
    let name = move || {
        run.with(|run| {
            run.section
                .as_ref()
                .map(|section| section.get())
                .unwrap_or_default()
        })
    };
    column((
        text(name).style(
            style()
                .font_size(10.5)
                .weight(600)
                .padding_x(7)
                .padding_top(6)
                .padding_bottom(2)
                .color(ColorName::Dim),
        ),
        rows,
    ))
    .role(Role::Group)
    .name(name)
    .into_view()
}

fn item_row<K: kivact::state::ItemKey>(
    session: Session<K>,
    item: Value<PickerItem<K>>,
    index: usize,
) -> View {
    let key = item.with_untracked(|item| item.key.clone());
    let part = session.combo.item(key);
    let state = part.state();
    let is_active = memo(move || state.get().active);
    let label = memo(move || item.with(|item| item.label.get()));
    let shortcut = memo(move || item.with(|item| item.shortcut.as_ref().map(|s| s.get())));
    let description = memo(move || item.with(|item| item.description.as_ref().map(|d| d.get())));
    let disabled = memo(move || item.with(|item| !item.enabled));
    let option = row((
        dynamic(memo(move || item.with(|item| item.icon.clone())), |glyph| {
            glyph.map(|glyph| icon(glyph).style(style().shrink(0)))
        }),
        text(label).style(style().grow(1).min_width(0).single_line()),
        dynamic(description, |description| {
            description.map(|description| {
                text(description)
                    .style(style().min_width(0).single_line())
                    .style(style().font_size(11).color(ColorName::Muted))
            })
        }),
        dynamic(shortcut, |shortcut| {
            shortcut.map(|shortcut| {
                text(shortcut).style(style().shrink(0)).style(move || {
                    let t = &use_theme().get();
                    let p = t.ext::<crate::tokens::Tokens>();
                    style()
                        .font_family(p.mono_font.as_str())
                        .font_size(10)
                        .color(p.dim)
                })
            })
        }),
    ))
    .on_press(move || {
        if let Some(column) = session.column {
            column.send(ColumnEvent::Activate(index));
        }
    })
    .part(part)
    .name(label)
    .description(move || description.get().unwrap_or_default())
    .selected(is_active)
    .disabled(disabled)
    .style(move || {
        let t = &use_theme().get();
        let p = t.ext::<crate::tokens::Tokens>();
        style()
            .height(p.row + 3.0)
            .gap(9)
            .padding_x(7)
            .radius(6)
            .align_items(Align::Center)
            .selected(style().background(p.selection).color(p.text_high))
    });
    node()
        .on_pointer_default(move |event: PointerEvent| {
            if !disabled.get()
                && matches!(
                    event.phase(),
                    PointerPhase::Enter | PointerPhase::Move | PointerPhase::Down(_)
                )
                && let Some(column) = session.column
            {
                column.send(ColumnEvent::Activate(index));
            }
            if matches!(event.phase(), PointerPhase::Down(_)) && disabled.get() {
                session.field.focus();
            }
            Handled::No
        })
        .child(option)
        .into_view()
}
