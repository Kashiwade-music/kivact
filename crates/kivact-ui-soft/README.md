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
| Avatar | Standard `avatar`, `avatar_group(...).total(...).overflow_content(...)`; Soft `AvatarAppearance`, `AvatarGroupAppearance`, `avatar_status_dot`, `avatar_silhouette` |
| Badge / Banner / Blockquote | Standard `badge` with semantic `tone` or Soft categorical `BadgeVariant`; `banner`, `blockquote` |
| Bottom Sheet / Bottom Sheet Switcher | `bottom_sheet`, `bottom_sheet_flow` |
| Breadcrumbs | Standard `breadcrumb` and `ListItem`; Soft `BreadcrumbsVariant` typography |
| Button / Icon Button | Standard `button` and `toggle`, `button_group`, `toggle_group`. Use `.icon(icon).hide_label(true)` for Icon Button. |
| Calendar | Standard `calendar` over single-date or ordered-range bindings |
| Card | standard `card`, `.on_press(...)`, `.selected(...)` |
| Carousel | Standard `carousel(index, slides)` and headless `interaction::use_carousel(index, count)` |
| Chat | `chat_layout`, `chat_message_list`, `chat_message`, `chat_bubble`, `chat_metadata`, `chat_system_message`, `chat_scroll_button`, `chat_composer`, `chat_composer_drawer`, `chat_composer_input`, `chat_dictation_button`, `chat_send_button`, `chat_token`, `chat_tokenized_text`, `chat_tool_calls` |
| Checkbox | Standard `checkbox`, `checkbox_list`, `checkbox_list_item` |
| Citation / Code / Code Block | `citation`, `code`, `code_block` |
| Collapsible | Standard `disclosure(title, open)` and `disclosure_group(selection)`. Use Soft `DisclosureVariant` for chevron position, density, and separators. |
| Command Palette | Standard `command_palette` provides the input and list. Set `.placeholder(..)`, `.empty_text(..)`, and `.footer(..)`. Create entries with `Command::new(..)` and `Command::group(..)`. |
| Context Menu | Standard `menu(..).attach_context(..)` and `MenuEntry::item(..)` |
| Date Input | Standard `date_input`, `date_range_input`, `date_time_input` |
| Dialog | Standard `dialog`, `dialog_header`, `alert_dialog`. Signals own imperative open state. |
| Divider | `divider` |
| Drawer | Standard `dialog(...).kind(DialogKind::Drawer(...))`, `drawer_header` |
| Dropdown Menu | Standard `menu`. Use `MenuEntry::item`, `checkbox`, `separator`, `group`, `radio`, and `submenu`. A radio group is a group of radio entries. |
| Empty State | Standard `empty_state`; `.announce_changes()` retains change announcements and `EmptyStateVariant::compact(value)` controls density |
| Field / Field Label | Pass the label to standard `field(label, control)`. Use `field_status`, standard `input_group`, `input_addon`, and standard `dropzone` for the surrounding controls. |
| Heading | Standard `kivact::ui::heading(value, level)`; `HeadingDisplay` is a visual variant. |
| Hover Card | Standard `hover_card`; shared hover behavior owns delayed visibility and focus. |
| Icon | `icon`, `ICON_NAMES` |
| Indicator | `checkbox_indicator`, `check_indicator`, `radio_indicator` |
| Item / Kbd | `item`, standard parsed `kbd` and typed `kbd_combo` |
| Layout | `center`, `form_layout`, `grid`, `grid_span`, `hstack`, `layout`, `layout_content`, `layout_footer`, `layout_header`, `layout_panel`, `section`, `stack`, `stack_item`, `vstack` |
| Lightbox / Link | `lightbox`, `use_lightbox`, `link` |
| List | `list`, `list_items`, `list_item` |
| Markdown | `markdown` |
| Metadata List | Standard `description_list` / `description`; share `MetadataLook` across list and pairs. |
| More Menu | `more_menu` |
| Navigation | `mobile_nav`, `mobile_nav_toggle`, `nav_heading_menu`, `nav_icon`, `side_nav`, `side_nav_collapse_button`, `side_nav_heading`, `side_nav_item`, `side_nav_section`, `top_nav`, `top_nav_heading`, `top_nav_item`, `top_nav_menu`, `top_nav_mega_menu`, `top_nav_mega_menu_item`, `top_nav_featured_card` |
| Number Input / Outline / Overflow List / Overlay / Pagination | Standard `number_input`, `outline`, `pagination`, `overflow_list`, `media_overlay` |
| Popover | Standard `dialog(...).kind(DialogKind::Popover)` and `kivact::view::overlay` |
| Power Search / Progress Bar | `power_search`, standard `progress` |
| Radio | `radio_list`, standard `ListItem` |
| Resizable | standard `splitter(...).collapsible(..).snaps(..)` |
| Rich Text Editor / Scrollable Area | `rich_text_editor`, `rich_text_toolbar`, `rich_text_view`, `scrollable_area` |
| Segmented Control | Standard `radio_group(...).variant(RadioGroupVariant::Segmented)` and `ListItem`; Soft `SegmentedLayout` and `SegmentedIconOnly` style options |
| Selector | Standard `select(...).open_on_arrow(true)`, `selection_field(...).content(...)`, and `combobox(...).chips()`. |
| Skeleton / Slider / Spinner / Status Dot | Standard `skeleton`, standard `spinner`, `status_dot`, standard `slider` and `range_slider` |
| Stepper | Standard `stepper`, `StepperItem`, `StepIndicator`, `StepStatus`; Soft `StepperAppearance` |
| Switch | Standard `switch` |
| Table | `table`, `table_header`, `table_head`, `table_body`, `table_cell`, `table_footer`, `table_row`, `table_selection_toolbar`, `table_pagination`, `table_row_index`, `table_row_status`. Use standard `data_table` and `DataColumn` for built-in sorting and selection. |
| Tabs | `tab_list`, `tab`, `tab_menu`, standard `tabs` |
| Text through Timestamp | `text`, `thumbnail`, `timer`, `timestamp`, standard `time_input`, `text_input` and `text_area` |
| Toast | Standard `toaster`, `Toast`, `toast_region` |
| Token / Tokenizer / Toolbar | `token`, standard `combobox(...).chips().exclude_selected()`, standard `toolbar`, Soft `with_key_hints` and `keyboard_hint` |
| Tooltip | Standard `tooltip`. The builder owns visibility and placement. |
| Tree List | `tree_list` and themed `kivact::ui::tree` over the shared tree rows |
| Typeahead | Standard `combobox` and `ListItem`; configure `.search(..)`, `.debounce(..)`, `.placeholder(..)`, and `.empty_text(..)`. |
| VisuallyHidden | `visually_hidden` |
| Utilities | `theme`, `tokens`, `media_theme`, `syntax_theme`, `kivact::platform::window_size`, and `kivact::platform::copy_text`. Internationalization Provider maps to `App::locale` and `App::messages`, Layer Provider to `kivact::view::overlay`, and Link Provider to `link_with` with an application callback. Kivact handles node references and diagnostics. |

