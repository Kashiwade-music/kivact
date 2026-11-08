//! Role: Text values: spacing, kerning, features, decoration, and their engine forms.
//!
//! The text vocabulary of `Style` (and of `Draw::text` and rich text
//! spans): letter and word spacing with `em` lengths, kerning, decoration lines, OpenType
//! features, overflow with an ellipsis, line clamps, case conversion, text
//! shadows and a text input's caret, selection and placeholder colors.
//! CSS's property names in snake case, typed values, resolved here against
//! the theme into the engine's text props.
//!
//! Invariant: like the other text properties, each is inherited on its
//! own (CSS's inheritance, property by property): the text of a node takes
//! each property from the nearest ancestor-or-self that sets it, and the
//! root holds the theme's size, weight, color, family and line height. A
//! node that sets only `weight` keeps its parent's color and letter
//! spacing. Unlike CSS, the decoration, `text_overflow` and `line_clamp`
//! are inherited too: set on a container, they reach the texts in it.
//!
//! Rules of the text style:
//! - `line_height`: a number is always a multiple of the font size, px is
//!   `LineHeight::px`, `Normal` is the font size times 1.3 rounded to px.
//!   Unset, the root's text props hold `Theme.text.line_height` (default
//!   `Scale(1.35)`) and a text inherits the nearest ancestor's value (like
//!   the family); an explicit style wins, and an explicit `Normal` stays
//!   `Normal` (it inherits nothing). A `provide_theme` subtree does not change
//!   the root's value. The value is checked again at the host boundary
//!   (not finite or not above zero is `Normal`).
//! - `letter_spacing` and `word_spacing` are [`TextLength`]s: a number is
//!   px, `em` a multiple of the font size; word spacing is added at every
//!   space.
//! - `font_kerning`: `Auto` is the font's default, `Normal` and `None` put
//!   `kern` 1 or 0 first among the features, and an explicit `kern` in
//!   `font_feature_settings` wins. `tabular_nums` adds `tnum` after the
//!   features. A [`FontFeature`] tag other than four ASCII characters is
//!   dropped and reported once per node as `Unsupported`; the engine's wire
//!   takes the shared value type once it is checked.
//! - `text_align(Justify)` widens the spaces of every line but the last
//!   (Parley's `Alignment::Justify`); the last line, and a single line
//!   without a width, are `Start`. `TextLine::rect` returns the widened
//!   width.
//! - Decoration lines are a bit set (`UNDERLINE`, ...) whose color
//!   defaults to the current color, with a thickness and an underline
//!   offset; `weight` is 1 to 1000 and goes straight to a variable font's
//!   `wght`. Ellipsis, `line_clamp`, `TextTransform` and `text_shadow`
//!   exist; a text shadow lists the topmost first, and its spread and inset
//!   do not apply to text (`Unsupported`). An editor can set its caret,
//!   selection background and placeholder colors.
//! - Case conversion, the ellipsis and the clamp change only what is drawn:
//!   the name, `Element::text` and `get_by_text` use the whole text.
//! - Rare properties live in the boxed `TextExtras`, so an unused style
//!   pays no size or merge cost for them.
//! - Rich text (`rich_text`, `IntoValue<Vec<Span>>`) writes one text
//!   payload of the joined content with byte-range styles; its spans hold
//!   only explicit weight, color, size, italic, family, spacing, kerning,
//!   decoration, features and band paint. A span's decoration and features
//!   replace the paragraph's (unlike CSS's decoration propagation); when
//!   features are replaced, the kerning feature is rebuilt from the span's
//!   or the paragraph's kerning. Inline boxes inside a paragraph are not
//!   supported.
//! - `Draw`'s `TextStyle` has the same vocabulary. A run with no line
//!   height takes the ancestor's, else the theme's; `TextRun`,
//!   `Draw::measure_text` and `Constraints::measure_text` follow the same
//!   rule, so their height matches a text node's. `measure_text` shapes the
//!   same run the drawing does and returns its size and first baseline.

use kivact_types as p;

use super::color::Color;
use super::effect::Shadow;
use super::paint::Paint;
use super::theme::value::Theme;

