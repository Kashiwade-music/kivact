# kivact-ui-pop

A Kivact UI kit for compact desktop business applications: calm slate
surfaces, narrow blue gradients and yellow accents.

The kit depends only on the public `kivact` crate. Its theme restyles the
standard components of `kivact::ui`, and it adds business components such as
stat cards, timelines and
conversation threads.

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

## Filled recipes

The theme fills these entries in `UiRecipes`:

<!-- kivact-recipes -->
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
- `list`
- `data_table`
- `progress`
- `disclosure`
- `command_palette`
- `tag_input`
- `number_input`
<!-- /kivact-recipes -->

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
