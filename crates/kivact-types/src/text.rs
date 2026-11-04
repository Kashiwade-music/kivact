//! Role: Text primitives and declared or computed text properties.
mod computed;
pub mod primitives;
mod properties;

pub use primitives::{
    DecorationLines, FontFeature, FontKerning, LineHeight, TextAlign, TextMetrics, TextOverflow,
    TextTransform, TextWrap,
};
pub use properties::{
    SpanProps, TextData, TextDecoration, TextLen, TextShadow, TextSpan, TextStyle,
};

pub use computed::ComputedTextStyle;
