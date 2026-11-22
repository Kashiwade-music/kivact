//! Role: The sidebar trigger button that toggles the provider's sidebar.

use super::look::SidebarTheme;
use super::look::themed;
use super::state::{current, part_state};
use crate::ButtonVariant;
use crate::ControlSize;
use kivact_core::i18n::message;
use kivact_core::state::Callback;
use kivact_core::view::{Element, icon};
use kivact_macros::component;

/// The button that opens and closes the sidebar, named "Toggle Sidebar".
///
/// It toggles the sidebar of the provider it is inside, wherever it sits: in
/// the sidebar or in the page beside it. Ctrl+B (or Cmd+B) does the same.
/// `on_press` runs after the toggle; it never replaces the toggle.
#[component]
pub fn sidebar_trigger(on_press: Option<Callback<()>>) -> impl Element {
    let cx = current();
    let after = on_press;
    let root = crate::button(message("ui.sidebar.toggle", "Toggle Sidebar"))
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(icon("panel-left"))
        .size(ControlSize::Small)
        .with_node(move |root| {
            root.on_press(move || {
                if let Some(cx) = cx {
                    cx.toggle();
                }
                if let Some(after) = after {
                    after.call(());
                }
            })
        });
    themed(root, |r: &SidebarTheme| r.trigger_recipe(), part_state(cx))
}
