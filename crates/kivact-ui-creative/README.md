# kivact-ui-creative

Dense creative-tool controls.
The library depends only on the public `kivact` facade, including its consumer tests.

```rust
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn inspector() -> impl IntoView {
    let exposure = signal(0.0);
    let enabled = signal(true);
    ui::panel("Render", column((
        ui::property_row("Exposure", ui::number("Exposure", exposure)
            .range(-10.0, 10.0).precision(3).unit(" EV")),
        kivact::ui::switch("Use Nodes", enabled),
        kivact::ui::button("Render").variant(kivact::ui::ButtonVariant::Primary)
            .disabled(move || !enabled.get()),
    )).role(kivact::view::Role::List))
}
```

Install `ui::theme(ThemeMode::Dark)` on an App or with
`kivact::view::provide_theme`: it is a plain `Theme` in the Iris accent and
the comfortable density. Choose others with
`ui::theme_with(ThemeMode::Dark, ui::ThemeOptions { accent: ui::Accent::Lime,
density: ui::Density::Compact })`. Use `..Default::default()` to keep omitted
options at their defaults. `Tokens` exposes the prototype's semantic color,
density and font tokens. Defaults use `sans-serif` and `monospace`; no font asset
registration is required. The catalog registers Geist, Geist Mono and Noto Sans JP
and sets `Tokens.body_font`/`mono_font` to reproduce the source typography.

The theme installs a complete `kivact::ui::UiRecipes` (built once and shared)
and an `IconSet` with every icon name the kit draws. Use standard buttons, checkboxes,
switches, text fields, selects, menus, dialogs, tooltips, tabs, radio groups,
sliders, splitters and the toast region through `kivact::ui`; they take the
kit's look from the recipes. Every component of the kit, like `kivact::ui`,
takes the values it edits as constructor arguments and keeps no hidden copy.
A segmented control is the standard radio group in the kit's look:
`radio_group(label, options, value).orientation(Orientation::Horizontal)
.variant(kivact::ui::RadioGroupVariant::Segmented)`. Domain controls such as
`document_tabs`, the numeric fields, dock and editor surfaces remain in this
kit; tool toggles, progress bars, inspector sections and split panes are the
standard `kivact::ui::toggle`, `progress`, `disclosure` and `split` in its look.

Components implement `kivact::control::NodeBuilder` and `IntoView`.
Use ordinary `NodeBuilder` modifiers for layout, styles, names, IDs and callbacks.
There is no official-component-only rendering or input route.

Icons are names in the theme's `IconSet`, drawn with
`kivact::control::icon("name")` like every icon: a 16px Tabler outline in the
inherited text color. The kit's theme has `click`, `adjustments-horizontal`,
`checkbox`, `bulb`, `adjustments`, `chart-bar`, `layout-columns`, `hierarchy`,
`table`, `stack`, `timeline`, `keyframes`, `schema`, `palette`, `color-picker`,
`menu`, `app-window`, `loader`, `crosshair`, `ruler`, `range`, `eraser`,
`zoom-in`, `hand`, `aperture`, `box`, `camera`, `check`, `chevron-down`,
`chevron-right`, `chevrons-left`, `chevrons-right`, `clock`, `x` (close),
`copy`, `density`, `diamond`, `eye`, `eye-off`, `folder`, `filter`, `info`,
`light`, `link`, `link-off`, `lock`, `lock-open`, `magnet`, `maximize`, `more`,
`move`, `pause`, `pencil`, `pin`, `play`, `play-backward`, `plus`, `pointer`,
`record`, `redo`, `repeat`, `rotate`, `scale`, `scissors`, `search`,
`settings`, `sidebar`, `start`, `step-back`, `step-forward`, `stop`, `trash`,
`undo`, `volume`, `warning` and `wave`. The kit's components (`DockTab::icon`, `TreeItem`, `PieEntry`, `PickerItem::icon`,
`WindowOptions::icon`) take such a name or any `kivact::style::Icon`, so they
draw under the kit's theme; another theme draws a name it lacks as nothing
and reports it. An application adds its own images to the theme's set under a
name, or passes an image:

```rust
use kivact::Asset;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

static NODE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 3l9 9-9 9-9-9z"/></svg>"#;
let theme = ui::theme(ThemeMode::Dark).with_icon("node", Asset::from(NODE));
let view = kivact::control::icon("node");
```

`LabelTone` (`Normal`, `Muted`, `Accent`, `Warning`, `Error`, `Custom(color)`)
names a label color that `LabelTone::color(&palette)` resolves against the
installed `Tokens`. `KeyGlyph` (`Diamond`, `Square`, `Circle`, `HalfDiamond`) is
the shape of a key: `Keyframe::glyph` sets the one a dope sheet draws, and
`KeyGlyph::paint(draw, center, radius, color)` draws it in an application's own
canvas.

