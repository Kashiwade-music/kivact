//! Role: Shape a paragraph with the App's text engine outside drawing and measurement.

use crate::view::state::ViewState;
use kivact_style::draw::{InlineParagraph, TextParagraph, TextRun, TextStyle};
use kivact_wire::{self as wire, TextShaper};
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
    let measurer = paragraph_measurer("shape_paragraph", Location::caller());
    let run = text_run(text, style);
    let shaped = measurer.shape(run.as_wire(), max_width).unwrap_or_default();
    TextParagraph::from_shaped(text, shaped, max_width)
}

/// Shape text and atomic inline boxes using the current App and theme defaults.
#[track_caller]
pub fn shape_inline_paragraph(
    text: &str,
    style: &TextStyle,
    max_width: Option<f32>,
    boxes: &[wire::InlineBox],
) -> Option<InlineParagraph> {
    let measurer = paragraph_measurer("shape_inline_paragraph", Location::caller());
    let run = text_run(text, style);
    let shaped = measurer.shape_inline(run.as_wire(), max_width, boxes)?;
    Some(InlineParagraph::from_shaped(text, shaped, max_width))
}

fn paragraph_measurer(api: &'static str, at: &'static Location<'static>) -> impl TextShaper {
    let rt = kivact_reactive::Owner::ambient(api, at).runtime();
    let theme = kivact_style::current_theme();
    let inherited = wire::ComputedText {
        family: theme.text().family.clone(),
        line_height: Some(theme.text().line_height),
        ..Default::default()
    };
    ViewState::tree(&rt).reader().paragraph_measurer(inherited)
}

fn text_run(text: &str, style: &TextStyle) -> TextRun {
    TextRun::from_style(
        text,
        wire::Point::default(),
        style,
        1.0,
        wire::Rgba::TRANSPARENT,
        None,
    )
}
