//! Role: Defines recoverable rendering errors.

use std::fmt;

/// Recoverable invalid scene, image, or output-size error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    /// The frame dimensions or display scale are invalid or exceed the budget.
    InvalidSize,
    /// A scene coordinate, paint, or stroke is invalid.
    InvalidGeometry,
    /// Clip, transform or layer push/pop commands are unbalanced,
    /// interleaved, or nested more than 256 deep.
    UnbalancedClip,
    /// The image's dimensions and byte count do not match.
    InvalidImage,
    /// Font bytes, face index, size, or glyph coordinates are invalid.
    InvalidFont,
    /// Shared textures cannot be imported: they do not match what they
    /// say or the window's device (size, format, adapter, graphics API),
    /// or the device refused their handles.
    External(String),
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
            Self::Backend(message) => return write!(f, "GPU backend failed: {message}"),
            Self::External(message) => {
                return write!(f, "the shared textures cannot be imported: {message}");
            }
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
            Self::Backend(_)
            | Self::NoAdapter(_)
            | Self::Surface(_)
            | Self::GpuLimit(_)
            | Self::External(_) => {
                unreachable!("GPU errors are formatted above")
            }
        })
    }
}

impl std::error::Error for RenderError {}
