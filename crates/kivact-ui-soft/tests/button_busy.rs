//! Role: Verify that the Soft busy reveal uses the standard button's opt-in theme timing.

use kivact::style::{Position, Theme, ThemeMode, rgb, style};
use kivact::test::{RenderOptions, render_with};
use kivact::ui::{ButtonBusyMotion, UiRecipes, button};
use kivact::view::Role;
use std::time::Duration;

#[test]
fn soft_delays_the_spinner_but_disables_input_immediately() {
    let theme = kivact_ui_soft::theme(ThemeMode::Light);
    let timing = theme.get::<ButtonBusyMotion>();
    assert_eq!(timing.spin_ms, 525);
    assert_eq!(timing.reduced_spin_ms, 3000);
    assert_eq!(timing.reveal_delay_ms, 225);
    assert_eq!(timing.reveal_fade_ms, 125);
    assert_eq!(
        *Theme::light().get::<ButtonBusyMotion>(),
        ButtonBusyMotion::default()
    );

    let recipes = (*theme.get::<UiRecipes>()).clone().button_busy(|_| {
        |_, _| {
            style()
                .position(Position::Absolute)
                .top(0)
                .left(0)
                .size(16, 16)
                .background(rgb(0xff0000))
        }
    });
    let screen = render_with(
        RenderOptions {
            theme: theme.with(recipes),
            ..RenderOptions::from_env()
        },
        || button("Loading example").loading(true),
    );
    let red_is_visible = || screen.screenshot().pixels.contains(&0xff0000);
    assert!(
        screen
            .get_by_role(Role::Button, "Loading example")
            .is_disabled()
    );
    assert!(!red_is_visible());
    screen.advance(Duration::from_millis(224));
    assert!(!red_is_visible());
    screen.advance(Duration::from_millis(176));
    assert!(red_is_visible());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
