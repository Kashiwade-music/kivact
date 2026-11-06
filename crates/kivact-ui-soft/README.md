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
| Badge / Banner / Blockquote | Standard `badge` with semantic `tone` or Soft categorical `BadgeVariant`; standard `alert` with Soft `banner_icon`, `BannerAppearance`, and `BannerContainer`; `blockquote` |
| Bottom Sheet / Bottom Sheet Switcher | `bottom_sheet`, `bottom_sheet_flow` |
| Breadcrumbs | Standard `breadcrumb` and `ListItem`; Soft `BreadcrumbsVariant` typography |
| Button / Icon Button | Standard `button` and `toggle`, `button_group`, `toggle_group`. Use `.icon(icon).hide_label(true)` for Icon Button. |
| Calendar | Standard `calendar` over single-date or ordered-range bindings |
| Card | standard `card`, `.on_press(...)`, `.selected(...)` |
| Carousel | Standard `carousel(index, slides)` and headless `interaction::use_carousel(index, count)` |
| Chat domains | `chat_dictation_button`, `chat_token`, `chat_tokenized_text`, `chat_tool_calls`; standard `ui::chat_thread`, `ui::chat_message`, `ui::chat_composer`, `ui::typing_indicator` |
| Checkbox | Standard `checkbox`, `checkbox_list`, `checkbox_list_item` |
| Citation / Code | `citation`, `code`, `code_highlight`; standard `ui::code_block` |
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
| Icon | Standard `kivact::view::icon`; Soft `icon_view`, `icon_style`, `IconSize`, `IconColor`, `ICON_NAMES` |
| Indicator | `checkbox_indicator`, `check_indicator`, `radio_indicator` |
| Item / Kbd | Standard `item_row`, parsed `kbd`, and typed `kbd_combo` |
| Layout | `center`, `flex_stack`, `form_layout`, `form_row`, `grid_span`, `hstack`, `layout`, `layout_content`, `layout_footer`, `layout_grid`, `layout_header`, `layout_panel`, `section`, `stack_item`, `vstack` |
| Lightbox / Link | `lightbox`, `use_lightbox`, `link` |
| List | `ui::item_list`, `ui::item_row`; `ListLook`, `ItemLook`, `ItemDensity`, `ItemAlign`, `ItemLayout`, `ListEdge` |
| Markdown | `markdown` |
| Metadata List | Standard `description_list` / `description`; share `MetadataLook` across list and pairs. |
| More Menu | `more_menu` |
| Navigation | `mobile_nav`, `mobile_nav_toggle`, `nav_heading_menu`, `nav_icon`, `side_nav`, `side_nav_collapse_button`, `side_nav_heading`, `side_nav_item`, `side_nav_section`, `top_nav`, `top_nav_heading`, `top_nav_item`, `top_nav_menu`, `top_nav_mega_menu`, `top_nav_mega_menu_item`, `top_nav_featured_card` |
| Number Input / Outline / Overflow List / Overlay / Pagination | Standard `number_input`, `outline`, `pagination`, `overflow_list`, `media_overlay` |
| Popover | Standard `dialog(...).kind(DialogKind::Popover)` and `kivact::view::overlay` |
| Power Search / Progress Bar | `power_search`, standard `progress` |
| Radio | `radio_list`, standard `ListItem` |
| Resizable | standard `splitter(...).collapsible(..).snaps(..)` |
| Rich Text Editor / Scrollable Area | `rich_text_editor`, `rich_text_toolbar`, `rich_text_view`, `scrollable_area`; `use_scroll_state`, `use_scroll_overflow`, and `use_overflow` observe scroll edges and fitting items |
| Segmented Control | Standard `radio_group(...).variant(RadioGroupVariant::Segmented)` and `ListItem`; Soft `SegmentedLook` (with `SegmentedLayout` and `icon_only`) |
| Selector | Standard `select(...).open_on_arrow(true)`, `selection_field(...).content(...)`, and `combobox(...).chips()`. |
| Skeleton / Slider / Spinner / Status Dot | Standard `skeleton`, standard `spinner`, `status_dot`, standard `slider` and `range_slider` |
| Stepper | Standard `stepper`, `StepperItem`, `StepIndicator`, `StepStatus`; Soft `StepperAppearance` |
| Switch | Standard `switch` |
| Table | Standard `table`, header/body/footer/row/cell builders, `data_table`, `DataColumn`, and `pagination`; Soft `TableRowAppearance`, `TableCellAppearance`, `table_selection_toolbar`, `table_row_index`, and `table_row_status`. |
| Tabs | `tab_list`, `tab`, `tab_menu`, standard `tabs` |
| Text through Timestamp | `styled_text`, `thumbnail`, `timer`, `timestamp`, standard `time_input`, `text_input` and `text_area` |
| Toast | Standard `toaster`, `Toast`, `toast_region` |
| Token / Tokenizer / Toolbar | `token`, standard `combobox(...).chips().exclude_selected()`, standard `toolbar`, Soft `with_key_hints` and `keyboard_hint` |
| Tooltip | Standard `tooltip`. The builder owns visibility and placement. |
| Tree List | `kivact::ui::tree` with `TreeLook` density and ancestor guides |
| Typeahead | Standard `combobox` and `ListItem`; configure `.search(..)`, `.debounce(..)`, `.placeholder(..)`, and `.empty_text(..)`. |
| VisuallyHidden | `visually_hidden` |
| Utilities | `theme`, `Tokens` (read with `kivact::style::use_tokens::<Tokens>()`), `media_theme`, `syntax_theme`, `kivact::platform::window_size`, and `kivact::platform::copy_text`. Internationalization Provider maps to `App::locale` and `App::messages`, Layer Provider to `kivact::view::overlay`, and Link Provider to `link_with` with an application callback. Kivact handles node references and diagnostics. |

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
| `soft.chat.tool_calls.complete` | Complete |
| `soft.chat.tool_calls.detail` | Details for {name} |
| `soft.chat.tool_calls.details` | Details |
| `soft.chat.tool_calls.error` | Error: {message} |
| `soft.chat.tool_calls.failed` | Failed |
| `soft.chat.tool_calls.group_label` | {count} tool calls |
| `soft.chat.tool_calls.pending` | Pending |
| `soft.chat.tool_calls.running` | Running |
| `soft.chat_dictation_button.label` | Dictation |
| `soft.citation.source` | Source {number} |
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
| `soft.mobile_nav.close` | Close navigation |
| `soft.mobile_nav.navigation` | Navigation |
| `soft.mobile_nav.open` | Open navigation |
| `soft.more_menu.label` | More options |
| `soft.outline.label` | Table of contents |
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
| `tree.toggle_children` | Toggle children |
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