## Theme, tokens, fonts, and icons

Pass `ThemeMode::Light` or `ThemeMode::Dark` to `theme`. Read `kivact::style::use_tokens::<Tokens>()`
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
| `soft.banner.collapse` | Collapse |
| `soft.banner.dismiss` | Dismiss |
| `soft.banner.dismiss_titled` | {dismiss} {title} |
| `soft.banner.expand` | Expand |
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
| `soft.date_time.date` | Date |
| `soft.date_time.time` | Time |
| `soft.dialog.close` | Close |
| `kbd.key.alt` | Alt |
| `kbd.key.backspace` | Backspace |
| `kbd.key.command` | Command |
| `kbd.key.control` | Control |
| `kbd.key.control_short` | Ctrl |
| `kbd.key.down` | Down arrow |
| `kbd.key.enter` | Enter |
| `kbd.key.escape` | Escape |
| `kbd.key.escape_short` | Esc |
| `kbd.key.left` | Left arrow |
| `kbd.key.plus` | Plus |
| `kbd.key.right` | Right arrow |
| `kbd.key.shift` | Shift |
| `kbd.key.tab` | Tab |
| `kbd.key.up` | Up arrow |
| `kbd.separator` | + |
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
| `soft.rich_text.bold` | Bold |
| `soft.rich_text.inline_code` | Inline code |
| `soft.rich_text.italic` | Italic |
| `soft.rich_text.preview` | Preview |
| `soft.rich_text.strikethrough` | Strikethrough |
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
| `soft.table.label` | Table |
| `soft.thumbnail.fallback_name` | Thumbnail |
| `soft.thumbnail.open` | Open {name} |
| `soft.thumbnail.remove` | Remove {name} |
| `soft.time.icon` | Time |
| `combobox.all_removed` | All items removed |
| `combobox.token_added` | Added {label} |
| `combobox.token_removed` | Removed {label} |
| `soft.token.remove` | Remove {label} |
| `soft.top_nav.label` | Top navigation |
| `soft.tree_list.toggle_children` | Toggle children |
| `combobox.empty` | No results found. |
| `combobox.failed` | Could not load suggestions. |
| `combobox.loading` | Loading… |
| `combobox.placeholder` | Search |

