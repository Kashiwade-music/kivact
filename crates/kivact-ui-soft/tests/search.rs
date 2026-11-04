//! Role: Verify async suggestion cancellation and token labels through the UI.
use kivact::UiError;
use kivact::interaction::ListItem;
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::style::style;
use kivact::task::sleep;
use kivact::test::RenderOptions;
use kivact::test::render_with_handle;
use kivact::view::Key;
use kivact::view::NodeBuilder;
use kivact::view::Role;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

fn options() -> RenderOptions {
    RenderOptions::from_env().theme(kivact_ui_soft::theme(ThemeMode::Light))
}

#[test]
fn debounces_and_discards_previous_suggestions() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let seen = calls.clone();
    let (screen, selected) = render_with_handle(options(), move || {
        let selected = signal(None::<String>);
        (
            selected,
            kivact_ui_soft::typeahead("Search", Vec::new(), selected)
                .style(style().width(260))
                .search(move |query| {
                    seen.borrow_mut().push(query.clone());
                    async move {
                        if query == "a" {
                            sleep(Duration::from_secs(1)).await;
                        }
                        let label = format!("Result {query}");
                        Ok::<_, UiError>(vec![ListItem::new(query, label)])
                    }
                }),
        )
    });
    assert!(calls.borrow().is_empty());
    screen.get_by_role(Role::ComboBox, "Search").click();
    screen.type_text("a");
    screen.advance(Duration::from_millis(149));
    assert!(!calls.borrow().iter().any(|query| query == "a"));
    screen.advance(Duration::from_millis(1));
    assert!(calls.borrow().iter().any(|query| query == "a"));
    screen.type_text("b");
    screen.advance(Duration::from_millis(150));
    screen.get_by_role(Role::Option, "Result ab");
    screen.advance(Duration::from_secs(1));
    assert!(screen.query_by_role(Role::Option, "Result a").is_none());
    screen.get_by_role(Role::Option, "Result ab").click();
    assert_eq!(selected.get_untracked().as_deref(), Some("ab"));
    assert_eq!(
        screen
            .get_by_role(Role::ComboBox, "Search")
            .value()
            .as_deref(),
        Some("Result ab")
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn tokenizer_keeps_remote_labels_and_excludes_selected_keys() {
    let (screen, selected) = render_with_handle(options(), || {
        let selected = signal(Vec::<&'static str>::new());
        (
            selected,
            kivact_ui_soft::tokenizer("Members", Vec::new(), selected)
                .style(style().width(260))
                .search(|_| async {
                    Ok::<_, UiError>(vec![
                        ListItem::new("ada", "Ada"),
                        ListItem::new("grace", "Grace"),
                    ])
                }),
        )
    });
    screen.get_by_role(Role::ComboBox, "Members").click();
    screen.get_by_role(Role::Option, "Ada").click();
    assert_eq!(selected.get_untracked(), vec!["ada"]);
    screen.get_by_text("Added Ada");
    screen.get_by_role(Role::Button, "Remove Ada");
    screen.get_by_role(Role::ComboBox, "Members").click();
    assert!(screen.query_by_role(Role::Option, "Ada").is_none());
    screen.get_by_role(Role::Option, "Grace");
    screen.press_key(Key::Escape);
    screen.get_by_role(Role::Button, "Remove Ada").click();
    assert!(selected.get_untracked().is_empty());
    screen.get_by_text("Removed Ada");
    let input = screen.get_by_role(Role::ComboBox, "Members");
    input.click();
    screen.get_by_role(Role::Option, "Grace").click();
    assert_eq!(selected.get_untracked(), ["grace"]);
    input.press_key(Key::Backspace);
    assert!(selected.get_untracked().is_empty());
    screen.get_by_text("Removed Grace");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
