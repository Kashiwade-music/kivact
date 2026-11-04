//! Role: Layout containers: column, row, grid, stack, scroll and spacer (box nodes).

use kivact_engine::api::PropValue;
use kivact_engine::contract::NodeType;
use kivact_runtime::{ErrorKind, Prop as InputProp, UiError};
use kivact_types::{Dimension, Edges, LayoutProps, Position, ScrollAxes};
use std::rc::Rc;

use super::builders::node;
use crate::state::IntoValue;
use crate::view::descriptor::Mcx;
use crate::view::descriptor::{mount, view_fn};
use crate::view::node::{Node, NodeBuilder};
use crate::view::state::ViewState;
use crate::view::{IntoView, Orientation, Role, View};
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
/// use kivact::style::{auto, style};
/// use kivact::view::{NodeBuilder, grid, span, text};
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
    container(children, style().display(kivact_types::Display::Grid))
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
/// use kivact::style::{style};
/// use kivact::view::{NodeBuilder, ScrollAxes, column, row, scroll, text};
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
        axes: InputProp::constant(ScrollAxes::Vertical),
    }
}

/// The scroll container made by [`scroll`].
pub struct Scroll {
    root: Node,
    child: View,
    axes: InputProp<ScrollAxes>,
}

impl Scroll {
    /// The axes the content scrolls along (default [`ScrollAxes::Vertical`]):
    /// `Horizontal` lays the content out in a row and scrolls sideways, `Both`
    /// scrolls both ways (a table or a code block wider and taller than the
    /// container). A reactive input updates layout and accessibility in place.
    pub fn axes(mut self, axes: impl IntoValue<ScrollAxes>) -> Self {
        self.axes = axes.into_prop();
        self
    }
}

impl NodeBuilder for Scroll {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Scroll {
    fn into_view(self) -> View {
        view_fn(move |cx| {
            // Resolve deferred parts under the mounted owner before deriving the
            // final scroll semantics, so parts receive the same conflict checks.
            let mut root = self.root.resolve_modifiers();
            let axes = self.axes;
            let style_axes = axes.clone();
            let base = InputProp::computed(Rc::new(move || {
                let axes = style_axes.get();
                let direction = match axes {
                    ScrollAxes::Horizontal => Direction::Row,
                    ScrollAxes::Vertical | ScrollAxes::Both => Direction::Column,
                };
                kivact_style::internal::with_scroll_axes(style().direction(direction), axes)
            }));
            // The first layer derives scrolling while caller style layers retain precedence.
            root.common.prepend_style(base);
            let caller = root
                .common
                .semantics
                .iter()
                .rev()
                .find_map(|patch| patch.orientation.clone());
            let at = root.at.expect("invariant: `scroll` locates its root");
            let weak = cx.weak();
            let owner = cx.owner;
            let reported = kivact_types::cell::ReportOnce::new();
            let fixed = axes.fixed().is_some()
                && caller
                    .as_ref()
                    .is_none_or(|source| source.fixed().is_some());
            let derive = move || {
                let orientation = scroll_orientation(axes.get());
                if caller
                    .as_ref()
                    .is_some_and(|source| Some(source.get()) != orientation)
                    && reported.note()
                    && let Some(rt) = weak.upgrade()
                {
                    rt.report(Some(owner), UiError::new(
                        ErrorKind::Unsupported,
                        "`.orientation(..)` on a scroll container disagrees with `.axes(..)`: its accessibility orientation follows the axes (Vertical: vertical, Horizontal: horizontal, Both: none); ignored",
                        Some(at),
                    ));
                }
                orientation.unwrap_or_default()
            };
            let orientation = if fixed {
                InputProp::constant(derive())
            } else {
                InputProp::computed(Rc::new(derive))
            };
            root.common.semantic_orientation(orientation);
            mount(root.child(self.child).into_view(), cx)
        })
    }
}

fn scroll_orientation(axes: ScrollAxes) -> Option<Orientation> {
    match axes {
        ScrollAxes::Vertical => Some(Orientation::Vertical),
        ScrollAxes::Horizontal => Some(Orientation::Horizontal),
        ScrollAxes::Both => None,
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
        let reader = ViewState::tree(cx.rt).reader();
        let transparent = reader.with_node(frag, |node| {
            node.is_some_and(|node| {
                node.layout
                    .as_ref()
                    .is_some_and(|layout| layout.display == kivact_types::Display::Contents)
            })
        });
        let items = if transparent {
            reader.children(frag)
        } else {
            vec![frag]
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
