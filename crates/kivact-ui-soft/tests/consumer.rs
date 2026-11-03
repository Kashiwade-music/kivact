//! Role: Verify the Soft theme through the public facade and kit APIs.
use kivact::control::NodeBuilder;
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, render_with};
use kivact::ui::button;
use kivact::view::Role;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn the_neutral_theme_mounts_standard_buttons_in_both_appearances() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let clicks = Rc::new(Cell::new(0));
        let pressed = Rc::clone(&clicks);
        let screen = render_with(
            RenderOptions {
                theme: kivact_ui_soft::theme(mode),
                ..RenderOptions::from_env()
            },
            move || button("Continue").on_press(move || pressed.set(pressed.get() + 1)),
        );
        screen.get_by_role(Role::Button, "Continue").click();
        assert_eq!(clicks.get(), 1);
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
