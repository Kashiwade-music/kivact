//! Role: Tests verify external components through the native window driver.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use kivact::prelude::*;
use kivact::primitive::{NodeRefExt, NodeToken, Owned, Owner};
use kivact::test::{RenderOptions, Screen, WindowMode, render_with};
use kivact::ui::{ListItem, MenuEntry};
use kivact::view::A11yAction;
use kivact::view::Modifiers;
use kivact_external_design_system::{button, dialog, menu, select};

fn render(root: impl FnOnce() -> Node + 'static) -> Screen {
    render_with(
        RenderOptions {
            window: WindowMode::Real,
            gpu: cfg!(feature = "gpu"),
            ..Default::default()
        },
        root,
    )
}

#[test]
fn a_custom_button_activates_once_per_native_action_and_respects_disabled() {
    let screen = render(|| {
        let count = signal(0);
        column((
            button("Save").on_press(move || count.update(|value| *value += 1)),
            button("Unavailable")
                .disabled(true)
                .on_press(move || count.update(|value| *value += 100)),
            text(move || count.get().to_string()).test_id("count"),
        ))
        .style(style().padding(24).gap(12))
    });

    let save = screen.get_by_role(Role::Button, "Save");
    save.click();
    assert_eq!(screen.get_by_test_id("count").text(), "1");
    save.press_key(Key::Enter);
    assert_eq!(screen.get_by_test_id("count").text(), "2");
    save.press_key(Key::Space);
    assert_eq!(screen.get_by_test_id("count").text(), "3");
    save.a11y(A11yAction::Activate);
    assert_eq!(screen.get_by_test_id("count").text(), "4");
    screen
        .get_by_role(Role::Button, "Unavailable")
        .force_click();
    assert_eq!(screen.get_by_test_id("count").text(), "4");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[derive(Clone, Copy)]
struct ContentSnapshot {
    owner: Owner,
    alive: Owned<()>,
    local: Signal<u32>,
    first: NodeToken,
}

#[test]
fn a_custom_modal_traps_focus_restores_the_opener_and_disposes_each_opening() {
    let snapshot = Rc::new(RefCell::new(None::<ContentSnapshot>));
    let mounted = snapshot.clone();
    let cleanups = Rc::new(Cell::new(0));
    let cleaned = cleanups.clone();
    let screen = render(move || {
        let open = signal(false);
        column((
            button("Open settings").on_press(move || open.set(true)),
            button("Behind"),
            dialog(open, "Settings", move || {
                let owner = Owner::current("external dialog content");
                let alive = owner.store("external dialog content", Rc::new(()));
                let local = signal(0);
                let first = node_ref();
                let mounted = mounted.clone();
                on_mount(move || {
                    *mounted.borrow_mut() = Some(ContentSnapshot {
                        owner,
                        alive,
                        local,
                        first: first.mounted_node().expect("the content is mounted"),
                    });
                });
                let cleaned = cleaned.clone();
                owner.on_cleanup(move || cleaned.set(cleaned.get() + 1));
                column((
                    button("First")
                        .node_ref(first)
                        .on_press(move || local.update(|value| *value += 1)),
                    button("Done").on_press(move || open.set(false)),
                    text(move || local.get().to_string()).test_id("local"),
                ))
                .into_view()
            }),
        ))
        .style(style().padding(24).gap(12))
    });

    let opener = screen.get_by_role(Role::Button, "Open settings");
    let mut previous_node = None;
    for opening in 1..=2 {
        opener.focus();
        opener.click();
        screen.get_by_role(Role::Dialog, "Settings");
        let first = screen.get_by_role(Role::Button, "First");
        let done = screen.get_by_role(Role::Button, "Done");
        assert!(first.is_focused(), "initial focus enters the dialog");
        let content = snapshot.borrow().expect("on_mount captured the content");
        assert!(content.alive.try_get().is_some());
        assert_eq!(content.owner.focused_node(), Some(content.first));
        assert_ne!(
            Some(content.first),
            previous_node,
            "a fresh node is mounted"
        );
        previous_node = Some(content.first);
        assert_eq!(screen.get_by_test_id("local").text(), "0");
        first.click();
        assert_eq!(screen.get_by_test_id("local").text(), "1");
        screen.press_tab(false);
        assert!(done.is_focused());
        screen.press_tab(false);
        assert!(first.is_focused(), "Tab wraps inside the modal");
        screen.press_tab(true);
        assert!(done.is_focused(), "Shift+Tab wraps inside the modal");
        screen.press_key(Key::Escape);
        assert!(screen.query_by_role(Role::Dialog, "Settings").is_none());
        assert!(opener.is_focused(), "Escape restores the opener");
        assert!(content.alive.try_get().is_none());
        assert_eq!(content.local.try_get(), None, "content state is disposed");
        assert_eq!(cleanups.get(), opening);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}

#[test]
fn a_custom_select_shares_keyboard_choice_and_focus_behavior() {
    let screen = render(|| {
        let selected = signal(Some("alpha"));
        column((
            select(
                vec![
                    ListItem::new("alpha", "Alpha"),
                    ListItem::new("beta", "Beta"),
                    ListItem::new("gamma", "Gamma"),
                ],
                selected,
            ),
            text(move || selected.get().unwrap_or("none").to_owned()).test_id("selected"),
        ))
        .style(style().padding(24))
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Choice");
    trigger.focus();
    trigger.press_key(Key::ArrowDown);
    assert_eq!(screen.get_by_test_id("selected").text(), "beta");
    trigger.press_key_with(
        Key::ArrowDown,
        Modifiers {
            alt: true,
            ..Default::default()
        },
    );
    assert!(screen.get_by_role(Role::Option, "Beta").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Option, "Gamma").is_focused());
    screen.press_key(Key::Enter);
    assert_eq!(screen.get_by_test_id("selected").text(), "gamma");
    assert!(screen.query_by_role(Role::ListBox, "Choice").is_none());
    assert!(trigger.is_focused());
    trigger.click();
    screen.get_by_role(Role::ListBox, "Choice");
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::ListBox, "Choice").is_none());
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_custom_menu_wraps_searches_and_runs_submenu_actions_once() {
    let screen = render(|| {
        let count = signal(0);
        column((
            menu(
                "Actions",
                vec![
                    MenuEntry::item("alpha", "Alpha"),
                    MenuEntry::submenu("export", "Export", vec![MenuEntry::item("pdf", "PDF")]),
                    MenuEntry::item("quit", "Quit"),
                ],
                move |_| count.update(|value| *value += 1),
            ),
            text(move || count.get().to_string()).test_id("count"),
        ))
        .style(style().padding(24))
    });
    let trigger = screen.get_by_role(Role::Button, "Actions");
    trigger.press_key(Key::Enter);
    assert!(screen.get_by_role(Role::MenuItem, "Alpha").is_focused());
    screen.press_key(Key::ArrowUp);
    assert!(screen.get_by_role(Role::MenuItem, "Quit").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::MenuItem, "Alpha").is_focused());
    screen.press_key(Key::Char('e'));
    assert!(screen.get_by_role(Role::MenuItem, "Export").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(screen.get_by_role(Role::MenuItem, "PDF").is_focused());
    screen.press_key(Key::ArrowLeft);
    assert!(screen.query_by_role(Role::MenuItem, "PDF").is_none());
    assert!(screen.get_by_role(Role::MenuItem, "Export").is_focused());
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::Enter);
    assert_eq!(screen.get_by_test_id("count").text(), "1");
    assert!(screen.query_by_role(Role::Menu, "Actions").is_none());
    assert!(trigger.is_focused());
    trigger.press_key(Key::Enter);
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Menu, "Actions").is_none());
    assert!(trigger.is_focused());
    assert_eq!(screen.get_by_test_id("count").text(), "1");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_external_editor_uses_native_composition_selection_and_accessibility() {
    let state = Rc::new(Cell::new(None));
    let setup = state.clone();
    let screen = render(move || {
        let value = signal(String::from("a🦀"));
        let reference = node_ref();
        setup.set(Some((value, reference)));
        kivact_external_design_system::text_input("Independent editor", value).node_ref(reference)
    });
    let (value, reference) = state.get().unwrap();
    let field = screen.get_by_role(Role::TextBox, "Independent editor");
    field.focus();
    assert_eq!(field.text_selection(), Some((2, 2)));
    field.compose("仮");
    assert_eq!(value.get_untracked(), "a🦀");
    assert!(reference.edit_state().unwrap().is_composing());
    field.commit("確定");
    assert_eq!(value.get_untracked(), "a🦀確定");
    assert!(!reference.edit_state().unwrap().is_composing());
    assert_eq!(field.text_selection(), Some((4, 4)));
    assert_eq!(
        reference.text_layout(0..0).unwrap().caret,
        *screen.ime_text().unwrap().char_rects.last().unwrap()
    );
    field.a11y_with_value(A11yAction::SetValue, "from AT🦀");
    assert_eq!(value.get_untracked(), "from AT🦀");
    assert_eq!(field.text_selection(), Some((8, 8)));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
