//! Role: The sidebar trigger button that toggles the provider's sidebar.

use super::state::current;
use kivact::i18n::message;
use kivact::state::{Callback, IntoValue};
use kivact::ui::{ButtonVariant, ControlSize, button};
use kivact::view::{IntoView, View};
use kivact::view::{Node, NodeBuilder, icon};

/// The button that opens and closes the sidebar. Created by [`sidebar_trigger`].
pub struct SidebarTrigger {
    root: kivact::ui::Button,
    on_press: Option<Callback<()>>,
}

/// A ghost icon button named "Toggle Sidebar" (`SidebarTrigger`).
///
/// It toggles the sidebar of the provider it is inside, wherever it sits: in
/// the sidebar or in the page beside it. Ctrl+B (or Cmd+B) does the same.
/// [`SidebarTrigger::on_press`] adds a callback that runs after the toggle.
#[track_caller]
pub fn sidebar_trigger() -> SidebarTrigger {
    SidebarTrigger {
        root: button(message("basic.sidebar.toggle", "Toggle Sidebar"))
            .variant(ButtonVariant::Ghost)
            .hide_label(true)
            .icon(icon("panel-left"))
            .size(ControlSize::Small),
        on_press: None,
    }
}

impl SidebarTrigger {
    /// Call `f` after the sidebar toggled; it never replaces the toggle.
    pub fn on_press(mut self, f: impl Into<kivact::state::Callback<()>>) -> Self {
        self.on_press = Some(f.into());
        self
    }
}

impl NodeBuilder for SidebarTrigger {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.root.node_mut()
    }
}

impl IntoView for SidebarTrigger {
    fn into_view(self) -> View {
        let after = self.on_press;
        self.root
            .with_node(move |root| {
                root.part(kivact::custom::Part::new(().into_value(), move |root| {
                    let cx = current();
                    root.on_press(move || {
                        if let Some(cx) = cx {
                            cx.toggle();
                        }
                        if let Some(after) = after {
                            after.call(());
                        }
                    })
                }))
            })
            .into_view()
    }
}
