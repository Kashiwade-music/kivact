# Build controls with your own look

[日本語](../../jp/how-to-guide/custom-controls.md)

Use this guide when the standard components do not look the way you need, such as a
pill switch, a segmented control, a custom dropdown, a draggable value or a rating
bar, but you still want what makes a control feel right: keyboard navigation, focus,
selection, screen-reader roles and the correct handling of touch and pointer. It
assumes you have read [Write reusable components](custom-components.md).

The idea is a split. A **controller** from `kivact::interaction` owns the behavior
and the state. It draws nothing. A **Part** it hands you carries the input, focus
and accessibility semantics. You attach the Part to a node you style yourself, and
you draw the node from the state the Part reports. Applications use `kivact::ui` or
a kit; the controllers are for the people who write controls and kits, and the
official kits are built from the same ones.

| You need | Controller | Parts it hands out |
|---|---|---|
| a button | `use_button` | `control()` |
| a check box, switch or toggle button | `use_checkbox`, `use_switch`, `use_toggle_button` | `control()` |
| one or several choices from a list | `use_list` | `list()`, `item(key)` |
| a dropdown that opens a list | `use_select` | `trigger()`, `content()`, `item(key)` |
| a menu | `use_menu` | `trigger()`, `context_trigger()`, `content()`, `item(key)` |
| a menu bar | `use_menubar` | `bar()`, `item(at)` |
| a search box with suggestions | `use_combobox` | `input()`, `content()`, `item(key)` |
| a command palette | `use_command_palette` | `content()`, `input()`, `list()`, `item(key)` |
| an open/close body, a popover, a tooltip | `use_disclosure`, `use_popup`, `use_tooltip` | `trigger()`, `content()` |
| a hover card, a delayed hint, a flashing confirmation | `use_delayed` | `is_open()`, `dismiss()`, `pulse()` |
| a dialog | `use_dialog` | `trigger()`, `content()`, `title()`, `description()`, `close_trigger()` |
| tabs | `use_tabs` | `list()`, `item(key)`, `panel(key)` |
| a tree | `use_tree` | `list()`, `item(key)` |
| a table | `use_data_table` | `grid()`, `header(column)`, `row(key)`, `cell(key, column)` |
| a slider, a number field | `use_scalar`, `use_range_scalar` | `thumb()`, `track(axis)`, `input()` |
| a resizable divider | `use_splitter` | `handle()` |
| a drag, zoom and pan | `use_drag`, `use_zoom_pan`, `use_space_pan` | `source(..)`, `viewport()`, `control()` |
| native text editing | `use_text_edit`, `use_text_area` | `control()` |

Controllers whose name is not listed here, and the details of each, are in
`cargo doc -p kivact-interaction --open`.

## The pattern: controller, Part, node

The shortest custom control is a switch. Everything it does comes from the
controller; the code only decides what it looks like.

```rust,test_harness
use kivact::state::Binding;
use kivact::interaction::use_switch;
use kivact::prelude::*;
use kivact::style::Direction;

/// A pill with a knob that slides to the right when on.
pub fn pill_switch(label: impl IntoText, on: impl IntoBinding<bool>) -> impl Component {
    let api = use_switch(on);
    let state = api.control_state();
    let control = api.control();
    node()
        .part(control)
        .name(label)
        .style(move || {
            let state = state.get();
            let on = state.checked.is_checked();
            style()
                .direction(Direction::Row)
                .justify_content(if on { Justify::End } else { Justify::Start })
                .size(44, 24)
                .padding(3)
                .radius(12)
                .background(if on { ColorToken::Accent } else { ColorToken::Border })
                .opacity(if state.input.pressed { 0.8 } else { 1.0 })
        })
        .child(node().style(style().size(18, 18).radius(9).background(ColorToken::Surface)))
}

fn app() -> impl IntoView {
    let notifications = signal(false);
    column((
        pill_switch("Notifications", notifications),
        text(move || if notifications.get() { "On" } else { "Off" }.to_owned()),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Switch").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_part_brings_the_role_the_keys_and_the_state() {
        let screen = kivact::test::render(app);
        let switch = screen.get_by_role(Role::Switch, "Notifications");
        assert_eq!(switch.checked(), Some(CheckedState::Unchecked));

        switch.click();
        assert_eq!(switch.checked(), Some(CheckedState::Checked));
        screen.get_by_text("On");

        switch.press_key(Key::Space);
        assert_eq!(switch.checked(), Some(CheckedState::Unchecked));
    }

    #[test]
    fn a_disabled_switch_ignores_input() {
        let screen = kivact::test::render(|| pill_switch("Locked", signal(false)).disabled(true));
        let switch = screen.get_by_role(Role::Switch, "Locked");
        assert!(switch.is_disabled());
        switch.force_click();
        assert_eq!(switch.checked(), Some(CheckedState::Unchecked));
    }
}
```

