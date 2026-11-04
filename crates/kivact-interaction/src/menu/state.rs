//! Role: Store menu entries and shared session state.

use kivact_core::custom::Reactive;

use crate::PopupApi;
use crate::list::typeahead::Typeahead;
use kivact_core::ErrorKind;
use kivact_core::custom::{PartState, raise};
use kivact_core::geometry::Point;
use kivact_core::state::IntoText;
use kivact_core::state::{Callback, MachineHandle, Signal, Value, Var};
use kivact_core::style::Icon;
use kivact_core::view::NodeRef;

/// Where an open menu was asked to appear.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuOrigin {
    /// At this window point: the pointer of a right click.
    Pointer(Point),
    /// At the trigger: the request came from the keyboard or from assistive
    /// technology, which have no pointer, or from a menu button.
    Anchor,
}

/// The checked state a check or radio item shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuCheck {
    /// A check item ([`MenuEntry::checkbox`]) with this state.
    Checkbox(bool),
    /// A radio item ([`MenuEntry::radio`]) with this state.
    Radio(bool),
}

impl MenuCheck {
    /// Whether the item is checked.
    pub fn is_checked(self) -> bool {
        matches!(self, Self::Checkbox(true) | Self::Radio(true))
    }
}

/// What choosing an item does: part of its row's identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuItemKind {
    /// A plain item ([`MenuEntry::item`]).
    Action,
    /// A check item ([`MenuEntry::checkbox`]).
    Checkbox,
    /// A radio item ([`MenuEntry::radio`]).
    Radio,
    /// An item that opens a submenu ([`MenuEntry::submenu`]).
    Submenu,
}

/// The state of a menu item for the caller's drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuItemState {
    /// Which row is under the held pointer during menu press tracking.
    /// None leaves highlighting to the ordinary focus and hover states.
    pub pressed_over: Option<bool>,
    pub input: PartState,
    /// The checked state of a check or radio item; `None` for other items.
    pub check: Option<MenuCheck>,
    /// A submenu item whose submenu is open.
    pub expanded: bool,
    /// The entry is marked destructive ([`MenuEntry::destructive`]), for a
    /// warning color.
    pub destructive: bool,
}

/// One row of a [`MenuApi`](super::MenuApi): an item, a check or radio item, a
/// submenu, a heading, a separator, or a group of rows.
///
/// An entry is plain data. The key is the application's identity for the row:
/// [`super::MenuApi`] looks rows up by key, hands the key of the chosen item to its
/// callback and keeps focus on the same key when the entries change. Keys must
/// be unique among the entries of one level (a duplicate is reported and the
/// later row is dropped). Separators, headings and groups have no key.
///
/// The constructor chooses the kind of the row: [`item`](Self::item),
/// [`checkbox`](Self::checkbox), [`radio`](Self::radio),
/// [`submenu`](Self::submenu), [`separator`](Self::separator),
/// [`heading`](Self::heading) and [`group`](Self::group). A modifier that does
/// not apply to the kind (a `shortcut` on a separator, `keep_open` on a
/// submenu) is a usage error.
///
/// A [`group`](Self::group) is a run of rows of the same level under an
/// optional heading, like Radix's `Menu.Group` and `Menu.RadioGroup`: the
/// level lists its heading as a [`MenuRow::Label`] and then its rows, and
/// the keys move through them as through the rest of the level. The radio
/// items of one group are one choice; like a check item, a radio item only
/// reports its key, and the application passes the new `checked` values in
/// the next entries.
///
/// The label is a [`Reactive`], so a localized `message(..)` label follows a
/// locale switch, and an entry can be built anywhere (a handler, a memo, a
/// test). Two labels are equal when they are equal constants or the same
/// reactive source.
///
/// ```
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::MenuEntry;
/// let entries = vec![
///     MenuEntry::item("new", "New").with_shortcut("Ctrl N").with_icon("file-plus"),
///     MenuEntry::checkbox("grid", "Show grid", true),
///     MenuEntry::separator(),
///     MenuEntry::group("Align", vec![
///         MenuEntry::radio("left", "Left", true),
///         MenuEntry::radio("right", "Right", false),
///     ]),
///     MenuEntry::heading("Share"),
///     MenuEntry::submenu("export", "Export", vec![
///         MenuEntry::item("png", "PNG").with_description("Raster image"),
///         MenuEntry::item("svg", "SVG").disabled(true),
///     ]),
/// ];
/// assert_eq!(entries.len(), 6);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MenuEntry<K> {
    kind: EntryKind<K>,
    /// What the row says; its accessible name and the text typeahead matches.
    /// Empty for a separator.
    pub(super) label: Reactive<String>,
    /// The row keeps the room of a check mark before its content, so that
    /// it lines up with the check and radio items of its level (Radix's
    /// `inset`). Only the drawing reads it.
    inset: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum EntryKind<K> {
    Item(ItemEntry<K>),
    Separator,
    Heading,
    Group(Vec<MenuEntry<K>>),
}