pub use kivact_types::PaintOrder;

/// One OpenType variation axis setting (CSS `font-variation-settings`): a
/// four-letter tag and its value.
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::FontVariation;
/// let s = style().font_variation_settings([FontVariation::new("wght", 650.0)]);
/// # let _ = s;
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontVariation {
    /// The axis tag; None when the text was not four ASCII characters (it is
    /// counted as unsupported).
    pub(crate) tag: Option<[u8; 4]>,
    pub(crate) value: f64,
}

impl FontVariation {
    /// A setting of the axis `tag` to `value`.
    pub fn new(tag: &str, value: impl Into<f64>) -> FontVariation {
        let bytes = tag.as_bytes();
        let tag = (bytes.len() == 4 && bytes.iter().all(u8::is_ascii))
            .then(|| [bytes[0], bytes[1], bytes[2], bytes[3]]);
        FontVariation {
            tag,
            value: value.into(),
        }
    }
}

impl From<(&str, f64)> for FontVariation {
    fn from((tag, value): (&str, f64)) -> Self {
        FontVariation::new(tag, value)
    }
}

impl From<(&str, i32)> for FontVariation {
    fn from((tag, value): (&str, i32)) -> Self {
        FontVariation::new(tag, value as f64)
    }
}

/// A text length: px (a number: `letter_spacing(1)`) or a multiple of the
/// font size of the text it applies to ([`em`]: CSS `0.08em` is
/// `em(0.08)`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextLength {
    /// Logical px.
    Px(f64),
    /// A multiple of the text's font size.
    Em(f64),
}

impl Default for TextLength {
    fn default() -> Self {
        TextLength::Px(0.0)
    }
}

impl From<i32> for TextLength {
    fn from(v: i32) -> Self {
        TextLength::Px(v as f64)
    }
}

impl From<u32> for TextLength {
    fn from(v: u32) -> Self {
        TextLength::Px(v as f64)
    }
}

impl From<f32> for TextLength {
    fn from(value: f32) -> Self {
        Self::Px(f64::from(value))
    }
}

impl From<f64> for TextLength {
    fn from(v: f64) -> Self {
        TextLength::Px(v)
    }
}

/// `v` times the font size of the text it applies to (CSS `em`), for the
/// text lengths: `letter_spacing(em(0.08))`, `text_underline_offset(em(0.2))`.
pub fn em(v: impl Into<f64>) -> TextLength {
    TextLength::Em(v.into())
}

impl TextLength {
    pub(crate) fn to_wire(self) -> p::TextLen {
        match self {
            TextLength::Px(v) => p::TextLen::Px(v),
            TextLength::Em(v) => p::TextLen::Em(v),
        }
    }
}

/// The distance between the baselines of a text's lines (CSS
/// `line-height`).
///
/// A plain number is a multiple of the font size, as CSS's unitless
/// `line-height`; a length in px is [`LineHeight::px`]:
///
/// | CSS | Kivact |
/// |---|---|
/// | `line-height: 1.5` | `line_height(1.5)` |
/// | `line-height: 20px` | `line_height(LineHeight::px(20))` |
/// | `line-height: normal` | `LineHeight::Normal` |
///
/// A number never changes meaning with its size: `line_height(4.0)` is four
/// times the font size and `line_height(4.5)` four and a half times.
#[doc(inline)]
pub use kivact_types::LineHeight;

/// Which lines decorate text (CSS `text-decoration-line`), combined with
/// `|`: `TextDecorationLine::UNDERLINE | TextDecorationLine::OVERLINE`.
#[doc(inline)]
pub use kivact_types::DecorationLines as TextDecorationLine;
pub use kivact_types::TextDecorationStyle;

/// What a line of text that does not fit shows (CSS `text-overflow`).
#[doc(inline)]
pub use kivact_types::TextOverflow;

/// Case conversion when text is drawn (CSS `text-transform`). The text's
/// accessible name and `Element::text` keep the text as written.
#[doc(inline)]
pub use kivact_types::TextTransform;

/// Pair kerning of text (CSS `font-kerning`).
#[doc(inline)]
pub use kivact_types::FontKerning;

