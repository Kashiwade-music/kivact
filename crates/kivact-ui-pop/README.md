# kivact-ui-pop

A Kivact UI kit for compact desktop business applications: calm slate
surfaces, narrow blue gradients and yellow accents.

The kit depends only on the public `kivact` crate. Its theme restyles the
standard components of `kivact::ui`, and adds timelines, scheduling rows and
conversation-preview rows.

```rust,no_run
use kivact::prelude::*;
use kivact::style::{ThemeMode};
use kivact_ui_pop as pop;

fn settings() -> impl IntoView {
    let sync = signal(true);
    let name = signal(String::new());
    kivact::ui::card(
        column((
            field("Display name", text_input("Display name", name)),
            switch("Sync reports", sync),
            button("Save").on_press(move || {
                // Persist through the application's own model or service.
            }),
        ))
        .style(style().gap(16)),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Settings")
        .theme(pop::theme(ThemeMode::Light))
        .run(settings)
}
```

- **Theme.** `pop::theme(ThemeMode::Light | Dark)` returns a plain `Theme` for
  `App::theme` or `provide_theme`. The kit's tokens are `pop::Tokens`; icons are
  the bundled Tabler set, drawn with `kivact::view::icon("name")`.
- **Fonts.** The application registers fonts. The catalog uses Inter, Barlow
  and Noto Sans JP.
- **State.** Components take their values as signals or bindings owned by the
  application. Collection components report `on_add` / `on_remove` requests;
  the application applies or rejects them.
- **Texts.** Default texts are English message keys prefixed with `pop.`;
  replace them with `App::messages`.

Pop uses Core row layout for automatic RTL reversal; recipes do not duplicate direction branches.

## Filled recipes

The theme fills these entries in `UiRecipes`:

<!-- kivact-recipes -->
- `chat_message`
- `chat_thread`
- `chat_composer`
- `typing_indicator`
- `code_block`
- `stat`
- `sparkline`
- `heading`
- `description_list`
- `breadcrumb`
- `pagination`
- `stepper`
- `navigation_menu`
- `item_list`
- `table`
- `tree`
- `calendar`
- `date_input`
- `date_range_input`
- `kbd`
- `empty_state`
- `alert`
- `card`
- `input_group`
- `avatar`
- `badge`
- `badge_label`
- `skeleton`
- `spinner`
- `toolbar`
- `toggle_group`
- `button_group`
- `dropzone`
- `field`
- `button`
- `button_icon`
- `button_content`
- `button_busy`
- `toggle`
- `checkbox`
- `checkbox_indicator`
- `checkbox_check`
- `checkbox_mixed`
- `switch`
- `switch_track`
- `switch_thumb`
- `splitter`
- `split`
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
- `list`
- `data_table`
- `progress`
- `disclosure`
- `command_palette`
- `tag_input`
- `number_input`
<!-- /kivact-recipes -->

## Migrating from the previous release

The kit now builds on the standard `kivact::ui` components: most kit
builders became standard components that the kit's `theme` draws in
its look, and the kit's looks are `.variant(..)` values. Previous
names and their replacements:

