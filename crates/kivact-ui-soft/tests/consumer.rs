//! Role: Verify the Soft theme through the public facade and kit APIs.
use kivact::control::{NodeBuilder, column};
use kivact::i18n::CalendarDate;
use kivact::signal;
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, render_with, render_with_handle};
use kivact::ui::button;
use kivact::view::{Key, Role};
use kivact_ui_soft::{
    BadgeVariant, BannerStatus, IconSize, StatusDotVariant, badge, banner, heading, icon,
    status_dot,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn dismissing_a_banner_restores_the_previous_external_focus() {
    let screen = render_with(
        RenderOptions {
            theme: kivact_ui_soft::theme(ThemeMode::Light),
            ..RenderOptions::from_env()
        },
        || {
            column((
                button("Continue"),
                banner(BannerStatus::Info, "Workspace updated").dismissable(true),
                button("Other"),
            ))
        },
    );
    let outside = screen.get_by_role(Role::Button, "Continue");
    outside.focus();
    let dismiss = screen.get_by_role(Role::Button, "Dismiss Workspace updated");
    dismiss.focus();
    dismiss.press_key(Key::Enter);
    assert!(outside.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

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

#[test]
fn content_components_publish_their_names_and_roles() {
    let screen = render_with(
        RenderOptions {
            theme: kivact_ui_soft::theme(ThemeMode::Dark),
            ..RenderOptions::from_env()
        },
        || {
            column((
                heading(2, "Workspace"),
                icon("search").size(IconSize::Lg).label("Search icon"),
                badge("Review").variant(BadgeVariant::Warning),
                status_dot(StatusDotVariant::Success, "Online"),
            ))
        },
    );
    screen.get_by_role(Role::Heading, "Workspace");
    screen.get_by_role(Role::Image, "Search icon");
    screen.get_by_text("Review");
    screen.get_by_role(Role::Image, "Online");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn date_and_time_fields_commit_through_the_public_bindings() {
    let (screen, (date, time)) = render_with_handle(
        RenderOptions {
            theme: kivact_ui_soft::theme(ThemeMode::Light),
            ..RenderOptions::from_env()
        },
        || {
            let date = signal(None::<CalendarDate>);
            let time = signal(None::<kivact_ui_soft::TimeOfDay>);
            (
                (date, time),
                column((
                    kivact_ui_soft::date_input("Due date", date),
                    kivact_ui_soft::time_input("Reminder", time),
                )),
            )
        },
    );
    let field = screen.get_by_role(Role::TextBox, "Due date");
    field.type_text("2026-11-03");
    field.press_key(Key::Enter);
    assert_eq!(date.get_untracked(), CalendarDate::checked(2026, 11, 3));
    let field = screen.get_by_role(Role::TextBox, "Reminder");
    field.type_text("14:35");
    field.press_key(Key::Enter);
    assert_eq!(
        time.get_untracked(),
        kivact_ui_soft::TimeOfDay::new(14, 35, 0)
    );
}
