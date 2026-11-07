//! Role: Defines recoverable rendering errors.

use std::fmt;

/// Recoverable invalid scene, image, output-size or composition error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    /// The frame dimensions or display scale are invalid or exceed the budget.
    InvalidSize,
    /// A scene coordinate, paint, or stroke is invalid.
    InvalidGeometry,
    /// Clip, transform or group push/pop commands are unbalanced,
    /// interleaved, or nested more than 256 deep.
    UnbalancedClip,
    /// The image's dimensions and byte count do not match.
    InvalidImage,
    /// Font bytes, face index, size, or glyph coordinates are invalid.
    InvalidFont,
    /// The composition handed to the compositor is invalid.
    InvalidComposition(String),
    /// GPU adapter, device, rasterization, or readback failed.
    Backend(String),
    /// No hardware GPU adapter is available.
    NoAdapter(String),
    /// A window's GPU surface cannot present (recreate it).
    Surface(String),
    /// The GPU renderer cannot draw this scene within its limits (an atlas
    /// or intermediate texture allocation failed, a texture binding is
    /// missing); the CPU renderer can, and the device is fine.
    GpuLimit(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidComposition(message) => {
                return write!(f, "invalid composition: {message}");
            }
            Self::Backend(message) => return write!(f, "GPU backend failed: {message}"),
            Self::NoAdapter(message) => return write!(f, "no hardware GPU adapter: {message}"),
            Self::Surface(message) => return write!(f, "GPU surface failed: {message}"),
            Self::GpuLimit(message) => {
                return write!(f, "the GPU renderer cannot draw the scene: {message}");
            }
            _ => {}
        }
        f.write_str(match self {
            Self::InvalidSize => "invalid or excessive frame size or scale",
            Self::InvalidGeometry => "invalid scene geometry or paint",
            Self::UnbalancedClip => "unbalanced or too deeply nested groups",
            Self::InvalidImage => "invalid image data",
            Self::InvalidFont => "invalid glyph run or font face",
            Self::InvalidComposition(_)
            | Self::Backend(_)
            | Self::NoAdapter(_)
            | Self::Surface(_)
            | Self::GpuLimit(_) => {
                unreachable!("formatted with their messages above")
            }
        })
    }
}

impl std::error::Error for RenderError {}
