//! Role: Neutral UI vocabulary: geometry, ids, input and style values.
//!
//! Responsibility: the types the adapter crates exchange, in two parts.
//! The vocabulary, at the crate root, is what users of the `kivact` facade
//! see: geometry, ids, colors, input, text style keywords, layout keywords,
//! easing curves, pointer and lifecycle values. The [`protocol`] module is
//! the workspace's host-to-Arvane protocol: display lists, shaped text,
//! semantics trees and what a window reports; the facade re-exports none of
//! it. Protocol uses vocabulary, never the reverse.
//!
//! Invariants:
//! - No dependency at all, not even vocabulary crates such as `kurbo` or
//!   `peniko`. Conversions live in the adapter that needs them.
//! - Plain data only: no I/O, no threads, no global state.
//! - A value the facade re-exports is defined in the vocabulary, never in
//!   [`protocol`].
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
//! Borrowed [`protocol::RichText`] inputs need only live for the shaping call.
//!
//! What the vocabulary holds: logical-pixel geometry; pointer ids, kinds
//! and contact phases; safe-area [`Insets`] and a window's [`Capabilities`];
//! the GPU adapter wished for and used ([`GpuPreference`], [`GpuAdapter`])
//! and the textures an external render engine shares ([`SharedTextures`],
//! drawn by [`protocol::DrawCommand::External`]).
//!
//! What the protocol holds: [`protocol::Transform`]s and the drawing
//! vocabulary of [`protocol::DrawCommand`] (shapes, paints, layers, masks,
//! shadows, backdrops, color matrices; `arvane-render`'s `draw.rs` gives it
//! its meaning, the same on the CPU and the GPU); glyph runs carrying their
//! font bytes, and the rich text a host hands Arvane to shape; semantics
//! trees; decoded straight-alpha images ([`protocol::ImageData`]) and
//! premultiplied RGBA8 frames ([`protocol::CpuFrame`]); and a window's
//! [`protocol::Environment`] (scale, refresh period, reduced motion, theme,
//! safe area, [`protocol::MouseSettings`], locale) and
//! [`protocol::PointerInput`].
//!
//! A value with one meaning is defined here once and re-exported by the
//! hosts above Arvane. Arvane itself uses these values directly, never
//! mirrored by a type of the same meaning and converted back and forth.
//! Only values whose meanings differ are converted where they meet (a
//! host's CSS box shadow as a [`protocol::DrawCommand::Shadow`] or a
//! [`protocol::BoxShadow`], a decomposed transform as a
//! [`protocol::Transform`] matrix). The builders and the standard trait
//! implementations of a value live with its definition.

#![forbid(unsafe_code)]

mod asset;
mod cursor;
mod effect;
mod geometry;
mod gpu_interop;
mod image;
mod input;
mod layout;
mod motion;
mod paint;
mod platform;
pub mod protocol;
mod semantics;
mod semantics_text;
mod shape;
mod style;
mod text;

pub use asset::Asset;
pub use cursor::Cursor;
pub use effect::BlendMode;
pub use geometry::{IntrinsicSize, NodeId, Point, Rect, RegionId, Size};
pub use gpu_interop::{
    AdapterMatch, ExternalFrame, ExternalImageId, GpuAdapter, GpuBackend, GpuPower, GpuPreference,
    SharedHandle, SharedTextureFormat, SharedTextures,
};
pub use image::ImageFit;
pub use input::{FilePhase, ImeEvent, Key, KeyCode, Modifiers, MouseButton};
pub use layout::{
    Align, FlexDirection, Justify, Overflow, OverlayAlign, Placement, Position, ScrollAxes,
};
pub use motion::{Easing, Repeat, Spring, StepPosition};
pub use paint::Extend;
pub use platform::{
    ButtonPress, Capabilities, Insets, LifecycleEvent, PointerButtons, PointerId, PointerKind,
    PointerPhase,
};
pub use semantics::{
    A11yAction, A11yRequest, CheckedState, CurrentItem, DismissReason, LiveRegion, Orientation,
    PopupKind, Role, SortDirection,
};
pub use semantics_text::{SemanticsText, SemanticsTextRun, TextCaret};
pub use shape::{BorderStyle, FillRule, LineCap, LineJoin, Stroke};
pub use style::Rgba;
pub use text::{
    DecorationLines, FontKerning, LineHeight, TextAlign, TextMetrics, TextOverflow, TextTransform,
};
