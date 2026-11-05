# kivact-ui-basic

The neutral look of the shadcn/ui "Lyra" preset (Neutral colors, square
corners, 12 px controls, Lucide icons) as reusable Kivact components.

The only normal and development dependency is the public `kivact` crate;
there are no build dependencies.
There is no dependence on the renderer, engine, layout, protocol, reactive crates,
or another official UI component crate.

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn settings() -> impl IntoView {
    let sync = signal(true);
    let name = signal(String::new());
    column((
        basic::h4("Profile"),
        basic::label("Display name"),
        kivact::ui::text_input("Display name", name),
        kivact::ui::switch("Sync reports", sync),
        basic::separator(),
        kivact::ui::button("Save").icon(icon("check")).on_press(move || {
            // Persist through the application's own model or service.
        }),
    ))
    .style(style().gap(12).padding(24))
}
```

Standard buttons, checks, switches, inputs, text areas, selects, menus,
dialogs, tooltips, tabs, radio groups, sliders, splitters and toast regions
come from `kivact::ui`. This kit installs their complete Lyra recipes
(button.tsx, checkbox.tsx, switch.tsx, input.tsx, textarea.tsx, select.tsx,
dropdown-menu.tsx, dialog.tsx, tooltip.tsx, tabs.tsx, radio-group.tsx,
slider.tsx, resizable.tsx and sonner's stack) and adds its own components;
it has no copies of the standard ones. `ButtonVariant::Primary` is shadcn's
`default`, `Secondary`, `Outline`, `Ghost` and `Link` keep their names and
`Danger` is `destructive`; `ControlSize::ExtraSmall`, `Small`, `Medium` and
`Large` are the `xs`, `sm`, `default` and `lg` sizes. A button with an icon and a
hidden label (`button(name).icon(icon(..)).hide_label(true)`) is the square `icon`
size. `kivact::ui::toggle_button` (variant `basic::ToggleVariant`), `progress`, `disclosure`
(an accordion is a column of them) and `split` (`.grip(true)` is `withHandle`) are
standard components in this look. A button that opens a popover or a menu does not move
while pressed and keeps the muted fill while its popup is open; tooltips
draw tooltip.tsx's arrow toward their trigger, and vertical `kivact::ui::tabs`
stack full-width tabs beside their panel.

Apply `basic::theme(kivact::style::ThemeMode::Light)` to `App` or a public `provide_theme`
boundary. `ThemeMode::Dark` carries the complete dark Neutral palette. An
application can instead install modified public `basic::Tokens` through
`Theme::with`; component styles read the tokens of the theme in effect,
so switching the theme restyles mounted components without remounting them.

Fonts are application-owned. Register Inter for text and Geist Mono for code
through Kivact's public font API and set the `family` of `theme.text()` with `Theme::with_text` (for example
`"Inter, sans-serif"`); code blocks and inline code ask for
`"Geist Mono", monospace`.

State lives in caller-owned signals and bindings. Collections are read-only
`impl IntoValue<Vec<T>>` inputs, and requests such as a code block's copy
press are reported through callbacks; writing to the clipboard, persistence
and business rules remain application concerns. `kivact::view::provide_direction` provides a
reading direction that the kit's direction-aware components honor; the
standard `kivact::ui` components do not mirror.

Builder components implement `NodeBuilder` and `IntoView` and install their
own default style and semantics first, so ordinary `NodeBuilder` modifiers
such as `.style(...)`, `.test_id(...)`, `.disabled(...)` and `.name(...)` win
over the kit's defaults. Parts without options return a `Node`. A slot
method that places content in a component (`.content(..)`, `.header(..)`,
`.footer(..)`, `.action(..)`, `.media(..)`, ...) takes a view
(`card().content(text("Body"))`); a
container's children are views.

Icons are drawn with the core `kivact::view::icon(name)`: the theme puts
every Lucide name of `basic::ICON_NAMES` in its icon set. Icon slots of single
components take a view (usually `icon(name)`); data records (`ListItem`,
`MenuEntry`, `Command`) and `basic::attachment_action` take the name. A name the set lacks draws nothing
and, in a debug build, is reported at the call.

Dialogs, alert dialogs, sheets and popovers are the standard
`kivact::ui::dialog` of the kinds `DialogMode::Modal`, `AlertModal`,
`Drawer(side)` and `Popover`, which the theme draws as dialog.tsx,
alert-dialog.tsx (its width class is `.variant(basic::AlertDialogSize::Sm)`,
its cancel and action buttons are the footer, and `.closable(false)` leaves
out the close button it does not have), sheet.tsx and popover.tsx. A
use_context menu is the standard `kivact::ui::menu(..).attach_context(target)`,
drawn with use_context-menu.tsx's wider top level. Standard dialogs build their content in a fresh branch whenever they open,
so `.content(..)` and `.footer(..)` take functions. Swipe drawers use
`dialog(title, open).mode(DialogMode::Drawer(edge)).variant(basic::DrawerVariant::Drawer)`;
`.closable(false)` preserves the drawer without a close button. The default
edge dialog recipe is the sheet look. Standard `kivact::ui::hover_card` attaches
with `.attach(trigger)`; the trigger takes its own modifiers. Mount the standard
`kivact::ui::toast_region(toaster()).hold_all_on_hover(true)` once near
the root to show the toasts pushed with
`toaster().push(Toast::new(title).tone(..))`: the core `kivact::interaction::Toast`
takes `.body(..)`, `.duration(..)`, `.action(..)` and `.icon(..)`, and the
theme draws the icon of its `Tone`. In this theme
the toasts stack behind the newest, open into a list while the pointer is
over them or the focus is in them, and hold their time while open. A
toast's action is a plain closure the toast owns, so a press handler can
build and push one.

## Texts and their message keys

The texts the components show or name themselves are English and read
through `kivact::i18n::message` (templates through `message_with`) under
these `basic.`-prefixed keys; an App replaces them with `App::messages`.

| Key | English default |
|---|---|
| `basic.breadcrumb.label` | Breadcrumb |
| `basic.breadcrumb.more` | More |
| `basic.calendar.caption` | {month} {year} |
| `basic.calendar.mon` … `basic.calendar.sun` | Mon … Sun |
| `basic.calendar.month` | Month |
| `basic.calendar.month_1` … `basic.calendar.month_12` | January … December |
| `basic.calendar.short_1` … `basic.calendar.short_12` | Jan … Dec |
| `basic.calendar.next` | Next month |
| `basic.calendar.previous` | Previous month |
| `basic.calendar.year` | Year |
| `basic.chart.area`, `basic.chart.bar`, `basic.chart.donut`, `basic.chart.line`, `basic.chart.pie`, `basic.chart.radar`, `basic.chart.radial`, `basic.chart.stacked_bar` | Area chart, Bar chart, Donut chart, Line chart, Pie chart, Radar chart, Radial chart, Stacked bar chart |
| `basic.chart.summary` | {kind}: {series} |
| `basic.chart.total` | Total |
| `basic.code_block.copied` | Copied |
| `basic.code_block.copy` | Copy |
| `basic.code_block.name` | Code |
| `basic.date_picker.format` | {month} {day}, {year} |
| `basic.date_picker.placeholder` | Pick a date |
| `basic.date_picker.range` | {start} - {end} |
| `basic.message_scroller.end` | Scroll to end |
| `basic.navigation_menu.label` | Navigation |
| `basic.pagination.label` | Pagination |
| `basic.pagination.more` | More pages |
| `basic.pagination.next` | Next |
| `basic.pagination.next_page` | Go to next page |
| `basic.pagination.previous` | Previous |
| `basic.pagination.previous_page` | Go to previous page |
| `basic.questionnaire.label` | Questionnaire |
| `basic.questionnaire.next` | Next |
| `basic.questionnaire.previous` | Previous |
| `basic.questionnaire.progress` | {current} of {total} |
| `basic.questionnaire.skip` | Skip |
| `basic.questionnaire.submit` | Submit |
| `basic.sidebar.toggle` | Toggle Sidebar |

The standard `kivact::ui` components the kit mounts keep their own
unprefixed keys:

| Key | English default |
|---|---|
| `data_table.actions` | Actions |
| `data_table.empty` | No results found. |
| `data_table.filter` | Filter… |
| `data_table.label` | Data table |
| `data_table.next` | Next |
| `data_table.page` | Page {page} of {pages} |
| `data_table.previous` | Previous |
| `data_table.resize` | Resize {title} |
| `data_table.select_all` | Select all |
| `data_table.select_named` | Select {row} |
| `data_table.selected` | {selected} of {total} row(s) selected. |
| `dialog.close` | Close |
| `slider.maximum` | Maximum |
| `slider.minimum` | Minimum |
| `split.handle` | Resize |
| `switch.off` | OFF |
| `switch.on` | ON |
| `toast.close` | Close |

## Catalog and tests

Run `cargo run -p kivact-ui-basic-catalog` from the repository root to view
every component with its source.

`tests/consumer.rs` imports only `kivact` and `kivact_ui_basic` and runs on
real windows (through `dev/tools/window-lock.sh`). It exercises the theme
switch, button variants and sizes, toggles, text editing, icons, code block
copy requests, the layout of the typography styles and the overlays
(accordion made of disclosures, collapsible, alert dialogs, sheets, drawers, popovers, hover
cards, the command palette and toasts).

| Slot kind | API |
|---|---|
| Mounted once: headers, media, actions, footers and other display content | `impl IntoView`, stored as `View` |
| Recreated when opened: dialogs, drawers, collapsibles and hover cards | `impl IntoViewFn<V>`, stored as `ViewFn` |
| Per-item content | `impl Fn(A) -> V`, stored as `ViewFn<A>` |
| Individually wrapped children | `impl IntoChildren` |

## Filled standard recipes

The theme overrides these `UiRecipes` entries; other entries keep the standard defaults.

<!-- kivact-recipes -->
- `field`
- `button`
- `button_content`
- `button_busy`
- `button_icon`
- `toggle`
- `checkbox`
- `switch`
- `text_input`
- `text_area`
- `select`
- `menu`
- `dialog`
- `tooltip`
- `tabs`
- `radio_group`
- `slider`
- `toast`
- `splitter`
- `list`
- `data_table`
- `split`
- `progress`
- `disclosure`
- `command_palette`
- `hover_card`
- `carousel`
- `combobox`
- `toggle_group`
- `button_group`
- `skeleton`
- `spinner`
- `kbd`
<!-- /kivact-recipes -->

`kivact::ui::carousel(index, slides)` uses this theme's Lyra recipes. Orientation,
items per view and looping keep their behavior; previous and next controls remain
square outline buttons outside the viewport. `kivact::ui::hover_card(content)`
uses the same 256 px preview surface, alignment, placement and 600/300 ms delays.
