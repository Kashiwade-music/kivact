//! Role: The icon menu in the console's look.
use crate::button::tool_button_of;
use kivact::component;
use kivact::custom::Caller;
use kivact::interaction::MenuEntry;
use kivact::state::{Callback, ItemKey, Value, callback};
use kivact::style::Icon;
use kivact::ui::menu;
use kivact::view::node;

/// A menu opened from a square tool button showing `icon`, with `label` as
/// its accessible name. `on_activate` receives the key of the chosen item; a
/// disabled menu passes no choice.
#[component]
pub fn icon_menu<K: ItemKey>(
    label: Value<String>,
    items: Value<Vec<MenuEntry<K>>>,
    #[prop(default = Icon::named("menu"))] icon: Value<Icon>,
    on_activate: Option<Callback<K>>,
    #[caller] caller: Caller,
) -> impl Element {
    let disabled = caller.disabled();
    let choose = on_activate.unwrap_or_else(|| callback(|_: K| ()));
    let trigger = tool_button_of(label)
        .hide_label(true)
        .icon(kivact::view::icon(icon));
    node().child(
        menu(label, items)
            .on_activate(move |key| {
                if disabled.is_some_and(|disabled| disabled.get()) {
                    Ok(())
                } else {
                    choose.try_call(key)
                }
            })
            .offset(0.)
            .attach(trigger),
    )
}
