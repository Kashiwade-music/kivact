# Show lists, selectable lists and tables

[日本語](../../jp/how-to-guide/lists-and-tables.md)

Pick the builder that matches the job: `for_each` for rows that keep their own
state, `for_index` for plain values, `virtual_list` for thousands of rows,
`list` for a list the user picks from, and `data_table` for sortable,
filterable, selectable tables. This guide shows each one and how a row writes
back to the list.

## Choose a builder

| You have | Use | Row identity |
| --- | --- | --- |
| Records with an id, rows with their own state or controls | `for_each(items, key, row)` | the key you return |
| Plain values (numbers, strings) with no id | `for_index(items, row)` | the position |
| Thousands of rows, all alike in height | `virtual_list(items, key, row)` | the key; only visible rows exist |
| A list the user selects from | `list(name, items, selected)` | the item's key |
| Columns that sort, filter, page and select | `data_table(rows, key, columns)` | the row's key |

All of them take the items as a `Signal<Vec<T>>`, a `Value`, a memo or a closure
that returns a `Vec<T>`, and re-run when it changes. `T` must be
`Clone + PartialEq`, because Kivact compares old and new items to find out which
rows to update.

## A keyed list with row state

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

fn app() -> impl IntoView {
    let todos = signal(vec![
        Todo { id: 1, title: "Buy milk".into(), done: false },
        Todo { id: 2, title: "Write code".into(), done: false },
        Todo { id: 3, title: "Go for a walk".into(), done: false },
    ]);

    column(for_each(
        todos,
        |todo| todo.id,
        move |id, todo| {
            // Write the checkbox back to this row of the list.
            let done = todos.item(|t| t.id, id).field(|t| &t.done, |t| &mut t.done);
            // State owned by the row: it lives as long as the key `id` does.
            let open = signal(false);
            column((
                row((
                    checkbox(move || todo.get().title, done),
                    button("Details").on_press(move || open.update(|o| *o = !*o)),
                    button("Remove")
                        .on_press(move || todos.update(|list| list.retain(|t| t.id != id))),
                ))
                .style(style().gap(8)),
                show(open, move || text(format!("Todo number {id}"))),
            ))
        },
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Todos").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_state_follows_the_key_not_the_position() {
        let screen = kivact::test::render(app);
        screen.all_by_role(Role::Button, "Details")[0].click();
        screen.get_by_text("Todo number 1");

        // Removing the second row leaves the first row's state alone.
        screen.all_by_role(Role::Button, "Remove")[1].click();
        screen.get_by_text("Todo number 1");
        assert!(screen.query_by_role(Role::CheckBox, "Write code").is_none());

        screen.get_by_role(Role::CheckBox, "Buy milk").click();
        assert_eq!(
            screen.get_by_role(Role::CheckBox, "Buy milk").checked(),
            Some(CheckedState::Checked)
        );
    }
}
```

How it works:

- `for_each(items, key, row)` calls `key(&item)` to get each row's identity. A
  row keeps its owner, and every `signal` created inside the row closure, as
  long as its key stays in the list. Reordering moves the existing rows;
  removing an item disposes only its row.
- The row closure runs **once per row**, with the key and a `Value<Todo>` that
  always holds the row's current item. Read it inside a closure
  (`move || todo.get().title`) so the text follows edits; do not read it once
  and store the result.
- The key type needs `Clone + Eq + Hash`. Use a stable id, never the position,
  or rows would swap state when the list changes order.
- Two items with the same key are reported as a mistake; keep ids unique.

## Write back to the list from a row

A row holds a copy of its item, so it must say which item to change. Three ways,
from most to least convenient:

1. **A binding to one field.** `todos.item(|t| t.id, id)` is a `Binding<Todo>`
   for the item whose key is `id`, and `.field(get, get_mut)` narrows it to one
   field. Hand that binding to a control, as the checkbox above does. Make it in
   the row closure's setup, not inside a handler. If the item disappears the
   binding keeps the last value it saw, and writing to it does nothing.
2. **An update by key** in a handler: `todos.update(|list| list.retain(|t| t.id
   != id))` or `list.iter_mut().find(|t| t.id == id)`. Use it for remove, move
   and anything that changes the list itself.
3. **State owned by the row** with `signal(..)` inside the row closure, for
   things the list does not need, such as an expanded panel.

Never capture the row's *index* in a handler: after a removal it points at a
different item.

## Plain values: `for_index`

`for_index(items, row)` identifies rows by position. Row `i` keeps its owner
while the list has an item at `i`, and its `Value<T>` follows whatever item is
now there. Use it when items have no identity:

```rust,no_run
use kivact::prelude::*;

fn scores() -> impl IntoView {
    let scores = signal(vec![30_u32, 10, 20]);
    column((
        column(for_index(scores, |i, score| {
            row((
                text(format!("#{}", i + 1)),
                text(move || score.get().to_string()),
            ))
            .style(style().gap(8))
        })),
        button("Sort").on_press(move || scores.update(|s| s.sort())),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Scores").run(scores)
}
```

After "Sort" the three rows stay where they are and only their numbers change.
If rows hold state that should travel with the item (an open editor, a focused
input), use `for_each` with an id.

## Thousands of rows: `virtual_list`

`virtual_list` mounts only the rows near the viewport, so ten thousand entries
cost the same as a screenful. It needs a bounded height and rows of a known
height.

```rust,test_harness
use kivact::prelude::*;
use kivact::view::RowHeight;

#[derive(Clone, PartialEq)]
struct Entry {
    id: u32,
    message: String,
}

fn app() -> impl IntoView {
    let entries = signal(
        (0..10_000)
            .map(|id| Entry { id, message: format!("Entry {id}") })
            .collect::<Vec<_>>(),
    );
    let target = signal(None::<u32>);

    column((
        button("Jump to 5000").on_press(move || target.set(Some(5_000))),
        virtual_list(entries, |e| e.id, |_, entry| text(move || entry.get().message))
            .row_height(RowHeight::Fixed(24.0))
            .scroll_to(target)
            .style(style().height(300).border(1, ColorToken::Border)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Log").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_rows_near_the_viewport_exist() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Entry 0");
        assert!(screen.query_by_text("Entry 9999").is_none());

        screen.get_by_role(Role::Button, "Jump to 5000").click();
        screen.find_by_text("Entry 5000");
        assert!(screen.query_by_text("Entry 0").is_none());
    }
}
```

- `RowHeight::Fixed(px)` is the fast path. `RowHeight::Measured { estimate }`
  measures rows as they appear for rows of different heights, using `estimate`
  until a row is seen.
- `.overscan(px)` mounts extra pixels above and below the viewport (200 by
  default). `.keep_focused(true)` (the default) keeps the row that holds focus
  mounted when it scrolls away.
- `.scroll_to(signal)` scrolls the row named by a `Some(key)` into view each time
  the value becomes `Some`, even if it is not mounted yet.
- The rows are rebuilt as they scroll into view, so keep row state in the item
  or in a signal outside the list.
- A `virtual_list` without a bounded height would mount every row; Kivact
  reports a warning when that means 100 rows or more. Give it a height, or let a
  parent with a definite height size it.

## A list the user picks from

`list(name, items, selected)` shows `ListItem`s as a listbox and writes the
chosen key to a `Signal<Option<K>>`. The name is the accessible name of the
list and is not drawn.

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let selected = signal(Some(2_u32));
    let picked_many = signal(Vec::<u32>::new());

    column((
        list(
            "Colors",
            vec![
                ListItem::new(1_u32, "Red"),
                ListItem::new(2, "Blue").description("A calm color"),
                ListItem::new(3, "Green").disabled(true),
            ],
            selected,
        ),
        text(move || format!("selected: {:?}", selected.get())),
        list(
            "Files",
            vec![ListItem::new(1_u32, "notes.txt"), ListItem::new(2, "plan.md")],
            signal(None),
        )
        .multiple(picked_many)
        .item(|item, state| {
            text(move || {
                let mark = if state.get().selected { "[x] " } else { "[ ] " };
                format!("{mark}{}", item.get().label())
            })
        }),
        text(move || format!("{} files picked", picked_many.with(Vec::len))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Pick").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_selects_an_item_and_a_disabled_one_is_skipped() {
        let screen = kivact::test::render(app);
        screen.get_by_text("selected: Some(2)");
        screen.get_by_role(Role::Option, "Red").click();
        screen.get_by_text("selected: Some(1)");
        screen.get_by_text("0 files picked");
    }
}
```

- The arrow keys move the selection, Tab enters at the selected item, typing a
  name jumps to the item that starts with it, and disabled items are skipped.
- `.multiple(keys)` switches to a `Signal<Vec<K>>`: a press selects one item,
  Ctrl toggles one, and Shift extends a range. The single
  `selected` binding you passed to `list` is then ignored.
- `.item(|item, state| view)` replaces what a row shows. `item` is a
  `Value<ListItem<K>>` and `state` a `Value<ListItemState>` with `selected`, `active`
  and the hover and focus state of the row; behavior and keyboard stay the
  component's.
- The key `K` is your id (`u32`, an enum, a `String`), not the position, so the
  selection survives re-sorting.
- A checkbox inside a row (a business flag such as `done`) is separate from the
  row's selection. Keep them in different places.

## A table with sorting, filtering, paging and selection

`data_table(rows, key, columns)` takes the rows, a key function and a
`Vec<DataColumn<R>>`. The table keeps no copy of your data: the selection, the
sort, the filter text and the page are signals you own.

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Payment {
    id: u32,
    email: String,
    amount: f64,
}

fn app() -> impl IntoView {
    let payments = signal(vec![
        Payment { id: 1, email: "ann@example.com".into(), amount: 120.0 },
        Payment { id: 2, email: "bob@example.com".into(), amount: 80.5 },
        Payment { id: 3, email: "cy@example.com".into(), amount: 310.0 },
    ]);
    let selected = signal(Vec::<u32>::new());
    let query = signal(String::new());
    let page = signal(0_usize);

    data_table(
        payments,
        |p: &Payment| p.id,
        vec![
            DataColumn::new("Email", |p: &Payment| p.email.clone())
                .sortable()
                .filter(|p, query| p.email.contains(query)),
            DataColumn::new("Amount", |p: &Payment| format!("{:.2}", p.amount))
                .compare(|a, b| a.amount.total_cmp(&b.amount))
                .numeric(true)
                .width(120.0),
        ],
    )
    .row_label(|p| p.email.clone())
    .selection(selected)
    .filter(query)
    .page(page, 10)
    .style(style().padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Payments").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_shows_its_rows_without_errors() {
        let screen = kivact::test::render(app);
        screen.get_by_text("ann@example.com");
        screen.get_by_text("310.00");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

Each feature appears only when you give it a binding:

| Call | What you get |
| --- | --- |
| `.selection(Signal<Vec<K>>)` | check boxes in each row and the header, and a "selected" count |
| `.sort(Binding<Option<(usize, SortOrder)>>)` | read or set the sorted column (by position) and direction; without it the header still sorts by itself |
| `.filter(Signal<String>)` | a filter box that keeps rows whose `DataColumn::filter` matches |
| `.page(Signal<usize>, size)` | pages of `size` rows with Previous and Next |
| `.resizable(Signal<Vec<f64>>)` | resize handles on the header; the widths are yours |
| `.row_actions(width, \|row\| view)` | a last column of buttons or menus |
| `.toolbar(\|ctx\| view)`, `.footer(\|ctx\| view)` | replace the top or bottom area; `ctx` exposes the model counts and the bindings |
| `.row_details(\|row\| view)`, `.cell_controls(\|row, column\| view)` | views built once per row key, so inputs keep their state while the row sorts or pages |

A column sorts when it has a comparison (`.compare(..)` or `.sortable()`, which
compares the cell text) and joins the filter when it has `.filter(..)`.
`.cell(|row| view)` replaces a cell's view while its text stays the cell's
accessible name. `.numeric(true)` right-aligns, `.width(px)` fixes the width and
the other columns share the rest, and `.visible(..)` hides a column.

By default the keyboard moves a focus between cells and Space selects the row.
`.focus(TableFocus::Rows(ListPolicy::default()))` makes a whole row the focus stop instead. For a table
with many thousands of rows, combine `.page(..)` with data you already
filtered, because every row of the current page is mounted.

## Large data

- Keep the item type cheap to compare: `for_each` and `data_table` compare items
  with `PartialEq` on every change. Put large payloads behind an `Rc` or an id.
- Update one item with `update` or `item(..).field(..)` instead of replacing the
  whole `Vec`; only the changed rows are touched.
- Prefer `virtual_list` for long logs and feeds, and `data_table` with `.page(..)`
  for tabular data.
- Do the heavy filtering or sorting in a `memo` and give the memo to the list.

## See also

- [Layout](layout.md): size the scroll area that a list sits in.
- [Forms](forms.md): inputs inside rows and the draft pattern.
- [Run async work from a component](async-and-tasks.md): load rows without blocking.
- [Write reusable components](custom-components.md): build your own list or table look
  from the `kivact::interaction` parts.
- Reference: `cargo doc -p kivact-ui --open` for `List`, `DataTable` and
  `DataColumn`.