#[derive(Clone, Debug, PartialEq)]
struct ItemEntry<K> {
    /// The application's identity for the row.
    key: K,
    role: ItemRole<K>,
    /// The shortcut text, published as the item's accessible description.
    shortcut: String,
    /// A disabled item cannot be chosen.
    disabled: bool,
    disabled_reason: Option<Reactive<String>>,
    /// An icon shown before the label: a name the theme resolves or an image.
    icon: Option<Icon>,
    /// A secondary line under the label.
    description: Option<Reactive<String>>,
    /// A destructive item (delete, remove): only the drawing reads it.
    destructive: bool,
    /// Choosing the item leaves the menu open (a check item toggled in a
    /// row of several).
    keep_open: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum ItemRole<K> {
    Action,
    Check(MenuCheck),
    Submenu(Vec<MenuEntry<K>>),
}

impl<K> MenuEntry<K> {
    fn of_item(key: K, label: impl IntoText, role: ItemRole<K>) -> Self {
        Self {
            kind: EntryKind::Item(ItemEntry {
                key,
                role,
                shortcut: String::new(),
                disabled: false,
                disabled_reason: None,
                icon: None,
                description: None,
                destructive: false,
                keep_open: false,
            }),
            label: Reactive::new(label.into_text()),
            inset: false,
        }
    }

    /// An item the user can choose. The label is its accessible name and the
    /// text typeahead matches.
    pub fn item(key: K, label: impl IntoText) -> Self {
        Self::of_item(key, label, ItemRole::Action)
    }

    /// A check item (`Role::MenuItemCheckBox`) with this state. The menu keeps
    /// no state: choosing the item calls the menu's callback and the
    /// application writes the new value into its entries.
    pub fn checkbox(key: K, label: impl IntoText, checked: bool) -> Self {
        Self::of_item(key, label, ItemRole::Check(MenuCheck::Checkbox(checked)))
    }

    /// A radio item (`Role::MenuItemRadio`) with this checked state. Put the
    /// radio items of one choice in a [`group`](Self::group). Like a check
    /// item, choosing it calls the menu's callback with its key and the menu
    /// keeps no state.
    pub fn radio(key: K, label: impl IntoText, checked: bool) -> Self {
        Self::of_item(key, label, ItemRole::Check(MenuCheck::Radio(checked)))
    }

    /// An item that opens `children` instead of being chosen. Enter, Space,
    /// a click and Right open the submenu, and hovering it opens the submenu
    /// after 140 ms.
    pub fn submenu(key: K, label: impl IntoText, children: Vec<MenuEntry<K>>) -> Self {
        Self::of_item(key, label, ItemRole::Submenu(children))
    }

    /// A divider with no key. It cannot be focused or chosen.
    pub fn separator() -> Self {
        Self {
            kind: EntryKind::Separator,
            label: Reactive::new(String::new()),
            inset: false,
        }
    }

    /// A heading with no key, such as the title of the rows after it. It
    /// cannot be focused or chosen.
    pub fn heading(text: impl IntoText) -> Self {
        Self {
            kind: EntryKind::Heading,
            label: Reactive::new(text.into_text()),
            inset: false,
        }
    }

    /// A group of rows of the same level under the heading `label` (an empty
    /// label shows no heading). Groups do not nest a level: the keys move
    /// through their rows as through the rest of the level.
    pub fn group(label: impl IntoText, entries: Vec<MenuEntry<K>>) -> Self {
        Self {
            kind: EntryKind::Group(entries),
            label: Reactive::new(label.into_text()),
            inset: false,
        }
    }

    /// The kind of the row in a usage message.
    fn what(&self) -> &'static str {
        match &self.kind {
            EntryKind::Item(item) if matches!(item.role, ItemRole::Submenu(_)) => "submenu",
            EntryKind::Item(_) => "item",
            EntryKind::Separator => "separator",
            EntryKind::Heading => "heading",
            EntryKind::Group(_) => "group",
        }
    }

