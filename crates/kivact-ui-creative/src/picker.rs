//! Role: A searchable, sectioned list of keyed items in a popup.

use crate::Color;
use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use std::hash::Hash;
use std::rc::Rc;

use kivact::control::{Node, NodeBuilder, column, icon, node, row, scroll, text};
use kivact::draw::Point;
use kivact::i18n::message;
use kivact::interaction::{
    Combobox, ListSelection, Popup, PopupCloseReason, use_combobox, use_popup,
};
use kivact::style::{Align, Icon, pct, style};
use kivact::ui::ListItem;
use kivact::view::{
    Handled, HandlerResult, IntoText, IntoValue, IntoView, Key, KeyEvent, NodeRef, Overlay,
    Placement, PointerEvent, PointerPhase, Reactive, Role, View, callback, dynamic, for_each,
    node_ref, overlay, overlay_at,
};
use kivact::{
    Binding, Callback, Signal, Value, Var, binding, effect, memo, on_cleanup, signal, untrack, var,
};

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
    /// The heading of the run of rows this one belongs to. Rows with no
    /// section have no heading.
    pub section: Option<Reactive<String>>,
    /// A line of detail after the label; also the row's accessible description.
    pub description: Option<Reactive<String>>,
    /// A shortcut hint at the end of the row. It does not bind a key.
    pub shortcut: Option<Reactive<String>>,
    /// An icon before the label: a name in the theme's icon set or an image.
    pub icon: Option<Icon>,
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
///     .attach(kivact::ui::button("Add layer").hide_label(true).icon(kivact::control::icon("plus")))
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
    pick: Option<Callback<K>>,
    preview: Option<Callback<Option<K>>>,
    cancel: Option<Callback<()>>,
}

#[derive(Clone)]
struct Column<K: 'static> {
    title: Value<String>,
    items: Value<Vec<PickerItem<K>>>,
}

/// A searchable picker over `items`. Setup only.
#[track_caller]
pub fn search_picker<K: Clone + Eq + Hash + 'static>(
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

impl<K: Clone + Eq + Hash + 'static> SearchPicker<K> {
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
    pub fn on_cancel<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.parts.cancel = Some(callback(move |()| f()));
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
    pub fn attach<T: IntoView + NodeBuilder>(self, mut trigger: T) -> View {
        let field = node_ref();
        let popup = use_popup(signal(false)).initial_focus(field);
        let part = popup.trigger();
        let anchor = part.node_ref();
        let layer = self.layer(popup, field, move |content| {
            overlay(anchor, Placement::Below, content).offset(5)
        });
        let root = std::mem::replace(trigger.node_mut(), node());
        *trigger.node_mut() = root.part(part);
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
    /// use kivact::draw::{Point};
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
    ///                 if e.key == Key::Char('Q') && e.modifiers.shift {
    ///                     let center = bounds.get_untracked().map(|r| r.center());
    ///                     at.set(e.pointer.or(center));
    ///                     return Handled::Yes;
    ///                 }
    ///                 Handled::No
    ///             }),
    ///         ui::search_picker(vec![ui::PickerItem::new(1, "Solid")]).at(viewer, at),
    ///     )
    /// }
    /// ```
    #[track_caller]
    pub fn at(self, anchor: NodeRef, position: impl Into<Binding<Option<Point>>>) -> View {
        let position = position.into();
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
        popup: Popup,
        field: NodeRef,
        place: impl Fn(Node) -> Overlay + 'static,
    ) -> View {
        let parts = self.parts;
        kivact::view::show(popup.shown(), move || {
            let panel = parts.clone().panel(popup, field);
            place(panel)
                .part(popup.content())
                .role(Role::Dialog)
                .name(parts.placeholder)
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

/// A run of neighbouring items with the same section.
#[derive(Clone, PartialEq)]
struct Run<K> {
    /// The position of the run, which keeps its rows while the list changes.
    index: usize,
    section: Option<Reactive<String>>,
    items: Vec<PickerItem<K>>,
}

/// The handles one opening of the picker shares.
struct Session<K: Clone + Eq + Hash + 'static> {
    active: Value<Option<K>>,
    combo: Combobox<K>,
    /// The key last reported to `on_preview`.
    reported: Var<Option<K>>,
    /// A pick or a cancel already happened.
    settled: Var<bool>,
    field: NodeRef,
    preview: Option<Callback<Option<K>>>,
    cancel: Option<Callback<()>>,
    column: Option<Signal<Option<usize>>>,
}

impl<K: Clone + Eq + Hash + 'static> Clone for Session<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: Clone + Eq + Hash + 'static> Copy for Session<K> {}

impl<K: Clone + Eq + Hash + 'static> Session<K> {
    fn end_preview(&self) {
        if self.reported.with(Option::is_some) {
            self.reported.set(None);
            if let Some(preview) = self.preview {
                preview.call(None);
            }
        }
    }

    /// The popup ends. Without a pick it was a cancel; either way the preview
    /// ends, after `on_cancel`.
    fn finish(&self) {
        if !self.settled.get() {
            self.settled.set(true);
            if let Some(cancel) = self.cancel {
                cancel.call(());
            }
        }
        self.end_preview();
    }
}

impl<K: Clone + Eq + Hash + 'static> Parts<K> {
    /// The content of one opening: the search field over the list.
    fn panel(self, popup: Popup, field: NodeRef) -> Node {
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
        let active_column = signal(Some(0usize));
        let empty = message("creative.picker.empty", "No matches");
        let reported = var(None::<K>);
        let settled = var(false);
        let options = memo(move || {
            items.with(|all| {
                all.iter()
                    .map(|item| {
                        ListItem::new(item.key.clone(), item.label.clone()).disabled(!item.enabled)
                    })
                    .collect::<Vec<_>>()
            })
        });
        let chosen = signal(None::<K>);
        let value = binding(chosen, move |key: Option<K>| {
            let Some(key) = key else { return };
            settled.set(true);
            if let Some(pick) = pick {
                pick.call(key);
            }
            if reported.with(Option::is_some) {
                reported.set(None);
                if let Some(preview) = preview {
                    preview.call(None);
                }
            }
            popup.close(PopupCloseReason::Selection);
        });
        let combo =
            use_combobox(options, ListSelection::Single(value)).filter(move |query, option| {
                let query = query.trim().to_lowercase();
                lists.with(|all| {
                    available_column(all, &query, active_column.get().unwrap_or(0), 1)
                        .and_then(|column| {
                            all[column]
                                .iter()
                                .find(|item| &item.key == option.key())
                                .filter(|item| accepts(&query, item))
                        })
                        .is_some()
                })
            });
        let query = combo.query();
        if multiple {
            effect(move || {
                let query = query.get().trim().to_lowercase();
                let next = lists.with(|all| {
                    available_column(all, &query, active_column.get_untracked().unwrap_or(0), 1)
                });
                if next != active_column.get_untracked() {
                    active_column.set(next);
                }
            });
        }
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
            reported,
            settled,
            field,
            preview,
            cancel,
            column: multiple.then_some(active_column),
        };
        let active = session.active;
        // The preview follows the active row.
        effect(move || {
            let key = active.get();
            untrack(|| {
                if session.reported.with(|last| *last != key) {
                    session.reported.set(key.clone());
                    if let Some(preview) = preview {
                        preview.call(key);
                    }
                }
            });
        });
        on_cleanup(move || session.finish());
        let count = titles.len();
        let search = search_field(session, placeholder, move |event| {
            if !multiple || event.modifiers.ctrl || event.modifiers.alt || event.modifiers.meta {
                return Handled::No;
            }
            let direction = match event.key {
                Key::Tab if event.modifiers.shift => -1,
                Key::Tab | Key::ArrowRight => 1,
                Key::ArrowLeft => -1,
                _ => return Handled::No,
            };
            let query = query.get_untracked().trim().to_lowercase();
            let current = active_column.get_untracked().unwrap_or(0);
            let start = if direction > 0 {
                (current + 1) % count
            } else {
                (current + count - 1) % count
            };
            active_column
                .set(lists.with_untracked(|all| available_column(all, &query, start, direction)));
            Handled::Yes
        });
        let list = if multiple {
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
        };
        column((
            search,
            scroll(list).style(style().max_height(320).width(pct(100))),
            dynamic(
                memo(move || shown.with(|lists| lists.iter().all(Vec::is_empty))),
                move |none| {
                    none.then(|| {
                        text(empty).style(move || {
                            let t = &current_theme();
                            let p = t.get::<crate::tokens::Tokens>();
                            style().height(p.row).padding_x(7).color(p.dim)
                        })
                    })
                },
            ),
        ))
        .style(move || {
            let t = &current_theme();
            glass_style(t)
                .gap(6)
                .align_items(Align::Stretch)
                .min_width(260)
        })
        // A press on a part that cannot take the focus (a disabled row, a
        // heading, the scroll bar) leaves the focus on the field.
        .on_pointer_default(move |event: PointerEvent| {
            if matches!(event.phase, PointerPhase::Down(_)) {
                field.focus();
            }
            Handled::No
        })
    }
}

