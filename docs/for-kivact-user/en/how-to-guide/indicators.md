# Indicators and keyboard shortcuts

Use the standard indicators with any kit theme. Their recipes receive the same semantic data; kit variants select their appearance.

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact::ui::{self, ControlSize, SkeletonShape};
use kivact::view::Modifiers;

fn indicators() -> impl IntoView {
    column((
        ui::badge("Ready").tone(Tone::Success),
        ui::avatar("Ada Lovelace").size(ControlSize::Small)
            .status(Some(Tone::Success)),
        ui::avatar_group((ui::avatar("Ada"), ui::avatar("Grace"))).max(1),
        ui::skeleton().lines(3).shape(SkeletonShape::Text),
        ui::spinner().size(ControlSize::Small),
        ui::kbd("mod+K"),
        ui::kbd_combo(vec![Key::Char('K')])
            .modifiers(move || Modifiers { meta: true, ..Modifiers::default() }),
    ))
}
```

`Avatar::src` and `fallback_src` accept reactive optional assets. While an image loads, or if decoding fails, initials remain available. The image node owns loading status; replacing its source retries naturally. `shape` chooses Circle, Rounded or Square. `status` supplies semantic presence and its accessible label; `status_content` supplies a custom overlay. Use `on_press` for a button and `on_open` for a link action. Attach `ui::tooltip(name)` when a tooltip is wanted. A group's maximum is reactive; arrow navigation belongs to the shared roving focus implementation.

Skeletons are decorative. Reduced motion pauses their clock and leaves them still. Spinners expose a status named by the `spinner.loading` message and use one animation. `label_content` adds visible content; `.name(...)` changes the accessible label. The default loading content of `async_view` uses this same spinner.

`kbd` accepts text and recognizes `mod`. `kbd_combo` accepts reactive logical keys; `.modifiers(...)` adds a chord's modifiers. `meta` uses `Capabilities.command_key`, so the host chooses Command or Ctrl. Physical Ctrl remains Ctrl. No compile-target OS test selects the symbol.

Configure `UiTokens::indicators` for timing and placeholder metrics. `UiRecipes` provides `badge`, `avatar`, `avatar_status`, `avatar_group`, `skeleton`, `spinner` and `kbd`. Their corresponding `_recipe()` getters let a theme wrap the existing recipe. Caller styles override the defaults.

## Moving existing kit options

| Existing kit option | Standard composition |
| --- | --- |
| Basic badge variant/icon/trailing_icon/on_press | `badge.variant/icon/trailing_icon/on_press` |
| Pop badge hue, Dot, Count, Slant; Game Solid/Outline/Dot; Soft semantic/decorative colors | Kit `Variant` values consumed by badge recipes; semantic colors use `tone` |
| Basic avatar image, badge and group count | `src`, `status_content`, `avatar_group.max` or a custom child |
| Pop avatar hue/halo/online; Game color/online | Kit variants and recipes; `status` carries presence |
| Kit avatar sizes, including arbitrary pixels | `ControlSize`, caller size styles, or a theme recipe |
| Soft avatar fallback photo, shape, alt, tooltip, press/open and custom status | `fallback_src`, `shape`, `.name`, attached tooltip, `on_press/on_open`, `status_content` |
| Soft group shared size/shape/label and overflow label | Configure children and the group `.name`; use `max` or a custom overflow child |
| Basic/Pop/Game skeleton rows | `lines` |
| Soft skeleton radius/index | `shape`, recipe variants and `index` stagger |
| Pop dots loader; Soft spinner shade/ink | Kit spinner variants and recipes |
| Soft spinner name, text label and arbitrary label view | `.name`, `label_content(text(...))`, `label_content` |
| Basic kbd group, Soft parsed shortcut | Parsed `kbd` or typed `kbd_combo` |

Creative's `KeyGlyph` paints editor keyframe markers and remains a domain type; it is unrelated to keyboard shortcuts.

`UiTokens::indicators.reduced_period_ms` optionally keeps the spinner moving at a slower essential cycle under reduced motion. `None` retains decorative pausing. A supplied cycle responds to live preference changes by replacing the glyph animation owner, keeping the status node and label stable.

`kbd("mod+shift+p")` and typed `kbd_combo` both normalize shortcuts into individual key caps. The `mod` key follows `Capabilities.command_key`; spoken names use localized full key labels while caps show the keyboard glyph. Aliases include command/cmd, control/ctrl, return/enter, esc/escape, arrows, and `plus` for a literal plus cap. Themes style the borderless row through `kbd` and each `KbdKeyState` through the `kbd_key` scalar recipe. Localize `kbd.key.*` and `kbd.separator` through the message catalog.

`UiTokens::indicators.skeleton_transition` optionally overrides the base skeleton tween. The component adds the configured initial/index stagger delay and alternating repetition, keeping one `Animated` value and the reduced-motion pause policy.
