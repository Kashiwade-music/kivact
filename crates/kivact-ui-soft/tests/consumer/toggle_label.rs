//! Role: Verify optional toggle label measurement through the Soft public theme.

use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Theme;
use kivact::style::ThemeMode;
use kivact::style::style;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::test::render_with_handle;
use kivact::ui::ToggleLabelReservation;
use kivact::ui::toggle_button;
use kivact::view::CheckedState;
use kivact::view::Role;
use kivact::view::column;
use kivact::view::icon;

#[test]
fn soft_toggle_keeps_its_width_when_pressed_and_when_its_label_is_hidden() {
    let theme = kivact_ui_soft::theme(ThemeMode::Light);
    assert_eq!(
        theme
            .ext::<kivact::ui::UiTokens>()
            .toggle_button
            .label_reservation,
        ToggleLabelReservation::Reserve {
            weight: 600,
            size: 14.0,
            extra_small_size: 12.0
        }
    );
    let (screen, pressed) = render_with_handle(RenderOptions::from_env().theme(theme), || {
        let pressed = signal(false);
        let controls = column((
            toggle_button("Highlight modified rows", pressed),
            toggle_button("Icon toggle without a visible label", pressed)
                .icon(icon("check"))
                .hide_label(true),
        ))
        .style(style().align_items(Align::Start));
        (pressed, controls)
    });
    let width = |label| {
        screen
            .get_by_role(Role::Button, label)
            .rect()
            .unwrap()
            .width
    };
    let initial = width("Highlight modified rows");
    let icon_width = width("Icon toggle without a visible label");
    assert_eq!(icon_width, 32.0);
    pressed.set(true);
    screen.run_until_idle();
    assert_eq!(
        screen
            .get_by_role(Role::Button, "Highlight modified rows")
            .checked(),
        Some(CheckedState::Checked)
    );
    assert_eq!(width("Highlight modified rows"), initial);
    assert_eq!(width("Icon toggle without a visible label"), icon_width);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn width_measurement_is_opt_in_and_follows_label_changes() {
    assert_eq!(
        Theme::light()
            .ext::<kivact::ui::UiTokens>()
            .toggle_button
            .label_reservation,
        ToggleLabelReservation::Natural
    );
    let plain = render_with(RenderOptions::from_env(), || {
        column(toggle_button("Label", signal(false))).style(style().align_items(Align::Start))
    });
    let natural = plain
        .get_by_role(Role::Button, "Label")
        .rect()
        .unwrap()
        .width;
    drop(plain);

    // Use a deliberately larger measuring size to prove that the reservation
    // affects the layout even when the bundled test font has only one weight.
    let (reserved, label) = render_with_handle(
        RenderOptions::from_env().theme(Theme::light().with(kivact::ui::UiTokens {
            toggle_button: kivact::ui::ToggleButtonTokens {
                label_reservation: ToggleLabelReservation::Reserve {
                    weight: 600,
                    size: 34.0,
                    extra_small_size: 34.0,
                },
            },
            ..Default::default()
        })),
        || {
            let label = signal("Label".to_owned());
            (
                label,
                column(toggle_button(label, signal(false)))
                    .style(style().align_items(Align::Start)),
            )
        },
    );
    let first = reserved
        .get_by_role(Role::Button, "Label")
        .rect()
        .unwrap()
        .width;
    assert!(first > natural);
    label.set("A longer label".to_owned());
    reserved.run_until_idle();
    let second = reserved
        .get_by_role(Role::Button, "A longer label")
        .rect()
        .unwrap()
        .width;
    assert!(second > first);
    assert!(reserved.errors().is_empty(), "{:?}", reserved.errors());
}
