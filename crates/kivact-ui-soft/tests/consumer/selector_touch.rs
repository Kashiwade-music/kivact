//! Role: Verify touch and keyboard Selector presentations over one binding.

use kivact::interaction::ListItem;
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::style::style;
use kivact::test::RenderOptions;
use kivact::test::render_with_handle;
use kivact::ui::text_input;
use kivact::view::Key;
use kivact::view::NodeBuilder;
use kivact::view::PointerKind;
use kivact::view::PopupKind;
use kivact::view::Role;
use std::time::Duration;

#[test]
fn compact_touch_uses_a_latched_dialog_and_keyboard_uses_the_popup() {
    let (screen, selected) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light))
            .window_size(500.0, 700.0),
        || {
            let selected = signal(Some("a"));
            let field = kivact::ui::select(
                "Team",
                vec![
                    ListItem::new("a", "Alpha"),
                    ListItem::new("skip", "Unavailable").disabled(true),
                    ListItem::new("b", "Beta"),
                ],
                selected,
            )
            .open_on_arrow(true)
            .presentation(kivact::ui::SelectPresentation::BottomSheet)
            .style(style().width(280));
            (selected, field)
        },
    );
    let combo = || screen.get_by_role(Role::ComboBox, "Team");
    assert_eq!(combo().popup_kind(), Some(PopupKind::ListBox));
    let rect = combo().rect().unwrap();
    let x = rect.x + rect.width / 2.0;
    let y = rect.y + rect.height / 2.0;
    let touch = screen.contact_down(PointerKind::Touch, x, y);
    assert!(screen.query_by_role(Role::Dialog, "Team").is_none());
    screen.contact_up(touch);
    screen.advance(Duration::from_millis(350));
    screen.get_by_role(Role::Dialog, "Team");
    assert_eq!(combo().popup_kind(), Some(PopupKind::Dialog));
    assert!(screen.get_by_role(Role::Option, "Alpha").is_focused());
    assert_eq!(selected.get(), Some("a"));

    screen.resize(900.0, 700.0);
    screen.get_by_role(Role::Dialog, "Team");
    screen.press_key(Key::ArrowDown);
    assert!(screen.get_by_role(Role::Option, "Beta").is_focused());
    assert_eq!(selected.get(), Some("a"));
    screen.press_key(Key::Enter);
    assert_eq!(selected.get(), Some("b"));
    assert!(screen.query_by_role(Role::Dialog, "Team").is_none());
    assert!(combo().is_focused());

    screen.resize(500.0, 700.0);
    combo().press_key(Key::ArrowDown);
    screen.get_by_role(Role::ListBox, "Team");
    assert!(screen.query_by_role(Role::Dialog, "Team").is_none());
    assert_eq!(combo().popup_kind(), Some(PopupKind::ListBox));
    assert!(combo().is_focused());
    assert_eq!(selected.get(), Some("b"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn multi_selector_filters_and_commits_in_the_touch_sheet() {
    let (screen, selected) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light))
            .window_size(500.0, 700.0),
        || {
            let selected = signal(vec!["a"]);
            let field = kivact::ui::combobox(
                "Teams",
                vec![ListItem::new("a", "Alpha"), ListItem::new("b", "Beta")],
                selected,
            )
            .chips()
            .presentation(kivact::ui::SelectPresentation::BottomSheet)
            .style(style().width(280));
            (selected, field)
        },
    );
    let input = screen.get_by_role(Role::ComboBox, "Teams");
    let rect = input.rect().unwrap();
    let touch = screen.contact_down(
        PointerKind::Touch,
        rect.x + rect.width / 2.0,
        rect.y + rect.height / 2.0,
    );
    screen.contact_up(touch);
    screen.advance(Duration::from_millis(350));
    screen.get_by_role(Role::Dialog, "Teams");
    screen.get_by_role(Role::ComboBox, "Search options").click();
    screen.type_text("bet");
    assert!(screen.query_by_role(Role::Option, "Alpha").is_none());
    screen.get_by_role(Role::Option, "Beta").click();
    assert_eq!(selected.get(), vec!["a", "b"]);
    screen.get_by_role(Role::Dialog, "Teams");
    screen.press_key(Key::Escape);
    assert!(screen.query_by_role(Role::Dialog, "Teams").is_none());
    assert!(input.is_focused());
}

#[test]
fn complex_selector_uses_the_same_search_binding_in_its_two_surfaces() {
    let (screen, query) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light))
            .window_size(500.0, 700.0),
        || {
            let query = signal(String::new());
            (
                query,
                kivact::ui::selection_field("Workspace", "Design")
                    .presentation(kivact::ui::SelectPresentation::BottomSheet)
                    .content(move || text_input("Filter workspaces", query)),
            )
        },
    );
    let trigger = screen.get_by_role(Role::Button, "Workspace");
    let rect = trigger.rect().unwrap();
    let touch = screen.contact_down(
        PointerKind::Touch,
        rect.x + rect.width / 2.0,
        rect.y + rect.height / 2.0,
    );
    screen.contact_up(touch);
    screen.advance(Duration::from_millis(350));
    screen
        .get_by_role(Role::TextBox, "Filter workspaces")
        .click();
    screen.type_text("des");
    assert_eq!(query.get(), "des");
    screen.press_key(Key::Escape);
    screen.advance(Duration::from_millis(350));
    trigger.press_key(Key::Enter);
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Filter workspaces")
            .value()
            .as_deref(),
        Some("des"),
    );
}

#[test]
fn selection_field_reopened_through_the_caller_binding_takes_a_fresh_surface() {
    let (screen, open) = render_with_handle(
        RenderOptions::with_theme(kivact_ui_soft::theme(ThemeMode::Light))
            .window_size(500.0, 700.0),
        || {
            let open = signal(false);
            (
                open,
                kivact::ui::selection_field("Workspace", "Design")
                    .presentation(kivact::ui::SelectPresentation::BottomSheet)
                    .open(open)
                    .content(|| kivact::view::text("Choices")),
            )
        },
    );
    // Only a drawer carries the drag-to-close handle; the popup opens without one.
    let drawer_shown = || screen.query_by_role(Role::Group, "Drag to close").is_some();
    let trigger = screen.get_by_role(Role::Button, "Workspace");
    let rect = trigger.rect().unwrap();
    let touch = screen.contact_down(
        PointerKind::Touch,
        rect.x + rect.width / 2.0,
        rect.y + rect.height / 2.0,
    );
    screen.contact_up(touch);
    screen.advance(Duration::from_millis(350));
    assert!(open.get());
    assert!(drawer_shown(), "a touch tap opens the drawer");

    open.set(false);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(350));
    assert!(screen.query_by_role(Role::Dialog, "Workspace").is_none());

    // The caller reopens: a new opening without a tap shows the popup.
    open.set(true);
    screen.run_until_idle();
    screen.advance(Duration::from_millis(350));
    assert!(
        !drawer_shown(),
        "the caller's reopening must not reuse the drawer"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