Toast uses the standard queue. Pass `ToastOptions` to `ToasterApi::push_with(..)`
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
other sort keys. Use `.tree(..)` for hierarchical rows and `.expand_all_control()`
for the header control. `DataColumn::filter_field(..)` and
`.typed_column_filters(..)` provide draft popovers or inline filters.
Number, date, and time filter fields validate values entered as text.

| Area | Current behavior |
| --- | --- |
| Reveal motion | Collapsible, Side Nav, Code Block, and Chat Tool Calls use the shared content-height reveal. `kivact::ui::tree(..).reveal(2.0, transition)` clips closing branches as following rows move, approximates the spring with a 734.77 ms curve, and mounts fresh branch content on reopen. Native views can release their layout slot during an exit and receive a completion callback. Soft toast rows animate their height and spacing while the dismissed card leaves the focus and accessibility trees immediately. Toast dismissal callbacks run when dismissal starts. |
| Layout transitions | Layout changes reflow directly. Interrupted paint transitions use Kivact's native retargeting. |
| Touch gestures | Bottom Sheet uses shared drag velocity, axis, slop, release projection, snap, and overshoot calculations. It supplies its own detents and dismissal rules. Carousel snaps after native scrolling settles. Slider uses relative touch tracking and release coast. Native scrolling can take over a same-axis Slider drag inside a scroll view. Toast pauses its timer during touch or pen swipes. Switch uses touch lift and drag with one paint transition per phase. |
| Focus and menus | Disabled Soft buttons and menu rows can receive keyboard focus. Use `MenuEntry::disabled_reason(..)` to supply a tooltip for a disabled menu row. Menus support press and drag selection. Edge autoscroll remains pending. Submenus and navigation flyouts use timed hover and preserve diagonal mouse travel. |
| Rich text | Kivact can lay out and mount interactive views in a shaped plain-text paragraph. Soft Markdown still composes marked text and views separately. The rich text editor supports source, block, and preview modes. Inline marks remain visible as Markdown delimiters while editing. Native selections are local to a block, and complex blocks use source and preview modes. |
| Pickers and selectors | The application supplies the appearance. Date and time offer a desktop path and a touch sheet. Soft Selector opens from a bare arrow and retains trigger focus while highlighting an option in the popup. The compact touch sheet moves focus into its modal and returns it on close. MultiSelector and ComplexSelector use the popup path. |
| Text and forms | Text shows a hint when native shaping reports truncation. Citation and timestamp use solid underlines. Text supports measured cap-height trimming. `form_row(..)` places labels and fields in the shared tracks of a horizontal-label form. |
| Reduced motion | Soft respects Kivact's reduced-motion preference and the application's override. System detection depends on the platform. |
| Lists and overflow | The kit supplies its own static list. Overflow List measures mounted items and uses provisional widths before layout. |

