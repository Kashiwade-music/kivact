//! Role: Shared pure UI values and backend-neutral text queries.
#![forbid(unsafe_code)]

pub mod asset;
pub mod cell;
pub mod check;
pub mod color;
pub mod defaults;
pub mod embed;
pub mod env;
pub mod geometry;
pub mod gpu;
pub mod ids;
pub mod input;
pub mod layout;
pub mod motion;
pub mod paint;
pub mod platform;
pub mod presence;
pub mod process;
pub mod render;
pub mod scene;
pub mod semantics;
pub mod semantics_text;
pub mod text;
pub mod text_shaping;
pub mod text_utf;

pub use asset::Asset;
pub use check::{LayoutCheck, LayoutFinding, LayoutLimit, LayoutRule};
pub use color::Rgba;
pub use embed::{EmbedOptions, ParentWindow};
pub use env::{
    Capabilities, CommandKey, ContextMenuTrigger, Environment, LifecycleEvent, PointerClass,
    PointerSettings, ThemeHint,
};
pub use geometry::{Edges, Insets, Point, Rect, Size, Transform};
pub use gpu::{
    AdapterMatch, ExternalFrame, ExternalImageId, ExternalImport, GpuAdapter, GpuBackend, GpuPower,
    GpuPreference, SharedHandle, SharedTextureFormat, SharedTextures,
};
pub use ids::{
    FocusTarget, IdHasher, IdMap, NodeId, RegionId, RequestId, Revision, SemanticsId, TextRunId,
};
pub use input::{
    ButtonPress, FileDrop, FilePhase, ImeInput, Key, KeyCode, KeyInput, KeyPhase, Modifiers,
    MouseButton, PointerButtons, PointerId, PointerKind, PointerPhase, PointerSample, WheelInput,
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
pub use platform::{Failure, FailureKind, FileDialog, PlatformRequest, PlatformResult};
pub use presence::Presence;
pub use render::{InlineBox, InlineBoxPlacement, ShapedInlineText, ShapedText, TextLine};
pub use scene::{
    BorderStyle, DrawCmd, DrawList, DrawingLayer, DrawingShape, FillRule, LineCap, LineJoin,
    PathCmd, PathData, Pixels, Region, ShapePaint, Stroke, TextRun,
};
pub use semantics::{
    A11yAction, A11yRequest, CheckedState, CurrentItem, DismissReason, Finite, LiveRegion,
    Orientation, PopupKind, Role, Semantics, SortDirection, check_text,
};
pub use semantics_text::{SemanticsText, SemanticsTextRun, TextCaret};
pub use text::{
    ComputedTextStyle, DecorationLines, FontFeature, FontKerning, LineHeight, SpanProps, TextAlign,
    TextData, TextDecoration, TextLen, TextMetrics, TextOverflow, TextShadow, TextSpan, TextStyle,
    TextTransform, TextWrap,
};
pub use text_shaping::TextShaper;

pub mod animatable;
pub use animatable::Animatable;
