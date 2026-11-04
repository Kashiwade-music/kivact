# Build docks, panel groups and outliners

[日本語](../../../jp/creative/how-to-guide/panels-docks-and-outliners.md)

Goal: arrange your tool's panels in a dock the user can rearrange, show a hierarchy that can be renamed, reordered and filtered, group inspector sections and let the user search for commands.

Each of these components shows state that you own. The dock's layout, the outliner's items and the panel group's list are plain values in your signals. This guide assumes you finished the [tutorial](../getting-started/README.md), which builds a small dock and outliner step by step.

## Arrange panels in a dock

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

fn workspace() -> impl IntoView {
    let layout = signal(ui::DockLayout::new(ui::DockNode::split(
        ui::DockAxis::Horizontal,
        [
            (
                0.25,
                ui::DockNode::tabs(vec![ui::DockTab::new("assets", "Assets").icon("folder")]),
            ),
            (
                0.75,
                ui::DockNode::split(
                    ui::DockAxis::Vertical,
                    [
                        (
                            0.7,
                            ui::DockNode::tabs(vec![
                                ui::DockTab::new("viewport", "Viewport").closable(false),
                                ui::DockTab::new("preview", "Preview"),
                            ]),
                        ),
                        (
                            0.3,
                            ui::DockNode::tabs(vec![ui::DockTab::new("console", "Console")]),
                        ),
                    ],
                ),
            ),
        ],
    )));

    ui::dock(layout, |panel: &'static str| match panel {
        "assets" => text("Assets").into_view(),
        "viewport" => text("Viewport").into_view(),
        "preview" => text("Preview").into_view(),
        "console" => text("Console").into_view(),
        _ => node().into_view(),
    })
    .style(style().width(pct(100)).height(pct(100)))
}

fn main() -> kivact::Result<()> {
    App::new("Dock")
        .theme(ui::theme(ThemeMode::Dark))
        .run(workspace)
}
```

The user can switch tabs, drag a tab into another container, beside one (splitting it) or onto an edge of the whole dock, resize with the dividers, rename a panel by double-clicking its title (or pressing `F2`), maximize it by double-clicking its icon, and use each container's menu (**Close Panel**, **Maximize Panel**, **Split Right** and so on).

### How the dock works

- **The layout is a value you own.** `DockLayout<K>` is a tree: a `DockNode::Split` holds weighted children along an axis, and a `DockNode::Tabs` is a container of `DockTab`s. `K` is the key type you choose for panels; it must be `Clone + Eq + Hash`. A `&'static str` is the simplest; an `enum` is better for a larger tool. Every change the user makes is written back to your signal as a complete, normalized layout, so you can store it, restore it and undo it. A splitter drag writes the layout on every pointer move.
- **Panels are created once per key.** The closure you pass to `dock` is called once for each key and the resulting view stays mounted when the tab is hidden, moved or the tree is restructured. A panel therefore keeps its internal state while the user rearranges things. Closing a tab disposes its view; opening the key again builds a new one.
- **A key identifies a panel exactly once.** Later duplicates are dropped when the layout is normalized.
- **Give the dock a size.** Use `.style(style().height(600))` or percentages, as above.

### Add, move, close and maximize panels from your code

`DockLayout` has plain methods that return whether they succeeded. Call them inside `layout.update(..)`:

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn commands(layout: Signal<ui::DockLayout<&'static str>>) {
    // Open a panel in a new container on the right edge.
    layout.update(|l| {
        l.insert(
            ui::DockTab::new("history", "History").icon("clock"),
            ui::DockPlacement::Edge(ui::DockSide::Right),
        )
    });
    // Put it next to the preview, as a tab.
    layout.update(|l| {
        l.move_tab(
            &"history",
            ui::DockPlacement::Tabs { container: "preview", index: None },
        )
    });
    layout.update(|l| l.maximize(&"viewport"));
    layout.update(|l| l.restore());
    layout.update(|l| l.close(&"history").is_some());
}
```

`DockPlacement` names the target container by any tab it contains, so a placement stays valid while the tree changes. `insert` refuses a key that exists already; to "show" a panel that might be open, check `layout.get().tab(&key)` first and call `activate` instead.

### React to closing and add menu commands

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn dock_with_hooks(layout: Signal<ui::DockLayout<&'static str>>) -> impl IntoView {
    ui::dock(layout, |panel: &'static str| text(panel))
        // After a tab is closed (the layout is already updated): release its resources.
        .on_close(move |key| {
            let _ = key;
        })
        // Your commands first, then a separator and the built-in ones.
        .menu(
            |_container| vec![MenuEntry::item("duplicate", "Duplicate Panel")],
            move |(key, command): (&'static str, &'static str)| {
                if command == "duplicate" {
                    layout.update(|l| {
                        l.insert(
                            ui::DockTab::new("copy", format!("{key} copy")),
                            ui::DockPlacement::Tabs { container: key, index: None },
                        )
                    });
                }
            },
        )
        // Splitter drags report Begin, Commit and Cancel, like numeric fields.
        .on_edit(|_phase| {})
}
```