## Migrating from the previous release

The kit now builds on the standard `kivact::ui` components: most kit
builders became standard components that the kit's `theme` draws in
its look, and the kit's looks are `.variant(..)` values. Previous
names and their replacements:

| Previous | Now |
| --- | --- |
| `Avatar`, `avatar`, `AvatarShape` | `kivact::ui::avatar(name)` (`ui::Avatar`); `.shape(ui::AvatarShape::..)`, `.src(..)`, `.status(Tone)`; size via `.variant(soft::AvatarSize::..)` or `.variant(soft::AvatarAppearance::new(size))`; anonymous: `.fallback_content(soft::avatar_silhouette)` |
| `AvatarGroup`, `avatar_group`, `AvatarGroupOverflow`, `avatar_group_overflow` | `ui::avatar_group(children).name(..).total(n).overflow_content(\|count\| ..)` (`ui::AvatarGroup`); `.variant(soft::AvatarGroupAppearance::new(size, shape))` |
| `AvatarTooltip` | Removed; `ui::tooltip(name).attach(ui::avatar(name))` (Text: other tooltip text; Off: no tooltip) |
| `Badge`, `badge` | `ui::badge(label).tone(Tone::..)` (`ui::Badge`); categorical colors `.variant(soft::BadgeVariant::..)` |
| `Banner`, `banner`, `BannerStatus` | `ui::alert(title).tone(Tone::Info\|Success\|Warning\|Danger).icon(soft::banner_icon(tone))` (Error -> `Tone::Danger`); `.actions(..)`, `.content(..).collapsible(true)`; `.variant(soft::BannerAppearance::new(container, elevation))` |
| `BreadcrumbItem`, `breadcrumb_item`, `Breadcrumbs`, `breadcrumbs` | `ui::breadcrumb(Vec<ListItem<K>>)` (`ui::Breadcrumb`); `.current(..)`, `.on_activate(..)`, `.item(..)`, `.separator(..)`; `.variant(soft::BreadcrumbsVariant::..)` |
| `ButtonGroup`, `button_group`, `GroupMember`, `GroupMembers` | `ui::button_group(label, children)` (`ui::ButtonGroup`); member overrides `ui::ButtonGroupMember::from(button).solid(..).keep_size(..)`; `.variant(soft::ButtonGroupVariant(soft::Elevation::..))` |
| `Calendar`, `calendar`, `calendar_range` | `ui::calendar(selection)` (`ui::Calendar`) bound to `Option<CalendarDate>` or `Option<DateRange>` (`kivact::i18n`) |
| `Card`, `card`, `clickable_card`, `selectable_card` | `ui::card(content)` (`ui::Card`); clickable `.name(..).on_press(..)`; selectable `.name(..).selected(binding)`; `.variant(soft::CardVariant::..)` or `soft::CardAppearance::new(..)`, `soft::with_card_insets` |
| `Carousel`, `carousel`, `CarouselHandle`, `use_carousel` | `ui::carousel(index, slides).name(label)` (`ui::Carousel`), `.mode(ui::CarouselMode::Scroll)`, `.snap(ui::Snap::..)`; headless `kivact::interaction::use_carousel(index, count)` -> `CarouselApi` |
| `ChatBubble`, `chat_bubble`, `BubbleGroup`, `ChatMetadata`, `chat_metadata`, `MessageStatus` | `ui::chat_message(author).content(..)`; `.group(ui::ChatBubbleGroup::..)`, `.metadata(..)`, `.time(..)`, `.status(ui::ChatMessageStatus::..)`; `.variant(soft::BubbleVariant::..)` |
| `ChatComposer`, `chat_composer`, `ComposerStatus`, `ComposerStatusPosition` | `ui::chat_composer(value)` (`ui::ChatComposer`); `.status(ui::ComposerStatus::..)`, `.status_position(ui::ComposerStatusPosition::..)`; `.variant(soft::ComposerElevation::..)` or `soft::ChatAppearance` |
| `ChatComposerDrawer`, `chat_composer_drawer` | `ui::chat_composer(..).drawer(ui::disclosure(title, open).content(..))` |
| `chat_composer_input` | `ui::text_area(label, value)` |
| `ChatLayout`, `chat_layout`, `ChatListAlign`, `ChatMessageList`, `chat_message_list`, `chat_scroll_button` | `ui::chat_thread(for_each(..))` (`ui::ChatThread`); `.composer(..)`, `.align(ui::ChatAlign::Start\|End)`, `.scroll_button(true)`, `.history_revision(..)`, `.empty(..)`, `.loading_top(..)` |
| `ChatMessage`, `chat_message`, `ChatSender` | `ui::chat_message(author).sender(ui::ChatSender::..).content(..)` (`ui::ChatMessage`) |
| `ChatSendButton`, `chat_send_button` | Built into `ui::chat_composer` (`.on_submit(..)`, `.send_icon(..)`, `.send_actions(..)`); standalone: `button(..)` |
| `ChatSystemMessage`, `chat_system_message` | `ui::chat_message(..).sender(ui::ChatSender::System)`; `.variant(soft::SystemMessageVariant::..)` |
| `CodeBlock`, `code_block`, `SyntaxTheme` | `ui::code_block(code)` (`ui::CodeBlock`) with `.highlight(palette, \|line, palette\| soft::code_highlight(&language, line, palette))`; `.variant(soft::CodeBlockSize\|CodeBlockWidth\|CodeBlockContainer\|CodeAppearance)`; `SyntaxTheme` -> `soft::SyntaxColors` |
| `Collapsible`, `collapsible`, `collapsible_default` | `ui::disclosure(title, open).content(..).animated(true)` (`ui::Disclosure`); default-open: pass `signal(true)`; `.variant(soft::DisclosureVariant { chevron, density, dividers })` |
| `CollapsibleConfig`, `CollapsibleState`, `use_collapsible` | `kivact::interaction::use_disclosure(open)` -> `DisclosureApi`; custom trigger: `ui::disclosure(..).content(..).attach(trigger)` |
| `CollapsibleGroup`, `collapsible_group`, `GroupMode` | `ui::disclosure_group(selection).panel(key, title, factory)`; `Option<K>` binding = single, `Vec<K>` = multiple |
| `DateInput`, `date_input`, `DatePresentation`, `DateRangeInput`, `date_range_input`, `DateRangePreset`, `DateTimeInput`, `date_time_input`, `TimeInput`, `time_input` | Same names in `kivact::ui` (builders also in prelude); `.presentation(ui::DatePresentation::..)`, `.presets(Vec<ui::DateRangePreset>)` |
| `DateTime`, `TimeOfDay` | `kivact::i18n::DateTime`, `kivact::i18n::TimeOfDay` |
| `EmptyState`, `empty_state` | `ui::empty_state(title).description(..).icon(..).actions(..)` (`ui::EmptyState`); `.announce_changes()`; `.variant(soft::EmptyStateVariant::compact(..))` |
| `FileInput`, `file_input` | `ui::dropzone(label).pick_on_press().on_files(..)`; `.multiple()`, `.filters(..)`, `.accept_drop(false)` for picker-only; `.variant(soft::DropzoneVariant::Field)` |
| `Grid`, `grid` | `soft::LayoutGrid`, `soft::layout_grid(children)` |
| `GridSpan` (struct), `Span` | `soft::GridSpanItem` (returned by `soft::grid_span`); the span enum is now `soft::GridSpan::{Cols, Full}` |
| `Heading`, `heading` | `ui::heading(value, level)` (`ui::Heading`; argument order swapped); display size `.variant(soft::HeadingDisplay::D1\|D2\|D3)` |
| `HoverCard`, `hover_card`, `FocusTrigger` | `ui::hover_card(factory).attach(trigger)` (`ui::HoverCard`); `.focus_trigger(ui::FocusTrigger::..)` |
| `Icon`, `icon` | `soft::IconView`, `soft::icon_view(source).size(..).color(..)`; unstyled: `kivact::view::icon` |
| `InputGroup`, `input_group`, `input_group_with`, `InputGroupItems`, `input_group_text` | `ui::input_group(label, children)`, `ui::input_group_with(label, factory)` (`ui::InputGroup`); `input_group_text` -> `ui::input_addon(label)` |
| `Instant` | `soft::TimestampValue` (`unix_seconds`, `unix_millis`, `parse_iso`) |
| `Item`, `item`, `list_item` | `ui::item_row(title)` (`ui::ItemRow`); `.description(..)`, `.media(..)`, `.actions(..)`, `.delegate_to(..)`; `.variant(soft::ItemLook\|ItemDensity\|ItemAlign\|ItemLayout)` |
| `kbd` | `ui::kbd("mod+shift+p")` or typed `ui::kbd_combo(keys)` |
| `List`, `list`, `list_items`, `IntoListItems`, `ListDensity`, `ListMarker` | `ui::item_list(children)` (`ui::ItemList`; keyed: `for_each` + `item_row(..).position(index, count)`); `.marker(ui::ItemListMarker::..)`, `.start(n)`; `.variant(soft::ListLook)`; `ListDensity` -> `soft::ItemDensity` |
| `MetadataItem`, `metadata_item`, `MetadataList`, `metadata_list`, `IntoMetadataItems` | `ui::description_list(children)`, `ui::description(term, detail)`; same `.variant(soft::MetadataLook)` on list and pairs |
| `Motion`, `Radius`, `Spacing`, `TextStyle`, `Typography` | `soft::MotionTokens`, `soft::RadiusTokens`, `soft::SpaceTokens`, `soft::TypeStyle`, `soft::TypeTokens` |
| `MultiSelector`, `multi_selector` | `ui::combobox(label, options, Vec<K> binding).chips()` |
| `Pagination`, `pagination`, `PaginationVariant`, `table_pagination` | `ui::pagination(count, page)` (zero-based `usize`; `ui::Pagination`); `.mode(ui::PaginationMode::..)`, `.window(ui::PaginationWindow::Buffered)`, `.summary(soft::pagination_status)`; `.variant(soft::PaginationAppearance::new(size))` |
| `PillPlacement`, `ResizableConfig`, `ResizableState`, `ResizeHandle`, `resize_handle`, `ResizeSize`, `use_resizable` | `ui::splitter(name, size, ScalarRange)` (`ui::Splitter`) with `.collapsible(..)`, `.snaps(..)`, `.handle(..)`; headless `kivact::interaction::use_splitter(value, range, axis)` |
| `Segmented`, `SegmentedControl`, `segmented_control`, `SegmentedIconOnly` | `radio_group(label, options, value).variant(ui::RadioGroupVariant::Segmented)` + `.variant(soft::SegmentedLook { layout, icon_only })` |
| `Selector`, `selector` | `ui::select(label, options, selected).open_on_arrow(true)` |
| `SelectorPresentation`, `complex_selector`, `complex_selector_with_presentation` | `ui::selection_field(label, value_label).content(factory).presentation(ui::SelectPresentation::..)` |
| `Skeleton`, `skeleton` | `ui::skeleton()` (`ui::Skeleton`); `.shape(ui::SkeletonShape::..)`, `.lines(n)`, `.index(n)`; `.variant(soft::SkeletonRadius::..)` |
| `Spinner`, `spinner`, `SpinnerSize` | `ui::spinner().size(ControlSize::ExtraSmall\|Small\|Medium\|Large)` (= Sm/Md/Lg/Xl) (`ui::Spinner`); `.variant(soft::SpinnerVariant { shade, ink })`, `.label_content(..)` |
| `Stack`, `stack` | `soft::FlexStack`, `soft::flex_stack(children)` |
| `Step`, `step`, `StepIndicator`, `StepStatus`, `Stepper`, `stepper` | `ui::stepper(vec![ui::StepperItem::new(label)], current)` (`ui::Stepper`; argument order swapped); `.status(ui::StepStatus::..)`, `.indicator(ui::StepIndicator::custom(..))`; `.variant(soft::StepperAppearance)` |
| `table`, `table_body`, `table_cell`, `table_footer`, `table_head`, `table_header`, `table_row`, `TableCell`, `TableRow` | Same builders in `kivact::ui` (`ui::Table`, `ui::TableCell`); `.variant(soft::TableRowAppearance::new(dividers, hover, striped))`; selected row `.with_node(\|n\| n.selected(..))` |
| `TableAlign`, `TableDensity` | `ui::TableCell::align(kivact::style::TextAlign::..)`; density `soft::ItemDensity` via `.variant(soft::TableCellAppearance::new(density, vertical_align, divider))` |
| `Text`, `text` | `soft::StyledText`, `soft::styled_text(value)` (prelude `text` is the unstyled primitive) |
| `kivact::interaction::Toaster` | `kivact::interaction::ToasterApi`, the handle `toaster()` returns |
| `ToggleButtonGroup`, `toggle_button_group`, `ToggleSelection` | `ui::toggle_group(label, options, selection)` (`ui::ToggleGroup`); `Option<K>` binding = single, `Vec<K>` = multiple; `ListItem::with_pressed_icon(..)`, `.icon_only(..)` |
| `tokens` | `kivact::style::use_tokens::<soft::Tokens>()` |
| `Tokenizer`, `tokenizer` | `ui::combobox(label, options, Vec<K> binding).chips().exclude_selected()` |
| `Toolbar`, `toolbar`, `Edge` | `ui::toolbar(label).start(..).center(..).end(..)` (`ui::Toolbar`); `.dividers(&[ui::ToolbarEdge::..])`; `.variant(soft::ToolbarVariant(soft::SectionVariant::..))` |
| `toolbar_item` | `soft::keyboard_hint(target)` inside `soft::with_key_hints(axis, \|\| ..)` |
| `TopNavHeading`, `TopNavMenuItem` | `soft::SideNavHeading`, `soft::NavHeadingMenuItem` (still returned by `top_nav_heading`, `top_nav_menu_item`) |
| `TreeDensity`, `TreeList`, `tree_list`, `TreeVariant` | `ui::tree(label, nodes).expanded(expanded)` (`ui::Tree`); `.variant(soft::TreeLook::default().density(soft::ItemDensity::..).guides(bool))` |
| `Typeahead`, `typeahead`, `base_typeahead` | `ui::combobox(label, options, selected)` (`ui::Combobox`); `.search(..)`, `.debounce(..)`, `.placeholder(..)`, `.empty_text(..)` |

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
- `table`
- `chat_thread`
- `chat_message`
- `chat_composer`
- `code_block`
- `breadcrumb`
- `empty_state`
- `input_group`
- `card`
- `avatar`
- `avatar_image`
- `avatar_status`
- `avatar_status_content`
- `avatar_group`
- `avatar_group_member`
- `avatar_group_overflow`
- `stepper`
- `alert`
- `pagination`
- `item_list`
- `navigation_menu`
<!-- /kivact-recipes -->

Pagination presentation helpers: `PaginationAppearance`, `pagination_count`, `pagination_page_summary`, and `pagination_status` use actual canonical summary facts.
