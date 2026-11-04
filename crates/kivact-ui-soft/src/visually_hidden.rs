//! Role: Expose noninteractive text to assistive technology without painting it.

use kivact::view::Node;
use kivact::view::text;

use kivact::state::IntoText;
use kivact::style::Color;
use kivact::style::Position;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::LiveRegion;
use kivact::view::NodeRef;
use kivact::view::View;

/// Text available to assistive technology without a visible layout box.
pub struct VisuallyHidden {
    root: Node,
}

/// Create visually hidden text.
pub fn visually_hidden(value: impl IntoText) -> VisuallyHidden {
    VisuallyHidden {
        root: text(value).hit_testable(false).style(
            style()
                .position(Position::Absolute)
                .size(1, 1)
                .clip(true)
                .text_wrap(false)
                .color(Color::TRANSPARENT),
        ),
    }
}

impl VisuallyHidden {
    /// Announce text changes at the given priority.
    pub fn live(mut self, priority: LiveRegion) -> Self {
        self.root = self.root.live_region(priority);
        self
    }

    /// Attach a reference for another node's accessible description or label.
    pub fn node_ref(mut self, reference: NodeRef) -> Self {
        self.root = self.root.node_ref(reference);
        self
    }
}

impl IntoView for VisuallyHidden {
    fn into_view(self) -> View {
        self.root.into_view()
    }
}
