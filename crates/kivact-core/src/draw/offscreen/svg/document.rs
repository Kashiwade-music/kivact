//! Role: Writes a recorded drawing's commands as one SVG document, in order.
use super::emitter::Emitter;
use crate::Result;
use crate::geometry::Size;
use kivact_engine::api::{OffscreenRenderer, TextMeasurer};
use kivact_types::{ComputedTextStyle, DrawList};
use std::panic::Location;

/// The SVG document of `scene`, whose logical size is `size`. Errors are
/// located at `at`, the call of `Offscreen::render_svg`.
pub(in crate::draw::offscreen) fn write(
    engine: &mut OffscreenRenderer,
    scene: &DrawList,
    measurer: &TextMeasurer,
    inherited: &ComputedTextStyle,
    size: Size,
    at: &'static Location<'static>,
) -> Result<String> {
    let mut emitter = Emitter::new(engine, measurer, inherited, at);
    for cmd in &scene.cmds {
        emitter.command(cmd)?;
    }
    emitter.finish(size.width, size.height)
}
