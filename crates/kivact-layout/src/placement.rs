//! Role: Overlay placement: where `OverlayProps` put an overlay.
//!
//! Invariants:
//! - An overlay with an anchor (a node's drawn bounds, or a window point:
//!   an empty rect) goes on the requested side of it, `offset` away (0
//!   when it is not finite). When
//!   it does not fit there, it flips to the opposite side (`Below`↔`Above`,
//!   `Start`↔`End`) if that side fits, or if neither fits and the
//!   opposite side has more room. `Center` centers it over the anchor.
//! - Along the side it is aligned to the anchor's start, centre or end
//!   (`PlacementAlign`). A start or end alignment that overflows the
//!   window flips to the other one when that fits (a right-aligned menu
//!   near the left edge opens rightwards; a menu at the pointer near the
//!   right edge opens leftwards).
//! - An overlay without an anchor (or whose anchor is gone) is centered in
//!   the window, whatever its placement.
//! - The result is then clamped into the window; an overlay larger than
//!   the window is aligned to its top-left corner.

use kivact_wire::{Placement, PlacementAlign, Point, Rect, Size};

/// Window-space origin of an overlay of `size` placed at `placement` of
/// `anchor` (its window-space rect, `None` when unanchored), aligned by
/// `align` and `offset` away from it, in a window of `window` (see the
/// module invariants).
pub fn place_overlay(
    placement: Placement,
    align: PlacementAlign,
    offset: f32,
    anchor: Option<Rect>,
    size: Size,
    window: Size,
) -> Point {
    let Some(a) = anchor else {
        return clamp(
            Point {
                x: (window.width - size.width) / 2.0,
                y: (window.height - size.height) / 2.0,
            },
            size,
            window,
        );
    };
    let offset = if offset.is_finite() { offset } else { 0.0 };
    let p = match placement {
        Placement::Below | Placement::Above => Point {
            x: along(align, a.x, a.width, size.width, window.width),
            y: side(
                placement == Placement::Below,
                a.y - offset,
                a.y + a.height + offset,
                size.height,
                window.height,
            ),
        },
        Placement::Start | Placement::End => Point {
            x: side(
                placement == Placement::End,
                a.x - offset,
                a.x + a.width + offset,
                size.width,
                window.width,
            ),
            y: along(align, a.y, a.height, size.height, window.height),
        },
        Placement::Center => Point {
            x: a.x + (a.width - size.width) / 2.0,
            y: a.y + (a.height - size.height) / 2.0,
        },
    };
    clamp(p, size, window)
}

/// Start coordinate on the placement's axis: after the anchor (`end`) or
/// before it (`start - size`), preferring `after`, flipped as the module
/// says.
fn side(after: bool, start: f32, end: f32, size: f32, limit: f32) -> f32 {
    let room_after = limit - end;
    let room_before = start;
    let fits_after = size <= room_after;
    let fits_before = size <= room_before;
    let use_after = if after {
        fits_after || (!fits_before && room_after >= room_before)
    } else {
        !(fits_before || (!fits_after && room_before >= room_after))
    };
    if use_after { end } else { start - size }
}

/// Start coordinate along the anchor's side (`start`, `length`) of an
/// overlay of `size` aligned by `align`, flipped as the module says.
fn along(align: PlacementAlign, start: f32, length: f32, size: f32, limit: f32) -> f32 {
    let at_start = start;
    let at_end = start + length - size;
    match align {
        PlacementAlign::Center => start + (length - size) / 2.0,
        PlacementAlign::Start if at_start + size > limit && at_end >= 0.0 => at_end,
        PlacementAlign::Start => at_start,
        PlacementAlign::End if at_end < 0.0 && at_start + size <= limit => at_start,
        PlacementAlign::End => at_end,
    }
}

fn clamp(p: Point, size: Size, window: Size) -> Point {
    Point {
        x: p.x.clamp(0.0, (window.width - size.width).max(0.0)),
        y: p.y.clamp(0.0, (window.height - size.height).max(0.0)),
    }
}
