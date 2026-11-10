//! Role: What mounting decides about a host node before it binds its effects.

use std::cell::RefCell;
use std::panic::Location;
use std::rc::Rc;

use kivact_engine::contract::NodeType;
use kivact_runtime::{Notifier, Owner};
use kivact_style::CompiledStyle;

/// What a node's first write decided, for the style and draw stages.
pub(super) struct NodeFacts {
    pub(super) kind: NodeType,
    // xtask-allow(state-fields): option: a builder input carried to mount; absent means not a focus stop
    /// The node is a focus stop: the theme's focus ring and disabled look
    /// apply under its style, styled or not (see `kivact_style::internal::compile_style`).
    pub(super) focus_stop: bool,
    pub(super) local_paint: LocalPaint,
}

/// How a node records its style's paint callback (`Style::paint`).
#[derive(Clone, Default)]
pub(crate) enum LocalPaint {
    /// Not a styled box (or an editor): a paint callback is reported as unsupported.
    #[default]
    None,
    /// A draw effect of the node records the callback from this source: one
    /// made at mount for a styled box with its own draw layers or canvas
    /// regions (`draw::bind`), or by the first callback (`OnFirstPaint`).
    WithDraw(LocalPaintSource),
    /// A styled box without draw layers that no callback reached yet: the
    /// first one creates the source, the size observer and the draw effect
    /// (`draw::bind_style_paint`), and the node is `WithDraw` from then on.
    OnFirstPaint,
}

/// Retained style paint and its render-safe metadata notification source.
#[derive(Clone)]
pub(crate) struct LocalPaintSource {
    pub(super) value: Rc<RefCell<Option<Box<CompiledStyle>>>>,
    pub(super) revision: Notifier,
}

impl LocalPaintSource {
    /// A source of `owner` with no paint yet.
    pub(super) fn new(owner: Owner, at: &'static Location<'static>) -> Self {
        Self {
            value: Rc::new(RefCell::new(None)),
            revision: owner.notifier(at),
        }
    }
}
