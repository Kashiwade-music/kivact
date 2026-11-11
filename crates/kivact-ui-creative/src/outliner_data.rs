//! Role: Consumer hierarchy values shared by outliner views and controls.

use crate::tokens::LabelTone;
use kivact::style::Icon;

/// Consumer-owned hierarchy node. Identities must be unique; parents may be absent.
///
/// Siblings are shown in the order of the items that name the same parent.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeItem {
    pub id: u64,
    // xtask-allow(state-fields): independent: the application's hierarchy, given in TreeItem::new and changed by the application after a TreeDrop; a root or a child may carry any visibility, selectability, render or placeholder flag
    pub parent: Option<u64>,
    pub label: String,
    /// The icon before the label: a name in the theme's icon set or an image.
    pub icon: Icon,
    // xtask-allow(state-fields): independent: the application's visibility, changed on a Restriction{flag: Visible} proposal (eye toggle or H); it only dims the label, so a hidden item can still be selectable, rendered or a placeholder
    pub visible: bool,
    // xtask-allow(state-fields): independent: the application's selectability, changed on a Restriction{flag: Selectable} proposal; it only disables the tree node, so it pairs with any visibility, render or placeholder state
    pub selectable: bool,
    // xtask-allow(state-fields): independent: the application's render toggle, changed on a Restriction{flag: Rendered} proposal; the outliner only shows it in its toggle column, so it pairs with any visibility, selectability or placeholder state
    pub rendered: bool,
    /// The label color, resolved against the current theme.
    pub tone: LabelTone,
    // xtask-allow(state-fields): independent: the application marks an unresolved item with TreeItem::placeholder; it only draws the label italic and muted, so the item keeps its parent and all restriction toggles
    /// An unresolved item, drawn italic and muted while remaining operable.
    pub placeholder: bool,
}
impl TreeItem {
    pub fn new(
        id: u64,
        parent: Option<u64>,
        label: impl Into<String>,
        icon: impl Into<Icon>,
    ) -> Self {
        Self {
            id,
            parent,
            label: label.into(),
            icon: icon.into(),
            visible: true,
            selectable: true,
            rendered: true,
            tone: LabelTone::Normal,
            placeholder: false,
        }
    }
    /// Set the label tone. An explicit non-normal tone overrides placeholder muting.
    pub fn tone(mut self, tone: LabelTone) -> Self {
        self.tone = tone;
        self
    }
    /// Draw an unresolved item in italic without changing selection or controls.
    /// A constant: a `TreeItem` is plain data inside the items value, so a
    /// changed flag is a new list.
    pub fn placeholder(mut self, placeholder: bool) -> Self {
        self.placeholder = placeholder;
        self
    }
}

/// Where a dragged item was dropped, as the place it takes in the hierarchy.
///
/// The pointer over a row decides it: the upper third of a row is before it
/// as a sibling, the lower third after it as a sibling, and the middle third
/// inside it, as its last child. An item is never dropped on itself or into
/// its own subtree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreeDrop {
    /// The item that was dragged.
    pub id: u64,
    // xtask-allow(state-fields): independent: the new parent drop_plan computes from the row under the pointer; None makes a root, and both a root and a child drop may land first (after None) or after a sibling
    /// Its new parent; `None` makes it a root.
    pub target: Option<u64>,
    // xtask-allow(state-fields): independent: the sibling under target that drop_plan puts the item after; None puts it first, which a drop into a root list and into a parent both allow
    /// The sibling it lands right after, among the children of `target`
    /// without the dragged item; `None` puts it first.
    pub after: Option<u64>,
}

/// The application-owned restriction a user proposes changing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestrictionFlag {
    Visible,
    Selectable,
    Rendered,
}

/// An absolute proposal for all affected items in the current gesture.
#[derive(Clone, Debug, PartialEq)]
pub struct Restriction {
    pub ids: Vec<u64>,
    pub flag: RestrictionFlag,
    pub value: bool,
}