    /// The item data for a modifier that only items have; any other kind is a
    /// usage error at the caller.
    #[track_caller]
    fn item_for(&mut self, name: &'static str) -> &mut ItemEntry<K> {
        let what = self.what();
        match &mut self.kind {
            EntryKind::Item(item) => item,
            _ => raise(
                ErrorKind::Message,
                format!("MenuEntry::{name}: a {what} has no {name}"),
            ),
        }
    }

    /// Show `icon` before the label: a name the theme resolves (`"check"`)
    /// or an image [`Asset`](kivact_core::view::Asset). Items only.
    #[track_caller]
    pub fn with_icon(mut self, icon: impl Into<Icon>) -> Self {
        self.item_for("icon").icon = Some(icon.into());
        self
    }

    /// A secondary line of text under the label. Items only.
    #[track_caller]
    pub fn with_description(mut self, text: impl IntoText) -> Self {
        self.item_for("description").description = Some(Reactive::new(text.into_text()));
        self
    }

    /// Text shown for the key combination that triggers the same command, and
    /// published as the item's accessible description. It only displays: the
    /// menu binds no key. Items only.
    #[track_caller]
    pub fn with_shortcut(mut self, text: impl Into<String>) -> Self {
        self.item_for("shortcut").shortcut = text.into();
        self
    }

