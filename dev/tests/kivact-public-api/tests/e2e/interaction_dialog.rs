//! Role: Tests for Dialog component interaction and accessibility.
//!
//! The unstyled `dialog`: the four kinds with their roles and relations, the
//! title and description parts, where the focus starts and returns, the three
//! ways to close it, and the modal containing Tab. The tests draw the frame
//! (scrim, panel, heading, body and footer) themselves, as a kit does.

use kivact::view::View;

use crate::support::mount;
use kivact::interaction::{DialogApi, use_dialog};
use kivact::prelude::*;
use kivact::style::Position;
use kivact::style::{Justify, Style, pct};
use kivact::test::Screen;
use kivact::interaction::{DialogKind, Side};
use kivact::view::{Placement, overlay};

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
        let footer = (!kivact::custom::view_is_empty(&footer))
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
fn a_popover_anchored_after_its_content_is_made_opens_and_returns_the_focus_to_the_anchor() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        let anchor = node_ref();
        let behavior = use_dialog(open, DialogKind::Popover);
        // The surface part exists before the anchor is given.
        let content = std::cell::RefCell::new(Some(behavior.content()));
        let hint = show(behavior.shown(), move || {
            let content = content
                .borrow_mut()
                .take()
                .expect("invariant: the popover opens once");
            overlay(anchor, Placement::Below, column((button("First"),)))
                .part(content)
                .name("Settings")
        });
        let _ = behavior.anchor(anchor);
        let opener = button("Open")
            .node_ref(anchor)
            .on_press(move || open.update(|v| *v = !*v));
        (open, column((opener, hint)))
    });
    screen.get_by_test_id("Open").click();
    assert!(open.get_untracked());
    screen.get_by_role(Role::Dialog, "Settings");
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(screen.get_by_test_id("Open").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
