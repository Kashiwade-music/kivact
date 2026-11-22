# Migrate an app to the reworked API

[日本語](../../jp/how-to-guide/migrating-to-rework.md)

Kivact's public API was reworked in one breaking change. There is no
compatibility layer: every old name below is gone, and each row gives what to
write instead. The tables are grouped by the module you import from. The
[tutorial](../getting-started/README.md) and the other guides already use the
new API.

## Before you start

**Pin every repository that uses Kivact to one revision.** Use the same `rev`
in the git dependency of all your apps and component libraries, and update them
together. Two revisions of Kivact in one build are two different crates, and
their types do not mix.

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", rev = "<commit>" }
```

**Optimize dependencies in debug builds with one line.** If your `Cargo.toml`
lists crates of Kivact under `[profile.dev.package.<name>]`, delete those
sections: the crates inside Kivact are split and merged when it changes. Write
this instead:

```toml
[profile.dev.package."*"]
opt-level = 3
```

Your own crates stay unoptimized and easy to debug; everything they depend on,
Kivact and its renderer included, is optimized.

**Work from the top of this page down.** Fix components first, then state,
then styles: most compiler errors in the later tables disappear once the
components have their new shape.

## Components

| Before | After |
|---|---|
| `fn header() -> impl Element { .. }`, `fn panel() -> impl IntoView { scope(move \|\| ..) }` | `#[component] fn header() -> impl Element { .. }` |
| `#[component] fn card(..) -> impl IntoView` (callers could not add `.style(..)`) | `#[component] fn card(..) -> impl Element`; the caller's `.style(..)`, `.name(..)`, `.test_id(..)`, `.disabled(..)` and handlers apply to the component's root |
| `#[prop(optional)] on_close: Option<Callback<()>>` | `on_close: Option<Callback<()>>`; an `Option<..>` prop is optional by itself and becomes a setter |
| a hand-written builder struct with `impl Element` and `impl IntoView` | a `#[component]` function |
| `constant_value!(Tone);` next to your own prop type | `#[derive(Clone, PartialEq, Prop)]` on the type |
| `scope(move \|\| view)` (in the prelude and `kivact::view`) | a `#[component]`; a component library that builds without the macro uses `kivact::custom::scope` |
| a helper taking `impl IntoText` and calling `.into_text_value()` | a `Value<String>` prop |
| a helper taking `impl IntoValue<T>` and calling `.into_value()` | a `Value<T>` prop |
| a helper taking `impl Fn() -> V` content | a `ViewFn` prop |
| `type Title = Value<String>;` used as a prop type | write `Value<String>`: the macro recognizes the literal names `Value`, `Signal`, `Callback`, `View`, `ViewFn` and `Vec<View>`, and an alias is a plain prop |

`App::run` takes any function that returns a view. Keep the root function plain when it is named `app`: `#[component]` would generate a type named `App`.

## State: `kivact::state`

| Before | After |
|---|---|
| `Binding<T>` | `Signal<T>` |
| a parameter `value: impl IntoBinding<T>` | `value: Signal<T>` |
| `binding(get, set)` returned `Binding<T>` | `binding(get, set)` returns `Signal<T>` |
| `task.project(\|t\| &t.done, \|t\| &mut t.done)` | `bind!(task.done)` |
| `for_each` over a `Signal<Vec<T>>` gave each row a `Binding<T>` | each row gets a `Signal<T>`; a readonly source still gives `Value<T>` |
| `task.get_untracked()` in a component body | `task.get()`: a component body does not track reads |
| `Notifier`, `notifier()` | `Trigger`, `trigger()` with `track()` and `notify()` |
| `signal.notify()` | a `Trigger` next to the signal: `changed.notify()` |
| `use_context::<T>()` returned a `Value<T>`: `session.get().user.get()` | it returns the `T`: `session.user.get()` |
| `try_use_context::<T>()` returned `Option<Value<T>>` | it returns `Option<T>` |
| `signal.update(f)` returned `R` and panicked on the signal of a removed component | it returns nothing: a write to the signal of a removed component is reported and does nothing; read a computed value with `with` after the update |
| `Callback::bind(..)`, `Callback::to_fn()` | pass the `Callback` itself: every `on_*` and every callback prop accepts it |
| `let c: Callback<()> = (move \|\| ..).into();` | `let c = callback(move \|_\| ..);` |
| `kivact::custom::Reactive<T>` | store a `Value<T>` |
| `IntoReactive<bool>` in the bounds of `show`, `show_some`, `dynamic` | `impl IntoValue<bool>`; call sites do not change |
| `kivact::state::Owner` | `kivact::custom::Owner` |
| `IntoValue`, `IntoText`, `IntoBinding`, `PropValue`, `ItemKey`, `IntoSelection`, `Selection` from the prelude | not in the prelude: `IntoValue`, `IntoText` and `IntoHandler` are in `kivact::state`, `IntoSelection` and `Selection` in `kivact::interaction`; `IntoBinding` is gone |