Four things happened without you writing them:

- **Role and state.** `use_switch(..).control()` sets `Role::Switch`, the checked
  state and the focus stop. A screen reader announces "Notifications, switch, off".
- **Input.** Click, Space and the screen-reader actions all flip the same
  `Binding`; a switch ignores Enter. The control writes the value to the binding
  you gave it, so the state stays with you.
- **Disabled.** `pill_switch(..).disabled(true)` works: a disabled node ignores
  input and the state reports `disabled`. The Part reads the final disabled state from its host during mounting,
  so one modifier disables both the input and its appearance.
- **State for drawing.** `api.control_state()` is a `Value` that follows the real
  target: pressed, hovered, focused, focus-visible, disabled and `checked`. Draw
  from it. Do not keep a second signal that tries to mimic it.

Three rules are common to every controller:

1. **Create the controller during setup** and call it once per control. It is a
   `Copy` handle.
2. **Read drawing state from the controller.** `api.control_state()` follows the
   actual attached node. Use an explicit `node_ref()` and `.node_ref(anchor)`
   when you need a reference; the Part only attaches behavior.
3. **Do not repeat what the controller does.** If you add your own `on_key` for
   Space, the control flips twice. Your `on_press`, `on_key`, `on_pointer` handlers
   run **before** the Part's defaults; return `Handled::PreventDefault` to run
   yours and skip the default, or `Handled::Yes` to stop everything.

## Choose from a list with keyboard navigation

A segmented control is a row of options where one is selected. `use_list` provides
the selection, the arrow keys, Home and End, the roving tab stop and the screen-reader
list semantics; you provide the keys, the order and the looks.

```rust,test_harness
use std::hash::Hash;

use kivact::state::Binding;
use kivact::interaction::{ListItem, ListOptions, Selection, use_list};
use kivact::prelude::*;

/// A row of options; the arrow keys move and select, Tab leaves the row.
pub fn segmented<K: ItemKey>(
    label: impl IntoText,
    options: Vec<ListItem<K>>,
    selected: impl IntoBinding<Option<K>>,
) -> impl Component {
    let options = options.into_value();
    let list = use_list(
        options,
        selected.into_binding(),
        ListOptions {
            axis: Some(Orientation::Horizontal),
            ..ListOptions::default()
        },
    );
    let rows = for_each(
        options,
        |option| option.key().clone(),
        move |option| {
            let key = option.with_untracked(|option| option.key().clone());
            let state = list.item_state(key.clone());
            let part = list.item(key);
            node()
                .part(part)
                .style(move || {
                    let state = state.get();
                    style()
                        .padding_x(12)
                        .padding_y(6)
                        .background(if state.selected { ColorToken::Accent } else { ColorToken::SurfaceAlt })
                        .color(if state.selected { ColorToken::AccentText } else { ColorToken::Text })
                        .opacity(if state.input.disabled { 0.4 } else { 1.0 })
                })
                .child(text(move || option.with(|option| option.label())))
        },
    );
    row(rows).part(list.list()).name(label)
}

fn app() -> impl IntoView {
    let range = signal(Some("week"));
    column((
        segmented(
            "Range",
            vec![
                ListItem::new("day", "Day"),
                ListItem::new("week", "Week"),
                ListItem::new("month", "Month"),
                ListItem::new("year", "Year").disabled(true),
            ],
            range,
        ),
        text(move || format!("range: {}", range.get().unwrap_or("none"))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Segmented").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_and_arrow_keys_move_the_selection_and_skip_disabled_items() {
        let screen = kivact::test::render(app);
        assert!(screen.get_by_role(Role::Option, "Week").is_selected());

        screen.get_by_role(Role::Option, "Month").click();
        screen.get_by_text("range: month");

        // Year is disabled: the arrow key stops at Month.
        screen.get_by_role(Role::Option, "Month").press_key(Key::ArrowRight);
        screen.get_by_text("range: month");
        screen.get_by_role(Role::Option, "Month").press_key(Key::ArrowLeft);
        screen.get_by_text("range: week");
    }
}
```