Use `timeline(clips, state)`, `dope_sheet(keys, state)`, `graph_editor(curves, state)`,
`node_editor(nodes, links, state)` or
`color_wheel(color, model)`. Each builder accepts its own application model.
An editor's view state (selection, playhead, zoom, pan and so on) is one state
argument: `TimeState`, `GraphState`, `NodeState`, `ArrangementState`,
`OutlinerState` or the mixer strip's `ChannelState`, Copy structs of bindings.
`TimeState::new()` makes a signal at the default for each field; replace a
field to keep that value elsewhere
(`TimeState { playhead: frame.into(), ..TimeState::new() }`). `Lane` IDs remain stable when
rows are reordered. A `Clip` stores start and duration; a `Keyframe` stores time
and an optional positive duration. `CurveKey` uses time/value units and relative
`CurveTangent` offsets. Graph curves are read-only inputs; `on_edit` reports
`EditorEvent<GraphRequest>` with absolute `KeyMove` proposals, deletion IDs and
interpolation requests. The application applies these requests and restores its
Begin snapshot on Cancel; ignored requests leave the rendered curves unchanged.
The `view` of `GraphState` holds the visible graph ranges.

Timeline and DopeSheet accept read-only item values. Their
`on_edit(|event: EditorEvent<TimeRequest>| ..)` reports `Begin`, absolute `Retime`
or `Delete` requests, and `Commit` or `Cancel`. Applications apply requests to
their own items and restore them on cancellation; ignored requests leave items
unchanged. Playhead and selection are bindings of `TimeState`; its `transform: Binding<ViewTransform>` owns both the horizontal scale and pixel scroll offset. Each drag proposal
uses the items at `Begin`, so applying only the latest request is sufficient.
In every domain editor a drag reports `Begin` at its first change, not at the
press: a press that does not move selects or, as a stationary double click,
activates, and reports nothing. Escape during a drag cancels it; Escape
otherwise clears the selection.

Timeline and DopeSheet share `time_format`, `ruler_ticks`, `zoom_range` (default
0.25..=4) and `min_duration` (default 1). GraphEditor has independent time/value
ranges. NodeData owns geometry, typed sockets and controls; NodeLink identifies
both socket indices, so parallel links and reconnection stay independent.
NodeEditor accepts read-only node and link values and reports `EditorEvent<NodeRequest>`
through `on_edit`. The application applies Move, Connect, Disconnect, Delete,
ToggleMute and Control requests and restores its snapshot on Cancel. Socket type
names and colors are explicit; `can_connect` guards connection reports and highlights.
The application owns duplicate-link and input-replacement rules. Node previews use
optional application-provided `Pixels`. NodeEditor uses the core `ViewTransform`
and `zoom_pan` interaction contract.

The application renders previews, viewports and scopes into RGBA8 `Pixels`.
The core `kivact::view::viewport(pixels)` displays pixels and fits them by default.
Attach `.transform(use_zoom_pan(binding))` for shared pan and zoom. Use
`ZoomPanApi::to_content` and `to_view` with standard pointer events, `draw_over`
and canvas regions. `on_resize` reports logical and physical dimensions and scale.
GPU frames use only the facade's `gpu-interop` feature; snapshots draw the
viewport's own CPU pixels. ColorWheel combines hue, saturation/value and alpha
in a single independent color component.

Numeric controls (`number`, `value_slider`, `knob` and `fader`, each with a label and
the value binding) support pointer scrubbing, direct arithmetic input, wheel,
keyboard, finite clamped accessibility `SetValue`, cancellation and reset to
`reset_to(..)`. They write through the core `use_scalar`, whose `scrub` makes
the pointer scrub an edit of the same session as the keys. `precision` controls
display formatting and preserves the consumer's numeric precision.
`on_edit(|phase: EditPhase| ..)` reports `Begin`, `Commit` and `Cancel`
around each scrub, each key, wheel or assistive technology write and each
confirmed text, so an app can make one Undo step of each (a scrub that Escape or
the engine abandons has the start value written back before `Cancel`).
`text_parser(|text, start| Ok(value) | Err(reason))` replaces the built-in
expression parser; it runs while the text is typed, and an `Err` marks the field
invalid, shows the reason under it as a status that the field is `described_by`,
and keeps the draft pending until Escape discards it; Enter, or leaving the
field with a changed valid text, commits it (the standard
`text_input(..).draft(..)`). `mixed(true)` shows `Mixed`, reports the field as indeterminate and turns
off scrubbing, steps, the wheel and the double click reset: a press opens the text
field, and typed text or `SetValue` are the only writes. `property_state` takes a
reactive value (a plain `PropertyState` for a constant), so the tint changes without
rebuilding the field. Node scalar parameters also support a direct-entry popover.
`timecode(fps)` formats whole frames and accepts complete or shortened colon
notation and signed relative frame entry. `range(label, value)` keeps two ordered
endpoints on a `use_range_scalar`, can move the whole interval while preserving
its duration, and reports each drag, key and action through `on_edit`.

