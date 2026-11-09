//! Role: The unstyled `Select`: a combo box that opens a list box.
//! Keys at the
//! closed trigger, opening, keys in the list, choosing, closing, focus return
//! and semantics.

use kivact::view::View;

use crate::support::mount_with;
use kivact::interaction::ListItem;
use kivact::interaction::{SelectApi, use_select};
use kivact::prelude::*;
use kivact::test::{Locator, MirrorMode, RenderOptions, Screen};
use kivact::view::Modifiers;
use kivact::view::{A11yAction, Placement, overlay};

type Opt = ListItem<&'static str>;

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

fn fruit() -> Vec<Opt> {
    vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("apricot", "Apricot"),
        ListItem::new("banana", "Banana").disabled(true),
        ListItem::new("blueberry", "Blueberry"),
        ListItem::new("cherry", "Cherry"),
    ]
}

fn alt() -> Modifiers {
    Modifiers {
        alt: true,
        ..Modifiers::default()
    }
}

/// The trigger `trigger` with the select's label and, while open, the list
/// below it: rows of 28 px, 7.5 of them before it scrolls.
fn select_view<K: ItemKey>(
    trigger: Node,
    name: &'static str,
    options: Value<Vec<ListItem<K>>>,
    select: SelectApi<K>,
) -> View {
    let part = select.trigger();
    let anchor = node_ref();
    let label = select.label();
    (
        trigger
            .part(part)
            .node_ref(anchor)
            .name(name)
            .child(text(label)),
        show(select.is_open(), move || {
            let rows = for_each(
                options,
                |option| option.key().clone(),
                move |option| {
                    let key = option.with(|option| option.key().clone());
                    let label = memo(move || option.with(|option| option.label()));
                    let part = select.item(key);
                    let selected = part.state();
                    node()
                        .part(part)
                        .style(style().height(28).shrink(0).padding_x(8).gap(8))
                        .child(dynamic(memo(move || selected.get().selected), |chosen| {
                            chosen.then(|| text("\u{2713}"))
                        }))
                        .child(text(label))
                },
            );
            overlay(
                anchor,
                Placement::Below,
                column(scroll(rows).style(style().min_height(0).max_height(7.5 * 28.0)))
                    .style(style().min_width(160).padding(4)),
            )
            .offset(4)
            .with_node(move |node| node.part(select.content()))
            .name(name)
        }),
    )
        .into_view()
}

/// The trigger "Fruit" and "After", the next tab stop after it.
fn mount_select(
    options: impl IntoValue<Vec<Opt>> + 'static,
    initial: Option<&'static str>,
) -> (Screen, Signal<Option<&'static str>>) {
    mount_with(real_window(), move || {
        let selected = signal(initial);
        let options = options.into_value();
        let fruit = use_select(options, selected);
        let trigger = select_view(node().style(style().size(160, 28)), "Fruit", options, fruit);
        (selected, column((trigger, action("After"))))
    })
}

fn option<'a>(screen: &'a Screen, name: &str) -> Locator<'a> {
    screen.get_by_role(Role::Option, name)
}

fn closed(screen: &Screen) -> bool {
    screen.query_by_role(Role::ListBox, "Fruit").is_none()
}

fn trigger(screen: &Screen) -> Locator<'_> {
    screen.get_by_role(Role::ComboBox, "Fruit")
}

