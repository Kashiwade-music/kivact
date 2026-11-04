//! Role: Tests for overlay placement and dismissal behavior.
//! `Overlay::on_dismiss` runs on an outside press (not on a press inside
//! the overlay or on its anchor), on an Escape nobody handled (topmost
//! overlay only) and on an assistive technology's Dismiss. While a modal
//! overlay is open, keys go to it and stop at it, and events aimed outside
//! it are dropped. Overlays flip and stay inside the window.

use std::cell::Cell;
use std::rc::Rc;

use kivact::ErrorKind;
use kivact::geometry::{Point, Rect};
use kivact::prelude::*;
use kivact::style::Position;
use kivact::test::{RenderOptions, render, render_with};
use kivact::view::{A11yAction, Handled, OverlayAlign, Placement, overlay, overlay_at};

#[test]
fn fitted_overlays_flip_slide_and_respect_the_theme_margin() {
    let screen = render_with(
        RenderOptions::from_env()
            .window_size(240.0, 180.0)
            .theme(Theme::light().with_overlay_margin(16.0)),
        || {
            let anchor = node_ref();
            let middle = node_ref();
            (
                node().node_ref(anchor).style(
                    style()
                        .position(Position::Fixed)
                        .left(208)
                        .top(126)
                        .size(16, 20),
                ),
                overlay(anchor, Placement::Below, node())
                    .align(OverlayAlign::Center)
                    .offset(4)
                    .style(style().size(104, 44))
                    .test_id("popup"),
                overlay_at(Point::new(0.0, 0.0), Placement::Center, node())
                    .style(style().size(400, 40))
                    .test_id("wide"),
                node().node_ref(middle).style(
                    style()
                        .position(Position::Fixed)
                        .left(110)
                        .top(80)
                        .size(20, 20),
                ),
                overlay(middle, Placement::Below, node())
                    .align(OverlayAlign::Center)
                    .keep_anchor_clear()
                    .offset(4)
                    .style(style().size(190, 110))
                    .test_id("tooltip"),
            )
        },
    );
    let popup = screen.get_by_test_id("popup").rect().expect("popup layout");
    assert!(popup.x >= 16.0 && popup.x + popup.width <= 224.0);
    assert!(popup.y + popup.height <= 122.0, "flips above the anchor");
    let wide = screen.get_by_test_id("wide").rect().expect("wide layout");
    assert!(wide.x >= 16.0 && wide.x + wide.width <= 224.0);
    assert!(wide.y >= 16.0);
    let tooltip = screen
        .get_by_test_id("tooltip")
        .rect()
        .expect("tooltip layout");
    assert!(tooltip.height > 0.0 && tooltip.height < 110.0);
    assert!(tooltip.y >= 104.0 || tooltip.y + tooltip.height <= 76.0);
}

/// Counts calls of the handlers it makes.
#[derive(Clone, Default)]
struct Count(Rc<Cell<u32>>);

impl Count {
    fn bump(&self) -> impl Fn() + Clone + 'static {
        let c = self.0.clone();
        move || c.set(c.get() + 1)
    }
    fn get(&self) -> u32 {
        self.0.get()
    }
}

