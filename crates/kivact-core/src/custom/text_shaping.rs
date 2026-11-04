//! Role: Shape a paragraph with the App's text engine outside drawing and measurement.

use crate::view::state::ViewState;
use kivact_style::draw::{InlineParagraph, TextFormat, TextParagraph, TextRun};
use kivact_types::{self as values, TextShaper};
use std::panic::Location;

/// Shape text outside drawing or measurement, using this App's text engine.
///
/// Callable in setup, handlers, effects and value closures. The current theme's
/// family and line height fill unspecified style fields; node-specific inherited
/// overrides require the same explicit fields here. Equal effective text, style
/// and width share the shaping cache with drawing and measurement. No engine
/// borrow survives this call and no reactive engine state is allocated.
#[track_caller]
pub fn shape_paragraph(text: &str, style: &TextFormat, max_width: Option<f64>) -> TextParagraph {
    let measurer = paragraph_measurer("shape_paragraph", Location::caller());
    let run = text_run(text, style);
    let shaped = measurer.shape(&run, max_width).unwrap_or_default();
    TextParagraph::from_shaped(text, shaped, max_width)
}

/// Shape text and atomic inline boxes using the current App and theme defaults.
#[track_caller]
pub fn shape_inline_paragraph(
    text: &str,
    style: &TextFormat,
    max_width: Option<f64>,
    boxes: &[values::InlineBox],
) -> Option<InlineParagraph> {
    let measurer = paragraph_measurer("shape_inline_paragraph", Location::caller());
    let run = text_run(text, style);
    let shaped = measurer.shape_inline(&run, max_width, boxes)?;
    Some(InlineParagraph::from_shaped(text, shaped, max_width))
}

fn paragraph_measurer(api: &'static str, at: &'static Location<'static>) -> impl TextShaper {
    let rt = kivact_runtime::Owner::ambient(api, at).runtime();
    let theme = kivact_style::use_theme().get();
    let inherited = values::ComputedTextStyle {
        family: theme.text().family.clone(),
        line_height: theme.text().line_height,
        ..Default::default()
    };
    ViewState::tree(&rt).reader().paragraph_measurer(inherited)
}

fn text_run(text: &str, style: &TextFormat) -> TextRun {
    kivact_style::draw::text_run(
        text,
        values::Point::default(),
        style,
        1.0,
        values::Rgba::TRANSPARENT,
        None,
    )
}
