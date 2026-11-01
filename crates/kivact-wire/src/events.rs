//! Role: Shared input values and events from the engine to the client.
//! Events arrive in production order through one queue, handed to each
//! turn. Kivact publicly re-exports the input keys, modifier state, mouse
//! buttons, pointer phases and kinds, and file drag phases.

use std::ops::Range;
use std::path::PathBuf;

use super::check::LayoutRule;
use super::ids::{NodeId, RegionId, RequestId, Revision};
use super::motion::MotionClock;
use super::semantics::A11yAction;
use super::{Point, Rect, Size};

pub use kivact_types::{FilePhase, Key, KeyCode, Modifiers, MouseButton, PointerId, PointerKind};

pub use kivact_types::{PointerButtons, PointerPhase};

/// Pointer event with node-local position. The host owns hit testing and
/// capture: while a pointer holds a button (a touch or pen is in contact),
/// its events go to the node that got its `Down`. The mouse is pointer 1;
/// each touch or pen contact has its own id while it lasts and presses as
/// the primary button. `is_primary` is Pointer Events' `isPrimary`: the
/// mouse, and the first touch or pen contact while no other of its kind is
/// in contact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointerEvent {
    /// The mouse's is 1; each touch or pen contact has its own.
    pub id: PointerId,
    /// Transition represented by this pointer event.
    pub phase: PointerPhase,
    /// Where the pointer is, in the node's untransformed local coordinates.
    pub pos: Point,
    /// The same point in the window (logical pixels).
    pub window_pos: Point,
    /// Button responsible for the transition, when applicable.
    pub button: Option<MouseButton>,
    /// Buttons held after this event, independent of its changed button.
    pub buttons: PointerButtons,
    /// Modifier keys held for the event.
    pub modifiers: Modifiers,
    /// The clicks (taps for a contact) Arvane counted for this press; a
    /// release has its press's: a double click or double tap is 2.
    pub click_count: u8,
    /// Mouse, touch or pen.
    pub kind: PointerKind,
    /// Pressure from 0 to 1, when the device reports it.
    pub pressure: Option<f32>,
    /// Pen tilt from vertical in degrees, when reported by the device.
    pub tilt: Option<(f32, f32)>,
    /// Whether this is the primary active pointer of its kind.
    pub is_primary: bool,
}

/// System preference for a light or dark color scheme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeHint {
    #[default]
    Light,
    Dark,
}

/// The window's environment as the client sees it: the whole value, at
/// start (`HostInfo::env`) and again whenever any of it changes
/// (`HostEvent::EnvChanged`). The host keeps one and sends a new one only
/// when it differs from the last it sent.
#[derive(Clone, Debug, PartialEq)]
pub struct HostEnv {
    /// Physical pixels per logical pixel.
    pub scale: f32,
    /// The system asks for reduced motion.
    pub reduced_motion: bool,
    /// Light at start; the host follows the system's theme from its first
    /// change.
    pub theme_hint: ThemeHint,
    /// Display refresh period (ns); used for `FrameRate::Display`.
    pub frame_period_ns: u64,
    /// The part of each window edge (logical pixels) system UI may cover
    /// (a notch, a status bar); zero on desktops. The host makes a negative
    /// or non-finite side zero.
    pub safe_area: crate::Insets,
    /// The window's size in logical pixels.
    pub window: Size,
    /// The system's language and region as a BCP 47 tag (`"ja-JP"`), or
    /// `None` where the host cannot tell.
    pub locale: Option<String>,
    /// The window is borderless fullscreen.
    pub fullscreen: bool,
    /// Frames can be drawn: the application is in the foreground and the
    /// window has a surface and is not empty. While it is `false` no frame
    /// ticks are sent.
    pub surface_active: bool,
}

/// Result of a platform request; cancellation is represented by an absent path.
#[derive(Clone, Debug, PartialEq)]
pub enum PlatformResult {
    Done,
    Text(String),
    Path(Option<PathBuf>),
    Paths(Option<Vec<PathBuf>>),
    Failed(String),
    Unsupported,
    /// The family names of a registered font file (`RegisterFont`).
    Fonts(Vec<String>),
}

