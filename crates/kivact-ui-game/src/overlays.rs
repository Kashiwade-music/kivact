//! Role: The icon menu in the console's look.
use kivact::control::{Node, NodeBuilder, icon};
use kivact::primitive::NodeExt;
use kivact::style::Icon;
use kivact::ui::MenuEntry;
use kivact::view::{HandlerResult, IntoText, IntoValue, IntoView, View, callback};
use kivact::{Callback, Value};
use std::hash::Hash;

/// Menu triggered by a square tool button, with an optional icon and callback.
pub struct IconMenu<K: 'static> {
    root: kivact::ui::Button,
    label: Value<String>,
    entries: Value<Vec<MenuEntry<K>>>,
    icon: Value<Icon>,
    choose: Callback<K>,
}
/// Required menu label and entries; the default icon is `menu`.
#[track_caller]
pub fn icon_menu<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    entries: impl IntoValue<Vec<MenuEntry<K>>>,
) -> IconMenu<K> {
    let label = label.into_text_value();
    IconMenu {
        root: crate::button::tool_button(label).hide_label(true),
        label,
        entries: entries.into_value(),
        icon: Icon::named("menu").into_value(),
        choose: callback(|_| ()),
    }
}
impl<K: 'static> IconMenu<K> {
    /// Reactive trigger icon; `menu` by default.
    pub fn icon(mut self, value: impl IntoValue<Icon>) -> Self {
        self.icon = value.into_value();
        self
    }
    /// Selection callback; no action by default.
    pub fn on_select<R: HandlerResult>(mut self, choose: impl Fn(K) -> R + 'static) -> Self {
        self.choose = callback(choose);
        self
    }
}
impl<K: 'static> NodeBuilder for IconMenu<K> {
    fn node_mut(&mut self) -> &mut Node {
        self.root.node_mut()
    }
}
impl<K: Clone + Eq + Hash + 'static> IntoView for IconMenu<K> {
    fn into_view(mut self) -> View {
        let disabled = self
            .root
            .accessible_disabled()
            .map(|read| (move || read()).into_value())
            .unwrap_or_else(|| false.into_value());
        let choose = self.choose;
        kivact::ui::menu(self.label, self.entries, move |key| {
            if disabled.get() {
                Ok(())
            } else {
                choose.try_call(key)
            }
        })
        .offset(0.)
        .attach(self.root.icon(icon(self.icon)).into_node())
        .into_view()
    }
}
