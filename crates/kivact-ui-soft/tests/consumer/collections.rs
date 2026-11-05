//! Role: Exercise Soft collections and the composer through the public facade.

use std::cell::RefCell;
use std::rc::Rc;

use kivact::interaction::TreeNode;
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::ui::button;
use kivact::view::Key;
use kivact::view::NodeBuilder;
use kivact::view::Role;
use kivact::view::column;
use kivact_ui_soft as soft;

#[test]
fn tree_list_expands_and_collapses_with_arrow_keys() {
    let screen = render_with(
        RenderOptions::with_theme(soft::theme(ThemeMode::Light)),
        || {
            let nodes = vec![
                TreeNode::new(1, None, "Folder"),
                TreeNode::new(2, Some(1), "Document"),
            ];
            kivact::ui::tree("Files", nodes).expanded(signal(Vec::new()))
        },
    );

    assert_eq!(
        screen.get_by_role(Role::TreeItem, "Folder").is_expanded(),
        Some(false)
    );
    assert!(screen.query_by_role(Role::TreeItem, "Document").is_none());

    screen
        .get_by_role(Role::TreeItem, "Folder")
        .press_key(Key::ArrowRight);
    assert_eq!(
        screen.get_by_role(Role::TreeItem, "Folder").is_expanded(),
        Some(true)
    );
    screen.get_by_role(Role::TreeItem, "Document");

    screen
        .get_by_role(Role::TreeItem, "Folder")
        .press_key(Key::ArrowLeft);
    assert!(screen.query_by_role(Role::TreeItem, "Document").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn reactive_list_adds_a_list_item() {
    let screen = render_with(
        RenderOptions::with_theme(soft::theme(ThemeMode::Light)),
        || {
            let items = signal(vec![String::from("First")]);
            let rows = kivact::state::memo(move || {
                items.get().into_iter().enumerate().collect::<Vec<_>>()
            });
            let count = kivact::state::memo(move || rows.with(Vec::len));
            column((
                button("Add second")
                    .on_press(move || items.update(|rows| rows.push("Second".into()))),
                kivact::ui::item_list((kivact::view::for_each(
                    rows,
                    |(_, item)| item.clone(),
                    move |row| {
                        kivact::ui::item_row(move || row.get().1)
                            .position(move || row.get().0, count)
                    },
                ),))
                .test_id("reactive-list"),
            ))
        },
    );

    let list = screen.get_by_test_id("reactive-list");
    assert_eq!(list.role(), Some(Role::List));
    assert!(screen.query_by_role(Role::ListItem, "Second").is_none());
    screen.get_by_role(Role::Button, "Add second").click();
    let first = screen.get_by_role(Role::ListItem, "First");
    let second = screen.get_by_role(Role::ListItem, "Second");
    assert_eq!(first.pos_in_set(), Some(1));
    assert_eq!(first.set_size(), Some(2));
    assert_eq!(second.pos_in_set(), Some(2));
    assert_eq!(second.set_size(), Some(2));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn chat_composer_submits_trimmed_text_and_clears_draft_on_enter() {
    let submitted = Rc::new(RefCell::new(Vec::new()));
    let received = submitted.clone();
    let screen = render_with(
        RenderOptions::with_theme(soft::theme(ThemeMode::Light)),
        move || {
            kivact::ui::chat_composer(signal(String::new())).on_submit(move |message| {
                received.borrow_mut().push(message);
            })
        },
    );

    let input = screen.get_by_role(Role::TextBox, "Message input");
    input.type_text("  Hello  ");
    input.press_key(Key::Enter);
    assert_eq!(submitted.borrow().as_slice(), ["Hello"]);
    assert!(input.value().unwrap_or_default().is_empty());

    input.press_key(Key::Enter);
    assert_eq!(submitted.borrow().as_slice(), ["Hello"]);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
