//! Role: Passive dock state, positioned items and menu callback values.

use super::geometry::{Divider, DropTarget, Frame};
use super::model::{DockLayout, DockTabs};
use kivact::draw::Rect;
use kivact::ui::EditPhase;
use kivact::view::View;
use kivact::{Binding, Callback, Signal, Value, Var};
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

pub(super) struct Shared<K: 'static> {
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
pub(super) struct DockCx<K: 'static> {
    pub(super) layout: Binding<DockLayout<K>>,
    pub(super) on_edit: Option<Callback<EditPhase>>,
    /// The normalized layout.
    pub(super) current: Value<DockLayout<K>>,
    pub(super) drag: Signal<Option<TabDrag<K>>>,
    pub(super) renaming: Signal<Option<K>>,
    pub(super) draft: Signal<String>,
    pub(super) root: Value<Option<Rect>>,
    pub(super) shared: Var<Rc<Shared<K>>>,
}

impl<K: 'static> Clone for DockCx<K> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<K: 'static> Copy for DockCx<K> {}

/// One positioned sibling of the dock root.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Item<K> {
    Chrome {
        tabs: DockTabs<K>,
        frame: Frame,
        shown: bool,
        maximized: bool,
    },
    Content {
        title: String,
        frame: Frame,
        shown: bool,
    },
    Divider {
        divider: Divider,
        leading: String,
        trailing: String,
        shown: bool,
    },
}

/// Builds the menu button of one container.
pub(super) type MenuBuilder<K> = Rc<dyn Fn(MenuInput<K>) -> View>;

/// What a container's menu button reads.
pub(super) struct MenuInput<K: 'static> {
    pub(super) cx: DockCx<K>,
    pub(super) tabs: Value<DockTabs<K>>,
    pub(super) maximized: Value<bool>,
    pub(super) title: Value<String>,
}