| Previous | Now |
|---|---|
| `alert`, `Alert` | `kivact::ui::alert(title)`, with `.description(..)`, `.tone(kivact::interaction::Tone)`, `.icon(view)` and `.actions(view)` (a view now, not a closure) |
| `avatar`, `Avatar` (`.size`, `.hue`, `.halo`, `.online`) | `kivact::ui::avatar(name).variant(pop::AvatarLook::default().size(..).hue(..).halo(..).online(..))`; for the halo and presence dot also add `.status_content(look.decorations())` |
| `badge`, `Badge` (`.hue`, `.variant`) | `kivact::ui::badge(label).variant(pop::BadgeHue::new(hue)).variant(pop::BadgeVariant::..)` |
| `breadcrumb(Vec<String>)` | `kivact::ui::breadcrumb(Vec<kivact::interaction::ListItem<K>>)`, with `.separator(..)`, `.current(..)` and `.on_activate(..)` |
| `calendar(selected, today)`, `Calendar` | `kivact::ui::calendar(selected).today(day)`; `week_starts_on` -> `.week_start(..)`, `year_month_menus(true)` -> `.caption(kivact::ui::CalendarCaption::Dropdown)`, a Pop-hue `weekend(..)` -> `.variant(pop::WeekendHue::new(..))` |
| `CalendarEvent` | `kivact::ui::CalendarEvent { date, color }`, passed with `calendar(..).events(..)` |
| `Color` | `pop::ColorName` |
| `date_input(label, value, today)`, `DateInput` | `kivact::ui::date_input(label, binding)` over `Option<CalendarDate>`; today is set with `.calendar(\|c\| c.today(..))` |
| `date_range_input((start_label, end_label), (start, end), today)`, `DateRangeInput` | `kivact::ui::date_range_input(label, binding)` over `Option<kivact::i18n::DateRange>`, with `.start_label(..)`, `.end_label(..)` and `.calendar(..)` |
| `dots_loader` | `kivact::ui::spinner().variant(pop::SpinnerLook::Dots)` |
| `dropzone`, `Dropzone` | `kivact::ui::dropzone(label)`; `.hint(..)` and `.filters(..)` unchanged; `.on_files(..)` gets `Vec<kivact::interaction::FileInfo>`; new `.multiple()`, `.pick_on_press()`, `.icon(..)` |
| `empty_state`, `EmptyState` (`.description`, `.content`) | `kivact::ui::empty_state(title).description(..).actions(view)`; the Pop artwork is `.icon(pop::empty_illustration())` |
| `key_value(key, value)` | `kivact::ui::description(term, view)` inside `kivact::ui::description_list((..))` |
| `list_item`, `ListItem` (`.subtitle`, `.icon`, `.tone`, `.trailing`) | `kivact::ui::item_row(title).description(..).media(icon(..)).variant(pop::ItemHue::new(hue)).actions(view)`, inside `kivact::ui::item_list(..)` |
| `message`, `Message` (`.timestamp`, `.mine`, `.initials`) | `kivact::ui::chat_message(author).content(text(..)).time(..).sender(kivact::ui::ChatSender::User)` (or `Assistant`) `.avatar(..).variant(pop::PopBubble)` |
| `message_group`, `MessageGroup` | One `kivact::ui::chat_message` per line with `.group(kivact::ui::ChatBubbleGroup::..)` and `.variant(pop::PopBubble)`, inside `kivact::ui::chat_thread(..)` |
| `notification_menu(trigger, items, mark_read)` | `pop::notification_menu(trigger, items).on_mark_read(..)` (returns `pop::NotificationMenu`) |
| `panel` | `kivact::ui::card(children)` |
| `progress_ring(value, label, hue)` | `kivact::ui::progress(Some(fraction)).name(label).variant(kivact::ui::ProgressVariant::Ring).variant(pop::ProgressHue(hue.into_value()))` |
| `removable_chip(label, remove)` | `pop::removable_chip(label).on_remove(..)` (returns `pop::RemovableChip`) |
| `section_heading(number, title, subtitle)` | Removed: a row of `kivact::ui::badge(number).variant(pop::SectionNumber)`, `kivact::ui::heading(title, 2)` and a muted `text(subtitle)` |
| `Segmented` | `kivact::ui::radio_group(..).variant(kivact::ui::RadioGroupVariant::Segmented)` |
| `skeleton`, `Skeleton` (`.rows(n)`) | `kivact::ui::skeleton().lines(n)`, with `.shape(kivact::ui::SkeletonShape::..)` |
| `spinner` | `kivact::ui::spinner()`, with `.size(kivact::ui::ControlSize::..)` |
| `stat_card`, `StatCard` (`.trend`, `.icon`, `.points`, `.tone`) | `kivact::ui::stat(label, value).delta(text).icon(view).chart(kivact::ui::sparkline(points)).variant(pop::StatHue::new(hue))` |
| `stepper(labels, current)`, `Stepper` (`.notes`) | `kivact::ui::stepper(items, current)` with `kivact::ui::StepperItem::new(label).description(note)` and `.variant(pop::DiamondSteps)` |
| `typing_message(initials)` | `kivact::ui::typing_indicator().avatar(kivact::ui::avatar(initials).variant(pop::AvatarLook::default().size(24)))` |

## Documentation

- Tutorial and how-to guides: [English](../../docs/for-kivact-ui-user/en/pop/getting-started/README.md),
  [日本語](../../docs/for-kivact-ui-user/jp/pop/getting-started/README.md)
- API reference: `cargo doc -p kivact-ui-pop --open`
- Catalog: `cargo run -p kivact-ui-pop-catalog` from the repository root

## License

MIT OR Apache-2.0. The bundled Tabler icons are MIT
([LICENSE-tabler-icons.txt](assets/LICENSE-tabler-icons.txt)).

| Slot kind | API |
|---|---|
| Mounted once: headers, media, actions, footers and other display content | `impl IntoView`, stored as `View` |
| Recreated when opened: dialogs, drawers, collapsibles and hover cards | `impl IntoViewFn<V>`, stored as `ViewFn` |
| Per-item content | `impl Fn(A) -> V`, stored as `ViewFn<A>` |
| Individually wrapped children | `impl IntoChildren` |
