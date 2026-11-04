//! Role: Text styles and runs of the painter, and the `Draw` text commands.

use super::recorder::Draw;
use crate::color::Color;
use crate::geometry::Point;
use crate::text::{FontFeature, FontKerning, LineHeight, TextDecorationLine, TextLength};
use crate::theme::value::Theme;
use kivact_types as p;
use kivact_types::{DrawCmd, TextMetrics};

/// Text style for `Draw::text` and `Draw::measure_text`: the text
/// vocabulary of element styles, so canvas text matches element text.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::draw::TextFormat;
/// use kivact::style::{TextDecorationLine, em};
/// let label = TextFormat {
///     size: 11.0,
///     weight: 600,
///     letter_spacing: em(0.08),
///     tabular_nums: true,
///     decoration: TextDecorationLine::UNDERLINE,
///     ..TextFormat::default()
/// };
/// # let _ = label;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct TextFormat {
    /// Horizontal placement relative to the anchor. `Justify` places like
    /// `Start`: a line drawn alone is the last of its paragraph.
    pub align: crate::TextAlign,
    /// Vertical placement relative to the anchor.
    pub baseline: TextBaseline,
    pub size: f64,
    pub weight: u16,
    pub color: Color,
    /// A family name or a generic family (`"system-ui"`, `"sans-serif"`,
    /// `"serif"`, `"monospace"`), or an ordered comma-separated CSS list;
    /// `None` (the default) uses the family the
    /// node inherits (`style().font_family(..)`, else the theme's).
    pub family: Option<String>,
    /// Use the italic face of the family when it has one (default `false`).
    pub italic: bool,
    /// The distance between lines (CSS `line-height`; a number is a
    /// multiple of `size`, `LineHeight::px(20)` is px). `None` (the
    /// default) uses the line height the node inherits
    /// (`style().line_height(..)`, else `Theme.text.line_height`), as
    /// text elements do.
    pub line_height: Option<LineHeight>,
    /// Extra space after every character (CSS `letter-spacing`; px or
    /// `em(..)` of `size`).
    pub letter_spacing: TextLength,
    /// Extra space after every space (CSS `word-spacing`; px or `em(..)`
    /// of `size`).
    pub word_spacing: TextLength,
    /// Pair kerning (CSS `font-kerning`).
    pub font_kerning: FontKerning,
    /// OpenType features (CSS `font-feature-settings`):
    /// `vec![("ss01", 1).into()]`.
    pub features: Vec<FontFeature>,
    /// Digits of one width (CSS `font-variant-numeric: tabular-nums`).
    pub tabular_nums: bool,
    /// Lines drawn with the text (CSS `text-decoration-line`).
    pub decoration: TextDecorationLine,
    /// Their color (default: `color`).
    pub decoration_color: Option<Color>,
}

impl Default for TextFormat {
    fn default() -> Self {
        TextFormat {
            align: crate::TextAlign::Start,
            baseline: TextBaseline::Top,
            size: 14.0,
            weight: 400,
            color: Color::token(crate::ColorToken::Text),
            family: None,
            italic: false,
            line_height: None,
            letter_spacing: TextLength::default(),
            word_spacing: TextLength::default(),
            font_kerning: FontKerning::Auto,
            features: Vec::new(),
            tabular_nums: false,
            decoration: TextDecorationLine::NONE,
            decoration_color: None,
        }
    }
}

pub fn text_run(
    text: &str,
    origin: p::Point,
    style: &TextFormat,
    scale: f64,
    color: p::Rgba,
    theme: Option<&Theme>,
) -> p::TextRun {
    let size = style.size * scale;
    let (features, _) = crate::text::features(Some(&style.features), style.tabular_nums);
    p::TextRun {
        text: text.to_string(),
        origin,
        font_size: size,
        weight: style.weight,
        color,
        family: style.family.clone(),
        italic: style.italic,
        line_height: style.line_height,
        letter_spacing: style.letter_spacing.to_wire().px(style.size) * scale,
        word_spacing: style.word_spacing.to_wire().px(style.size) * scale,
        kerning: style.font_kerning,
        features,
        decoration: p::TextDecoration {
            lines: style.decoration,
            // Measuring needs no color (and no theme).
            color: theme.and_then(|t| style.decoration_color.map(|c| c.resolve(t))),
            ..p::TextDecoration::default()
        },
    }
}

/// A canvas text anchor's vertical alignment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextBaseline {
    #[default]
    Top,
    Middle,
    Alphabetic,
    Bottom,
}

impl Draw {
    pub fn text(&mut self, text: &str, pos: Point, style: TextFormat) {
        let mut pos = pos;
        if style.align != crate::TextAlign::Start || style.baseline != TextBaseline::Top {
            let m = self.measure_text(text, &style);
            pos.x -= match style.align {
                crate::TextAlign::Center => m.size.width / 2.0,
                crate::TextAlign::End => m.size.width,
                // A line drawn alone is the last of its paragraph.
                crate::TextAlign::Start | crate::TextAlign::Justify => 0.0,
            };
            pos.y -= match style.baseline {
                TextBaseline::Top => 0.0,
                TextBaseline::Middle => m.size.height / 2.0,
                TextBaseline::Alphabetic => m.baseline,
                TextBaseline::Bottom => m.size.height,
            };
        }
        let run = text_run(
            text,
            pos,
            &style,
            1.0,
            self.color(style.color),
            Some(&self.theme),
        );
        self.cmds.push(DrawCmd::Text(run));
    }

    /// The extent `text(text, pos, style)` draws `text` with, before the
    /// layer transforms (in this painter's coordinates): size a label's
    /// pill, center a label (`(size.height - m.size.height) / 2`), place a
    /// baseline. The same shaping as the drawing, so they agree; zero
    /// where there is no engine to measure with.
    ///
    /// ```
    /// use kivact::prelude::*;
    /// use kivact::draw::{Draw, Point, Rect};
    /// use kivact::draw::TextFormat;
    /// // A pill as wide as its label plus 12 px.
    /// fn pill(d: &mut Draw, label: &str, at: Point) {
    ///     let style = TextFormat { size: 11.0, ..TextFormat::default() };
    ///     let m = d.measure_text(label, &style);
    ///     d.rounded_rect(Rect::new(at.x, at.y, m.size.width + 12.0, m.size.height + 4.0), 4.0, ColorToken::Accent);
    ///     d.text(label, Point::new(at.x + 6.0, at.y + 2.0), style);
    /// }
    /// # let _ = pill;
    /// ```
    pub fn measure_text(&self, text: &str, style: &TextFormat) -> TextMetrics {
        let Some(m) = &self.measurer else {
            return TextMetrics::default();
        };
        let origin = p::Point { x: 0.0, y: 0.0 };
        let run = text_run(text, origin, style, 1.0, p::Rgba::TRANSPARENT, None);
        m.measure(&run)
    }
}
