//! Role: Values and callbacks exchanged by the engine and its client.
mod events;
pub use events::{DrawableTexturesHandle, HostEvent};
mod host;
pub use host::{ExportRequestMode, HostError, HostInfo, HostRequest, SnapshotResult, TurnOutput};
mod measure;
pub use measure::{LeafMeasure, LeafTextMeasurer};
mod node;
pub use node::{
    BoxData, DrawableMode, DrawableRole, EditingData, ImeSessionProps, NodeInput, NodeKind,
    NodeProperties, NodeType, RovingFocus, SceneLayer,
};

mod routed;
pub use routed::{Activation, Routed};

mod close;
pub use close::CloseSignal;
