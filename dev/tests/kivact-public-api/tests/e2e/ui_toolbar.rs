//! Role: Verify the public toolbar, toggle group and joined action group contracts.
use crate::support::mount;
use kivact::prelude::*;
use kivact::ui::{
    ButtonGroupMember, button_group, button_group_separator, button_group_text, toggle_group,
    toolbar,
};
use kivact::view::A11yAction;
use kivact::view::Orientation;

#[test]
fn toolbar_moves_focus_with_arrows_as_one_tab_stop() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                button("Before"),
                toolbar("Edit")
                    .start((button("Cut"), button("Copy")))
                    .end((button("Paste"),)),
                button("After"),
            )),
        )
    });
    screen.get_by_role(Role::Toolbar, "Edit");
    screen.get_by_role(Role::Button, "Before").focus();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "Cut").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::Button, "Copy").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::Button, "Paste").is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    screen.press_tab(true);
    assert!(screen.get_by_role(Role::Button, "Paste").is_focused());
    screen.press_key(Key::ArrowLeft);
    assert!(screen.get_by_role(Role::Button, "Copy").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn toggle_group_single_and_multiple_selection() {
    let (screen, (single, multiple)) = mount(|| {
        let single = signal(None::<&'static str>);
        let multiple = signal(vec!["hidden"]);
        (
            (single, multiple),
            column((
                toggle_group(
                    "Alignment",
                    vec![
                        ListItem::new("left", "Left"),
                        ListItem::new("right", "Right")
                            .with_icon("dot")
                            .with_pressed_icon("check"),
                    ],
                    single,
                ),
                toggle_group(
                    "Formatting",
                    vec![
                        ListItem::new("bold", "Bold"),
                        ListItem::new("italic", "Italic"),
                        ListItem::new("locked", "Locked").disabled(true),
                    ],
                    multiple,
                ),
            )),
        )
    });
    screen.get_by_role(Role::Group, "Alignment");
    screen.get_by_role(Role::Button, "Left").click();
    assert_eq!(single.get_untracked(), Some("left"));
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::Button, "Right").is_focused());
    assert_eq!(single.get_untracked(), Some("left"));
    screen.press_key(Key::Space);
    assert_eq!(single.get_untracked(), Some("right"));
    assert_eq!(
        screen.get_by_role(Role::Button, "Right").checked(),
        Some(CheckedState::Checked)
    );
    screen.press_key(Key::Enter);
    assert_eq!(single.get_untracked(), None);
    screen
        .get_by_role(Role::Button, "Bold")
        .a11y(A11yAction::Activate);
    screen.get_by_role(Role::Button, "Italic").click();
    assert_eq!(multiple.get_untracked(), vec!["hidden", "bold", "italic"]);
    screen.get_by_role(Role::Button, "Bold").click();
    assert_eq!(multiple.get_untracked(), vec!["hidden", "italic"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn button_group_is_one_tab_stop() {
    let (screen, ()) = mount(|| {
        (
            (),
            column((
                button("Before"),
                button_group(
                    "Actions",
                    (
                        ButtonGroupMember::from(button("Archive"))
                            .solid(|| true)
                            .keep_size(true),
                        button_group_separator().test_id("action-divider"),
                        button_group_text("or"),
                        button("Delete"),
                    ),
                )
                .orientation(Orientation::Vertical),
                button("After"),
            )),
        )
    });
    assert_eq!(
        screen.get_by_test_id("action-divider").orientation(),
        Orientation::Horizontal
    );
    screen.get_by_role(Role::Button, "Before").focus();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "Archive").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Button, "Delete").is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "After").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn group_orientation_updates_layout_and_roving_without_losing_selection() {
    let (screen, (axis, selection)) = mount(|| {
        let axis = signal(Orientation::Horizontal);
        let selection = signal(Some("a"));
        (
            (axis, selection),
            column((
                toggle_group(
                    "Reactive toggles",
                    vec![
                        ListItem::new("a", "Choice A"),
                        ListItem::new("b", "Choice B"),
                    ],
                    selection,
                )
                .orientation(axis),
                button_group(
                    "Reactive actions",
                    (
                        button("First action"),
                        button_group_separator().test_id("reactive-divider"),
                        button("Last action"),
                    ),
                )
                .orientation(axis),
            )),
        )
    });
    let first = screen.get_by_role(Role::Button, "Choice A");
    first.focus();
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::Button, "Choice B").is_focused());
    axis.set(Orientation::Vertical);
    screen.run_until_idle();
    assert_eq!(selection.get_untracked(), Some("a"));
    assert_eq!(
        screen.get_by_test_id("reactive-divider").orientation(),
        Orientation::Horizontal
    );
    screen.press_key(Key::ArrowUp);
    assert!(first.is_focused());
    screen.get_by_role(Role::Button, "First action").focus();
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Button, "Last action").is_focused());
    axis.set(Orientation::Horizontal);
    screen.run_until_idle();
    assert_eq!(
        screen.get_by_test_id("reactive-divider").orientation(),
        Orientation::Vertical
    );
    screen.press_key(Key::ArrowLeft);
    assert!(
        screen
            .get_by_role(Role::Button, "First action")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn toggle_group_icon_only_keeps_iconless_items_out_of_the_square_look() {
    let (screen, ()) = mount(|| {
        (
            (),
            toggle_group(
                "Tools",
                vec![
                    ListItem::new("pen", "Pen").with_icon("dot"),
                    ListItem::new("text", "Text"),
                ],
                signal(None::<&'static str>),
            )
            .icon_only(true),
        )
    });
    let pen = screen
        .get_by_role(Role::Button, "Pen")
        .rect()
        .expect("laid out");
    assert!((pen.width - pen.height).abs() <= 0.5, "{pen:?}");
    let text = screen
        .get_by_role(Role::Button, "Text")
        .rect()
        .expect("laid out");
    assert!((text.width - text.height).abs() > 0.5, "{text:?}");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
