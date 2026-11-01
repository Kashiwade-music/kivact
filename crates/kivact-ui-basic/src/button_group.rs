//! Role: The Lyra button group: joined controls, a group separator and a text cell.

use crate::children::GroupItems;
use crate::theme_access::tokens;
use kivact::control::{Node, NodeBuilder, node, row, text};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Color, style};
use kivact::view::{IntoText, IntoView, Orientation, Role, View, provide, try_context};
use kivact::view::{ReadingDirection, current_direction};

/// Controls joined into one bar (button-group.tsx). Created by [`button_group`].
pub struct ButtonGroup {
    root: Node,
    /// The reading direction when the group was made.
    rtl: bool,
    /// The children, made once the axis is known.
    children: Box<dyn FnOnce(bool) -> Vec<View>>,
    orientation: Orientation,
}

/// The axis a [`button_group`] gives the separators inside it: `true` when
/// the group is vertical.
#[derive(Clone, Copy)]
struct GroupAxis(bool);

/// A group of `children` whose neighbouring borders collapse into one line.
///
/// As button-group.tsx does, every child but the first drops the border it
/// shares with the one before it (`border-l-0`, `border-t-0` when vertical),
/// so each line between two controls is drawn once, like the outer edges. In
/// a right-to-left [`direction`](kivact::view::provide_direction) the row
/// reverses and the dropped border is the right one. The children stretch
/// across the group (`items-stretch`), so a [`button_group_text`] is as high
/// as the buttons beside it.
///
/// Unlike the other containers, which take one `impl IntoView`, this takes
/// the children one by one, because it styles each: a tuple of builders
/// (`(button(..), button_group_separator(), button_group_text(..))`) or a
/// `Vec` of one kind of builder.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn actions() -> impl IntoView {
///     basic::button_group((
///         button("Archive").variant(kivact::ui::ButtonVariant::Outline),
///         button("Report").variant(kivact::ui::ButtonVariant::Outline),
///     ))
/// }
/// ```
#[track_caller]
pub fn button_group(children: impl GroupItems) -> ButtonGroup {
    let rtl = current_direction() == ReadingDirection::Rtl;
    ButtonGroup {
        root: node().role(Role::Group),
        rtl,
        children: Box::new(move |vertical| {
            // A layer after each child's own, so it wins over its border.
            children.into_styled_views(|index| {
                let s = style().shrink(0);
                match (index, vertical, rtl) {
                    (0, _, _) => s,
                    (_, true, _) => s.border_top(0, Color::TRANSPARENT),
                    (_, false, false) => s.border_left(0, Color::TRANSPARENT),
                    (_, false, true) => s.border_right(0, Color::TRANSPARENT),
                }
            })
        }),
        orientation: Orientation::Horizontal,
    }
}

impl ButtonGroup {
    /// Choose the axis: [`Orientation::Horizontal`] side by side,
    /// [`Orientation::Vertical`] stacked (`Unspecified` is horizontal). A
    /// constant: the children's shared borders and the separators' axis are
    /// set from it when the group is built.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

impl NodeBuilder for ButtonGroup {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for ButtonGroup {
    fn into_view(self) -> View {
        let vertical = self.orientation == Orientation::Vertical;
        let rtl = self.rtl;
        let axis = match (vertical, rtl) {
            (true, _) => kivact::style::Direction::Column,
            (false, false) => kivact::style::Direction::Row,
            (false, true) => kivact::style::Direction::RowReverse,
        };
        let children = (self.children)(vertical);
        // The separators among the children read the group's axis from here.
        self.root
            .base_style(
                style()
                    .direction(axis)
                    .align_items(Align::Stretch)
                    .radius(0),
            )
            .child(provide(GroupAxis(vertical), move || children))
            .into_view()
    }
}

/// A line between two controls of a [`button_group`] (button-group.tsx
/// separator): 1 px of `--input`.
///
/// It runs across the group, one pixel short of each end (`my-px`, `mx-px`):
/// a vertical line in a horizontal group, a horizontal line in a vertical
/// one. Outside a group it is vertical.
#[track_caller]
pub fn button_group_separator() -> Node {
    let across = || try_context::<GroupAxis>().is_some_and(|GroupAxis(vertical)| vertical);
    node()
        .role(Role::Separator)
        .orientation(move || {
            if across() {
                Orientation::Horizontal
            } else {
                Orientation::Vertical
            }
        })
        .style(move || {
            let s = style()
                .shrink(0)
                .align_self(Align::Stretch)
                .background(tokens().input);
            if across() {
                s.height(1).margin_x(1)
            } else {
                s.width(1).margin_y(1)
            }
        })
}

/// A text cell of a [`button_group`] (`ButtonGroupText`): a prefix or a
/// suffix that is not a control, such as the `https://` before a URL field or
/// a unit after a number. Muted fill, border, 12 px medium text, centered on
/// the group's height (the group stretches it to its neighbours' height).
#[track_caller]
pub fn button_group_text(text_value: impl IntoText) -> Node {
    row(text(text_value)).style(
        style()
            .align_items(Align::Center)
            .gap(8)
            .padding_x(10)
            .border(1, crate::Color::Border)
            .radius(0)
            .background(crate::Color::Muted)
            .color(crate::Color::Foreground)
            .font_size(12)
            .weight(500)
            .text_wrap(false),
    )
}
