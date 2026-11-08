//! Role: Shared pure UI values and backend-neutral text queries.
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
pub mod geometry;
pub mod geometry3d;
pub mod gpu;
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
pub mod render;
pub mod scene;
pub mod scroll;
pub mod semantics;
pub mod semantics_text;
pub mod text;
pub mod text_shaping;
pub mod text_utf;
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
pub use embed::{EmbedOptions, ParentWindow};
pub use env::{
    Capabilities, CommandKey, ContextMenuTrigger, Environment, LifecycleEvent, PointerClass,
    PointerSettings, ThemeHint,
};
pub use geometry::{Edge, Edges, Insets, Point, Rect, Size, Transform};
pub use geometry3d::{Decomposed, Matrix4, Polygon, Projective, W_EPS};
pub use gpu::{
    AdapterMatch, ExternalFrame, ExternalImageId, ExternalImport, GpuAdapter, GpuBackend, GpuPower,
    GpuPreference, SHARED_PREPARED, SharedHandle, SharedTextureFormat, SharedTextures,
};
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
mod prop_interp;
pub use pixel::{BlendMode, ColorMatrix, Pixels};
pub use platform::{Failure, FailureKind, FileDialog, PlatformRequest, PlatformResult};
pub use presence::Presence;
pub use prop_interp::{PropBasis, PropInterp, apply_components, prop_interp};
pub use render::{
    InlineBox, InlineBoxPlacement, ShapedInlineText, ShapedText, ShapedTextSnapshot, TextLine,
};
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
pub use text::{
    ComputedTextStyle, DecorationLines, FontFeature, FontKerning, FontVariation, LineHeight,
    PaintOrder, SpanProps, TextAlign, TextData, TextDecoration, TextDecorationStyle, TextLen,
    TextMetrics, TextOverflow, TextShadow, TextSpan, TextStroke, TextStyle, TextTransform,
    TextWrap,
};
pub use text_shaping::TextShaper;
pub use window::{WindowId, WindowMode, WindowPlacement, WindowRequest};

pub mod animatable;
pub use animatable::Animatable;

pub mod view_transform;
pub use view_transform::ViewTransform;
