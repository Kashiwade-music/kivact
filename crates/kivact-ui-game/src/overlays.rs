//! Role: The icon menu in the console's look.
use kivact::view::{Element, Node, icon};

use kivact::interaction::MenuEntry;
use kivact::state::{Callback, Value};
use kivact::state::{IntoText, IntoValue, callback};
use kivact::style::Icon;
use kivact::view::{HandlerResult, IntoView, View};

/// Menu triggered by a square tool button, with an optional icon and callback.
pub struct IconMenu<K: 'static> {
    root: kivact::ui::Button,
    label: Value<String>,
    entries: Value<Vec<MenuEntry<K>>>,
    icon: Value<Icon>,
    choose: Callback<K>,
}
/// The required menu label and items; the default icon is `menu`.
#[track_caller]
pub fn icon_menu<K: kivact::state::ItemKey>(
    label: impl IntoText,
    items: impl IntoValue<Vec<MenuEntry<K>>>,
) -> IconMenu<K> {
    let label = label.into_text_value();
    IconMenu {
        root: crate::button::tool_button(label).hide_label(true),
        label,
        entries: items.into_value(),
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
    /// Activation callback: the key of the chosen item; no action by default.
    pub fn on_activate<R: HandlerResult>(mut self, activate: impl Fn(K) -> R + 'static) -> Self {
        self.choose = callback(activate);
        self
    }
}
impl<K: kivact::state::ItemKey> Element for IconMenu<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.root.node_mut()
    }
}
impl<K: kivact::state::ItemKey> IntoView for IconMenu<K> {
    fn into_view(mut self) -> View {
        let disabled = self
            .root
            .node_mut()
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let choose = self.choose;
        kivact::ui::menu(self.label, self.entries)
            .on_activate(move |key| {
                if disabled.get() {
                    Ok(())
                } else {
                    choose.try_call(key)
                }
            })
            .offset(0.)
            .attach(self.root.icon(icon(self.icon)))
            .into_view()
    }
}