## Event handlers

| Before | After |
|---|---|
| `on_press(impl Into<Callback<()>>)` | a handler without an event (`on_press` and every `Callback<()>` prop) takes a closure `move \|\| ..` or a `Callback<()>` |
| a handler with an event (`on_key`, `on_activate`, `on_commit`, `on_edit`, a `Callback<E>` prop) | unchanged: a closure `move \|event\| ..`; to pass a shared `Callback<E>`, write `move \|e\| cb.call(e)` |
| `.on_draft_commit(f)` | `.on_commit(f)` |
| `stepper(items, current).on_step_press(f)` | `stepper(items, current)` with `current: Signal<usize>`; read the signal, and `.on_edit(\|phase\| ..)` reports each edit phase |
| `tag_input(label, tags).on_add(f).on_remove(g)` | `tag_input(tags).label(label)` with `tags: Signal<Vec<String>>` |
| `avatar(..).on_open(f)` | removed: what the avatar opens takes a `Signal<bool>`; see the `avatar` rustdoc |
| `hover_card(..).on_open_change(f)`, `navigation_menu(..).on_open_change(f)` | `.open(signal)` and read the signal |
| a closure shared by two handlers (`let save = move \|\| ..;` passed twice), or one built with `Callback::from(..)` | `let save = callback(move \|_\| ..);` and pass `save` to both: every shared handler is a `Callback`, with or without an event |

## Views and elements: `kivact::view`

| Before | After |
|---|---|
| `dynamic(move \|\| r.state(), \|state\| match state { .. .into_view() })` for loading, error and ready | `async_view(r, ready).loading(\|\| ..).fallback(\|error, retry\| ..)` |
| `Retry` in `.fallback(\|error, retry: Retry\| ..)`, `retry.run()` | `retry` is a `Callback<()>`: `.on_press(retry)` or `retry.call(())` |
| `kivact::task::AsyncSnapshot` | `AsyncState` |
| `card(..).disabled(..)`, `card(..).on_press(..)` (methods of one component) | the same calls, now the `Element` methods every component has |
| `.variant(..)` defined by each component | the same call, now `Element::variant` |
| `use_direction()` returned `Value<ReadingDirection>` | it returns `ReadingDirection` |
| an invisible node and a timer to make the focused field commit its draft before saving | `kivact::view::commit_focused_edit().await` |
| `spacer()` could not shrink below its content | `spacer()` has a minimum size of 0 |
| — | `PointerEvent::target_size()` and `DragUpdate::target_size()`: the size of the node the handler is on |

## Standard components: `kivact::ui`

| Before | After |
|---|---|
| `text_input("Name", value)` | `text_input(value).label("Name")` |
| `field("Name", text_input("Name", value))` | `field("Name", text_input(value))`: `field` names the control |
| `text_area("Bio", value)`, `select("Color", items, value)`, `combobox("Colors", items, value)`, `slider("Volume", value)`, `number_input("Count", value)`, `date_input("Date", day)`, `date_range_input(..)`, `time_input(..)`, `date_time_input(..)` | the name moves to `.label("..")` or to the enclosing `field("..", ..)` |
| `button(label)`, `checkbox(label, value)`, `switch(label, value)`, `radio_group(name, items, value)` | unchanged: their label is visible |
| a focusable control without a name | reported when it mounts (an error in debug builds and in `kivact::test`) |
| `tooltip(text).attach(trigger)` | `tooltip(text, trigger)` |
| `toaster()` | `use_toaster()` |
| `announcer()` | `use_announcer()` |
| `selection_field(label, value_label).content(f)` | `select(items, value).label(label).trigger(view)` |
| `input_group_with(label, \|\| children)` | `input_group(label, children)` |
| `dialog(title, open).mode(DialogMode::Popover)` | `kivact::interaction::use_popup(open)` and its `surface(placement, content)` |
| `use_dialog(open, DialogMode::Popover).anchor(node)` | `kivact::interaction::use_popup(open).anchor(node)` and its `surface(placement, content)` (a non-modal surface is a popup; `use_dialog` is modal only) |
| `MenuApi::while_open(f)`, `CommandPaletteApi::while_open(f)` | `api.surface(placement, content)`, or `show(api.is_open(), f)` |
| `use_menu_with_open(items, open)` | `use_menu(items, open)`; the `open` signal is required by every opening behavior |
| `api.control_state()`, `api.item_state(key)` and the other `*_state()` getters | `api.control().state()`, `api.item(key).state()`: `Part::state()` |
| `kivact::ui::Snap` | `kivact::interaction::Snap` |
| `kivact::interaction::ComboboxMode` | `kivact::ui::ComboboxMode` |
| `kivact::interaction::ViewTransform` | `kivact::interaction::ZoomPan` |
| `UiRecipes`, `theme.ext::<UiRecipes>()`, `recipes.button(\|previous\| ..)` | `kivact::ui::theme::ButtonTheme` and the other `<Name>Theme` tables: `theme.with(ButtonTheme::default().root(\|theme, look, base\| ..))` |
| `UiTokens` and the `*Tokens` of each component | the theme variables of each component, such as `kivact::ui::theme::button::HEIGHT_MD` |
| `*Recipes`, `*State`, `Look`, `Variant` in `kivact::ui` | `kivact::ui::theme` |
| — | `icon_button(icon, label)` |

