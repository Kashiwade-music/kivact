//! Role: Verify native Soft menu press and drag selection.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use kivact::interaction::MenuEntry;
use kivact::style::ThemeMode;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::ui::menu;
use kivact::view::MouseButton;
use kivact::view::Role;

#[test]
fn soft_mouse_opens_on_press_and_drag_release_selects_once() {
    let chosen = Rc::new(Cell::new(Vec::new()));
    let report = chosen.clone();
    let screen = render_with(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)),
        move || {
            menu(
                "Actions",
                vec![
                    MenuEntry::item("open", "Open"),
                    MenuEntry::item("copy", "Copy"),
                ],
            )
            .on_activate(move |key| {
                let mut entries = report.take();
                entries.push(key);
                report.set(entries);
            })
        },
    );

    let trigger = screen.get_by_role(Role::Button, "Actions").rect().unwrap();
    screen.mouse_move(
        trigger.x + trigger.width * 0.5,
        trigger.y + trigger.height * 0.5,
    );
    screen.mouse_down(MouseButton::Primary);
    assert!(screen.query_by_role(Role::MenuItem, "Copy").is_some());
    screen.advance(Duration::from_millis(180));
    let copy = screen.get_by_role(Role::MenuItem, "Copy").rect().unwrap();
    screen.mouse_move(copy.x + copy.width * 0.5, copy.y + copy.height * 0.5);
    screen.mouse_up(MouseButton::Primary);
    assert_eq!(chosen.take(), vec!["copy"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
