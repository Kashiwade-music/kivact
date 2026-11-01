//! Role: Tests for overlay placement and dismissal behavior.
//! `Overlay::on_dismiss` runs on an outside press (not on a press inside
//! the overlay or on its anchor), on an Escape nobody handled (topmost
//! overlay only) and on an assistive technology's Dismiss. While a modal
//! overlay is open, keys go to it and stop at it, and events aimed outside
//! it are dropped. Overlays flip and stay inside the window.

use kivact::primitive::NodeExt;
use std::cell::Cell;
use std::rc::Rc;

use kivact::draw::{Point, Rect};
use kivact::prelude::*;
use kivact::style::pct;
use kivact::test::render;
use kivact::view::{A11yAction, Handled, OverlayAlign, Placement, PointerEvent, overlay};

/// Transactional editors distinguish an Escape rollback from an outside commit
/// without intercepting keys or reaching into the overlay input policy.
#[test]
fn dismissal_reasons_preserve_transactional_commit_and_cancel() {
    use kivact::view::DismissReason;
    let (screen, (value, reasons)) = crate::support::mount(|| {
        let value = signal(10);
        let original = signal(10);
        let open = signal(false);
        let reasons = signal(Vec::new());
        let anchor = node_ref();
        (
            (value, reasons),
            column((
                {
                    let label = ("Edit transaction").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(anchor)
                .on_press(move || {
                    original.set(value.get_untracked());
                    open.set(true);
                }),
                {
                    let label = ("Outside transaction").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                },
                show(open, move || {
                    overlay(
                        anchor,
                        Placement::Below,
                        {
                            let label = ("Change transaction").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .on_press(move || value.update(|v| *v += 10)),
                    )
                    .role(Role::Dialog)
                    .name("Transaction")
                    .on_dismiss(|_| -> () { panic!("the later handler replaces this one") })
                    .on_dismiss(move |reason| {
                        reasons.update(|r| r.push(reason));
                        if reason == DismissReason::Escape {
                            value.set(original.get_untracked());
                        }
                        open.set(false);
                    })
                }),
            )),
        )
    });
    let edit = || {
        screen.get_by_role(Role::Button, "Edit transaction").click();
        screen
            .get_by_role(Role::Button, "Change transaction")
            .click();
    };
    edit();
    screen.press_key(Key::Escape);
    assert_eq!(value.get_untracked(), 10);
    assert!(screen.query_by_role(Role::Dialog, "Transaction").is_none());
    edit();
    screen
        .get_by_role(Role::Button, "Outside transaction")
        .click();
    assert_eq!(value.get_untracked(), 20);
    edit();
    screen
        .get_by_role(Role::Dialog, "Transaction")
        .a11y(A11yAction::Dismiss);
    assert_eq!(value.get_untracked(), 30);
    assert_eq!(
        reasons.get_untracked(),
        [
            DismissReason::Escape,
            DismissReason::OutsidePress,
            DismissReason::Accessibility
        ]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
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
fn menu(dismissed: Count, other: Count) -> impl IntoView {
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
            .role(Role::Menu)
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
fn an_assistive_dismiss_reaches_on_dismiss() {
    let (dismissed, other) = (Count::default(), Count::default());
    let d = dismissed.clone();
    let s = render(move || menu(d, other));
    s.get_by_role(Role::Button, "メニュー").click();
    s.get_by_test_id("menu").a11y(A11yAction::Dismiss);
    assert!(s.query_by_test_id("menu").is_none());
    assert_eq!(dismissed.get(), 1);
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
fn a_handled_escape_does_not_dismiss() {
    let handle = Rc::new(Cell::new(true));
    let h = handle.clone();
    let s = render(move || {
        let open = signal(true);
        let anchor = node_ref();
        column((
            kivact::control::node()
                .name("検索")
                .part(kivact::interaction::use_text_edit(kivact::signal(String::new())).control())
                .node_ref(anchor)
                .on_key(move |k| {
                    if k.key == Key::Escape && h.get() {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                }),
            show(open, move || {
                overlay(anchor, Placement::Below, text("候補"))
                    .test_id("list")
                    .on_dismiss(move |_| open.set(false))
            }),
        ))
    });
    let field = s.get_by_role(Role::TextBox, "検索");
    field.press_key(Key::Escape);
    assert!(s.query_by_test_id("list").is_some(), "handled: kept open");
    handle.set(false);
    field.press_key(Key::Escape);
    assert!(s.query_by_test_id("list").is_none(), "unhandled: dismissed");
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
                    .modal(true)
                    .role(Role::Dialog)
                    .on_key(move |event| {
                        assert_eq!(event.target_role, Some(Role::Dialog));
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

#[test]
fn events_aimed_outside_an_open_modal_are_dropped() {
    let (pressed, dismissed) = (Count::default(), Count::default());
    let (p, d) = (pressed.clone(), dismissed.clone());
    let s = render(move || {
        let open = signal(true);
        let anchor = node_ref();
        let d = d.bump();
        column((
            text("見出し").node_ref(anchor),
            {
                let label = ("後ろ").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .on_press(p.bump()),
            show(open, move || {
                let d = d.clone();
                overlay(anchor, Placement::Below, {
                    let label = ("閉じる").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                })
                .modal(true)
                .test_id("dialog")
                .on_dismiss(move |_| {
                    d();
                    open.set(false);
                })
            }),
        ))
    });
    let behind = s.get_by_role(Role::Button, "後ろ");
    // The host offers assistive technology no action behind the modal, and
    // carries out none. (The core's own drop of such events, from a host
    // that did not confine them, is tested by kivact's screen tests in
    // src/test/screen.)
    assert!(behind.a11y_actions().is_empty());
    assert!(
        !s.get_by_role(Role::Button, "閉じる")
            .a11y_actions()
            .is_empty()
    );
    behind.a11y(A11yAction::Activate);
    behind.a11y(A11yAction::Expand);
    assert_eq!(pressed.get(), 0, "Activate behind the modal is dropped");
    // A press outside reaches nothing but dismisses the modal.
    behind.force_click();
    assert_eq!((pressed.get(), dismissed.get()), (0, 1));
    assert!(s.query_by_test_id("dialog").is_none());
    s.get_by_role(Role::Button, "後ろ").click();
    assert_eq!(pressed.get(), 1, "closed: input reaches the page again");
}

#[test]
fn an_overlay_flips_above_at_the_window_bottom_and_stays_inside() {
    let s = render(|| {
        let (low, right) = (node_ref(), node_ref());
        column((
            spacer(),
            row((
                {
                    let label = ("下").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(low),
                spacer(),
                {
                    let label = ("右").into_value();
                    node()
                        .role(Role::Button)
                        .focusable(true)
                        .name(label)
                        .child(text(label))
                }
                .node_ref(right),
            )),
            overlay(
                low,
                Placement::Below,
                node().test_id("low").style(style().width(100).height(120)),
            ),
            overlay(
                right,
                Placement::End,
                node().test_id("right").style(style().width(100).height(40)),
            ),
        ))
        .style(style().height(600))
    });
    let rect = |id: &str| s.get_by_test_id(id).rect().expect("laid out");
    let (anchor, low) = (s.get_by_role(Role::Button, "下").rect(), rect("low"));
    let anchor = anchor.expect("laid out");
    assert!(low.y + low.height <= anchor.y + 0.5, "{anchor:?} {low:?}");
    let right = rect("right");
    assert!(right.x + right.width <= 800.5, "{right:?}");
}

#[test]
fn several_node_refs_bind_to_one_node() {
    let r = Rc::new(Cell::new(None));
    let r2 = r.clone();
    let s = render(move || {
        let (a, b) = (node_ref(), node_ref());
        r2.set(Some((a.focused(), b.focused())));
        // The same ref twice is bound once, without an error.
        {
            let label = ("ボタン").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        }
        .node_ref(a)
        .node_ref(b)
        .node_ref(a)
    });
    let (a, b) = r.get().expect("setup ran");
    s.get_by_role(Role::Button, "ボタン").focus();
    assert!(a.get_untracked() && b.get_untracked());
    assert!(s.errors().is_empty(), "{:?}", s.errors());
}

#[test]
fn the_later_of_two_dismiss_handlers_wins() {
    let (first, second) = (Count::default(), Count::default());
    let (a, b) = (first.clone(), second.clone());
    let s = render(move || {
        overlay(node_ref(), Placement::Center, {
            let label = ("中").into_value();
            node()
                .role(Role::Button)
                .focusable(true)
                .name(label)
                .child(text(label))
        })
        .role(Role::Dialog)
        .test_id("o")
        .on_dismiss({
            let f = a.bump();
            move |_| f()
        })
        .on_dismiss({
            let f = b.bump();
            move |_| f()
        })
    });
    s.get_by_test_id("o").a11y(A11yAction::Dismiss);
    assert_eq!((first.get(), second.get()), (0, 1), "only the later runs");
}

#[test]
fn a_drag_and_a_wheel_inside_a_modal_do_not_reach_a_listener_around_it() {
    let (pointers, wheels) = (Count::default(), Count::default());
    let (p, w) = (pointers.clone(), wheels.clone());
    let s = render(move || {
        let (p, w) = (p.bump(), w.bump());
        node()
            .on_pointer(move |_| {
                p();
                Handled::Yes
            })
            .on_wheel(move |_| {
                w();
                Handled::Yes
            })
            .child(
                overlay(
                    node_ref(),
                    Placement::Center,
                    node().test_id("pad").style(style().width(100).height(100)),
                )
                .modal(true),
            )
    });
    let pad = s.get_by_test_id("pad");
    pad.drag(Point::new(10.0, 10.0), Point::new(60.0, 60.0));
    pad.scroll(0.0, 20.0);
    assert_eq!((pointers.get(), wheels.get()), (0, 0));
}

/// Outside presses close nested overlays innermost first (a press inside
/// the parent closes only the child), an overlay without `on_dismiss`
/// stays, and a press on a modal's anchor dismisses the modal without
/// activating the anchor.
#[test]
fn outside_presses_close_the_innermost_first_and_spare_overlays_without_dismiss() {
    let log: Rc<std::cell::RefCell<Vec<&'static str>>> = Rc::default();
    let out = log.clone();
    let s = render(move || {
        let (menu_on, sub_on, modal_on) = (signal(true), signal(true), signal(false));
        let (menu_anchor, item, modal_anchor) = (node_ref(), node_ref(), node_ref());
        let (o1, o2, o3) = (out.clone(), out.clone(), out.clone());
        column((
            {
                let label = ("menu").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .node_ref(menu_anchor),
            {
                let label = ("sticky anchor").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            },
            {
                let label = ("modal").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
            }
            .node_ref(modal_anchor)
            .on_press(move || {
                o3.borrow_mut().push("modal pressed");
                modal_on.set(true);
            }),
            spacer(),
            show(menu_on, move || {
                let (o1, o2) = (o1.clone(), o2.clone());
                overlay(
                    menu_anchor,
                    Placement::Below,
                    column((
                        {
                            let label = ("item").into_value();
                            node()
                                .role(Role::Button)
                                .focusable(true)
                                .name(label)
                                .child(text(label))
                        }
                        .node_ref(item),
                        node().test_id("menu pad").style(style().size(80, 30)),
                        show(sub_on, move || {
                            let o2 = o2.clone();
                            overlay(
                                item,
                                Placement::End,
                                column(({
                                    let label = ("sub item").into_value();
                                    node()
                                        .role(Role::Button)
                                        .focusable(true)
                                        .name(label)
                                        .child(text(label))
                                },)),
                            )
                            .on_dismiss(move |_| {
                                o2.borrow_mut().push("sub");
                                sub_on.set(false);
                            })
                        }),
                    )),
                )
                .on_dismiss(move |_| {
                    o1.borrow_mut().push("menu");
                    menu_on.set(false);
                })
            }),
            overlay(
                node_ref(),
                Placement::Center,
                node().test_id("no dismiss").style(style().size(40, 40)),
            ),
            show(modal_on, move || {
                let o = out.clone();
                overlay(
                    modal_anchor,
                    Placement::End,
                    column(({
                        let label = ("in modal").into_value();
                        node()
                            .role(Role::Button)
                            .focusable(true)
                            .name(label)
                            .child(text(label))
                    },)),
                )
                .modal(true)
                .on_dismiss(move |_| {
                    o.borrow_mut().push("modal dismissed");
                    modal_on.set(false);
                })
            }),
        ))
        .style(style().height(600))
    });
    s.get_by_test_id("menu pad").click();
    assert_eq!(
        *log.borrow(),
        ["sub"],
        "a press inside the menu closes the submenu only"
    );
    assert!(s.query_by_role(Role::Button, "sub item").is_none());
    s.click_at(790.0, 590.0);
    assert_eq!(*log.borrow(), ["sub", "menu"]);
    assert!(s.query_by_role(Role::Button, "item").is_none());
    assert!(
        s.get_by_test_id("no dismiss").is_visible(),
        "no on_dismiss: stays"
    );

    log.borrow_mut().clear();
    s.get_by_role(Role::Button, "modal").click();
    assert_eq!(*log.borrow(), ["modal pressed"]);
    s.get_by_role(Role::Button, "modal").force_click();
    assert_eq!(
        *log.borrow(),
        ["modal pressed", "modal dismissed"],
        "the anchor of a modal is outside it: dismissed, not activated"
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
        let push = signal(0.0_f32);
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

/// A 100 × 30 box placed at `(x, y)` of the window, bound to `r`.
fn anchor_at(r: NodeRef, x: i32, y: i32) -> Node {
    node().node_ref(r).style(
        style()
            .position(kivact::style::Position::Absolute)
            .left(x)
            .top(y)
            .size(100, 30),
    )
}

/// A `w × h` overlay content with test id `id`.
fn panel(id: &'static str, w: i32, h: i32) -> Node {
    node().test_id(id).style(style().size(w, h))
}

#[test]
fn overlays_align_to_the_anchors_end_keep_an_offset_and_flip_their_alignment() {
    let s = render(|| {
        let (menu, left, item, edge) = (node_ref(), node_ref(), node_ref(), node_ref());
        node()
            .child((
                anchor_at(menu, 400, 100),
                anchor_at(left, 0, 300),
                anchor_at(item, 300, 540),
                anchor_at(edge, 650, 200),
                // .menu--right { top: calc(100% + 6px); right: 0 }
                overlay(menu, Placement::Below, panel("right", 160, 80))
                    .align(OverlayAlign::End)
                    .offset(6),
                // End-aligned at the left edge: no room, so start-aligned.
                overlay(left, Placement::Below, panel("left", 160, 40)).align(OverlayAlign::End),
                // A submenu beside its item, top-aligned; near the bottom it
                // aligns its bottom with the item's instead.
                overlay(item, Placement::End, panel("sub", 120, 90)).offset(2),
                // A submenu with no room on the right opens on the left.
                overlay(edge, Placement::End, panel("flipped", 120, 60)).offset(4),
                // Centered along a vertical side.
                overlay(menu, Placement::Start, panel("mid", 50, 10)).align(OverlayAlign::Center),
            ))
            .style(style().size(800, 600))
    });
    let rect = |id: &str| s.get_by_test_id(id).rect().expect("laid out");
    let near = |a: f32, b: f32| (a - b).abs() <= 0.5;
    let r = rect("right");
    assert!(
        near(r.x, 500.0 - 160.0) && near(r.y, 136.0),
        "end-aligned, 6 px below: {r:?}"
    );
    let l = rect("left");
    assert!(
        near(l.x, 0.0) && near(l.y, 330.0),
        "start-aligned at the edge: {l:?}"
    );
    let sub = rect("sub");
    assert!(
        near(sub.x, 402.0) && near(sub.y, 570.0 - 90.0),
        "beside its item, bottom-aligned: {sub:?}"
    );
    let f = rect("flipped");
    assert!(
        near(f.x, 650.0 - 4.0 - 120.0) && near(f.y, 200.0),
        "flipped left: {f:?}"
    );
    let m = rect("mid");
    assert!(
        near(m.x, 400.0 - 50.0) && near(m.y, 110.0),
        "centered beside: {m:?}"
    );
}

#[test]
fn an_overlay_at_the_pointer_opens_there_and_flips_near_the_edges() {
    use kivact::view::{MouseButton, PointerPhase, overlay_at};
    let s = render(|| {
        let at = signal(None::<Point>);
        let pressed = signal(Point::default());
        column((
            node()
                .test_id("area")
                .on_pointer(move |e: PointerEvent| {
                    if e.phase == PointerPhase::Down && e.button == Some(MouseButton::Secondary) {
                        pressed.set(e.pos);
                        at.set(Some(e.window_pos));
                    }
                })
                .style(style().grow(1).margin(20)),
            show_some(at, move |p| {
                overlay_at(p, Placement::Below, panel("context", 150, 100))
                    .role(Role::Menu)
                    .on_dismiss(move |_| at.set(None))
            }),
            text(move || format!("local {:?}", pressed.get())).test_id("local"),
        ))
        .style(style().height(pct(100)))
    });
    s.mouse_move(200.0, 150.0);
    s.mouse_down(MouseButton::Secondary);
    s.mouse_up(MouseButton::Secondary);
    let near = |a: f32, b: f32| (a - b).abs() <= 0.5;
    let c = s.get_by_test_id("context").rect().expect("open");
    assert!(
        near(c.x, 200.0) && near(c.y, 150.0),
        "at the pointer: {c:?}"
    );
    // The event's own position stays local to the area (margin 20).
    assert_eq!(
        s.get_by_test_id("local").text(),
        "local Point { x: 180.0, y: 130.0 }"
    );

    // Near the bottom right corner it opens up and to the left.
    s.click_at(10.0, 10.0);
    assert!(
        s.query_by_test_id("context").is_none(),
        "an outside press dismissed it"
    );
    s.mouse_move(760.0, 540.0);
    s.mouse_down(MouseButton::Secondary);
    s.mouse_up(MouseButton::Secondary);
    let c = s.get_by_test_id("context").rect().expect("open");
    assert!(
        near(c.x, 760.0 - 150.0) && near(c.y, 540.0 - 100.0),
        "flipped up and left: {c:?}"
    );
}
