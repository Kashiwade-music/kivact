//! Role: Text primitives and declared or computed text properties.
pub mod primitives;
mod properties;

pub use primitives::{
    DecorationLines, FontKerning, LineHeight, TextAlign, TextMetrics, TextOverflow, TextTransform,
};
pub use properties::{
    ComputedText, SpanProps, TextData, TextDecoration, TextLen, TextMore, TextProps, TextShadow,
    TextSpan,
};