What the list gives you:

- **Roles.** The container is a `ListBox` (multi-selectable for
  `Selection::Multiple`); each item is an `Option` named by its label and
  `selected` when it is part of the selection.
- **Selection is yours.** The list never keeps a copy. It writes the user's choice to the
  binding you passed (`Selection::Single` over `Binding<Option<K>>`, or
  `Selection::Multiple` over `Binding<Vec<K>>` with Shift and Ctrl ranges), and a
  selected key that a filter hides is not cleared.
- **Navigation.** `ListOptions` picks the axis, wrapping at the ends, whether moving
  selects, typeahead and whether the choice happens on activation. The presets
  `ListOptions::tabs()`, `radio()`, `menu()` and `listbox()` cover the common
  widgets. The policy is the third argument of `use_list`.
- **Only one tab stop.** The row is one stop of Tab, and the arrow keys move inside.
  Disabled items (`ListItem::disabled(true)`) are skipped.

## Open and close a list

A dropdown adds opening, closing, focus return and dismissal on top of a list.
`use_select` hands out three Parts: the **trigger** (a combo box), the **content**
(the floating surface) and an **item** per option. You mount the content with `show`
and `overlay` while `is_open()` is true.

```rust,test_harness
use std::hash::Hash;

use kivact::state::Binding;
use kivact::interaction::{ListItem, use_select};
use kivact::prelude::*;
use kivact::view::{Placement, overlay};

pub fn dropdown<K: ItemKey>(
    label: impl IntoText,
    options: Vec<ListItem<K>>,
    value: impl IntoBinding<Option<K>>,
) -> impl IntoView {
    let label = label.into_text_value();
    let options = options.into_value();
    let select = use_select(options, value);
    let trigger = select.trigger();
    let anchor = node_ref();
    (
        node()
            .part(trigger)
            .node_ref(anchor)
            .name(label)
            .style(
                style()
                    .padding(8)
                    .border(1, ColorToken::Border)
                    .radius(6)
                    .min_width(120),
            )
            .child(text(select.label())),
        show(select.is_open(), move || {
            let rows = for_each(
                options,
                |option| option.key().clone(),
                move |option| {
            let key = option.with_untracked(|option| option.key().clone());
                    let state = select.item_state(key.clone());
                    let part = select.item(key);
                    node()
                        .part(part)
                        .style(move || {
                            let state = state.get();
                            style()
                                .padding(8)
                                .background(if state.active { ColorToken::Selection } else { ColorToken::Raised })
                        })
                        .child(text(move || option.with(|option| option.label())))
                },
            );
            overlay(anchor, Placement::Below, column(rows).style(style().min_width(120)))
                .with_node(|host| host.part(select.content()))
                .name(label)
        }),
    )
}

fn app() -> impl IntoView {
    let size = signal(None::<&'static str>);
    column((
        dropdown(
            "Size",
            vec![ListItem::new("s", "Small"), ListItem::new("m", "Medium"), ListItem::new("l", "Large")],
            size,
        ),
        text(move || format!("size: {}", size.get().unwrap_or("none"))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Dropdown").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_trigger_opens_the_list_and_a_choice_closes_it() {
        let screen = kivact::test::render(app);
        let trigger = screen.get_by_role(Role::ComboBox, "Size");
        assert_eq!(trigger.is_expanded(), Some(false));

        trigger.click();
        assert_eq!(trigger.is_expanded(), Some(true));
        screen.get_by_role(Role::Option, "Medium").click();

        screen.get_by_text("size: m");
        assert!(screen.query_by_role(Role::Option, "Medium").is_none());
        // Focus went back to the trigger.
        assert!(screen.get_by_role(Role::ComboBox, "Size").is_focused());
    }
}
```

The controller kept all of these promises: Up and Down at the closed trigger change
the value without opening, Alt+Down, F4, Enter, Space and a click open it, the list
takes focus on the selected option, Escape closes without choosing, and focus goes
back to the trigger. The same pairing of `trigger()`, `content()` and `show` works
for `use_popup`, `use_disclosure`, `use_tooltip`, `use_dialog` and `use_menu`; only
the Parts differ. For a modal surface, a dialog's content also traps Tab inside
and returns focus to the opener when it closes.

## Drag to change a value

`use_drag` turns pointer movement into a gesture with a start, updates and an end, and
`EditSession` makes the gesture one edit: `Begin`, the writes, then `Commit`,
or `Cancel` when the user presses Escape (the value goes back to where it started).

