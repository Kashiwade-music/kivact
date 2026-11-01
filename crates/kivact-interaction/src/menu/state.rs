//! Role: Store menu entries and shared session state.

use crate::Popup;
use crate::list::typeahead::Typeahead;
use kivact_core::draw::Point;
use kivact_core::primitive::PartState;
use kivact_core::style::Icon;
use kivact_core::view::{IntoText, NodeRef, Reactive};
use kivact_core::{Callback, Signal, Value, Var};

/// Where an open menu was asked to appear.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuOrigin {
    /// At this window point: the pointer of a right click.
    Pointer(Point),
    /// At the trigger: the request came from the keyboard or from assistive
    /// technology, which have no pointer, or from a menu button.
    Anchor,
}

/// The state of a menu item for the caller's drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuItemState {
    pub input: PartState,
    /// The checked state of a check or radio item; `None` for other items.
    pub checked: Option<bool>,
    /// The item is a radio item ([`MenuEntry::radio`]).
    pub radio: bool,
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
///     MenuEntry::item("new", "New").shortcut("Ctrl N").icon("file-plus"),
///     MenuEntry::item("grid", "Show grid").checked(true),
///     MenuEntry::separator(),
///     MenuEntry::group("Align", vec![
///         MenuEntry::radio("left", "Left", true),
///         MenuEntry::radio("right", "Right", false),
///     ]),
///     MenuEntry::heading("Share"),
///     MenuEntry::submenu("export", "Export", vec![
///         MenuEntry::item("png", "PNG").description("Raster image"),
///         MenuEntry::item("svg", "SVG").disabled(true),
///     ]),
/// ];
/// assert_eq!(entries.len(), 6);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MenuEntry<K> {
    /// The application's identity for the row; `None` for a separator.
    pub(crate) key: Option<K>,
    /// What the row says; its accessible name and the text typeahead matches.
    pub(crate) label: Reactive<String>,
    /// The shortcut text, published as the item's accessible description.
    pub(crate) shortcut: String,
    /// A disabled item is skipped by the keys and cannot be chosen.
    pub(crate) disabled: bool,
    /// The checked state of a check or radio item; `None` for other rows.
    pub(crate) checked: Option<bool>,
    /// A radio item ([`radio`](Self::radio)): `Role::MenuItemRadio`.
    pub(crate) radio: bool,
    /// A heading ([`heading`](Self::heading)): the keys skip it.
    pub(crate) heading: bool,
    /// The entries of a submenu item or of a [`group`](Self::group);
    /// `None` for other rows.
    pub(crate) children: Option<Vec<MenuEntry<K>>>,
    /// An icon shown before the label: a name the theme resolves or an image.
    pub(crate) icon: Option<Icon>,
    /// A secondary line under the label.
    pub(crate) description: Option<Reactive<String>>,
    /// A destructive item (delete, remove): only the drawing reads it.
    pub(crate) destructive: bool,
    /// Choosing the item leaves the menu open (a check item toggled in a
    /// row of several).
    pub(crate) keep_open: bool,
    /// The row keeps the room of a check mark before its content, so that
    /// it lines up with the check and radio items of its level (Radix's
    /// `inset`). Only the drawing reads it.
    pub(crate) inset: bool,
}

impl<K> MenuEntry<K> {
    fn new(key: Option<K>, label: Reactive<String>, children: Option<Vec<MenuEntry<K>>>) -> Self {
        Self {
            key,
            label,
            shortcut: String::new(),
            disabled: false,
            checked: None,
            radio: false,
            heading: false,
            children,
            icon: None,
            description: None,
            destructive: false,
            keep_open: false,
            inset: false,
        }
    }

    /// An item the user can choose. The label is its accessible name and the
    /// text typeahead matches.
    pub fn item(key: K, label: impl IntoText) -> Self {
        Self::new(Some(key), Reactive::new(label.into_text()), None)
    }

    /// An item that opens `children` instead of being chosen. Enter, Space,
    /// a click and Right open the submenu, and hovering it opens the submenu
    /// after 140 ms.
    pub fn submenu(key: K, label: impl IntoText, children: Vec<MenuEntry<K>>) -> Self {
        Self::new(Some(key), Reactive::new(label.into_text()), Some(children))
    }

    /// A divider with no key. It cannot be focused or chosen.
    pub fn separator() -> Self {
        Self::new(None, Reactive::new(String::new()), None)
    }

    /// A radio item (`Role::MenuItemRadio`) with this checked state. Put the
    /// radio items of one choice in a [`group`](Self::group). Like a check
    /// item, choosing it calls the menu's callback with its key and the menu
    /// keeps no state.
    pub fn radio(key: K, label: impl IntoText, checked: bool) -> Self {
        let mut entry = Self::new(Some(key), Reactive::new(label.into_text()), None);
        entry.checked = Some(checked);
        entry.radio = true;
        entry
    }

    /// A heading with no key, such as the title of the rows after it. It
    /// cannot be focused or chosen.
    pub fn heading(text: impl IntoText) -> Self {
        let mut entry = Self::new(None, Reactive::new(text.into_text()), None);
        entry.heading = true;
        entry
    }

    /// A group of rows of the same level under the heading `label` (an empty
    /// label shows no heading). Groups do not nest a level: the keys move
    /// through their rows as through the rest of the level.
    pub fn group(label: impl IntoText, entries: Vec<MenuEntry<K>>) -> Self {
        Self::new(None, Reactive::new(label.into_text()), Some(entries))
    }

