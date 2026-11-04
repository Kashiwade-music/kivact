//! Role: Independent node, event, scene, measurement and motion protocol.
//!
//! Clients and hosts exchange these owned values without depending on a
//! window, renderer, reactive runtime or retained-tree implementation. Node
//! properties describe desired state, scenes describe drawing, and ordered
//! events report host results. Concrete write/read handles belong to the host
//! adapter, outside this crate.
//!
//! Geometry, input and semantics use the shared `kivact-types` vocabulary. Editing
//! payloads use `kivact-editing`; motion calculations are pure functions shared
//! by client animations and host state transitions. [`IdMap`] and [`IdHasher`]
//! are for monotonically allocated, never-reused node identities, not
//! attacker-controlled byte strings. The external texture values and
//! scene/event variants of GPU texture sharing are always part of the protocol;
//! a host or client built without its own `gpu-interop` never sends them and
//! ignores them when it receives them.
//!
//! Invariant: defines no value the kivact facade re-exports; facade-visible
//! values are `kivact-types` vocabulary re-exported here.
#![forbid(unsafe_code)]

mod check;
mod drawing;
mod events;
mod grid;
mod host;
mod id_hash;
mod ids;
mod measure;
pub mod motion;
mod node;
mod paint;
mod paint_vec;
mod scene;
mod semantics;
mod style;
mod text;
mod values;

pub use check::{LayoutCheck, LayoutFinding, LayoutLimit, LayoutRule};
pub use drawing::{DrawingLayer, DrawingShape, LineCap, LineJoin, Pixels, ShapePaint, Stroke};
/// GPU texture sharing: how the window answered an import.
pub use events::ExternalImport;
pub use events::{
    ButtonPress, FilePhase, HostEnv, HostEvent, Key, KeyCode, KeyPhase, Modifiers, MouseButton,
    PlatformResult, PointerButtons, PointerEvent, PointerId, PointerKind, PointerPhase, ThemeHint,
};
pub use grid::{
    Breadth, GridContainer, GridFlow, GridLine, GridPos, GridProps, GridTrack, Repeat, TrackSize,
};
/// The key of a text run in a node's [`SemanticsText`]: any value, the same
/// across updates for the same run (the host gives the platform its own
/// identity).
pub use kivact_types::TextRunId;
pub use kivact_types::protocol::{
    InlineBox, InlineBoxPlacement, PathCmd, ShapedInlineText, ShapedText, TextLine,
};
/// GPU texture sharing: kivact-types' vocabulary.
pub use kivact_types::{
    AdapterMatch, ExternalFrame, ExternalImageId, GpuAdapter, GpuBackend, GpuPower, GpuPreference,
    SharedHandle, SharedTextureFormat, SharedTextures,
};
pub use kivact_types::{
    Align, BorderStyle, FlexDirection, ImeEvent, Justify, LifecycleEvent, Overflow, OverlayAlign,
    Placement, Position, RegionId, ScrollAxes, TextMetrics,
};
pub use kivact_types::{Asset, ImageFit};
pub use kivact_types::{DecorationLines, FillRule, Insets, Point, Rect, Size};
pub use kivact_types::{SemanticsText, SemanticsTextRun, TextCaret};

pub use host::{Capabilities, HostError, HostInfo, TurnOutput};
pub use id_hash::{IdHasher, IdMap};
pub use ids::{FocusTarget, NodeId, RequestId, Revision};
pub use measure::{
    AvailableSpace, Baseline, LeafMeasure, LeafSize, LeafTextMeasurer, MeasureRequest, TextShaper,
};
pub use motion::MotionClock;
pub use node::{
    BoxData, EditLineLayout, EditingData, FileDialogRequest, ImageData, ImeSession, IntrinsicSize,
    NodeInput, NodeKind, NodeProperties, NodeSemantics, NodeType, OverlayAt, OverlayFit,
    OverlayProps, PlatformRequest, RovingFocus, SceneLayer, TextData,
};
pub use paint::{
    BlendMode, Border, ClipShape, CoordinateGradient, CoordinateShape, CornerRadius, Corners,
    Effects, Fill, FillImage, FillLayer, FillSize, FilterOp, Gradient, GradientExtend,
    GradientKind, GradientStop, Len, RadialExtent, Ring,
};
pub use paint_vec::PaintVec;
pub use scene::{Region, Scene, SceneCmd, TextRun};
pub use semantics::{
    A11yAction, A11yRequest, CheckedState, CurrentItem, DismissReason, LiveRegion, Orientation,
    PopupKind, Role, SemanticsProperties, SortDirection, check_text,
};
pub use style::{
    Cursor, ElementTransform, LayoutProps, PaintDelta, PaintProps, ScrollbarDelta, ScrollbarProps,
    Shadow, StateStyles,
};
pub use text::{
    ComputedText, FontFeature, FontKerning, LineHeight, SpanProps, TextAlign, TextDecoration,
    TextLen, TextMore, TextOverflow, TextProps, TextShadow, TextSpan, TextTransform, TextWrap,
};
pub use values::{Dimension, Edges, Rgba};

pub use kivact_types::protocol::ParentWindow;
