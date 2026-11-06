//! Role: Observe per-axis scroll geometry and fit item rows into an available width.
use crate::scrollable_area::ScrollAxis;
use crate::stack::rtl;

use kivact::state::Value;
use kivact::state::memo;
use kivact::view::NodeRef;

// xtask-allow(state-fields): Independent read-only projections of one axis geometry, not a hidden state machine.
/// Observed state of one logical scroll axis.
#[derive(Clone, Copy)]
pub struct AxisScroll {
    /// The content is more than one pixel larger than the viewport on this axis.
    pub is_scrollable: Value<bool>,
    /// The viewport rests at the logical start (also `true` when not scrollable).
    pub at_start: Value<bool>,
    /// The viewport rests at the logical end (also `true` when not scrollable).
    pub at_end: Value<bool>,
}

/// Observed geometry of a viewport and its content box on both logical axes.
#[derive(Clone, Copy)]
pub struct ScrollState {
    /// Horizontal axis, mirrored in right-to-left layouts.
    pub inline: AxisScroll,
    /// Vertical axis.
    pub block: AxisScroll,
}

/// Observe the scroll extent of `viewport` around `content` on the axes that `axis` enables.
///
/// Disabled axes report not scrollable and rest at both ends. Setup only (it creates memos).
pub fn use_scroll_state(viewport: NodeRef, content: NodeRef, axis: ScrollAxis) -> ScrollState {
    ScrollState {
        inline: observe_axis(viewport, content, true, axis != ScrollAxis::Block),
        block: observe_axis(viewport, content, false, axis != ScrollAxis::Inline),
    }
}

fn observe_axis(viewport: NodeRef, content: NodeRef, inline: bool, enabled: bool) -> AxisScroll {
    let outer = viewport.rect();
    let inner = content.rect();
    let offset = viewport.scroll_offset();
    let extent = move || {
        let (Some(outer), Some(inner)) = (outer.get(), inner.get()) else {
            return 0.0;
        };
        if inline {
            (inner.width - outer.width).max(0.0)
        } else {
            (inner.height - outer.height).max(0.0)
        }
    };
    let at_boundary = move |start: bool| {
        let extent = extent();
        if !enabled || extent <= 1.0 {
            return true;
        }
        let offset = offset.get();
        let value = if inline { offset.x } else { offset.y };
        let physical_start = if inline && rtl() { !start } else { start };
        if physical_start {
            value <= 1.0
        } else {
            value >= extent - 1.0
        }
    };
    AxisScroll {
        is_scrollable: memo(move || enabled && extent() > 1.0),
        at_start: memo(move || at_boundary(true)),
        at_end: memo(move || at_boundary(false)),
    }
}

// xtask-allow(state-fields): Independent read-only projections of inline overflow, not a hidden state machine.
/// Hidden content on either inline side of a horizontally scrolling viewport.
#[derive(Clone, Copy)]
pub struct ScrollOverflow {
    /// Content is hidden before the logical start.
    pub start: Value<bool>,
    /// Content is hidden after the logical end.
    pub end: Value<bool>,
    /// The content overflows the viewport inline.
    pub any: Value<bool>,
}

/// Observe inline overflow of `viewport` around `content`, for edge fades and scroll buttons.
/// Setup only (it creates memos).
pub fn use_scroll_overflow(viewport: NodeRef, content: NodeRef) -> ScrollOverflow {
    let axis = use_scroll_state(viewport, content, ScrollAxis::Inline).inline;
    ScrollOverflow {
        start: memo(move || !axis.at_start.get()),
        end: memo(move || !axis.at_end.get()),
        any: axis.is_scrollable,
    }
}

/// Spacing and collapse rules for [`use_overflow`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverflowOptions {
    /// Gap between items, and between the last visible item and the indicator.
    pub gap: f64,
    /// Items that stay visible even when they do not fit.
    pub min_visible_items: usize,
    /// Hide items from the start instead of the end.
    pub collapse_from_start: bool,
}

/// Count the items of `widths` that fit in `available`, reserving room for an overflow
/// indicator of width `indicator` whenever some items are hidden. Setup only (it creates a memo).
pub fn use_overflow(
    widths: Value<Vec<f64>>,
    available: Value<f64>,
    indicator: Value<f64>,
    options: OverflowOptions,
) -> Value<usize> {
    memo(move || fitting_items(&widths.get(), available.get(), indicator.get(), options))
}

fn fitting_items(
    widths: &[f64],
    available: f64,
    indicator: f64,
    options: OverflowOptions,
) -> usize {
    let total = widths.len();
    let mut taken = 0.0;
    for index in 0..total {
        let width = widths[if options.collapse_from_start {
            total - index - 1
        } else {
            index
        }];
        taken += width + if index == 0 { 0.0 } else { options.gap };
        let reserve = if index + 1 < total {
            options.gap + indicator
        } else {
            0.0
        };
        if index >= options.min_visible_items && taken + reserve > available {
            return index;
        }
    }
    total
}