    /// Show `icon` before the label: a name the theme resolves (`"check"`)
    /// or an image [`Asset`](kivact_core::Asset).
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// A secondary line of text under the label.
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(Reactive::new(text.into_text()));
        self
    }

    /// Whether this row is a [`group`](Self::group).
    pub fn is_group(&self) -> bool {
        self.key.is_none() && self.children.is_some()
    }

    /// Text shown for the key combination that triggers the same command, and
    /// published as the item's accessible description. It only displays: the
    /// menu binds no key.
    pub fn shortcut(mut self, text: impl Into<String>) -> Self {
        self.shortcut = text.into();
        self
    }

    /// A disabled item is skipped by the arrow keys and typeahead and cannot
    /// be chosen. A disabled submenu does not open.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Make the item a check item (`Role::MenuItemCheckBox`) with this state.
    /// The menu keeps no state: choosing the item calls the menu's callback
    /// and the application writes the new value into its entries.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Mark the item destructive (deleting, removing): its
    /// [`MenuItemState::destructive`] tells the drawing to warn. The
    /// behavior and semantics do not change.
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    /// Indent the row (an item, a submenu item, a heading or the heading of
    /// a group) by the room of a check mark, so that it lines up with the
    /// check and radio items of its level. The behavior and semantics do
    /// not change.
    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    /// Leave the menu open when the item is chosen: the callback runs and
    /// the focus stays on the item, as for the check items of a column
    /// visibility menu. A submenu item ignores it.
    pub fn keep_open(mut self, keep_open: bool) -> Self {
        self.keep_open = keep_open;
        self
    }

    /// The application's identity of the row; `None` for a separator, a
    /// heading and a group.
    pub fn key(&self) -> Option<&K> {
        self.key.as_ref()
    }

    /// What the row says, its accessible name and the text typeahead
    /// matches; tracked when read inside a computation.
    #[track_caller]
    pub fn label(&self) -> String {
        self.label.get()
    }

    /// The shortcut text ([`shortcut`](Self::shortcut)); empty without one.
    pub fn shortcut_value(&self) -> &str {
        &self.shortcut
    }

    /// Whether the item is unavailable ([`disabled`](Self::disabled)).
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// The checked state of a check or radio item; `None` for other rows.
    pub fn checked_value(&self) -> Option<bool> {
        self.checked
    }

    /// Whether the row is a [`radio`](Self::radio) item.
    pub fn is_radio(&self) -> bool {
        self.radio
    }

    /// Whether the row is a [`heading`](Self::heading).
    pub fn is_heading(&self) -> bool {
        self.heading
    }

    /// The rows of a submenu item or of a [`group`](Self::group); `None` for
    /// other rows.
    pub fn children(&self) -> Option<&[MenuEntry<K>]> {
        self.children.as_deref()
    }

    /// The icon given to [`icon`](Self::icon).
    pub fn icon_value(&self) -> Option<&Icon> {
        self.icon.as_ref()
    }

    /// The description's current text; tracked when read inside a
    /// computation.
    #[track_caller]
    pub fn description_value(&self) -> Option<String> {
        self.description.as_ref().map(Reactive::get)
    }

    /// Whether the item is marked [`destructive`](Self::destructive).
    pub fn is_destructive(&self) -> bool {
        self.destructive
    }

    /// Whether choosing the item leaves the menu open
    /// ([`keep_open`](Self::keep_open)).
    pub fn keeps_open(&self) -> bool {
        self.keep_open
    }

    /// Whether the row keeps the room of a check mark
    /// ([`inset`](Self::inset)).
    pub fn is_inset(&self) -> bool {
        self.inset
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
        MenuEntry {
            key: self.key.map(f),
            label: self.label,
            shortcut: self.shortcut,
            disabled: self.disabled,
            checked: self.checked,
            radio: self.radio,
            heading: self.heading,
            icon: self.icon,
            description: self.description,
            destructive: self.destructive,
            keep_open: self.keep_open,
            inset: self.inset,
            children: self
                .children
                .map(|children| children.into_iter().map(|child| child.map_key(f)).collect()),
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
    /// An item, a submenu item (`submenu`), a check item (`check`) or a
    /// radio item (`check` and `radio`).
    Item {
        key: K,
        submenu: bool,
        check: bool,
        radio: bool,
    },
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

/// A focus request for a level of the open menu, consumed by the level at that
/// depth once it is mounted.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Want<K> {
    Edge(usize, Edge),
    Key(usize, K),
}

/// State shared by every level of one menu. All handles are `Copy`.
pub(super) struct Session<K: 'static> {
    pub(super) entries: Value<Vec<MenuEntry<K>>>,
    pub(super) choose: Callback<K>,
    /// The top level's popup; its anchor is the trigger.
    pub(super) popup: Popup,
    pub(super) anchor: NodeRef,
    /// Where the open menu was asked to appear.
    pub(super) origin: Signal<Option<MenuOrigin>>,
    /// Keys of the submenu items that are open, outermost first.
    pub(super) path: Signal<Vec<K>>,
    /// The item that has the focus: its depth and key.
    pub(super) focus_key: Signal<Option<(usize, K)>>,
    /// How many levels the current entries keep (see `super::order::kept_levels`).
    pub(super) kept: Value<usize>,
    pub(super) want: Signal<Option<Want<K>>>,
    pub(super) typed: Var<Typeahead>,
}

impl<K: 'static> Clone for Session<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: 'static> Copy for Session<K> {}
