# Show items in lists and trees

[日本語](../../../jp/game/how-to-guide/item-lists-and-selection.md)

Use this guide to show items the player picks from: an inventory, a list of
missions, a file tree or a table of units. The rule is the same everywhere: the
**app owns the items and the selection** as signals. A component draws them and
writes the player's choices back; it keeps no copy.

## Pick one item from a list

`list(label, items, selected)` is a standard component. The Game theme draws
it with hard edges and a selection edge.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game as game;

fn inventory() -> impl IntoView {
    let selected = signal(Some(2_u32));
    let items = vec![
        ListItem::new(1, "Rail Rifle").description("POWER 42").icon("bolt"),
        ListItem::new(2, "Pulse Blade").description("POWER 35"),
        ListItem::new(3, "Prototype Cannon").description("LOCKED").disabled(true),
    ];
    column((
        list("Inventory", items, selected),
        game::mono(move || format!("SELECTED {:?}", selected.get())),
    ))
    .style(style().gap(12).width(320).margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Inventory")
        .theme(game::theme(ThemeMode::Dark))
        .run(inventory)
}
```

- The first argument names the list for assistive technology. It is not drawn,
  so draw a heading yourself.
- A `ListItem` has a **key** (here a `u32`), a label, and optionally a
  description, an icon name from the kit and `disabled(true)`. The key can be
  any type that is `Clone + Eq + Hash` (an id, a string, your own `enum`); it is
  what ends up in the selection signal.
- The selection is a `Signal<Option<K>>`; `None` means nothing is picked.
  Writing the signal from your code moves the selection too.
- Keyboard: arrows move and select, Home and End jump, typing a name jumps to
  the item that starts with it, and a disabled item is skipped. Tab enters the
  list at the selected item.

## Build the items from your data

Keep your own item type in a signal and derive the `ListItem`s with a `memo`.
The list rebuilds whenever the data changes, and your type can carry anything:

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Weapon {
    id: u32,
    name: String,
    ammo: u32,
}

fn weapons(all: Signal<Vec<Weapon>>, selected: Signal<Option<u32>>) -> impl IntoView {
    let rows = memo(move || {
        all.with(|all| {
            all.iter()
                .map(|w| ListItem::new(w.id, w.name.clone()).description(format!("AMMO {}", w.ammo)))
                .collect::<Vec<_>>()
        })
    });
    list("Weapons", rows, selected)
}
# fn main() {}
```

When you remove an item, clear the selection if it pointed at that item; the
list never edits your signals except to write the player's choice.

## Pick several items

`.multiple(signal)` turns the list into a multi-select. The list then reads and
writes only that signal (a `Signal<Vec<K>>`); the single `selected` argument is
unused, so pass any signal. A click selects one, Ctrl (Cmd on macOS) toggles
one, and Shift extends from the last one.

```rust,no_run
use kivact::prelude::*;

fn squad() -> impl IntoView {
    let picked = signal(vec![1_u32]);
    list(
        "Squad",
        vec![
            ListItem::new(1, "Aya"),
            ListItem::new(2, "Ben"),
            ListItem::new(3, "Cho"),
        ],
        signal(None::<u32>),
    )
    .multiple(picked)
}
# fn main() {}
```

## Draw each row yourself

`.item(|item, state| view)` replaces what a row shows but keeps its keys,
keyboard behavior and selection. `item` is a `Value<ListItem<K>>` and `state`
says whether the row is selected or active:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn missions() -> impl IntoView {
    let selected = signal(Some("alpha"));
    list(
        "Missions",
        vec![ListItem::new("alpha", "Alpha"), ListItem::new("bravo", "Bravo")],
        selected,
    )
    .item(|item, state| {
        row((
            game::mono(move || if state.get().selected { ">" } else { " " }),
            text(move || item.with(|item| item.label())),
        ))
        .style(style().gap(8))
    })
}
# fn main() {}
```

Read the signals inside the closures (`state.get()`, `item.with(..)`), so the
row updates when they change.

## Show a hierarchy as a tree

`game::tree(entries, selection)` draws a file tree. `entries` is a list of
`game::TreeEntry`, built with `TreeEntry::file(id, label)` and
`TreeEntry::folder(id, label, children)`; `selection` is a
`Signal<Option<String>>` holding the id of the selected **leaf**, `None` while
none is selected.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn files() -> impl IntoView {
    let selected = signal(Some(String::from("save-1")));
    let entries = vec![
        game::TreeEntry::folder(
            "saves",
            "saves",
            vec![
                game::TreeEntry::file("save-1", "slot-1.sav"),
                game::TreeEntry::file("save-2", "slot-2.sav").marker("M"),
            ],
        ),
        game::TreeEntry::folder("mods", "mods", vec![]).collapsed(),
        game::TreeEntry::file("readme", "README.txt"),
    ];
    column((
        game::tree(entries, selected),
        game::mono(move || format!("OPEN {}", selected.get().unwrap_or_default())),
    ))
    .style(style().gap(12).width(300).margin(24))
}
# fn main() {}
```

The whole tree is one tab stop. Up and Down move over visible rows, Right and
Left open and close a folder or move between a folder and its children. Only a
leaf is written to the signal; a folder opens and closes with its chevron,
Enter or a double click. `.collapsed()` starts a folder closed, `.marker("M")`
adds a small mark at the end of a row. The entries are read when the tree is
built and also follow a signal, so a renamed entry shows the new label.

## Show records in a table

`data_table(rows, key, columns)` is the standard table; the Game theme draws it
as a console table. `game::TableCell` gives a column's cells a console look
through `.cell(..)`:

```rust,no_run
use kivact::prelude::*;
use kivact::ui::Tone;
use kivact_ui_game as game;

#[derive(Clone, PartialEq)]
struct Unit {
    id: u32,
    name: &'static str,
    status: &'static str,
    hp: f64,
}

fn units(rows: Signal<Vec<Unit>>) -> impl IntoView {
    let selected = signal(Vec::<u32>::new());
    data_table(
        rows,
        |unit: &Unit| unit.id,
        vec![
            DataColumn::new("Unit", |unit: &Unit| unit.name.to_owned()).sortable(),
            DataColumn::new("Status", |unit: &Unit| unit.status.to_owned()).cell(|unit: &Unit| {
                game::TableCell::Badge {
                    label: unit.status.to_owned(),
                    tone: if unit.status == "ALIVE" { Tone::Success } else { Tone::Danger },
                }
            }),
            DataColumn::new("HP", |unit: &Unit| format!("{:.0}", unit.hp))
                .compare(|a, b| a.hp.total_cmp(&b.hp))
                .cell(|unit: &Unit| game::TableCell::Progress { value: unit.hp, tone: Tone::Info })
                .numeric(true),
        ],
    )
    .selection(selected)
}
# fn main() {}
```

The table keeps nothing but what you pass: the rows, the selection, and (with
`.sort(..)`, `.filter(..)`, `.page(..)`) the sort, the search text and the page.
`.toolbar(|context| view)` replaces the area above the table, for example with
bulk actions on the selected rows, and `.footer(|context| view)` the area
below. Cells: `TableCell::Identifier`, `Detail { title, description }`,
`User { name, initials, color }`, `Badge { label, tone }` and
`Progress { value, tone }` (0 to 100).

## See also

- [Show meters and status](meters-and-status.md)
- [Offer choices and quick actions](choices-and-toolbars.md)
- The tabs in the [tutorial](../getting-started/03-tabs-and-items.md), which
  put a list inside each tab
- Component API: `cargo doc -p kivact-ui-game --open`, and `kivact::ui::List`,
  `DataTable` in `cargo doc -p kivact --open`
