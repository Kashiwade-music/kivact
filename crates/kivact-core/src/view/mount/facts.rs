//! Role: What mounting decides about a host node before it binds its effects.

use std::cell::RefCell;
use std::rc::Rc;

use kivact_engine::contract::NodeType;
use kivact_runtime::Notifier;
use kivact_style::CompiledStyle;

/// What a node's first write decided, for the style and draw stages.
pub(super) struct NodeFacts {
    pub(super) kind: NodeType,
    // xtask-allow(state-fields): passive mount record - The node is a focus stop: the theme's focus ring and disabled look apply under its style, styled or not (see `kivact_style::internal::compile_style`)
    /// The node is a focus stop: the theme's focus ring and disabled look
    /// apply under its style, styled or not (see `kivact_style::internal::compile_style`).
    pub(super) focus_stop: bool,
    // xtask-allow(state-fields): passive mount record - local paint
    pub(super) local_paint: Option<LocalPaintSource>,
}

/// Retained style paint and its render-safe metadata notification source.
#[derive(Clone)]
pub(crate) struct LocalPaintSource {
    pub(super) value: Rc<RefCell<Option<CompiledStyle>>>,
    pub(super) revision: Notifier,
}