A drag starts at the press by default. Add `.slop(10)` to wait until movement exceeds 10 logical pixels, and `.axis_lock()` to keep its delta on the first movement axis after that threshold. Add `.track_velocity()` to read `DragUpdate.velocity` in logical pixels per millisecond over the latest 100 ms; without it, velocity is zero. Pending presses that end before the threshold produce no start, update, end or edit callbacks. These are setup options on the existing drag handle.

Use `.source_with(move || value.get_untracked())` when the payload must snapshot live application data at the accepted press. The factory runs once, including when the press first enters the pending slop state. All updates retain that snapshot; the next accepted press samples again. `.source(payload)` keeps fixed setup data.

```rust,test_harness
use std::cell::RefCell;
use std::rc::Rc;

use kivact::geometry::Point;
use kivact::interaction::{DragCoordinates, DragUpdate, EditSession, use_drag};
use kivact::view::{EditPhase};
use kivact::prelude::*;

/// A number you change by dragging sideways.
pub fn scrub(label: impl IntoText, value: Signal<f64>, history: Rc<RefCell<Vec<EditPhase>>>) -> impl Component {
    let edit = EditSession::new(move |initial| value.set(initial))
        .on_edit(move |phase| history.borrow_mut().push(phase));
    let origin = signal(0.0);
    let drag = use_drag::<()>(DragCoordinates::Window)
        .edit(edit, move || value.get_untracked())
        .on_start(move |_| origin.set(value.get_untracked()))
        .on_update(move |update: DragUpdate<()>| {
            value.set(origin.get_untracked() + update.delta.x);
        })
;
    let state = drag.source_state();
    let source = drag.source(());
    node()
        .part(source)
        .name(label)
        .focusable(true)
        .style(move || {
            style()
                .width(120)
                .padding(8)
                .border(1, ColorToken::Border)
                .background(if state.get().active { ColorToken::SurfaceAlt } else { ColorToken::Surface })
        })
        .child(text(move || format!("{:.0}", value.get())))
}

fn app() -> impl IntoView {
    let value = signal(10.0);
    let history = Rc::new(RefCell::new(Vec::new()));
    scrub("Width", value, history).test_id("scrub")
}

fn main() -> kivact::Result<()> {
    App::new("Scrub").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drag_is_one_edit() {
        let history = Rc::new(RefCell::new(Vec::new()));
        let log = history.clone();
        let screen = kivact::test::render(move || {
            let value = signal(10.0);
            scrub("Width", value, log).test_id("scrub")
        });
        let field = screen.get_by_test_id("scrub");
        field.drag(Point::new(20.0, 10.0), Point::new(70.0, 10.0));
        screen.get_by_text("60");
        assert_eq!(*history.borrow(), vec![EditPhase::Begin, EditPhase::Commit]);
    }
}
```

Use `edit_on_change` instead of `edit` when merely pressing must not count as an edit,
as on a canvas where a press selects and only movement edits. For the common value
controls, skip all this: `use_scalar` and `use_range_scalar` give sliders and number
fields (`thumb()`, `track(axis)`, `input()`) with keys, steps, ranges and the same
edit phases, and `use_splitter` gives a resizable divider.

## Say what your control is

A controller sets the role and states for you. A control you build from a bare node
says them itself, through these modifiers. `role`, `checked`, `expanded`, `selected`
and `value_text` are methods of `Node`, and `Node` provides them to a builder through `with_node`.

| Modifier | Meaning |
|---|---|
| `.role(Role::Slider)` | What the control is. Pick the role a user of assistive technology expects. |
| `.name("Rating")` | The accessible name. Every control needs one. |
| `.focusable(true)` | The control takes the keyboard (a stop of Tab). |
| `.value_text(..)`, `.value_min(..)`, `.value_max(..)`, `.value_now(..)` | The value of a slider or a progress bar, in words and as numbers. |
| `.checked(..)`, `.selected(..)`, `.expanded(..)` | Check, selection and disclosure state. |
| `.a11y_action(A11yAction::Increment, ..)` | An action a screen reader can invoke. Offer only what the control can do. |

Note that `.on_press(..)` on a node with no role and no focus is only reachable by
pointer, and debug builds report it. Reach for a controller (`use_button`) first.

## Measure and draw your own shape

When a control has a size no layout rule can express, use a **primitive**. A primitive
is a leaf element whose size you compute with a function. It then has everything a
node has: draw layers, input handlers, focus, accessibility and test ids. The API is
experimental.

