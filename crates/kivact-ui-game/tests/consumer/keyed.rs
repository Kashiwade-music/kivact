//! Role: Stable identity through the public styled game controls.

use super::*;
use kivact::ui::ListItem;
use kivact::view::{View, show};

fn choices() -> Vec<ListItem<String>> {
    vec![
        ListItem::new("department:operations".into(), "Operations"),
        ListItem::new("department:telemetry".into(), "Telemetry"),
        ListItem::new("department:locked".into(), "Locked").disabled(true),
    ]
}

/// A radio group's binding over the selection of tabs, which always hold one
/// key: the group shows it and choosing writes it.
fn radio(selected: Signal<String>) -> kivact::Binding<Option<String>> {
    selected.optional()
}

/// The departments with the telemetry one described by `description` and
/// shown with `icon`.
fn described(
    description: Signal<String>,
    label: &str,
    icon: &'static str,
) -> Vec<ListItem<String>> {
    let mut items = choices();
    items[1] = ListItem::new(items[1].key().clone(), label)
        .description(description)
        .icon(icon);
    items
}

/// The count after the telemetry option.
fn telemetry_count(
    counter: Signal<String>,
) -> impl Fn(Value<ListItem<String>>) -> View + Clone + 'static {
    move |item| {
        show(
            move || item.with(|item| item.key() == "department:telemetry"),
            move || text(counter),
        )
        .into_view()
    }
}