`menu`'s first closure receives the container the menu belongs to and returns your entries; the second receives that container's active tab key and the key of the chosen entry.

## Show a hierarchy with an outliner

`ui::outliner(items, state)` shows a list of `TreeItem`s. Each item names its parent, and siblings appear in the order of the items. The outliner supports keyboard navigation (arrows, `Home`, `End`, `F2` to rename), range and additive selection with `Shift` and `Ctrl`, filtering, renaming, drag-and-drop reparenting and per-item visibility, selectability and render toggles.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

/// Move an item as `drop` says: under `drop.target`, right after `drop.after`.
fn apply_drop(items: &mut Vec<ui::TreeItem>, drop: ui::TreeDrop) {
    let Some(from) = items.iter().position(|item| item.id == drop.id) else {
        return;
    };
    let mut moved = items.remove(from);
    moved.parent = drop.target;
    let at = match drop.after {
        Some(after) => items
            .iter()
            .position(|item| item.id == after)
            .map_or(items.len(), |index| index + 1),
        None => items
            .iter()
            .position(|item| item.parent == drop.target)
            .unwrap_or(items.len()),
    };
    items.insert(at, moved);
}

fn scene_outliner() -> impl IntoView {
    let items = signal(vec![
        ui::TreeItem::new(1, None, "Scene", "folder"),
        ui::TreeItem::new(2, Some(1), "Camera", "camera"),
        ui::TreeItem::new(3, Some(1), "Lights", "folder"),
        ui::TreeItem::new(4, Some(3), "Key light", "bulb"),
    ]);
    let state = ui::OutlinerState::new([1, 3]);
    let filter = signal(String::new());
    let snapshot = signal(None::<Vec<ui::TreeItem>>);

    column((
        kivact::ui::text_input("Filter", filter),
        ui::outliner(items, state)
            .filter(filter)
            .on_rename(move |(id, name): (u64, String)| {
                items.update(|all| {
                    if let Some(item) = all.iter_mut().find(|item| item.id == id) {
                        item.label = name;
                    }
                });
            })
            .on_item_drop(move |drop| items.update(|all| apply_drop(all, drop)))
            .on_restriction(move |event| match event {
                ui::EditorEvent::Begin => snapshot.set(Some(items.get_untracked())),
                ui::EditorEvent::Request(change) => items.update(|all| {
                    for item in all.iter_mut().filter(|item| change.ids.contains(&item.id)) {
                        match change.flag {
                            ui::RestrictionFlag::Visible => item.visible = change.value,
                            ui::RestrictionFlag::Selectable => item.selectable = change.value,
                            ui::RestrictionFlag::Rendered => item.rendered = change.value,
                        }
                    }
                }),
                ui::EditorEvent::Commit => snapshot.set(None),
                ui::EditorEvent::Cancel => {
                    if let Some(before) = snapshot.update(|saved| saved.take()) {
                        items.set(before);
                    }
                }
            }),
    ))
    .style(style().gap(6).padding(8).width(280))
}

