//! Role: Verify that the Soft busy reveal uses the standard button's opt-in theme timing.

use kivact::state::signal;
use kivact::style::Position;
use kivact::style::Theme;
use kivact::style::ThemeMode;
use kivact::style::rgb;
use kivact::style::style;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::ui::ButtonBusyMotion;
use kivact::ui::UiRecipes;
use kivact::ui::button;
use kivact::ui::tooltip;
use kivact::view::Key;
use kivact::view::NodeBuilder;
use kivact::view::Role;
use std::time::Duration;

#[test]
fn soft_disabled_button_can_show_its_tooltip_with_keyboard_focus() {
    let screen = render_with(
        RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            let attempts = signal(0);
            kivact::view::column((
                tooltip("Select an item first").attach(
                    button("Remove")
                        .disabled(true)
                        .on_press(move || attempts.update(|count| *count += 1)),
                ),
                kivact::view::text(move || format!("Attempts {}", attempts.get())),
            ))
        },
    );
    let remove = screen.get_by_role(Role::Button, "Remove");
    screen.press_tab(false);
    assert!(remove.is_focused());
    assert!(remove.is_disabled());
    assert_eq!(remove.description(), "Select an item first");
    let before_tooltip = screen.screenshot().hash();
    screen.advance(Duration::from_millis(1000));
    assert_ne!(screen.screenshot().hash(), before_tooltip);
    remove.press_key(Key::Enter);
    screen.get_by_text("Attempts 0");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn soft_delays_the_spinner_but_disables_input_immediately() {
    let theme = kivact_ui_soft::theme(ThemeMode::Light);
    let timing = theme.ext::<kivact::ui::UiTokens>().button.busy_motion;
    assert_eq!(timing.spin_ms, 525);
    assert_eq!(timing.reduced_spin_ms, 3000);
    assert_eq!(timing.reveal_delay_ms, 225);
    assert_eq!(timing.reveal_fade_ms, 125);
    assert_eq!(
        Theme::light()
            .ext::<kivact::ui::UiTokens>()
            .button
            .busy_motion,
        ButtonBusyMotion::default()
    );

    let recipes = (*theme.ext::<UiRecipes>()).clone().button_busy(|_| {
        |_, _| {
            style()
                .position(Position::Absolute)
                .top(0)
                .left(0)
                .size(16, 16)
                .background(rgb(0xff0000))
        }
    });
    let screen = render_with(RenderOptions::from_env().theme(theme.with(recipes)), || {
        button("Loading example").busy(true)
    });
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