    /// A disabled item cannot be chosen, and a disabled submenu cannot open.
    /// By default the arrow keys and typeahead skip disabled items. Items only.
    #[track_caller]
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.item_for("disabled").disabled = disabled;
        self
    }

    /// Explain why the item is unavailable. Themes retaining disabled menu
    /// focus can show this reason as a tooltip. This does not disable the item.
    #[track_caller]
    pub fn with_disabled_reason(mut self, reason: impl IntoText) -> Self {
        self.item_for("disabled_reason").disabled_reason = Some(Reactive::new(reason.into_text()));
        self
    }

    /// Mark the item destructive (deleting, removing): its
    /// [`MenuItemState::destructive`] tells the drawing to warn. The
    /// behavior and semantics do not change. Items only.
    #[track_caller]
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.item_for("destructive").destructive = destructive;
        self
    }

    /// Indent the row (an item, a submenu item, a heading or the heading of
    /// a group) by the room of a check mark, so that it lines up with the
    /// check and radio items of its level. The behavior and semantics do
    /// not change. A separator has none.
    #[track_caller]
    pub fn inset(mut self, inset: bool) -> Self {
        if matches!(self.kind, EntryKind::Separator) {
            raise(
                ErrorKind::Message,
                "MenuEntry::inset: a separator has no inset",
            );
        }
        self.inset = inset;
        self
    }

    /// Leave the menu open when the item is chosen: the callback runs and
    /// the focus stays on the item, as for the check items of a column
    /// visibility menu. Items and check items only; a submenu has none.
    #[track_caller]
    pub fn keep_open(mut self, keep_open: bool) -> Self {
        let item = self.item_for("keep_open");
        if matches!(item.role, ItemRole::Submenu(_)) {
            raise(
                ErrorKind::Message,
                "MenuEntry::keep_open: a submenu has no keep_open",
            );
        }
        item.keep_open = keep_open;
        self
    }

    fn item_data(&self) -> Option<&ItemEntry<K>> {
        match &self.kind {
            EntryKind::Item(item) => Some(item),
            _ => None,
        }
    }

    /// The application's identity of the row; `None` for a separator, a
    /// heading and a group.
    pub fn key(&self) -> Option<&K> {
        self.item_data().map(|item| &item.key)
    }

    /// What the row says, its accessible name and the text typeahead
    /// matches; tracked when read inside a computation.
    #[track_caller]
    pub fn label(&self) -> String {
        self.label.get()
    }

    /// The shortcut text ([`shortcut`](Self::shortcut)); empty without one.
    pub fn shortcut(&self) -> &str {
        self.item_data().map_or("", |item| item.shortcut.as_str())
    }

    /// Whether the item is unavailable ([`disabled`](Self::disabled)).
    pub fn is_disabled(&self) -> bool {
        self.item_data().is_some_and(|item| item.disabled)
    }

    /// The explanation supplied by [`disabled_reason`](Self::disabled_reason).
    #[track_caller]
    pub fn disabled_reason(&self) -> Option<String> {
        self.item_data()?
            .disabled_reason
            .as_ref()
            .map(Reactive::get)
    }

    /// The checked state of a check or radio item; `None` for other rows.
    pub fn check(&self) -> Option<MenuCheck> {
        match self.item_data()?.role {
            ItemRole::Check(check) => Some(check),
            ItemRole::Action | ItemRole::Submenu(_) => None,
        }
    }

    /// What choosing the row does; `None` for a separator, a heading and a
    /// group.
    pub fn item_kind(&self) -> Option<MenuItemKind> {
        Some(match &self.item_data()?.role {
            ItemRole::Action => MenuItemKind::Action,
            ItemRole::Check(MenuCheck::Checkbox(_)) => MenuItemKind::Checkbox,
            ItemRole::Check(MenuCheck::Radio(_)) => MenuItemKind::Radio,
            ItemRole::Submenu(_) => MenuItemKind::Submenu,
        })
    }

    /// Whether the row is a [`heading`](Self::heading).
    pub fn is_heading(&self) -> bool {
        matches!(self.kind, EntryKind::Heading)
    }

    /// Whether this row is a [`group`](Self::group).
    pub fn is_group(&self) -> bool {
        matches!(self.kind, EntryKind::Group(_))
    }

    /// The rows of a submenu item or of a [`group`](Self::group); `None` for
    /// other rows.
    pub fn children(&self) -> Option<&[MenuEntry<K>]> {
        match &self.kind {
            EntryKind::Group(entries) => Some(entries),
            EntryKind::Item(ItemEntry {
                role: ItemRole::Submenu(children),
                ..
            }) => Some(children),
            _ => None,
        }
    }

    /// The rows of a submenu item.
    pub(super) fn submenu_entries(&self) -> Option<&[MenuEntry<K>]> {
        match &self.kind {
            EntryKind::Item(ItemEntry {
                role: ItemRole::Submenu(children),
                ..
            }) => Some(children),
            _ => None,
        }
    }

    /// The rows of a group.
    pub(super) fn group_entries(&self) -> Option<&[MenuEntry<K>]> {
        match &self.kind {
            EntryKind::Group(entries) => Some(entries),
            _ => None,
        }
    }

    /// The icon given to [`icon`](Self::icon).
    pub fn icon(&self) -> Option<&Icon> {
        self.item_data()?.icon.as_ref()
    }

    /// The description's current text; tracked when read inside a
    /// computation.
    #[track_caller]
    pub fn description(&self) -> Option<String> {
        self.item_data()?.description.as_ref().map(Reactive::get)
    }

    /// Whether the item is marked [`destructive`](Self::destructive).
    pub fn is_destructive(&self) -> bool {
        self.item_data().is_some_and(|item| item.destructive)
    }

    /// Whether choosing the item leaves the menu open
    /// ([`keep_open`](Self::keep_open)).
    pub fn keeps_open(&self) -> bool {
        self.item_data().is_some_and(|item| item.keep_open)
    }

    /// Whether the row keeps the room of a check mark
    /// ([`inset`](Self::inset)).
    pub fn is_inset(&self) -> bool {
        self.inset
    }

    /// A heading with the label and the inset of this group.
    pub(super) fn heading_of_group(&self) -> Self {
        Self {
            kind: EntryKind::Heading,
            label: self.label.clone(),
            inset: self.inset,
        }
    }

    /// The same entry with every key (this one and its children's) mapped by
    /// `f`. A separator has no key and passes through unchanged.
    ///
    /// A component that adds its own commands to an application's entries
    /// wraps the application's key in its own key type, and unwraps it when the
    /// menu reports the chosen key.
    ///
    /// ```
    /// # mod kivact {
    /// #     //! Role: Supply facade paths for this documentation example.
    /// #     pub extern crate kivact_interaction as interaction;
    /// # }
    /// use kivact::interaction::MenuEntry;
    /// let app = vec![
    ///     MenuEntry::item("new", "New"),
    ///     MenuEntry::submenu("export", "Export", vec![MenuEntry::item("png", "PNG")]),
    /// ];
    /// let wrapped: Vec<MenuEntry<(u8, &str)>> =
    ///     app.into_iter().map(|entry| entry.map_key(&|key| (1, key))).collect();
    /// assert_eq!(
    ///     wrapped[1],
    ///     MenuEntry::submenu(
    ///         (1, "export"),
    ///         "Export",
    ///         vec![MenuEntry::item((1, "png"), "PNG")],
    ///     ),
    /// );
    /// ```
    pub fn map_key<L>(self, f: &impl Fn(K) -> L) -> MenuEntry<L> {
        let kind = match self.kind {
            EntryKind::Item(item) => EntryKind::Item(ItemEntry {
                key: f(item.key),
                role: match item.role {
                    ItemRole::Action => ItemRole::Action,
                    ItemRole::Check(check) => ItemRole::Check(check),
                    ItemRole::Submenu(children) => ItemRole::Submenu(
                        children.into_iter().map(|child| child.map_key(f)).collect(),
                    ),
                },
                shortcut: item.shortcut,
                disabled: item.disabled,
                disabled_reason: item.disabled_reason,
                icon: item.icon,
                description: item.description,
                destructive: item.destructive,
                keep_open: item.keep_open,
            }),
            EntryKind::Separator => EntryKind::Separator,
            EntryKind::Heading => EntryKind::Heading,
            EntryKind::Group(entries) => {
                EntryKind::Group(entries.into_iter().map(|child| child.map_key(f)).collect())
            }
        };
        MenuEntry {
            kind,
            label: self.label,
            inset: self.inset,
        }
    }
}

