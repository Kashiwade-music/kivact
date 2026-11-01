//! Role: Tests for the IconButton component.
//!
//! `IconButton`, the part with an icon:
//! a third-party part whose face is a decorative monochrome image, built from the
//! public API of `kivact` only (this crate depends on
//! nothing else, so compiling it is the first check).
//!
//! The icon's tint (the button's text color, its hover and pressed colors)
//! is drawn by the host: the primitive image tests check it in pixels and
//! the GUI harness with OS input. Here: what assistive technology is given, the layout
//! of the face, and that the button works with any `Asset` source.

use kivact::Asset;
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{SemanticNode, render};
use kivact_external_component::parts::{CROSS_ICON, SEARCH_ICON, icon_button};

/// The node with test id `id` under `n`.
fn by_test_id<'a>(n: &'a SemanticNode, id: &str) -> Option<&'a SemanticNode> {
    if n.test_id.as_deref() == Some(id) {
        return Some(n);
    }
    n.children.iter().find_map(|c| by_test_id(c, id))
}

/// Every node under `n` as `(role, name)`, depth first.
fn nodes(n: &SemanticNode, out: &mut Vec<(Option<Role>, String)>) {
    out.push((n.role, n.name.clone()));
    for c in &n.children {
        nodes(c, out);
    }
}

/// What the hosts expose of the nodes under `n`: a node with a role,
/// except `Generic` and an `Image` without a name (the shared semantics
/// policy). The trees here hold no text node (exposed by its kind).
fn exposed(n: &SemanticNode) -> Vec<(Role, String)> {
    let mut all = Vec::new();
    nodes(n, &mut all);
    all.into_iter()
        .filter_map(|(role, name)| match role {
            None | Some(Role::Generic) => None,
            Some(Role::Image) if name.is_empty() => None,
            Some(r) => Some((r, name)),
        })
        .collect()
}

#[test]
fn only_the_buttons_name_reaches_assistive_technology() {
    let screen = render(|| {
        let presses = signal(0);
        column((
            icon_button("Close", CROSS_ICON)
                .test_id("close")
                .on_press(move || presses.update(|n| *n += 1)),
            text(move || format!("pressed {}", presses.get())),
        ))
    });
    let snap = screen.snapshot();
    let close = by_test_id(&snap, "close").expect("the button");
    let mut tree = Vec::new();
    nodes(close, &mut tree);
    assert_eq!(
        tree,
        vec![
            (Some(Role::Button), "Close".to_string()),
            (Some(Role::Generic), String::new()),
        ],
        "the button and its unnamed decorative primitive image"
    );
    assert_eq!(exposed(close), vec![(Role::Button, "Close".to_string())]);
    screen.get_by_role(Role::Button, "Close").click();
    screen.get_by_text("pressed 1");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_icon_is_centered_in_the_square_button() {
    let screen = render(|| {
        row((icon_button("Close", CROSS_ICON).test_id("close"),))
            .style(style().align_items(Align::Start))
    });
    let button = screen.get_by_test_id("close");
    // The face is decorative (not exposed): found by structure.
    let face = button.children()[0];
    assert_eq!(face.role(), None);
    let (b, f) = (
        button.rect().expect("laid out"),
        face.rect().expect("laid out"),
    );
    let side = Theme::light().control_height() - 8.0;
    assert_eq!((b.width, b.height), (side, side), "a square button");
    assert_eq!((f.width, f.height), (16.0, 16.0), "the icon's default size");
    assert_eq!(
        (f.x - b.x, f.y - b.y),
        ((side - 16.0) / 2.0, (side - 16.0) / 2.0),
        "centered"
    );
}

#[test]
fn the_face_can_change_and_a_disabled_button_does_not_press() {
    let screen = render(|| {
        let searching = signal(false);
        let presses = signal(0);
        column((
            icon_button("Toggle search", move || {
                Asset::from(if searching.get() {
                    CROSS_ICON
                } else {
                    SEARCH_ICON
                })
            })
            .on_press(move || searching.update(|s| *s = !*s)),
            icon_button("Disabled", CROSS_ICON)
                .disabled(true)
                .on_press(move || presses.update(|n| *n += 1)),
            text(move || format!("{} {}", searching.get(), presses.get())),
        ))
    });
    screen.get_by_role(Role::Button, "Toggle search").click();
    screen.get_by_text("true 0");
    // A press on the disabled button (a checked `click` would refuse it).
    screen.get_by_role(Role::Button, "Disabled").force_click();
    screen.get_by_text("true 0");
    assert!(screen.get_by_role(Role::Button, "Disabled").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
