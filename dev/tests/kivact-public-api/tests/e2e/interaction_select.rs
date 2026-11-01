//! Role: The unstyled `Select`: a combo box that opens a list box.
//! Keys at the
//! closed trigger, opening, keys in the list, choosing, closing, focus return
//! and semantics.

use kivact::primitive::NodeExt;
use kivact::view::View;
use std::hash::Hash;
use std::time::Duration;

use crate::support::mount_with;
use kivact::i18n::{Locale, Messages, message};
use kivact::interaction::{SelectApi, use_select};
use kivact::prelude::*;
use kivact::test::{Element, RenderOptions, Screen, WindowMode};
use kivact::ui::ListItem;
use kivact::view::Modifiers;
use kivact::view::{A11yAction, Placement, overlay};
use kivact::view::{provide_locale, provide_messages};

type Opt = ListItem<&'static str>;

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
fn select_view<K: Clone + Eq + Hash + 'static>(
    trigger: Node,
    name: &'static str,
    options: Value<Vec<ListItem<K>>>,
    select: SelectApi<K>,
) -> View {
    let part = select.trigger();
    let anchor = part.node_ref();
    let label = select.label();
    (
        trigger.part(part).name(name).child(text(label)),
        show(select.shown(), move || {
            let rows = for_each(
                options,
                |option| option.key().clone(),
                move |key, option| {
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
            .part(select.content())
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

fn option<'a>(screen: &'a Screen, name: &str) -> Element<'a> {
    screen.get_by_role(Role::Option, name)
}

fn closed(screen: &Screen) -> bool {
    screen.query_by_role(Role::ListBox, "Fruit").is_none()
}

fn trigger(screen: &Screen) -> Element<'_> {
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
fn letters_at_a_closed_trigger_select_the_next_label_that_starts_with_them() {
    let (screen, selected) = mount_select(fruit(), None);
    let trigger = trigger(&screen);
    trigger.press_key(Key::Char('a'));
    assert_eq!(selected.get_untracked(), Some("apple"));
    screen.press_key(Key::Char('p'));
    assert_eq!(
        selected.get_untracked(),
        Some("apple"),
        "\"ap\" within 500 ms is one prefix: Apple stays"
    );
    screen.press_key(Key::Char('r'));
    assert_eq!(selected.get_untracked(), Some("apricot"), "\"apr\"");
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('b'));
    assert_eq!(
        selected.get_untracked(),
        Some("blueberry"),
        "the disabled Banana is skipped"
    );
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('a'));
    assert_eq!(selected.get_untracked(), Some("apple"), "it cycles round");
    screen.press_key(Key::Char('z'));
    assert_eq!(
        selected.get_untracked(),
        Some("apple"),
        "no match: no change"
    );
    assert!(closed(&screen), "typing does not open the list");
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
fn the_open_list_focuses_the_selected_option_or_else_the_first_enabled_one() {
    let (screen, selected) = mount_select(fruit(), Some("blueberry"));
    let trigger = trigger(&screen);
    trigger.press_key(Key::Enter);
    assert!(option(&screen, "Blueberry").is_focused());
    screen.press_key(Key::Escape);

    selected.set(None);
    trigger.press_key(Key::Enter);
    assert!(
        option(&screen, "Apple").is_focused(),
        "none: the first enabled"
    );
    screen.press_key(Key::Escape);

    // A selected option that is disabled cannot take the focus.
    let (screen, _) = mount_select(
        vec![
            ListItem::new("x", "Locked").disabled(true),
            ListItem::new("y", "Open"),
        ],
        Some("x"),
    );
    self::trigger(&screen).press_key(Key::Enter);
    assert!(option(&screen, "Open").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn arrows_in_the_open_list_move_the_focus_and_leave_the_selection() {
    let (screen, selected) = mount_select(fruit(), Some("apple"));
    trigger(&screen).press_key_with(Key::ArrowDown, alt());
    assert!(option(&screen, "Apple").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(option(&screen, "Apricot").is_focused());
    screen.press_key(Key::ArrowDown);
    assert!(
        option(&screen, "Blueberry").is_focused(),
        "the disabled option is skipped"
    );
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    assert!(
        option(&screen, "Cherry").is_focused(),
        "Down stops at the end"
    );
    screen.press_key(Key::Home);
    assert!(option(&screen, "Apple").is_focused());
    screen.press_key(Key::ArrowUp);
    assert!(
        option(&screen, "Apple").is_focused(),
        "Up stops at the start"
    );
    screen.press_key(Key::End);
    assert!(option(&screen, "Cherry").is_focused());
    screen.press_key(Key::ArrowRight);
    assert!(option(&screen, "Cherry").is_focused());
    assert_eq!(
        selected.get_untracked(),
        Some("apple"),
        "moving never selects"
    );

    screen.press_key(Key::Char('a'));
    assert!(
        option(&screen, "Apple").is_focused(),
        "typeahead moves the focus"
    );
    screen.press_key(Key::Char('p'));
    screen.press_key(Key::Char('r'));
    assert!(option(&screen, "Apricot").is_focused());
    screen.advance(Duration::from_millis(600));
    screen.press_key(Key::Char('b'));
    assert!(option(&screen, "Blueberry").is_focused());
    assert_eq!(
        selected.get_untracked(),
        Some("apple"),
        "typing never selects"
    );
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
fn escape_and_alt_up_close_without_choosing_and_tab_moves_the_focus_on() {
    let (screen, selected) = mount_select(fruit(), Some("apple"));
    let trigger = trigger(&screen);

    trigger.press_key(Key::Enter);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Escape);
    assert!(closed(&screen));
    assert!(trigger.is_focused(), "Escape gives the focus back");
    assert_eq!(selected.get_untracked(), Some("apple"));

    trigger.press_key(Key::Enter);
    screen.press_key(Key::ArrowDown);
    screen.press_key_with(Key::ArrowUp, alt());
    assert!(closed(&screen), "Alt+Up closes an open list");
    assert!(trigger.is_focused());
    assert_eq!(selected.get_untracked(), Some("apple"));

    trigger.press_key(Key::Enter);
    screen.press_key(Key::ArrowDown);
    screen.press_tab(false);
    assert!(closed(&screen), "Tab closes it");
    assert!(
        screen.get_by_role(Role::Button, "After").is_focused(),
        "and the focus goes where Tab goes"
    );
    assert_eq!(selected.get_untracked(), Some("apple"));

    trigger.press_key(Key::Enter);
    screen.press_tab(true);
    assert!(closed(&screen), "Shift+Tab closes it");
    assert!(trigger.is_focused());
    assert_eq!(selected.get_untracked(), Some("apple"));
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
fn a_long_list_is_bounded_by_max_rows_and_scrolls_to_the_focus() {
    let (screen, _) = mount_with(real_window(), || {
        let rows: Vec<ListItem<u32>> = (0..40)
            .map(|i| ListItem::new(i, format!("Item {i:02}")))
            .collect();
        let selected = signal(None::<u32>);
        let rows = rows.into_value();
        (
            (),
            select_view(node(), "Number", rows, use_select(rows, selected)),
        )
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Number");
    trigger.press_key(Key::Enter);
    let list = screen
        .get_by_role(Role::ListBox, "Number")
        .rect()
        .expect("laid out");
    let row = screen
        .get_by_role(Role::Option, "Item 00")
        .rect()
        .expect("laid out");
    // 7.5 rows of the list, plus its padding.
    assert!(
        list.height < 40.0 * row.height / 2.0,
        "the list is bounded: {list:?} for rows of {row:?}"
    );
    assert!(
        list.height > 7.0 * row.height && list.height < 9.0 * row.height,
        "about 7.5 rows: {list:?} for rows of {row:?}"
    );
    for step in 0..42 {
        screen.press_key(Key::ArrowDown);
        let focused = screen.focused().expect("a row has the focus");
        let rect = focused.rect().expect("laid out");
        assert!(
            rect.y >= list.y - 0.5 && rect.y + rect.height <= list.y + list.height + 0.5,
            "step {step}: {rect:?} outside {list:?}"
        );
    }
    assert_eq!(
        screen.focused().map(|e| e.name()).as_deref(),
        Some("Item 39"),
        "Down stops at the last row"
    );
    screen.press_key(Key::Home);
    let first = screen
        .focused()
        .expect("a row has the focus")
        .rect()
        .expect("laid out");
    assert!(first.y >= list.y - 0.5, "Home scrolls back: {first:?}");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_placeholder_shows_until_something_is_selected() {
    let (screen, selected) = mount_with(real_window(), || {
        let selected = signal(None::<&'static str>);
        let options = (fruit()).into_value();
        let default = use_select(options, selected);
        let custom = use_select(options, selected).placeholder("Pick a fruit");
        (
            selected,
            column((
                select_view(node(), "Default", options, default),
                select_view(node(), "Custom", options, custom),
            )),
        )
    });
    let default = screen.get_by_role(Role::ComboBox, "Default");
    let custom = screen.get_by_role(Role::ComboBox, "Custom");
    assert_eq!(default.value().as_deref(), Some("Select…"));
    assert_eq!(custom.value().as_deref(), Some("Pick a fruit"));
    selected.set(Some("cherry"));
    screen.run_until_idle();
    assert_eq!(default.value().as_deref(), Some("Cherry"));
    assert_eq!(custom.value().as_deref(), Some("Cherry"));
    selected.set(Some("gone"));
    screen.run_until_idle();
    assert_eq!(
        default.value().as_deref(),
        Some("Select…"),
        "a key that is not among the options shows the placeholder"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_labels_and_the_placeholder_follow_a_locale_switch() {
    let (screen, tag) = mount_with(real_window(), || {
        let tag = signal(Locale::new("en-US"));
        let messages = signal(Messages::new());
        let selected = signal(None::<&'static str>);
        let view = provide_locale(tag, move || {
            provide_messages(messages, move || {
                let options = (vec![
                    ListItem::new("yes", message("test.yes", "Yes")),
                    ListItem::new("no", message("test.no", "No")),
                ])
                .into_value();
                let answer = use_select(options, selected);
                column((
                    select_view(node(), "Answer", options, answer),
                    action("to-ja").on_press(move || {
                        tag.set(Locale::new("ja-JP"));
                        messages.set(
                            Messages::new()
                                .with("select.placeholder", "選択")
                                .with("test.yes", "はい")
                                .with("test.no", "いいえ"),
                        );
                    }),
                ))
            })
        });
        (selected, view)
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Answer");
    assert_eq!(trigger.value().as_deref(), Some("Select…"));
    tag.set(Some("no"));
    screen.run_until_idle();
    assert_eq!(trigger.value().as_deref(), Some("No"));
    trigger.press_key(Key::Enter);
    screen.get_by_role(Role::Option, "Yes");
    screen.press_key(Key::Escape);

    screen.get_by_role(Role::Button, "to-ja").click();
    assert_eq!(
        trigger.value().as_deref(),
        Some("いいえ"),
        "the label follows"
    );
    trigger.press_key(Key::Enter);
    screen.get_by_role(Role::Option, "はい");
    screen.get_by_role(Role::Option, "いいえ");
    screen.press_key(Key::Escape);
    tag.set(None);
    screen.run_until_idle();
    assert_eq!(
        trigger.value().as_deref(),
        Some("選択"),
        "so does the placeholder"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn a_selected_key_survives_options_that_leave_and_return() {
    let (screen, (options, selected)) = mount_with(real_window(), || {
        let options = signal(fruit());
        let selected = signal(Some("cherry"));
        let fruit = use_select(options, selected);
        (
            (options, selected),
            select_view(node(), "Fruit", options.into_value(), fruit),
        )
    });
    let trigger = trigger(&screen);
    options.update(|rows| rows.retain(|row| *row.key() != "cherry"));
    screen.run_until_idle();
    assert_eq!(selected.get_untracked(), Some("cherry"));
    assert_eq!(trigger.value().as_deref(), Some("Select…"));
    trigger.press_key(Key::Enter);
    assert!(
        option(&screen, "Apple").is_focused(),
        "no selected row: the first"
    );
    screen.press_key(Key::Escape);
    options.update(|rows| rows.push(ListItem::new("cherry", "Cherry")));
    screen.run_until_idle();
    assert_eq!(trigger.value().as_deref(), Some("Cherry"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn open_follows_the_list_opening_and_closing() {
    let (screen, open) = mount_with(real_window(), || {
        let options = (fruit()).into_value();
        let fruit = use_select(options, signal(None::<&'static str>));
        let open = fruit.shown();
        (open, select_view(node(), "Fruit", options, fruit))
    });
    let trigger = trigger(&screen);
    assert!(!open.get_untracked());
    trigger.press_key(Key::Enter);
    assert!(open.get_untracked(), "opening");
    screen.press_key(Key::Escape);
    assert!(!open.get_untracked(), "Escape closes");
    trigger.click();
    assert!(open.get_untracked(), "a click opens");
    option(&screen, "Cherry").click();
    assert!(!open.get_untracked(), "choosing closes");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

/// The selection and every `on_preview` call of a select.
type Preview = (
    Signal<Option<&'static str>>,
    Signal<Vec<Option<&'static str>>>,
);

/// A select that records every `on_preview` call.
fn mount_preview(initial: Option<&'static str>) -> (Screen, Preview) {
    mount_with(real_window(), move || {
        let selected = signal(initial);
        let seen = signal(Vec::<Option<&'static str>>::new());
        let options = (fruit()).into_value();
        let fruit = use_select(options, selected)
            .on_preview(move |key: Option<&'static str>| seen.update(|seen| seen.push(key)));
        let trigger = select_view(node().style(style().size(160, 28)), "Fruit", options, fruit);
        ((selected, seen), column((trigger, action("After"))))
    })
}

#[test]
fn a_preview_follows_the_focused_option_and_ends_when_the_list_closes() {
    let (screen, (selected, seen)) = mount_preview(None);
    assert!(seen.get_untracked().is_empty(), "nothing while closed");
    trigger(&screen).press_key(Key::Enter);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::End);
    screen.press_key(Key::Escape);
    assert_eq!(
        seen.get_untracked(),
        [
            Some("apple"),
            Some("apricot"),
            Some("blueberry"),
            Some("cherry"),
            None
        ],
        "the disabled option is skipped and Escape ends the preview"
    );
    assert_eq!(selected.get_untracked(), None, "a preview never selects");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn choosing_ends_the_preview_after_the_selection_is_written() {
    let (screen, (selected, seen)) = mount_preview(Some("apricot"));
    trigger(&screen).press_key(Key::Enter);
    assert_eq!(seen.get_untracked(), [Some("apricot")], "the selected one");
    screen.press_key(Key::ArrowDown);
    screen.press_key(Key::Enter);
    assert_eq!(selected.get_untracked(), Some("blueberry"));
    assert_eq!(
        seen.get_untracked(),
        [Some("apricot"), Some("blueberry"), None]
    );

    seen.set(Vec::new());
    trigger(&screen).click();
    option(&screen, "Cherry").click();
    assert_eq!(selected.get_untracked(), Some("cherry"));
    assert_eq!(
        seen.get_untracked().last(),
        Some(&None),
        "a click ends it too"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn the_pointer_previews_the_row_it_rests_on_over_the_focus() {
    let (screen, (selected, seen)) = mount_preview(None);
    trigger(&screen).press_key(Key::Enter);
    seen.set(Vec::new());
    option(&screen, "Cherry").hover();
    assert_eq!(seen.get_untracked(), [Some("cherry")]);
    option(&screen, "Apricot").hover();
    assert_eq!(seen.get_untracked(), [Some("cherry"), Some("apricot")]);
    assert!(
        option(&screen, "Apple").is_focused(),
        "hovering moves no focus"
    );
    assert_eq!(selected.get_untracked(), None);
    screen.press_key(Key::Escape);
    assert_eq!(seen.get_untracked().last(), Some(&None));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn reopening_focuses_the_current_selection_or_first_enabled_option() {
    let (screen, selected) = mount_select(fruit(), Some("apricot"));
    trigger(&screen).click();
    assert!(option(&screen, "Apricot").is_focused());
    screen.press_key(Key::End);
    assert!(option(&screen, "Cherry").is_focused());
    screen.press_key(Key::Escape);
    selected.set(None);
    trigger(&screen).click();
    assert!(option(&screen, "Apple").is_focused());
    screen.press_key(Key::End);
    screen.press_key(Key::Escape);
    selected.set(Some("blueberry"));
    trigger(&screen).click();
    assert!(option(&screen, "Blueberry").is_focused());
    assert!(screen.errors().is_empty());
}

#[test]
fn an_outside_press_closes_the_select_and_keeps_the_pressed_destination() {
    // "After" is placed below the list, so the press reaches it.
    let (screen, selected) = mount_with(real_window(), || {
        let selected = signal(Some("apple"));
        let options = fruit().into_value();
        let select = use_select(options, selected);
        let trigger = select_view(
            node().style(style().size(160, 28)),
            "Fruit",
            options,
            select,
        );
        (
            selected,
            column((trigger, action("After").style(style().margin_top(300)))),
        )
    });
    trigger(&screen).press_key(Key::Enter);
    let destination = screen.get_by_role(Role::Button, "After");
    destination.click();
    assert!(closed(&screen));
    assert!(destination.is_focused());
    assert!(!trigger(&screen).is_focused());
    assert_eq!(selected.get_untracked(), Some("apple"));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
