//! Role: These tests verify tab parts, keyboard navigation and lifetimes.
//!
//! The unstyled `Tabs`: roles and relations, keys, activation, orientation,
//! mounting of panels, the initial selection and label changes.

use std::cell::Cell;
use std::rc::Rc;

use crate::support::mount_with;
use kivact::i18n::{Messages, message};
use kivact::interaction::{TabsApi, use_tabs};
use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, WindowMode};
use kivact::ui::{ListActivation, ListItem};
use kivact::view::Orientation;
use kivact::view::provide_messages;

type Option_ = ListItem<&'static str>;

fn real_window() -> RenderOptions {
    RenderOptions {
        window: WindowMode::Real,
        ..RenderOptions::from_env()
    }
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
    options: Signal<Vec<Option_>>,
    built: Rc<Cell<u32>>,
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
        let built = Rc::new(Cell::new(0));
        let counter = built.clone();
        let tabs = configure(use_tabs(list, selected));
        (
            Handles {
                selected,
                options: list,
                built,
            },
            column((
                action("before"),
                tab_list(tabs, list),
                panels(tabs, list, keep, move |key| {
                    counter.set(counter.get() + 1);
                    text(format!("content {key}"))
                }),
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

#[test]
fn a_vertical_list_uses_up_and_down() {
    let (screen, h) = mount_tabs("a", |tabs| tabs.orientation(Orientation::Vertical));
    assert_eq!(
        screen.get_by_role(Role::TabList, "").orientation(),
        Orientation::Vertical
    );
    focus_first_tab(&screen);
    screen.press_key(Key::ArrowRight);
    assert!(
        tab_of(&screen, "Alpha").is_focused(),
        "horizontal arrows idle"
    );
    screen.press_key(Key::ArrowDown);
    assert!(tab_of(&screen, "Beta").is_focused());
    assert_eq!(h.selected.get_untracked(), "b");
    screen.press_key(Key::ArrowUp);
    assert!(tab_of(&screen, "Alpha").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn home_and_end_jump_and_the_arrows_wrap_and_skip_disabled_tabs() {
    let (screen, h) = mount_tabs("a", |tabs| tabs);
    h.options
        .update(|rows| rows[1] = rows[1].clone().disabled(true));
    screen.run_until_idle();
    assert!(tab_of(&screen, "Beta").is_disabled());
    focus_first_tab(&screen);
    screen.press_key(Key::ArrowRight);
    assert!(tab_of(&screen, "Gamma").is_focused(), "Beta is skipped");
    assert_eq!(h.selected.get_untracked(), "c");
    screen.press_key(Key::ArrowRight);
    assert!(tab_of(&screen, "Alpha").is_focused(), "wraps to the first");
    screen.press_key(Key::ArrowLeft);
    assert!(tab_of(&screen, "Gamma").is_focused(), "wraps to the last");
    screen.press_key(Key::Home);
    assert!(tab_of(&screen, "Alpha").is_focused());
    screen.press_key(Key::End);
    assert!(tab_of(&screen, "Gamma").is_focused());
    assert_eq!(h.selected.get_untracked(), "c");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn only_the_selected_panel_is_mounted_and_it_is_built_again_on_return() {
    let (screen, h) = mount_tabs("a", |tabs| tabs);
    assert_eq!(h.built.get(), 1);
    tab_of(&screen, "Beta").click();
    assert_eq!(h.built.get(), 2);
    assert!(screen.query_by_text("content a").is_none());
    let panels = screen.all_by_role(Role::TabPanel, "");
    assert_eq!(panels.len(), 1);
    assert_eq!(panels[0].labelled_by()[0].name(), "Beta");
    tab_of(&screen, "Alpha").click();
    assert_eq!(h.built.get(), 3, "coming back builds it again");
    screen.get_by_text("content a");
    // A selected key without an option shows no panel and stays selected.
    h.options
        .update(|rows| rows.retain(|row| *row.key() != "a"));
    screen.run_until_idle();
    assert!(screen.query_by_text("content a").is_none());
    assert_eq!(h.selected.get_untracked(), "a");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn keep_mounted_builds_every_panel_once_and_hides_the_others() {
    let (screen, h) = mount_tabs_mode("a", |tabs| tabs, true);
    assert_eq!(h.built.get(), 3);
    // A hidden panel is not found by its text.
    screen.get_by_text("content a");
    assert!(screen.query_by_text("content b").is_none());
    assert_eq!(screen.all_by_role(Role::TabPanel, "").len(), 1);
    tab_of(&screen, "Beta").click();
    assert_eq!(h.built.get(), 3, "no panel is built again");
    assert!(screen.query_by_text("content a").is_none());
    screen.get_by_text("content b");
    tab_of(&screen, "Alpha").click();
    assert_eq!(h.built.get(), 3);
    screen.get_by_text("content a");
    let controlled = tab_of(&screen, "Gamma").controls();
    assert_eq!(controlled.len(), 1, "every tab names its own panel");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn labels_follow_a_locale_switch_and_a_rename_without_remounting() {
    let (screen, (messages, list)) = mount_with(real_window(), || {
        let messages = signal(Messages::new());
        let selected = signal("a");
        let list = signal(Vec::<Option_>::new());
        (
            (messages, list),
            provide_messages(messages, move || {
                let alpha = message("tab.alpha", "Alpha");
                list.set(vec![ListItem::new("a", alpha), ListItem::new("b", "Beta")]);
                let tabs = use_tabs(list, selected);
                column((
                    tab_list(tabs, list),
                    panels(tabs, list, false, |key| text(format!("content {key}"))),
                ))
            }),
        )
    });
    tab_of(&screen, "Alpha");
    messages.set(Messages::new().with("tab.alpha", "アルファ"));
    screen.run_until_idle();
    let renamed = tab_of(&screen, "アルファ");
    assert!(renamed.is_selected());
    assert!(screen.query_by_role(Role::Tab, "Alpha").is_none());
    let creates = screen.stats().creates;
    list.update(|rows| rows[1] = ListItem::new(*rows[1].key(), "Beta renamed"));
    screen.run_until_idle();
    tab_of(&screen, "Beta renamed");
    assert_eq!(screen.stats().creates, creates, "a rename mounts nothing");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn tab_list(tabs: TabsApi<&'static str>, options: Signal<Vec<Option_>>) -> Node {
    node().part(tabs.list()).child(for_each(
        options,
        |option| *option.key(),
        move |key, option| {
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
        move |key, _| {
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

#[test]
fn final_caller_constraints_and_dynamic_order_control_roving_candidates() {
    let (screen, (rows, hidden, disabled, eligible)) = mount_with(real_window(), || {
        let rows = signal(options());
        let hidden = signal(false);
        let disabled = signal(false);
        let eligible = signal(true);
        let tabs = use_tabs(rows, signal("a"));
        let list = node().part(tabs.list()).child(for_each(
            rows,
            |row| *row.key(),
            move |key, row| {
                let label = memo(move || row.with(|row| row.label()));
                tab(label)
                    .part(tabs.item(key))
                    .hidden(move || key == "b" && hidden.get())
                    .disabled(move || key == "b" && disabled.get())
                    .focusable(move || key != "b" || eligible.get())
            },
        ));
        (
            (rows, hidden, disabled, eligible),
            column((action("before"), list)),
        )
    });
    focus_first_tab(&screen);
    hidden.set(true);
    screen.run_until_idle();
    screen.press_key(Key::ArrowRight);
    assert!(tab_of(&screen, "Gamma").is_focused());
    hidden.set(false);
    disabled.set(true);
    screen.run_until_idle();
    screen.press_key(Key::ArrowLeft);
    assert!(tab_of(&screen, "Alpha").is_focused());
    disabled.set(false);
    eligible.set(false);
    screen.run_until_idle();
    screen.press_key(Key::ArrowRight);
    assert!(tab_of(&screen, "Gamma").is_focused());
    eligible.set(true);
    rows.update(|rows| rows.swap(0, 1));
    screen.run_until_idle();
    screen.press_key(Key::Home);
    assert!(tab_of(&screen, "Beta").is_focused());
    rows.update(|rows| rows.retain(|row| *row.key() != "b"));
    screen.run_until_idle();
    assert!(tab_of(&screen, "Alpha").is_focused());
    rows.update(|rows| rows.insert(0, ListItem::new("b", "Beta")));
    screen.run_until_idle();
    screen.press_key(Key::Home);
    assert!(tab_of(&screen, "Beta").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn duplicate_tab_ownership_is_reported_and_rejected() {
    let (screen, selected) = mount_with(real_window(), || {
        let selected = signal("a");
        let tabs = use_tabs(options(), selected);
        (
            selected,
            column((node().part(tabs.list()).child((
                tab(("Alpha".to_owned()).into_value()).part(tabs.item("a")),
                tab(("Duplicate".to_owned()).into_value())
                    .part(tabs.item("a"))
                    .name("Duplicate"),
                tab(("Beta".to_owned()).into_value()).part(tabs.item("b")),
            )),)),
        )
    });
    assert!(tab_of(&screen, "Duplicate").is_disabled());
    tab_of(&screen, "Beta").click();
    assert_eq!(selected.get(), "b");
    assert!(
        screen
            .errors()
            .iter()
            .any(|error| error.to_string().contains("already mounted"))
    );
}