`arrangement(clips, tracks, state).length(length)` reads application-owned clips and tracks.
Its `on_edit` reports `EditorEvent<ArrangementRequest>`; the application applies
requests and restores its snapshot on Cancel. Split IDs, names and source offsets
belong to the application. Loop, playhead, tool and view values are the bindings of `ArrangementState`.
`zoom_range`, `min_duration` and `loop_min_duration` configure proposals;
`automation_label` supplies the optional lane heading. Track regions
are allocated independently of model identities, including large knot IDs.
`document_tabs(label, options, selected).on_reorder(on_reorder).on_close(on_close)` is `kivact::ui::tabs`
with a tab slot that closes a tab and drags it 60 px to swap with a
neighbor; the application applies both to its options. `.content(|key| ..)`
and `.keep_mounted(true)` are the standard tabs'.
The standard `kivact::ui::data_table(rows, key, columns)` is drawn dense in this
kit's theme (its `DataTableRecipes`): `.focus(TableFocus::Rows(ListPolicy::default()))` makes the rows
one tab stop (the row focused last), and Up, Down, Home, End, Page Up and Page
Down move and select (Shift extends a range, Ctrl or Cmd moves the focus
alone), Space and Ctrl or Cmd+Space select and toggle the focused row, Ctrl or
Cmd+A selects every row shown, and typing a letter jumps to the next row whose
label starts with it (`ListPolicy { typeahead: true, .. }` in the variant). `.row_height(..)` and `.row_color(..)`
reach the row recipe, `.cell_controls(..)` keeps controls mounted while their
row changes, `.resizable(widths)` resizes columns and `.row_details(..)` puts
inline details under a row. `outliner` supports range selection, rename and safe
reparenting, subtree expansion and restriction painting. Item lists passed to
`outliner(items, state)` and the standard `data_table(rows, ..)` take read-only reactive values.
Applications apply `on_item_drop`, `on_rename` and `on_reorder` requests themselves;
without drop/reorder callbacks, dragging is disabled. Outliner expansion,
selection and rename state are the bindings of `OutlinerState`.
`on_restriction` reports `EditorEvent<Restriction>`: Begin, absolute proposals
with accumulated IDs and one uniform value, then Commit or Cancel. Applications
restore their Begin snapshot on Cancel. Selected rows toggle together to the
opposite of the pressed or focused row's initial flag. `panel_group` retains
contents while collapsing peers or reordering panels. `pie_menu` supports a
consumer-defined held shortcut through its public state. `color_popup` restores
the opening color on Escape and commits on outside dismissal. Its swatch keeps
its color when hovered, pressed or focused, and retains its accessible name.
`search_picker(items)` is a searchable, sectioned popup list of keyed
`PickerItem`s (label, section, description, shortcut, icon, enabled): `.attach(button)`
opens it below a button and `.at(anchor, position)` at a window point. The focus stays
in the search field, Up and Down move the active row over the enabled items and stop at
the ends, Enter or a click picks (`on_pick`), Escape, an outside press and Tab cancel
(`on_cancel`), and `on_preview` reports the active key for a live preview and `None`
when it ends.
`popup_button` and `dialog_button` open a popover below the button and a modal
dialog: they are the core `dialog` parts (`DialogKind::Popover` and
`DialogKind::Modal`) drawn in Creative's look, named like the button, with the
title as a heading, a Close button under the content, and Escape, outside
press and assistive technology's Dismiss closing them.
`window_button(label, options, content)` provides a draggable, collapsible
window whose title remains recoverable inside the viewport. Escape and Close
restore its opener focus; outside dismissal preserves the newly focused
control. `WindowOptions` sets a separate title, width, center or top-right
placement, modal containment and chrome spacing (`WindowOptions::default()` is
a 520-pixel nonmodal centered window). Its content factory receives a
`ToolWindowApi` whose `close()` closes the window and restores opener focus. Collapsing retains edited content; reopening mounts
the factory again. Modal backdrop presses keep the confirmation open,
while Escape closes it. Optional backdrop blur uses ordinary public styles.

```rust
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn tool_window() -> impl IntoView {
    ui::window_button("Settings…",
        ui::WindowOptions::default().title("Settings").width(340.0)
            .position(ui::WindowPosition::TopRight { right: 40.0, top: 120.0 }),
        |window: ui::ToolWindowApi| kivact::ui::button("Apply").on_press(move || window.close()))
}
```

