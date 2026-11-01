//! Role: Tests the list of `kivact::ui` on real windows.
//!
//! Its roles and names, the keys and the pointer that select, single and
//! multiple selection, the edits it reports and its item slot, through the
//! public API.

use std::cell::RefCell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::prelude::*;
use kivact::test::{RenderOptions, WindowMode};
use kivact::ui::EditPhase;
use kivact::view::{A11yAction, Modifiers};

use EditPhase::{Begin, Commit};

type Log = Rc<RefCell<Vec<EditPhase>>>;

fn options() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}

fn items() -> Vec<ListItem<&'static str>> {
    vec![
        ListItem::new("red", "Red"),
        ListItem::new("green", "Green").description("The calm one"),
        ListItem::new("blue", "Blue").disabled(true),
        ListItem::new("pink", "Pink"),
    ]
}

#[test]
fn a_list_selects_by_keys_pointer_and_invoke_each_change_one_edit() {
    let log = Log::default();
    let seen = log.clone();
    let (screen, selected) = mount_with(options(), move || {
        let selected = signal(Some("green"));
        let view = column((
            button("Before"),
            list("Colors", items(), selected).on_edit(move |phase| seen.borrow_mut().push(phase)),
        ));
        (selected, view)
    });
    let colors = screen.get_by_role(Role::ListBox, "Colors");
    assert_eq!(colors.children().len(), 4);
    assert!(!colors.is_multiselectable());
    assert!(screen.get_by_role(Role::Option, "Green").is_selected());
    screen.find_by_text("The calm one");
    // Tab enters at the selected item; the arrows select and skip the
    // disabled one.
    screen.get_by_role(Role::Button, "Before").focus();
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Option, "Green").is_focused());
    screen.press_key(Key::ArrowDown);
    assert_eq!(selected.get_untracked(), Some("pink"));
    assert!(screen.get_by_role(Role::Option, "Pink").is_focused());
    screen.press_key(Key::ArrowUp);
    assert_eq!(selected.get_untracked(), Some("green"));
    screen.get_by_role(Role::Option, "Red").click();
    assert_eq!(selected.get_untracked(), Some("red"));
    screen
        .get_by_role(Role::Option, "Pink")
        .a11y(A11yAction::Select);
    assert_eq!(selected.get_untracked(), Some("pink"));
    assert!(screen.get_by_role(Role::Option, "Blue").is_disabled());
    assert_eq!(
        *log.borrow(),
        [Begin, Commit, Begin, Commit, Begin, Commit, Begin, Commit],
        "each change is a whole edit; nothing is left to cancel"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_multiple_list_toggles_with_ctrl_and_extends_with_shift_in_its_own_binding() {
    let (screen, (single, picked)) = mount_with(options(), || {
        let single = signal(None::<&'static str>);
        let picked = signal(Vec::<&'static str>::new());
        (
            (single, picked),
            list("Colors", items(), single).multiple(picked),
        )
    });
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    assert!(
        screen
            .get_by_role(Role::ListBox, "Colors")
            .is_multiselectable()
    );
    screen.get_by_role(Role::Option, "Red").click();
    assert_eq!(picked.get_untracked(), ["red"]);
    // Ctrl+Space toggles the focused item.
    screen.press_key_with(Key::Space, ctrl);
    assert!(picked.get_untracked().is_empty());
    screen.press_key_with(Key::Space, ctrl);
    assert_eq!(picked.get_untracked(), ["red"]);
    // Shift+Arrow extends the selection over the next items, skipping the
    // disabled one.
    screen.press_key_with(
        Key::ArrowDown,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(picked.get_untracked(), ["red", "green"]);
    assert_eq!(single.get_untracked(), None, "the single binding is unused");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn an_item_slot_replaces_what_an_item_shows_and_follows_the_selection() {
    let (screen, selected) = mount_with(options(), || {
        let selected = signal(Some("red"));
        let view = list("Colors", items(), selected).item(|item, state| {
            text(move || {
                let mark = if state.get().selected { "> " } else { "" };
                format!("{mark}{}", item.get().label())
            })
        });
        (selected, view)
    });
    screen.get_by_text("> Red");
    screen.get_by_role(Role::Option, "Pink").click();
    assert_eq!(selected.get_untracked(), Some("pink"));
    screen.get_by_text("> Pink");
    assert!(screen.query_by_text("> Red").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_list_is_drawn_with_the_theme_and_the_callers_style_wins() {
    let (screen, ()) = mount_with(options(), || {
        (
            (),
            list("Colors", items(), signal(Some("red")))
                .style(style().width(240))
                .test_id("colors"),
        )
    });
    assert_eq!(
        screen
            .get_by_test_id("colors")
            .rect()
            .expect("laid out")
            .width,
        240.0
    );
    let red = screen
        .get_by_role(Role::Option, "Red")
        .rect()
        .expect("laid out");
    let green = screen
        .get_by_role(Role::Option, "Green")
        .rect()
        .expect("laid out");
    assert!(green.y > red.y, "the items are stacked in a column");
    screen.assert_layout();
}
