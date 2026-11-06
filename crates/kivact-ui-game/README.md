# kivact-ui-game

A Kivact UI kit for dense, hard-edged operation consoles, game control panels
and HUDs.

The kit depends only on the public `kivact` crate. Its theme restyles the
standard components of `kivact::ui` (slanted buttons, diamond radios,
parallelogram slider thumbs), and it adds console components such as gauges,
KPIs, file trees, banners and activity logs.

```rust,no_run
use kivact::prelude::*;
use kivact::style::{ThemeMode};
use kivact_ui_game::{Density, Slant, ThemeOptions, theme_with};

fn screen() -> impl IntoView {
    let approved = signal(false);
    column((
        checkbox("Approved", approved),
        button("Approve")
            .variant(Slant)
            .on_press(move || approved.set(true)),
    ))
    .style(style().gap(12).padding(20))
}

fn main() -> kivact::Result<()> {
    App::new("Operations")
        .theme(theme_with(ThemeMode::Light, ThemeOptions { density: Density::Compact }))
        .run(screen)
}
```

- **Theme.** `theme(mode)` returns a plain `Theme` for `App::theme` or
  `provide_theme`. Use `theme_with(mode, ThemeOptions { density, ..Default::default() })`
  to choose the density.
  The kit's tokens are `Tokens`; icons are the bundled Tabler set, drawn with
  `kivact::view::icon("name")`.
- **Looks of standard components.** `radio_group(..).variant(kivact::ui::RadioGroupVariant::Segmented)` or
  `.variant(ChoiceCard)`, `tabs(..).variant(Boxed)` and
  `tabs(..).variant(Navigation)` give the kit's other looks.
- **State.** Components take their values as signals or bindings owned by the
  application. Core builders expose their controlled bindings and action callbacks;
  Game recipes supply their visual treatment.
- **Texts.** Default texts are English; replace them with `App::messages`.

- **Reading direction.** Standard components inherit the Core reading direction and
  automatically reverse their rows where appropriate; the Game theme supplies visual recipes.

## Migrating from the previous release

The kit now builds on the standard `kivact::ui` components: most kit
builders became standard components that the kit's `theme` draws in
its look, and the kit's looks are `.variant(..)` values. Previous
names and their replacements:

