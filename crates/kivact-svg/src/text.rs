//! Role: Writes one line of drawn text as an SVG `<text>` element.
//!
//! The run's origin is the top of its line box; the SVG baseline is that
//! origin plus the baseline the host's text engine measures for the run, so
//! the text sits where a window draws it. Fill and stroke paints, font
//! stretch, kerning, features, variations and decorations are written as SVG
//! presentation attributes and CSS properties.

use super::emit::stroke_attributes;
use super::emitter::Emitter;
use super::encode::{css_color, escape, hex, number, opacity_attribute};
use super::fill::{Units, paint_attributes};
use crate::SvgError;
use kivact_types::{
    DecorationLines, FontKerning, Rect, TextDecoration, TextDecorationStyle, TextRun,
};

/// The generic families SVG names without quotes.
const GENERIC_FAMILIES: [&str; 4] = ["serif", "sans-serif", "monospace", "cursive"];

/// The box that the gradients of a run are drawn over: the unit square, since
/// the writer does not know the width of the run.
const UNIT_BOX: Rect = Rect {
    x: 0.0,
    y: 0.0,
    width: 1.0,
    height: 1.0,
};

/// The CSS name of each line a decoration can draw.
const LINE_NAMES: [(DecorationLines, &str); 3] = [
    (DecorationLines::UNDERLINE, "underline"),
    (DecorationLines::OVERLINE, "overline"),
    (DecorationLines::LINE_THROUGH, "line-through"),
];

impl Emitter<'_> {
    pub(super) fn text(&mut self, run: &TextRun) -> Result<(), SvgError> {
        let baseline = self.resources.baseline(run);
        let family = run
            .family
            .clone()
            .or_else(|| self.resources.fallback_family().map(str::to_owned))
            .unwrap_or_else(|| "sans-serif".to_owned());
        let mut attributes = format!(
            "x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\"",
            number(run.origin.x),
            number(run.origin.y + baseline),
            family_list(&family),
            number(run.font_size),
            run.weight
        );
        if run.italic {
            attributes.push_str(" font-style=\"italic\"");
        }
        let fill = match &run.fill {
            None => format!(
                "fill=\"{}\"{}",
                hex(run.color),
                opacity_attribute("fill", run.color)
            ),
            Some(fill) => paint_attributes(self, "fill", fill, UNIT_BOX, Units::ObjectBox)?,
        };
        attributes.push_str(&format!(" {fill}"));
        if let Some((stroke, paint)) = &run.stroke {
            let paint = paint_attributes(self, "stroke", paint, UNIT_BOX, Units::ObjectBox)?;
            attributes.push_str(&format!(" {paint} {}", stroke_attributes(stroke)));
        }
        if run.font_width != 100.0 {
            attributes.push_str(&format!(" font-stretch=\"{}%\"", number(run.font_width)));
        }
        if run.letter_spacing != 0.0 {
            attributes.push_str(&format!(
                " letter-spacing=\"{}\"",
                number(run.letter_spacing)
            ));
        }
        if run.word_spacing != 0.0 {
            attributes.push_str(&format!(" word-spacing=\"{}\"", number(run.word_spacing)));
        }
        let css = text_css(run);
        if !css.is_empty() {
            attributes.push_str(&format!(" style=\"{css}\""));
        }
        self.body.push_str(&format!(
            "<text {attributes} xml:space=\"preserve\">{}</text>\n",
            escape(&run.text)
        ));
        Ok(())
    }
}

/// The `font-family` value of a family name: a generic family as is, a
/// name with a quoted-string form otherwise, with `system-ui` as sans-serif.
fn family_list(family: &str) -> String {
    let family = family.trim();
    if family == "system-ui" || GENERIC_FAMILIES.contains(&family) {
        let generic = if family == "system-ui" {
            "sans-serif"
        } else {
            family
        };
        return escape(generic);
    }
    format!("'{}', sans-serif", escape(family))
}

/// The CSS declarations of a run's kerning, features, variations and
/// decorations, joined by `;` (empty when none applies).
fn text_css(run: &TextRun) -> String {
    let mut declarations = Vec::new();
    match run.kerning {
        FontKerning::Normal => declarations.push("font-kerning:normal".to_owned()),
        FontKerning::None => declarations.push("font-kerning:none".to_owned()),
        FontKerning::Auto => {}
    }
    if !run.features.is_empty() {
        let features: Vec<String> = run
            .features
            .iter()
            .map(|feature| format!("{} {}", quoted_tag(feature.tag), feature.value))
            .collect();
        declarations.push(format!("font-feature-settings:{}", features.join(", ")));
    }
    if !run.variations.is_empty() {
        let axes: Vec<String> = run
            .variations
            .iter()
            .map(|axis| format!("{} {}", quoted_tag(axis.tag), number(axis.value)))
            .collect();
        declarations.push(format!("font-variation-settings:{}", axes.join(", ")));
    }
    declarations.extend(decoration_declarations(&run.decoration, run.font_size));
    declarations.join(";")
}

/// The CSS declarations of the lines drawn with a run (none when no line is
/// drawn). An `em` length is of `font_size`.
fn decoration_declarations(decoration: &TextDecoration, font_size: f64) -> Vec<String> {
    if decoration.lines.is_empty() {
        return Vec::new();
    }
    let names: Vec<&str> = LINE_NAMES
        .iter()
        .filter(|(line, _)| decoration.lines.contains(*line))
        .map(|(_, name)| *name)
        .collect();
    let mut declarations = vec![
        format!("text-decoration-line:{}", names.join(" ")),
        format!("text-decoration-style:{}", style_name(decoration.style)),
    ];
    if let Some(color) = decoration.color {
        declarations.push(format!("text-decoration-color:{}", css_color(color)));
    }
    if let Some(thickness) = decoration.thickness {
        declarations.push(format!(
            "text-decoration-thickness:{}px",
            number(thickness.px(font_size))
        ));
    }
    let offset = decoration.underline_offset.px(font_size);
    if offset != 0.0 {
        declarations.push(format!("text-underline-offset:{}px", number(offset)));
    }
    declarations
}

/// The CSS name of a decoration's line pattern.
fn style_name(style: TextDecorationStyle) -> &'static str {
    match style {
        TextDecorationStyle::Solid => "solid",
        TextDecorationStyle::Dashed => "dashed",
        TextDecorationStyle::Dotted => "dotted",
        TextDecorationStyle::Wavy => "wavy",
    }
}

/// An OpenType tag in CSS quotes, such as `'tnum'`.
fn quoted_tag(tag: [u8; 4]) -> String {
    format!("'{}'", escape(&String::from_utf8_lossy(&tag)))
}