fn column_rows<K: Clone + Eq + Hash + 'static>(
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
        move |_, run| run_view(session, run, index),
    ))
    .style(style().gap(1))
}

/// The search field with its icon and its keys.
fn search_field<K: Clone + Eq + Hash + 'static>(
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
        .part(session.combo.input())
        .placeholder(placeholder)
        .node_ref(session.field)
        .on_key(navigate)
        .style(style().grow(1).min_width(0));
    row((
        icon("search")
            .style(style().shrink(0))
            .style(style().color(Color::Dim)),
        search,
    ))
    .style(style().gap(6).align_items(Align::Center).padding_left(6))
}

/// One run of items: its rows, in a named group below a heading if it has a
/// section.
fn run_view<K: Clone + Eq + Hash + 'static>(
    session: Session<K>,
    run: Value<Run<K>>,
    index: usize,
) -> View {
    let rows = for_each(
        memo(move || run.with(|run| run.items.clone())),
        |item| item.key.clone(),
        move |_, item: Value<PickerItem<K>>| item_row(session, item, index),
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
                .color(Color::Dim),
        ),
        rows,
    ))
    .role(Role::Group)
    .name(name)
    .into_view()
}

fn item_row<K: Clone + Eq + Hash + 'static>(
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
                    .style(style().font_size(11).color(Color::Muted))
            })
        }),
        dynamic(shortcut, |shortcut| {
            shortcut.map(|shortcut| {
                text(shortcut).style(style().shrink(0)).style(move || {
                    let t = &current_theme();
                    let p = t.get::<crate::tokens::Tokens>();
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
            column.set(Some(index));
        }
    })
    .part(part)
    .name(label)
    .description(move || description.get().unwrap_or_default())
    .selected(is_active)
    .disabled(disabled)
    .style(move || {
        let t = &current_theme();
        let p = t.get::<crate::tokens::Tokens>();
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
                    event.phase,
                    PointerPhase::Enter | PointerPhase::Move | PointerPhase::Down(_)
                )
                && let Some(column) = session.column
            {
                column.set(Some(index));
            }
            if matches!(event.phase, PointerPhase::Down(_)) && disabled.get() {
                session.field.focus();
            }
            Handled::No
        })
        .child(option)
        .into_view()
}
