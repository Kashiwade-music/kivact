//! Role: Pure shared UI values and small std-only synchronization helpers.
//!
//! The vocabulary every Kivact and Arvane crate shares: geometry, colors,
//! ids, input, semantics, the node props a style compiles to, recorded
//! drawing and shaped text. `cell` and `process` hold the few std-only
//! helpers with interior mutability (latches, mailboxes, deferred
//! values, the detach tracker). The frame protocol of Arvane is
//! `arvane-protocol`; engine-only computation lives in the engine.
#![forbid(unsafe_code)]

pub mod asset;
pub mod cell;
pub mod check;
pub mod color;
pub mod color_space;
pub mod defaults;
pub mod drag_velocity;
pub mod effect;
pub mod embed;
pub mod env;
pub mod export;
pub mod geometry;
pub mod geometry3d;
pub mod gpu;
pub mod gradient_stops;
pub mod ids;
pub mod image_status;
pub mod input;
pub mod layout;
pub mod motion;
pub mod paint;
pub mod pixel;
pub mod platform;
pub mod presence;
pub mod process;
pub mod scene;
pub mod scroll;
pub mod semantics;
pub mod semantics_text;
pub mod shaped;
pub mod style_state;
pub mod text;
pub mod text_shaping;
pub mod text_utf;
pub mod type_map;
pub mod window;

pub use asset::{Asset, AssetBytes};
pub use check::{LayoutCheck, LayoutFinding, LayoutLimit, LayoutRule};
pub use color::Rgba;
pub use color_space::{
    ColorInterpolation, ColorSpace, HueInterpolation, SpaceColor, delta_e_ok, interpolate, mix_rgba,
};
pub use drag_velocity::DragVelocity;
pub use effect::{
    Channel, CompositeOp, EdgeMode, FilterColorSpace, FilterGraph, FilterInput, Light,
    MorphologyOp, ParamSlot, ParamType, Primitive, PrimitiveOp, ShaderKind, ShaderProgram,
    ShaderUse, TransferFn, TurbulenceKind,
};
pub use embed::{EmbedOptions, ResizeAnswer, UnhandledKeys};
pub use env::{
    Capabilities, CommandKey, ContextMenuTrigger, Environment, LifecycleEvent, PointerClass,
    PointerSettings, ThemeHint,
};
pub use export::{ExportFrame, ExportedFrame, FrameStats};
pub use geometry::{Edge, Edges, Insets, Point, Rect, Size, Transform};
pub use geometry3d::{Decomposed, Matrix4, Polygon, Projective, W_EPS};
pub use gpu::{
    AdapterIds, AdapterMatch, ExternalFrame, ExternalImageId, ExternalImport, GpuAdapter,
    GpuBackend, GpuPower, GpuPreference, GpuStart, SHARED_PREPARED, SharedHandle,
    SharedTextureFormat, SharedTextures,
};
pub use gradient_stops::{color_at, expand_gradient, normalized_stops};
pub use ids::{
    DrawableId, FocusTarget, IdHasher, IdMap, NodeId, RegionId, RequestId, Revision, SemanticsId,
    TextRunId,
};
pub use image_status::ImageStatus;
pub use input::{
    ButtonPress, FileDrop, FilePhase, ImeInput, InputPurpose, Key, KeyCode, KeyInput, KeyPhase,
    Modifiers, MouseButton, PointerButtons, PointerId, PointerKind, PointerPhase, PointerSample,
    WheelInput,
};
pub use layout::{
    Align, AvailableSpace, Baseline, Breadth, Dimension, Display, EditLineLayout, FlexDirection,
    GridContainer, GridFlow, GridLine, GridPos, GridProps, GridRepeat, GridTrack, IntrinsicSize,
    Justify, LayoutProps, LeafSize, MeasureRequest, Overflow, OverlayAlign, OverlayAt, OverlayFit,
    OverlayProps, Placement, Position, ScrollAxes, TrackSize, Visibility,
};
pub use motion::{
    Easing, Keyframe, Kind, MotionClock, PlayDirection, Repeat, Retarget, Spring, SpringRun,
    StepPosition, TimeScale, Transition, TransitionProp, lerp,
};
pub use paint::{
    BackfaceVisibility, BackgroundClip, BackgroundOrigin, Border, BorderImage, BorderImageRepeat,
    ClipShape, CoordinateGradient, CoordinateShape, CornerRadius, Corners, Cursor, Effects,
    ElementTransform, Extend, Fill, FillImage, FillLayer, FillSize, FilterOp, Gradient,
    GradientKind, GradientStop, ImageData, ImageFit, Isolation, Len, MaskComposite, MaskLayer,
    MaskMode, PaintDelta, PaintProps, Perspective, RadialExtent, Ring, ScrollbarDelta,
    ScrollbarProps, ShaderClock, ShaderSpec, Shadow, StateStyles, TransformStyle, WillChange,
};
pub use pixel::{BlendMode, ColorMatrix, Composite, Mix, Pixels};
pub use platform::{Failure, FailureKind, FileDialog, PlatformRequest, PlatformResult};
pub use presence::Presence;
pub use scene::{
    BorderStyle, DrawCmd, DrawList, DrawingLayer, DrawingShape, FillRule, LineCap, LineJoin,
    PathCmd, PathData, Region, ShapePaint, Stroke, TextRun,
};
pub use scroll::{ScrollSnap, nearest_snap};
pub use semantics::{
    A11yAction, A11yRequest, CheckedState, CurrentItem, DismissReason, Finite, LiveRegion,
    Orientation, PopupKind, Role, Semantics, SortDirection, check_text,
};
pub use semantics_text::{SemanticsText, SemanticsTextRun, SemanticsTextRuns, TextCaret};
pub use shaped::{
    InlineBox, InlineBoxPlacement, ShapedInlineText, ShapedText, ShapedTextSnapshot, TextLine,
};
pub use style_state::{StateSet, StyleState};
pub use text::{
    ComputedTextStyle, DecorationLines, FontFeature, FontKerning, FontVariation, LineHeight,
    PaintOrder, SpanProps, TextAlign, TextData, TextDecoration, TextDecorationStyle, TextLen,
    TextMetrics, TextOverflow, TextShadow, TextSpan, TextStroke, TextStyle, TextTransform,
    TextWrap,
};
pub use text_shaping::TextShaper;
pub use type_map::{LocalTypeMap, SharedTypeMap};
pub use window::{
    NativeWindow, SurfaceSize, WindowContent, WindowId, WindowMode, WindowPlacement, WindowRequest,
};

pub mod animatable;
pub use animatable::Animatable;

pub mod view_transform;
pub use view_transform::ViewTransform;
