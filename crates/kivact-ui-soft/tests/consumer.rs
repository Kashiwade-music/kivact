//! Role: Verify the Soft theme through the public facade and kit APIs.
use kivact::i18n::CalendarDate;
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::test::render_with_handle;
use kivact::ui::ControlSize;
use kivact::ui::badge;
use kivact::ui::button;
use kivact::ui::input_group_with;
use kivact::view::Key;
use kivact::view::NodeBuilder;
use kivact::view::Role;
use kivact::view::column;
use kivact_ui_soft::{IconSize, StatusDotVariant, heading, icon, status_dot};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

#[test]
fn markdown_wraps_a_live_link_with_the_surrounding_paragraph() {
    let (screen, opened) = render_with_handle(
        RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            let opened = signal(String::new());
            let paragraph = kivact_ui_soft::markdown(
                "The project reference lives in [the guide](https://example.com/guide) today.",
            )
            .on_link(move |url| opened.set(url.to_owned()))
            .style(kivact::style::style().width(150))
            .test_id("markdown-paragraph");
            (opened, paragraph)
        },
    );
    let paragraph = screen.get_by_test_id("markdown-paragraph").rect().unwrap();
    let link = screen.get_by_role(Role::Link, "the guide");
    let bounds = link.rect().unwrap();
    assert!(bounds.y > paragraph.y + 8.0);
    assert!(bounds.x + bounds.width <= paragraph.x + paragraph.width + 1.0);
    link.click();
    assert_eq!(opened.get_untracked(), "https://example.com/guide");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn code_block_disclosure_releases_its_height_as_it_closes() {
    let screen = render_with(
        RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            column((
                kivact::ui::code_block("let a = 1;\nlet b = 2;\nlet c = 3;".to_owned())
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
        RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light)),
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
        RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            column((
                button("Continue"),
                kivact::ui::alert("Workspace updated")
                    .tone(kivact::interaction::Tone::Info)
                    .icon(kivact_ui_soft::banner_icon(kivact::interaction::Tone::Info))
                    .dismissible(true)
                    .dismiss_label("DismissOn Workspace updated"),
                button("Other"),
            ))
        },
    );
    let outside = screen.get_by_role(Role::Button, "Continue");
    outside.focus();
    let dismiss = screen.get_by_role(Role::Button, "DismissOn Workspace updated");
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
            RenderOptions::from_env().theme(kivact_ui_soft::theme(mode)),
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
        RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Dark)),
        || {
            column((
                heading(2, "Workspace"),
                icon("search").size(IconSize::Lg).label("Search icon"),
                badge("Review").tone(kivact::interaction::Tone::Warning),
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
        RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            let date = signal(None::<CalendarDate>);
            let time = signal(None::<kivact::i18n::TimeOfDay>);
            (
                (date, time),
                column((
                    kivact::ui::date_input("Due date", date)
                        .presentation(kivact::ui::DatePresentation::Desktop),
                    kivact::ui::time_input("Reminder", time)
                        .presentation(kivact::ui::DatePresentation::Desktop),
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
        Some(kivact::i18n::TimeOfDay {
            hour: 14,
            minute: 35,
            second: 0
        })
    );
}

#[test]
fn rich_text_source_toolbar_wraps_and_unwraps_native_unicode_selection() {
    let screen = render_with(
        RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light)),
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

#[test]
fn drop_policy_ends_the_actual_hover_visit_and_keeps_native_picking() {
    use kivact::geometry::Point;
    use kivact::interaction::DropzoneState;
    use kivact::style::style;
    use kivact::test::render_with_handle;
    use kivact::ui::UiRecipes;
    use kivact::view::FilePhase;
    use std::sync::{Arc, Mutex};
    let observed = Arc::new(Mutex::new(Vec::<DropzoneState>::new()));
    let recorded = observed.clone();
    let theme = kivact_ui_soft::theme(ThemeMode::Light);
    let recipes = (*theme.ext::<UiRecipes>())
        .clone()
        .dropzone(move |recipes| {
            recipes.root(move |previous| {
                move |theme, state| {
                    recorded
                        .lock()
                        .expect("invariant: observation lock is healthy")
                        .push((**state).clone());
                    previous(theme, state)
                }
            })
        });
    let (screen, (accepts, files)) =
        render_with_handle(RenderOptions::from_env().theme(theme.with(recipes)), || {
            let accepts = signal(true);
            let files = signal(Vec::<std::path::PathBuf>::new());
            (
                (accepts, files),
                kivact::ui::dropzone("Upload")
                    .pick_on_press()
                    .accept_drop(accepts)
                    .on_files(move |items| {
                        files.set(items.into_iter().map(|item| item.path).collect())
                    })
                    .style(style().width(240).height(96)),
            )
        });
    let target = screen.get_by_role(Role::Button, "Upload");
    let bounds = target.rect().expect("invariant: drop target has layout");
    let point = Point::new(
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    );
    screen.file_drag(FilePhase::Enter, vec!["hover.txt".into()], point);
    assert!(matches!(
        observed
            .lock()
            .expect("invariant: observation lock is healthy")
            .last(),
        Some(DropzoneState::DragOver { .. })
    ));
    accepts.set(false);
    screen.run_until_idle();
    assert!(matches!(
        observed
            .lock()
            .expect("invariant: observation lock is healthy")
            .last(),
        Some(DropzoneState::Idle)
    ));
    screen.file_drag(FilePhase::Drop, vec!["ignored.txt".into()], point);
    assert!(files.get_untracked().is_empty());
    screen.respond_files_dialog(Some(vec!["picked.txt".into()]));
    target.press_key(Key::Enter);
    assert_eq!(
        files.get_untracked(),
        vec![std::path::PathBuf::from("picked.txt")]
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn canonical_avatar_overflow_tracks_real_omissions_and_keeps_its_action() {
    let (screen, (total, opened)) = render_with_handle(
        RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light)),
        || {
            let total = signal(6usize);
            let opened = signal(0usize);
            (
                (total, opened),
                kivact::ui::avatar_group((
                    kivact::ui::avatar("Avery Middle Morgan"),
                    kivact::ui::avatar("Taylor Chen"),
                ))
                .total(total)
                .overlap(9.0)
                .overflow_content(move |count| {
                    button(move || format!("Show {} more", count.get()))
                        .on_press(move || opened.set(count.get_untracked()))
                }),
            )
        },
    );
    assert!(screen.get_by_role(Role::Text, "AM").exists());
    screen.get_by_role(Role::Button, "Show 4 more").click();
    assert_eq!(opened.get_untracked(), 4);
    total.set(8);
    screen.run_until_idle();
    screen.get_by_role(Role::Button, "Show 6 more").click();
    assert_eq!(opened.get_untracked(), 6);
    total.set(2);
    screen.run_until_idle();
    assert!(screen.query_by_role(Role::Button, "Show 6 more").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
