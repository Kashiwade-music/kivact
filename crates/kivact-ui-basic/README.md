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
        kivact::ui::heading("Profile", 4),
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
`impl IntoValue<Vec<T>>` inputs. Core handles code-block clipboard writes and
reports copy presses through callbacks; persistence and business rules remain
application concerns. `kivact::view::provide_direction` provides a
reading direction that the kit's direction-aware components honor; standard `kivact::ui` components read the same direction context.

Builder components implement `NodeBuilder` and `IntoView` and install their
own default style and semantics first, so ordinary `NodeBuilder` modifiers
such as `.style(...)`, `.test_id(...)`, `.disabled(...)` and `.name(...)` win
over the kit's defaults. Parts without options return a `Node`. A slot
method that places content in a component (`.content(..)`, `.header(..)`,
`.footer(..)`, `.action(..)`, `.media(..)`, ...) takes a view
(`kivact::ui::card(text("Body"))`); a
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
| `basic.breadcrumb.more` | More |
| `basic.chart.area`, `basic.chart.bar`, `basic.chart.donut`, `basic.chart.line`, `basic.chart.pie`, `basic.chart.radar`, `basic.chart.radial`, `basic.chart.stacked_bar` | Area chart, Bar chart, Donut chart, Line chart, Pie chart, Radar chart, Radial chart, Stacked bar chart |
| `basic.chart.summary` | {kind}: {series} |
| `basic.chart.total` | Total |
| `basic.plot.summary` | {label}: {count} series |
| `basic.questionnaire.label` | Questionnaire |
| `basic.questionnaire.next` | Next |
| `basic.questionnaire.previous` | Previous |
| `basic.questionnaire.progress` | {current} of {total} |
| `basic.questionnaire.skip` | Skip |
| `basic.questionnaire.submit` | Submit |
| `basic.sidebar.toggle` | Toggle Sidebar |

The standard `kivact::ui` components the kit mounts keep their own
standard message keys:

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

## Migrating from the previous release

The kit now builds on the standard `kivact::ui` components: most kit
builders became standard components that the kit's `theme` draws in
its look, and the kit's looks are `.variant(..)` values. Previous
names and their replacements:

