//! Role: Passive dock state, texts and menu callback values.

use super::geometry::DropTarget;
use super::model::{DockLayout, DockTabs};
use kivact::geometry::Rect;
use kivact::interaction::InlineEditApi;
use kivact::state::{Binding, Callback, Signal, Value, Var};
use kivact::view::EditPhase;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

/// A tab drag that has left its start threshold.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct TabDrag<K> {
    pub(super) source: K,
    pub(super) target: Option<DropTarget<K>>,
}

/// Registered tab rectangles with the token of their registering row.
pub(super) type TabRects<K> = RefCell<HashMap<K, (u64, Value<Option<Rect>>)>>;

pub(super) struct Shared<K: kivact::state::ItemKey> {
    pub(super) rects: TabRects<K>,
    pub(super) next_rect: Cell<u64>,
    pub(super) menu: MenuBuilder<K>,
    pub(super) on_close: Option<Callback<K>>,
    pub(super) texts: Texts,
}

/// The dock's own texts. Each follows the App's messages.
#[derive(Clone, Copy)]
pub(super) struct Texts {
    pub(super) rename: Value<String>,
    pub(super) close_panel: Value<String>,
    pub(super) close_others: Value<String>,
    pub(super) close_container: Value<String>,
    pub(super) maximize: Value<String>,
    pub(super) restore: Value<String>,
    pub(super) maximize_shortcut: Value<String>,
    pub(super) move_left: Value<String>,
    pub(super) move_right: Value<String>,
    /// Left, Right, Top, Bottom.
    pub(super) split: [Value<String>; 4],
    pub(super) move_to: Value<String>,
    pub(super) edge: Value<String>,
    pub(super) into_tabs: Value<String>,
    /// Left, Right, Top, Bottom.
    pub(super) side: [Value<String>; 4],
}

/// Copy handles shared by every row of one dock.
pub(super) struct DockCx<K: kivact::state::ItemKey> {
    pub(super) layout: Binding<DockLayout<K>>,
    pub(super) on_edit: Option<Callback<EditPhase>>,
    /// The normalized layout.
    pub(super) current: Value<DockLayout<K>>,
    pub(super) drag: Signal<Option<TabDrag<K>>>,
    pub(super) rename: InlineEditApi<K>,
    pub(super) root: Value<Option<Rect>>,
    pub(super) shared: Var<Rc<Shared<K>>>,
}

impl<K: kivact::state::ItemKey> Clone for DockCx<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: kivact::state::ItemKey> Copy for DockCx<K> {}

/// Builds the menu button of one container.
pub(super) type MenuBuilder<K> = kivact::view::ViewFn<MenuInput<K>>;

/// What a container's menu button reads.
pub(super) struct MenuInput<K: kivact::state::ItemKey> {
    pub(super) cx: DockCx<K>,
    pub(super) tabs: Value<DockTabs<K>>,
    pub(super) maximized: Value<bool>,
    pub(super) title: Value<String>,
}
