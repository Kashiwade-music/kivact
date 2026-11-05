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
  application. Collection components report requests (`on_add`, `on_remove`,
  `on_dismiss`); the application applies or rejects them.
- **Texts.** Default texts are English; replace them with `App::messages`.

## Documentation

- Tutorial and how-to guides: [English](../../docs/for-kivact-ui-user/en/game/getting-started/README.md),
  [譌･譛ｬ隱枉(../../docs/for-kivact-ui-user/jp/game/getting-started/README.md)
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
