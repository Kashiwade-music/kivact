# Step 3: Tabs and items

[日本語](../../../jp/game/getting-started/03-tabs-and-items.md)

In this step you add weapon and armor tabs. Each tab lists items; you pick one,
read its details and press **Equip**. The total power follows what is equipped.
At the end you write a test that plays through it.

## The program

Replace `src/main.rs` with the complete program, tests included:

```rust,test_harness
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::ui::Tone;
use kivact_ui_game::{self as game, Density, ThemeOptions as _};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Slot {
    Weapons,
    Armor,
}

#[derive(Clone, PartialEq)]
struct Item {
    id: u32,
    slot: Slot,
    name: &'static str,
    power: u32,
    equipped: bool,
}

fn item(id: u32, slot: Slot, name: &'static str, power: u32) -> Item {
    Item { id, slot, name, power, equipped: false }
}

fn loadout() -> impl IntoView {
    let items = signal(vec![
        item(1, Slot::Weapons, "Rail Rifle", 42),
        item(2, Slot::Weapons, "Pulse Blade", 35),
        item(3, Slot::Armor, "Plate Vest", 30),
        item(4, Slot::Armor, "Ghost Cloak", 18),
    ]);
    let slot = signal(Slot::Weapons);
    let selected = signal(None::<u32>);

    // The item being looked at, if it belongs to the open tab.
    let chosen = memo(move || {
        let id = selected.get()?;
        items.with(|all| all.iter().find(|i| i.id == id && i.slot == slot.get()).cloned())
    });
    let total = memo(move || {
        items.with(|all| all.iter().filter(|i| i.equipped).map(|i| i.power).sum::<u32>())
    });

    // One bar of the gauge for every ten points of power.
    let bars = memo(move || (total.get() / 10) as usize);

    let equip = move || {
        if let Some(chosen) = chosen.get() {
            items.update(|all| {
                for other in all.iter_mut().filter(|i| i.slot == chosen.slot) {
                    other.equipped = other.id == chosen.id;
                }
            });
        }
    };

    let panel = move |slot: Slot| {
        let rows = memo(move || {
            items.with(|all| {
                all.iter()
                    .filter(|i| i.slot == slot)
                    .map(|i| {
                        let mark = if i.equipped { " / EQUIPPED" } else { "" };
                        ListItem::new(i.id, i.name)
                            .description(format!("POWER {}{mark}", i.power))
                    })
                    .collect::<Vec<_>>()
            })
        });
        list("Items", rows, selected)
    };

    game::hud(
        column((
            game::section_heading("01", "LOADOUT", "Squad 04"),
            game::badge("READY", Tone::Success),
            tabs(
                "Slot",
                vec![
                    ListItem::new(Slot::Weapons, "WEAPONS"),
                    ListItem::new(Slot::Armor, "ARMOR"),
                ],
                slot,
            )
            .variant(game::Boxed)
            .content(panel),
            game::property_list(vec![
                (
                    "NAME".to_owned(),
                    game::mono(move || chosen.get().map_or("-".to_owned(), |i| i.name.to_owned())),
                ),
                (
                    "POWER".to_owned(),
                    game::mono(move || chosen.get().map_or("-".to_owned(), |i| i.power.to_string())),
                ),
            ]),
            row((
                button("Equip")
                    .variant(game::Slant)
                    .disabled(move || chosen.get().is_none())
                    .on_press(equip),
                game::mono(move || format!("TOTAL POWER {}", total.get())),
                game::gauge("Total power", bars).total(10).tone(Tone::Info),
            ))
            .style(style().gap(16).align_items(Align::Center)),
        ))
        .style(style().gap(16).padding(20)),
    )
    .style(style().width(420).margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Loadout")
        .theme(game::theme(ThemeMode::Dark).density(Density::Comfortable))
        .run(loadout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{RenderOptions, Screen, render_with};

    fn mount() -> Screen {
        let options = RenderOptions::default().window_size(800., 800.0).theme(game::theme(ThemeMode::Dark).density(Density::Comfortable));
        render_with(options, loadout)
    }

    #[test]
    fn equipping_a_weapon_adds_its_power() {
        let screen = mount();
        screen.get_by_text("TOTAL POWER 0");
        assert!(screen.get_by_role(Role::Button, "Equip").is_disabled());
        screen.get_by_role(Role::Option, "Rail Rifle").click();
        screen.get_by_role(Role::Button, "Equip").click();
        screen.get_by_text("TOTAL POWER 42");
        let gauge = screen.get_by_role(Role::Meter, "Total power");
        assert_eq!(gauge.value_range().2, Some(4.0));
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }

    #[test]
    fn a_second_weapon_replaces_the_first_and_armor_adds_up() {
        let screen = mount();
        screen.get_by_role(Role::Option, "Rail Rifle").click();
        screen.get_by_role(Role::Button, "Equip").click();
        screen.get_by_role(Role::Option, "Pulse Blade").click();
        screen.get_by_role(Role::Button, "Equip").click();
        screen.get_by_text("TOTAL POWER 35");

        screen.get_by_role(Role::Tab, "ARMOR").click();
        screen.get_by_role(Role::Option, "Plate Vest").click();
        screen.get_by_role(Role::Button, "Equip").click();
        screen.get_by_text("TOTAL POWER 65");
    }
}
```

