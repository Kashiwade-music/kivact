//! Role: What mounting decides about a host node before it binds its effects.

use std::cell::RefCell;
use std::rc::Rc;

use kivact_engine::contract::NodeType;
use kivact_runtime::Trigger;
use kivact_style::CompiledStyle;

/// What a node's first write decided, for the style and draw stages.
pub(super) struct NodeFacts {
    pub(super) kind: NodeType,
    /// The node is a focus stop: the theme's focus ring and disabled look
    /// apply under its style, styled or not (see `Style::compile`).
    pub(super) focus_stop: bool,
    pub(super) local_paint: Option<LocalPaintSource>,
}

/// Retained style paint and its render-safe metadata notification source.
#[derive(Clone)]
pub(crate) struct LocalPaintSource {
    pub(super) value: Rc<RefCell<Option<CompiledStyle>>>,
    pub(super) revision: Trigger,
}