`window(anchor, open, options).content(content)` is the same window that the
application opens and closes through a `Binding<bool>`; `window_button` is
a button and a state of its own on top of it. Every way to close it (the close
button, Escape, Dismiss, `ToolWindowApi::close`, an outside press on a
nonmodal window, unmounting) writes `false`. A nonmodal window gives the focus
back with `anchor.restore_focus()`, which does nothing if the anchor has gone;
a modal window gives it back to the element that had it when it opened. Each
opening starts expanded and at the initial position.

`kivact::ui::split((first, second), sizes)` puts panes side by side, or stacked
with `.orientation(Orientation::Vertical)`, with the core `use_splitter` handle
between them, drawn like the dock's. `sizes` holds the fractions of the panes
and `min_size` keeps a fraction for each. `property_row(label, value)` is an inspector row: a
right-aligned one-line label that shows its full text in a tooltip, the value,
and the `leading` and `trailing` slots; `property_state_button` is the 20 px
animation-state button for `leading`. `panel(..).header(|| view)` replaces the
panel's title bar. The slots (`header`, `leading`, `trailing`) take a closure
that builds the view when the component mounts. `kivact::ui::disclosure(title, open).content(|| content)`
is a section that the `open` binding opens and closes. Closing unmounts its
content; keep persistent drafts in state outside the content factory.

```text
cargo run -p kivact-ui-creative-catalog
cargo test -p kivact-ui-creative --test consumer -- --test-threads=1
```

`KIVACT_TEST_GPU=1` requests and asserts the GPU renderer for consumer tests;
the default asserts the CPU renderer. Cargo must also enable the facade's `gpu`
feature when requesting GPU. The tests use `MirrorMode::Required`. Catalog evidence
uses `KIVACT_CATALOG_EVIDENCE=<directory>` and `KIVACT_CATALOG_GPU=1`.

Tabler SVG
glyphs retain their [MIT notice](LICENSE-tabler-icons.txt).

The catalog contains 22 component chapters plus one hero section. Its procedural
media, audio, grading and job fixtures demonstrate reusable controls; decoding,
project storage, compositing, export, playback clocks, IDs, names and connection
rules belong to the application.

`transport(state)` binds only playing, recording and cycling. Use `on_to_start`
to handle seeking; the application owns frame, frame rate, loop bounds and clock.
`mixer_strip(label, color, channel)` edits the gain, pan, mute and solo of its
`ChannelState` and displays the supplied final `level` unchanged by gain or mute.
Gain uses `gain_range(min, max)` (default 0..1); audio applications can supply
`gain_range(-60.0, 6.0)`. Inserts default to empty labels; routing appears only with
`routes(labels, route)` and nonempty labels. The application applies DSP and routing choices.
`fader` maps a normalized value `t` to position `sqrt(t)` and position `n` back
to `min + (max - min) * n²` over its configured range; audio applications can
explicitly choose `range(-60.0, 6.0)`.
`GradientStop::new(position, color)` has no identity. `gradient_editor(stops, selection)`
takes the selected stop as an `Option<usize>` into the stop vector; adding and
deleting adjust that index.

Toasts are `kivact::ui::toast_region(toaster)` in the kit's theme;
`Toast::new(title).tone(Tone::Success)` is a core `Toast` whose tone icon the
theme draws, and `.body(..)` adds its second line.

`meter(label, level)` and a `fader` lie vertically until
`.orientation(Orientation::Horizontal)` (constant or reactive).
`range(label, value)` is the range scalar's track, thumbs and body on one
canvas with one keyboard focus: a press on the track moves the nearer end, a
press on an end or on the movable interval drags it, and the keys edit what
was pressed, selected or moved last. Context menus are
`kivact::ui::menu(label, entries, choose).attach_context(target)`.

`arrangement(clips, tracks, state).length(beats)` defaults to 16 beats. `split_button(label, primary, entries).on_select(choose)` keeps the primary action required. `window(anchor, open, options).content(factory)` rebuilds content on opening and defaults to empty content. Document-tab close/reorder callbacks default to no action. Mixer inserts/routes and pie-menu entries accept reactive read-only lists. Segmented radio groups use the standard item slot.

The theme fills these `UiRecipes` entries:

<!-- kivact-recipes -->
- `button`
- `button_content`
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
- `toggle_group`
- `table`
- `item_list`
- `badge`
- `badge_label`
- `avatar`
- `avatar_status`
- `avatar_group`
- `skeleton`
- `spinner`
- `kbd`
- `kbd_key`
- `alert`
- `empty_state`
- `card`
- `input_group`
- `heading`
- `stat`
- `description_list`
<!-- /kivact-recipes -->

Rows follow the core layout direction automatically, including right-to-left layouts.
