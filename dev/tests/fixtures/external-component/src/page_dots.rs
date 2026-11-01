//! Role: Page dots expose carousel selection through a measured primitive.
//! A primitive of this crate's own (`kivact::primitive`,
//! experimental), written against the public API alone: no composition of
//! boxes, no core fork.
//!
//! - Measure: as wide as its pages (16 px each), 16 px high; a new page
//!   count measures it again, a new current page only draws again.
//! - Setup: keyed region Parts expose `Tab` leaves named "Page n".
//! - Draw: the dots at the laid-out size and each page's region geometry.
//! - Input: a press on a dot shows its page (`PointerEvent::region()`),
//!   ArrowLeft / ArrowRight move one page, Home / End to the ends.
//! - Accessibility: a focusable `TabList` valued "Page n of m".
//!
//! Invariant: the current page is one [`Binding`] (the caller's); it is
//! shown clamped to the pages there are.

use kivact::control::{Node, NodeBuilder};
use kivact::draw::{Rect, RegionId, Size};
use kivact::primitive::NodeExt;
use kivact::primitive::{Constraints, Primitive, primitive};
use kivact::style::ColorToken;
use kivact::view::{
    Handled, IntoValue, IntoView, Key, KeyEvent, PointerEvent, PointerPhase, Role, View,
};
use kivact::{Binding, Value};

/// The width of one page's cell (and the height of the strip).
const CELL: f32 = 16.0;
/// The diameter of a dot.
const DOT: f32 = 8.0;

/// Page dots. Built by [`page_dots`]; the standard modifiers come from
/// `NodeBuilder` and apply to its node.
pub struct PageDots {
    root: Primitive,
    count: Value<usize>,
    page: Binding<usize>,
}

/// Dots for `count` pages (a constant, a signal or a closure) showing the
/// page of `page` (0-based), which a press, the arrow keys, Home and End
/// change.
#[track_caller]
pub fn page_dots(count: impl IntoValue<usize>, page: impl Into<Binding<usize>>) -> PageDots {
    let count = count.into_value();
    PageDots {
        root: primitive()
            .measure(count, |n: &usize, _: Constraints| {
                Size::new(*n as f32 * CELL, CELL)
            })
            .role(Role::TabList)
            .focusable(true),
        count,
        page: page.into(),
    }
}

impl NodeBuilder for PageDots {
    fn node_mut(&mut self) -> &mut Node {
        self.root.node_mut()
    }
}

impl IntoView for PageDots {
    fn into_view(self) -> View {
        let PageDots {
            mut root,
            count,
            page,
        } = self;
        let current = page.into_value();
        let shown = move || current.get().min(count.get().saturating_sub(1));
        let drawing = std::mem::replace(root.node_mut(), kivact::control::node());
        *root.node_mut() = drawing.regions(
            move || (0..count.get()).collect::<Vec<_>>(),
            |index| RegionId(*index as u64),
            |_, index| {
                kivact::primitive::part(|state| state)
                    .role(Role::Tab)
                    .name(move || format!("Page {}", index.get() + 1))
            },
        );
        root.value_text(move || format!("Page {} of {}", shown() + 1, count.get()))
            .draw(move |d, size: Size| {
                let y = (size.height - DOT) / 2.0;
                let at = shown();
                for i in 0..count.get() {
                    let x = i as f32 * CELL;
                    let color = if i == at {
                        ColorToken::Accent
                    } else {
                        ColorToken::Border
                    };
                    d.rounded_rect(
                        Rect::new(x + (CELL - DOT) / 2.0, y, DOT, DOT),
                        DOT / 2.0,
                        color,
                    );
                    d.region(RegionId(i as u64), Rect::new(x, 0.0, CELL, size.height));
                }
            })
            .on_pointer(move |e: PointerEvent| match (e.phase, e.region()) {
                (PointerPhase::Down, Some(RegionId(i))) => {
                    page.set(i as usize);
                    Handled::Yes
                }
                _ => Handled::No,
            })
            .on_key(move |e: KeyEvent| {
                let (at, last) = (
                    current.get_untracked(),
                    count.get_untracked().saturating_sub(1),
                );
                let next = match e.key {
                    Key::ArrowRight => (at + 1).min(last),
                    Key::ArrowLeft => at.saturating_sub(1),
                    Key::Home => 0,
                    Key::End => last,
                    _ => return Handled::No,
                };
                page.set(next);
                Handled::Yes
            })
            .into_view()
    }
}
