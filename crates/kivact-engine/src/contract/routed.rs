//! Role: Routing metadata wrapped around shared input samples.

use kivact_types::{NodeId, Point, RegionId, WindowId};

/// One shared input sample and the engine's routing decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Routed<T> {
    /// The window the input belongs to.
    pub window: WindowId,
    /// The receiving node and region, or no target for a window-directed key
    /// event.
    pub target: Option<RouteTarget>,
    /// Position in the target's untransformed local coordinates.
    pub local: Point,
    /// The result of consuming adoption, once supplied by the engine.
    pub activation: Option<Activation>,
    /// The unchanged normalized input payload.
    pub input: T,
    /// Whether the input came from the platform or was injected.
    pub origin: InputOrigin,
}

/// Where an input event came from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputOrigin {
    /// The platform's input (or the test driver's raw input).
    #[default]
    Platform,
    /// Injected by the request (the DevTools' `inject_input`, `HostRequest::Inject`).
    Injected(kivact_types::RequestId),
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