| Previous | Now |
|---|---|
| `alert`, `Alert` (`.on_dismiss`, `.actions` closure) | `kivact::ui::alert(title).description(..).tone(..).actions(view).dismissible(true).on_dismiss(..)` |
| `avatar`, `Avatar` (`.size`, `.color`, `.online`) | `kivact::ui::avatar(name).variant(game::AvatarVariant { side, color }).status(Option<Tone>)` |
| `avatar_more(count)` | `kivact::ui::avatar_group(avatars).total(n).max(m)`, which draws the overflow count (a custom count view: `.overflow_content(..)`) |
| `avatar_stack(Vec<(String, Color)>)` | `kivact::ui::avatar_group(..)` of `kivact::ui::avatar(..).variant(game::AvatarVariant { side, color })` |
| `badge(label, tone)` | `kivact::ui::badge(label).tone(tone)` (the default look is `game::BadgeVariant::Dot`) |
| `banner(title, tone, action)`, `Banner` | `kivact::ui::alert(title).variant(kivact::ui::AlertVariant::Banner).tone(tone).actions(action).content(..)` |
| `breadcrumb(Vec<String>)` | `kivact::ui::breadcrumb(Vec<kivact::interaction::ListItem<K>>)` |
| `button_group(children)` | `kivact::ui::button_group(label, children)` |
| `Color` | `game::ColorName` |
| `dropzone`, `Dropzone` | `kivact::ui::dropzone(label)`; `.hint(..)` and `.filters(..)` unchanged; `.on_files(..)` gets `Vec<kivact::interaction::FileInfo>` |
| `empty_state`, `EmptyState` (`.content`) | `kivact::ui::empty_state(title).description(..).icon(..).actions(view)` |
| `gauge(label, value).total(n).tone(t)`, `Gauge` | `kivact::ui::progress(Some(value / total)).name(label).variant(kivact::ui::ProgressVariant::Gauge).variant(game::GaugeSegments(total)).variant(tone)` |
| `input_adorned(prefix, control, suffix)` | `kivact::ui::input_group(label, (control,)).variant(game::GameInputGroupVariant::Adorned).start(prefix).end(suffix)` |
| `input_group(prefix, control, suffix)`, `input_group_content(..)` | `kivact::ui::input_group(label, (control,)).variant(game::GameInputGroupVariant::Inline).start(..).end(..)`; plain text cells are `kivact::ui::input_addon(text)` |
| `kpi(label, target)` (`.heading(closure)`) | Same function, drawn as a standard `kivact::ui::stat`; `.heading(view)` takes a view |
| `pagination(page, count)` | `kivact::ui::pagination(count, page)` (arguments swapped); custom item views with `.item(..)` |
| `property_list(Vec<(String, V)>)` | `kivact::ui::description_list((kivact::ui::description(term, view), ..))` |
| `ring(label, value, tone)` | `kivact::ui::progress(Some(fraction)).name(label).variant(kivact::ui::ProgressVariant::Ring).variant(tone)` |
| `section_heading(number, title, description)` | Removed: a row of the number `text(..)`, `kivact::ui::heading(title, 2).variant(game::HeadingVariant::Section)` and a description `text(..)` |
| `Segmented` | `kivact::ui::radio_group(..).variant(kivact::ui::RadioGroupVariant::Segmented)` |
| `skeleton`, `Skeleton` (`.rows(n)`) | `kivact::ui::skeleton().variant(game::SkeletonVariant::Rows).lines(n)` |
| `stepper(labels, current)` | `kivact::ui::stepper(Vec<kivact::ui::StepperItem>, current)` |
| `TableCell` | `game::GameCellVariant` (the same cases), returned from `DataColumn::cell` |
| `tag_badge(label, tone, variant)` | `kivact::ui::badge(label).tone(tone).variant(game::BadgeVariant::..)` |
| `theme(mode).density(d)` (`ThemeOptions` trait) | `game::theme_with(mode, game::ThemeOptions { density })` (`ThemeOptions` is a struct) |
| `toolbar(children)` | `kivact::ui::toolbar(label).start(..).center(..).end(..)` |
| `tree(entries, selection)`, `TreeEntry` | `kivact::ui::tree(label, nodes)` of `kivact::interaction::TreeNode::new(key, parent, label)`, with `.selection(..)`, `.expanded(..)` and `.marker_content(..)` |

## Documentation

- Tutorial and how-to guides: [English](../../docs/for-kivact-ui-user/en/game/getting-started/README.md),
  [日本語](../../docs/for-kivact-ui-user/jp/game/getting-started/README.md)
- API reference: `cargo doc -p kivact-ui-game --open`
- Catalog: `cargo run -p kivact-ui-game-catalog` from the repository root

## License

MIT OR Apache-2.0. The bundled Tabler icons are MIT
([LICENSE-tabler-icons.txt](assets/LICENSE-tabler-icons.txt)).

| Slot kind | API |
|---|---|
| Mounted once: headers, media, actions, footers and other display content | `impl IntoView`, stored as `View` |
| Recreated when opened: dialogs, drawers, collapsibles and hover cards | `impl IntoViewFn<V>`, stored as `ViewFn` |
| Per-item content | `impl Fn(A) -> V`, stored as `ViewFn<A>` |
| Individually wrapped children | `impl IntoChildren` |

The recipe files follow the same categories as Pop: base controls in `recipes.rs`,
selection and overlay geometry in `recipes_extra.rs`, and dedicated avatar, badge,
collection, feedback, field, file, group, indicator, input, item and palette leaves.
Game-specific metric, navigation and segmented progress geometry have their own leaves.
The catalog reads byte-identical Noto Sans JP and its OFL notice from
[`examples/shared-assets/fonts`](../../examples/shared-assets/fonts/SOURCE.md).

## Filled recipes

<!-- kivact-recipes -->
- `pagination`
- `breadcrumb`
- `stepper`
- `navigation_menu`
- `toolbar`
- `toggle_group`
- `button_group`
- `input_group`
- `dropzone`
- `tree`
- `table`
- `item_list`
- `badge`
- `badge_label`
- `avatar`
- `avatar_status`
- `avatar_group`
- `avatar_group_overflow`
- `skeleton`
- `empty_state`
- `alert`
- `heading`
- `stat`
- `description_list`
- `field`
- `button`
- `button_content`
- `button_busy`
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
- `progress`
- `disclosure`
- `command_palette`
- `tag_input`
- `number_input`
<!-- /kivact-recipes -->