#[test]
fn a_closed_trigger_moves_the_selection_with_arrows_and_stops_at_the_ends() {
    let (screen, selected) = mount_select(fruit(), None);
    let trigger = trigger(&screen);
    trigger.press_key(Key::ArrowDown);
    assert_eq!(
        selected.get_untracked(),
        Some("apple"),
        "nothing: the first"
    );
    assert!(closed(&screen), "moving does not open the list");
    screen.press_key(Key::ArrowDown);
    assert_eq!(selected.get_untracked(), Some("apricot"));
    screen.press_key(Key::ArrowDown);
    assert_eq!(
        selected.get_untracked(),
        Some("blueberry"),
        "the disabled option is skipped"
    );
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert_eq!(
        selected.get_untracked(),
        Some("cherry"),
        "Down stops at the end"
    );
    screen.press_key(Key::ArrowUp);
    assert_eq!(selected.get_untracked(), Some("blueberry"));
    screen.press_key(Key::Home);
    assert_eq!(selected.get_untracked(), Some("apple"));
    screen.press_key(Key::ArrowUp);
    assert_eq!(
        selected.get_untracked(),
        Some("apple"),
        "Up stops at the start"
    );
    screen.press_key(Key::End);
    assert_eq!(selected.get_untracked(), Some("cherry"));
    screen.press_key(Key::ArrowRight);
    assert_eq!(
        selected.get_untracked(),
        Some("cherry"),
        "horizontal keys do nothing"
    );
    assert!(closed(&screen));
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn alt_down_f4_enter_space_a_click_and_expand_open_the_list() {
    let (screen, _) = mount_select(fruit(), None);
    let trigger = trigger(&screen);
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(closed(&screen));

    trigger.press_key_with(Key::ArrowDown, alt());
    assert_eq!(trigger.is_expanded(), Some(true), "Alt+Down");
    screen.press_key(Key::Escape);
    assert!(closed(&screen));

    trigger.press_key_with(Key::ArrowUp, alt());
    assert_eq!(
        trigger.is_expanded(),
        Some(true),
        "Alt+Up opens a closed one"
    );
    screen.press_key(Key::Escape);

    trigger.press_key(Key::F(4));
    assert_eq!(trigger.is_expanded(), Some(true), "F4");
    screen.press_key(Key::Escape);

    trigger.press_key(Key::Enter);
    assert_eq!(trigger.is_expanded(), Some(true), "Enter");
    screen.press_key(Key::Escape);

    trigger.press_key(Key::Space);
    assert_eq!(trigger.is_expanded(), Some(true), "Space");
    screen.press_key(Key::Escape);

    trigger.click();
    assert_eq!(trigger.is_expanded(), Some(true), "a click");
    trigger.click();
    assert_eq!(trigger.is_expanded(), Some(false), "a second click closes");

    trigger.a11y(A11yAction::Expand);
    assert_eq!(trigger.is_expanded(), Some(true), "assistive Expand");
    trigger.a11y(A11yAction::Collapse);
    assert_eq!(trigger.is_expanded(), Some(false), "assistive Collapse");
    assert!(trigger.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn enter_space_a_click_and_invoke_choose_the_focused_option_and_close() {
    let (screen, selected) = mount_select(fruit(), None);
    let trigger = trigger(&screen);

    trigger.press_key(Key::Enter);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), Some("apricot"), "Enter");
    assert!(closed(&screen));
    assert!(trigger.is_focused(), "the focus is back on the trigger");
    assert_eq!(trigger.value().as_deref(), Some("Apricot"));

    trigger.press_key(Key::Enter);
    screen.press_key(Key::End);
    screen.press_key(Key::Space);
    assert_eq!(selected.get_untracked(), Some("cherry"), "Space");
    assert!(closed(&screen));
    assert!(trigger.is_focused());

    trigger.press_key(Key::Enter);
    option(&screen, "Blueberry").click();
    assert_eq!(selected.get_untracked(), Some("blueberry"), "a click");
    assert!(closed(&screen));
    assert!(trigger.is_focused());

    trigger.press_key(Key::Enter);
    option(&screen, "Apple").a11y(A11yAction::Activate);
    assert_eq!(
        selected.get_untracked(),
        Some("apple"),
        "assistive Activate"
    );
    assert!(closed(&screen));

    trigger.press_key(Key::Enter);
    assert!(option(&screen, "Banana").is_disabled());
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(
        selected.get_untracked(),
        Some("blueberry"),
        "the disabled option cannot take the focus, so it is never chosen"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_trigger_is_a_combo_box_and_the_list_a_list_box_of_options() {
    let (screen, _) = mount_select(fruit(), Some("cherry"));
    let trigger = trigger(&screen);
    assert_eq!(trigger.role(), Some(Role::ComboBox));
    assert_eq!(trigger.value().as_deref(), Some("Cherry"));
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(trigger.controls().is_empty(), "no list, nothing to control");

    trigger.press_key(Key::Enter);
    assert_eq!(trigger.is_expanded(), Some(true));
    let list = screen.get_by_role(Role::ListBox, "Fruit");
    let controlled = trigger.controls();
    assert_eq!(controlled.len(), 1);
    assert_eq!(controlled[0].role(), Some(Role::ListBox));
    assert!(list.exists());
    assert!(option(&screen, "Cherry").is_selected());
    assert!(!option(&screen, "Apple").is_selected());
    assert!(option(&screen, "Banana").is_disabled());
    assert_eq!(
        screen.all_by_role(Role::Option, "Apple").len(),
        1,
        "one row per option"
    );

    option(&screen, "Apple").a11y(A11yAction::Activate);
    assert_eq!(trigger.value().as_deref(), Some("Apple"));
    trigger.press_key(Key::Enter);
    assert!(option(&screen, "Apple").is_selected());
    assert!(!option(&screen, "Cherry").is_selected());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_preview_given_after_the_content_part_is_reported() {
    let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::<Option<&'static str>>::new()));
    let seen = log.clone();
    let (screen, _selected) = mount_with(real_window(), move || {
        let selected = signal(None::<&'static str>);
        let options = fruit().into_value();
        let api = use_select(options, selected);
        // The surface part exists before the preview is given.
        let content = std::cell::RefCell::new(Some(api.content()));
        let api = api.on_active_change(move |key| seen.borrow_mut().push(key));
        let part = api.trigger();
        let anchor = node_ref();
        let label = api.label();
        let list = show(api.is_open(), move || {
            let rows = for_each(
                options,
                |option| *option.key(),
                move |option| {
                    let key = option.with(|option| *option.key());
                    node()
                        .part(api.item(key))
                        .style(style().height(28).shrink(0))
                        .child(text(move || option.with(|option| option.label())))
                },
            );
            let content = content
                .borrow_mut()
                .take()
                .expect("invariant: the list opens once");
            overlay(anchor, Placement::Below, column(rows))
                .with_node(move |node| node.part(content))
                .name("Fruit")
        });
        (
            selected,
            column((
                node()
                    .part(part)
                    .node_ref(anchor)
                    .name("Fruit")
                    .style(style().size(160, 28))
                    .child(text(label)),
                list,
            )),
        )
    });
    trigger(&screen).click();
    assert_eq!(log.borrow().last(), Some(&Some("apple")));
    screen.press_key(Key::Escape);
    assert_eq!(
        log.borrow().last(),
        Some(&None),
        "the preview ends with the list"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
