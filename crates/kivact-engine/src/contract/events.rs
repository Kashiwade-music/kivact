//! Role: Engine client events contracts.
use kivact_types::{
    A11yRequest, Environment, ExternalImport, ImeInput, LayoutRule, LifecycleEvent, Modifiers,
    MotionClock, NodeId, PlatformResult, Point, PointerId, Rect, RegionId, RequestId, Revision,
    Size,
};

/// What the engine tells the client. Every kind goes into one ordered
/// queue, in the order produced (never one list per kind), handed whole to
/// each turn: input (`Pointer`, `Wheel`, `File`, `Key`, `Activate`), focus
/// and hover (`FocusChanged`, `FocusResult`, `HoverChanged`), text editing
/// (`TextChanged`, `SelectionChanged`), the input method of a node with an
/// IME session (`Ime`), geometry (`ScrollChanged`,
/// `LayoutChanged`), assistive technology and dismissal (`A11y`), time and platform answers (`FrameTick`,
/// `EffectResult`), the window and the platform (`EnvChanged`,
/// `Lifecycle`, `CloseRequested`), the GPU texture sharing
/// (`ExternalImported`, `GpuAdapterChanged`; a host or client built without its
/// own `gpu-interop` never sends them and ignores them) and reports
/// (`HostFailure`, `TurnsCut`, `AssetFailed`, `DrawDropped`, `LayoutWarning`).
#[derive(Clone, Debug, PartialEq)]
pub enum HostEvent {
    /// A requested removal has finished, including any retained exit paint.
    ExitComplete {
        node: NodeId,
    },
    Pointer(super::Routed<kivact_types::PointerSample>),
    Wheel(super::Routed<kivact_types::WheelInput>),
    File(super::Routed<kivact_types::FileDrop>),
    /// Key input, including a missing target for window-directed keys.
    Key(super::Routed<kivact_types::KeyInput>),
    /// The modifier keys held changed between events: a modifier key went
    /// down or up, or the window lost the keyboard (none are held). Sent
    /// only when they differ from the last ones; key, pointer and wheel
    /// events also carry the modifiers held for them.
    ModifiersChanged(Modifiers),
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
        target: Option<kivact_types::FocusTarget>,
    },
    /// Passive mouse location in logical window pixels, even without a hit.
    MousePosition(Option<Point>),
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
        event: ImeInput,
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
        /// A native ellipsis or line clamp cut this node's text.
        text_truncated: bool,
    },
    /// A request for an offered action: from assistive technology, or a
    /// `Dismiss(OutsidePress)` when a pointer press lands outside a shown
    /// overlay offering `Dismiss` and outside its anchor's subtree.
    A11y {
        node: NodeId,
        region: Option<RegionId>,
        request: A11yRequest,
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
    /// The answer to a focus request: `Err` names why the target did not take
    /// the focus.
    FocusResult {
        request: RequestId,
        result: Result<(), String>,
    },
    /// Lifecycle notification after updated environment and host cache release.
    Lifecycle(LifecycleEvent),
    /// The window's environment changed: the new value, whole. Sent
    /// whenever any field of `Environment` differs from the last one the client
    /// was given (the scale or logical size, the theme, reduced motion,
    /// refresh period, safe area, locale, fullscreen, or whether anything
    /// can be drawn: no surface, an empty window, or the application
    /// suspended in the background, `surface_active`), one event for any
    /// of them.
    EnvChanged {
        environment: Environment,
        surface_active: bool,
    },
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
        amount: f64,
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
