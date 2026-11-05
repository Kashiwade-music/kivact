//! Role: Passive native editor paint facts prepared by the model for frame rendering.

use kivact_types::{Point, Rect, Rgba};

pub(crate) struct PreparedInputPaint {
    pub(crate) area: Rect,
    pub(crate) ops: Vec<InputPaintOp>,
}

#[derive(Clone, Copy)]
pub(crate) enum InputPaintColor {
    Source(Rgba),
    Render(Rgba),
}

pub(crate) enum InputPaintOp {
    Rect {
        rect: Rect,
        color: InputPaintColor,
    },
    Line {
        from: Point,
        to: Point,
        width: f64,
        color: Rgba,
    },
    RangeDecoration {
        rect: Rect,
        kind: kivact_editing::RangeDecorationKind,
        color: Rgba,
    },
    Decoration {
        rect: Rect,
        color: Rgba,
        style: kivact_types::TextDecorationStyle,
    },
    Glyphs {
        shaped: kivact_types::ShapedTextSnapshot,
        origin: Point,
        color: Option<Rgba>,
    },
}
