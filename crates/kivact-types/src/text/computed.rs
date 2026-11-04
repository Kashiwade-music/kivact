//! Role: Resolve inherited typography without losing relative length semantics.
use crate::Rgba;
use crate::text::{
    FontFeature, FontKerning, LineHeight, TextAlign, TextDecoration, TextLen, TextOverflow,
    TextShadow, TextStyle, TextTransform, TextWrap,
};

/// Typography after inheritance. Optional values here have semantic meaning:
/// an unnamed family uses the system font, absent decoration colors use current
/// text color, and an absent line clamp means unlimited lines.
/// Relative lengths remain relative to this node's resolved font size, including
/// when inherited from a parent with a different font size.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputedTextStyle {
    pub font_size: f64,
    pub weight: u16,
    pub color: Rgba,
    pub line_height: LineHeight,
    pub wrap: TextWrap,
    pub align: TextAlign,
    pub family: Option<String>,
    pub italic: bool,
    pub letter_spacing: TextLen,
    pub word_spacing: TextLen,
    pub kerning: FontKerning,
    /// Explicit OpenType feature declarations, before the tabular-number feature.
    pub features: Vec<FontFeature>,
    pub tabular_nums: bool,
    pub decoration: TextDecoration,
    pub overflow: TextOverflow,
    pub max_lines: Option<u32>,
    pub capsize: bool,
    pub transform: TextTransform,
    pub shadows: Vec<TextShadow>,
    pub caret_color: Option<Rgba>,
    pub selection_color: Option<Rgba>,
    pub placeholder_color: Option<Rgba>,
}

impl Default for ComputedTextStyle {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            weight: 400,
            color: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            line_height: LineHeight::Normal,
            wrap: TextWrap::Wrap,
            align: TextAlign::Start,
            family: None,
            italic: false,
            letter_spacing: TextLen::default(),
            word_spacing: TextLen::default(),
            kerning: FontKerning::Auto,
            features: Vec::new(),
            tabular_nums: false,
            decoration: TextDecoration::default(),
            overflow: TextOverflow::Clip,
            max_lines: None,
            capsize: false,
            transform: TextTransform::None,
            shadows: Vec::new(),
            caret_color: None,
            selection_color: None,
            placeholder_color: None,
        }
    }
}

impl ComputedTextStyle {
    /// Resolve each declaration against the parent, preserving relative lengths.
    /// Invalid font sizes use 14px; invalid line heights use normal. Capsize is
    /// deliberately local, matching the engine's previous ancestor resolver.
    pub fn inherit(parent: &Self, own: &TextStyle) -> Self {
        let font_size = own.font_size.unwrap_or(parent.font_size);
        let font_size = if font_size.is_finite() && font_size > 0.0 {
            font_size
        } else {
            14.0
        };
        let line_height = own.line_height.unwrap_or(parent.line_height);
        let line_height = match line_height {
            LineHeight::Scale(v) | LineHeight::Px(v) if !v.is_finite() || v <= 0.0 => {
                LineHeight::Normal
            }
            value => value,
        };
        Self {
            font_size,
            weight: own.weight.unwrap_or(parent.weight),
            color: own.color.unwrap_or(parent.color).sanitize(),
            line_height,
            wrap: own.wrap.unwrap_or(parent.wrap),
            align: own.align.unwrap_or(parent.align),
            family: own.family.clone().or_else(|| parent.family.clone()),
            italic: own.italic.unwrap_or(parent.italic),
            letter_spacing: own.letter_spacing.unwrap_or(parent.letter_spacing),
            word_spacing: own.word_spacing.unwrap_or(parent.word_spacing),
            kerning: own.kerning.unwrap_or(parent.kerning),
            features: own
                .features
                .clone()
                .unwrap_or_else(|| parent.features.clone()),
            tabular_nums: own.tabular_nums.unwrap_or(parent.tabular_nums),
            decoration: TextDecoration {
                lines: own.decoration_lines.unwrap_or(parent.decoration.lines),
                color: own
                    .decoration_color
                    .or(parent.decoration.color)
                    .map(Rgba::sanitize),
                thickness: own.decoration_thickness.or(parent.decoration.thickness),
                underline_offset: own
                    .underline_offset
                    .unwrap_or(parent.decoration.underline_offset),
            },
            overflow: own.overflow.unwrap_or(parent.overflow),
            max_lines: own
                .max_lines
                .map_or(parent.max_lines, |n| (n > 0).then_some(n)),
            capsize: own.capsize.unwrap_or(false),
            transform: own.transform.unwrap_or(parent.transform),
            shadows: own
                .shadows
                .clone()
                .unwrap_or_else(|| parent.shadows.clone()),
            caret_color: own.caret_color.or(parent.caret_color).map(Rgba::sanitize),
            selection_color: own
                .selection_color
                .or(parent.selection_color)
                .map(Rgba::sanitize),
            placeholder_color: own
                .placeholder_color
                .or(parent.placeholder_color)
                .map(Rgba::sanitize),
        }
    }

    /// Resolve a relative text length against this node's font size.
    /// Non-finite results become zero, as at the previous lowering boundary.
    pub fn resolve(&self, length: TextLen) -> f64 {
        let value = match length {
            TextLen::Px(v) => v,
            TextLen::Em(v) => v * self.font_size,
        };
        if value.is_finite() { value } else { 0.0 }
    }

    /// OpenType declarations with tabular numbers applied last when requested.
    pub fn shaping_features(&self) -> Vec<FontFeature> {
        let mut features = self.features.clone();
        if self.tabular_nums {
            features.push(FontFeature::new(*b"tnum", 1));
        }
        features
    }

    /// Positive finite decoration thickness in pixels, or the font's thickness.
    pub fn decoration_thickness_px(&self) -> Option<f64> {
        self.decoration
            .thickness
            .map(|value| self.resolve(value))
            .filter(|value| *value > 0.0)
    }

    /// Whether overflow or a line limit requires an ellipsis.
    pub fn ellipsizes(&self) -> bool {
        self.overflow == TextOverflow::Ellipsis || self.max_lines.is_some()
    }
}
