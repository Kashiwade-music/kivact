//! Role: Tests for Dialog component interaction and accessibility.
//!
//! The unstyled `dialog`: the four kinds with their roles and relations, the
//! title and description parts, where the focus starts and returns, the three
//! ways to close it, and the modal containing Tab. The tests draw the frame
//! (scrim, panel, heading, body and footer) themselves, as a kit does.

use kivact::primitive::NodeExt;
use kivact::view::View;
use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount;
use kivact::interaction::{DialogApi, use_dialog};
use kivact::motion::{Easing, Transition};
use kivact::prelude::*;
use kivact::style::Position;
use kivact::style::{Justify, Style, pct};
use kivact::test::Screen;
use kivact::ui::{DialogKind, Dismiss, Side};
use kivact::view::{A11yAction, Placement, error_boundary, overlay};
use std::time::Duration;

fn button(name: &'static str) -> Node {
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .test_id(name)
        .style(style().size(100, 30).shrink(0))
        .child(text(name))
}

/// What the test frame draws besides the heading, the body and the footer.
#[derive(Clone, Default)]
struct Frame {
    /// A text under the title with the description part.
    description: Option<&'static str>,
    /// A "Close" button with the close part at the end of the header.
    close_button: bool,
    /// Leave the heading with the title part out.
    hidden_title: bool,
    /// The panel's enter paint and transition, shown on each opening.
    enter: Option<Style>,
}

/// The frame a kit draws: a scrim and a panel with the heading, the
/// description, `body` and `footer` (when it shows something), placed for the
/// kind, over the parts of `dialog`.
fn framed<B: IntoView, F: IntoView>(
    dialog: DialogApi,
    kind: DialogKind,
    anchor: NodeRef,
    title: impl IntoText,
    frame: Frame,
    body: impl Fn() -> B + 'static,
    footer: impl Fn() -> F + 'static,
) -> View {
    let title = title.into_text_value();
    show(dialog.shown(), move || {
        let heading = (!frame.hidden_title).then(|| text(title).part(dialog.title()));
        let closer = frame
            .close_button
            .then(|| button("Close").part(dialog.close_trigger()));
        let intro = frame
            .description
            .map(|d| text(d).part(dialog.description()));
        let content = node().child(body());
        let content = match kind {
            DialogKind::Drawer(_) => scroll(content)
                .style(style().grow(1).min_height(0))
                .into_view(),
            _ => content.into_view(),
        };
        let footer = footer().into_view();
        let footer = (!kivact::primitive::view_is_empty(&footer))
            .then(|| row(footer).style(style().justify_content(Justify::End)));
        let mut panel = column((row((heading, closer)), intro, content, footer))
            .style(style().padding(16).gap(12).background(ColorToken::Surface))
            .style(match kind {
                DialogKind::Drawer(side) => edge(side),
                DialogKind::Popover => style().width(288),
                _ => style().width(480),
            });
        if let Some(enter) = frame.enter.clone() {
            panel = panel.style(enter);
        }
        let popover = kind == DialogKind::Popover;
        let placement = if popover {
            Placement::Below
        } else {
            Placement::Center
        };
        let layer = overlay(anchor, placement, panel)
            .part(dialog.content())
            .name(title);
        if popover {
            layer.offset(4).into_view()
        } else {
            let scrim = node().style(style().position(Position::Fixed).inset(0));
            (scrim, layer).into_view()
        }
    })
    .into_view()
}

/// A drawer's panel, fixed to its edge of the window and as long as it.
fn edge(side: Side) -> Style {
    let fixed = style().position(Position::Fixed).width(400);
    match side {
        Side::Left => fixed.left(0).top(0).bottom(0),
        Side::Right => fixed.right(0).top(0).bottom(0),
        Side::Top => fixed.left(0).right(0).top(0).width(pct(100)),
        Side::Bottom => fixed.left(0).right(0).bottom(0).width(pct(100)),
    }
}