/// An OpenType feature setting (one entry of CSS `font-feature-settings`):
/// a four letter tag and its value. `("tnum", 1)`, `("ss01", 1)` and
/// `"cv11"` (on) convert to it.
///
/// It is the authoring form, kept so that a bad tag can be diagnosed; once
/// checked, the engine's wire takes the shared value type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontFeature {
    /// `None`: not four ASCII characters (dropped and reported).
    tag: Option<[u8; 4]>,
    value: u16,
}

impl FontFeature {
    /// The feature `tag` (four ASCII characters: `"tnum"`) set to `value`
    /// (`1` on, `0` off, or an alternate's index). Another tag is dropped
    /// and reported as `ErrorKind::Unsupported` by the node that uses it.
    pub fn new(tag: &str, value: u16) -> Self {
        let tag = <[u8; 4]>::try_from(tag.as_bytes())
            .ok()
            .filter(|t| t.iter().all(|b| b.is_ascii_graphic() || *b == b' '));
        FontFeature { tag, value }
    }

    fn to_wire(self) -> Option<p::FontFeature> {
        self.tag.map(|tag| p::FontFeature {
            tag,
            value: self.value,
        })
    }
}

impl From<(&str, i32)> for FontFeature {
    fn from((tag, value): (&str, i32)) -> Self {
        FontFeature::new(tag, value.clamp(0, i32::from(u16::MAX)) as u16)
    }
}

impl From<(&str, u16)> for FontFeature {
    fn from((tag, value): (&str, u16)) -> Self {
        FontFeature::new(tag, value)
    }
}

impl From<&str> for FontFeature {
    fn from(tag: &str) -> Self {
        FontFeature::new(tag, 1)
    }
}

// xtask-allow(state-fields): Independent authored text properties use None for unspecified cascading values.
/// The text properties of a style that are rare enough to keep out of
/// line (boxed in `Style`).
#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct TextExtras {
    pub(crate) letter_spacing: Option<TextLength>,
    pub(crate) word_spacing: Option<TextLength>,
    pub(crate) kerning: Option<FontKerning>,
    pub(crate) features: Option<Vec<FontFeature>>,
    pub(crate) tabular_nums: Option<bool>,
    pub(crate) decoration_line: Option<TextDecorationLine>,
    pub(crate) decoration_style: Option<DecorationStyleValue>,
    pub(crate) decoration_color: Option<Color>,
    pub(crate) decoration_thickness: Option<TextLength>,
    pub(crate) underline_offset: Option<TextLength>,
    pub(crate) overflow: Option<TextOverflow>,
    pub(crate) line_clamp: Option<u32>,
    pub(crate) capsize: Option<bool>,
    pub(crate) transform: Option<TextTransform>,
    pub(crate) shadows: Option<Vec<Shadow>>,
    pub(crate) caret_color: Option<Color>,
    pub(crate) selection_color: Option<Color>,
    pub(crate) placeholder_color: Option<Color>,
    pub(crate) stroke: Option<(TextLength, Color)>,
    pub(crate) paint_order: Option<PaintOrder>,
    pub(crate) variations: Option<Vec<FontVariation>>,
    pub(crate) optical_sizing: Option<bool>,
    pub(crate) font_width: Option<f64>,
}

impl TextExtras {
    /// Overlay `o` (fields set in `o` win).
    pub(crate) fn merge(&mut self, o: &TextExtras) {
        macro_rules! take {
            ($($f:ident),*) => {$(
                if o.$f.is_some() {
                    self.$f = o.$f.clone();
                }
            )*};
        }
        take!(
            letter_spacing,
            word_spacing,
            kerning,
            features,
            tabular_nums,
            decoration_line,
            decoration_style,
            decoration_color,
            decoration_thickness,
            underline_offset,
            overflow,
            line_clamp,
            capsize,
            transform,
            shadows,
            caret_color,
            selection_color,
            placeholder_color,
            stroke,
            paint_order,
            variations,
            optical_sizing,
            font_width
        );
    }
}

