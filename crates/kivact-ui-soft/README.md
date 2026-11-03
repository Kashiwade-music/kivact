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
and split panes.

| Astryx | Standard Kivact option |
| --- | --- |
| `primary`, `secondary`, `ghost`, `destructive` | `ButtonVariant::Primary`, `Secondary`, `Ghost`, `Danger` |
| `sm`, `md`, `lg` | `ControlSize::Small`, `Medium`, `Large` |
| Icon-only button with an accessible label | `button(label).icon(icon).hide_label(true)` |
| Loading button | `button(label).loading(loading)` |
| Disabled control | `.disabled(disabled)` |

Soft also renders `ControlSize::ExtraSmall` buttons at 24 logical pixels.

## Astryx component mapping

The standard APIs in this table come from `kivact::ui`.

| Astryx group | Kivact API |
| --- | --- |
| App Shell | `app_shell`, `use_app_shell_mobile` |
| Aspect Ratio | `aspect_ratio` |
| Avatar | `avatar`, `avatar_group`, `avatar_group_overflow`, `avatar_status_dot` |
| Badge / Banner / Blockquote | `badge`, `banner`, `blockquote` |
| Bottom Sheet | `bottom_sheet`, `bottom_sheet_flow` |
| Breadcrumbs | `breadcrumbs`, `breadcrumb_item` |
| Button | Standard `button` and `toggle`, `button_group`, `toggle_button_group` |
| Calendar | `calendar`, `calendar_range` |
| Card | `card`, `clickable_card`, `selectable_card` |
| Carousel | `carousel`, `use_carousel` |
| Chat | `chat_layout`, `chat_message_list`, `chat_message`, `chat_bubble`, `chat_metadata`, `chat_system_message`, `chat_scroll_button`, `chat_composer`, `chat_composer_drawer`, `chat_composer_input`, `chat_dictation_button`, `chat_send_button`, `chat_token`, `chat_tokenized_text`, `chat_tool_calls` |
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
| Field | Standard `field`, `field_status`, `input_group`, `input_group_text`, `file_input` |
| Heading | `heading` |
| Hover Card | `hover_card`. The builder owns delayed visibility. |
| Icon | `icon`, `ICON_NAMES` |
| Indicator | `checkbox_indicator`, `check_indicator`, `radio_indicator` |
| Item / Kbd | `item`, `kbd` |
| Layout | `center`, `form_layout`, `grid`, `grid_span`, `hstack`, `layout` and its regions, `section`, `stack`, `stack_item`, `vstack` |
| Lightbox / Link | `lightbox`, `use_lightbox`, `link` |
| List | `list`, `list_items`, `list_item` |
| Markdown | `markdown` |
| Metadata List | `metadata_list`, `metadata_item` |
| More Menu | `more_menu` |
| Navigation | `mobile_nav`, `mobile_nav_toggle`, `nav_heading_menu`, `nav_icon`, `side_nav` and its slots, `top_nav` and its menu and mega-menu builders |
| Number Input / Outline / Overflow List / Overlay / Pagination | Standard `number_input`, `outline`, `pagination`, `overflow_list`, `media_overlay` |
| Popover | Standard `dialog(...).kind(DialogKind::Popover)` and `kivact::view::overlay` |
| Power Search / Progress Bar | `power_search`, standard `progress` |
| Radio | `radio_list`, standard `ListItem` |
| Resizable | `use_resizable`, `resize_handle`, standard `split` |
| Rich Text Editor / Scrollable Area | `rich_text_editor`, `rich_text_toolbar`, `rich_text_view`, `scrollable_area` |
| Segmented Control | `segmented_control`, standard `ListItem` |
| Selector | `selector`, `complex_selector`, `multi_selector`. The Kivact interaction layer supplies combobox state. |
| Skeleton / Slider / Spinner / Status Dot | `skeleton`, `spinner`, `status_dot`, standard `slider` and `range_slider` |
| Stepper | `stepper`, `step` |
| Switch | Standard `switch` |
| Table | `table`, `table_header`, `table_head`, `table_body`, `table_cell`, `table_footer`, `table_row`, `table_selection_toolbar`, `table_pagination`, `table_row_index`, `table_row_status`. Use standard `data_table` and `DataColumn` for built-in sorting and selection. |
| Tabs | `tab_list`, `tab`, `tab_menu`, standard `tabs` |
| Text through Timestamp | `text`, `thumbnail`, `timer`, `timestamp`, `time_input`, standard `text_input` and `text_area` |
| Toast | Standard `toaster`, `Toast`, `toast_region` |
| Token / Tokenizer / Toolbar | `token`, `tokenizer`, `toolbar`, `toolbar_item` |
| Tooltip | Standard `tooltip`. The builder owns visibility and placement. |
| Tree List | `tree_list` and themed `kivact::ui::tree` over the shared tree rows |
| Typeahead | `typeahead`, `base_typeahead`, standard `ListItem`. The Kivact interaction layer supplies combobox state. |
| VisuallyHidden | `visually_hidden` |
| Utilities | `theme`, `tokens`, `media_theme`, `syntax_theme`, `kivact::i18n`, `kivact::platform::window_size`, `kivact::platform::copy_text`, Kivact interaction and overlay APIs. Links use application callbacks on native. React ref merging and React development warnings are unnecessary for native nodes. |

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
| `soft.app_shell.main` | Main content |
| `soft.app_shell.skip_to_content` | Skip to content |
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
| `soft.calendar.chosen` | Date selected |
| `soft.calendar.fri` | Fr |
| `soft.calendar.mon` | Mo |
| `soft.calendar.month_1` | January |
| `soft.calendar.month_10` | October |
| `soft.calendar.month_11` | November |
| `soft.calendar.month_12` | December |
| `soft.calendar.month_2` | February |
| `soft.calendar.month_3` | March |
| `soft.calendar.month_4` | April |
| `soft.calendar.month_5` | May |
| `soft.calendar.month_6` | June |
| `soft.calendar.month_7` | July |
| `soft.calendar.month_8` | August |
| `soft.calendar.month_9` | September |
| `soft.calendar.next` | Next month |
| `soft.calendar.previous` | Previous month |
| `soft.calendar.range_cancelled` | Range selection cancelled |
| `soft.calendar.range_chosen` | Date range selected |
| `soft.calendar.range_started` | Start date selected |
| `soft.calendar.sat` | Sa |
| `soft.calendar.sun` | Su |
| `soft.calendar.thu` | Th |
| `soft.calendar.today` | Today |
| `soft.calendar.tue` | Tu |
| `soft.calendar.wed` | We |
| `soft.carousel.scroll_left` | Scroll left |
| `soft.carousel.scroll_right` | Scroll right |
| `soft.carousel.slide` | Slide {current} of {total} |
| `soft.chat.tool_calls.complete` | Complete |
| `soft.chat.tool_calls.detail` | Details for {name} |
| `soft.chat.tool_calls.details` | Details |
| `soft.chat.tool_calls.error` | Error: {message} |
| `soft.chat.tool_calls.failed` | Failed |
| `soft.chat.tool_calls.group_label` | {count} tool calls |
| `soft.chat.tool_calls.pending` | Pending |
| `soft.chat.tool_calls.running` | Running |
| `soft.chat_composer.input` | Message input |
| `soft.chat_composer.placeholder` | Type a message… |
| `soft.chat_composer_drawer.items` | Items |
| `soft.chat_dictation_button.label` | Dictation |
| `soft.chat_layout_scroll_button.scroll_to_bottom` | Scroll to bottom |
| `soft.chat_send_button.send` | Send |
| `soft.chat_send_button.stop` | Stop |
| `soft.citation.source` | Source {number} |
| `soft.code_block.code` | Code |
| `soft.code_block.copied` | Copied |
| `soft.code_block.copy_code` | Copy code |
| `soft.date.open` | Choose date |
| `soft.date_range.placeholder` | Select date range |
| `soft.date_range_input.presets` | Preset date ranges |
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
| `soft.lightbox.image_unavailable` | Image unavailable |
| `soft.lightbox.media_viewer` | Media viewer |
| `soft.lightbox.zoom` | Zoom |
| `soft.link.new_tab` | (opens in new tab) |
| `soft.markdown.task_list` | Task list |
| `soft.markdown_heading_links.copy` | Copy link to {heading} |
| `soft.metadata_list.show_less` | Show less |
| `soft.metadata_list.show_more` | Show more |
| `soft.mobile_nav.close` | Close navigation |
| `soft.mobile_nav.navigation` | Navigation |
| `soft.mobile_nav.open` | Open navigation |
| `soft.more_menu.label` | More options |
| `soft.outline.label` | Table of contents |
| `soft.pagination.count` | {from}–{to} of {total} |
| `soft.pagination.first` | Go to first page |
| `soft.pagination.go_to_page` | Go to page {page} |
| `soft.pagination.go_to_page_input` | Go to page |
| `soft.pagination.items_per_page` | Items per page |
| `soft.pagination.label` | Pagination |
| `soft.pagination.last` | Go to last page |
| `soft.pagination.next` | Go to next page |
| `soft.pagination.next_by` | Go forward {step} pages |
| `soft.pagination.of_total_pages` | / {total} |
| `soft.pagination.page_announce` | Page {current} |
| `soft.pagination.page_indicators` | Page indicators |
| `soft.pagination.page_label` | Page |
| `soft.pagination.page_of_total` | Page {current} of {total} |
| `soft.pagination.previous` | Go to previous page |
| `soft.pagination.previous_by` | Go back {step} pages |
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
| `soft.resize_handle.label` | Resize handle |
| `soft.rich_text.preview` | Preview |
| `soft.rich_text.toolbar` | Editor toolbar |
| `soft.rich_text.write` | Write |
| `soft.side_nav.collapse` | Collapse sidebar |
| `soft.side_nav.expand` | Expand sidebar |
| `soft.side_nav.heading_dialog` | Navigation menu |
| `soft.side_nav.heading_open_menu` | {heading}, Open menu |
| `soft.side_nav.item_collapse` | Collapse {label} |
| `soft.side_nav.item_expand` | Expand {label} |
| `soft.side_nav.item_submenu` | {label} submenu |
| `soft.side_nav.label` | Side navigation |
| `soft.side_nav.resize` | Resize sidebar |
| `soft.spinner.loading` | Loading |
| `soft.stepper.label` | Progress |
| `soft.stepper.next` | Next step |
| `soft.stepper.optional` | • Optional |
| `soft.stepper.previous` | Previous step |
| `soft.table.label` | Table |
| `soft.thumbnail.fallback_name` | Thumbnail |
| `soft.thumbnail.open` | Open {name} |
| `soft.thumbnail.remove` | Remove {name} |
| `soft.time.icon` | Time |
| `soft.tokenizer.all_removed` | All items removed |
| `soft.tokenizer.token_added` | Added {label} |
| `soft.tokenizer.token_removed` | Removed {label} |
| `soft.token.remove` | Remove {label} |
| `soft.top_nav.label` | Top navigation |
| `soft.tree_list.toggle_children` | Toggle children |
| `soft.typeahead.empty` | No results found. |
| `soft.typeahead.failed` | Could not load suggestions. |
| `soft.typeahead.loading` | Loading… |
| `soft.typeahead.placeholder` | Search |

