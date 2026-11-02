//! Role: Shape a paragraph with the App's text engine outside drawing and measurement.

use crate::view::state::ViewState;
use kivact_style::draw::{TextParagraph, TextRun, TextStyle};
use kivact_wire::{self as wire};
use std::panic::Location;

/// Shape text outside drawing or measurement, using this App's text engine.
///
/// Callable in setup, handlers, effects and value closures. The current theme's
/// family and line height fill unspecified style fields; node-specific inherited
/// overrides require the same explicit fields here. Equal effective text, style
/// and width share the shaping cache with drawing and measurement. No engine
/// borrow survives this call and no reactive engine state is allocated.
#[track_caller]
pub fn shape_paragraph(text: &str, style: &TextStyle, max_width: Option<f32>) -> TextParagraph {
    let rt = kivact_reactive::Owner::ambient("shape_paragraph", Location::caller()).runtime();
    let theme = kivact_style::current_theme();
    let inherited = wire::ComputedText {
        family: theme.text().family.clone(),
        line_height: Some(theme.text().line_height),
        ..Default::default()
    };
    let measurer = ViewState::tree(&rt).reader().paragraph_measurer(inherited);
    let run = TextRun::from_style(
        text,
        wire::Point::default(),
        style,
        1.0,
        wire::Rgba::TRANSPARENT,
        None,
    );
    let shaped = measurer.shape(run.as_wire(), max_width).unwrap_or_default();
    TextParagraph::from_shaped(text, shaped, max_width)
}
