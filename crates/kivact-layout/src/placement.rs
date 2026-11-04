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
//!   (`OverlayAlign`). A start or end alignment that overflows the
//!   window flips to the other one when that fits (a right-aligned menu
//!   near the left edge opens rightwards; a menu at the pointer near the
//!   right edge opens leftwards).
//! - An overlay without an anchor (or whose anchor is gone) is centered in
//!   the window, whatever its placement.
//! - The result is then clamped into the window; an overlay larger than
//!   the window is aligned to its top-left corner.

use kivact_types::{OverlayAlign, OverlayProps, Placement, Point, Rect, Size};

/// Window-space origin of an overlay of `size` placed at `placement` of
/// `anchor` (its window-space rect, `None` when unanchored), aligned by
/// `align` and `offset` away from it, in a window of `window` (see the
/// module invariants).
fn place_without_inset(
    placement: Placement,
    align: OverlayAlign,
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
fn along(align: OverlayAlign, start: f32, length: f32, size: f32, limit: f32) -> f32 {
    let at_start = start;
    let at_end = start + length - size;
    match align {
        OverlayAlign::Center => start + (length - size) / 2.0,
        OverlayAlign::Start if at_start + size > limit && at_end >= 0.0 => at_end,
        OverlayAlign::Start => at_start,
        OverlayAlign::End if at_end < 0.0 && at_start + size <= limit => at_start,
        OverlayAlign::End => at_end,
    }
}

fn clamp(p: Point, size: Size, window: Size) -> Point {
    Point {
        x: p.x.clamp(0.0, (window.width - size.width).max(0.0)),
        y: p.y.clamp(0.0, (window.height - size.height).max(0.0)),
    }
}

/// Apply the opted-in viewport inset and the ordered anchor fallbacks.
pub fn place_overlay(
    props: &OverlayProps,
    anchor: Option<Rect>,
    size: Size,
    window: Size,
) -> Point {
    let Some(fit) = props.fit else {
        return place_without_inset(
            props.placement,
            props.align,
            props.offset,
            anchor,
            size,
            window,
        );
    };
    let viewport = fit.viewport(window);
    let a = anchor.unwrap_or(Rect {
        x: viewport.x + viewport.width / 2.0,
        y: viewport.y + viewport.height / 2.0,
        width: 0.0,
        height: 0.0,
    });
    let offset = if props.offset.is_finite() {
        props.offset
    } else {
        0.0
    };
    let (side, align) = match (fit.rtl, props.placement) {
        (true, Placement::Start) => (Placement::End, props.align),
        (true, Placement::End) => (Placement::Start, props.align),
        (true, Placement::Above | Placement::Below) => (props.placement, reverse(props.align)),
        _ => (props.placement, props.align),
    };
    let side = if anchor.is_some() {
        side
    } else {
        Placement::Center
    };
    let geometry = FitGeometry {
        anchor: a,
        size,
        viewport,
        offset,
    };
    let position = geometry.position(side, align);
    if align != OverlayAlign::Center
        && (a.width > 0.0 || a.height > 0.0)
        && (a.right() <= 0.0 || a.bottom() <= 0.0 || a.x >= window.width || a.y >= window.height)
    {
        let anchored = geometry.candidate(side, align);
        return match side {
            Placement::Above | Placement::Below => Point {
                x: anchored.x,
                ..position
            },
            Placement::Start | Placement::End => Point {
                y: anchored.y,
                ..position
            },
            Placement::Center => position,
        };
    }
    if fit.keep_anchor_clear && anchor.is_some() && geometry.overlaps_anchor(position) {
        geometry.clear_anchor(side, fit.rtl).unwrap_or(position)
    } else {
        position
    }
}

/// A size limit for a fitted tooltip whose full box would cover its anchor.
pub fn anchor_clear_size(
    props: &OverlayProps,
    anchor: Rect,
    size: Size,
    window: Size,
) -> Option<Size> {
    let fit = props.fit.filter(|fit| fit.keep_anchor_clear)?;
    let viewport = fit.viewport(window);
    let offset = if props.offset.is_finite() {
        props.offset.max(0.0)
    } else {
        0.0
    };
    let geometry = FitGeometry {
        anchor,
        size,
        viewport,
        offset,
    };
    if !geometry.overlaps_anchor(place_overlay(props, Some(anchor), size, window)) {
        return None;
    }
    let room = |before: f32, after: f32| (before.max(after) - offset).max(0.0);
    Some(
        if matches!(props.placement, Placement::Start | Placement::End) {
            Size::new(
                room(anchor.x - viewport.x, viewport.right() - anchor.right()),
                viewport.height,
            )
        } else {
            Size::new(
                viewport.width,
                room(anchor.y - viewport.y, viewport.bottom() - anchor.bottom()),
            )
        },
    )
}

fn reverse(align: OverlayAlign) -> OverlayAlign {
    match align {
        OverlayAlign::Start => OverlayAlign::End,
        OverlayAlign::End => OverlayAlign::Start,
        OverlayAlign::Center => OverlayAlign::Center,
    }
}

fn opposite(side: Placement) -> Placement {
    match side {
        Placement::Above => Placement::Below,
        Placement::Below => Placement::Above,
        Placement::Start => Placement::End,
        Placement::End => Placement::Start,
        Placement::Center => Placement::Center,
    }
}

struct FitGeometry {
    anchor: Rect,
    size: Size,
    viewport: Rect,
    offset: f32,
}

impl FitGeometry {
    fn candidate(&self, side: Placement, align: OverlayAlign) -> Point {
        let a = self.anchor;
        let s = self.size;
        let aligned = |start, length, size| match align {
            OverlayAlign::Start => start,
            OverlayAlign::End => start + length - size,
            OverlayAlign::Center => start + (length - size) / 2.0,
        };
        match side {
            Placement::Above | Placement::Below => Point {
                x: aligned(a.x, a.width, s.width),
                y: if side == Placement::Above {
                    a.y - self.offset - s.height
                } else {
                    a.bottom() + self.offset
                },
            },
            Placement::Start | Placement::End => Point {
                x: if side == Placement::Start {
                    a.x - self.offset - s.width
                } else {
                    a.right() + self.offset
                },
                y: aligned(a.y, a.height, s.height),
            },
            Placement::Center => Point {
                x: a.x + (a.width - s.width) / 2.0,
                y: a.y + (a.height - s.height) / 2.0,
            },
        }
    }

    fn clamp(&self, point: Point) -> Point {
        let v = self.viewport;
        let local = Point {
            x: point.x - v.x,
            y: point.y - v.y,
        };
        let p = clamp(
            local,
            self.size,
            Size {
                width: v.width,
                height: v.height,
            },
        );
        Point {
            x: p.x + v.x,
            y: p.y + v.y,
        }
    }

    fn fits(&self, p: Point) -> bool {
        let v = self.viewport;
        p.x >= v.x
            && p.y >= v.y
            && p.x + self.size.width <= v.right()
            && p.y + self.size.height <= v.bottom()
    }

    fn slide(&self, side: Placement, p: Point) -> Point {
        let clamped = self.clamp(p);
        if matches!(side, Placement::Above | Placement::Below) {
            Point { x: clamped.x, ..p }
        } else {
            Point { y: clamped.y, ..p }
        }
    }

    fn position(&self, side: Placement, align: OverlayAlign) -> Point {
        let base = self.candidate(side, align);
        if self.fits(base) || side == Placement::Center {
            return self.clamp(base);
        }
        let other = opposite(side);
        let flipped = reverse(align);
        let flips = if matches!(side, Placement::Above | Placement::Below) {
            [(other, align), (side, flipped), (other, flipped)]
        } else {
            [(side, flipped), (other, align), (other, flipped)]
        };
        let mirrored = flips
            .into_iter()
            .map(|(side, align)| self.candidate(side, align));
        let spans = [side, other].into_iter().flat_map(|side| {
            [OverlayAlign::End, OverlayAlign::Start].map(|align| self.candidate(side, align))
        });
        let slides =
            [side, other].map(|side| self.slide(side, self.candidate(side, OverlayAlign::Center)));
        mirrored
            .chain(spans.filter(|_| align == OverlayAlign::Center))
            .chain(slides)
            .find(|point| self.fits(*point))
            .unwrap_or_else(|| self.clamp(base))
    }

    fn overlaps_anchor(&self, p: Point) -> bool {
        p.x < self.anchor.right()
            && p.x + self.size.width > self.anchor.x
            && p.y < self.anchor.bottom()
            && p.y + self.size.height > self.anchor.y
    }

    fn clear_anchor(&self, side: Placement, rtl: bool) -> Option<Point> {
        let (end, start) = if rtl {
            (Placement::Start, Placement::End)
        } else {
            (Placement::End, Placement::Start)
        };
        let vertical = if side == Placement::Center {
            [Placement::Above, Placement::Below]
        } else {
            [opposite(side), side]
        };
        [end, start]
            .into_iter()
            .chain(vertical)
            .map(|side| self.slide(side, self.candidate(side, OverlayAlign::Center)))
            .find(|point| self.fits(*point) && !self.overlaps_anchor(*point))
    }
}
