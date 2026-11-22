//! Role: Text styles and runs of the painter, and the `Draw` text commands.

use super::recorder::Draw;
use crate::color_value::Color;
use crate::geometry::Point;
use crate::length::Resolve;
use crate::paint::{Paint, fill_layer};
use crate::path::Path;
use crate::text::{
    FontFeature, FontKerning, FontVariation, LineHeight, TextDecorationLine, wire_variations,
};
use crate::theme::value::Theme;
use crate::transform::Transform;
use kivact_types as p;
use kivact_types::{DrawCmd, Fill, Length, Stroke, TextMetrics};

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
    pub letter_spacing: Length,
    /// Extra space after every space (CSS `word-spacing`; px or `em(..)`
    /// of `size`).
    pub word_spacing: Length,
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
    /// The glyph fill: any paint (a gradient too); None fills with `color`.
    pub fill: Option<Paint>,
    /// The glyph outline, its stroke and paint (drawn with the fill; the
    /// order is the `paint-order` of the text: fill first by default).
    pub stroke: Option<(Stroke, Paint)>,
    /// OpenType variation settings (CSS `font-variation-settings`).
    pub variations: Vec<FontVariation>,
    /// The font width in percent (CSS `font-stretch`; 100 is normal).
    pub font_width: f64,
}

impl Default for TextFormat {
    fn default() -> Self {
        TextFormat {
            align: crate::TextAlign::Start,
            baseline: TextBaseline::Top,
            size: 14.0,
            weight: 400,
            color: crate::color::TEXT,
            family: None,
            italic: false,
            line_height: None,
            letter_spacing: Length::Px(0.0),
            word_spacing: Length::Px(0.0),
            font_kerning: FontKerning::Auto,
            features: Vec::new(),
            tabular_nums: false,
            decoration: TextDecorationLine::NONE,
            decoration_color: None,
            fill: None,
            stroke: None,
            variations: Vec::new(),
            font_width: 100.0,
        }
    }
}

/// The engine's fill of `paint` in `theme`; a length the paint holds that
/// the theme cannot take is counted in `dropped`.
pub(crate) fn fill_for(paint: &Paint, theme: &Theme, dropped: &mut u32) -> Fill {
    match paint.solid() {
        Some(c) => Fill::Solid(c.resolve(theme)),
        None => {
            let mut cx = Resolve::of_theme(theme, dropped);
            Fill::Layers(vec![fill_layer(paint, &mut cx)])
        }
    }
}

/// The px of a spacing `length` at `size`, its theme variables read in
/// `theme`; an invalid one is counted in `unsupported`.
fn spacing_px(length: Length, size: f64, theme: &Theme, unsupported: &mut u32) -> f64 {
    crate::text::text_length(length, theme, unsupported).map_or(0.0, |l| l.px(size))
}

/// The text run of `text`, its lengths resolved in `theme` (the theme the
/// text is drawn or measured in, so both agree). An invalid length counts
/// in `unsupported`.
pub fn text_run(
    text: &str,
    origin: p::Point,
    style: &TextFormat,
    scale: f64,
    color: p::Rgba,
    theme: &Theme,
    unsupported: &mut u32,
) -> p::TextRun {
    let size = style.size * scale;
    let (features, _) = crate::text::features(Some(&style.features), style.tabular_nums);
    let (variations, _) = wire_variations(&style.variations);
    p::TextRun {
        text: text.to_string(),
        origin,
        font_size: size,
        weight: style.weight,
        color,
        fill: style.fill.as_ref().map(|f| fill_for(f, theme, unsupported)),
        stroke: style
            .stroke
            .as_ref()
            .map(|(stroke, paint)| (stroke.clone(), fill_for(paint, theme, unsupported))),
        variations,
        font_width: style.font_width,
        family: style.family.clone(),
        italic: style.italic,
        line_height: style.line_height,
        letter_spacing: spacing_px(style.letter_spacing, style.size, theme, unsupported) * scale,
        word_spacing: spacing_px(style.word_spacing, style.size, theme, unsupported) * scale,
        kerning: style.font_kerning,
        features,
        decoration: p::TextDecoration {
            lines: style.decoration,
            color: style.decoration_color.map(|c| c.resolve(theme)),
            ..p::TextDecoration::default()
        },
    }
}

/// The glyph outlines of every run of `shaped`, in the shaped text's own
/// coordinates (the outlines of a run are in its glyph positions' space).
pub(crate) fn outline_runs(
    shaper: &dyn p::TextShaper,
    shaped: &p::ShapedTextSnapshot,
) -> Vec<p::PathCmd> {
    shaped
        .as_shared()
        .runs
        .iter()
        .flat_map(|run| shaper.outline(run.as_shared()))
        .collect()
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
        let pos = self.anchored(text, pos, &style);
        let run = text_run(
            text,
            pos,
            &style,
            1.0,
            self.color(style.color),
            &self.theme,
            &mut self.unsupported,
        );
        self.cmds.push(DrawCmd::Text(run));
    }

    /// Where `text` is drawn for its anchor `pos` and its alignment.
    fn anchored(&self, text: &str, pos: Point, style: &TextFormat) -> Point {
        let mut pos = pos;
        if style.align != crate::TextAlign::Start || style.baseline != TextBaseline::Top {
            let m = self.measure_text(text, style);
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
        pos
    }

    /// The outline of `text` drawn at `pos` with `style` as a path (Canvas
    /// has none; SVG does), placed as `text` places it.
    pub fn text_path(&mut self, text: &str, pos: Point, style: &TextFormat) -> Path {
        let origin = self.anchored(text, pos, style);
        let run = text_run(
            text,
            p::Point { x: 0.0, y: 0.0 },
            style,
            1.0,
            p::Rgba::TRANSPARENT,
            &self.theme,
            &mut self.unsupported,
        );
        let m = &self.measurer;
        let cmds = match m.shape(&run, None) {
            Some(shaped) => outline_runs(m.as_ref(), &shaped),
            None => Vec::new(),
        };
        Path::from_cmds(cmds).transformed(Transform::new().translate(origin.x, origin.y))
    }

    /// The extent `text(text, pos, style)` draws `text` with, before the
    /// layer transforms (in this painter's coordinates): size a label's
    /// pill, center a label (`(size.height - m.size.height) / 2`), place a
    /// baseline. The same shaping as the drawing, so they agree; zero
    /// where there is no engine to measure with.
    ///
    /// ```
    /// use kivact::prelude::*;
    /// use kivact::style::Color;
    /// use kivact::draw::{Draw};
    /// use kivact::geometry::{Point, Rect};
    /// use kivact::draw::TextFormat;
    /// // A pill as wide as its label plus 12 px.
    /// fn pill(d: &mut Draw, label: &str, at: Point) {
    ///     let style = TextFormat { size: 11.0, ..TextFormat::default() };
    ///     let m = d.measure_text(label, &style);
    ///     d.rounded_rect(Rect::new(at.x, at.y, m.size.width + 12.0, m.size.height + 4.0), 4.0, kivact::style::color::ACCENT);
    ///     d.text(label, Point::new(at.x + 6.0, at.y + 2.0), style);
    /// }
    /// # let _ = pill;
    /// ```
    pub fn measure_text(&self, text: &str, style: &TextFormat) -> TextMetrics {
        let origin = p::Point { x: 0.0, y: 0.0 };
        let run = text_run(
            text,
            origin,
            style,
            1.0,
            p::Rgba::TRANSPARENT,
            &self.theme,
            &mut 0,
        );
        self.measurer.measure(&run)
    }
}