## Catalog and tests

Run the component catalog with `cargo run -p kivact-ui-soft-catalog`. Run the
kit and catalog tests with the repository window lock.

```text
dev/tools/window-lock.sh -- cargo test -p kivact-ui-soft -p kivact-ui-soft-catalog
```

## Current native behavior

Toolbar and `input_group_with(..)` provide a default size to their control
factories. An explicit control size takes precedence. `input_group(..)` joins
already built controls at a shared height. Menus open
by activation. Submenus wait 150 ms to open on hover and 200 ms to close when
the pointer moves over a sibling item. Diagonal pointer tracking remains
pending. Touch-specific date pickers and adaptive selector sheets remain
pending. Some layout changes update immediately, while Collapsible supports an
optional height animation. The rich text editor offers source editing and
formatted preview. Inline Markdown decoration uses native text and view
composition. The application chooses the appearance. Overflow lists measure
mounted items and use provisional widths until measurements arrive.

Toolbar slots use one Tab stop. Arrow keys move among eligible controls and
respect the caret inside a text field. `toolbar_item(..)` registers a control
for the keyboard hint.

Date and time formatting uses the desktop input path. Truncation hints can
show the full label on hover. Citation and timestamp underlines are solid.
Carousel scrolls to slides in response to its controls. Font cap-height
trimming uses the native line box.

