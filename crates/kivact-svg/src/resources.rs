//! Role: The answers the SVG writer asks of the renderer that recorded it.

use crate::SvgError;
use kivact_types::{DrawCmd, Pixels, TextRun};

/// What the writer asks of the host that recorded the drawing. The host keeps
/// the text engine and the image renderer; the writer sees only the answers.
pub trait Resources {
    /// The family that a text run naming none takes (the theme's text family).
    fn fallback_family(&self) -> Option<&str>;

    /// The distance from the top of a run's line box to its baseline, as the
    /// text engine measures it.
    fn baseline(&self, run: &TextRun) -> f64;

    /// `cmd` (an image command placed at the origin) drawn into a `width` ×
    /// `height` pixel image. A size the host refuses is
    /// [`SvgError::Malformed`]; an image that cannot be drawn is
    /// [`SvgError::Resource`].
    fn rasterize(&mut self, cmd: DrawCmd, width: u32, height: u32) -> Result<Pixels, SvgError>;
}
