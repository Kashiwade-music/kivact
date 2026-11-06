//! Role: Classify external text and rendering failures using shared failure values.

pub use kivact_types::{Failure, FailureKind};

pub(crate) fn text_kind(error: &arvane_text::TextError) -> FailureKind {
    match error {
        arvane_text::TextError::InvalidMetrics
        | arvane_text::TextError::InvalidSpans
        | arvane_text::TextError::InvalidInlineBoxes
        | arvane_text::TextError::UnsupportedInlineStyle => FailureKind::Content,
        arvane_text::TextError::InvalidFont | arvane_text::TextError::MissingFont => {
            FailureKind::Resource
        }
    }
}

pub(crate) fn text(error: arvane_text::TextError) -> Failure {
    Failure::new(text_kind(&error), error.to_string())
}

/// Rendering failures; an invalid image in a scene is invalid content.
pub(crate) fn render(error: arvane_render::RenderError) -> Failure {
    let kind = match &error {
        arvane_render::RenderError::Backend(_) => FailureKind::Backend,
        arvane_render::RenderError::NoAdapter(_) => FailureKind::Unavailable,
        arvane_render::RenderError::Surface(_) => FailureKind::Surface,
        _ => FailureKind::Content,
    };
    Failure::new(kind, error.to_string())
}
