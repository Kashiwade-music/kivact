# Kivact UI Soft

Soft brings the Astryx Neutral theme to Kivact in light and dark appearances.
Install `kivact_ui_soft::theme(mode)` on your app to use it with the standard
controls. Register Figtree with the app to use the original typeface.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;

fn main() -> kivact::Result<()> {
    App::new("Soft")
        .theme(kivact_ui_soft::theme(ThemeMode::Light))
        .run(|| button("Save").variant(ButtonVariant::Primary))
}
```

## Standard components

Soft applies Neutral recipes to buttons and toggles, checkboxes and radios,
switches, sliders, progress, text and number inputs, fields, selectors, menus,
command palettes, dialogs, tooltips, toasts, tabs, disclosures, lists, tables,
and split panes. Use `ControlSize::Small`, `Medium`, and `Large` for the
standard control sizes. Use the button variant `Primary` for emphasis, `Ghost`
for a quiet action, and `Danger` for destructive actions.

## Astryx component mapping

The standard APIs in this table come from `kivact::ui`.

| Astryx group | Kivact API |
| --- | --- |
| App Shell | Pending |
| Aspect Ratio | `aspect_ratio` |
| Avatar | `avatar`, `avatar_group`, `avatar_group_overflow`, `avatar_status_dot` |
| Badge / Banner / Blockquote | `badge`, `banner`, `blockquote` |
| Bottom Sheet | `bottom_sheet`, `bottom_sheet_flow` |
| Breadcrumbs | `breadcrumbs`, `breadcrumb_item` |
| Button | Standard `button` and `toggle`, `button_group`, `toggle_button_group` |
| Calendar | `calendar`, `calendar_range` |
| Card | `card`, `clickable_card`, `selectable_card` |
| Carousel | Pending |
| Chat | `chat_layout`, `chat_message_list`, `chat_message`, `chat_bubble`, `chat_metadata`, `chat_system_message`, `chat_scroll_button` |
| Checkbox | Standard `checkbox`, `checkbox_list`, `checkbox_list_item` |
| Citation / Code / Code Block | `citation`, `code`, `code_block` |
| Collapsible | `collapsible`, `collapsible_group`. Disclosure state belongs to the binding. |
| Command Palette | Standard `command_palette` and `Command` |
| Context Menu | Standard `menu(...).attach_context(...)` and `MenuEntry` |
| Date Input | `date_input`, `date_range_input`, `date_time_input` |
| Dialog | Standard `dialog`, `dialog_header`, `alert_dialog`. Signals own imperative open state. |
| Divider | `divider` |
| Drawer | Standard `dialog(...).kind(DialogKind::Drawer(...))`, `drawer_header` |
| Dropdown Menu | Standard `menu` and `MenuEntry` |
| Empty State | `empty_state` |
| Field | Standard `field`, `input_group`, `input_group_text`, `file_input` |
| Heading | `heading` |
| Hover Card | `hover_card`. The builder owns delayed visibility. |
| Icon | `icon`, `ICON_NAMES` |
| Indicator | `checkbox_indicator`, `check_indicator`, `radio_indicator` |
| Item / Kbd | `item`, `kbd` |
| Layout | `center`, `form_layout`, `grid`, `grid_span`, `hstack`, `layout` and its regions, `section`, `stack`, `stack_item`, `vstack` |
| Lightbox / Link | `lightbox`, `use_lightbox`, `link` |
| List | `list`, `list_item` |
| Markdown | `markdown` |
| Metadata List | `metadata_list`, `metadata_item` |
| More Menu | `more_menu` |
| Navigation | `nav_icon`. Other navigation builders pending. |
| Number Input / Outline / Overflow List / Overlay / Pagination | Standard `number_input`, `overflow_list`, `media_overlay`. Outline and pagination pending. |
| Popover | Standard `dialog(...).kind(DialogKind::Popover)` and `kivact::view::overlay` |
| Power Search / Progress Bar | `power_search`, standard `progress` |
| Radio | `radio_list`, standard `ListItem` |
| Resizable | Standard `split` |
| Rich Text Editor / Scrollable Area | `rich_text_editor`, `rich_text_toolbar`, `rich_text_view`, `scrollable_area` |
| Segmented Control | `segmented_control`, standard `ListItem` |
| Selector | `selector`, `complex_selector`, `multi_selector`. The Kivact interaction layer supplies combobox state. |
| Skeleton / Slider / Spinner / Status Dot | `skeleton`, `spinner`, `status_dot`, standard `slider` and `range_slider` |
| Stepper | Pending |
| Switch | Standard `switch` |
| Table | Standard `data_table`, `DataColumn` and their built-in sorting and selection |
| Tabs | Standard `tabs`, `ListItem` |
| Text through Timestamp | `text`, `thumbnail`, `timer`, `timestamp`, `time_input`, standard `text_input` and `text_area` |
| Toast | Standard `toaster`, `Toast`, `toast_region` |
| Token / Tokenizer / Toolbar | `token`, `tokenizer`, `toolbar`, `toolbar_item` |
| Tooltip | Standard `tooltip`. The builder owns visibility and placement. |
| Tree List | `tree_list` over `kivact::interaction::use_tree` |
| Typeahead | `typeahead`, `base_typeahead`, standard `ListItem`. The Kivact interaction layer supplies combobox state. |
| VisuallyHidden | `visually_hidden` |
| Utilities | `theme`, `tokens`, `SyntaxTheme`, `kivact::i18n`, `kivact::platform::window_size`, `kivact::platform::copy_text`, Kivact interaction and overlay APIs. React ref merging and React development warnings are unnecessary for native nodes. |

## Theme, tokens, fonts, and icons

Pass `ThemeMode::Light` or `ThemeMode::Dark` to `theme`. Read `tokens()`
inside a style closure to access the active Neutral colors, spacing, radius,
typography, shadows, and motion. Register Figtree with the app when supplying
the font. The catalog includes the font and license. `ICON_NAMES` lists the
bundled Lucide icon names, which can be passed to `icon`.

## Localization

Use Kivact’s internationalization provider to override these default messages.
Controls also use the standard Kivact messages where applicable.

| Message key | English default |
| --- | --- |
| `soft.alert_dialog.cancel` | Cancel |
| `soft.avatar.name_with_status` | {name}, {status} |
| `soft.avatar_group.keyboard_hint` | Use arrow keys to move between avatars |
| `soft.avatar_group.label` | Avatars |
| `soft.avatar_group.overflow` | {count} more |
| `soft.banner.collapse` | Collapse |
| `soft.banner.dismiss` | Dismiss |
| `soft.banner.dismiss_titled` | {dismiss} {title} |
| `soft.banner.expand` | Expand |
| `soft.breadcrumbs.label` | Breadcrumb |
| `soft.calendar.caption` | {month} {year} |
| `soft.calendar.next` | Next month |
| `soft.calendar.previous` | Previous month |
| `soft.calendar.today` | Today |
| `soft.citation.source` | Source {number} |
| `soft.code_block.code` | Code |
| `soft.code_block.copied` | Copied |
| `soft.code_block.copy_code` | Copy code |
| `soft.date.open` | Choose date |
| `soft.date_range.placeholder` | Select date range |
| `soft.date_time.date` | Date |
| `soft.date_time.time` | Time |
| `soft.dialog.close` | Close |
| `soft.file_input.choose_one` | Choose one file. |
| `soft.file_input.clear` | Clear files |
| `soft.file_input.invalid_file` | One or more files are not allowed. |
| `soft.file_input.too_large` | One or more files exceed the size limit or could not be read. |
| `soft.kbd.key.alt` | Alt |
| `soft.kbd.key.backspace` | Backspace |
| `soft.kbd.key.command` | Command |
| `soft.kbd.key.control` | Control |
| `soft.kbd.key.control_short` | Ctrl |
| `soft.kbd.key.down` | Down arrow |
| `soft.kbd.key.enter` | Enter |
| `soft.kbd.key.escape` | Escape |
| `soft.kbd.key.escape_short` | Esc |
| `soft.kbd.key.left` | Left arrow |
| `soft.kbd.key.plus` | Plus |
| `soft.kbd.key.right` | Right arrow |
| `soft.kbd.key.shift` | Shift |
| `soft.kbd.key.tab` | Tab |
| `soft.kbd.key.up` | Up arrow |
| `soft.kbd.shortcut` | {shortcut} + {key} |
| `soft.keyboard_hint.to_navigate` | to navigate |
| `soft.lightbox.media_viewer` | Media viewer |
| `soft.lightbox.zoom` | Zoom |
| `soft.link.new_tab` | (opens in new tab) |
| `soft.markdown.task_list` | Task list |
| `soft.markdown_heading_links.copy` | Copy link to {heading} |
| `soft.more_menu.label` | More options |
| `soft.power_search.add` | Add filter |
| `soft.power_search.apply` | Apply |
| `soft.power_search.contains` | Contains |
| `soft.power_search.field` | Field |
| `soft.power_search.is` | Is |
| `soft.power_search.is_not` | Is not |
| `soft.power_search.operator` | Operator |
| `soft.power_search.value` | Value |
| `soft.relative_time.now` | now |
| `soft.relative_time.yesterday` | yesterday |
| `soft.spinner.loading` | Loading |
| `soft.table.label` | Table |
| `soft.thumbnail.fallback_name` | Thumbnail |
| `soft.thumbnail.open` | Open {name} |
| `soft.thumbnail.remove` | Remove {name} |
| `soft.time.icon` | Time |
| `soft.token.remove` | Remove {label} |
| `soft.typeahead.empty` | No results found. |
| `soft.typeahead.placeholder` | Search |

## Catalog and tests

Run the component catalog with `cargo run -p kivact-ui-soft-catalog`. Run the
kit and catalog tests with the repository window lock.

```text
dev/tools/window-lock.sh -- cargo test -p kivact-ui-soft -p kivact-ui-soft-catalog
```

## Current native behavior

Toolbar item sizes are explicit where needed. Text color changes immediately.
Standard busy buttons still use the native spinner timing. Standard menus
open by activation and use the native submenu timing. Touch-specific date
pickers and adaptive selector sheets remain pending. Some disclosure and
layout changes update immediately. The rich text editor offers source editing
and formatted preview. Inline Markdown decoration uses the native
text and view composition available to the kit. The application chooses the
appearance, and overflow list sizing uses item-width hints.
