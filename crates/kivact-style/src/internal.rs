//! Role: Bridge style values and recording to core host machinery.

pub use crate::draw::recorder::record;
use crate::paint::fill_layer;
pub use crate::resolve::compile_style;
pub use crate::sheet::with_scroll_axes;
use crate::text::{CompiledSpanStyle, SpanStyle, TextLength, decoration, features};
use crate::theme::value::Theme;
pub use crate::theme::value::{install_theme, theme_of};
use kivact_types as p;

/// Compile the properties that are set, resolving colors and the
/// background against `theme`.
pub fn compile_span_style(style: &SpanStyle, theme: &Theme) -> CompiledSpanStyle {
    let mut unsupported = 0;
    let features = (style.features.is_some() || style.tabular_nums.is_some()).then(|| {
        let (features, lost) =
            features(style.features.as_deref(), style.tabular_nums == Some(true));
        unsupported += lost;
        features
    });
    let variations = style.variations.as_deref().map(|list| {
        let (list, bad) = crate::text::wire_variations(list);
        unsupported += bad;
        list
    });
    let decorated = style.decoration_line.is_some()
        || style.decoration_color.is_some()
        || style.decoration_thickness.is_some()
        || style.underline_offset.is_some();
    let props = p::SpanProps {
        font_size: style.font_size,
        weight: style.weight,
        color: style.color.map(|c| c.resolve(theme)),
        family: style.family.clone(),
        italic: style.italic,
        letter_spacing: style.letter_spacing.map(TextLength::to_wire),
        word_spacing: style.word_spacing.map(TextLength::to_wire),
        kerning: style.kerning,
        features,
        decoration: decorated.then(|| {
            decoration(
                style.decoration_line,
                style.decoration_color,
                style.decoration_thickness,
                style.underline_offset,
                theme,
            )
        }),
        stroke: style.stroke.map(|(width, color)| p::TextStroke {
            width: width.to_wire(),
            color: color.resolve(theme),
        }),
        variations,
        font_width: style.font_width,
        background: style.background.as_ref().map(|paint| match paint.solid() {
            Some(c) => p::Fill::Solid(c.resolve(theme)),
            None => p::Fill::Layers(vec![fill_layer(paint, theme)]),
        }),
    };
    CompiledSpanStyle { props, unsupported }
}

/// Resolve an editing range's decoration using the current theme.
pub fn compile_text_decoration(value: crate::TextDecoration, theme: &Theme) -> p::TextDecoration {
    p::TextDecoration {
        lines: value.lines,
        color: value.color.map(|color| color.resolve(theme)),
        thickness: value.thickness.map(TextLength::to_wire),
        underline_offset: value.underline_offset.to_wire(),
        style: value.style,
    }
}
