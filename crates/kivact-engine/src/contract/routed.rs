//! Role: Routing metadata wrapped around shared input samples.

use kivact_types::{NodeId, Point, RegionId};

/// One shared input sample and the engine's routing decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Routed<T> {
    /// The receiving node, or no node for a window-directed key event.
    pub target: Option<NodeId>,
    /// The receiving scene region, when the target is a region.
    pub region: Option<RegionId>,
    /// Position in the target's untransformed local coordinates.
    pub local: Point,
    /// Whether pointer capture selected the target.
    pub captured: bool,
    /// The result of consuming adoption, once supplied by the engine.
    pub activation: Option<Activation>,
    /// The unchanged normalized input payload.
    pub input: T,
}

/// Whether the pointer's click was suppressed by its adopted gesture.
/// The writer prepares this result immediately before activation dispatch,
/// after the release callbacks, and consumes the adoption exactly once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Activation(pub bool);