/// The OpenType features of `settings` and the numeric variant
/// `tabular` (CSS `font-variant-numeric: tabular-nums` adds `tnum` after
/// the settings), and how many were dropped (not four ASCII characters).
pub(crate) fn features(
    settings: Option<&[FontFeature]>,
    tabular: bool,
) -> (Vec<p::FontFeature>, u32) {
    let settings = settings.unwrap_or(&[]);
    let mut out: Vec<p::FontFeature> = settings.iter().filter_map(|f| f.to_wire()).collect();
    let dropped = (settings.len() - out.len()) as u32;
    if tabular {
        out.push(p::FontFeature {
            tag: *b"tnum",
            value: 1,
        });
    }
    (out, dropped)
}

/// A decoration as the engine takes it.
pub(crate) fn decoration(
    line: Option<TextDecorationLine>,
    color: Option<Color>,
    thickness: Option<TextLength>,
    offset: Option<TextLength>,
    theme: &Theme,
) -> p::TextDecoration {
    p::TextDecoration {
        style: TextDecorationStyle::default(),
        lines: line.unwrap_or_default(),
        color: color.map(|c| c.resolve(theme)),
        thickness: thickness.map(TextLength::to_wire),
        underline_offset: offset.map(TextLength::to_wire).unwrap_or_default(),
    }
}

/// The wire variation settings of `list`, and how many were dropped (a tag
/// that is not four ASCII characters).
pub(crate) fn wire_variations(list: &[FontVariation]) -> (Vec<p::FontVariation>, u32) {
    let mut dropped = 0;
    let out = list
        .iter()
        .filter_map(|v| match v.tag {
            Some(tag) => Some(p::FontVariation {
                tag,
                value: v.value,
            }),
            None => {
                dropped += 1;
                None
            }
        })
        .collect();
    (out, dropped)
}

/// Fill `t` (the engine's text props of a style) with the extras `x` it
/// sets, and return how many values were dropped (a feature tag that is
/// not four ASCII characters, a text shadow's spread or inset).
pub(crate) fn resolve_extras(t: &mut p::TextStyle, x: &TextExtras, theme: &Theme) -> u32 {
    let mut dropped = 0;
    let features = x.features.as_deref().map(|settings| {
        let (features, bad) = self::features(Some(settings), false);
        dropped += bad;
        features
    });
    let shadows = x.shadows.as_ref().map(|shadows| {
        shadows
            .iter()
            .map(|s| {
                if s.spread != 0.0 || s.inset {
                    dropped += 1;
                }
                p::TextShadow {
                    x: s.offset.x,
                    y: s.offset.y,
                    blur: s.blur,
                    color: s.color.resolve(theme),
                }
            })
            .collect()
    });
    let color = |c: Option<Color>| c.map(|c| c.resolve(theme));
    let variations = x.variations.as_deref().map(|list| {
        let (list, bad) = wire_variations(list);
        dropped += bad;
        list
    });
    t.stroke = x.stroke.map(|(width, c)| p::TextStroke {
        width: width.to_wire(),
        color: c.resolve(theme),
    });
    t.paint_order = x.paint_order;
    t.variations = variations;
    t.optical_sizing = x.optical_sizing;
    t.font_width = x.font_width;
    t.letter_spacing = x.letter_spacing.map(TextLength::to_wire);
    t.word_spacing = x.word_spacing.map(TextLength::to_wire);
    t.kerning = x.kerning;
    t.features = features;
    t.tabular_nums = x.tabular_nums;
    t.decoration_lines = x.decoration_line;
    t.decoration_style = x.decoration_style.as_ref().map(DecorationStyleValue::get);
    t.decoration_color = color(x.decoration_color);
    t.decoration_thickness = x.decoration_thickness.map(TextLength::to_wire);
    t.underline_offset = x.underline_offset.map(TextLength::to_wire);
    t.overflow = x.overflow;
    t.max_lines = x.line_clamp;
    t.capsize = x.capsize;
    t.transform = x.transform;
    t.shadows = shadows;
    t.caret_color = color(x.caret_color);
    t.selection_color = color(x.selection_color);
    t.placeholder_color = color(x.placeholder_color);
    dropped
}

