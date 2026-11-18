//! Role: Rounds unrounded layouts to whole pixels, and marks the columns whose layout changed.

use crate::LayoutTree;
use taffy::Layout;

/// Record that column `index` got a new unrounded layout in pass `stamp`,
/// and that every ancestor (up to the viewport, column 0) has such a node below it.
pub(super) fn touch(
    tree: &(impl LayoutTree + ?Sized),
    touched: &mut [u32],
    below: &mut [u32],
    stamp: u32,
    index: usize,
) {
    touched[index] = stamp;
    let mut at = Some(index);
    while let Some(i) = at {
        if below[i] == stamp {
            break;
        }
        below[i] = stamp;
        at = if i == 0 {
            None
        } else {
            Some(tree.parent(i - 1).map_or(0, |p| p + 1))
        };
    }
}

/// The rounded copy of `unrounded`, whose parent's cumulative origin is `cum`
/// (the `layout` part of taffy 0.14's `round_layout_inner`, and its
/// `round_scrollable_overflow_rect`: the same arithmetic in the same order,
/// with taffy's `round`). Taffy 0.15 rounds a location from the cumulative
/// position instead; this crate keeps the parent-relative rounding, so the
/// upgrade moves no rounded box.
pub(super) fn rounded(unrounded: Layout, cum: (f32, f32)) -> Layout {
    let mut layout = unrounded;
    let x = cum.0 + unrounded.location.x;
    let y = cum.1 + unrounded.location.y;
    layout.location.x = round(unrounded.location.x);
    layout.location.y = round(unrounded.location.y);
    layout.size.width = round(x + unrounded.size.width) - round(x);
    layout.size.height = round(y + unrounded.size.height) - round(y);
    layout.scrollbar_size.width = round(unrounded.scrollbar_size.width);
    layout.scrollbar_size.height = round(unrounded.scrollbar_size.height);
    layout.border.left = round(x + unrounded.border.left) - round(x);
    layout.border.right =
        round(x + unrounded.size.width) - round(x + unrounded.size.width - unrounded.border.right);
    layout.border.top = round(y + unrounded.border.top) - round(y);
    layout.border.bottom = round(y + unrounded.size.height)
        - round(y + unrounded.size.height - unrounded.border.bottom);
    layout.padding.left = round(x + unrounded.padding.left) - round(x);
    layout.padding.right =
        round(x + unrounded.size.width) - round(x + unrounded.size.width - unrounded.padding.right);
    layout.padding.top = round(y + unrounded.padding.top) - round(y);
    layout.padding.bottom = round(y + unrounded.size.height)
        - round(y + unrounded.size.height - unrounded.padding.bottom);
    let overflow = unrounded.scrollable_overflow_rect;
    layout.scrollable_overflow_rect.left = round(x + overflow.left) - round(x);
    layout.scrollable_overflow_rect.right = round(x + overflow.right) - round(x);
    layout.scrollable_overflow_rect.top = round(y + overflow.top) - round(y);
    layout.scrollable_overflow_rect.bottom = round(y + overflow.bottom) - round(y);
    layout
}

/// Taffy's rounding (`(value + 0.5).floor()`, not `f32::round`): halves
/// round toward positive infinity, also below zero.
#[inline(always)]
fn round(value: f32) -> f32 {
    (value + 0.5).floor()
}