/// What the platform's input method (IME) does with a node that has an IME
/// session (`NodeProperties::ime`). Offsets are UTF-8 byte offsets.
#[derive(Clone, Debug, PartialEq)]
pub enum ImeEvent {
    /// The composition (preedit) is now `text`; `selection` is the part of
    /// it the IME marks (its caret), if it says.
    Preedit {
        text: String,
        selection: Option<Range<usize>>,
    },
    /// The IME committed `text`; any composition ended with it.
    Commit(String),
    /// The IME took this range of the session's text for reconversion: the
    /// next composition replaces it.
    Selection(Range<usize>),
    /// The IME session ended without a commit: the composition is gone.
    End,
}

/// An actual application lifecycle transition or platform memory warning.
/// Surface availability and visibility remain current state in [`HostEnv`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// Application execution has been suspended by the shell.
    Suspended,
    /// The shell resumed application execution, possibly without a surface yet.
    Resumed,
    /// The platform asks the application to release rebuildable resources.
    MemoryWarning,
}

/// What the engine tells the client. Every kind goes into one ordered
/// queue, in the order produced (never one list per kind), handed whole to
/// each turn: input (`Pointer`, `Wheel`, `File`, `Key`, `Activate`), focus
/// and hover (`FocusChanged`, `FocusResult`, `HoverChanged`), text editing
/// (`TextChanged`, `SelectionChanged`), the input method of a node with an
/// IME session (`Ime`), geometry (`ScrollChanged`,
/// `LayoutChanged`), assistive technology and dismissal (`A11y`,
/// `OverlayDismiss`), time and platform answers (`FrameTick`,
/// `EffectResult`), the window and the platform (`EnvChanged`,
/// `Lifecycle`, `CloseRequested`), the GPU texture sharing
/// (`ExternalImported`, `GpuAdapterChanged`; a host or client built without its
/// own `gpu-interop` never sends them and ignores them) and reports
/// (`HostFailure`, `TurnsCut`, `AssetFailed`, `DrawDropped`, `LayoutWarning`).
#[derive(Clone, Debug, PartialEq)]
pub enum HostEvent {
    Pointer {
        node: NodeId,
        region: Option<RegionId>,
        event: PointerEvent,
    },
    Wheel {
        node: NodeId,
        region: Option<RegionId>,
        delta: Point,
        pos: Point,
        modifiers: Modifiers,
    },
    File {
        node: NodeId,
        phase: FilePhase,
        files: Vec<PathBuf>,
        pos: Point,
        window_pos: Point,
    },
    /// Key down (with `repeat`), sent to the focused node (None = window).
    Key {
        node: Option<NodeId>,
        region: Option<RegionId>,
        key: Key,
        code: KeyCode,
        released: bool,
        modifiers: Modifiers,
        repeat: bool,
        text: Option<String>,
        /// Where the mouse was in the window (logical pixels) when the key
        /// arrived; `None` while it is outside the window and before it
        /// first entered. A touch or pen contact does not move it.
        pointer: Option<Point>,
    },
    /// Default action by pointer click (a tap included) or assistive
    /// technology (UI Automation's Invoke) only. Enter/Space arrive as
    /// `Key` and are turned into activation by the core.
    Activate {
        node: NodeId,
        region: Option<RegionId>,
        /// The pointer whose click produced it (`None`: assistive
        /// technology), so the core can skip a click whose gesture a canvas
        /// adopted (`Writer::adopt_pointer`).
        pointer: Option<PointerId>,
    },
    FocusChanged {
        node: Option<NodeId>,
        region: Option<RegionId>,
        visible: bool,
    },
    /// The node under the pointer (the head of the host's hover chain:
    /// the hit itself, whatever pointer listener or capture there is), sent
    /// when it changes. `None`: nothing (outside the
    /// window, a cancelled pointer, a hit removed, hidden or outside the
    /// shown modal).
    HoverChanged {
        node: Option<NodeId>,
    },
    /// Host-side edit of a TextInput: the changes of its text in order
    /// (byte ranges, each in the text the previous one left), its session
    /// after them, and its whole text when its `ReportText` prop is set.
    TextChanged {
        node: NodeId,
        changes: Vec<kivact_editing::TextChange>,
        session: kivact_editing::TextSession,
        text: Option<String>,
    },
    /// The editing session of a TextInput whose `ReportSession` prop is
    /// set changed (a caret move, a selection, an edit): the session now.
    SelectionChanged {
        node: NodeId,
        session: kivact_editing::TextSession,
    },
    /// The input method acts on the focused node that has an IME session
    /// (not a native editor): composition, commit or reconversion.
    Ime {
        node: NodeId,
        event: ImeEvent,
    },
    ScrollChanged {
        node: NodeId,
        offset: Point,
        viewport: Size,
        content: Size,
    },
    /// Geometry of an observed node: where its box is drawn (the window
    /// bounds of its layout box mapped by its transforms) and its layout
    /// size (what it draws and lays out in).
    LayoutChanged {
        node: NodeId,
        rect: Rect,
        size: Size,
    },
    A11y {
        node: NodeId,
        region: Option<RegionId>,
        action: A11yAction,
        value: Option<String>,
    },
    /// An outside pointer press requested the overlay's offered Dismiss action.
    /// This preserves its origin separately from an accessibility request.
    OverlayDismiss {
        node: NodeId,
    },
    FrameTick {
        now_ns: u64,
        estimated_present_ns: u64,
        /// The engine's motion clock at this tick.
        clock: MotionClock,
    },
    EffectResult {
        request: RequestId,
        result: PlatformResult,
    },
    FocusResult {
        request: RequestId,
        ok: bool,
        reason: Option<String>,
    },
    /// Lifecycle notification after updated environment and host cache release.
    Lifecycle(LifecycleEvent),
    /// The window's environment changed: the new value, whole. Sent
    /// whenever any field of `HostEnv` differs from the last one the client
    /// was given (the scale or logical size, the theme, reduced motion,
    /// refresh period, safe area, locale, fullscreen, or whether anything
    /// can be drawn: no surface, an empty window, or the application
    /// suspended in the background, `surface_active`), one event for any
    /// of them.
    EnvChanged(HostEnv),
    /// A physical failure applying `revision`, reported as a diagnostic.
    HostFailure {
        revision: Revision,
        reason: String,
    },
    /// The host stopped running turns after `turns` turns in a row each
    /// left new events (a handler or an effect answering every layout or
    /// focus report with a write that reports again); the rest runs at its
    /// next wake. Reported as an `ErrorKind::Cycle` diagnostic.
    TurnsCut {
        turns: u32,
    },
    /// An asset of `node` failed: its image could not be loaded or decoded,
    /// or the font family its text asks for is not installed. `message`
    /// names the source (for a file, its absolute path and the current
    /// directory) and the reason. Sent once per node and image, and once
    /// per missing family (on the first node using it); the node draws
    /// nothing (an image) or uses the system UI font (a family).
    AssetFailed {
        node: NodeId,
        message: String,
    },
    /// The engine dropped `count` commands of the draw scenes of `node`
    /// because their values cannot be drawn (a non-finite coordinate, arc
    /// or gradient stop, a stroke or radius that is negative or not finite).
    /// Sent once per draw layer of the node, again only after the layer has
    /// been drawn without dropping a command. A gradient by coordinates (a
    /// canvas gradient) that cannot be drawn paints nothing and is not
    /// counted.
    DrawDropped {
        node: NodeId,
        count: u32,
    },
    /// The text `node` has a layout defect (debug builds only; sent once
    /// per node and rule, again when its content or box changes). Not an
    /// error: the check is a lint (`LayoutCheck`).
    LayoutWarning {
        node: NodeId,
        rule: LayoutRule,
        amount: f32,
    },
    CloseRequested,
    /// The window carried out `Writer::import_external` of `image` (after
    /// the turn that wrote it).
    ExternalImported {
        image: kivact_types::ExternalImageId,
        result: ExternalImport,
    },
    /// The adapter of the window's GPU device that can import shared
    /// textures (`Window::gpu_adapter`) changed: a device became ready, a
    /// new device replaced it (a new generation: every image imported
    /// before is gone) or the window draws on the CPU (`None`). The client
    /// starts with `None`; the host sends the first adapter when the
    /// device is ready.
    GpuAdapterChanged(Option<kivact_types::GpuAdapter>),
}

/// How the window answered an import of shared textures
/// (`HostEvent::ExternalImported`).
#[derive(Clone, Debug, PartialEq)]
pub enum ExternalImport {
    /// Imported into the device of this generation
    /// (`GpuAdapter::generation`).
    Ready { generation: u64 },
    /// The window has no device that can import (it draws on the CPU, its
    /// device is not ready, or the platform has none): Arvane's
    /// `ErrorKind::Unavailable`, with its message.
    Unavailable(String),
    /// The textures do not fit what they say or the window's device, or
    /// the device failed (Arvane's `Content` and `Backend`), with its
    /// message.
    Failed(String),
}