## Catalog and tests

Run the component catalog with `cargo run -p kivact-ui-soft-catalog`. Run the
kit and catalog tests with the repository window lock.

```text
dev/tools/window-lock.sh -- cargo test -p kivact-ui-soft -p kivact-ui-soft-catalog
```

## Current native behavior

Toolbar and `input_group_with(..)` provide a default size to their control
factories. An explicit control size takes precedence. `input_group(..)` joins
already built controls at a shared height. Soft menus support mouse press and
drag selection, touch and pen long press, and keyboard activation. Submenus
wait 150 ms to open on hover and 200 ms to close when
the pointer moves over a sibling item. Submenus and navigation flyouts retain
their open state while the mouse moves diagonally toward the surface.
On touch-capable platforms, date and time fields can open a bottom sheet
with a calendar or picker wheels. `SelectorPresentation::Adaptive` opens a
sheet for touch activation on compact viewports. Some layout changes update
immediately, while Collapsible supports an
optional height animation. The rich text editor offers source editing and
formatted preview. `.block_editing(true)` adds wrapped native editors for
headings, prose, lists, tasks, quotes, and code. Inline Markdown decoration uses
native text and view composition. The application chooses the appearance. Overflow lists measure
mounted items and use provisional widths until measurements arrive.

Toolbar slots use one Tab stop. Arrow keys move among eligible controls and
respect the caret inside a text field. `with_key_hints(axis, || view)` owns one timed hint; `keyboard_hint(target)` observes each actual target.

Date and time wheels currently use numeric labels. Truncation hints show
the full label on hover when native layout cuts the text. Citation and
timestamp underlines are solid.
`carousel(index, slides).mode(CarouselMode::Scroll)` scrolls to slides in response to its controls. Enable `.snap(Snap::Mandatory)`
to settle to a slide after scrolling. `text(..).capsize(true)` trims to the
first visible line's measured cap height and the last visible baseline.

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
selection, proportional widths, column resizing with per-column minimums,
header context menus, and a custom empty state. Use `.expanded_rows(..)` with
`.row_details(..)` for keyboard-operated row details, and `.column_filters(..)`
for per-column queries. Use `.row_selectable(..)` to exclude rows from
selection while keeping them in keyboard navigation. `.column_order(..)`
sets the displayed order, `.sticky_columns(..)` pins columns to the logical
edges, and `.density(..)` selects row spacing. `.row_index(..)` displays row
numbers, `.row_status(..)` supplies named status markers, and `.group_by(..)`
adds collapsible group headings with counts for the current page. Use
`.page_sizes(..)` for a page-size control and `.remote_page(..)` when the
application supplies one page at a time. `.multi_sort(..)` enables
multi-column sorting. Shift activation changes one column while preserving the
other sort keys. Use `.tree(..)` for hierarchical rows and `.tree_expand_all(true)`
for the header control. `DataColumn::filter_field(..)` and
`.typed_column_filters(..)` provide draft popovers or inline filters (C-29).
Number, date, and time filter fields validate values entered as text.

