//! Role: Engine client host contracts.
use super::devtools::EngineEmulation;
use kivact_types::{Asset, Capabilities, Environment, PlatformRequest};
use std::fmt;

/// What the client is given at start: the window's environment as the
/// host read it (`Window::environment`). Later changes arrive as
/// `HostEvent::EnvChanged`, with the whole `Environment` again.
#[derive(Clone, Debug, PartialEq)]
pub struct HostInfo {
    /// What the window's backend can do (touch, pen, software keyboard,
    /// safe area, memory warnings, IME reconversion, native accessibility,
    /// GPU), reported once at start.
    pub capabilities: Capabilities,
    /// Host clock at start (ns, monotonic, host-defined epoch).
    pub now_ns: u64,
    /// The window's environment at start.
    pub env: Environment,
    pub surface_active: bool,
}

/// Result of one turn. A turn may commit several revisions (user effects
/// that write state produce a follow-up revision within the same turn).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TurnOutput {
    /// Host clock time at which the client wants the next turn even without
    /// events (timers). Frame-synchronised work uses
    /// the bridge's `Writer::request_frame`.
    pub next_wake_ns: Option<u64>,
    /// The App finished (close accepted and queues drained).
    pub exit: bool,
}

/// A host operation failure reported to its client.
#[derive(Clone, Debug, PartialEq)]
pub struct HostError {
    /// Human-readable cause of the failed host operation.
    pub message: String,
}

impl HostError {
    /// Construct a failure with a human-readable cause.
    pub fn new(message: impl Into<String>) -> Self {
        HostError {
            message: message.into(),
        }
    }
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for HostError {}

/// A service performed by the host, or delegated to its platform window.
#[derive(Clone, Debug, PartialEq)]
pub enum HostRequest {
    /// Capture CPU pixels after the current turn's layout and draw have settled.
    Snapshot(kivact_types::NodeId),
    Platform {
        window: kivact_types::WindowId,
        request: PlatformRequest,
    },
    /// Register a font before the next layout measures text.
    RegisterFont(Asset),
    /// Override reduced motion, or follow the system preference.
    ReducedMotion(Option<bool>),
    /// Deliver `inputs` to `window` after the turn; the answer is
    /// `HostEvent::InjectDelivered { window, request }` with the request
    /// this effect was given.
    Inject {
        window: kivact_types::WindowId,
        inputs: Vec<super::InjectedInput>,
    },
    /// Override the environment of every window.
    Emulate(EngineEmulation),
    /// Multiply the motion clock of every window.
    TimeScale(f64),
    /// Start `window`'s GPU device now: the App needs it (a composition
    /// with a user shader, or a GPU texture it shares with the window).
    /// Starting again changes nothing.
    DemandGpu {
        /// The window whose device is needed.
        window: kivact_types::WindowId,
    },
    /// What a drawable's export asks of the window: its frames of
    /// `generation`, as `mode`, at `scale` device px per logical px. `None`
    /// follows the window's scale.
    DrawableExport {
        id: kivact_types::DrawableId,
        generation: u64,
        mode: ExportRequestMode,
        scale: Option<f64>,
    },
    /// Where the app shows a drawable (drawable box → the viewport's local
    /// logical px), for the request of `generation`; `None` hides it.
    SetDrawableGeometry {
        id: kivact_types::DrawableId,
        generation: u64,
        geometry: Option<kivact_types::geometry3d::Matrix4>,
    },
}

/// The export a drawable asks of the window (`HostRequest::DrawableExport`).
#[derive(Clone, Debug, PartialEq)]
pub enum ExportRequestMode {
    /// Nothing is exported (or the window cannot export yet).
    None,
    /// Frames are read back as pixels.
    Pixels,
    /// Frames are shared as textures on this window's GPU adapter.
    Gpu(kivact_types::GpuAdapter),
}

/// The settled host's answer to a subtree CPU capture request.
#[derive(Clone, Debug, PartialEq)]
pub enum SnapshotResult {
    /// Straight-alpha RGBA pixels at logical scale 1.
    Pixels(kivact_types::Pixels),
    /// The target no longer has mounted layout.
    Unmounted,
    /// The scene contains commands without CPU rendering support.
    Unsupported,
    /// A rendering resource failed, with its classified diagnostic text.
    Failed(String),
}
