# 3. Put it in a dock

[日本語](../../../jp/creative/getting-started/03-dock-and-outliner.md)

In this step the inspector becomes one panel of a dock workspace. A second panel, an outliner, lists the layers; selecting a layer there shows its values in the inspector. You finish with a test that drives the finished app the way a user does.

The finished app keeps all layers in one signal. The outliner and the inspector are two views of it, and the **selection** is a third piece of state they share.

Replace `src/main.rs` with the whole program below. It reuses the fields from step 2, now bound to a *layer* instead of to loose signals.

```rust,test_harness
use kivact::state::Binding;
use kivact::prelude::*;
use kivact::style::{Color, ThemeMode, rgb};
use kivact_ui_creative as ui;

#[derive(Clone, PartialEq)]
struct Layer {
    id: u64,
    name: String,
    x: f64,
    opacity: f64,
    fill: Color,
    easing: ui::CubicBezier,
}

fn sample_layers() -> Vec<Layer> {
    [
        (1, "Background", 0x20242c),
        (2, "Title", 0x8b7cff),
        (3, "Subtitle", 0xffd25a),
    ]
    .into_iter()
    .map(|(id, name, fill)| Layer {
        id,
        name: name.into(),
        x: 0.0,
        opacity: 1.0,
        fill: rgb(fill),
        easing: ui::CubicBezier::default(),
    })
    .collect()
}

/// A binding to one field of the layer that `current` names.
fn layer_field<T: Clone + PartialEq + 'static>(
    layers: Signal<Vec<Layer>>,
    current: Value<Option<u64>>,
    fallback: T,
    read: fn(&Layer) -> T,
    write: fn(&mut Layer, T),
) -> Binding<T> {
    binding(
        move || {
            layers.with(|all| {
                let id = current.get()?;
                all.iter().find(|layer| layer.id == id).map(read)
            })
            .unwrap_or_else(|| fallback.clone())
        },
        move |value: T| {
            layers.update(|all| {
                let id = current.get_untracked();
                if let Some(layer) = all.iter_mut().find(|layer| Some(layer.id) == id) {
                    write(layer, value);
                }
            });
        },
    )
}

fn layer_form(layers: Signal<Vec<Layer>>, current: Value<Option<u64>>) -> impl IntoView {
    let x = layer_field(layers, current, 0.0, |l| l.x, |l, v| l.x = v);
    let opacity = layer_field(layers, current, 1.0, |l| l.opacity, |l, v| l.opacity = v);
    let fill = layer_field(layers, current, rgb(0), |l| l.fill, |l, v| l.fill = v);
    let easing = layer_field(
        layers,
        current,
        ui::CubicBezier::default(),
        |l| l.easing,
        |l, v| l.easing = v,
    );
    column((
        ui::property_row(
            "X",
            ui::number("X", x)
                .range(kivact::interaction::ScalarRange::new(-1000.0, 1000.0).expect("valid range"))
                .step(1.0)
                .precision(1)
                .unit(" px")
                .label_visible(false),
        ),
        ui::property_row(
            "Opacity",
            ui::number("Opacity", opacity)
                .range(kivact::interaction::ScalarRange::new(0.0, 1.0).expect("valid range"))
                .step(0.01)
                .precision(2)
                .label_visible(false),
        ),
        ui::property_row("Fill", ui::color_popup("Fill", fill)),
        ui::bezier_curve(easing),
    ))
    .style(style().gap(6).padding(10))
}

fn editor(layers: Signal<Vec<Layer>>) -> impl IntoView {
    let outline = ui::OutlinerState::new([]);
    outline.selection.set(vec![2]);
    // The layer the inspector edits: the first selected one.
    let current = memo(move || outline.selection.get().first().copied());

    let dock_layout = signal(ui::DockLayout::new(ui::DockNode::split(
        ui::DockAxis::Horizontal,
        [
            (
                0.4,
                ui::DockNode::tabs(vec![
                    ui::DockTab::new("layers", "Layers").icon("stack").closable(false),
                ]),
            ),
            (
                0.6,
                ui::DockNode::tabs(vec![
                    ui::DockTab::new("inspector", "Inspector").icon("adjustments").closable(false),
                ]),
            ),
        ],
    )));

    ui::dock(dock_layout, move |panel: &'static str| match panel {
        "layers" => ui::outliner(
            move || {
                layers
                    .get()
                    .iter()
                    .map(|layer| ui::TreeItem::new(layer.id, None, layer.name.clone(), "box"))
                    .collect::<Vec<_>>()
            },
            outline,
        )
        .restrictions(false)
        .on_rename(move |(id, name): (u64, String)| {
            layers.update(|all| {
                if let Some(layer) = all.iter_mut().find(|layer| layer.id == id) {
                    layer.name = name;
                }
            });
        })
        .into_view(),
        "inspector" => show_some(current, move |_| layer_form(layers, current))
            .fallback(|| text("Select a layer"))
            .into_view(),
        _ => node().into_view(),
    })
    .style(style().width(pct(100)).height(pct(100)))
}

fn main() -> kivact::Result<()> {
    App::new("Layer inspector")
        .theme(ui::theme(ThemeMode::Dark))
        .window_size(900, 520)
        .run(|| editor(signal(sample_layers())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{RenderOptions, render_with_handle};

    #[test]
    fn the_inspector_edits_the_selected_layer() {
        let options = RenderOptions::from_env().theme(ui::theme(ThemeMode::Dark));
        let (screen, layers) = render_with_handle(options, || {
            let layers = signal(sample_layers());
            (layers, editor(layers))
        });

        // "Title" starts selected: End jumps its X to the top of the range.
        screen.get_by_role(Role::SpinButton, "X").focus();
        screen.press_key(Key::End);
        assert_eq!(layers.get_untracked()[1].x, 1000.0);

        // Selecting another layer in the outliner re-targets the inspector.
        screen.get_by_role(Role::TreeItem, "Subtitle").click();
        screen.get_by_role(Role::SpinButton, "X").focus();
        screen.press_key(Key::Home);
        assert_eq!(layers.get_untracked()[2].x, -1000.0);
        assert_eq!(layers.get_untracked()[1].x, 1000.0, "Title is untouched");
    }
}
```

