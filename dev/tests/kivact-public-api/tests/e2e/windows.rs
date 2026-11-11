//! Role: Contracts of windows opened from views.

use std::time::Duration;

use crate::support::{mount, mount_with};
use kivact::ErrorKind;
use kivact::i18n::{Locale, provide_locale, use_locale};
use kivact::platform;
use kivact::prelude::*;
use kivact::style::provide_theme;
use kivact::test::RenderOptions;
use kivact::view::A11yAction;

#[test]
fn opening_a_window_mounts_its_content_under_its_title() {
    let (screen, open) = mount(|| {
        let open = signal(false);
        (
            open,
            column((
                text("main"),
                window("Settings", open).content(|| text("settings content")),
            )),
        )
    });
    open.set(true);
    screen.run_until_idle();
    let settings = screen.window("Settings");
    assert_eq!(settings.title(), "Settings");
    settings.get_by_text("settings content");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn closing_a_window_through_its_open_binding_closes_the_handle() {
    let (screen, open) = mount(|| {
        let open = signal(true);
        (
            open,
            column((
                text("main"),
                window("Settings", open).content(|| text("settings content")),
            )),
        )
    });
    let settings = screen.window("Settings");
    settings.get_by_text("settings content");
    open.set(false);
    screen.run_until_idle();
    assert!(!settings.is_open(), "open false closes the window");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_window_closes_when_its_declaring_show_branch_disappears() {
    let (screen, declared) = mount(|| {
        let declared = signal(true);
        let open = signal(true);
        (
            declared,
            column((
                text("main"),
                show(declared, move || {
                    window("Settings", open).content(|| text("settings content"))
                }),
            )),
        )
    });
    let settings = screen.window("Settings");
    declared.set(false);
    screen.run_until_idle();
    assert!(
        !settings.is_open(),
        "the window closes with its declaring branch"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_window_inherits_the_context_theme_and_locale_of_its_declaration_site() {
    let (screen, ()) = mount(|| {
        let open = signal(true);
        (
            (),
            column((
                text("main"),
                provide("provided".to_owned(), move || {
                    provide_theme(Theme::dark(), move || {
                        provide_locale(Locale::new("fr-FR"), move || {
                            window("Settings", open).content(|| {
                                let context = use_context::<String>();
                                let locale = use_locale();
                                let theme = use_theme();
                                (
                                    text(move || context.get()),
                                    text(move || locale.get().tag().to_owned()),
                                    text(move || format!("{:?}", theme.get().mode())),
                                )
                            })
                        })
                    })
                }),
            )),
        )
    });
    let settings = screen.window("Settings");
    settings.get_by_text("provided");
    settings.get_by_text("fr-FR");
    settings.get_by_text("Dark");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_close_request_lets_the_dismiss_handler_decide() {
    let (screen, (plain_open, allowed_open, prevented_open, failed_open)) = mount(|| {
        let plain_open = signal(true);
        let allowed_open = signal(true);
        let prevented_open = signal(true);
        let failed_open = signal(true);
        (
            (plain_open, allowed_open, prevented_open, failed_open),
            column((
                window("Plain", plain_open).content(|| text("plain")),
                window("Allowed", allowed_open)
                    .content(|| text("allowed"))
                    .on_dismiss(|_| ()),
                window("Prevented", prevented_open)
                    .content(|| text("prevented"))
                    .on_dismiss(|_| Handled::PreventDefault),
                window("Failed", failed_open)
                    .content(|| text("failed"))
                    .on_dismiss(|_| -> Result<(), UiError> { Err(UiError::msg("dismiss failed")) }),
            )),
        )
    });
    let plain = screen.window("Plain");
    assert!(plain.request_close(), "no handler: the window closes");
    assert!(!plain.is_open());
    assert!(!plain_open.get_untracked());

    let allowed = screen.window("Allowed");
    assert!(allowed.request_close(), "() lets the window close");
    assert!(!allowed.is_open());
    assert!(!allowed_open.get_untracked());

    let prevented = screen.window("Prevented");
    assert!(!prevented.request_close(), "PreventDefault keeps it open");
    assert!(prevented.is_open());
    assert!(prevented_open.get_untracked());

    let failed = screen.window("Failed");
    assert!(!failed.request_close(), "a failed handler keeps it open");
    assert!(failed.is_open());
    assert!(failed_open.get_untracked());
    let errors = screen.errors();
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].message().contains("dismiss failed"), "{errors:?}");
}

#[test]
fn closing_the_main_window_exits_the_app() {
    let (screen, ()) = mount(|| ((), text("main")));
    assert!(
        screen.request_close(),
        "a close request on the main window ends the App"
    );
    assert!(screen.has_exited());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn focus_and_tab_order_are_independent_in_each_window() {
    let (screen, ()) = mount(|| {
        let open = signal(true);
        let main_a = node()
            .role(Role::Button)
            .focusable(true)
            .test_id("main-a")
            .child(text("main a"));
        let main_b = node()
            .role(Role::Button)
            .focusable(true)
            .test_id("main-b")
            .child(text("main b"));
        (
            (),
            column((
                column((main_a, main_b)),
                window("Settings", open).content(|| {
                    let settings_a = node()
                        .role(Role::Button)
                        .focusable(true)
                        .test_id("settings-a")
                        .child(text("settings a"));
                    let settings_b = node()
                        .role(Role::Button)
                        .focusable(true)
                        .test_id("settings-b")
                        .child(text("settings b"));
                    column((settings_a, settings_b))
                }),
            )),
        )
    });
    let settings = screen.window("Settings");
    screen.get_by_test_id("main-a").focus();
    settings.get_by_test_id("settings-a").focus();
    assert!(screen.get_by_test_id("main-a").is_focused());
    assert!(settings.get_by_test_id("settings-a").is_focused());

    settings.press_tab(false);
    assert!(
        settings.get_by_test_id("settings-b").is_focused(),
        "Tab moves the focus of its window"
    );
    assert!(
        screen.get_by_test_id("main-a").is_focused(),
        "the other window keeps its focus"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_modal_window_blocks_pointer_and_accessibility_input_to_its_parent() {
    let (screen, (parent_presses, modal_presses)) = mount(|| {
        let parent_presses = signal(0);
        let modal_presses = signal(0);
        let open = signal(true);
        (
            (parent_presses, modal_presses),
            column((
                button("Parent").test_id("parent").on_press(move || {
                    parent_presses.set(parent_presses.get_untracked() + 1);
                }),
                window("Sheet", open)
                    .mode(WindowMode::Modal)
                    .content(move || {
                        button("Modal").test_id("modal").on_press(move || {
                            modal_presses.set(modal_presses.get_untracked() + 1);
                        })
                    }),
            )),
        )
    });
    let sheet = screen.window("Sheet");
    screen.get_by_test_id("parent").click();
    assert_eq!(
        parent_presses.get_untracked(),
        0,
        "the modal owns the pointer input of its parent"
    );
    screen.get_by_test_id("parent").a11y(A11yAction::Activate);
    assert_eq!(
        parent_presses.get_untracked(),
        0,
        "and the accessibility actions"
    );
    sheet.get_by_test_id("modal").click();
    assert_eq!(
        modal_presses.get_untracked(),
        1,
        "the modal itself still receives input"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn window_size_reads_the_size_of_the_calling_window() {
    let (screen, ()) = mount(|| {
        let open = signal(true);
        (
            (),
            column((
                text(move || format!("main {:.0}", platform::window_size().get().width)),
                window("Settings", open).size(320, 240).content(|| {
                    text(move || format!("settings {:.0}", platform::window_size().get().width))
                }),
            )),
        )
    });
    screen.get_by_text("main 800");
    let settings = screen.window("Settings");
    settings.get_by_text("settings 320");
    settings.resize(500.0, 300.0);
    settings.get_by_text("settings 500");
    screen.get_by_text("main 800");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_single_window_platform_reports_unsupported_and_opens_nothing() {
    let (screen, ()) = mount_with(RenderOptions::default().single_window(), || {
        let open = signal(true);
        (
            (),
            column((
                text("main"),
                window("Settings", open).content(|| text("settings")),
            )),
        )
    });
    screen.run_until_idle();
    assert_eq!(
        screen.windows().len(),
        1,
        "no window beyond the first opens"
    );
    let errors = screen.errors();
    assert!(!errors.is_empty(), "the refused window is reported");
    assert!(
        errors.iter().all(|e| *e.kind() == ErrorKind::Unsupported),
        "{errors:?}"
    );
}

#[test]
fn a_signal_read_by_text_in_two_windows_updates_both_in_one_turn() {
    let (screen, ()) = mount(|| {
        let value = signal(0);
        let open = signal(true);
        (
            (),
            column((
                text(move || format!("value {}", value.get())),
                button("Bump").on_press(move || value.set(value.get_untracked() + 1)),
                window("Settings", open)
                    .content(move || text(move || format!("value {}", value.get()))),
            )),
        )
    });
    let settings = screen.window("Settings");
    screen.get_by_text("value 0");
    settings.get_by_text("value 0");
    let before = screen.stats().batches;
    screen.get_by_role(Role::Button, "Bump").click();
    screen.get_by_text("value 1");
    settings.get_by_text("value 1");
    assert_eq!(
        screen.stats().batches,
        before + 1,
        "one turn updates both windows"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn destroying_the_main_surface_does_not_stop_or_rewind_another_window_animation() {
    let (screen, (main_target, child_target, main_shown, child_shown)) = mount(|| {
        let main_target = signal(0.0f64);
        let main_shown = animated(main_target, Transition::tween(1000, Easing::Linear));
        let child_target = signal(0.0f64);
        let child_shown = animated(child_target, Transition::tween(1000, Easing::Linear));
        let open = signal(true);
        (
            (main_target, child_target, main_shown, child_shown),
            column((
                text(move || format!("main {:.1}", main_shown.get())),
                window("Settings", open)
                    .content(move || text(move || format!("settings {:.1}", child_shown.get()))),
            )),
        )
    });
    let settings = screen.window("Settings");
    main_target.set(100.0);
    child_target.set(100.0);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(100));
    let main_before = main_shown.get_untracked();
    let child_before = child_shown.get_untracked();
    assert!(child_before > 0.0, "the settings animation runs");

    screen.script().destroy_surface();
    screen.advance(Duration::from_millis(100));
    let child_after = child_shown.get_untracked();
    assert!(
        child_after > child_before,
        "the other window's animation keeps running"
    );

    screen.script().create_surface();
    screen.advance(Duration::from_millis(100));
    let main_after = main_shown.get_untracked();
    assert!(
        main_after >= main_before,
        "the main animation does not rewind: {main_before} -> {main_after}"
    );
    assert!(settings.is_open(), "the other window stays open");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
