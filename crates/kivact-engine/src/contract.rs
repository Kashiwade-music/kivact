//! Role: Values and callbacks exchanged by the engine and its client.
mod devtools;
pub(crate) use devtools::DevtoolsChange;
pub use devtools::{
    BoxModelRects, DevtoolsKeys, DevtoolsOverlay, EngineEmulation, EngineRecords, FrameRecord,
    ImeStateInfo, KeyChord, OverlayBox, OverlayFill, OverlayLabel, OverlayOutline, PickPhase,
    SyncRecord,
};
// The core's DevTools agent builds injected input and cannot depend on the
// shell itself.
// xtask-allow(internal-contract): the core's DevTools agent builds injected input and cannot depend on the shell
pub use arvane::InjectedInput;
pub use arvane_shell::pipeline::{PlatformKey, RawDrop, RawIme, RawInput, RawKey, RawKeyPhase};
mod events;
pub use events::{DrawableTexturesHandle, ForeignChange, HostEvent};
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
pub use routed::{Activation, RouteTarget, Routed};

mod close;
pub use close::CloseSignal;
