//! Role: Environment and capability tests.
//!
//! A window's environment snapshot and capabilities, and what a host asks
//! of the window (software keyboard, IME area, cursor, title, minimum size,
//! clipboard, file and folder dialogs, opening and revealing paths).

use arvane::Window as _;
use arvane::test_display::{VirtualWindow, VirtualWindowOptions};
use kivact_types::protocol::{Environment, MouseSettings};
use kivact_types::{Cursor, Insets, Size};

use crate::support::{PERIOD, open, options, rect};

#[test]
fn the_environment_is_one_snapshot_that_follows_its_changes() {
    let mut window = VirtualWindow::open(VirtualWindowOptions {
        dark: true,
        reduced_motion: true,
        frame_period_ns: PERIOD / 2,
        locale: Some("ja-JP".into()),
        ..options(100.0, 80.0)
    })
    .expect("a test window opens");
    assert_eq!(
        window.environment(),
        Environment {
            scale: 1.0,
            frame_period_ns: PERIOD / 2,
            reduced_motion: true,
            dark: true,
            safe_area: Insets::default(),
            mouse: MouseSettings::default(),
            locale: Some("ja-JP".into()),
        }
    );
    // What a host is told about with `EnvironmentChanged` (the scale with
    // `Resized`) is in the next snapshot.
    let insets = Insets {
        top: 24.0,
        right: 0.0,
        bottom: 16.0,
        left: 0.0,
    };
    window.set_dark(false);
    window.set_reduced_motion(false);
    window.set_frame_period(PERIOD);
    window.set_safe_area(insets);
    window.set_locale(None);
    window.set_scale(1.5);
    let environment = window.environment();
    assert_eq!(
        environment,
        Environment {
            scale: 1.5,
            frame_period_ns: PERIOD,
            reduced_motion: false,
            dark: false,
            safe_area: insets,
            mouse: MouseSettings::default(),
            locale: None,
        }
    );
    assert_eq!(window.scale(), 1.5);
    window.set_locale(Some("fr-CA"));
    assert_eq!(window.environment().locale.as_deref(), Some("fr-CA"));
    // A resize and a scale change move the frame's plan.
    window.resize(Size::new(60.0, 40.0));
    assert_eq!(window.size(), Size::new(60.0, 40.0));
    let plan = window.frame_plan();
    assert_eq!((plan.size, plan.scale), (window.size(), 1.5));
}

#[test]
fn the_software_keyboard_and_the_ime_follow_the_host() {
    let window = open(100.0, 80.0);
    assert_eq!(window.soft_keyboard(), (false, 0));
    window.set_soft_keyboard(true);
    window.set_soft_keyboard(true);
    assert_eq!(window.soft_keyboard(), (true, 1), "shown once");
    window.set_soft_keyboard(false);
    assert_eq!(window.soft_keyboard(), (false, 2));
    let area = rect(10.0, 20.0, 2.0, 16.0);
    window.set_ime_allowed(true);
    window.set_ime_cursor_area(area);
    assert_eq!(window.ime_area(), Some(area));
    window.set_ime_allowed(false);
    assert_eq!(window.ime_area(), None);
    window.set_cursor(Cursor::Text);
    assert_eq!(window.cursor(), Cursor::Text);
    window.set_title("Arvane");
    assert_eq!(window.title(), "Arvane");
    assert_eq!(window.min_size(), None);
    let min = Size::new(60.0, 40.0);
    window.set_min_size(Some(min));
    assert_eq!(window.min_size(), Some(min));
    window.set_min_size(None);
    assert_eq!(window.min_size(), None);
    assert!(!window.redraw_requested());
    window.request_redraw();
    assert!(window.take_redraw());
    assert!(!window.take_redraw());
}
