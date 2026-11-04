//! Role: Values and callbacks exchanged by the engine and its client.
mod events;
pub use events::HostEvent;
mod host;
pub use host::{HostError, HostInfo, HostRequest, TurnOutput};
mod measure;
pub use measure::{LeafMeasure, LeafTextMeasurer};
mod node;
pub use node::{
    BoxData, EditingData, ImeSessionProps, NodeInput, NodeKind, NodeProperties, NodeType,
    RovingFocus, SceneLayer,
};

mod routed;
pub use routed::{Activation, Routed};

mod close;
pub use close::CloseSignal;