| Previous | Now |
|---|---|
| `alert`, `Alert`, `AlertVariant` | `kivact::ui::alert(title)` / `kivact::ui::Alert`; `.variant(basic::AlertLook::Destructive)` (not the core `kivact::ui::AlertVariant`); `.action(closure)` -> `.actions(view)` |
| `aspect_ratio(r, child)` | Removed: `node().style(style().width(pct(100)).aspect_ratio(r).overflow(kivact::style::Overflow::Hidden)).child(child)` |
| `avatar`, `Avatar`, `AvatarSize` | `kivact::ui::avatar(name).fallback_content(\|\| text(..))` / `kivact::ui::Avatar`; `.image(a)` -> `.src(Some(kivact::view::Asset::from(a)))`; `.size(kivact::ui::ControlSize::Small)` (or `Large`); `.badge(..)` -> `.status(Some(kivact::interaction::Tone::..))` or `.status_content(view)` |
| `avatar_group`, `avatar_group_count` | `kivact::ui::avatar_group(children)`; the count is `.max(n).total(m)` (or `.overflow_content(..)`) |
| `badge`, `Badge` | `kivact::ui::badge(label).variant(basic::BadgeVariant::..)` / `kivact::ui::Badge` |
| `breadcrumb`, `breadcrumb_content`, `breadcrumb_link`, `BreadcrumbLink`, `breadcrumb_page`, `breadcrumb_separator` | `kivact::ui::breadcrumb(vec![ListItem::new(key, label), ..])` (the last item is the page, or `.current(Some(key))`); `.on_activate(\|key\| ..)`, `.separator(\|\| view)`, `.item(\|item\| view)`; `basic::breadcrumb_ellipsis()` goes in `.item(..)` |
| `bubble`, `Bubble`, `BubbleAlign`, `BubbleReactionsSide` | `kivact::ui::chat_message("").variant(basic::ChatLayout::Bubble).content(..)`; `.variant(basic::BubbleVariant::..)`; `.align(kivact::ui::ChatAlign::End)`; `.reactions_side(kivact::ui::ChatReactionsSide::Top)`; `.reactions(view)` |
| `bubble_group`, `message_group` | Removed: `column((..)).style(style().gap(8))` |
| `button_group`, `ButtonGroup`, `button_group_separator`, `button_group_text` | `kivact::ui::button_group(label, children)` / `kivact::ui::ButtonGroup`; `kivact::ui::button_group_separator()`, `kivact::ui::button_group_text(..)`; `.orientation(..)` |
| `calendar`, `calendar_range`, `Calendar`, `CalendarCaption` | `kivact::ui::calendar(signal(None::<CalendarDate>))`, or `calendar(signal(Some(DateRange::new(..))))` for a range, with `.variable_rows(true)` for this look; `kivact::ui::Calendar`, `kivact::ui::CalendarCaption` |
| `card`, `Card` (`.header`, `.content`, `.footer`, `.image`, `.size(CardSize)`) | `kivact::ui::card(content).header(basic::card_header(..)).media(view).footer(view)` / `kivact::ui::Card`; `.size(..)` -> `.variant(basic::CardSize::Sm)` |
| `card_header(..).bordered(b)`, `.action(closure)` | `.variant(basic::CardLook::default().header_bordered(b))` on the card; `.action(view)` |
| `carousel(items, index)`, `Carousel` | `kivact::ui::carousel(index, items)` (arguments swapped) / `kivact::ui::Carousel`; `.looping(true)` -> `.looping()`; `.items_per_view(..)` |
| `code_block(lines)`, `CodeBlock`, `code_lines` | `kivact::ui::code_block(source)` / `kivact::ui::CodeBlock` (plain text needs no `code_lines`); `.size(..)` -> `.variant(basic::CodeBlockSize::Sm)` (reactive: `basic::CodeBlockLook`); the copy button is built in (`.copy_button(false)` hides it); `.footer(view)` |
| `collapsible`, `Collapsible` | `kivact::ui::disclosure(title, open).variant(basic::DisclosureVariant::Plain).content(..).attach(trigger)` / `kivact::ui::Disclosure` |
| `Color` | `basic::ColorName` |
| `combobox(items, value)`, `Combobox` | `kivact::ui::combobox(label, items, selection)` / `kivact::ui::Combobox`; `.chips()` for a multi-select with chips |
| `date_picker`, `DatePicker` | `kivact::ui::date_input(label, date).presentation(kivact::ui::DatePresentation::Desktop).variant(basic::DatePickerVariant::Flush).attach(button(..))` / `kivact::ui::DateInput`; `.placeholder(..)` and `.format(..)` -> the attached button's label |
| `date_range_picker`, `DateRangePicker` | `kivact::ui::date_range_input(label, range)` with the same options / `kivact::ui::DateRangeInput`; `.calendar(\|c\| ..)` unchanged |
| `drawer(open, title)`, `Drawer` | `kivact::ui::dialog(title, open).mode(kivact::interaction::DialogMode::Drawer(kivact::geometry::Edge::Bottom)).variant(basic::DrawerVariant::Drawer)` / `kivact::ui::Dialog`; `.side(s)` -> the `Drawer(edge)` value |
| `empty_state`, `EmptyState` | `kivact::ui::empty_state(title)` / `kivact::ui::EmptyState`; `.media(..)` -> `.icon(view)`; `.media_variant(v)` -> `.variant(basic::EmptyMediaVariant::Icon)`; `.content(..)` -> `.actions(view)` |
| `GroupItems` | `kivact::view::IntoChildren` (a tuple or `Vec`; one child is `(view,)`) |
| `h1`, `h2`, `h3`, `h4` | `kivact::ui::heading(text, level)` |
| `hover_card`, `HoverCard` | `kivact::ui::hover_card(content).attach(trigger)` / `kivact::ui::HoverCard` |
| `input_group(control)`, `InputGroup` | `kivact::ui::input_group(label, (control,))` / `kivact::ui::InputGroup`; `.start(view)`, `.end(view)`, `.block_end(view)`, `.invalid(..)` |
| `input_group_button`, `InputGroupButton` | `kivact::ui::button(label).variant(ButtonVariant::Ghost).variant(basic::InputGroupButtonSize::Xs)` (reactive: `basic::InputGroupButtonLook::default().size(..)`) |
| `input_group_input`, `input_group_textarea` | `kivact::ui::text_input(..)` / `kivact::ui::text_area(..)` with `.variant(basic::InputGroupControl)` |
| `input_group_text` | `kivact::ui::input_addon(text)` |
| `item`, `Item` | `kivact::ui::item_row(title)` / `kivact::ui::ItemRow`; `.media(view)`, `.actions(view)`; `.media_variant(v)` -> `.variant(basic::ItemMediaVariant::..)`; `.variant(basic::ItemVariant::..)`; `.size(s)` -> `.variant(basic::ItemSize::Sm)` (reactive: `basic::ItemLook`) |
| `item_group` | `kivact::ui::item_list(children)`, with `basic::item_separator()` between rows |
| `kbd` | `kivact::ui::kbd(keys)`, grouped with `basic::kbd_group(..)` |
| `menubar(menus, on_activate)` | `basic::menubar(menus).on_activate(..)` (returns `basic::Menubar`) |
| `message`, `Message`, `MessageAlign` | `kivact::ui::chat_message(author).content(..).variant(basic::BubbleVariant::..)` / `kivact::ui::ChatMessage`; the author argument replaces `.header(..)`; `.avatar(view)`, `.footer(view)`; `.align(kivact::ui::ChatAlign::End)` |
| `message_scroller`, `MessageScroller` | `kivact::ui::chat_thread(content)` / `kivact::ui::ChatThread` |
| `navigation_menu(vec![NavItem..])`, `NavItem` | `kivact::ui::navigation_menu(label)` with `.link(key, label, panel)` (was `NavItem::menu`) and `.action(key, label, f)` (was `NavItem::link`) |
| `navigation_menu_link`, `NavLink` | `kivact::ui::navigation_menu_link(title)` / `kivact::ui::NavigationMenuLink` |
| `pagination(page, count)`, `PaginationPages` | `kivact::ui::pagination(count, page)` (arguments swapped) / `kivact::ui::Pagination`; `.label_format(..)` unchanged |
| `pagination_content`, `pagination_link`, `PaginationLink`, `pagination_previous`, `pagination_next`, `pagination_ellipsis` | Removed: `kivact::ui::pagination(count, page).item(\|item: kivact::ui::PageItem\| view)` with `.siblings(n)` |
| `rust_lines(src)` (returned `Vec<CodeLine>`) | `basic::rust_source(src)`; `basic::rust_lines` is now the callback of `kivact::ui::code_block(src).highlight(move \|\| (lines.clone(), kivact::style::use_tokens::<basic::Tokens>().code.clone()), basic::rust_lines)` |
| `SelectionValue` | `kivact::interaction::IntoSelection` |
| `skeleton`, `Skeleton`, `spinner` | `kivact::ui::skeleton()` / `kivact::ui::Skeleton`, `kivact::ui::spinner()` |
| `table`, `table_body`, `table_cell`, `table_footer`, `table_head`, `table_header`, `table_row`, `TableCell`, `TableRow`, `TableAlign` | `kivact::ui::table*` with tuple or `Vec` children (`table_header((row,))`); `kivact::ui::TableCell`, `kivact::ui::Table`, `kivact::ui::TableRow`; `.align(kivact::style::TextAlign::End)`; `table_row(..).selected(b)` (`kivact::ui::TableRow`); `basic::table_caption(..)` after the table |
| `kivact::interaction::Toaster` | `kivact::interaction::ToasterApi`, the handle `toaster()` returns |
| `toggle_group(items, value)`, `ToggleGroup` | `kivact::ui::toggle_group(label, items, selection)` / `kivact::ui::ToggleGroup` |
| `trace_collapsible(open, line, content)` | Same function; `content` is a closure `\|\| view` that runs each time the line opens |

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
- `badge`
- `calendar`
- `empty_state`
- `alert`
- `avatar`
- `avatar_image`
- `avatar_group`
- `avatar_group_member`
- `avatar_group_overflow`
- `avatar_status`
- `table`
- `item_list`
- `card`
- `heading`
- `input_group`
- `navigation_menu`
- `breadcrumb`
- `pagination`
- `code_block`
- `chat_message`
- `chat_thread`
<!-- /kivact-recipes -->

`kivact::ui::carousel(index, slides)` uses this theme's Lyra recipes. Orientation,
items per view and looping keep their behavior; previous and next controls remain
square outline buttons outside the viewport. `kivact::ui::hover_card(content)`
uses the same 256 px preview surface, alignment, placement and 600/300 ms delays.

`MenuVariant::Bar` is the public compact menu appearance used by the Basic menubar.
`AlertLook::{Default, Destructive}` controls Basic callout colors independently of
Core's structural `AlertVariant`. `DatePickerVariant::Flush` removes popup insets
for a calendar trigger. All are ordinary `.variant(..)` values; no private variant
is required to reproduce the kit's appearance. The remaining visual enum values
also implement `kivact::ui::Variant`.
