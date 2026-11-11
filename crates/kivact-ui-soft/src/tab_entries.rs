//! Role: Tab and tab-menu entries of the tab strip.

use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::NodeRef;
use kivact::view::View;
use kivact::view::ViewFn;
use kivact::view::node;
use kivact::view::text;

/// A keyed tab with retained icon and trailing-content slots.
pub struct Tab<K> {
    pub(crate) root: Node,
    pub(crate) key: K,
    pub(crate) label: Value<String>,
    // xtask-allow(state-fields): option: mounted once when the tab is built; absent means no icon
    pub(crate) icon: Option<View>,
    // xtask-allow(state-fields): option: mounted once for the selected tab; absent means the resting icon
    pub(crate) selected_icon: Option<View>,
    pub(crate) label_hidden: Value<bool>,
    // xtask-allow(state-fields): option: mounted once when the tab is built; absent means no trailing content
    pub(crate) end: Option<View>,
    // xtask-allow(state-fields): option: caller-provided panel reference; absent means no associated panel
    pub(crate) panel: Option<NodeRef>,
}

/// Create a tab identified by `key` and named by `label`.
pub fn tab<K>(key: K, label: impl IntoText) -> Tab<K> {
    Tab {
        root: node(),
        key,
        label: label.into_text_value(),
        icon: None,
        selected_icon: None,
        label_hidden: (false).into_value(),
        end: None,
        panel: None,
    }
}

impl<K> Tab<K> {
    /// Mount the tab's leading icon once.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    /// Mount an alternative icon once for the selected tab.
    pub fn selected_icon(mut self, icon: impl IntoView) -> Self {
        self.selected_icon = Some(icon.into_view());
        self
    }
    /// Hide the visible label while retaining its accessible name.
    pub fn label_hidden(mut self, hidden: impl IntoValue<bool>) -> Self {
        self.label_hidden = hidden.into_value();
        self
    }
    /// Mount trailing tab content once.
    pub fn end_content(mut self, end: impl IntoView) -> Self {
        self.end = Some(kivact::view::scope(move || end.into_view()));
        self
    }
    /// Associate the tab with an existing panel node reference.
    pub fn panel(mut self, panel: NodeRef) -> Self {
        self.panel = Some(panel);
        self
    }
}

macro_rules! keyed_element {
    ($type:ident) => {
        impl<K: 'static> Element for $type<K> {
            #[doc(hidden)]
            fn node_mut(&mut self) -> &mut Node {
                &mut self.root
            }
        }
    };
}
keyed_element!(Tab);

impl<K: 'static> IntoView for Tab<K> {
    /// A tab outside a `tab_list` shows its icon and label only; it has no
    /// tab semantics or selection.
    fn into_view(self) -> View {
        self.root.child((self.icon, text(self.label))).into_view()
    }
}

#[derive(Clone)]
/// A keyed option inside a tab-strip menu.
pub struct TabMenuOption<K> {
    pub(crate) key: K,
    pub(crate) label: Value<String>,
    pub(crate) icon: Option<ViewFn>,
    // xtask-allow(state-fields): option: caller availability from TabMenuOption::disabled; false keeps the option selectable in the strip menu.
    pub(crate) disabled: bool,
}

impl<K> TabMenuOption<K> {
    /// Create a selectable menu option with its destination key and label.
    pub fn new(key: K, label: impl IntoText) -> Self {
        Self {
            key,
            label: label.into_text_value(),
            icon: None,
            disabled: false,
        }
    }
    /// Build this option's icon when its menu content opens.
    pub fn icon<V: IntoView>(mut self, icon: impl IntoViewFn<V>) -> Self {
        self.icon = Some(icon.into_view_fn());
        self
    }
    /// Prevent this option from being chosen.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A menu trigger embedded in the tab strip.
pub struct TabMenu<K> {
    pub(crate) root: Node,
    pub(crate) label: Value<String>,
    pub(crate) options: Vec<TabMenuOption<K>>,
}

/// Create a labeled tab-strip menu from keyed items in visible order.
pub fn tab_menu<K>(label: impl IntoText, items: Vec<TabMenuOption<K>>) -> TabMenu<K> {
    TabMenu {
        root: node(),
        label: label.into_text_value(),
        options: items,
    }
}

keyed_element!(TabMenu);

impl<K: 'static> IntoView for TabMenu<K> {
    /// A tab menu outside a `tab_list` shows its label only.
    fn into_view(self) -> View {
        self.root.child(text(self.label)).into_view()
    }
}
