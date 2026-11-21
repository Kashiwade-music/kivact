//! Role: The style of a resizable sidebar's width and the handle that drags it.

use super::look::{SidebarState, SidebarTheme};
use super::recipes::themed;
use super::state::{SidebarCx, SidebarResize, SidebarSide};
use kivact_core::i18n::message;
use kivact_core::state::Value;
use kivact_core::style::{Style, style};
use kivact_core::view::{Node, Orientation, node};
use kivact_interaction::{ScalarRange, use_splitter};

/// The width of an open resizable sidebar: its signal, kept between the bounds.
pub(super) fn open_width(cx: SidebarCx, resize: SidebarResize) -> Style {
    if cx.open.get() {
        style().width(resize.width.get().clamp(resize.min, resize.max))
    } else {
        style()
    }
}

/// The handle on the sidebar's inner edge, named "Resize sidebar". It is a
/// splitter: arrow keys move it 8 px, and it is hidden while the sidebar is closed.
///
/// The splitter moves the size of its leading pane. A sidebar on the left
/// leads with its width. A sidebar on the right trails the page, so its
/// pointer travel is negated to grow it when the handle moves left.
pub(super) fn resize_handle(
    cx: SidebarCx,
    resize: SidebarResize,
    state: Value<SidebarState>,
) -> Node {
    let range = ScalarRange::new(resize.min, resize.max)
        .expect("invariant: a resizable sidebar's bounds are finite and ordered");
    let scale = match cx.side {
        SidebarSide::Left => 1.0,
        SidebarSide::Right => -1.0,
    };
    let grip = node()
        .part(
            use_splitter(resize.width, range, Orientation::Horizontal)
                .pointer_scale(scale)
                .key_step(8.0)
                .expect("invariant: the sidebar's keyboard step is positive")
                .handle(),
        )
        .name(message("ui.sidebar.resize", "Resize sidebar"))
        .hidden(move || !cx.open.get());
    themed(grip, |r: &SidebarTheme| r.resize_handle_recipe(), state)
}