```rust,test_harness
use kivact::state::Binding;
use kivact::geometry::{Point, Size};
use kivact::prelude::*;
use kivact::custom::{Constraints, primitive};
use kivact::view::{A11yAction, Handled, KeyEvent, PointerEvent, PointerPhase};

const CELL: f64 = 20.0;

/// A row of `max` dots; the number of lit dots is the rating.
pub fn rating(max: u32, value: impl IntoBinding<u32>) -> impl Component {
    let value = value.into_binding();
    primitive()
        // The size depends only on `max`: the layout measures again when it changes.
        .measure(max, |max: &u32, _: Constraints| Size::new(*max as f64 * CELL, CELL))
        .draw(move |d, size| {
            for i in 0..max {
                let color = if i < value.get() { ColorToken::Accent } else { ColorToken::Border };
                d.circle(Point::new(i as f64 * CELL + CELL / 2.0, size.height / 2.0), 7.0, color);
            }
        })
        .with_node(move |host| host.role(Role::Slider)
        .name("Rating")
        .value_min(0.0)
        .value_max(f64::from(max))
        .value_now(move || f64::from(value.get()))
        .value_text(move || format!("{} of {max}", value.get()))
        .focusable(true)
        .on_pointer(move |event: PointerEvent| {
            if matches!(event.phase(), PointerPhase::Down(_)) {
                value.set(((event.pos().x / CELL) as u32 + 1).min(max));
            }
        })
        .on_key(move |event: KeyEvent| match event.key() {
            Key::ArrowRight => {
                value.set((value.get() + 1).min(max));
                Handled::Yes
            }
            Key::ArrowLeft => {
                value.set(value.get().saturating_sub(1));
                Handled::Yes
            }
            _ => Handled::No,
        })
        .a11y_action(A11yAction::Increment, move |_| value.set((value.get() + 1).min(max)))
        .a11y_action(A11yAction::Decrement, move |_| value.set(value.get().saturating_sub(1)))
        .test_id("rating"))
}

fn app() -> impl IntoView {
    let stars = signal(2);
    column((rating(5, stars), text(move || format!("stars: {}", stars.get()))))
        .style(style().gap(8).padding(16).align_items(Align::Start))
}

fn main() -> kivact::Result<()> {
    App::new("Rating").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_is_sized_by_its_content_and_acts_like_a_slider() {
        let screen = kivact::test::render(app);
        let rating = screen.get_by_role(Role::Slider, "Rating");
        let size = rating.rect().expect("laid out");
        assert_eq!((size.width, size.height), (100.0, 20.0));
        assert_eq!(rating.value().as_deref(), Some("2 of 5"));

        rating.press_key(Key::ArrowRight);
        screen.get_by_text("stars: 3");
        rating.click();
        screen.get_by_text("stars: 3");
    }
}
```

Points to remember about measuring:

- **The measure function is pure.** It runs inside the layout, so it cannot read
  signals, the theme or the tree. Put everything the size depends on into the content
  (here `max`), and the layout measures again only when the content changes.
- **`Constraints` gives the room and the text engine.** `max_width()` is the width you may
  fill, and `measure_text` and `measure_paragraph` measure text with the same shaping
  the screen uses. Return a `Measurement` instead of a `Size` to add a first baseline
  for `Align::Baseline`.
- **A size the layout already fixed wins.** In a column, children are stretched
  across the width by default, and `known_width` then replaces what you measured.
  Align the parent to `Align::Start` (as the example does) when the control should
  keep its own width.