Run it with `cargo run`. You should see a window split in two: a **Layers** panel on the left listing `Background`, `Title` and `Subtitle` (with `Title` highlighted), and an **Inspector** panel on the right showing that layer's X, Opacity, Fill and easing curve.

## Try it

1. Change **X** for `Title`. Click `Subtitle` in the outliner: the inspector now shows `Subtitle`'s own values. Click `Title` again and its X is still there.
2. Double-click `Subtitle`, type a new name and press `Enter`. The outliner shows the new name. `Esc` abandons the rename.
3. Drag the divider between the panels, or open the menu button at the right of a panel's tab strip and choose **Split Right**. Drag the **Inspector** tab onto another edge of the window to dock it elsewhere.
4. Run `cargo test`. The test at the bottom opens the app in a real window, selects layers and edits them through the keyboard.

## What you wrote

- **One model, two views.** All layers live in the single `layers` signal. The outliner shows `layers` as a list of `TreeItem`s, and the inspector edits one field of one layer. Neither owns the data, so neither can drift from the other.
- **`ui::OutlinerState`** bundles the outliner's view state as bindings you own: `expanded` (open items), `selection` and `renaming`. Because the selection is a `Binding<Vec<u64>>` that you hold, the inspector can read it. That is how the two panels share the selection. Every editor of the kit has such a state struct (`TimeState`, `GraphState`, ...).
- **`layer_field`** turns "this field of whichever layer is selected" into a `Binding<T>`. `binding(get, set)` joins a reader (a closure that is re-run when what it reads changes) and a writer. The component sees an ordinary binding; it neither knows nor cares that a list sits behind it. `memo` caches `current`, so readers re-run only when the selected id actually changes.
- **`show_some(current, ..).fallback(..)`** shows the form while a layer is selected and a hint otherwise. Moving from one selected layer to another keeps the same form; only the bindings' results change.
- **The dock layout is data.** `ui::DockLayout<K>` is a tree of splits (`DockNode::split`) with tabbed containers (`DockNode::tabs`) at its leaves. `K` is any key you choose that is `Clone + Eq + Hash`; here a `&'static str` names each panel. You hand the layout to `ui::dock(layout, factory)` as a signal, and every change the user makes (dragging a tab, splitting, resizing) is written back to it, so you can save it, restore it or undo it like any other state.
- **The panel factory runs once per key.** Switching tabs hides a panel without destroying it, so a panel keeps its own state. `closable(false)` removes the tab's close button.
- **The test** uses `kivact::test::render_with_handle`. It builds the app inside a test window, lets your setup closure return values you want to inspect (here the `layers` signal), and returns a `Screen`. You find elements by their accessibility role and name (`Role::SpinButton` named `X`), the way a screen reader sees them, and drive them with `focus()`, `click()` and `press_key(..)`. `RenderOptions::from_env()` reads `KIVACT_TEST_MIRROR` (`off`, `auto`, or `required`; default `auto`) and `KIVACT_TEST_GPU=1`. Set `KIVACT_TEST_MIRROR=off` to run without a display.

## Where you are

You built a property inspector: numeric fields with undo hooks, a color popup, an easing curve, an outliner and a dock, all bound to application state, plus a test. The kit never held your data at any point.

## Next

- Wire a timeline to your own clips: [Edit a timeline and an arrangement](../how-to-guide/timeline-and-arrangement.md)
- Edit more values the same way: [Choose colors, curves and gradients](../how-to-guide/color-curves-and-gradients.md)
- More on panels: [Build docks, panel groups and outliners](../how-to-guide/panels-docks-and-outliners.md)
- All guides: [Creative how-to guides](../how-to-guide/README.md)
