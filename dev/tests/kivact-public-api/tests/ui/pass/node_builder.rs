#![allow(dead_code, unused_variables)]
// Writing a component library: an external builder that
// holds a root `Node` and implements `NodeBuilder` gets every standard
// modifier from `NodeBuilder` and `NodeExt`, using only the prelude (which
// names `Node` and `NodeBuilder`) and `kivact::primitive::NodeExt`. Every
// `NodeBuilder` and `NodeExt` method is called once.
use kivact::style::current_theme;
use std::panic::Location;

use kivact::prelude::*;
use kivact::draw::{IntrinsicSize, Size};
use kivact::view::{A11yAction, Handled, View};
use kivact::primitive::NodeExt;

pub struct Card {
    root: Node,
}

#[track_caller]
pub fn card(title: impl IntoText) -> Card {
    let title = title.into_text_value();
    Card {
        root: node()
            .role(Role::Group)
            .name(title)
            .style(move || { let t = &current_theme(); style().padding(t.space().md) })
            .child(text(title)),
    }
}

impl Card {
    /// A component's own method next to the standard ones.
    pub fn on_open<F: Fn() + 'static>(self, f: F) -> Self {
        self.on_a11y(A11yAction::Expand, f)
    }
}

impl NodeBuilder for Card {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Card {
    fn into_view(self) -> View {
        self.root.into_view()
    }
}

/// A helper bounded by `NodeBuilder` accepts a component and a plain `Node`.
fn framed<B: NodeBuilder>(b: B) -> B {
    b.style(move || { let t = &current_theme(); style().radius(t.radius().sm) })
}

fn every_modifier() -> impl IntoView {
    let r = node_ref();
    let busy = signal(false);
    let c = card("Profile")
        .at(Location::caller())
        .test_id("card")
        .node_ref(r)
        .style(style().margin(4))
        .style(move || { let t = &current_theme(); style().gap(t.space().sm) })
        .role(Role::Dialog)
        .name("Profile card")
        .description(move || format!("busy: {}", busy.get()))
        .value_text("3 items")
        .selected(busy)
        .disabled(false)
        .hidden(false)
        .focusable(true)
        .on_key(|e| if e.key == Key::Escape { Handled::Yes } else { Handled::No })
        .on_pointer(|e| {})
        .on_wheel(|e| Handled::No)
        .on_press(|| {})
        .draw(|d, size| {})
        .draw_over(|d, size| {})
        .intrinsic_size(IntrinsicSize {
            min: None,
            preferred: Some(Size::new(10.0, 10.0)),
            max: None,
        })
        .on_open(|| {});
    column((framed(c), framed(node())))
}

fn main() {
    let _ = || App::new("NodeBuilder").run(every_modifier);
}