- **A primitive is a leaf.** Children, hit areas beyond its box and native widgets are
  not supported. For several hit areas inside one drawing, declare regions as shown in
  [Draw with a canvas](canvas.md#make-drawn-parts-clickable-and-accessible).

### Draw layers

`.draw(..)` draws behind the children, and `.draw_over(..)` (from `Node`) draws over
them; both receive a `Draw` and the laid-out `Size`. They run again when the size or
a signal they read changes, and only then. A style's `paint` is for themes: use it to
draw a default look that a caller can still replace, and keep `.draw(..)` for what
belongs to the control.

## Edit text with your own frame

`use_text_edit` gives a node native text editing: caret, selection, IME composition,
clipboard, undo and the standard key bindings. It draws nothing, so the frame, the
padding and a counter are yours. The standard `text_input` is this controller plus a
themed frame.

```rust,test_harness
use kivact::state::Binding;
use kivact::interaction::use_text_edit;
use kivact::prelude::*;
use kivact::style::Align;

/// A field with a character counter. `max` is enforced by the editor.
pub fn limited_input(label: impl IntoText, value: impl IntoBinding<String>, max: usize) -> impl Component {
    let value = value.into_binding();
    let api = use_text_edit(value).max_length(max).placeholder("Type here");
    let state = api.control_state();
    let control = api.control();
    let editor = node().part(control).name(label).style(move || {
        style()
            .grow(1)
            .border_bottom(if state.get().focused { 2 } else { 1 }, ColorToken::Accent)
            .padding_y(6)
    });
    row((editor, text(move || format!("{}/{max}", value.with(|text| text.chars().count())))))
        .style(style().align_items(Align::Center).gap(8))
}

fn app() -> impl IntoView {
    let title = signal(String::new());
    column((limited_input("Title", title, 8), text(move || format!("title: {}", title.get()))))
        .style(style().gap(8).padding(16).width(320))
}

fn main() -> kivact::Result<()> {
    App::new("Limited").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_is_limited_and_counted() {
        let screen = kivact::test::render(app);
        let input = screen.get_by_role(Role::TextBox, "Title");
        input.type_text("Hello, Kivact");
        screen.get_by_text("title: Hello, K");
        screen.get_by_text("8/8");
    }
}
```

The binding receives each committed value; during IME composition the editor keeps the
preedit text to itself and the binding sees the final text. To run code on every
change, pass `binding(read, write)` instead of a signal. A few more settings:
`use_text_area(value)` for several lines, `.password(..)`, `.read_only(..)`,
`.on_submit(..)` for Enter in a single line, and `.forward_keys([...])` to hand
arrow, Tab or Escape keys to your own handlers (a suggestion list, say).

For documents too large to copy on every keystroke, such as a code editor, the lower
layer `kivact::custom::TextEditing` reports range edits, takes a key policy and
colors lines. It is experimental; see its rustdoc.

## Write behavior with a state machine

A `Machine` makes states and events explicit. Its pure `transition` returns a
`Step`: the next state, effects to execute and delayed events. Read reactive
inputs in the factory passed to `machine`; run side effects in its effect runner.
The handle exposes reactive state through `get()` and accepts events with `send()`.
Timers belong to the machine owner and are cancelled when it is disposed.

```rust,no_run
use kivact::prelude::*;
use kivact::state::{Machine, Step, machine};
use std::time::Duration;

#[derive(Clone, PartialEq)]
enum CopyState { Idle, Copied }
#[derive(Clone, PartialEq)]
enum CopyEvent { Pressed, Expired }
enum CopyEffect { Write(String) }

struct CopyButton { text: String }               // Self が遷移の読む入力
impl Machine for CopyButton {
    type State = CopyState;
    type Event = CopyEvent;
    type Effect = CopyEffect;
    fn transition(&self, state: &CopyState, event: CopyEvent) -> Step<CopyState, CopyEvent, CopyEffect> {
        match (state, event) {
            (_, CopyEvent::Pressed) => Step::to(CopyState::Copied)
                .with(CopyEffect::Write(self.text.clone()))
                .after(Duration::from_secs(2), CopyEvent::Expired),
            (CopyState::Copied, CopyEvent::Expired) => Step::to(CopyState::Idle),
            (CopyState::Idle, CopyEvent::Expired) => Step::stay(),
        }
    }
}

fn copy_button(text: impl IntoValue<String>) -> impl Component {
    let text = text.into_value();
    let m = machine(CopyState::Idle, move || CopyButton { text: text.get() }, |_, effect| match effect {
        CopyEffect::Write(s) => { spawn(kivact::platform::copy_text(s)); }
    });
    button(move || if m.get() == CopyState::Copied { "Copied" } else { "Copy" })
        .on_press(move || m.send(CopyEvent::Pressed))
}
```

This example teaches the protocol; use the existing clipboard controller when it
fits your control.

## See also

- [Write reusable components](custom-components.md) for props, builders, themes and
  forwarding accessibility.
- [Show a dialog, menu, tooltip or toast](overlays.md) for the standard overlays that are
  built from the same controllers.
- [Draw with a canvas](canvas.md) for regions and the drawing vocabulary used in draw
  layers.
- [Test your app](testing.md) to check roles, keys and focus the way the tests above do.
- `cargo doc -p kivact-interaction --open` for every controller and its Parts, and
  `cargo doc -p kivact --open` for `kivact::custom`.
