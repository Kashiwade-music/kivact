//! Role: Verify overlay interaction through the Soft public API.
use kivact::control::{NodeBuilder, column, row};
use kivact::signal;
use kivact::style::{ThemeMode, style};
use kivact::test::{RenderOptions, WindowMode, render_with};
use kivact::ui::button;
use kivact::view::{Key, Role, node_ref};
use kivact_ui_soft::{DialogPurpose, ShowOn, bottom_sheet, media_overlay};
use std::cell::Cell;
use std::rc::Rc;

fn options() -> RenderOptions {
    RenderOptions {
        theme: kivact_ui_soft::theme(ThemeMode::Light),
        window: WindowMode::Headless,
        ..RenderOptions::default()
    }
}

#[test]
fn closed_media_overlay_passes_pointer_and_keyboard_to_base() {
    let presses = Rc::new(Cell::new(0));
    let base_presses = presses.clone();
    let screen = render_with(options(), move || {
        media_overlay(|| button("Overlay action"))
            .open(false)
            .attach(
                button("Base")
                    .style(style().size(120, 60))
                    .on_press(move || {
                        base_presses.set(base_presses.get() + 1);
                    }),
            )
    });
    assert!(
        screen
            .query_by_role(Role::Button, "Overlay action")
            .is_none()
    );
    screen.press_key(Key::Tab);
    assert_eq!(screen.focused().unwrap().name(), "Base");
    screen.get_by_role(Role::Button, "Base").click();
    assert_eq!(presses.get(), 1);
}

#[test]
fn focus_mode_reveals_the_overlay_even_with_a_noninteractive_base() {
    let screen = render_with(options(), || {
        media_overlay(|| button("Overlay action"))
            .show_on(ShowOn::Focus)
            .attach(kivact::control::node().style(style().size(120, 60)))
    });
    assert!(
        screen
            .query_by_role(Role::Button, "Overlay action")
            .is_none()
    );
    screen.press_key(Key::Tab);
    assert!(
        screen
            .query_by_role(Role::Button, "Overlay action")
            .is_some()
    );
    screen.press_key(Key::Tab);
    assert_eq!(screen.focused().unwrap().name(), "Overlay action");
}

#[test]
fn sheet_escape_honors_purpose_and_programmatic_close_restores_final_focus() {
    for purpose in [
        DialogPurpose::Info,
        DialogPurpose::Form,
        DialogPurpose::Required,
    ] {
        let screen = render_with(options(), move || {
            let open = signal(false);
            let final_focus = node_ref();
            column((
                row((
                    button("Open sheet").on_press(move || open.set(true)),
                    button("Final focus").node_ref(final_focus),
                )),
                bottom_sheet("Sheet", open)
                    .purpose(purpose)
                    .final_focus(final_focus)
                    .content(move || {
                        button("Programmatic close").on_press(move || open.set(false))
                    }),
            ))
        });
        screen.get_by_role(Role::Button, "Open sheet").click();
        let role = if purpose == DialogPurpose::Required {
            Role::AlertDialog
        } else {
            Role::Dialog
        };
        assert!(screen.query_by_role(role, "Sheet").is_some());
        assert_eq!(screen.focused().unwrap().name(), "Programmatic close");
        screen.press_key(Key::Escape);
        if purpose == DialogPurpose::Required {
            assert!(screen.query_by_role(role, "Sheet").is_some());
            screen.press_key(Key::Enter);
        }
        assert!(screen.query_by_role(role, "Sheet").is_none());
        assert_eq!(screen.focused().unwrap().name(), "Final focus");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
