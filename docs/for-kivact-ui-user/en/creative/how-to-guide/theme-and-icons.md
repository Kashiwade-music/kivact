# Theme the Creative kit, set fonts and add icons

[日本語](../../../jp/creative/how-to-guide/theme-and-icons.md)

Goal: install the kit's theme, pick an accent color and density, switch light and dark at run time, use your own fonts, add your own icons and translate the texts the kit shows itself.

The kit's look is a plain Kivact `Theme`. Everything, including the standard components from `kivact::ui` (buttons, text fields, menus, dialogs), takes its appearance from it. This guide assumes you finished the [tutorial](../getting-started/README.md). The theme system itself is explained in [Change the theme and the look of components](../../../../for-kivact-user/en/how-to-guide/theming.md).

## Install the theme

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

fn root() -> impl IntoView {
    text("Hello, creative tools")
}

fn main() -> kivact::Result<()> {
    App::new("Creative app")
        .theme(ui::theme(ThemeMode::Dark))
        .run(root)
}
```

`ui::theme(mode)` is a dark (or light) theme in the *Iris* accent with comfortable spacing. It also installs the kit's recipes, so the standard components match the kit's own, and an icon set containing every icon the kit draws. It needs no font files: text uses the system `sans-serif` and `monospace` families.

## Choose an accent and a density

Import the `ThemeOptions` trait (as `_`, because you only need its methods) and chain:

```rust,no_run
use kivact::style::{Theme, ThemeMode};
use kivact_ui_creative::{self as ui, ThemeOptions as _};

fn compact_lime() -> Theme {
    ui::theme(ThemeMode::Dark)
        .accent(ui::Accent::Lime)
        .density(ui::Density::Compact)
}
```

`Accent` has `Iris`, `Lime`, `Aqua`, `Coral` and `Amber`. The accent colors the selection, the focus ring and active controls. `Density::Compact` reduces the control height from 24 to 20 pixels and tightens gaps. Each call keeps everything else the theme already had.

## Switch theme while the app runs

`App::theme` is fixed for the app. To change the theme at run time, wrap the part of the UI that should change in `provide_theme` with a closure that reads signals; it re-evaluates when they change.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::view::provide_theme;
use kivact_ui_creative::{self as ui, ThemeOptions as _};

fn themed() -> impl IntoView {
    let dark = signal(true);
    let accent = signal(ui::Accent::Iris);

    provide_theme(
        move || {
            let mode = if dark.get() { ThemeMode::Dark } else { ThemeMode::Light };
            ui::theme(mode).accent(accent.get())
        },
        move || {
            column((
                kivact::ui::switch("Dark mode", dark),
                row((
                    button("Iris").on_press(move || accent.set(ui::Accent::Iris)),
                    button("Lime").on_press(move || accent.set(ui::Accent::Lime)),
                    button("Coral").on_press(move || accent.set(ui::Accent::Coral)),
                ))
                .style(style().gap(6)),
            ))
            // `provide_theme` makes no node of its own, so color the top node.
            .style(
                style()
                    .gap(10)
                    .padding(16)
                    .background(ui::Color::Canvas)
                    .color(ui::Color::Text),
            )
        },
    )
}
```

`ui::Color` is the kit's own palette as a style value (`Canvas`, `Island`, `Header`, `Field`, `Hair`, `Text`, `Muted`, `Accent` and more). Anywhere a style takes a color it follows theme switches by itself, with no closure that reads the theme.

## Use your own fonts

The default families need no setup. To use bundled fonts, do two things: register the font data with the app, and tell the theme the family names.

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::theme::Typography;
use kivact_ui_creative as ui;

/// `font` is the bytes of a font file, for example
/// `include_bytes!("../assets/Geist[wght].ttf")`.
fn app_with_font(font: &'static [u8]) -> App {
    let theme = ui::theme(ThemeMode::Dark);

    // The kit's tokens: change the families and put them back.
    let mut tokens = (*theme.get::<ui::Tokens>()).clone();
    tokens.body_font = "Geist".into();
    tokens.mono_font = "Geist Mono".into();

    // The core text style follows the body family, too.
    let text = Typography {
        family: Some(tokens.body_font.clone()),
        ..theme.text().clone()
    };
    let theme = theme.with_text(text).with(tokens);

    App::new("Fonts").font(font).theme(theme)
}
```

`body_font` is used for labels and `mono_font` for numbers and codes. The family name must be the name stored inside the font file. Call `.font(..)` once for each font file you use. The kit's catalog does exactly this with Geist, Geist Mono and Noto Sans JP, as a reference.

## Draw and add icons

An icon is a name in the theme's icon set. The kit's theme includes 16-pixel Tabler outline icons such as `play`, `pause`, `eye`, `eye-off`, `lock`, `folder`, `stack`, `layout-columns`, `adjustments`, `keyframes`, `palette`, `color-picker`, `chevron-down`, `plus`, `trash`, `search` and `x`. Draw one with `icon(name)`; it takes the surrounding text color.

```rust,no_run
use kivact::Asset;
use kivact::control::icon;
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

static NODE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3l9 9-9 9-9-9z"/></svg>"#;

fn main() -> kivact::Result<()> {
    // Your own image under a name of your choice.
    let theme = ui::theme(ThemeMode::Dark).with_icon("node", Asset::from(NODE));
    App::new("Icons").theme(theme).run(|| {
        row((icon("play"), icon("node"), text("Play the node")))
        .style(style().gap(8).padding(12))
    })
}
```

Names also appear as arguments in the kit's components: `DockTab::icon("palette")`, `TreeItem::new(.., "box")`, `PickerItem::icon(..)`, `PieEntry` and `WindowOptions::icon(..)`. Each also accepts a `kivact::style::Icon` such as an image, so you can pass an image directly without registering it. A theme that lacks the name you ask for draws nothing and reports a diagnostic, so keep the kit's theme (or add the name) when you switch themes.

`ui::KeyGlyph` (`Diamond`, `Square`, `Circle`, `HalfDiamond`) is the shape of a keyframe marker: set it per key with `Keyframe::glyph(..)`. `ui::LabelTone` (`Normal`, `Muted`, `Accent`, `Warning`, `Error`, `Custom(color)`) names a text color that follows the theme; `TreeItem::tone(..)` uses it.

## Translate the texts the kit shows

The few texts the kit draws itself (the dock's menu commands, a transport button's names, a dialog's Close button, the outliner's accessible name) are English by default. Replace them with `App::messages`, which maps a key to your text. Keys that you leave out keep their English default.

```rust,no_run
use kivact::i18n::Messages;
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn translated(app: App) -> App {
    app.messages(
        Messages::new()
            .with("creative.dock.close_panel", "Fermer le panneau")
            .with("creative.dock.maximize", "Agrandir le panneau")
            .with("creative.transport.play", "Lecture")
            .with("creative.outliner.label", "Hiérarchie"),
    )
}
```

All keys start with `creative.`. Each component's rustdoc lists the keys it reads (`cargo doc -p kivact-ui-creative --open`). To switch languages while the app runs, see [Translate the app and set its locale](../../../../for-kivact-user/en/how-to-guide/i18n.md).

## See also

- [Change the theme and the look of components](../../../../for-kivact-user/en/how-to-guide/theming.md): the core theme system.
- [Build docks, panel groups and outliners](panels-docks-and-outliners.md): where the icons and texts appear.
- The kit's README lists every built-in icon name: `crates/kivact-ui-creative/README.md` in the repository.
- Full API: `cargo doc -p kivact-ui-creative --open`
- All the controls in the kit's look: `cargo run -p kivact-ui-creative-catalog`
