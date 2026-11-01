//! Role: Image primitives preserve sizing, decoration policy and inherited state tint.

use kivact::draw::{Draw, RegionId, Size};
use kivact::prelude::*;
use kivact::style::{Align, pct};
use kivact::test::{SemanticNode, render};
use kivact::view::A11yAction;
use kivact::view::MouseButton;

/// A 24×24 square (an original test shape).
static SQUARE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="4" y="4" width="16" height="16" fill="black"/></svg>"#;

fn size_of(screen: &kivact::test::Screen, id: &str) -> (f32, f32) {
    let r = screen.get_by_test_id(id).rect().expect("laid out");
    (r.width, r.height)
}

#[test]
fn monochrome_image_fills_its_container_and_later_style_wins() {
    let screen = render(|| {
        row((
            node()
                .role(Role::Image)
                .style(style().size(16, 16).shrink(0))
                .child(
                    image(SQUARE)
                        .monochrome(true)
                        .role(Role::Generic)
                        .style(style().size(pct(100), pct(100))),
                )
                .test_id("default"),
            node()
                .role(Role::Image)
                .style(style().size(16, 16).shrink(0))
                .child(
                    image(SQUARE)
                        .monochrome(true)
                        .role(Role::Generic)
                        .style(style().size(pct(100), pct(100))),
                )
                .style(style().size(24, 24))
                .test_id("sized"),
            node()
                .role(Role::Image)
                .style(style().size(16, 16).shrink(0))
                .child(
                    image(SQUARE)
                        .monochrome(true)
                        .role(Role::Generic)
                        .style(style().size(pct(100), pct(100))),
                )
                .style(style().size(20, 20))
                .test_id("styled"),
            node()
                .role(Role::Image)
                .style(style().size(16, 16).shrink(0))
                .child(
                    image(SQUARE)
                        .monochrome(true)
                        .role(Role::Generic)
                        .style(style().size(pct(100), pct(100))),
                )
                .style(style().size(32, 32))
                .style(style().width(40))
                .test_id("later"),
        ))
        .style(style().align_items(Align::Start))
    });
    assert_eq!(size_of(&screen, "default"), (16.0, 16.0));
    assert_eq!(size_of(&screen, "sized"), (24.0, 24.0));
    assert_eq!(size_of(&screen, "styled"), (20.0, 20.0));
    assert_eq!(
        size_of(&screen, "later"),
        (40.0, 32.0),
        "the later layer wins"
    );
    let root = screen.get_by_test_id("sized");
    let image = root.children()[0];
    assert_eq!(image.rect(), root.rect(), "the image fills the icon");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The node with test id `id` under `n`.
fn by_test_id<'a>(n: &'a SemanticNode, id: &str) -> Option<&'a SemanticNode> {
    if n.test_id.as_deref() == Some(id) {
        return Some(n);
    }
    n.children.iter().find_map(|c| by_test_id(c, id))
}

/// The nodes under `n` as `(role, name)`, depth first.
fn roles(n: &SemanticNode, out: &mut Vec<(Option<Role>, String)>) {
    out.push((n.role, n.name.clone()));
    for c in &n.children {
        roles(c, out);
    }
}

