//! Role: The sidebar trigger button that toggles the provider's sidebar.

use super::state::{current, deferred};
use kivact::Callback;
use kivact::control::{Node, NodeBuilder, icon};
use kivact::i18n::message;
use kivact::ui::{ButtonVariant, ControlSize, button};
use kivact::view::HandlerResult;
use kivact::view::{IntoView, View, callback};

/// The button that opens and closes the sidebar. Created by [`sidebar_trigger`].
pub struct SidebarTrigger {
    root: Node,
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
            .size(ControlSize::Small)
            .into_node(),
        on_press: None,
    }
}

impl SidebarTrigger {
    /// Call `f` after the sidebar toggled; it never replaces the toggle.
    pub fn on_press<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_press = Some(callback(move |_| f()));
        self
    }
}

impl NodeBuilder for SidebarTrigger {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SidebarTrigger {
    fn into_view(self) -> View {
        deferred(move || {
            let cx = current();
            let after = self.on_press;
            self.root.on_press(move || {
                if let Some(cx) = cx {
                    cx.toggle();
                }
                if let Some(after) = after {
                    after.call(());
                }
            })
        })
    }
}