## Async: `kivact::task`

| Before | After |
|---|---|
| `action(ActionMode::Latest, f)` | `action(f)`: a new run cancels the one in flight |
| `action(ActionMode::Queue, f)`, `action(ActionMode::QueueLatest, f)` | `queued_action(f)`: runs go one at a time, in order, and every run is kept |
| `action.close()` | gone: a queued action finishes with its owner |
| `.on_complete(\|outcome\| match outcome { ActionOutcome::Succeeded(v) => .., ActionOutcome::Failed(e) => .. })` | `.on_complete(\|result: Result<T, UiError>\| ..)` |
| a channel plus `spawn` to receive values from your thread | `let (value, sender) = kivact::task::remote(initial);`: `value` is a `Signal<T>`, `sender` is `Send` and has `set`; the UI gets the latest value once per turn, and a `set` after the component is removed is dropped |
| a capacity-1 channel to wake the UI from your thread | `let (changed, wake) = kivact::task::remote_trigger();`: `wake.notify()` never blocks |
| a timer per widget for a clock | `kivact::task::clock(every)`: one shared timer per period |
| `blocking(..)` outside a running App | reported as `WrongApp` at the call site |

## App and platform: `kivact::App`, `kivact::platform`

| Before | After |
|---|---|
| `App::new(..).fonts(fonts)`, `App::new(..).font(bytes)` | `App::new(..).theme(Theme::light().with_fonts([regular, bold]))`: each item is a `Vec<u8>`, static bytes (`include_bytes!(..).as_slice()`), a path or shared bytes |
| `kivact::launch::{GpuPreference, GpuPower, GpuBackend, GpuStart, AdapterMatch}` | `kivact::platform::{GpuPreference, GpuPower, GpuBackend, GpuStart, AdapterMatch}` |
| `prefer_gpu(false)`, `EmbedOptions.prefer_gpu` | `GpuStart::Never` in `GpuPreference` |
| `ARVANE_RENDERER=cpu` / `gpu` | `cpu`, `gpu` or `auto` = `GpuStart::Never`, `AtLaunch` or `OnDemand` |
| `app.clone()` | `App` is not `Clone`; take `App::handle()` before `run` to reach it from elsewhere |
| `kivact::AppOptions`, `kivact_core::launch::AppOptions` | the setters of `App` |
| `platform::on_close_request(f)` covered the main window only | it covers the window of the component that calls it (the main window, a `window(..)`, a foreign window) |
| `kivact::test::TestFonts::Only(fonts)` | `RenderOptions::with_theme(theme.with_fonts([font]))` |

## Embedding: `kivact::embed`

| Before | After |
|---|---|
| `kivact::embed::Visibility` | `kivact::embed::EmbedVisibility` |
| `EmbedNotice::Stopped`, then `embedded.status()` | `EmbedNotice::Stopped(EmbedStatus)`: `Closed`, `Failed(..)` or `Panicked(..)`, sent once |
| `#[cfg(..)]` around embed code on platforms without embedding | `kivact::embed` compiles everywhere; `App::embed` returns `ErrorKind::Unsupported` where embedding is not available |

## Styles and themes: `kivact::style`

