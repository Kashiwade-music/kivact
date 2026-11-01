//! Role: Optional vertical resizing around an ordinary public text field.

use kivact::control::{Node, NodeBuilder, node};
use kivact::draw::Point;
use kivact::i18n::message;
use kivact::interaction::use_splitter;
use kivact::signal;
use kivact::style::{ColorToken, Cursor, Position, pct, style};
use kivact::ui::ScalarRange;
use kivact::view::{Handled, MouseButton, Orientation, PointerPhase, node_ref};

/// The tallest a resizable field becomes, in logical pixels. A minimum above
/// this raises the maximum to the minimum.
const MAX_HEIGHT: f64 = 4000.0;

/// Wrap a configured text field with a lower-right grip that resizes its
/// height.
///
/// Only the height changes, the way a `textarea`'s `resize: vertical` does:
/// the width is the form layout's, which wraps the text, so the resizing has
/// no orientation to choose.
///
/// Configure bindings, placeholder, validation and editing callbacks on `field`
/// before wrapping it. The field stays mounted while its height changes, so its
/// value, selection and editing focus survive pointer resizing. The wrapper's
/// width is controlled by normal node modifiers and never changes during a drag.
/// `text_area` itself retains its fixed-height default.
///
/// Heights are logical pixels between the minimum and 4000. Invalid minimums
/// fall back to 72; invalid initial heights fall back to 80. The initial height
/// and every resize stay within that range.
/// The grip is a [`use_splitter`](kivact::interaction::use_splitter) handle over stacked panes: ArrowUp/ArrowDown change the
/// height by 8, Home and End jump to the limits, and accessibility value actions
/// and a drag follow the same range. A press on the grip leaves the focus in
/// the field so typing continues, so Escape does not cancel a drag.
#[track_caller]
pub fn resizable_text_area(
    field: kivact::ui::TextInput,
    initial_height: f64,
    min_height: f64,
) -> Node {
    let min = if min_height.is_finite() && min_height > 0.0 {
        min_height
    } else {
        72.0
    };
    let max = MAX_HEIGHT.max(min);
    let initial = if initial_height.is_finite() {
        initial_height.max(min)
    } else {
        80.0f64.max(min)
    }
    .min(max);
    let height = signal(initial);
    let editor = node_ref();
    let range = ScalarRange::new(min, max).expect("invariant: finite ordered height range");
    let grip = node()
        .name(message("pop.resize.name", "Resize text area"))
        .description(message(
            "pop.resize.description",
            "Drag vertically, or use Up and Down to change the height.",
        ))
        .style(
            style()
                .position(Position::Absolute)
                .right(2)
                .bottom(2)
                .width(14)
                .height(14)
                .cursor(Cursor::NsResize),
        )
        .on_pointer(move |event| {
            if event.phase == PointerPhase::Down && event.button == Some(MouseButton::Primary) {
                editor.focus();
            }
            Handled::No
        })
        .draw(|draw, _| {
            let color = draw.theme().colors().get(ColorToken::TextMuted);
            for offset in [4.0, 7.0, 10.0] {
                draw.line(
                    Point::new(offset, 12.0),
                    Point::new(12.0, offset),
                    1.0,
                    color,
                );
            }
        })
        .part(
            use_splitter(height, range, Orientation::Vertical)
                .key_step(8.0)
                .expect("invariant: positive finite key step")
                .handle(),
        );
    node()
        .style(move || {
            style()
                .position(Position::Relative)
                .width(pct(100))
                .height(height.get())
                .min_height(min)
                .shrink(0)
        })
        .child(
            field
                .node_ref(editor)
                .style(style().width(pct(100)).height(pct(100))),
        )
        .child(grip)
}