Run `cargo run`. The WEAPONS tab is open with two items. Click **Rail Rifle**:
its name and power appear in the rows under the tabs and **Equip** becomes
available. Press it: the item's second line reads "POWER 42 / EQUIPPED", the
total becomes 42 and four of the ten bars of the gauge light up. Equip
**Pulse Blade** and it replaces the rifle. Switch to ARMOR, which has its own
items, and equip a vest: the total adds up. Inside a list, the arrow keys move
and select; on the tab row, the left and right arrow keys change tabs.

Run `cargo test` to play the same game through a test.

## How it works

**The app owns the items.** `items` is a `Signal<Vec<Item>>`, and `selected` is
a `Signal<Option<u32>>`. The list and the tabs show them and write the user's
choices back into the signals you pass; they keep no copy. When you equip an
item, you change `items`, and everything derived from it updates.

**`list(label, items, selected)`** is a standard component: a list of options
with keyboard movement, typeahead and selection. Its first argument names the
list for assistive technology and is not drawn. Each row is a
`ListItem::new(key, label)` with an optional `.description(..)`, and the key
(`u32` here) is what ends up in `selected`. The rows come from a `memo` so the
list rebuilds when `items` changes. The Game theme draws the rows with hard
edges and a selection edge.

**`tabs(label, tab_items, selected_key)`** is a standard component too. The
selected tab is always one of the tabs, so the signal holds the key itself
(`Slot`), not an `Option`. Any type that is `Clone + Eq + Hash` works as a key,
and an `enum` is a good fit. `.variant(game::Boxed)` is the Game kit's boxed
look; without it the tabs are underlined. `.content(|key| view)` builds the
panel for the open tab each time the tab changes.

**The gauge** from step 2 now shows the total power, one bar for every ten
points. `bars` is a `memo` derived from `total`, so the gauge follows
whatever you equip.

**`game::property_list(entries)`** shows label and value rows. Each value is a
view; to make one follow a signal, read the signal inside the view, as the two
`game::mono(move || ..)` calls do.

**The test** gives `RenderOptions` the same theme as the app and drives it by
roles and names: a list row is an `Option`, a tab is a `Tab`. These are the
words a screen reader announces, so the test also checks that the app is
usable that way.

## What you built

You have a game panel with a heading, meters, tabs, item lists and a passing
test. Next, take a look at the how-to guides, which cover the theme, lists,
meters and choice controls in more depth.

Next: [Game how-to guides](../how-to-guide/README.md)