| Before | After |
|---|---|
| `ColorToken::Accent`, `Color::token(ColorToken::Accent)` | `color::ACCENT` (also `SURFACE`, `SURFACE_ALT`, `TEXT`, `TEXT_MUTED`, `ACCENT_TEXT`, `BORDER`, `FOCUS`, `DANGER`, `SUCCESS`, `WARNING`, `OVERLAY`, `SHADOW`, `SELECTION`, `HOVER`, `PRESSED`, `RAISED`) |
| `ThemeColor`, `Color::of(..)` | `ThemeVar<Color>` and `Color::var(&VAR)` |
| `Palette { accent: .., ..base.colors().clone() }` and `Theme::with_colors(palette)` | `theme.set(&color::ACCENT_VAR, rgb(..))`, one call per variable |
| `Spacing`, `Theme::with_space`, `theme.space().md` | `space::MD` (`XS`, `SM`, `MD`, `LG`, `XL`); `theme.set(&space::MD_VAR, 12)` |
| `Radii`, `Theme::with_radius`, `theme.radius().md` | `radius::MD` (`SM`, `MD`, `LG`); `theme.set(&radius::MD_VAR, 6)` |
| `Theme::with_text`, `theme.text().size` | `font::SIZE` (`SIZE_SMALL`, `SIZE_LARGE`); `theme.set(&font::SIZE_VAR, 15)` |
| `theme.colors().accent` | `theme.get(&color::ACCENT_VAR)` |
| `use_tokens()` | theme variables |
| `.style(move \|\| { let t = use_theme().get(); style().padding(t.space().md) })` | `.style(style().padding(space::MD))`: no closure, the value follows the theme |
| a `ThemeExtension` that holds colors or sizes | your own `ThemeVar` statics; `ThemeExtension` stays for tables of functions |
| `theme.with_control_height(h)`, `theme.control_height()` | `theme.set(&control::HEIGHT_VAR, h)`, `theme.get(&control::HEIGHT_VAR)`; `control::HEIGHT` in a style (`kivact::style::control`) |
| `theme.with_disabled_opacity(o)`, `theme.disabled_opacity()` | `theme.set(&control::DISABLED_OPACITY_VAR, o)`, `theme.get(&control::DISABLED_OPACITY_VAR)` |
| `theme.with_overlay_margin(m)`, `theme.overlay_margin()` | `theme.set(&control::OVERLAY_MARGIN_VAR, m)`, `theme.get(&control::OVERLAY_MARGIN_VAR)` |
| `ThemeHint` | `ThemeMode` |
| `.weight(700)` | `.font_weight(700)` |
| `.italic(true)` | `.font_style(FontStyle::Italic)` |
| `.radius(8)`, `.radius_top_left(8)` and the other `radius_*` | `.border_radius(8)`, `.border_radius_top_left(8)`, `border_radius_*` |
| `.ring(2, color)`, `.ring_offset(2)`, `.ring_none()` | `.outline(2, color)`, `.outline_offset(2)`, `.outline_none()` |
| `.basis(..)`, `.grow(1)`, `.shrink(0)`, `.wrap(..)`, `.direction(..)` | `.flex_basis(..)`, `.flex_grow(1)`, `.flex_shrink(0)`, `.flex_wrap(..)`, `.flex_direction(..)` |
| `Direction::Row` | `FlexDirection::Row` |
| `.pressed(style())` | `.active(style())` |
| `text_decoration_style(move \|\| ..)` | `text_decoration_style(TextDecorationStyle::Wavy)` takes a plain value; make the whole style reactive with `.style(move \|\| ..)` |
| `color.components()`, `a.mix(b, t)` | `color.components(&theme)`, `a.mix(b, t, &theme)`; new `relative_luminance()` and `contrast_ratio(other)` |
| `Dimension`, `TextLen` | `Length`, `TextLength` |
| `GridFlow` | `GridAutoFlow` |
| `Insets` | `Edges<f64>` |
| `SpanStyle`, `TextExtras`, `DecorationStyleValue` | `TextStyle`, the one set of text properties used by paragraphs, spans and editor ranges |
| a state style that accepted only paint properties | `.hover(..)`, `.active(..)`, `.focus(..)`, `.focus_visible(..)`, `.disabled(..)`, `.selected(..)`, `.group_hover(..)`, `.group_focus(..)` accept every property |
| `Shadow`, `Filter`, `Transform`, `Paint`, gradients, `Shape`, `CornerRadius`, `BlendMode`, `Mask*`, `BorderImage*`, `PaintShader`, `FilterShader`, `FilterGraph` from `kivact::draw` | from `kivact::style` |

## Drawing: `kivact::draw`