#[test]
fn rich_choices_keep_descriptions_icons_and_trailing_content_with_their_keys() {
    let (screen, (options, selected, description, counter)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let description = signal("Live sensors".to_owned());
            let counter = signal("12".to_owned());
            let options = signal(described(description, "Telemetry", "activity"));
            let selected = signal("department:telemetry".to_owned());
            let count = telemetry_count(counter);
            (
                (options, selected, description, counter),
                column((
                    kivact::ui::radio_group("Rich radios", options, radio(selected)),
                    kivact::ui::radio_group("Rich cards", options, radio(selected))
                        .variant(g::ChoiceCard)
                        .item({
                            let count = count.clone();
                            move |item, state| (g::card_label(item, state), count(item))
                        }),
                    kivact::ui::tabs("Tabs", options, selected)
                        .item({
                            let count = count.clone();
                            move |item, _| {
                                row((
                                    text(move || item.get().label()),
                                    text(move || {
                                        item.get().description_value().unwrap_or_default()
                                    }),
                                    count(item),
                                ))
                            }
                        })
                        .name("Rich tabs"),
                    kivact::ui::tabs("Tabs", options, selected)
                        .orientation(kivact::view::Orientation::Vertical)
                        .variant(g::Navigation)
                        .item(move |item, state| (g::navigation_label(item, state), count(item)))
                        .name("Rich navigation"),
                ))
                .style(style().width(560).padding(20).gap(20)),
            )
        });
    let radios = screen.get_by_role(Role::RadioGroup, "Rich radios");
    let cards = screen.get_by_role(Role::RadioGroup, "Rich cards");
    let tabs = screen.get_by_role(Role::TabList, "Rich tabs");
    let navigation = screen.get_by_role(Role::TabList, "Rich navigation");
    assert!(radios.get_by_text("Live sensors").exists());
    for group in [cards, tabs, navigation] {
        assert!(group.get_by_text("Live sensors").exists());
        assert!(group.get_by_text("12").exists());
    }
    tabs.get_by_role(Role::Tab, "Telemetry")
        .press_key(Key::ArrowLeft);
    selected.set("department:telemetry".to_owned());
    screen.run_until_idle();
    assert!(tabs.get_by_role(Role::Tab, "Operations").is_focused());
    let before_icon = screen.screenshot().hash();
    options.set(described(description, "Telemetry", "settings"));
    screen.run_until_idle();
    assert_ne!(screen.screenshot().hash(), before_icon);
    assert!(tabs.get_by_role(Role::Tab, "Operations").is_focused());
    description.set("Streaming sensors".to_owned());
    counter.set("27".to_owned());
    options.update(|items| {
        items[1] = ListItem::new(items[1].key().clone(), "Telemetry renamed")
            .description(description)
            .icon("settings");
        items.reverse();
    });
    screen.run_until_idle();
    assert!(radios.get_by_text("Streaming sensors").exists());
    for group in [cards, tabs, navigation] {
        assert!(group.get_by_text("Streaming sensors").exists());
        assert!(group.get_by_text("27").exists());
    }
    assert_eq!(
        radios
            .get_by_role(Role::RadioButton, "Telemetry renamed")
            .is_checked(),
        Some(true)
    );
    assert_eq!(
        cards
            .get_by_role(Role::RadioButton, "Telemetry renamed")
            .is_checked(),
        Some(true)
    );
    assert!(
        tabs.get_by_role(Role::Tab, "Telemetry renamed")
            .is_selected()
    );
    assert!(
        navigation
            .get_by_role(Role::Tab, "Telemetry renamed")
            .is_selected()
    );
    assert!(tabs.get_by_role(Role::Tab, "Operations").is_focused());
    options.update(|items| items.retain(|item| item.key() != "department:telemetry"));
    screen.run_until_idle();
    assert_eq!(selected.get_untracked(), "department:telemetry");
    assert!(screen.query_by_text("Streaming sensors").is_none());
    assert!(screen.query_by_text("27").is_none());
    options.update(|items| {
        items.push(
            ListItem::new("department:telemetry".to_owned(), "Telemetry restored")
                .description(description),
        );
    });
    screen.run_until_idle();
    for group in [cards, tabs, navigation] {
        assert!(group.get_by_text("Streaming sensors").exists());
        assert!(group.get_by_text("27").exists());
    }
    assert!(tabs.get_by_role(Role::Tab, "Operations").is_focused());
    navigation
        .get_by_role(Role::Tab, "Telemetry restored")
        .a11y(A11yAction::Activate);
    navigation
        .get_by_role(Role::Tab, "Telemetry restored")
        .press_key(Key::Home);
    assert_eq!(selected.get_untracked(), "department:operations");
    cards
        .get_by_role(Role::RadioButton, "Telemetry restored")
        .a11y(A11yAction::Select);
    assert_eq!(selected.get_untracked(), "department:telemetry");
    assert!(
        cards
            .get_by_role(Role::RadioButton, "Telemetry restored")
            .is_focused()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn game_select_bounds_a_long_list_and_scrolls_the_last_row_into_view() {
    let screen = kivact::test::render_with(crate::options(true), || {
        let rows: Vec<ListItem<u32>> = (0..30)
            .map(|i| ListItem::new(i, format!("Item {i:02}")))
            .collect();
        kivact::ui::select("Select", rows, signal(None::<u32>))
            .placeholder("")
            .name("Pick")
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Pick");
    trigger.press_key_with(
        Key::ArrowDown,
        kivact::view::Modifiers {
            alt: true,
            ..Default::default()
        },
    );
    assert_eq!(trigger.is_expanded(), Some(true), "Alt+Down opens");
    let list = screen
        .get_by_role(Role::ListBox, "Pick")
        .rect()
        .expect("laid out");
    assert!(list.height < 30.0 * 20.0, "the list is bounded: {list:?}");
    screen.press_key(Key::End);
    let last = screen.focused().expect("a row has the focus").rect();
    let last = last.expect("laid out");
    assert!(
        last.y + last.height <= list.y + list.height + 0.5,
        "End scrolls the last row into the list: {last:?} in {list:?}"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn game_select_flips_its_chevron_while_open_and_ends_the_row_with_the_check() {
    let (screen, ()) = kivact::test::render_with_handle(crate::options(true), || {
        (
            (),
            kivact::ui::select(
                "Select",
                vec![ListItem::new(1, "One"), ListItem::new(2, "Two")],
                signal(Some(1)),
            )
            .placeholder("")
            .name("Pick"),
        )
    });
    let trigger = screen.get_by_role(Role::ComboBox, "Pick");
    let rect = trigger.rect().expect("laid out");
    let indicator = |shot: kivact::test::Screenshot| {
        shot.crop(
            (rect.x + rect.width - 24.) as u32,
            (rect.y + (rect.height - 16.) / 2.) as u32,
            12,
            16,
        )
        .hash()
    };
    let closed = indicator(screen.screenshot());
    trigger.press_key(Key::Enter);
    assert_ne!(
        closed,
        indicator(screen.screenshot()),
        "opening flips the painted chevron"
    );
    let row = screen.get_by_role(Role::Option, "One");
    let row_rect = row.rect().expect("laid out");
    let label = row.get_by_text("One").rect().expect("laid out");
    let mark_x = row_rect.x + row_rect.width - 26.;
    assert!(mark_x >= label.x + label.width - 0.5, "check follows label");
    let shot = screen.screenshot();
    assert!(
        (0..14).any(|x| (0..20).any(|y| shot
            .pixel(mark_x as u32 + x, (row_rect.y + 6.) as u32 + y)
            == Some(0xffffff))),
        "selected row paints the trailing check"
    );
    screen.press_key(Key::Escape);
    assert_eq!(closed, indicator(screen.screenshot()));
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
