//! Role: Image node sizing using CSS img rules.
//!
//! The size of an `Image` node: the rule of CSS `<img>`, applied in the
//! layout pass. Pure: the caller supplies the natural size its asset
//! layer found.
//!
//! Invariants:
//! - A dimension the node's style sets (`width`, `height`; a percentage as
//!   the layout resolved it) wins. Dimensions the layout derives on its
//!   own (a flex basis, stretching) are not inputs of the rule; the layout
//!   applies them over its result as for every leaf.
//! - Otherwise use the intrinsic hint's preferred size, or its minimum,
//!   then the natural size. Without any available size it is 0 × 0.
//! - With one dimension set by the style, the other follows the aspect
//!   ratio: the natural one when the image is loaded, else the hint's (as
//!   CSS uses the `width`/`height` attributes' ratio until the image
//!   loads). Without a usable ratio (none, or an empty side) it is the
//!   reference size's.

use kivact_wire::{Dimension, IntrinsicSize, LayoutProps, MeasureRequest, Size};

/// The content size of an image node from the dimensions its style sets
/// (`width`, `height`), its optional intrinsic size hint and its natural
/// size in logical pixels. `None` means the resource provider has no
/// natural size available, including while loading or after failure.
pub fn image_content_size(
    width: Option<f32>,
    height: Option<f32>,
    intrinsic: Option<&IntrinsicSize>,
    natural: Option<Size>,
) -> Size {
    let hint = intrinsic.and_then(|i| i.preferred.or(i.min));
    let reference = hint.or(natural).unwrap_or_default();
    let ratio = natural.or(hint).filter(|r| r.width > 0.0 && r.height > 0.0);
    match (width, height) {
        (Some(width), Some(height)) => Size { width, height },
        (Some(width), None) => Size {
            width,
            height: ratio.map_or(reference.height, |r| width * r.height / r.width),
        },
        (None, Some(height)) => Size {
            width: ratio.map_or(reference.width, |r| height * r.width / r.height),
            height,
        },
        (None, None) => reference,
    }
}

/// The dimensions of `request` that the style `layout` sets (lengths, not
/// `auto` nor a content size).
pub fn style_known(
    request: MeasureRequest,
    layout: Option<&LayoutProps>,
) -> (Option<f32>, Option<f32>) {
    let set = |d: Option<Dimension>| {
        d.is_some_and(|d| {
            matches!(
                d,
                Dimension::Px(_) | Dimension::Percent(_) | Dimension::Calc { .. }
            )
        })
    };
    (
        request.known_width.filter(|_| set(layout.map(|l| l.width))),
        request
            .known_height
            .filter(|_| set(layout.map(|l| l.height))),
    )
}
