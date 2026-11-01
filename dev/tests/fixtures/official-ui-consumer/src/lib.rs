//! Role: An independent consumer implements a public third-party component.
//! An independent consumer and a third-party component. This package lives
//! outside Kivact's workspace and imports the same public facade as the
//! official libraries, with no engine, protocol or host dependencies.

use kivact::control::{Node, node, text};
use kivact::prelude::*;
use kivact::view::View;

pub mod interactions;

/// A third-party action supporting the exact same modifiers as an official
/// builder by implementing only public `NodeBuilder` and `IntoView`.
pub struct ExternalAction {
    root: Node,
}

/// Make a component without borrowing any official library's implementation.
pub fn external_action(label: &str) -> ExternalAction {
    ExternalAction {
        root: node()
            .role(Role::Button)
            .focusable(true)
            .name(label)
            .style(style().size(160, 36).background(rgb(0x2b8bd8)))
            .child(text(label)),
    }
}

impl NodeBuilder for ExternalAction {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for ExternalAction {
    fn into_view(self) -> View {
        self.root.into_view()
    }
}
