//! Role: Verify the Soft theme through the public facade and kit APIs.
use kivact::control::{NodeBuilder, column};
use kivact::i18n::CalendarDate;
use kivact::signal;
use kivact::style::ThemeMode;
use kivact::test::{RenderOptions, render_with, render_with_handle};
use kivact::ui::{ControlSize, button};
use kivact::view::{Key, Role};
use kivact_ui_soft::{
    BadgeVariant, BannerStatus, IconSize, StatusDotVariant, badge, banner, code_block, heading,
    icon, input_group_with, status_dot,
};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

#[test]
fn code_block_disclosure_releases_its_height_as_it_closes() {
    let screen = render_with(
        RenderOptions {
            theme: kivact_ui_soft::theme(ThemeMode::Light),
            reduced_motion: false,
            ..RenderOptions::from_env()
        },
        || {
            column((
                code_block("let a = 1;\nlet b = 2;\nlet c = 3;".to_owned())
                    .title("Snippet")
                    .language_label(false)
                    .collapsible(true)
                    .collapsible_threshold(2),
                button("After code"),
            ))
        },
    );
    let snippet = screen.get_by_role(Role::Button, "Snippet");
    let after = screen.get_by_role(Role::Button, "After code");
    let open_y = after.rect().expect("laid out sibling").y;
    assert_eq!(snippet.controls().len(), 1);
    snippet.press_key(Key::Enter);
    assert_eq!(snippet.is_expanded(), Some(false));
    screen.advance(Duration::from_millis(85));
    let intermediate_y = after.rect().expect("laid out sibling").y;
    screen.advance(Duration::from_millis(300));
    let closed_y = after.rect().expect("laid out sibling").y;
    assert!(closed_y < intermediate_y && intermediate_y < open_y);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn input_group_inherits_size_without_overriding_an_explicit_member() {
    let screen = render_with(
        RenderOptions {
            theme: kivact_ui_soft::theme(ThemeMode::Light),
            ..RenderOptions::from_env()
        },
        || {
            input_group_with("Actions", || {
                (
                    button("Inherited"),
                    button("Explicit").size(ControlSize::Small),
                )
            })
            .size(ControlSize::Large)
        },
    );
    let inherited = screen.get_by_role(Role::Button, "Inherited");
    let explicit = screen.get_by_role(Role::Button, "Explicit");
    assert_eq!(inherited.rect().expect("laid out").height, 36.0);
    assert_eq!(explicit.rect().expect("laid out").height, 28.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

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

#[test]
fn rich_text_source_toolbar_wraps_and_unwraps_native_unicode_selection() {
    let screen = render_with(
        RenderOptions {
            theme: kivact_ui_soft::theme(ThemeMode::Light),
            ..RenderOptions::from_env()
        },
        || {
            let source = signal("café 東京".to_owned());
            kivact_ui_soft::rich_text_editor("Markdown source", source)
        },
    );
    let editor = screen.get_by_role(Role::TextBox, "Markdown source");
    editor.press_key(Key::End);
    editor.press_key_with(
        Key::Home,
        kivact::view::Modifiers {
            shift: true,
            ..Default::default()
        },
    );
    for (command, wrapped) in [
        ("Bold", "**café 東京**"),
        ("Italic", "*café 東京*"),
        ("Strikethrough", "~~café 東京~~"),
        ("Inline code", "`café 東京`"),
    ] {
        let button = screen.get_by_role(Role::Button, command);
        button.click();
        assert_eq!(editor.value().as_deref(), Some(wrapped));
        assert!(editor.is_focused());
        button.click();
        assert_eq!(editor.value().as_deref(), Some("café 東京"));
    }
    screen.get_by_role(Role::Button, "Preview").click();
    assert!(
        screen
            .query_by_role(Role::TextBox, "Markdown source")
            .is_none()
    );
    screen.get_by_role(Role::Button, "Write").click();
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Markdown source")
            .value()
            .as_deref(),
        Some("café 東京")
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
