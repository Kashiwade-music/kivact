//! Role: Source interaction regressions exercised as an ordinary external consumer.
use super::*;

#[test]
fn chips_add_and_remove_keep_the_callers_collection_and_keyboard_action() {
    let (screen, tags) = kivact::test::render_with_handle(crate::options(), || {
        let tags = signal(vec!["Initial".to_string()]);
        (
            tags,
            ui::chip_list(tags)
                .on_add(move || tags.update(|items| items.push("New".into())))
                .on_remove(move |index| {
                    tags.update(|items| {
                        items.remove(index);
                    })
                }),
        )
    });
    screen
        .get_by_role(Role::Button, "Add")
        .press_key(Key::Enter);
    assert_eq!(tags.get_untracked(), vec!["Initial", "New"]);
    screen.get_by_role(Role::Button, "Remove New").click();
    assert_eq!(tags.get_untracked(), vec!["Initial"]);
    assert!(screen.query_by_role(Role::Button, "Remove New").is_none());
    screen
        .get_by_role(Role::Button, "Add")
        .a11y(A11yAction::Activate);
    assert_eq!(tags.get_untracked(), vec!["Initial", "New"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn upload_progress_and_actual_metadata_change_the_visible_completion_state() {
    let (screen, (progress, bytes)) = kivact::test::render_with_handle(crate::options(), || {
        let progress = signal(0.0f64);
        let bytes = signal(Some(2048u64));
        (
            (progress, bytes),
            ui::upload_row("Report.txt", bytes, progress),
        )
    });
    screen.get_by_text("2KB");
    screen.get_by_role(Role::Status, "Uploading");
    let before = screen.screenshot().hash();
    progress.set(50.0);
    screen.run_until_idle();
    assert_ne!(before, screen.screenshot().hash());
    assert!(
        screen
            .query_by_role(Role::Status, "Upload complete")
            .is_none()
    );
    progress.set(100.0);
    bytes.set(Some(2_097_152));
    screen.run_until_idle();
    screen.get_by_text("2.0MB");
    screen.get_by_role(Role::Status, "Upload complete");
    assert!(screen.query_by_role(Role::Status, "Uploading").is_none());
    bytes.set(None);
    screen.run_until_idle();
    screen.get_by_text("Unknown size");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn thread_read_state_and_timed_typing_use_reactive_public_values() {
    let (screen, (unread, typing)) = kivact::test::render_with_handle(crate::options(), || {
        let unread = signal(2usize);
        let typing = signal(false);
        (
            (unread, typing),
            column((
                ui::thread("MN", "Conversation")
                    .preview("Preview")
                    .unread(unread)
                    .selected(false)
                    .on_press(move || unread.set(0)),
                kivact::ui::button("Send").on_press(move || {
                    kivact::task::spawn(async move {
                        kivact::task::sleep(Duration::from_millis(400)).await;
                        typing.set(true);
                        kivact::task::sleep(Duration::from_millis(1300)).await;
                        typing.set(false);
                    });
                }),
                show(typing, || ui::typing_message("MN")),
            )),
        )
    });
    let thread = screen.get_by_role(Role::Button, "Conversation");
    thread.get_by_text("2");
    thread.click();
    assert_eq!(unread.get_untracked(), 0);
    assert!(screen.within(thread).query_by_text("2").is_none());
    screen.get_by_role(Role::Button, "Send").click();
    assert!(screen.query_by_role(Role::Status, "Typing").is_none());
    screen.advance(Duration::from_millis(410));
    assert!(typing.get_untracked());
    screen.get_by_role(Role::Status, "Typing");
    screen.advance(Duration::from_millis(1310));
    assert!(!typing.get_untracked());
    assert!(screen.query_by_role(Role::Status, "Typing").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn notification_history_mark_read_updates_rows_and_trigger_without_closing() {
    let (screen, items) = kivact::test::render_with_handle(crate::options(), || {
        let items = signal(vec![
            ui::NotificationItem {
                unread: true,
                ..ui::NotificationItem::new(1, "New season", "New goals", "5 minutes ago")
            },
            ui::NotificationItem {
                unread: false,
                ..ui::NotificationItem::new(2, "Reward", "Credits received", "Today")
            },
        ]);
        (
            items,
            ui::notification_menu(
                kivact::ui::button("Inbox")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(kivact::control::icon("bell")),
                items,
                move || items.update(|items| items.iter_mut().for_each(|item| item.unread = false)),
            ),
        )
    });
    screen.get_by_role(Role::Status, "Unread notifications");
    let trigger = screen.get_by_role(Role::Button, "Inbox");
    trigger.click();
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(screen.get_by_role(Role::Group, "New season").is_selected());
    screen.get_by_role(Role::Button, "Mark all as read").click();
    assert!(items.get_untracked().iter().all(|item| !item.unread));
    assert!(!screen.get_by_role(Role::Group, "New season").is_selected());
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(
        screen
            .query_by_role(Role::Status, "Unread notifications")
            .is_none()
    );
    screen.press_key(Key::Escape);
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(screen.query_by_role(Role::Group, "Notifications").is_none());
    assert!(trigger.is_focused());
    trigger.a11y(A11yAction::Expand);
    screen
        .get_by_role(Role::Group, "Notifications")
        .a11y(A11yAction::Dismiss);
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn command_group_hint_search_wrap_and_hover_select_the_executed_item() {
    let (screen, (open, chosen)) = kivact::test::render_with_handle(crate::options(), || {
        let open = signal(true);
        let chosen = signal(0usize);
        (
            (open, chosen),
            kivact::ui::command_palette(
                open,
                vec![
                    Command::new(1, "First").group("Actions").hint("Panel"),
                    Command::new(2, "Second").group("Actions").hint("Panel"),
                ],
                move |key: usize| chosen.set(key),
            ),
        )
    });
    let search = screen.get_by_role(Role::TextBox, "Search commands");
    search.type_text(" Panel ");
    screen.get_by_role(Role::Option, "First");
    screen.get_by_role(Role::Option, "Second");
    search.press_key(Key::ArrowUp);
    assert!(screen.get_by_role(Role::Option, "Second").is_selected());
    search.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Option, "First").is_selected());
    screen.get_by_role(Role::Option, "Second").hover();
    assert!(screen.get_by_role(Role::Option, "Second").is_selected());
    search.press_key(Key::Enter);
    assert_eq!(chosen.get_untracked(), 2);
    assert!(!open.get_untracked());
    open.set(true);
    screen.run_until_idle();
    screen
        .get_by_role(Role::TextBox, "Search commands")
        .type_text("Actions");
    screen.get_by_role(Role::Option, "First");
    screen.get_by_role(Role::Option, "Second");
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(
        screen
            .query_by_role(Role::Dialog, "Command palette")
            .is_none()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn table_source_changes_preserve_page_and_search_matches_chosen_fields() {
    #[derive(Clone, PartialEq)]
    struct Member {
        id: u64,
        name: &'static str,
        club: &'static str,
    }
    let (screen, rows) = kivact::test::render_with_handle(crate::options(), || {
        let rows = signal(vec![
            Member {
                id: 1,
                name: "One",
                club: "alpha",
            },
            Member {
                id: 2,
                name: "Two",
                club: "beta",
            },
            Member {
                id: 3,
                name: "Three",
                club: "gamma",
            },
            Member {
                id: 4,
                name: "Four",
                club: "delta",
            },
        ]);
        let found = |text: &str, query: &str| text.to_lowercase().contains(&query.to_lowercase());
        (
            rows,
            kivact::ui::data_table(
                rows,
                |m: &Member| m.id,
                vec![
                    kivact::ui::DataColumn::new("Name", |m: &Member| m.name.to_owned())
                        .width(120.0)
                        .filter(move |m, q| found(m.name, q)),
                    kivact::ui::DataColumn::new("Club", |m: &Member| m.club.to_owned())
                        .width(120.0)
                        .filter(move |m, q| found(m.club, q)),
                    kivact::ui::DataColumn::new("Role", |_: &Member| "Role".to_owned())
                        .width(100.0),
                ],
            )
            .selection(signal(Vec::new()))
            .filter(signal(String::new()))
            .page(signal(0_usize), 2),
        )
    });
    screen.get_by_role(Role::Button, "Next").click();
    screen.get_by_text("Three");
    rows.update(|rows| rows.retain(|row| row.id != 4));
    screen.run_until_idle();
    screen.get_by_text("Three");
    assert!(screen.query_by_text("One").is_none());
    screen
        .get_by_role(Role::TextBox, "Filter…")
        .type_text(" beta ");
    screen.get_by_text("Two");
    assert!(screen.query_by_text("One").is_none());
    let search = screen.get_by_role(Role::TextBox, "Filter…");
    search.commit("Role");
    screen.get_by_text("No results found.");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn drawer_title_tracks_the_current_row_and_tag_container_focuses_its_editor() {
    let (screen, (open, title)) = kivact::test::render_with_handle(crate::options(), || {
        let open = signal(false);
        let title = signal("First row".to_string());
        let tags = signal(vec!["Initial".to_string()]);
        (
            (open, title),
            column((
                kivact::ui::tag_input(
                    "Tags",
                    tags,
                    move |tag| tags.update(|items| items.push(tag)),
                    move |index| {
                        tags.update(|items| {
                            items.remove(index);
                        })
                    },
                )
                .test_id("tags"),
                kivact::ui::button("Details").on_press(move || open.set(true)),
                kivact::ui::dialog(title, open)
                    .kind(kivact::ui::DialogKind::Drawer(kivact::ui::Side::Right))
                    .content(|| text("Details")),
            )),
        )
    });
    let rect = screen.get_by_test_id("tags").rect().expect("tags laid out");
    screen.click_at(rect.x + 3.0, rect.y + 18.0);
    assert!(screen.get_by_role(Role::TextBox, "Tags").is_focused());
    screen.get_by_role(Role::Button, "Details").click();
    screen.get_by_role(Role::Dialog, "First row");
    title.set("Second row".into());
    screen.run_until_idle();
    screen.get_by_role(Role::Dialog, "Second row");
    screen.get_by_text("Second row");
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn popover_acknowledgement_closes_the_same_open_state_and_returns_focus() {
    let (screen, open) = kivact::test::render_with_handle(crate::options(), || {
        let open = signal(false);
        (
            open,
            column((kivact::ui::dialog("Help topic", open)
                .kind(kivact::ui::DialogKind::Popover)
                .content(move || {
                    column((
                        text("Help content"),
                        kivact::ui::button("Understood").on_press(move || open.set(false)),
                    ))
                })
                .attach(kivact::ui::button("Help")),)),
        )
    });
    let trigger = screen.get_by_role(Role::Button, "Help");
    trigger.click();
    assert!(open.get_untracked());
    screen.get_by_role(Role::Dialog, "Help topic");
    screen.get_by_role(Role::Button, "Understood").click();
    assert!(!open.get_untracked());
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(screen.query_by_text("Help content").is_none());
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dialog_drawer_and_popover_are_named_dialogs_that_assistive_technology_can_dismiss() {
    let (screen, opens) = kivact::test::render_with_handle(crate::options(), || {
        let (modal, sheet, popover) = (signal(false), signal(false), signal(false));
        (
            (modal, sheet, popover),
            column((
                kivact::ui::button("Open modal").on_press(move || modal.set(true)),
                kivact::ui::button("Open sheet").on_press(move || sheet.set(true)),
                kivact::ui::dialog("Rename", modal)
                    .description("The new name shows everywhere.")
                    .content(|| kivact::ui::text_input("Name", kivact::signal(String::new()))),
                kivact::ui::dialog("Details", sheet)
                    .kind(kivact::ui::DialogKind::Drawer(kivact::ui::Side::Right))
                    .content(|| text("Body")),
                kivact::ui::dialog("Help topic", popover)
                    .kind(kivact::ui::DialogKind::Popover)
                    .content(|| text("Help content"))
                    .attach(kivact::ui::button("Open help")),
            )),
        )
    });
    let (modal, sheet, popover) = opens;
    for (opener, title, open) in [
        ("Open modal", "Rename", modal),
        ("Open sheet", "Details", sheet),
        ("Open help", "Help topic", popover),
    ] {
        let opener = screen.get_by_role(Role::Button, opener);
        opener.focus();
        opener.click();
        let dialog = screen.get_by_role(Role::Dialog, title);
        screen.get_by_role(Role::Heading, title);
        dialog.a11y(A11yAction::Dismiss);
        assert!(!open.get_untracked(), "{title}: Dismiss closes it");
        assert!(opener.is_focused(), "{title}: focus returns to the opener");
    }
    screen.get_by_role(Role::Button, "Open modal").click();
    let described: Vec<String> = screen
        .get_by_role(Role::Dialog, "Rename")
        .described_by()
        .iter()
        .map(|e| e.text())
        .collect();
    assert_eq!(described, ["The new name shows everywhere."]);
    screen.get_by_role(Role::Button, "Close").click();
    assert!(!modal.get_untracked(), "the close button closes the dialog");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn item_requests_can_be_ignored_by_the_caller() {
    let (screen, (tags, additions, removals)) =
        kivact::test::render_with_handle(crate::options(), || {
            let tags = signal(vec!["Initial".to_string()]);
            let additions = signal(Vec::<String>::new());
            let removals = signal(Vec::<usize>::new());
            (
                (tags, additions, removals),
                column((
                    kivact::ui::tag_input(
                        "Requested tags",
                        tags,
                        move |tag| additions.update(|items| items.push(tag)),
                        move |index| removals.update(|items| items.push(index)),
                    ),
                    ui::chip_list(vec!["Chip".to_string()]),
                )),
            )
        });
    let input = screen.get_by_role(Role::TextBox, "Requested tags");
    input.type_text("  proposed  ");
    input.press_key(Key::Enter);
    assert_eq!(additions.get_untracked(), vec!["proposed"]);
    input.press_key(Key::Backspace);
    screen.get_by_role(Role::Button, "Remove Initial").click();
    assert_eq!(removals.get_untracked(), vec![0, 0]);
    // The request is reported and the input is emptied; the caller's list
    // is the caller's to change.
    assert_eq!(input.value().as_deref(), Some(""));
    screen.get_by_role(Role::Button, "Add").click();
    screen.get_by_role(Role::Button, "Remove Chip").click();
    screen.get_by_role(Role::Button, "Remove Chip");
    assert_eq!(tags.get_untracked(), vec!["Initial"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
