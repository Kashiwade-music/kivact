# Get started with the Creative kit

[日本語](../../../jp/creative/getting-started/README.md)

The Creative kit (`kivact-ui-creative`) provides the controls of creative-editing tools: dense numeric fields, color wheels and easing curves, outliners, docks, timelines, node graphs and viewports. This tutorial builds the property **Inspector** of an imaginary layer editor: you edit a layer's numbers, color and easing curve in a panel, then place that panel in a dock next to an outliner of layers.

You need to know the basics of Kivact first: signals, components as functions and how to run an app. If you have not done so, work through the [core tutorial](../../../../for-kivact-user/en/getting-started/README.md) before this one.

By the end you will have a window with a dock of two panels, a test that drives it, and an idea of how the kit separates its job (drawing and gestures) from yours (owning the data).

| Step | You build | You learn |
|------|-----------|-----------|
| [1. Set up the kit](01-set-up-the-kit.md) | A themed window with one numeric field | Adding the dependency, the kit theme, `panel`, `property_row`, `number` |
| [2. Edit values](02-edit-values.md) | Position, opacity, color and easing editors with an undo button | One value in several editors, `EditPhase`, `color_popup`, `bezier_curve` |
| [3. Put it in a dock](03-dock-and-outliner.md) | A dock with a layer outliner and the inspector, plus a test | `dock`, `outliner`, a shared selection, testing |

To see everything the kit offers before you start, run its catalog from a checkout of the Kivact repository:

```text
cargo run -p kivact-ui-creative-catalog
```

When you are done, the [how-to guides](../how-to-guide/README.md) cover timelines, node graphs, curves and gradients, and showing images your application renders.
