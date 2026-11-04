//! Role: Shared pure UI values and backend-neutral text queries.
#![forbid(unsafe_code)]

pub mod asset;
pub mod check;
pub mod color;
pub mod env;
pub mod geometry;
pub mod gpu;
pub mod ids;
pub mod input;
pub mod layout;
pub mod motion;
pub mod paint;
pub mod platform;
pub mod protocol;
pub mod scene;
pub mod semantics;
pub mod semantics_text;
pub mod text;
pub mod text_shaping;

pub use asset::Asset;
pub use check::{LayoutCheck, LayoutFinding, LayoutLimit, LayoutRule};
pub use color::Rgba;
pub use env::{Capabilities, HostEnv, LifecycleEvent, ThemeHint};
pub use geometry::{Edges, Insets, Point, Rect, Size};
pub use gpu::{
    AdapterMatch, ExternalFrame, ExternalImageId, ExternalImport, GpuAdapter, GpuBackend, GpuPower,
    GpuPreference, SharedHandle, SharedTextureFormat, SharedTextures,
};
pub use ids::{
    FocusTarget, IdHasher, IdMap, NodeId, RegionId, RequestId, Revision, SemanticsId, TextRunId,
};
pub use input::{
    ButtonPress, FilePhase, ImeEvent, Key, KeyCode, KeyPhase, Modifiers, MouseButton,
    PointerButtons, PointerEvent, PointerId, PointerKind, PointerPhase,
};
pub use layout::{
    Align, AvailableSpace, Baseline, Breadth, Dimension, EditLineLayout, FlexDirection,
    GridContainer, GridFlow, GridLine, GridPos, GridProps, GridRepeat, GridTrack, IntrinsicSize,
    Justify, LayoutProps, LeafSize, MeasureRequest, Overflow, OverlayAlign, OverlayAt, OverlayFit,
    OverlayProps, Placement, Position, ScrollAxes, TrackSize,
};
pub use motion::{
    Easing, Kind, MotionClock, PlayDirection, Repeat, Spring, SpringRun, StepPosition, Transition,
    lerp,
};
pub use paint::{
    BlendMode, Border, ClipShape, CoordinateGradient, CoordinateShape, CornerRadius, Corners,
    Cursor, Effects, ElementTransform, Extend, Fill, FillImage, FillLayer, FillSize, FilterOp,
    Gradient, GradientKind, GradientStop, ImageData, ImageFit, Len, PaintDelta, PaintProps,
    PaintVec, RadialExtent, Ring, ScrollbarDelta, ScrollbarProps, Shadow, StateStyles,
};
pub use platform::{FileDialogRequest, PlatformRequest, PlatformResult};
pub use protocol::{
    FontFeature, InlineBox, InlineBoxPlacement, ParentWindow, PathCmd, ShapedInlineText,
    ShapedText, TextLine, TextWrap,
};
pub use scene::{
    BorderStyle, DrawingLayer, DrawingShape, FillRule, LineCap, LineJoin, Pixels, Region, Scene,
    SceneCmd, ShapePaint, Stroke, TextRun,
};
pub use semantics::{
    A11yAction, A11yRequest, CheckedState, CurrentItem, DismissReason, LiveRegion, Orientation,
    PopupKind, Role, SemanticsProperties, SortDirection, check_text,
};
pub use semantics_text::{SemanticsText, SemanticsTextRun, TextCaret};
pub use text::{
    ComputedText, DecorationLines, FontKerning, LineHeight, SpanProps, TextAlign, TextData,
    TextDecoration, TextLen, TextMetrics, TextMore, TextOverflow, TextProps, TextShadow, TextSpan,
    TextTransform,
};
pub use text_shaping::TextShaper;