/// "Open" (opens the dialog), the dialog with "First" and "Second" buttons in
/// its body and "Done" (closes it) in its footer, and "Behind", another
/// button of the page.
fn page(
    kind: DialogKind,
    tweak: impl Fn(DialogApi) -> DialogApi + 'static,
) -> (Screen, Signal<bool>) {
    page_with(kind, tweak, Frame::default())
}

/// [`page`] drawn with `frame`.
fn page_with(
    kind: DialogKind,
    tweak: impl Fn(DialogApi) -> DialogApi + 'static,
    frame: Frame,
) -> (Screen, Signal<bool>) {
    page_ordered(kind, tweak, frame, false)
}

/// [`page`] with "Behind" before the "Open" button in the tab order, so that
/// the focus a removed surface leaves (the next tab stop, wrapping to the
/// first) is not the opener.
fn page_opener_second(
    kind: DialogKind,
    tweak: impl Fn(DialogApi) -> DialogApi + 'static,
) -> (Screen, Signal<bool>) {
    page_ordered(kind, tweak, Frame::default(), true)
}

fn page_ordered(
    kind: DialogKind,
    tweak: impl Fn(DialogApi) -> DialogApi + 'static,
    frame: Frame,
    behind_first: bool,
) -> (Screen, Signal<bool>) {
    mount(move || {
        let open = signal(false);
        let anchor = node_ref();
        let behavior = tweak(use_dialog(open, kind));
        let behavior = if kind == DialogKind::Popover {
            behavior.anchor(anchor)
        } else {
            behavior
        };
        let opener = button("Open")
            .node_ref(anchor)
            .on_press(move || open.update(|v| *v = !*v));
        let behind = button("Behind");
        let (first, second) = if behind_first {
            (behind, opener)
        } else {
            (opener, behind)
        };
        (
            open,
            column((
                first,
                second,
                button("Far").style(style().margin_top(300)),
                framed(
                    behavior,
                    kind,
                    anchor,
                    "Settings",
                    frame,
                    || column((button("First"), button("Second"))),
                    move || button("Done").on_press(move || open.set(false)),
                ),
            )),
        )
    })
}

fn kinds() -> [DialogKind; 5] {
    [
        DialogKind::Modal,
        DialogKind::AlertModal,
        DialogKind::Drawer(Side::Right),
        DialogKind::Drawer(Side::Left),
        DialogKind::Popover,
    ]
}

fn role_of(kind: DialogKind) -> Role {
    match kind {
        DialogKind::AlertModal => Role::AlertDialog,
        _ => Role::Dialog,
    }
}

fn noop(dialog: DialogApi) -> DialogApi {
    dialog
}

