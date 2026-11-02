//! Role: Layout containers: column, row, grid, stack, scroll and spacer (box nodes).

use kivact_engine::bridge::PropValue;
use kivact_reactive::{ErrorKind, UiError};
use kivact_wire::{Dimension, Edges, LayoutProps, NodeType, Position, ScrollAxes};

use super::node::node;
use crate::view::descriptor::Mcx;
use crate::view::descriptor::{mount, view_fn};
use crate::view::node::{Node, NodeBuilder};
use crate::view::state::ViewState;
use crate::view::{IntoValue, IntoView, Orientation, Role, View};
use kivact_style::{Direction, Style, style};

/// A box with `children` and the base style `s`, set before the user's
/// modifiers so a later `.style(..)` layers on top. The box's location is
/// the caller of the public builder.
#[track_caller]
fn container(children: impl IntoView, s: Style) -> Node {
    node().child(children).style(s)
}

/// Vertical flex container. Children: a tuple, a `Vec`, or any view.
#[track_caller]
pub fn column(children: impl IntoView) -> Node {
    container(children, style().direction(Direction::Column))
}

/// Horizontal flex container.
#[track_caller]
pub fn row(children: impl IntoView) -> Node {
    container(children, style().direction(Direction::Row))
}

/// A grid container (CSS `display: grid`): its children fill the cells of
/// the tracks its style sets (`grid_template_columns`, `grid_template_rows`;
/// without them, one column), placed in order or where their
/// `grid_column` and `grid_row` say.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::prelude::*;
/// use kivact::style::{GridPlacement, Track};
/// // .cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(330px, 1fr)); gap: 16px }
/// let cards = grid((text("A"), text("B").style(style().grid_column(GridPlacement::span(2)))))
///     .style(
///         style()
///             .grid_template_columns(Track::repeat_auto_fill(Track::minmax(330, Track::fr(1))))
///             .gap(16),
///     );
/// # let _ = cards;
/// ```
#[track_caller]
pub fn grid(children: impl IntoView) -> Node {
    container(children, style().display_grid())
}

/// A scroll container. Wheel at the edge of a short area passes to the parent.
///
/// It scrolls vertically; [`axes`](Scroll::axes) scrolls horizontally or both
/// ways. Its accessibility orientation follows the axes (vertical,
/// horizontal, none for both); an `.orientation(..)` on it that disagrees is
/// reported and ignored. The modifiers of [`NodeBuilder`] apply to the
/// container.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::prelude::*;
///
/// let list = scroll(column((text("A"), text("B")))).style(style().height(120));
/// let strip = scroll(row((text("A"), text("B")))).axes(ScrollAxes::Horizontal);
/// # let _ = (list, strip);
/// ```
#[track_caller]
pub fn scroll(child: impl IntoView) -> Scroll {
    Scroll {
        root: node().role(Role::ScrollView),
        child: child.into_view(),
        axes: ScrollAxes::Vertical,
    }
}

/// The scroll container made by [`scroll`].
pub struct Scroll {
    root: Node,
    child: View,
    axes: ScrollAxes,
}

impl Scroll {
    /// The axes the content scrolls along (default [`ScrollAxes::Vertical`]):
    /// `Horizontal` lays the content out in a row and scrolls sideways, `Both`
    /// scrolls both ways (a table or a code block wider and taller than the
    /// container). A constant: fixed when the container is built.
    pub fn axes(mut self, axes: ScrollAxes) -> Self {
        self.axes = axes;
        self
    }

    /// Accessibility role, overriding the container's default role (a list
    /// or a log that scrolls).
    ///
    /// Same as [`NodeExt::role`](crate::primitive::NodeExt::role).
    pub fn role(mut self, r: crate::view::Role) -> Self {
        self.root = self.root.role(r);
        self
    }
}

impl NodeBuilder for Scroll {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Scroll {
    fn into_view(mut self) -> View {
        let direction = match self.axes {
            ScrollAxes::Horizontal => Direction::Row,
            ScrollAxes::Vertical | ScrollAxes::Both => Direction::Column,
        };
        let base = style().direction(direction).with_scroll_axes(self.axes);
        // The base style is the first layer, so the caller's `.style(..)`
        // wins whatever the order of the calls.
        self.root.common.prepend_style(base.into_prop());
        let derived = match self.axes {
            ScrollAxes::Vertical => Some(Orientation::Vertical),
            ScrollAxes::Horizontal => Some(Orientation::Horizontal),
            ScrollAxes::Both => None,
        };
        // The caller's `.orientation(..)` (NodeExt) is taken out and replaced
        // by the axes' value.
        let caller = self
            .root
            .common
            .semantics
            .as_mut()
            .and_then(|sources| sources.orientation.take());
        if let Some(orientation) = derived {
            self.root.common.semantics_mut().orientation = Some(orientation.into_prop());
        }
        let at = self
            .root
            .at
            .expect("invariant: `scroll` is #[track_caller] and locates its root");
        let view = self.root.child(self.child).into_view();
        let Some(caller) = caller else {
            return view;
        };
        view_fn(move |cx| {
            // A value that agrees with the axes (a list part names its own
            // orientation) is redundant; one that disagrees is reported.
            if Some(kivact_reactive::untrack(|| caller.get())) != derived {
                cx.rt.report(
                    Some(cx.owner),
                    UiError::new(
                        ErrorKind::Unsupported,
                        "`.orientation(..)` on a scroll container disagrees with `.axes(..)`: its accessibility orientation follows the axes                          (Vertical: vertical, Horizontal: horizontal, Both: none); ignored",
                        Some(at),
                    ),
                );
            }
            mount(view, cx)
        })
    }
}

/// Flexible empty space.
#[track_caller]
pub fn spacer() -> Node {
    node().style(style().grow(1))
}

/// Layers children on top of each other: the first child sizes the stack,
/// later children fill it.
#[track_caller]
pub fn stack(children: impl IntoView) -> Node {
    let children = children.into_view();
    let layered = view_fn(move |cx: &Mcx<'_>| {
        let frag = mount(children, cx);
        let items: Vec<_> = ViewState::tree(cx.rt)
            .get(frag)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        let items = if items.is_empty()
            && ViewState::tree(cx.rt)
                .get(frag)
                .is_some_and(|n| n.is_host())
        {
            vec![frag]
        } else {
            items
        };
        let wrap = cx.fragment();
        // The children's own fragment loses its children to the layers; it
        // stays in the tree (under `wrap`, empty) instead of as a detached
        // record nothing reaches or deletes.
        let leftover = (!items.contains(&frag)).then_some(frag);
        for (i, c) in items.into_iter().enumerate() {
            let w = cx.host(NodeType::Box);
            let layout = if i == 0 {
                LayoutProps::default()
            } else {
                LayoutProps {
                    position: Position::Absolute,
                    inset: Edges::all(Dimension::Px(0.0)),
                    ..LayoutProps::default()
                }
            };
            cx.update(w, PropValue::Layout(layout));
            {
                let mut tree = ViewState::tree_mut(cx.rt);
                if let Some(p) = tree.get(c).and_then(|n| n.parent)
                    && let Some(pn) = tree.get_mut(p)
                {
                    pn.children.retain(|x| *x != c);
                }
                if let Some(n) = tree.get_mut(c) {
                    n.parent = None;
                }
            }
            cx.append(w, c);
            cx.append(wrap, w);
        }
        if let Some(frag) = leftover {
            cx.append(wrap, frag);
        }
        wrap
    });
    container(layered, style().position(kivact_style::Position::Relative))
}
