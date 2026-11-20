//! Role: Text primitives and declared or computed text properties.
mod computed;
pub mod primitives;
mod properties;

pub use primitives::{
    DecorationLines, FontFeature, FontKerning, LineHeight, TextAlign, TextMetrics, TextOverflow,
    TextTransform, TextWrap,
};
pub use properties::{
    FontVariation, PaintOrder, SpanProps, TextData, TextDecoration, TextDecorationStyle,
    TextLength, TextShadow, TextSpan, TextStroke, TextStyle, em,
};

pub use computed::ComputedTextStyle;
