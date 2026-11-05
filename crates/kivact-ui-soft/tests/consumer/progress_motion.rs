//! Role: Verify Soft progress motion through the standard progress component.

use kivact::state::signal;
use kivact::style::Theme;
use kivact::style::ThemeMode;
use kivact::style::style;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::test::render_with_handle;
use kivact::ui::ProgressMotion;
use kivact::ui::progress;
use kivact::view::NodeBuilder;
use kivact::view::Role;
use std::time::Duration;

#[test]
fn soft_progress_starts_at_its_value_and_animates_updates() {
    let theme = kivact_ui_soft::theme(ThemeMode::Light);
    let motion = theme.ext::<kivact::ui::UiTokens>().progress.motion;
    assert!(!motion.animate_initial);
    assert_eq!(motion.determinate_ms, 300);
    assert_eq!(motion.period_ms, 1500);
    assert_eq!(motion.reduced_period_ms, Some(4000));
    assert_eq!(
        Theme::light().ext::<kivact::ui::UiTokens>().progress.motion,
        ProgressMotion::default()
    );
    let (screen, value) = render_with_handle(RenderOptions::with_theme(theme), || {
        let value = signal(Some(0.5));
        (
            value,
            progress(value).name("Upload").style(style().width(100)),
        )
    });
    let fill_at = |x| {
        screen
            .get_by_role(Role::ProgressBar, "Upload")
            .screenshot()
            .pixel(x, 4)
    };
    assert_eq!(fill_at(25), Some(0x1b1b1b));
    assert_eq!(fill_at(75), Some(0xd4d4d4));

    value.set(Some(0.9));
    assert_eq!(fill_at(75), Some(0xd4d4d4));
    screen.advance(Duration::from_millis(333));
    assert_eq!(fill_at(75), Some(0x1b1b1b));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn reduced_motion_progress_pulses_in_place() {
    let screen = render_with(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light)).reduced_motion(),
        || {
            progress(None::<f64>)
                .name("Preparing")
                .style(style().width(100))
        },
    );
    let fill = || {
        screen
            .get_by_role(Role::ProgressBar, "Preparing")
            .screenshot()
            .pixel(50, 4)
            .unwrap()
    };
    let initial = fill();
    screen.advance(Duration::from_millis(2000));
    let dimmed = fill();
    assert!(dimmed > initial && dimmed < 0xd4d4d4);
    screen.advance(Duration::from_millis(2000));
    assert_eq!(fill(), initial);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