// xtask-allow(state-fields): Independent authored span overrides use None to inherit their paragraph properties.
/// The text properties of one span of rich text; each one is optional and an
/// unset one is the paragraph's. A span's decoration lines and features
/// replace the paragraph's. The `Span` builders of `kivact::view` fill
/// it, and [`crate::internal::compile_span_style`] turns it into the engine's form.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SpanStyle {
    /// Font size in logical px.
    pub font_size: Option<f64>,
    /// Font weight.
    pub weight: Option<u16>,
    /// Color of the glyphs.
    pub color: Option<Color>,
    /// Font family: a name, a generic family or an ordered list.
    pub family: Option<String>,
    /// Italic, or upright with `false`.
    pub italic: Option<bool>,
    /// Space after each character.
    pub letter_spacing: Option<TextLength>,
    /// Space after each word.
    pub word_spacing: Option<TextLength>,
    /// Pair kerning.
    pub kerning: Option<FontKerning>,
    /// OpenType features replacing the paragraph's.
    pub features: Option<Vec<FontFeature>>,
    /// Tabular figures (adds the `tnum` feature when `true`).
    pub tabular_nums: Option<bool>,
    /// Decoration lines replacing the paragraph's.
    pub decoration_line: Option<TextDecorationLine>,
    /// Decoration color (default: the text color).
    pub decoration_color: Option<Color>,
    /// Decoration thickness.
    pub decoration_thickness: Option<TextLength>,
    /// Underline offset from its normal place.
    pub underline_offset: Option<TextLength>,
    /// A band behind the glyphs on each line the span is on.
    pub background: Option<Paint>,
    /// OpenType variation axis settings replacing the paragraph's.
    pub variations: Option<Vec<FontVariation>>,
    /// CSS font-width (font-stretch) in percent, 50 to 200.
    pub font_width: Option<f64>,
    /// Outline of its glyphs (CSS `-webkit-text-stroke`): width and color.
    pub stroke: Option<(TextLength, Color)>,
}

/// A [`SpanStyle`] in the engine's form, made by [`crate::internal::compile_span_style`].
pub struct CompiledSpanStyle {
    /// The properties the span sets (an unset one stays unset).
    pub props: p::SpanProps,
    /// Font feature tags dropped because they are not four ASCII characters.
    pub unsupported: u32,
}

/// A tracked decoration input, preserving constant equality and dynamic identity.
#[derive(Clone)]
pub(crate) enum DecorationStyleValue {
    Fixed(TextDecorationStyle),
    Dynamic(std::rc::Rc<kivact_runtime::Prop<TextDecorationStyle>>),
}
impl DecorationStyleValue {
    pub(crate) fn new(value: impl kivact_runtime::IntoValue<TextDecorationStyle>) -> Self {
        let value = value.into_prop();
        match value.fixed() {
            Some(value) => Self::Fixed(value),
            None => Self::Dynamic(std::rc::Rc::new(value)),
        }
    }
    fn get(&self) -> TextDecorationStyle {
        match self {
            Self::Fixed(value) => *value,
            Self::Dynamic(value) => value.get(),
        }
    }
}
impl std::fmt::Debug for DecorationStyleValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DecorationStyleValue(..)")
    }
}
impl PartialEq for DecorationStyleValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Fixed(a), Self::Fixed(b)) => a == b,
            (Self::Dynamic(a), Self::Dynamic(b)) => std::rc::Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

// xtask-allow(state-fields): Foreground, background and decoration independently override one byte range.
/// Paint overrides for a UTF-8 byte range within one editing line.
#[derive(Clone, Debug, PartialEq)]
pub struct StyledRange {
    pub range: std::ops::Range<usize>,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub decoration: Option<TextDecoration>,
}

// xtask-allow(state-fields): Color and thickness are independently optional authored decoration properties.
/// Lines, pattern and dimensions of a text decoration.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TextDecoration {
    pub lines: TextDecorationLine,
    pub color: Option<Color>,
    pub thickness: Option<TextLength>,
    pub underline_offset: TextLength,
    pub style: TextDecorationStyle,
}
