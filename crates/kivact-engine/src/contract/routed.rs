//! Role: Routing metadata wrapped around shared input samples.

use kivact_types::{NodeId, Point, RegionId, WindowId};

/// One shared input sample and the engine's routing decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Routed<T> {
    /// The window the input belongs to.
    pub window: WindowId,
    // xtask-allow(state-fields): option: absent means the input is window-directed (a key with no receiving node)
    /// The receiving node and region, or no target for a window-directed key
    /// event.
    pub target: Option<RouteTarget>,
    /// Position in the target's untransformed local coordinates.
    pub local: Point,
    // xtask-allow(state-fields): option: absent until the engine supplies the adoption result before activation dispatch
    /// The result of consuming adoption, once supplied by the engine.
    pub activation: Option<Activation>,
    /// The unchanged normalized input payload.
    pub input: T,
}

/// The node that receives an input, and the scene region when it is a region.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouteTarget {
    /// The receiving node.
    pub node: NodeId,
    /// The receiving scene region, when the target is a region.
    pub region: Option<RegionId>,
}

/// Whether the pointer's click was suppressed by its adopted gesture.
/// The writer prepares this result immediately before activation dispatch,
/// after the release callbacks, and consumes the adoption exactly once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Activation(pub bool);
