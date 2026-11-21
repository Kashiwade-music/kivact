//! Role: The clip shapes the console's recipes share.
use kivact::style::{Shape, pct};

/// Two opposite cut corners, `cut` logical pixels deep.
pub fn cut_corner(cut: impl Into<f64>) -> Shape {
    let cut = cut.into();
    Shape::polygon()
        .point(0, 0)
        .point(pct(100) - cut, 0)
        .point(pct(100), cut)
        .point(pct(100), pct(100))
        .point(cut, pct(100))
        .point(0, pct(100) - cut)
}
