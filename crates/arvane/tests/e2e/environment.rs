//! Role: Environment and capability tests.
//!
//! A window's environment snapshot and capabilities, and what a host asks
//! of the window (software keyboard, IME area, cursor, title, minimum size,
//! clipboard, file and folder dialogs, opening and revealing paths).

use std::path::{Path, PathBuf};

use arvane::test_display::{VirtualWindow, VirtualWindowOptions};
use arvane::{ShellFileDialog, Window as _};
use kivact_types::{Cursor, Environment, Insets, MouseSettings, Size};

use crate::support::{PERIOD, open, options, rect};

#[test]
fn multiple_file_dialog_has_the_same_cancellation_and_error_contract() {
    let mut window = open(100.0, 80.0);
    let dialog = ShellFileDialog::default();
    assert_eq!(
        window.pick_files(&dialog).expect("unscripted cancels"),
        None
    );
    let paths = vec![PathBuf::from("a.pdf"), PathBuf::from("日本語.png")];
    window.push_files_dialog(Some(paths.clone()));
    window.fail_next_effect("picker unavailable");
    assert_eq!(
        window.pick_files(&dialog).expect_err("failed").to_string(),
        "picker unavailable"
    );
    assert_eq!(
        window.pick_files(&dialog).expect("selection retained"),
        Some(paths)
    );
    window.push_files_dialog(None);
    assert_eq!(
        window.pick_files(&dialog).expect("explicit cancellation"),
        None
    );
    window.push_file_dialog(Some(PathBuf::from("single.pdf")));
    assert_eq!(
        window.pick_file(&dialog).expect("single picker unchanged"),
        Some("single.pdf".into())
    );
}

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
    window.resize(Size {
        width: 60.0,
        height: 40.0,
    });
    assert_eq!(
        window.size(),
        Size {
            width: 60.0,
            height: 40.0
        }
    );
    let plan = window.frame_plan();
    assert_eq!((plan.size, plan.scale), (window.size(), 1.5));
}

#[test]
fn a_test_window_can_do_what_any_platform_does() {
    let window = open(100.0, 80.0);
    let capabilities = window.capabilities();
    assert!(capabilities.touch);
    assert!(capabilities.pen);
    assert!(capabilities.soft_keyboard);
    assert!(capabilities.safe_area);
    assert!(capabilities.memory_warnings);
    assert!(capabilities.ime_reconversion);
    assert_eq!(capabilities.native_accessibility, window.has_window());
    assert!(!capabilities.gpu, "a CPU window");
    assert!(window.accessibility_active());
    assert!(!window.defers_accessibility());
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
    let min = Size {
        width: 60.0,
        height: 40.0,
    };
    window.set_min_size(Some(min));
    assert_eq!(window.min_size(), Some(min));
    window.set_min_size(None);
    assert_eq!(window.min_size(), None);
    assert!(!window.redraw_requested());
    window.request_redraw();
    assert!(window.take_redraw());
    assert!(!window.take_redraw());
}

#[test]
fn platform_requests_answer_from_memory_and_a_script() {
    let mut window = open(100.0, 80.0);
    window.copy_text("copied").expect("copy");
    assert_eq!(window.read_text().expect("paste"), "copied");
    assert_eq!(window.clipboard(), "copied");
    let dialog = ShellFileDialog::default();
    assert_eq!(window.pick_file(&dialog).expect("cancelled"), None);
    window.push_file_dialog(Some(PathBuf::from("a.txt")));
    assert_eq!(
        window.save_file(&dialog).expect("picked"),
        Some(PathBuf::from("a.txt"))
    );
    window.fail_next_effect("denied");
    window.set_title("a window operation keeps the service failure pending");
    assert_eq!(
        window
            .copy_text("rejected")
            .expect_err("denied")
            .to_string(),
        "denied"
    );
    assert_eq!(
        window.clipboard(),
        "copied",
        "a failed copy changes nothing"
    );
    window
        .copy_text("accepted")
        .expect("the failure was consumed");
    window.fail_next_effect("no clipboard");
    assert_eq!(
        window.read_text().expect_err("denied").to_string(),
        "no clipboard"
    );
    assert_eq!(
        window.read_text().expect("the failure was consumed"),
        "accepted"
    );

    window.push_file_dialog(Some(PathBuf::from("open.txt")));
    window.fail_next_effect("cannot open");
    assert_eq!(
        window.pick_file(&dialog).expect_err("denied").to_string(),
        "cannot open"
    );
    assert_eq!(
        window.pick_file(&dialog).expect("the answer was kept"),
        Some(PathBuf::from("open.txt"))
    );
    window.push_file_dialog(Some(PathBuf::from("save.txt")));
    window.fail_next_effect("cannot save");
    assert_eq!(
        window.save_file(&dialog).expect_err("denied").to_string(),
        "cannot save"
    );
    assert_eq!(
        window.save_file(&dialog).expect("the answer was kept"),
        Some(PathBuf::from("save.txt"))
    );
    assert_eq!(
        window.save_file(&dialog).expect("unscripted: cancelled"),
        None
    );
}

#[test]
fn open_and_reveal_are_recorded_in_order_and_share_the_failure_contract() {
    let mut window = open(100.0, 80.0);
    assert!(window.opened_paths().is_empty());
    assert!(window.revealed_paths().is_empty());
    window.open_path(Path::new("a.txt")).expect("recorded");
    window
        .reveal_path(Path::new("dir/b.txt"))
        .expect("recorded");
    window.open_path(Path::new("dir")).expect("recorded");
    window.fail_next_effect("no handler");
    let error = window.reveal_path(Path::new("c.txt")).expect_err("failed");
    assert_eq!(error.to_string(), "no handler");
    assert!(!error.is_unsupported());
    window.fail_next_effect_unsupported("not on this platform");
    let error = window
        .open_path(Path::new("d.txt"))
        .expect_err("unsupported");
    assert!(error.is_unsupported());
    assert_eq!(
        window.opened_paths(),
        [PathBuf::from("a.txt"), PathBuf::from("dir")]
    );
    assert_eq!(
        window.revealed_paths(),
        [PathBuf::from("dir/b.txt")],
        "a failed request is not recorded"
    );
}

#[test]
fn the_folder_dialog_answers_from_the_dialog_script() {
    let mut window = open(100.0, 80.0);
    let dialog = ShellFileDialog::default();
    assert_eq!(
        window.pick_folder(&dialog).expect("unscripted cancels"),
        None
    );
    window.push_file_dialog(Some(PathBuf::from("project")));
    window.fail_next_effect("picker unavailable");
    assert_eq!(
        window.pick_folder(&dialog).expect_err("failed").to_string(),
        "picker unavailable"
    );
    assert_eq!(
        window.pick_folder(&dialog).expect("the answer was kept"),
        Some(PathBuf::from("project"))
    );
    window.push_file_dialog(None);
    assert_eq!(window.pick_folder(&dialog).expect("cancelled"), None);
}