/// The identity of one row of a menu level, which keys the caller's
/// `for_each` over [`MenuApi::entries`](super::MenuApi::entries).
///
/// An item's kind is part of its identity, so a row that turns into a
/// submenu or a check item is built again with the matching parts.
/// Separators and headings have no key and are counted instead. The rows of
/// a group follow its heading in the level.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MenuRow<K> {
    /// An item of this kind.
    Item { key: K, kind: MenuItemKind },
    /// The n-th separator of the level, counted from 0.
    Separator(usize),
    /// The n-th heading of the level, counted from 0: a
    /// [`heading`](MenuEntry::heading) or the heading of a group.
    Label(usize),
}

/// Which end of a level a focus request aims at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Edge {
    First,
    Last,
}

/// What a focus request of a menu level aims at.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Target<K> {
    Edge(Edge),
    Key(K),
}

/// A focus request for a level of the open menu, consumed by that level once
/// it is mounted.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Want<K> {
    /// A key, a submenu opening, a closing level or assistive technology
    /// asked for it: the pointer never replaces it.
    Command { depth: usize, target: Target<K> },
    /// The pointer rests on this item: any later request replaces it.
    Hover { depth: usize, key: K },
}

impl<K> Want<K> {
    /// The level the request is for.
    pub(super) fn depth(&self) -> usize {
        match self {
            Self::Command { depth, .. } | Self::Hover { depth, .. } => *depth,
        }
    }
}

/// How the open menu was opened.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Opened {
    Button,
    Context(MenuOrigin),
}

/// State shared by every level of one menu. All handles are `Copy`.
pub(super) struct Session<K: kivact_core::state::ItemKey> {
    pub(super) entries: Value<Vec<MenuEntry<K>>>,
    pub(super) choose: Var<Callback<K>>,
    /// The top level's popup; its anchor is the trigger.
    pub(super) popup: PopupApi,
    pub(super) anchor: NodeRef,
    /// How the open menu was opened; `None` while it is closed.
    pub(super) opened: Value<Option<Opened>>,
    /// Where the open menu was asked to appear, derived from `opened`.
    pub(super) origin: Value<Option<MenuOrigin>>,
    /// Keys of the submenu items that are open, outermost first.
    pub(super) path: Value<Vec<K>>,
    /// The item that has the focus: its depth and key.
    pub(super) focus_key: Value<Option<(usize, K)>>,
    /// How many levels the current entries keep (see `super::order::kept_levels`).
    pub(super) kept: Value<usize>,
    pub(super) want: Value<Option<Want<K>>>,
    pub(super) machine: MachineHandle<super::session::SessionMachine<K>>,
    pub(super) typed: kivact_core::state::Binding<Typeahead>,
    pub(super) press: Option<super::press_state::Press>,
    pub(super) surface_rects: Signal<Vec<(Vec<K>, kivact_core::geometry::Rect)>>,
}

impl<K: kivact_core::state::ItemKey> Clone for Session<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact_core::state::ItemKey> Copy for Session<K> {}
