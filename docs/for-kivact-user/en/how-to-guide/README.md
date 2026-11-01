# How-to guides

[日本語](../../jp/how-to-guide/README.md)

These guides each solve one concrete problem in an app built with Kivact. They
assume you have finished the [tutorial](../getting-started/README.md), which
teaches the basics in order. Ready-made themes and components for particular
kinds of apps (business tools, games, creative tools) are in the
[kits guides](../../../for-kivact-ui-user/en/README.md).

Kivact is in early development, and breaking changes happen without notice. Every
Rust example in these guides is compiled by the repository's tests.

If you know React or SolidJS, start with the first guide: it maps what you know
to Kivact.

| Guide | What it solves |
| --- | --- |
| [Map what you know from React or SolidJS to Kivact](coming-from-react-or-solid.md) | The concept map, setup-once components, `dynamic` match arms and the Rust points that trip up newcomers |

## Build screens

| Guide | What it solves |
| --- | --- |
| [Lay out a screen](layout.md) | Rows, columns, grids, scroll areas, sizes, alignment and a two-pane layout |
| [Validate a form before saving](forms.md) | Drafts, validation, `field` errors, every input type and submit on Enter |
| [Show lists, selectable lists and tables](lists-and-tables.md) | `for_each`, `for_index`, `virtual_list`, `list` and `data_table`, and writing back from a row |
| [Show a dialog, menu, tooltip or toast](overlays.md) | Modal dialogs, popovers, menus, context menus, tooltips and notifications |
| [Show images and use custom fonts](images-and-fonts.md) | PNG, JPEG and SVG images, alt text, sizing, icons and registering fonts |

## Look and language

| Guide | What it solves |
| --- | --- |
| [Change the theme and the look of components](theming.md) | Light and dark themes, tokens, recipes, variants, state looks, icons and your own tokens |
| [Translate the app and set its locale](i18n.md) | Locale, messages, switching language at run time and text direction |

## Behavior

| Guide | What it solves |
| --- | --- |
| [Run async work from a component](async-and-tasks.md) | Load data, save in the background, wait on timers and run heavy work off the UI thread |
| [Animate values and elements](animation.md) | Move a value toward a target, run code every frame and animate elements in and out |
| [Use platform services](platform-services.md) | Clipboard, file dialogs, opening paths and other operating system features |
| [Draw with a canvas](canvas.md) | Draw your own graphics, charts and effects |

## Quality

| Guide | What it solves |
| --- | --- |
| [Test your app](testing.md) | Render a screen, find elements as a screen reader would and click and type in tests |
| [Find out why something failed or updated](diagnostics.md) | Find, read and report errors, warnings and unexpected updates |

## Extend Kivact

| Guide | What it solves |
| --- | --- |
| [Write reusable components](custom-components.md) | Turn repeated UI into components other code can reuse |
| [Build controls with your own look](custom-controls.md) | Build a control with your own behavior and appearance from the interaction parts |

## Experimental

These features are available but their API can change more than the rest.

| Guide | What it solves |
| --- | --- |
| [Open a Kivact UI inside a plugin window](embed.md) | Run a Kivact screen inside another application's window, such as a plugin editor |
| [Show textures from your own GPU renderer](gpu-texture-sharing.md) | Show textures that your own GPU code renders inside a Kivact UI |