| Before | After |
|---|---|
| `Layer::new().blend(BlendMode::Multiply)` | `Layer::new().mix(Mix::Blend(BlendMode::Multiply))` |
| `Layer::new().composite(Composite::SourceIn)` | `Layer::new().mix(Mix::Composite(Composite::SourceIn))` |
| `Offscreen` from `kivact-core` | `kivact::draw::Offscreen` (now its own crate behind the facade): `render(width, height)`, `render_svg(width, height)`, `.scale(f64)` |
| SVG export failed on font features, variations, blend modes and text decorations | they are written; only features with no SVG 1.1 or CSS equivalent fail |

## Kits and plots

| Before | After |
|---|---|
| `kivact_ui_basic::{plot, PlotModel, PlotSeries, PlotAxis, PlotRange, AxisScale, TickFormat, LineDash}` | `kivact_plot::{..}` (the crate `kivact-plot`); it works with every kit |
| a kit's `ColorName` and `Tokens` fields (`theme.ext::<Tokens>().colors.primary`) | the kit's theme variables, such as `kivact_ui_basic::PRIMARY` |
| a kit's `*Appearance`, `*Look`, `*Hue`, `*Tone` and kit-prefixed variant types | the kit's `<Component>Variant` types; see the kit's guide |

## Changes to calls and behavior

These changes were made in the same rework and are not in the tables above.

### Calls that take a value now

- `BorderImageSource::Gradient` holds a `Box<Gradient>`. Build a border image from `linear_gradient(..)` as before; you do not write the `Box`.
- `chat_thread`: `empty(content)` replaces `empty(when, content)`. The empty view shows when there are no messages and `loading_top` is off.
- `code_block`: `highlight(highlighter(content, f))` replaces `highlight(content, f)`; `highlighter(content, f)` returns the `Highlighter` that `highlight` takes.
- `tree`: `reveal(t)` turns on the reveal with a `Transition`, and `reveal_gap(gap)` sets the gap. They replace `reveal(gap, transition)`.
- `tabs`: `keep_mounted(v)` takes a `bool`, as `disclosure` does. The `tabs` root is one column that holds the list and the panel, so a modifier you add to `tabs` applies to the whole widget, not only to the list.
- `calendar`: `touch(v)` takes a `bool`; `min_span(n)` and `max_span(n)` replace `range_span(min, max)`; `min_date(d)` and `max_date(d)` replace `bounds(min, max)`.
- `splitter`: `key_step(step)` returns `Self`. A step that is not positive and finite is reported once, and the default step is used. Remove the `?` or `unwrap()` after `key_step(..)`.
- `dropzone`: `multiple(v)` and `pick_on_press(v)` take a `bool`.
- `expand_all_control(v)` takes a `bool`: write `expand_all_control(true)`.
- `empty_state`: `announce(v)` and `announce_changes(v)` take a `bool`: write `.announce(true)`.

### Behavior

- A press handler and other modifiers set on `Card`, `ChatMessage`, `Badge` or `ItemRow` are on the root, so the whole item is pressable. The parts of `use_button()` are on the root too.
- Every item of `stepper` can be pressed. Its value is the `current` signal; to block a step, use `StepperItem::disabled(true)`.
- The focus ring of a `combobox` is the `FOCUS_RING` token, not the theme's `focus_ring()`.
- `Timecode`: `step` and `precision` take the value you set, whatever the order of the calls.
- `menu` with a custom trigger: the trigger keeps its own name (`button("Custom")` is named "Custom"), and the menu's surface is labelled by the trigger. `label` names only the default trigger and the surface of a context menu.
- `SidebarMenuButton`: `test_id` and `style` are on its row (the list item).
- Soft `tab_list` no longer has a Navigation pattern. To move between pages, use `kivact::ui::navigation_menu` with its links.
- Soft `app_shell`: `side_nav` is now `sidebar`.
- Soft `app_shell`: the drawer does not close by itself when a navigation item is pressed. Call `hide()` in that press handler.
- Soft side navigation: a hover on the rail no longer shows child items; it is the core `sidebar`.
- Opening a panel only after an approval (the `on_open_change` gating) is gone: `open: Signal<bool>` is the only state of an open panel.
- `selection_field` is gone, with its search field and its content: search with a `combobox`.
- A chat composer does not recall the last sent message.

## See also

- [Write reusable components](custom-components.md): the one way to write a component.
- [Map what you know from React or SolidJS to Kivact](coming-from-react-or-solid.md): the new API in one page.
- [Change the theme and the look of components](theming.md): theme variables and component themes.