fn main() -> kivact::Result<()> {
    App::new("Outliner")
        .theme(ui::theme(ThemeMode::Dark))
        .run(scene_outliner)
}
```

- **Read-only items, requests out.** Like every editor of the kit, the outliner never changes `items`. A rename arrives through `on_rename` as `(id, new_name)`, a drag-and-drop through `on_item_drop` as a `TreeDrop { id, target, after }`, and a toggle of an item's visibility, selectability or render flag through `on_restriction`. You apply them, and the outliner shows the result.
- **Dragging is off without `on_item_drop`.** The pointer's vertical position over a row decides the place: the upper third inserts before it as a sibling, the lower third after it, the middle third inside it as its last child. An item cannot be dropped on itself or into its own subtree. `Esc` cancels a drag.
- **Restriction paints.** Pressing a visibility toggle and dragging across rows sets many rows to the same value in one gesture: a `Begin`, requests carrying all the ids touched so far with one `value`, and a `Commit` or `Cancel`. If several selected rows are involved, they all take the opposite of the pressed row's flag. Hide the toggles with `.restrictions(false)`.
- **`OutlinerState`** bundles `expanded` (open ids), `selection` and `renaming`. Replace any of them with your own signal to share it, as the tutorial does with the selection.
- **More row options.** `.active(id)` marks one item as current (separate from the selection) and `reveal_active` opens its ancestors; `.on_item_activate(f)` runs on `Enter` or a double click (a double click then no longer renames; `F2` does). `.cell(width, render)` adds a fixed-width cell of your own controls to every row. `TreeItem::tone(..)` and `placeholder(true)` change the label's look.

## Group inspector sections

`ui::panel_group(panels, content)` shows collapsible sections that the user can reorder by dragging their grips. The list of `InspectorPanel`s is your signal; `content` builds the body of each section.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn sections() -> impl IntoView {
    let panels = signal(vec![
        ui::InspectorPanel::new(1, "Transform"),
        ui::InspectorPanel::new(2, "Appearance").collapsed(),
    ]);
    let position = signal(0.0);
    let opacity = signal(1.0);

    ui::panel_group(panels, move |panel: Value<ui::InspectorPanel>| {
        match panel.get_untracked().id {
            1 => ui::property_row("X", ui::number("X", position).label_visible(false)).into_view(),
            _ => ui::property_row(
                "Opacity",
                ui::number("Opacity", opacity).range(kivact::interaction::ScalarRange::new(0.0, 1.0).expect("valid range")).label_visible(false),
            )
            .into_view(),
        }
    })
}
```

The bodies stay mounted while a section is collapsed or reordered, so a field keeps its focus and state. `Ctrl` (or `Cmd`) plus a click on a heading opens only that section and collapses the others. Inside a section, `ui::property_row(label, control)` lines up labels in a column; give it `.leading(..)` and `.trailing(..)` slots for a state button or an action. For one titled box without grouping, use `ui::panel(title, content)`; for two resizable panes, the standard `kivact::ui::split(sizes, (first, second))` is drawn like the dock's dividers.

## Let the user search for a command or an item

`ui::search_picker(items)` is a searchable list in a popup: type to filter, `Up` and `Down` to move, `Enter` or a click to pick, `Esc` to cancel. The focus stays in the search field.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn add_node_button() -> impl IntoView {
    let last = signal(String::from("nothing yet"));
    let picker = ui::search_picker(vec![
        ui::PickerItem::new("blur", "Blur").section("Filter").shortcut("Shift B"),
        ui::PickerItem::new("noise", "Noise")
            .section("Generate")
            .description("Fractal noise")
            .icon("wave"),
        ui::PickerItem::new("solid", "Solid").section("Generate").enabled(false),
    ])
    .on_pick(move |key: &'static str| last.set(key.to_string()))
    .on_preview(|_key: Option<&'static str>| {
        // `Some(key)` while a row is active, `None` when the picker closes.
    });
    row((
        picker.attach(button("Add node…")),
        text(move || format!("Picked: {}", last.get())),
    ))
    .style(style().gap(8))
}
```

`attach(button)` opens the picker below the button; `.at(anchor, position)` opens it at a window point instead, for a shortcut-triggered picker. The key type (here `&'static str`) is yours, and the picker hands back the key of the picked item. Several `.column(title, items)` calls make a multi-column picker with a shared search.

Radial menus work similarly: `ui::pie_menu(entries, state, choose)` shows `PieEntry`s around a point, and its public `PieState` lets you open it from your own shortcut (open on key down, commit on key up).

## See also

- [Tutorial step 3: Put it in a dock](../getting-started/03-dock-and-outliner.md)
- [Theme the Creative kit, set fonts and add icons](theme-and-icons.md): icon names for tabs and rows, and translating the dock's menu texts.
- [Edit a timeline, dope sheet, graph or arrangement](timeline-and-arrangement.md): the same request and snapshot pattern.
- Full API: `cargo doc -p kivact-ui-creative --open`
- The catalog's layout, outliner and picker chapters: `cargo run -p kivact-ui-creative-catalog`
