//! Role: Tests for listbox choice policy implementation.
//!
//! `ListPolicy::listbox()` through the public `List`: vertical
//! arrows that stop at the ends and leave selection to Enter.

use crate::support::mount_with;
use kivact::interaction::{ListPolicy, ListSelection, use_list};
use kivact::prelude::*;
use kivact::test::{RenderOptions, WindowMode};
use kivact::ui::ListItem;

fn item(key: &str) -> ListItem<String> {
    ListItem::new(key.to_string(), "")
}

fn action(name: &'static str) -> Node {
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .on_press(|| ())
        .child(text(name))
        .style(style().size(120, 28))
}

#[test]
fn listbox_policy_moves_vertically_stops_at_the_ends_and_leaves_selection_to_enter() {
    let (screen, selected) = mount_with(
        RenderOptions {
            window: WindowMode::Real,
            ..RenderOptions::from_env()
        },
        || {
            let selected = signal(None::<String>);
            let group = use_list(
                vec![item("a"), item("b"), item("c")],
                ListSelection::Single((selected).into()),
            )
            .policy(ListPolicy::radio())
            .policy(ListPolicy::listbox());
            let options = column((
                node()
                    .part(group.item("a".to_owned()).role(Role::Option).name("a"))
                    .child(text("a")),
                node()
                    .part(group.item("b".to_owned()).role(Role::Option).name("b"))
                    .child(text("b")),
                node()
                    .part(group.item("c".to_owned()).role(Role::Option).name("c"))
                    .child(text("c")),
            ));
            (
                selected,
                column((
                    node().part(group.list().role(Role::ListBox)).child(options),
                    action("after"),
                )),
            )
        },
    );
    let option = |name: &str| screen.get_by_role(Role::Option, name);
    screen.press_tab(false);
    assert!(option("a").is_focused(), "no selection: the first option");
    screen.press_key(Key::ArrowUp);
    assert!(option("a").is_focused(), "Up stops at the first option");
    screen.press_key(Key::ArrowRight);
    assert!(option("a").is_focused(), "horizontal arrows do not move");
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(option("c").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(option("c").is_focused(), "Down stops at the last option");
    assert_eq!(selected.get_untracked(), None, "moving does not select");
    screen.press_key(Key::Home);
    assert!(option("a").is_focused());
    screen.press_key(Key::End);
    assert!(option("c").is_focused());
    screen.press_key(Key::Enter);
    assert_eq!(selected.get_untracked().as_deref(), Some("c"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
