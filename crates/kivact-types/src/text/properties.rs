//! Role: Authored text content with declared and computed typography.
use crate::text::primitives::{FontKerning, LineHeight, TextAlign, TextOverflow, TextTransform};
use crate::{Fill, FontFeature, Rgba, TextWrap};
use std::ops::Range;

/// Owned text content and optional per-span styling.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextData {
    /// Owned text displayed by this node.
    pub content: Option<String>,
    /// Per-range overrides over the node text.
    pub spans: Option<Vec<TextSpan>>,
}

/// A text length: px, or a multiple of the font size of the text it
/// applies to (CSS `em`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextLen {
    Px(f32),
    Em(f32),
}

impl Default for TextLen {
    fn default() -> Self {
        TextLen::Px(0.0)
    }
}

impl TextLen {
    /// In px, for text of `font_size`.
    pub fn px(self, font_size: f32) -> f32 {
        match self {
            TextLen::Px(v) => v,
            TextLen::Em(v) => v * font_size,
        }
    }
}

/// The distance between the baselines of a text's lines (CSS
/// `line-height`).

/// An OpenType feature setting: a four byte tag and its value.

/// Lines drawn with text (CSS `text-decoration`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TextDecoration {
    /// The lines to draw.
    pub lines: crate::DecorationLines,
    /// `None`: the text's color.
    pub color: Option<Rgba>,
    /// `None`: the font's.
    pub thickness: Option<TextLen>,
    /// CSS `text-underline-offset`: further from the text than the font's
    /// position.
    pub underline_offset: TextLen,
}

/// What text that does not fit shows (CSS `text-overflow`).

/// Case conversion when shaping (CSS `text-transform`).

/// A shadow of text (CSS `text-shadow`).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TextShadow {
    pub x: f32,
    pub y: f32,
    /// Blur radius (the gaussian's standard deviation is half of it).
    pub blur: f32,
    pub color: Rgba,
}

/// What the text style of a node sets, for the text in it and below: each
/// `None` is inherited from the nearest ancestor that sets that property
/// (the root carries the theme's), and with none the default of
/// [`ComputedText`]. Every property is inherited on its own, as CSS's
/// inherited properties are: a node that sets only `weight` keeps its
/// parent's color and size. The decoration, overflow and line clamp are
/// inherited too (a container's reach the texts in it).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextProps {
    pub font_size: Option<f32>,
    pub weight: Option<u16>,
    pub color: Option<Rgba>,
    /// An explicit `LineHeight::Normal` is not inherited over.
    pub line_height: Option<LineHeight>,
    pub wrap: Option<TextWrap>,
    pub align: Option<TextAlign>,
    /// A family name or a generic family (`system-ui`, `sans-serif`,
    /// `serif`, `monospace`), resolved by the host: a family that is not
    /// installed falls back to the system UI font and is reported once as
    /// `HostEvent::AssetFailed`.
    pub family: Option<String>,
    pub italic: Option<bool>,
    pub letter_spacing: Option<TextLen>,
    pub word_spacing: Option<TextLen>,
    pub kerning: Option<FontKerning>,
    /// CSS `font-feature-settings`.
    pub features: Option<Vec<FontFeature>>,
    /// CSS `font-variant-numeric: tabular-nums`: `tnum` after the features.
    pub tabular_nums: Option<bool>,
    pub decoration_lines: Option<crate::DecorationLines>,
    pub decoration_color: Option<Rgba>,
    pub decoration_thickness: Option<TextLen>,
    pub underline_offset: Option<TextLen>,
    pub overflow: Option<TextOverflow>,
    /// `Some(0)`: no clamp.
    pub max_lines: Option<u32>,
    pub capsize: Option<bool>,
    pub transform: Option<TextTransform>,
    pub shadows: Option<Vec<TextShadow>>,
    pub caret_color: Option<Rgba>,
    pub selection_color: Option<Rgba>,
    pub placeholder_color: Option<Rgba>,
}

macro_rules! text_fields {
    ($m:ident) => {
        $m!(
            font_size,
            weight,
            color,
            line_height,
            wrap,
            align,
            family,
            italic,
            letter_spacing,
            word_spacing,
            kerning,
            features,
            tabular_nums,
            decoration_lines,
            decoration_color,
            decoration_thickness,
            underline_offset,
            overflow,
            max_lines,
            transform,
            shadows,
            caret_color,
            selection_color,
            placeholder_color
        )
    };
}

impl TextProps {
    /// Take each property this style leaves unset from `ancestor`'s.
    pub fn inherit(&mut self, ancestor: &TextProps) {
        macro_rules! fill {
            ($($f:ident),*) => {$(
                if self.$f.is_none() && ancestor.$f.is_some() {
                    self.$f = ancestor.$f.clone();
                }
            )*};
        }
        text_fields!(fill);
    }

    /// Whether it sets every property (nothing more to inherit).
    pub fn is_complete(&self) -> bool {
        macro_rules! all {
            ($($f:ident),*) => { $(self.$f.is_some())&&* };
        }
        text_fields!(all)
    }

