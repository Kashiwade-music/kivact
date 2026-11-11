//! Role: The `Style` setters for text and the rarer text properties.

use super::fields::Style;
use crate::color::Color;
use crate::effect::Shadow;
use crate::text::{
    FontFeature, FontKerning, FontVariation, PaintOrder, TextDecorationLine, TextLength,
    TextOverflow, TextTransform,
};
use crate::theme::value::ThemeColor;

macro_rules! text_color_setter {
    ($(#[$m:meta])* $name:ident, $field:ident) => {
        $(#[$m])*
        pub fn $name(mut self, v: impl ThemeColor) -> Self {
            let set = self
                .text
                .as_deref()
                .and_then(|text| text.text.as_deref())
                .is_some_and(|x| x.$field.is_some());
            if set {
                self.duplicates += 1;
            }
            self.text_mut().text.get_or_insert_with(Default::default).$field = Some(Color::of(v));
            self
        }
    };
}

macro_rules! text_setter {
    ($(#[$m:meta])* $name:ident, $field:ident, $t:ty, $conv:expr) => {
        $(#[$m])*
        pub fn $name(mut self, v: $t) -> Self {
            let set = self
                .text
                .as_deref()
                .and_then(|text| text.text.as_deref())
                .is_some_and(|x| x.$field.is_some());
            if set {
                self.duplicates += 1;
            }
            #[allow(clippy::redundant_closure_call, reason = "`$conv` may be a closure literal called in place")]
            {
                self.text_mut().text.get_or_insert_with(Default::default).$field = Some(($conv)(v.into()));
            }
            self
        }
    };
}

impl Style {
    text_setter!(
        /// CSS `letter-spacing`: extra space after every character, in px
        /// (`letter_spacing(1)`) or em (`letter_spacing(em(0.08))`, of the
        /// text's font size; negative tightens).
        letter_spacing, letter_spacing, impl Into<TextLength>, |v: TextLength| v);
    text_setter!(
        /// CSS `word-spacing`: extra space after every space, in px
        /// (`word_spacing(4)`) or em (`word_spacing(em(0.25))`, of the
        /// text's font size; negative tightens). It adds to
        /// `text_align(TextAlign::Justify)`'s widening.
        word_spacing, word_spacing, impl Into<TextLength>, |v: TextLength| v);
    text_setter!(
        /// CSS `font-kerning`: `FontKerning::None` turns the OpenType `kern`
        /// feature off, `Normal` on, `Auto` (the default) leaves it to the
        /// font. A `("kern", ..)` in `font_feature_settings` wins.
        font_kerning, kerning, FontKerning, |v| v);
    text_setter!(
        /// CSS `text-decoration-line` (and the `text-decoration: underline`
        /// shorthand): `text_decoration(TextDecorationLine::UNDERLINE)`.
        /// The lines are drawn in the text's color unless
        /// `text_decoration_color` says otherwise; a span of rich text can
        /// set its own.
        text_decoration, decoration_line, TextDecorationLine, |v| v);
    text_color_setter!(
        /// CSS `text-decoration-color` (default: the text's color).
        text_decoration_color, decoration_color);
    text_setter!(
        /// CSS `text-decoration-thickness` in px or em (default: the font's).
        text_decoration_thickness,
        decoration_thickness,
        impl Into<TextLength>,
        |v: TextLength| v
    );
    text_setter!(
        /// CSS `text-underline-offset`: the underline this much further
        /// from the text than the font puts it (`text_underline_offset(3)`).
        text_underline_offset, underline_offset, impl Into<TextLength>, |v: TextLength| v);
    text_setter!(
        /// CSS `text-overflow`: `TextOverflow::Ellipsis` cuts a line that
        /// does not fit its box and ends it in "…" (with
        /// `text_wrap(false)`, CSS's `white-space: nowrap`; such a text can
        /// shrink down to the ellipsis in a row). The accessible name is
        /// the whole text.
        text_overflow, overflow, TextOverflow, |v| v);
    text_setter!(
        /// CSS `line-clamp`: show at most `n` lines, the last ending in
        /// "…" when more follow (the box is as high as the lines shown).
        line_clamp, line_clamp, u32, |v| v);
    text_setter!(
        /// Trim the first line to the measured cap height and the last to its baseline.
        text_capsize, capsize, bool, |v| v);
    text_setter!(
        /// CSS `text-transform`: draw the text in upper case, lower case
        /// or capitalized; its accessible name keeps it as written.
        text_transform, transform, TextTransform, |v| v);
    /// CSS `-webkit-text-stroke`: the outline of the glyphs, in px or em, in
    /// `color`, drawn over or under the fill as `paint_order` says.
    pub fn text_stroke(mut self, width: impl Into<TextLength>, color: impl ThemeColor) -> Self {
        let set = self
            .text
            .as_deref()
            .and_then(|text| text.text.as_deref())
            .is_some_and(|x| x.stroke.is_some());
        if set {
            self.duplicates += 1;
        }
        self.text_mut()
            .text
            .get_or_insert_with(Default::default)
            .stroke = Some((width.into(), Color::of(color)));
        self
    }

    text_setter!(
        /// CSS `paint-order`: whether the fill or the stroke of the glyphs is drawn first.
        paint_order, paint_order, PaintOrder, |v| v);

    /// CSS `font-variation-settings`: axis settings such as `wght`.
    pub fn font_variation_settings(
        mut self,
        settings: impl IntoIterator<Item = impl Into<FontVariation>>,
    ) -> Self {
        let set = self
            .text
            .as_deref()
            .and_then(|text| text.text.as_deref())
            .is_some_and(|x| x.variations.is_some());
        if set {
            self.duplicates += 1;
        }
        self.text_mut()
            .text
            .get_or_insert_with(Default::default)
            .variations = Some(settings.into_iter().map(Into::into).collect());
        self
    }

    text_setter!(
        /// CSS `font-optical-sizing`: whether the optical size axis follows the font size.
        font_optical_sizing, optical_sizing, bool, |v| v);

    text_setter!(
        /// CSS `font-stretch` as a percentage of the normal width (50 to 200).
        font_width, font_width, impl Into<f64>, |v: f64| v);

    text_color_setter!(
        /// CSS `caret-color` of a text field.
        caret_color, caret_color);
    text_color_setter!(
        /// The band behind a text field's selected text (CSS
        /// `::selection { background }`).
        selection_color, selection_color);
    text_color_setter!(
        /// A text field's placeholder color (CSS `::placeholder { color }`;
        /// default: the text color at half its alpha).
        placeholder_color, placeholder_color);

    /// CSS `font-feature-settings`: OpenType features of the font,
    /// `font_feature_settings([("tnum", 1), ("ss01", 1)])` (a tag alone,
    /// `"cv11"`, is on). The font must have them; others do nothing.
    pub fn font_feature_settings(
        mut self,
        features: impl IntoIterator<Item = impl Into<FontFeature>>,
    ) -> Self {
        let set = self
            .text
            .as_deref()
            .and_then(|text| text.text.as_deref())
            .is_some_and(|x| x.features.is_some());
        if set {
            self.duplicates += 1;
        }
        self.text_mut()
            .text
            .get_or_insert_with(Default::default)
            .features = Some(features.into_iter().map(Into::into).collect());
        self
    }

    /// CSS `font-variant-numeric: tabular-nums`: digits of one width (the
    /// font's `tnum` feature), so numbers in a column or a timer do not
    /// jiggle.
    pub fn tabular_nums(mut self) -> Self {
        let set = self
            .text
            .as_deref()
            .and_then(|text| text.text.as_deref())
            .is_some_and(|x| x.tabular_nums.is_some());
        if set {
            self.duplicates += 1;
        }
        self.text_mut()
            .text
            .get_or_insert_with(Default::default)
            .tabular_nums = Some(true);
        self
    }

    /// Add a text shadow (CSS `text-shadow: x y blur color`); several
    /// stack, the first on top. They shadow the glyphs and decoration
    /// lines (a shadow's spread and inset do not apply to text and are
    /// reported as unsupported).
    pub fn text_shadow(mut self, s: Shadow) -> Self {
        let x = self.text_mut().text.get_or_insert_with(Default::default);
        x.shadows.get_or_insert_with(Vec::new).push(s);
        self
    }
}