/// An icon without a name is an `Image` with no name, which no host
/// exposes (the shared semantics policy, as HTML `<img alt="">`); its
/// image child is `Generic`, never exposed. `.name(..)` names the root, so
/// it is exposed as an `Image` with that name.
#[test]
fn image_container_is_decorative_until_named() {
    let screen = render(|| {
        column((
            node()
                .role(Role::Image)
                .style(style().size(16, 16).shrink(0))
                .child(
                    image(SQUARE)
                        .monochrome(true)
                        .role(Role::Generic)
                        .style(style().size(pct(100), pct(100))),
                )
                .test_id("plain"),
            node()
                .role(Role::Image)
                .style(style().size(16, 16).shrink(0))
                .child(
                    image(SQUARE)
                        .monochrome(true)
                        .role(Role::Generic)
                        .style(style().size(pct(100), pct(100))),
                )
                .name("保存")
                .test_id("named"),
        ))
    });
    let snap = screen.snapshot();
    let mut plain = Vec::new();
    roles(by_test_id(&snap, "plain").expect("plain"), &mut plain);
    assert_eq!(
        plain,
        vec![
            (Some(Role::Image), String::new()),
            (Some(Role::Generic), String::new()),
        ]
    );
    let mut named = Vec::new();
    roles(by_test_id(&snap, "named").expect("named"), &mut named);
    assert_eq!(
        named,
        vec![
            (Some(Role::Image), "保存".to_string()),
            (Some(Role::Generic), String::new()),
        ]
    );
    assert_eq!(
        screen.get_by_role(Role::Image, "保存").rect(),
        screen.get_by_test_id("named").rect()
    );
}

/// The icon is painted in its button's text color (from the frame's
/// pixels) and follows the button's hover and pressed colors under the
/// real pointer.
#[test]
fn monochrome_image_inherits_ancestor_state_text_color() {
    let screen = render(|| {
        column((node()
            .role(Role::Button)
            .name("Close")
            .on_press(|| {})
            .style(
                style()
                    .size(24, 24)
                    .color(rgb(0x202020))
                    .hover(style().color(rgb(0x2040d0)))
                    .pressed(style().color(rgb(0xd02020))),
            )
            .child(
                node()
                    .role(Role::Image)
                    .style(style().size(16, 16).shrink(0))
                    .child(
                        image(SQUARE)
                            .monochrome(true)
                            .role(Role::Generic)
                            .style(style().size(pct(100), pct(100))),
                    )
                    .test_id("icon"),
            ),))
    });
    let r = screen.get_by_test_id("icon").rect().expect("laid out");
    // The square covers the middle of the 16×16 icon.
    let (x, y) = (r.x + 8.0, r.y + 8.0);
    let tint = || screen.screenshot().pixel(x as u32, y as u32);
    assert_eq!(tint(), Some(0x202020), "the button's text color");
    screen.mouse_move(x, y);
    assert_eq!(tint(), Some(0x2040d0), "the button's hover color");
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(tint(), Some(0xd02020), "the button's pressed color");
    screen.mouse_up(MouseButton::Primary);
    screen.mouse_leave();
    assert_eq!(tint(), Some(0x202020), "back to the text color");
}

/// An unnamed image is decorative only while nothing but its pixels would
/// be exposed: a clickable thumbnail and a drawn chart with scene regions
/// stay exposed as `Image` (with an empty name) so that their action and
/// regions reach assistive technology; a test id alone exposes nothing.
#[test]
fn an_unnamed_clickable_image_or_drawn_chart_stays_exposed() {
    let screen = render(|| {
        column((
            image(SQUARE).test_id("thumb").on_press(|| {}),
            node()
                .role(Role::Image)
                .test_id("chart")
                .style(style().size(40, 20))
                .region(
                    RegionId(1),
                    kivact::primitive::part(|state| state)
                        .role(Role::Image)
                        .name("bar"),
                )
                .draw(|d: &mut Draw, size: Size| {
                    d.region(RegionId(1), size.to_rect());
                }),
            image(SQUARE).test_id("plain"),
        ))
    });
    let thumb = screen.get_by_test_id("thumb");
    assert_eq!((thumb.role(), thumb.name()), (Some(Role::Image), "".into()));
    assert_eq!(thumb.a11y_actions(), vec![A11yAction::Activate]);
    let chart = screen.get_by_test_id("chart");
    assert_eq!((chart.role(), chart.name()), (Some(Role::Image), "".into()));
    assert_eq!(screen.get_by_role(Role::Image, "bar").name(), "bar");
    assert_eq!(
        screen.get_by_test_id("plain").role(),
        None,
        "a test id alone"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