Chat Layout follows new messages while the viewport is near the end. Pass
`.history_revision(..)` and update it in the same batch as a history prepend
to preserve the reader's position.

Toast uses the standard queue. Pass `ToastOptions` to `Toaster::push_with(..)`
to deduplicate by key or register a callback when a visible toast is dismissed.
Hovering or focusing one toast pauses its timer. Dismissing a focused toast
restores the last available focus outside it.

Typeahead and Tokenizer accept a debounced asynchronous source through
`.search(..)`. The source publishes one batch for each query. Power Search
supports editable text filters. The static table builders and `TableFocus::None`
provide native table semantics. The standard data table supports sorting,
selection, column resizing, and a custom empty state. Grouped, expandable, and sticky
data-table rows and columns require additional integration. Advanced column
filtering, multi-column sorting, relative widths, and column reordering remain
pending (C-29).

| Work-order item | Current behavior |
| --- | --- |
| C-06, C-07 | Collapsible and Side Nav use the shared content-height reveal. Other layout changes can be immediate. An exiting toast can retain its row until the card transition finishes. Toast dismissal callbacks run when dismissal starts. |
| C-08, C-10 | Layout changes reflow directly. Interrupted paint transitions use Kivact's native retargeting. |
| C-09, C-37 | Bottom Sheet combines native drag tracking with local detent and release logic. Carousel uses native scrolling. Toast and Slider use their standard dismissal and input controls. |
| C-11, C-17, C-18, C-39 | Disabled Soft buttons can receive focus for tooltips. Menus skip disabled rows, use activation to open, and use timed hover for submenus. The safe triangle and drag-to-select behavior remain pending. |
| C-15 | Inline Markdown uses native text and view composition. The rich text editor has source and preview modes. |
| C-20, C-21, C-24, C-35, C-36 | The application supplies the appearance. Date and time use the desktop input flow and its formatting. Touch wheels and adaptive selector sheets remain pending. |
| C-22, C-25, C-38 | Truncation hints can appear whenever truncation is enabled. Citation and timestamp use solid underlines. Text uses the native line box. |
| C-26 | Soft respects Kivact's reduced-motion preference and the application's override. System detection depends on the platform. |
| C-33, C-34 | The kit supplies its own static list. Overflow List measures mounted items and uses provisional widths before layout. |