    /// The text style of these props, the defaults of [`ComputedText`]
    /// where they set nothing.
    pub fn computed(self) -> ComputedText {
        let d = ComputedText::default();
        let mut features = self.features.unwrap_or_default();
        if self.tabular_nums == Some(true) {
            features.push(FontFeature {
                tag: *b"tnum",
                value: 1,
            });
        }
        let more = TextMore {
            letter_spacing: self.letter_spacing.unwrap_or_default(),
            word_spacing: self.word_spacing.unwrap_or_default(),
            kerning: self.kerning.unwrap_or_default(),
            features,
            decoration: TextDecoration {
                lines: self.decoration_lines.unwrap_or_default(),
                color: self.decoration_color,
                thickness: self.decoration_thickness,
                underline_offset: self.underline_offset.unwrap_or_default(),
            },
            overflow: self.overflow.unwrap_or_default(),
            max_lines: self.max_lines.filter(|n| *n > 0),
            capsize: self.capsize.unwrap_or_default(),
            transform: self.transform.unwrap_or_default(),
            shadows: self.shadows.unwrap_or_default(),
            caret_color: self.caret_color,
            selection_color: self.selection_color,
            placeholder_color: self.placeholder_color,
        };
        ComputedText {
            font_size: self.font_size.unwrap_or(d.font_size),
            weight: self.weight.unwrap_or(d.weight),
            color: self.color.unwrap_or(d.color),
            line_height: self.line_height,
            wrap: self.wrap.unwrap_or(d.wrap),
            align: self.align.unwrap_or(d.align),
            family: self.family,
            italic: self.italic.unwrap_or(d.italic),
            more: (more != TextMore::DEFAULT).then(|| Box::new(more)),
        }
    }
}

/// The text style a text is laid out and drawn with: the [`TextProps`] of
/// the node and its ancestors, each property from the nearest that sets
/// it, and the defaults below where none does.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputedText {
    pub font_size: f32,
    pub weight: u16,
    pub color: Rgba,
    /// `None`: no ancestor sets one, drawn and measured as
    /// `LineHeight::Normal`.
    pub line_height: Option<LineHeight>,
    pub wrap: TextWrap,
    pub align: TextAlign,
    /// `None`: no ancestor names one (the system UI font).
    pub family: Option<String>,
    /// Use the italic face of the family when it has one.
    pub italic: bool,
    /// The rest of the text vocabulary (`None`: all of it default), out
    /// of line so a plain text style stays small and quick to copy.
    pub more: Option<Box<TextMore>>,
}

impl ComputedText {
    /// The rest of the text vocabulary (the defaults without it).
    pub fn more(&self) -> &TextMore {
        static DEFAULT: TextMore = TextMore::DEFAULT;
        self.more.as_deref().unwrap_or(&DEFAULT)
    }
}

/// The text props beyond the common ones ([`ComputedText::more`]).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextMore {
    /// CSS `letter-spacing`.
    pub letter_spacing: TextLen,
    /// CSS `word-spacing`: extra space after every space.
    pub word_spacing: TextLen,
    /// CSS `font-kerning`.
    pub kerning: FontKerning,
    /// CSS `font-feature-settings` (with `font-variant-numeric`'s).
    pub features: Vec<FontFeature>,
    pub decoration: TextDecoration,
    pub overflow: TextOverflow,
    /// CSS `line-clamp`: the most lines shown, the last ending in "…".
    pub max_lines: Option<u32>,
    pub capsize: bool,
    pub transform: TextTransform,
    /// CSS `text-shadow`, the first on top.
    pub shadows: Vec<TextShadow>,
    /// CSS `caret-color` of a text input (`None`: the text color).
    pub caret_color: Option<Rgba>,
    /// The band behind a text input's selected text (CSS `::selection`
    /// background; `None`: the host's).
    pub selection_color: Option<Rgba>,
    /// A text input's placeholder color (CSS `::placeholder`; `None`: the
    /// text color at half its alpha).
    pub placeholder_color: Option<Rgba>,
}

impl TextMore {
    /// Every prop at its default.
    pub const DEFAULT: TextMore = TextMore {
        letter_spacing: TextLen::Px(0.0),
        word_spacing: TextLen::Px(0.0),
        kerning: FontKerning::Auto,
        features: Vec::new(),
        decoration: TextDecoration {
            lines: crate::DecorationLines::NONE,
            color: None,
            thickness: None,
            underline_offset: TextLen::Px(0.0),
        },
        overflow: TextOverflow::Clip,
        max_lines: None,
        capsize: false,
        transform: TextTransform::None,
        shadows: Vec::new(),
        caret_color: None,
        selection_color: None,
        placeholder_color: None,
    };
}

impl Default for ComputedText {
    fn default() -> Self {
        ComputedText {
            font_size: 14.0,
            weight: 400,
            color: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            line_height: None,
            wrap: TextWrap::Wrap,
            align: TextAlign::Start,
            family: None,
            italic: false,
            more: None,
        }
    }
}

/// What a span of rich text sets over its node's text style (`None`: the
/// node's).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SpanProps {
    pub font_size: Option<f32>,
    pub weight: Option<u16>,
    pub color: Option<Rgba>,
    pub family: Option<String>,
    pub italic: Option<bool>,
    pub letter_spacing: Option<TextLen>,
    pub word_spacing: Option<TextLen>,
    pub kerning: Option<FontKerning>,
    pub features: Option<Vec<FontFeature>>,
    pub decoration: Option<TextDecoration>,
    /// Filled behind the span's glyphs, one band per line (a `<mark>`).
    pub background: Option<Fill>,
}

/// A byte range of a rich text's `Content` and its style.
#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    pub range: Range<usize>,
    pub style: SpanProps,
}
