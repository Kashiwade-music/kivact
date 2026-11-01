//! Role: Neutral UI vocabulary: geometry, ids, input events and draw commands.
//!
//! Responsibility: the types the adapter crates exchange. Text hands glyph
//! runs to render, the facade hands semantics to accessibility, and so on,
//! only through the types defined here.
//!
//! Invariants:
//! - No dependency at all, not even vocabulary crates such as `kurbo` or
//!   `peniko`. Conversions live in the adapter that needs them.
//! - Plain data only: no I/O, no threads, no global state.
//!
//! Coordinates are logical pixels with x increasing rightwards and y
//! downwards. Each operation names its origin: scenes and input use window
//! space, shaped text uses its layout origin, and semantics use unscrolled
//! content space. Image source rectangles use image pixels; raster dimensions
//! use device pixels. Text spans index the input's UTF-8 bytes; carets
//! index Unicode scalars of the text after case conversion.
//!
//! Owned values are portable snapshots. Font and image bytes use shared
//! `Arc` allocations, so a scene can outlive the engine that produced it.
//! Borrowed [`RichText`] inputs need only live for the shaping call.
//!
//! What it holds: logical-pixel geometry and [`Transform`]s; the drawing
//! vocabulary of [`DrawCommand`] (shapes, paints, layers, masks, shadows,
//! backdrops, color matrices; `arvane-render`'s `draw.rs` gives it its
//! meaning, the same on the CPU and the GPU); glyph runs carrying their
//! font bytes, and the text style values; semantics trees; decoded
//! straight-alpha images ([`ImageData`]) and premultiplied RGBA8 frames
//! ([`CpuFrame`]); pointer ids, kinds and contact phases; a window's
//! [`Environment`] (scale, refresh period, reduced motion, theme, safe area
//! [`Insets`], [`MouseSettings`], locale) and its [`Capabilities`]; the GPU
//! adapter wished for and used ([`GpuPreference`], [`GpuAdapter`]) and the
//! textures an external render engine shares ([`SharedTextures`], drawn by
//! [`DrawCommand::External`]).
//!
//! A value with one meaning is defined here once and re-exported by the
//! hosts above Arvane. Arvane itself uses these values directly, never
//! mirrored by a type of the same meaning and converted back and forth.
//! Only values whose meanings differ are converted where they meet (a
//! host's CSS box shadow as a [`DrawCommand::Shadow`] or a [`BoxShadow`],
//! a decomposed transform as a [`Transform`] matrix). The builders and the
//! standard trait implementations of a value live with its definition.

#![forbid(unsafe_code)]

mod asset;
mod color;
mod cursor;
mod effect;
mod geometry;
mod gpu_interop;
mod image;
mod input;
mod paint;
mod platform;
mod scene;
mod semantics;
mod semantics_text;
mod shape;
mod style;
mod text;
mod transform;

pub use asset::Asset;
pub use color::ColorMatrix;
pub use cursor::Cursor;
pub use effect::{Backdrop, BlendMode, BoxShadow, Layer, LayerFilter, Mask};
pub use geometry::{IntrinsicSize, NodeId, Point, Rect, Size};
pub use gpu_interop::{
    AdapterMatch, ExternalFrame, ExternalImageId, GpuAdapter, GpuBackend, GpuPower, GpuPreference,
    SharedHandle, SharedTextureFormat, SharedTextures,
};
pub use image::{ImageData, ImageFit};
pub use input::{FilePhase, Key, KeyCode, Modifiers, MouseButton};
pub use paint::{Extend, Gradient, GradientShape, Paint, Pattern, PatternSource};
pub use platform::{
    Capabilities, Environment, Insets, MouseSettings, PointerButtons, PointerId, PointerInput,
    PointerKind, PointerPhase,
};
pub use scene::{
    CpuFrame, DrawCommand, Glyph, GlyphRun, SHADOW_REACH, Scene, ShapedText, TextCaret, TextLine,
};
pub use semantics::{
    A11yAction, LiveRegion, Orientation, Role, SemanticsNode, SemanticsProperties, SemanticsTree,
    SemanticsUpdate, SortDirection,
};
pub use semantics_text::{SemanticsText, SemanticsTextRun};
pub use shape::{FillRule, LineCap, LineJoin, PathCmd, Radius, RoundedRect, Shape, Stroke};
pub use style::Rgba;
pub use text::{
    DecorationLines, FontFeature, FontKerning, LineHeight, RichText, SpanStyle, TextAlign,
    TextDecoration, TextDecorationRect, TextHighlight, TextMetrics, TextOverflow, TextSpan,
    TextStyle, TextTransform, TextWrap,
};
pub use transform::Transform;

pub use platform::ParentWindow;
