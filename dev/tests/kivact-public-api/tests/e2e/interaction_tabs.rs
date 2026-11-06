//! Role: These tests verify tab parts, keyboard navigation and lifetimes.
//!
//! The unstyled `Tabs`: roles and relations, keys, activation, orientation,
//! mounting of panels, the initial selection and label changes.

use std::rc::Rc;

use crate::support::mount_with;
use kivact::interaction::{ListActivation, ListItem};
use kivact::interaction::{TabsApi, use_tabs};
use kivact::prelude::*;
use kivact::test::{MirrorMode, RenderOptions, Screen};
use kivact::view::Orientation;

type Option_ = ListItem<&'static str>;

fn real_window() -> RenderOptions {
    RenderOptions::from_env().mirror(MirrorMode::Required)
}

fn action(name: &'static str) -> Node {
    node()
        .role(Role::Button)
        .name(name)
        .focusable(true)
        .child(text(name))
        .style(style().size(120, 28))
}

fn options() -> Vec<Option_> {
    vec![
        ListItem::new("a", "Alpha"),
        ListItem::new("b", "Beta"),
        ListItem::new("c", "Gamma"),
    ]
}

fn tab(label: Value<String>) -> Node {
    node()
        .focusable(true)
        .child(text(label))
        .style(style().size(100, 28))
}

struct Handles {
    selected: Signal<&'static str>,
}

fn mount_tabs(
    initial: &'static str,
    configure: impl FnOnce(TabsApi<&'static str>) -> TabsApi<&'static str> + 'static,
) -> (Screen, Handles) {
    mount_tabs_mode(initial, configure, false)
}

fn mount_tabs_mode(
    initial: &'static str,
    configure: impl FnOnce(TabsApi<&'static str>) -> TabsApi<&'static str> + 'static,
    keep: bool,
) -> (Screen, Handles) {
    mount_with(real_window(), move || {
        let selected = signal(initial);
        let list = signal(options());
        let tabs = configure(use_tabs(list, selected));
        (
            Handles { selected },
            column((
                action("before"),
                tab_list(tabs, list),
                panels(tabs, list, keep, move |key| text(format!("content {key}"))),
                action("after"),
            )),
        )
    })
}

fn tab_of<'a>(screen: &'a Screen, name: &str) -> kivact::test::Element<'a> {
    screen.get_by_role(Role::Tab, name)
}

fn focus_first_tab(screen: &Screen) {
    screen.get_by_role(Role::Button, "before").click();
    screen.press_tab(false);
}

#[test]
fn the_list_tabs_and_the_panel_publish_roles_and_relations() {
    let (screen, h) = mount_tabs("b", |tabs| tabs);
    let list = screen.get_by_role(Role::TabList, "");
    assert_eq!(list.orientation(), Orientation::Horizontal);
    assert!(tab_of(&screen, "Beta").is_selected());
    assert!(!tab_of(&screen, "Alpha").is_selected());
    let controlled = tab_of(&screen, "Beta").controls();
    assert_eq!(controlled.len(), 1);
    assert_eq!(controlled[0].role(), Some(Role::TabPanel));
    let panel = screen.get_by_role(Role::TabPanel, "");
    let labels: Vec<String> = panel.labelled_by().iter().map(|e| e.name()).collect();
    assert_eq!(labels, ["Beta"]);
    screen.get_by_text("content b");
    // The panel takes Tab even though nothing inside it is focusable.
    focus_first_tab(&screen);
    assert!(tab_of(&screen, "Beta").is_focused());
    screen.press_tab(false);
    assert!(panel.is_focused());
    screen.press_tab(false);
    assert!(screen.get_by_role(Role::Button, "after").is_focused());
    assert_eq!(h.selected.get_untracked(), "b");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn automatic_activation_selects_the_tab_that_gets_the_focus() {
    let (screen, h) = mount_tabs("a", |tabs| tabs);
    focus_first_tab(&screen);
    assert!(tab_of(&screen, "Alpha").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(tab_of(&screen, "Beta").is_focused());
    assert_eq!(h.selected.get_untracked(), "b");
    screen.get_by_text("content b");
    assert!(screen.query_by_text("content a").is_none());
    screen.press_key(Key::ArrowDown);
    assert_eq!(h.selected.get_untracked(), "b", "vertical arrows idle");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn manual_activation_moves_focus_and_enter_or_space_selects() {
    let (screen, h) = mount_tabs("a", |tabs| tabs.activation(ListActivation::Manual));
    focus_first_tab(&screen);
    screen.press_key(Key::ArrowRight);
    assert!(tab_of(&screen, "Beta").is_focused());
    assert_eq!(h.selected.get_untracked(), "a");
    screen.get_by_text("content a");
    screen.press_key(Key::Enter);
    assert_eq!(h.selected.get_untracked(), "b");
    screen.get_by_text("content b");
    screen.press_key(Key::ArrowRight);
    screen.press_key(Key::Space);
    assert_eq!(h.selected.get_untracked(), "c");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn tab_list(tabs: TabsApi<&'static str>, options: Signal<Vec<Option_>>) -> Node {
    node().part(tabs.list()).child(for_each(
        options,
        |option| *option.key(),
        move |option| {
            let key = option.with(|option| *option.key());
            let label = memo(move || option.with(|option| option.label()));
            tab(label).part(tabs.item(key))
        },
    ))
}

fn panels<V: IntoView>(
    tabs: TabsApi<&'static str>,
    options: Signal<Vec<Option_>>,
    keep: bool,
    content: impl Fn(&&'static str) -> V + 'static,
) -> Node {
    let content = Rc::new(content);
    node().child(for_each(
        options,
        |option| *option.key(),
        move |row_value| {
            let key = row_value.with(|option| *option.key());
            let content = content.clone();
            let build = move || node().part(tabs.panel(key)).child(content(&key));
            if keep {
                build().into_view()
            } else {
                let selected = tabs.selected();
                show(move || selected.get() == key, build).into_view()
            }
        },
    ))
}