| Work-order item | Current behavior |
| --- | --- |
| C-06, C-07, C-12 | Collapsible, Side Nav, Code Block, and Chat Tool Calls use the shared content-height reveal. `TreeList::animated(true)` clips closing branches as following rows move, approximates the spring with a 734.77 ms curve, and mounts fresh branch content on reopen. Native views can release their layout slot during an exit and receive a completion callback. Soft toast rows animate their height and spacing while the dismissed card leaves the focus and accessibility trees immediately. Toast dismissal callbacks run when dismissal starts. |
| C-08, C-10 | Layout changes reflow directly. Interrupted paint transitions use Kivact's native retargeting. |
| C-09, C-37 | Bottom Sheet uses shared drag velocity, axis, slop, release projection, snap, and overshoot calculations. It supplies its own detents and dismissal rules. Carousel snaps after native scrolling settles. Slider uses relative touch tracking and release coast. Native scrolling can take over a same-axis Slider drag inside a scroll view. Toast pauses its timer during touch or pen swipes. Switch uses touch lift and drag with one paint transition per phase. |
| C-11, C-17, C-18, C-39 | Disabled Soft buttons and menu rows can receive keyboard focus. Use `MenuEntry::disabled_reason(..)` to supply a tooltip for a disabled menu row. Menus support press and drag selection. Edge autoscroll remains pending. Submenus and navigation flyouts use timed hover and preserve diagonal mouse travel. |
| C-15 | Kivact can lay out and mount interactive views in a shaped plain-text paragraph. Soft Markdown still composes marked text and views separately. The rich text editor supports source, block, and preview modes. Inline marks remain visible as Markdown delimiters while editing. Native selections are local to a block, and complex blocks use source and preview modes. |
| C-20, C-21, C-24, C-35, C-36 | The application supplies the appearance. Date and time offer a desktop path and a touch sheet. Soft Selector opens from a bare arrow and retains trigger focus while highlighting an option in the popup. The compact touch sheet moves focus into its modal and returns it on close. MultiSelector and ComplexSelector use the popup path. |
| C-22, C-25, C-38 | Text shows a hint when native shaping reports truncation. Citation and timestamp use solid underlines. Text supports measured cap-height trimming. `form_row(..)` places labels and fields in the shared tracks of a horizontal-label form. |
| C-26 | Soft respects Kivact's reduced-motion preference and the application's override. System detection depends on the platform. |
| C-33, C-34 | The kit supplies its own static list. Overflow List measures mounted items and uses provisional widths before layout. |

## Filled standard recipes

<!-- kivact-recipes -->
- `stat`
- `description_list`
- `heading`
- `button`
- `button_content`
- `button_icon`
- `button_busy`
- `toggle`
- `toolbar`
- `button_group`
- `toggle_group`
- `checkbox`
- `checkbox_indicator`
- `checkbox_check`
- `checkbox_mixed`
- `switch`
- `switch_track`
- `switch_thumb`
- `radio_group`
- `slider`
- `progress`
- `skeleton`
- `badge`
- `spinner`
- `kbd`
- `kbd_key`
- `dropzone`
- `carousel`
- `calendar`
- `date_input`
- `date_range_input`
- `date_time_input`
- `time_input`
- `text_input`
- `text_area`
- `field`
- `number_input`
- `tag_input`
- `select`
- `combobox`
- `menu`
- `dialog`
- `hover_card`
- `tooltip`
- `toast`
- `command_palette`
- `tabs`
- `disclosure`
- `splitter`
- `split`
- `list`
- `tree`
- `data_table`
<!-- /kivact-recipes -->

Filled recipe table: `breadcrumb` (root, list, actual item, separator).

Filled recipe table: `empty_state` (root, title, description, icon, actions).

Filled recipe table: `input_group` (root, label, members, member, addon).

Filled recipe table: `card` (root, header, media, content, footer, overlay).

Filled scalar recipes: `avatar`, `avatar_image`, `avatar_status`, `avatar_status_content`, `avatar_group`, `avatar_group_member`, `avatar_group_overflow`.

Filled recipe table: `stepper` (root, item, layout, body, track, leading, trailing, progress, indicator, label, description, content, end_content, previous, next).