#[test]
fn every_kind_is_a_named_overlay_with_its_role_and_a_heading_for_the_title() {
    for kind in kinds() {
        let (screen, open) = page(kind, noop);
        assert!(
            screen.query_by_role(role_of(kind), "Settings").is_none(),
            "{kind:?} is closed at first"
        );
        screen.get_by_role(Role::Button, "Open").click();
        assert!(open.get_untracked());
        let dialog = screen.get_by_role(role_of(kind), "Settings");
        assert_eq!(dialog.role(), Some(role_of(kind)), "{kind:?}");
        assert_eq!(dialog.name(), "Settings", "{kind:?}");
        let heading = screen.get_by_role(Role::Heading, "Settings");
        assert_eq!(heading.text(), "Settings");
        assert_eq!(heading.level(), Some(2), "{kind:?}");
        let labels: Vec<String> = dialog.labelled_by().iter().map(|e| e.text()).collect();
        assert_eq!(labels, ["Settings"], "{kind:?}: labelled by the title part");
        screen.get_by_role(Role::Button, "Done").click();
        assert!(!open.get_untracked());
        assert!(
            screen.query_by_role(role_of(kind), "Settings").is_none(),
            "{kind:?} closes"
        );
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn the_title_follows_a_reactive_value() {
    let (screen, title) = mount(|| {
        let open = signal(true);
        let title = signal("First".to_string());
        (
            title,
            framed(
                use_dialog(open, DialogKind::Modal),
                DialogKind::Modal,
                node_ref(),
                title,
                Frame::default(),
                || (),
                || (),
            ),
        )
    });
    screen.get_by_role(Role::Dialog, "First");
    title.set("Second".into());
    screen.run_until_idle();
    screen.get_by_role(Role::Dialog, "Second");
    screen.get_by_role(Role::Heading, "Second");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_description_is_what_the_dialog_is_described_by() {
    for kind in kinds() {
        let frame = Frame {
            description: Some("Changes apply at once."),
            ..Frame::default()
        };
        let (screen, _) = page_with(kind, noop, frame);
        screen.get_by_role(Role::Button, "Open").click();
        let dialog = screen.get_by_role(role_of(kind), "Settings");
        let described: Vec<String> = dialog.described_by().iter().map(|e| e.text()).collect();
        assert_eq!(described, ["Changes apply at once."], "{kind:?}");
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
    let (screen, _) = page(DialogKind::Modal, noop);
    screen.get_by_role(Role::Button, "Open").click();
    let dialog = screen.get_by_role(Role::Dialog, "Settings");
    assert!(
        dialog.described_by().is_empty(),
        "no description, no relation"
    );
}

#[test]
fn the_focus_starts_on_the_first_focusable_node_or_on_the_initial_focus() {
    for kind in kinds() {
        let (screen, _) = page(kind, noop);
        screen.get_by_role(Role::Button, "Open").focus();
        screen.get_by_role(Role::Button, "Open").click();
        assert_eq!(
            screen.focused().map(|e| e.name()),
            Some("First".to_string()),
            "{kind:?}: first focusable"
        );
    }
    for kind in [DialogKind::Modal, DialogKind::Popover] {
        let (screen, open) = mount(move || {
            let second = node_ref();
            let open = signal(false);
            let anchor = node_ref();
            let behavior = use_dialog(open, kind).initial_focus(second);
            let behavior = if kind == DialogKind::Popover {
                behavior.anchor(anchor)
            } else {
                behavior
            };
            (
                open,
                column((
                    button("Open")
                        .node_ref(anchor)
                        .on_press(move || open.set(true)),
                    framed(
                        behavior,
                        kind,
                        anchor,
                        "Settings",
                        Frame::default(),
                        move || column((button("First"), button("Second").node_ref(second))),
                        || (),
                    ),
                )),
            )
        });
        screen.get_by_role(Role::Button, "Open").click();
        assert!(open.get_untracked());
        assert_eq!(
            screen.focused().map(|e| e.name()),
            Some("Second".to_string()),
            "{kind:?}: initial focus"
        );
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn escape_closes_every_kind_and_the_focus_returns_to_the_opener() {
    for kind in kinds() {
        let (screen, open) = page(kind, noop);
        let opener = screen.get_by_role(Role::Button, "Open");
        opener.focus();
        opener.click();
        assert!(open.get_untracked(), "{kind:?}");
        screen.press_key(Key::Escape);
        assert!(!open.get_untracked(), "{kind:?}: escape closes");
        assert!(screen.query_by_role(role_of(kind), "Settings").is_none());
        assert!(opener.is_focused(), "{kind:?}: focus returns to the opener");
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn assistive_dismiss_closes_every_kind_and_the_focus_returns() {
    for kind in kinds() {
        let (screen, open) = page(kind, noop);
        let opener = screen.get_by_role(Role::Button, "Open");
        opener.focus();
        opener.click();
        screen
            .get_by_role(role_of(kind), "Settings")
            .a11y(A11yAction::Dismiss);
        assert!(!open.get_untracked(), "{kind:?}: dismiss closes");
        assert!(opener.is_focused(), "{kind:?}: focus returns to the opener");
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn an_outside_press_closes_all_but_an_alert_modal() {
    for kind in kinds() {
        let (screen, open) = page(kind, noop);
        screen.get_by_role(Role::Button, "Open").click();
        let inside = screen.get_by_role(Role::Button, "Second");
        inside.click();
        assert!(open.get_untracked(), "{kind:?}: a press inside keeps it");
        // The window corner the panel is farthest from.
        let panel = screen
            .get_by_role(Role::Heading, "Settings")
            .rect()
            .unwrap();
        let x = if panel.x > 400.0 { 5.0 } else { 795.0 };
        let y = if panel.y > 300.0 { 5.0 } else { 595.0 };
        screen.click_at(x, y);
        let expected = kind == DialogKind::AlertModal;
        assert_eq!(
            open.get_untracked(),
            expected,
            "{kind:?}: outside press {}",
            if expected { "does nothing" } else { "closes" }
        );
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn dismiss_chooses_which_requests_close_it() {
    // An alert modal can be told to close on an outside press.
    let (screen, open) = page(DialogKind::AlertModal, |d| {
        d.dismiss(Dismiss {
            escape: false,
            outside: true,
        })
    });
    screen.get_by_role(Role::Button, "Open").click();
    screen.press_key(Key::Escape);
    assert!(open.get_untracked(), "escape is off");
    screen.click_at(790.0, 590.0);
    assert!(!open.get_untracked(), "outside is on");

    // The Dismiss action of assistive technology cannot be switched off.
    let (screen, open) = page(DialogKind::Modal, |d| {
        d.dismiss(Dismiss {
            escape: false,
            outside: false,
        })
    });
    screen.get_by_role(Role::Button, "Open").click();
    screen.press_key(Key::Escape);
    screen.click_at(790.0, 590.0);
    assert!(open.get_untracked(), "neither escape nor outside closes");
    screen
        .get_by_role(Role::Dialog, "Settings")
        .a11y(A11yAction::Dismiss);
    assert!(!open.get_untracked(), "assistive technology always can");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_popover_sits_below_its_anchor_and_leaves_the_page_usable() {
    let (screen, open) = page(DialogKind::Popover, noop);
    let anchor = screen.get_by_role(Role::Button, "Open");
    anchor.click();
    let popover = screen.get_by_role(Role::Dialog, "Settings");
    let (a, p) = (anchor.rect().unwrap(), popover.rect().unwrap());
    assert!(p.y >= a.y + a.height, "below the anchor: {p:?} vs {a:?}");
    assert!((p.x - a.x).abs() < 1.0, "lined up with the anchor's start");
    // Not modal: a press on the page behind goes through and closes it.
    screen.get_by_role(Role::Button, "Far").click();
    assert!(!open.get_untracked(), "an outside press closes it");
    assert!(
        screen.get_by_role(Role::Button, "Far").is_focused(),
        "the press went through to the page behind and took the focus"
    );
    // A press on the anchor is not outside: it toggles.
    anchor.click();
    assert!(open.get_untracked());
    anchor.click();
    assert!(!open.get_untracked(), "the anchor toggles it");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_popover_closes_when_the_focus_leaves_it_and_its_anchor() {
    let (screen, open) = page(DialogKind::Popover, noop);
    screen.get_by_role(Role::Button, "Open").click();
    assert_eq!(
        screen.focused().map(|e| e.name()),
        Some("First".to_string())
    );
    screen.press_tab(false);
    screen.press_tab(false);
    screen.press_tab(false);
    assert_eq!(screen.focused().map(|e| e.name()), Some("Open".to_string()));
    assert!(
        open.get_untracked(),
        "the anchor is part of the focus range"
    );
    screen.press_tab(false);
    assert_eq!(
        screen.focused().map(|e| e.name()),
        Some("Behind".to_string())
    );
    assert!(!open.get_untracked(), "Tab out of both closes it");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_modal_contains_tab_and_blocks_the_page_behind_it() {
    for kind in [
        DialogKind::Modal,
        DialogKind::AlertModal,
        DialogKind::Drawer(Side::Right),
    ] {
        let (screen, _) = page(kind, noop);
        screen.get_by_role(Role::Button, "Open").click();
        let mut seen = Vec::new();
        for _ in 0..6 {
            seen.push(screen.focused().map(|e| e.name()).unwrap_or_default());
            screen.press_tab(false);
        }
        assert_eq!(
            seen,
            ["First", "Second", "Done", "First", "Second", "Done"],
            "{kind:?}: Tab wraps inside"
        );
        screen.press_tab(true);
        assert_eq!(
            screen.focused().map(|e| e.name()),
            Some("Done".to_string()),
            "{kind:?}: Shift+Tab wraps backwards"
        );
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn a_modal_blocks_the_page_behind_it() {
    let (screen, open) = page(DialogKind::Modal, |d| {
        d.dismiss(Dismiss {
            escape: true,
            outside: false,
        })
    });
    screen.get_by_role(Role::Button, "Open").click();
    let behind = screen.get_by_role(Role::Button, "Behind").rect().unwrap();
    screen.click_at(behind.x + 10.0, behind.y + 10.0);
    assert!(open.get_untracked());
    assert_ne!(
        screen.focused().map(|e| e.name()),
        Some("Behind".to_string()),
        "the press does not reach the page"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_drawer_sits_at_its_edge() {
    type Rect = kivact::draw::Rect;
    type AtEdge = fn(Rect, Rect) -> bool;
    // The heading is at the start of the panel and "Done" at its end.
    let cases: [(Side, AtEdge); 4] = [
        (Side::Right, |_, done| done.x + done.width >= 760.0),
        (Side::Left, |heading, _| heading.x <= 40.0),
        (Side::Top, |heading, _| heading.y <= 40.0),
        (Side::Bottom, |_, done| done.y + done.height >= 560.0),
    ];
    for (side, at_edge) in cases {
        let (screen, _) = page(DialogKind::Drawer(side), noop);
        screen.get_by_role(Role::Button, "Open").click();
        let heading = screen
            .get_by_role(Role::Heading, "Settings")
            .rect()
            .unwrap();
        let done = screen.get_by_role(Role::Button, "Done").rect().unwrap();
        assert!(at_edge(heading, done), "{side:?}: {heading:?} {done:?}");
    }
}

#[test]
fn the_footer_is_left_out_when_it_shows_nothing() {
    let (screen, _) = mount(|| {
        let open = signal(true);
        (
            (),
            framed(
                use_dialog(open, DialogKind::Modal),
                DialogKind::Modal,
                node_ref(),
                "Notice",
                Frame::default(),
                || text("Saved."),
                || (),
            ),
        )
    });
    screen.get_by_role(Role::Dialog, "Notice");
    screen.get_by_text("Saved.");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_popover_without_an_anchor_is_a_usage_error() {
    let caught = Rc::new(RefCell::new(Vec::<kivact::UiError>::new()));
    let seen = caught.clone();
    let before = line!();
    let (screen, ()) = mount(move || {
        let open = signal(true);
        (
            (),
            error_boundary(move || {
                let orphan = use_dialog(open, DialogKind::Popover);
                show(orphan.shown(), move || {
                    overlay(node_ref(), Placement::Center, node())
                        .part(orphan.content())
                        .name("Orphan")
                })
            })
            .on_error(move |e| seen.borrow_mut().push(e)),
        )
    });
    let after = line!();
    assert!(screen.query_by_role(Role::Dialog, "Orphan").is_none());
    let caught = caught.borrow();
    let error = caught.iter().find(|e| e.message().contains("anchor"));
    assert!(error.is_some(), "{caught:?}");
    let at = error.unwrap().location().expect("located misuse");
    assert_eq!(at.file(), file!());
    assert!((before..after).contains(&at.line()));
}

#[test]
fn the_close_button_part_closes_and_returns_the_focus() {
    for kind in kinds() {
        let frame = Frame {
            close_button: true,
            ..Frame::default()
        };
        let (screen, open) = page_with(kind, noop, frame);
        let opener = screen.get_by_role(Role::Button, "Open");
        opener.focus();
        opener.click();
        screen.get_by_role(Role::Button, "Close").click();
        assert!(!open.get_untracked(), "{kind:?}: the button closes it");
        assert!(opener.is_focused(), "{kind:?}: focus returns to the opener");
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn a_button_that_closes_it_returns_the_focus_for_every_kind() {
    for kind in kinds() {
        let (screen, open) = page(kind, noop);
        let opener = screen.get_by_role(Role::Button, "Open");
        opener.focus();
        opener.click();
        let done = screen.get_by_role(Role::Button, "Done");
        done.focus();
        done.click();
        assert!(!open.get_untracked(), "{kind:?}");
        assert!(
            opener.is_focused(),
            "{kind:?}: closing with a footer button returns the focus"
        );
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn closing_while_the_focus_is_outside_leaves_the_focus_alone() {
    // A popover with nothing to focus, opened from code while "Behind" has
    // the focus: the focus never enters it, so closing does not move it.
    let (screen, open) = mount(|| {
        let open = signal(false);
        let anchor = node_ref();
        (
            open,
            column((
                button("Open").node_ref(anchor),
                button("Behind"),
                framed(
                    use_dialog(open, DialogKind::Popover).anchor(anchor),
                    DialogKind::Popover,
                    anchor,
                    "Note",
                    Frame::default(),
                    || text("Nothing to focus"),
                    || (),
                ),
            )),
        )
    });
    let behind = screen.get_by_role(Role::Button, "Behind");
    behind.focus();
    open.set(true);
    screen.run_until_idle();
    screen.get_by_role(Role::Dialog, "Note");
    assert!(behind.is_focused());
    open.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Dialog, "Note").is_none());
    assert!(behind.is_focused(), "the focus stays where it was");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_outside_press_that_closes_a_popover_keeps_the_pressed_focus() {
    let (screen, open) = page(DialogKind::Popover, noop);
    screen.get_by_role(Role::Button, "Open").click();
    screen.get_by_role(Role::Button, "Far").click();
    assert!(!open.get_untracked());
    assert!(screen.get_by_role(Role::Button, "Far").is_focused());
}

fn entering() -> Frame {
    Frame {
        enter: Some(
            style()
                .enter(style().opacity(0.0).scale(0.5).translate(0.0, 40.0))
                .transition(
                    kivact::style::TransitionProp::Paint,
                    Transition::tween(100, Easing::Linear),
                ),
        ),
        ..Frame::default()
    }
}

fn heading_rect(screen: &Screen) -> kivact::draw::Rect {
    screen
        .get_by_role(Role::Heading, "Settings")
        .rect()
        .expect("laid out")
}

#[test]
fn a_panel_comes_in_from_its_enter_values_each_time_it_opens() {
    for kind in [
        DialogKind::Modal,
        DialogKind::Drawer(Side::Right),
        DialogKind::Popover,
    ] {
        let (screen, open) = page(kind, noop);
        screen.get_by_role(Role::Button, "Open").click();
        let end = heading_rect(&screen);
        screen.get_by_role(Role::Button, "Done").click();
        assert!(!open.get_untracked());

        let (screen, open) = page_with(kind, noop, entering());
        for round in 0..2 {
            screen.get_by_role(Role::Button, "Open").click();
            let start = heading_rect(&screen);
            assert!(
                start.width < end.width * 0.6,
                "{kind:?} #{round}: starts scaled down: {start:?} vs {end:?}"
            );
            assert!(
                start.y > end.y + 20.0,
                "{kind:?} #{round}: starts offset: {start:?} vs {end:?}"
            );
            screen.advance(Duration::from_millis(50));
            let mid = heading_rect(&screen);
            assert!(
                mid.width > start.width && mid.width < end.width,
                "{kind:?} #{round}: on its way: {mid:?}"
            );
            screen.advance(Duration::from_millis(100));
            let done = heading_rect(&screen);
            assert!(
                (done.width - end.width).abs() < 0.5 && (done.y - end.y).abs() < 0.5,
                "{kind:?} #{round}: arrives: {done:?} vs {end:?}"
            );
            screen.get_by_role(Role::Button, "Done").click();
            assert!(!open.get_untracked());
        }
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn reduced_motion_shows_the_end_state_at_once() {
    let kind = DialogKind::Modal;
    let (screen, _) = page(kind, noop);
    screen.get_by_role(Role::Button, "Open").click();
    let end = heading_rect(&screen);

    let (screen, _) = page_with(kind, noop, entering());
    screen.set_reduced_motion(true);
    screen.get_by_role(Role::Button, "Open").click();
    let shown = heading_rect(&screen);
    assert!(
        (shown.width - end.width).abs() < 0.5 && (shown.y - end.y).abs() < 0.5,
        "{shown:?} vs {end:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_hidden_title_keeps_the_name_and_renders_no_heading() {
    for kind in kinds() {
        let frame = Frame {
            hidden_title: true,
            ..Frame::default()
        };
        let (screen, _open) = page_with(kind, noop, frame);
        let opener = screen.get_by_role(Role::Button, "Open");
        opener.focus();
        opener.click();
        let role = if kind == DialogKind::AlertModal {
            Role::AlertDialog
        } else {
            Role::Dialog
        };
        screen.get_by_role(role, "Settings");
        assert!(
            screen.query_by_role(Role::Heading, "Settings").is_none(),
            "{kind:?}: no heading is rendered"
        );
        assert!(
            screen.errors().is_empty(),
            "{kind:?}: {:?}",
            screen.errors()
        );
    }
}

#[test]
fn a_modal_removed_with_an_external_branch_returns_to_the_actual_opener() {
    let (screen, open) = page_opener_second(DialogKind::Modal, noop);
    let opener = screen.get_by_role(Role::Button, "Open");
    opener.focus();
    opener.click();
    screen.get_by_role(Role::Button, "Second").focus();
    // Removing the shown branch from outside produces no dismissal callback:
    // the overlay's own cleanup returns the focus. Without it the focus would
    // go to the next tab stop, "Behind".
    open.set(false);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Dialog, "Settings").is_none());
    assert!(opener.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_popover_dismissed_by_assistive_technology_returns_to_its_anchor() {
    let (screen, open) = page_opener_second(DialogKind::Popover, noop);
    let anchor = screen.get_by_role(Role::Button, "Open");
    anchor.focus();
    anchor.click();
    screen.get_by_role(Role::Button, "Second").focus();
    screen
        .get_by_role(Role::Dialog, "Settings")
        .a11y(A11yAction::Dismiss);
    assert!(!open.get_untracked());
    assert!(anchor.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_refused_outside_press_keeps_a_later_programmatic_close_from_returning_the_focus() {
    // Without an outside attempt the popover is removed from outside and the
    // focus inside it goes back to the anchor, not to the next tab stop.
    let (screen, open) = page_opener_second(DialogKind::Popover, noop);
    let anchor = screen.get_by_role(Role::Button, "Open");
    anchor.focus();
    anchor.click();
    screen.get_by_role(Role::Button, "Second").focus();
    open.set(false);
    screen.run_until_idle();
    assert!(anchor.is_focused(), "an unmount returns to the anchor");

    // An outside press the dismiss rules refuse still keeps the pressed
    // destination for the rest of that opening: a later close from outside
    // does not return the focus, which goes to the next tab stop.
    let (screen, open) = page_opener_second(DialogKind::Popover, |d| {
        d.dismiss(Dismiss {
            escape: true,
            outside: false,
        })
    });
    let anchor = screen.get_by_role(Role::Button, "Open");
    anchor.focus();
    anchor.click();
    screen.click_at(795.0, 595.0);
    assert!(open.get_untracked(), "the outside press is refused");
    screen.get_by_role(Role::Button, "Second").focus();
    open.set(false);
    screen.run_until_idle();
    assert!(!anchor.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