/// A menu button toggling a menu overlay below it, and another button.
fn menu(dismissed: Count, other: Count) -> impl Component {
    let open = signal(false);
    let anchor = node_ref();
    column((
        {
            let label = ("メニュー").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .node_ref(anchor)
        .on_press(move || open.update(|o| *o = !*o)),
        {
            let label = ("ほか").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .on_press(other.bump()),
        show(open, move || {
            let d = dismissed.bump();
            overlay(
                anchor,
                Placement::Below,
                column(({
                    let label = ("コピー").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                },)),
            )
            // A role exposes the menu (and its Dismiss) to assistive
            // technology.
            .with_node(move |node| node.role(Role::Menu))
            .test_id("menu")
            .on_dismiss(move |_| {
                d();
                open.set(false);
            })
        }),
    ))
}

#[test]
fn an_outside_press_dismisses_but_the_overlay_and_its_anchor_do_not() {
    let (dismissed, other) = (Count::default(), Count::default());
    let (d, o) = (dismissed.clone(), other.clone());
    let s = render(move || menu(d, o));
    s.get_by_role(Role::Button, "メニュー").click();
    s.get_by_role(Role::Button, "コピー").click();
    assert!(
        s.query_by_test_id("menu").is_some(),
        "a press inside keeps it"
    );
    // The anchor toggles the menu itself: no dismissal first.
    s.get_by_role(Role::Button, "メニュー").click();
    assert!(s.query_by_test_id("menu").is_none(), "the anchor toggles");
    assert_eq!(dismissed.get(), 0);
    s.get_by_role(Role::Button, "メニュー").click();
    s.get_by_role(Role::Button, "ほか").click();
    assert!(
        s.query_by_test_id("menu").is_none(),
        "an outside press closes"
    );
    assert_eq!(dismissed.get(), 1);
    assert_eq!(other.get(), 1, "the press still reaches the node under it");
}

#[test]
fn an_unhandled_escape_dismisses_the_topmost_overlay_only() {
    let s = render(|| {
        let menu = signal(false);
        let sub = signal(false);
        let (anchor, item) = (node_ref(), node_ref());
        column((
            {
                let label = ("メニュー").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .node_ref(anchor)
            .on_press(move || menu.set(true)),
            show(menu, move || {
                overlay(
                    anchor,
                    Placement::Below,
                    column((
                        {
                            let label = ("その他").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .node_ref(item)
                        .on_press(move || sub.set(true)),
                        show(sub, move || {
                            overlay(item, Placement::End, {
                                let label = ("詳細").into_value();
                                node()
                                    .role(Role::Button)
                                    .focusable(true)
                                    .name(label)
                                    .child(text(label))
                            })
                            .test_id("sub")
                            .on_dismiss(move |_| sub.set(false))
                        }),
                    )),
                )
                .test_id("menu")
                .on_dismiss(move |_| menu.set(false))
            }),
        ))
    });
    s.get_by_role(Role::Button, "メニュー").click();
    s.get_by_role(Role::Button, "その他").click();
    assert!(s.query_by_test_id("sub").is_some());
    s.press_key(Key::Escape);
    assert!(
        s.query_by_test_id("sub").is_none(),
        "the inner overlay closes"
    );
    assert!(s.query_by_test_id("menu").is_some(), "the outer one stays");
    s.press_key(Key::Escape);
    assert!(s.query_by_test_id("menu").is_none(), "then the outer one");
    // Nothing left to dismiss: Escape does nothing.
    s.press_key(Key::Escape);
    assert!(s.errors().is_empty(), "{:?}", s.errors());
}

#[test]
fn keys_go_to_an_open_modal_without_focus_and_stop_there() {
    let (page, modal) = (Count::default(), Count::default());
    let (p, m) = (page.clone(), modal.clone());
    let s = render(move || {
        let open = signal(false);
        let opener = node_ref();
        let (p, m) = (p.bump(), m.bump());
        column((
            {
                let label = ("開く").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .node_ref(opener)
            .on_press(move || open.set(true)),
            show(open, move || {
                let m = m.clone();
                // No tab stop inside: nothing is focused while it is open.
                overlay(opener, Placement::Center, text("お知らせ"))
                    .modal()
                    .with_node(move |node| node.role(Role::Dialog))
                    .on_key(move |event| {
                        assert_eq!(event.target_role(), Some(Role::Dialog));
                        m();
                        Handled::No
                    })
            }),
        ))
        .on_key(move |_| {
            p();
            Handled::No
        })
    });
    s.get_by_role(Role::Button, "開く")
        .press_key(Key::Char('a'));
    assert_eq!(
        (page.get(), modal.get()),
        (1, 0),
        "no modal: bubbles to the page"
    );
    s.get_by_role(Role::Button, "開く").click();
    assert!(s.focused().is_none(), "the modal has no tab stop");
    s.press_key(Key::Char('a'));
    assert_eq!(
        (page.get(), modal.get()),
        (1, 1),
        "the modal gets the key and the page behind it does not"
    );
}

/// Placement against the window edges: an `End` popup at the right edge
/// flips to the start, a wide popup below a right-edge anchor stays below
/// and is pushed inside, an unanchored overlay is centred, an anchored one
/// is out of flow (the next sibling does not move), and a popup follows
/// its anchor when the anchor moves.
#[test]
fn overlays_flip_clamp_center_stay_out_of_flow_and_follow_their_anchor() {
    let s = render(|| {
        let (left, right) = (node_ref(), node_ref());
        let push = signal(0.0_f64);
        column((
            node()
                .test_id("push")
                .style(move || style().height(push.get()))
                .on_press(move || push.set(40.0)),
            row((
                {
                    let label = ("left").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(left),
                spacer(),
                {
                    let label = ("right").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(right),
            )),
            overlay(
                left,
                Placement::Below,
                node().test_id("below").style(style().size(60, 20)),
            ),
            node().test_id("after").style(style().size(60, 20)),
            overlay(
                right,
                Placement::End,
                node().test_id("flipped").style(style().size(100, 20)),
            ),
            overlay(
                right,
                Placement::Below,
                node().test_id("clamped").style(style().size(300, 20)),
            ),
            overlay(
                node_ref(),
                Placement::Below,
                node().test_id("centred").style(style().size(100, 50)),
            ),
        ))
    });
    let rect = |id: &str| s.get_by_test_id(id).rect().expect("laid out");
    let button_rect = |name: &str| s.get_by_role(Role::Button, name).rect().expect("laid out");
    let (l, r) = (button_rect("left"), button_rect("right"));
    assert_eq!(rect("after").y, l.y + l.height, "out of flow");
    assert_eq!((rect("below").x, rect("below").y), (l.x, l.y + l.height));
    let flipped = rect("flipped");
    assert!(
        flipped.x + flipped.width <= r.x + 0.5,
        "flipped to the start: {flipped:?} {r:?}"
    );
    let clamped = rect("clamped");
    assert_eq!(clamped.y, r.y + r.height, "stays below");
    assert!(
        clamped.x + clamped.width <= 800.5 && clamped.x >= 0.0,
        "{clamped:?}"
    );
    assert_eq!(rect("centred"), Rect::new(350.0, 275.0, 100.0, 50.0));
    s.get_by_test_id("below").click();

    s.get_by_test_id("push").a11y(A11yAction::Activate);
    let l = button_rect("left");
    assert_eq!(rect("below").y, l.y + l.height, "follows the moved anchor");
    s.get_by_test_id("below").click();
}

/// An overlay placed at a point has no anchor width to follow: asking for it is reported.
#[test]
fn at_least_anchor_width_on_a_point_overlay_is_reported() {
    let screen = render(|| {
        overlay_at(Point::new(10.0, 10.0), Placement::Below, text("menu")).at_least_anchor_width()
    });
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(*errors[0].kind(), ErrorKind::Unsupported);
    assert!(
        errors[0].message().contains("at_least_anchor_width"),
        "{}",
        errors[0].message()
    );
}
