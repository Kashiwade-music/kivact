//! Role: Styled spans for rich text paragraphs.
//!
//! Rich text: one paragraph of [`Span`]s, each with its own text style
//! and an optional background band (`<b>`, `<small>`, `<mark>`), built
//! with [`span`] and shown by [`rich_text`](crate::view::rich_text).
//!
//! Invariant: a span sets only what it names; the rest is the
//! paragraph's (the text node's inherited text style). A span's
//! decoration lines and features replace the paragraph's.

use kivact_types as p;

use crate::style::Paint;
use kivact_style::text::{
    FontFeature, FontKerning, FontVariation, TextDecorationLine, TextDecorationStyle,
};
use kivact_style::{Color, FontStyle, Length, Style, Theme};

/// A piece of rich text with its own style. Built by [`span`]; a `&str`
/// or `String` is a span in the paragraph's style.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::view::{rich_text, span};
/// use kivact::view::{Element};
/// use kivact::style::{rgba};
/// // <p>Open <mark>fi</mark>le</p>, the match bold and highlighted
/// let line = rich_text(vec![
///     span("Open "),
///     span("fi").font_weight(700).background(rgba(0xffd400, 0.4)),
///     span("le"),
/// ]);
/// # let _ = line;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub(crate) text: kivact_runtime::Prop<String>,
    style: Style,
}

/// A span of rich text showing `text`, in the paragraph's style until its
/// methods say otherwise.
pub fn span(text: impl kivact_runtime::IntoText) -> Span {
    Span {
        text: kivact_runtime::Prop::new(text.into_text()),
        style: Style::default(),
    }
}

impl From<&str> for Span {
    fn from(text: &str) -> Self {
        span(text)
    }
}

impl From<String> for Span {
    fn from(text: String) -> Self {
        span(text)
    }
}

impl Span {
    /// CSS `font-size`.
    pub fn font_size(mut self, v: impl Into<f64>) -> Self {
        let v: f64 = v.into();
        self.style = self.style.font_size(v);
        self
    }

    /// CSS `font-weight` (`700`: `<b>`; any value from 1 to 1000).
    pub fn font_weight(mut self, v: u16) -> Self {
        self.style = self.style.font_weight(v);
        self
    }

    /// CSS `color` of its glyphs.
    pub fn color(mut self, v: impl Into<Color>) -> Self {
        self.style = self.style.color(v);
        self
    }

    /// CSS `font-family`: a name, generic family, or ordered comma-separated list.
    pub fn font_family(mut self, v: impl Into<String>) -> Self {
        self.style = self.style.font_family(v);
        self
    }

    /// CSS `font-style`: `FontStyle::Italic` (`<em>`, `<i>`) or `FontStyle::Normal`.
    pub fn font_style(mut self, v: FontStyle) -> Self {
        self.style = self.style.font_style(v);
        self
    }

    /// CSS `letter-spacing` (px or `em(..)` of its font size).
    pub fn letter_spacing(mut self, v: impl Into<Length>) -> Self {
        self.style = self.style.letter_spacing(v);
        self
    }

    /// CSS `word-spacing` (px or `em(..)` of its font size).
    pub fn word_spacing(mut self, v: impl Into<Length>) -> Self {
        self.style = self.style.word_spacing(v);
        self
    }

    /// CSS `font-kerning`.
    pub fn font_kerning(mut self, v: FontKerning) -> Self {
        self.style = self.style.font_kerning(v);
        self
    }

    /// CSS `font-feature-settings` (replacing the paragraph's).
    pub fn font_feature_settings(
        mut self,
        features: impl IntoIterator<Item = impl Into<FontFeature>>,
    ) -> Self {
        self.style = self.style.font_feature_settings(features);
        self
    }

    /// CSS `font-variation-settings` (replacing the paragraph's).
    pub fn font_variation_settings(
        mut self,
        settings: impl IntoIterator<Item = impl Into<FontVariation>>,
    ) -> Self {
        self.style = self.style.font_variation_settings(settings);
        self
    }

    /// CSS `-webkit-text-stroke` (replacing the paragraph's): the outline's
    /// width and color.
    pub fn text_stroke(mut self, width: impl Into<Length>, color: impl Into<Color>) -> Self {
        self.style = self.style.text_stroke(width, color);
        self
    }

    /// CSS `font-stretch` (font-width) in percent, 50 to 200.
    pub fn font_width(mut self, v: impl Into<f64>) -> Self {
        let v: f64 = v.into();
        self.style = self.style.font_width(v);
        self
    }

    /// CSS `font-variant-numeric: tabular-nums`.
    pub fn tabular_nums(mut self) -> Self {
        self.style = self.style.tabular_nums();
        self
    }

    /// CSS `text-decoration-line` (replacing the paragraph's lines).
    pub fn text_decoration(mut self, v: TextDecorationLine) -> Self {
        self.style = self.style.text_decoration(v);
        self
    }

    /// CSS `text-decoration-style`: the line's pattern (solid, dashed, dotted or wavy).
    pub fn text_decoration_style(mut self, v: TextDecorationStyle) -> Self {
        self.style = self.style.text_decoration_style(v);
        self
    }

    /// CSS `text-decoration-color` (default: its text color).
    pub fn text_decoration_color(mut self, v: impl Into<Color>) -> Self {
        self.style = self.style.text_decoration_color(v);
        self
    }

    /// CSS `text-decoration-thickness`.
    pub fn text_decoration_thickness(mut self, v: impl Into<Length>) -> Self {
        self.style = self.style.text_decoration_thickness(v);
        self
    }

    /// CSS `text-underline-offset`.
    pub fn text_underline_offset(mut self, v: impl Into<Length>) -> Self {
        self.style = self.style.text_underline_offset(v);
        self
    }

    /// CSS `background` of an inline element: a band behind its glyphs on
    /// each line it is on (its advance by its font's ascent and descent),
    /// a color or a gradient (`<mark>`, a marker pen:
    /// `linear_gradient(180).stop(Color::TRANSPARENT, pct(62)).stop(c, pct(62))`).
    pub fn background(mut self, v: impl Into<Paint>) -> Self {
        let v: Paint = v.into();
        self.style = self.style.background(v);
        self
    }
}

/// The text of `spans` joined and their spans as the engine takes them
/// (colors and paints resolved against `theme`), and how many values were
/// dropped (feature tags that are not four ASCII characters).
pub(crate) fn resolve(spans: &[Span], theme: &Theme) -> (String, Vec<p::TextSpan>, u32) {
    let mut text = String::with_capacity(spans.iter().map(|s| s.text.get().len()).sum());
    let mut out = Vec::with_capacity(spans.len());
    let mut dropped = 0;
    for span in spans {
        let start = text.len();
        text.push_str(&span.text.get());
        let (style, background, unsupported) =
            kivact_style::internal::compile_span(&span.style, theme);
        dropped += unsupported;
        if style == p::TextStyle::default() && background.is_none() {
            continue;
        }
        out.push(p::TextSpan {
            range: start..text.len(),
            style,
            background,
        });
    }
    (text, out, dropped)
}
