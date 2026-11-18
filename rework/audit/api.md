# Public API audit (API)

Scope: `crates/kivact` (facade, prelude, launch), `crates/kivact-macros`, the public modules of
`kivact-core` re-exported by the facade, the surface of `kivact::interaction` / `kivact::ui`,
`examples/kivact/*`, `templates/basic`, `docs/for-kivact-user/en` (jp checked: code blocks differ
only by translated strings), `README.md`. Read-only; nothing was built.

## 0. Summary

1. **Five ways to write a component** (plain fn `-> impl Element`, plain fn `-> impl IntoView`, plain fn +
   `scope(..)`, `#[component]`, hand-written struct + `impl Element` + `impl IntoView`). `#[component]`
   cannot produce an `Element` (no `.style()` on it) and is used by **zero** kit/standard components
   (169 hand-written `impl Element for` in `kivact-ui*`, 0 `#[component]`).
2. **Two writable handles** (`Signal`, `Binding`) plus `IntoBinding`, `binding()`, `project`, `bind!`,
   `.optional()`; for_each rows switch between `Binding<T>` and `Value<T>` by source type.
3. **Handler shapes disagree**: `on_press(impl Into<Callback<()>>)`, `on_key<F,R>(Fn(KeyEvent)->R)`,
   `Callback<E>` props accept only closures (a `Callback<E>` handle must be re-wrapped), `on_press`
   allocates an arena `Callback` per node.
4. **Context is two-level**: `use_context::<T>()` returns `Value<T>` of a constant (`session.get().user.get()`),
   five near-identical `provide_*` with different reactivity rules, `use_app_state` returns `T`.
5. **Surface is too wide and leaky**: prelude ≈150 names (kit types, conversion traits, `list`/`field`/`window`
   that shadow locals); same type under 2 paths (`Owner`, `BorderImage`, `Snap`, `ComboboxMode`); docs
   reference APIs that do not exist (`Signal::field`, `kivact::view::Reactive`, `Value<Theme>::space()`).

Target shape: one authoring path (`#[component]`, returning `impl Element` when stylable), three state
handles a JS developer meets (`Signal<T>` read/write incl. lenses, `Value<T>` read-only, `Callback<E>`) plus
`Var<T>`, one handler trait accepted by every `on_*` and every callback prop, `use_context::<T>() -> T`,
a ≈75-name prelude, and each public item at exactly one path. No new rendering model: all changes are
handle representation, macro expansion and re-export layout.

## 1. Current structure map

| Crate / module (facade path) | Responsibility | LOC | Dependents | Boundary verdict |
|---|---|---|---|---|
| `kivact` (`crates/kivact`) | facade: re-exports, `prelude`, `launch` | 132 | apps, 5 kits | real; `launch` module is a misplaced 1-item re-export |
| `kivact-macros` | `#[component]` | 634 | `kivact` | real; but emits `::kivact::` paths, so `kivact-ui`/`kivact-interaction` (which depend on `kivact-core`) cannot use it — why no standard component uses it |
| `kivact::state` (`kivact-core/src/state.rs`) | re-export of `kivact-runtime` reactive + `provide`, `Owner`, `use_app_state` | 17 | all | shim; mixes app-level (`signal`) and layer-internal (`Notifier`, `Owner`, `IntoReactive`) |
| `kivact::view` | elements, flow, input events, windows, overlays, scroll timelines, drawable share | 18 532 | all | too wide (~110 exports); fine as module, but kit-only items mixed in |
| `kivact::style` | `kivact_style::facade::*` + `provide_theme` + `theme` | 23 | all | ok; overlaps `draw` |
| `kivact::draw` | canvas vocabulary + paints/filters/transform + `Offscreen` | 411 | apps, kits | overlaps `style` (`BorderImage`, `BorderImageRepeat` at both paths) |
| `kivact::custom` | kit-author contract: `Owner`, `Owned`, `Part`, `primitive`, IME, text editing, `Reactive` | 1 457 | kits | real boundary (kit authors), but `Owner` duplicated in `state` |
| `kivact::task`, `motion`, `i18n`, `diagnostics`, `platform`, `geometry` | async; animation; locale; why/errors; OS services; 6 geometry types | 15–1 149 | apps | ok |
| `kivact::embed`, `gpu_interop` (features) | plugin embed; GPU texture share | 424 / 1 370 | downstream apps | ok (experimental) |
| `kivact_core::launch` (not in facade) | `AppOptions`, `Launch`, `ClientMode`, `TestTiming`, `Inspect`, `LaunchHandle` | 1 036 | `kivact-test` only | fake: test-harness contract published as core API |
| `kivact::interaction` | headless `use_*` behaviors → `*Api` | 25 445 (≈200 pub names) | `kivact-ui`, kits | real |
| `kivact::ui` | standard components + recipe/tokens/state contracts for kits | 34 291 (≈366 pub names) | apps, kits | real, but app-facing components and kit-facing `*Recipes/*Tokens/*State` share one flat namespace |
| `examples/kivact/*`, `templates/basic` | 5 examples + template | 380 + 50 | users | contain process artefacts |
| `docs/for-kivact-user/en` | tutorial (8) + how-to (36) | 13 424 | users | teaches 3 component shapes; 3 wrong API references |

Handle / conversion catalogue (what a user can meet today):

| Name | Meaning | Overlaps with |
|---|---|---|
| `Signal<T>` / `signal(v)` | writable source; `get/with/set/update/notify/project/optional/try_get*` | `Binding<T>` (both writable), `Notifier`+`Signal::notify` |
| `Value<T>` / `memo(f)` | read-only: memo, constant, prop, `Signal` view | `Reactive<T>` (ownerless prop), `Prop<T>` (hidden), `Animated<T>`, `ThemeRef<T>` |
| `Var<T>` / `var(v)` | untracked cell (`useRef`), Copy | `Rc<RefCell<T>>` |
| `Binding<T>` / `binding(get,set)`, `bind!`, `.project`, `.optional()` | two-way: a `Signal` or a (Value, Callback) pair | `Signal<T>` |
| `Callback<E>` / `callback(f)`; `call`, `try_call`, `call_if_alive`, `bind`, `to_fn` | Copy owner-bound closure | `Retry` (another callback type), plain closures, `Handler<E>` (runtime-internal) |
| `Reactive<T>` (`kivact::custom`) | ownerless stored prop (ListItem labels, macro storage) | `Value<T>`, `Prop<T>` |
| `Notifier` / `notifier()` | value-less source | `signal(())` + `notify()` |
| `Selector<K>` / `selector` | Solid `createSelector` | — (undocumented for users) |
| `Resource<T>`, `Action<T,I>`, `Task`, `Retry`, `AsyncState`, `AsyncSnapshot` (hidden) | async | `async_view` vs `dynamic(state)` |
| `View`, `ViewFn<A>`, `Node`, `Element` (trait), `IntoView`, `IntoViewFn`, `IntoChildren` | view descriptions / factories | `ViewFn` vs `impl Fn() -> V` vs `FnOnce` across flow APIs |
| `IntoValue<T>`, `IntoText`, `IntoBinding<T>`, `IntoReactive<T>`, `IntoSelection<K>`, `ListSource<T>`, `AsyncSource<T>`, `ActionFn`, `BlockingWork`, `HandlerResult` | input-conversion traits in signatures | `IntoText` ≈ `IntoValue<String>` (needed for coherence) |
| `PropValue`, `ItemKey` | blanket aliases for `Clone+PartialEq` / `Clone+Eq+Hash` (diagnostics) | std derives |
| `constant_value!`, `bind!`, `#[component]` | macros | `constant_value!` vs `state::__private::Prop` manual impls |

Concepts a beginner meets today: **form** ≈ 22 (App/run, `impl IntoView`, signal, Signal, memo, Value,
get/with/set/update, `move`, tuple children, style(), text, text_input+IntoBinding, field+Element bound, button,
disabled, on_press, show_some+Value arg, InputPurpose, project/bind!, Binding, `.optional()`, get_untracked);
**list** +9 (derive Clone/PartialEq, for_each, row type Binding-vs-Value, key fn, var/Var, show+fallback,
checkbox, `update(|l| l.retain..)`, `#[derive]` on key); **async** +11 (resource, Resource, async move,
UiError::msg, kivact::Result, AsyncState, async_view **or** dynamic+`.into_view()`, task::sleep, action+ActionMode,
`.run(())`, on_complete/ActionOutcome); **reusable component** +10 (`#[component]`, Value<T>, Callback<E>,
`Option`+`#[prop(optional)]`, View/ViewFn/Vec<View>, Element-vs-IntoView, scope, constant_value!,
IntoText/into_text_value, with_node). Total ≈ 52. Target (section 4): ≈ 33.

## 2. Findings

### API-01 Five component-authoring shapes; `#[component]` is a second-class path
- kind: API / STRUCT / DUP
- where: `crates/kivact-macros/src/expand.rs:97-120` (always `impl IntoView`, never `Element`);
  `crates/kivact-macros/src/parse.rs:418-423` (return must be `impl IntoView`); `docs/for-kivact-user/en/how-to-guide/custom-components.md:16-20,144-175,344-361`;
  `docs/for-kivact-user/en/getting-started/02-components-and-layout.md:13-90`; `README.md:33-35`;
  `crates/kivact-ui/src/card.rs:12-85` (typical hand-written builder: struct + setters + `impl Element` + `impl IntoView`).
- evidence: docs teach `fn header() -> impl Element`, `fn x() -> impl IntoView`, `scope(move || ..)`,
  `#[component]`, and "When you need an `Element`" hand-written structs. The macro's struct "implements `IntoView`,
  not `Element`, so a caller cannot write `.style(..)`" (custom-components.md:88-91), so `field(label, control)`
  (`crates/kivact-ui/src/field.rs:60`, `C: Element`) rejects every `#[component]`. `grep "#[component]"`: only the
  component example, the template and public-API tests. 169 `impl … Element for` in `kivact-ui*`. Plain helpers
  that take props need `impl IntoText` + `.into_text_value()` + `scope` (custom-components.md:154-167), i.e. the
  conversion traits leak exactly where a beginner writes code.
- fix: `#[component]` is the only documented authoring path, for apps **and** kits.
  1. Return type `-> impl Element` makes the generated struct an `Element`: the struct gets a hidden field
     `__root: ::kivact::view::Node` (an empty `node()` that collects caller modifiers), `impl Element { fn node_mut(&mut self) -> &mut Node { &mut self.__root } }`,
     and `into_view` = `scope(move || { let body = { <stmts> }; body.with_node(|n| n.absorb(__root)) })`.
     Add `pub(crate)`-backed `Node::absorb(self, mods: Node) -> Node` in `kivact-core/src/view/elements/builders.rs`
     that appends `mods.common.style` after the body's layers, prepends handlers (same order rule as
     `prepend_key`), and overrides name/description/disabled/test_id/node_ref. `-> impl IntoView` keeps today's behaviour.
  2. Zero-argument functions are allowed (already parse) and documented: `#[component] fn header() -> impl Element`.
  3. Optional = the type is `Option<X>`; delete `#[prop(optional)]` (parse.rs:490-521 keeps only `default`).
  4. The macro resolves its crate root with `proc-macro-crate` (`kivact` → `::kivact`, else `kivact-core` →
     `::kivact_core`, with `interaction`/`ui` paths only used when present) so `kivact-ui` can use it.
  5. Move `scope` from prelude/`view` to `kivact::custom` (kit escape hatch). Rewrite tutorial step 2, custom-components.md
     "three shapes" table, README paragraph on `scope`, to one shape.
- breaking: yes   kpi: -   size: M (macro + absorb) / L (migrating kit builders is the kit auditor's work order)
- confidence: high

### API-02 Two writable handle types (`Signal`, `Binding`) and four projection spellings
- kind: API / DUP
- where: `crates/kivact-runtime/src/reactive/data.rs:592-600` (`BindingRepr::{Signal, Pair}`);
  `handles.rs:231` (`Signal::project`), `:641` (`Binding::project`), `:691/708` (`optional` twice), `:725-742`
  (`IntoBinding`), `:762-767` (`bind!`); `setup.rs:73` (`binding`); `crates/kivact-core/src/view/flow/list.rs:373-420`
  (`ListSource::Row` = `Binding<T>` or `Value<T>`); docs `05-lists-and-conditions.md:29-31,113-118`, `forms.md:118-123`.
- evidence: every input control takes `impl IntoBinding<T>`; users must learn that a `Signal` "is" a binding, that
  rows get `Binding<Task>`, and that `bind!(task.done)` ≡ `task.project(|t| &t.done, |t| &mut t.done)` (both taught).
  `Binding` already contains a `Signal` variant: it is a Signal with a second representation.
- fix: merge into one type. `pub struct Signal<T> { app: AppId, repr: SignalRepr }` with
  `enum SignalRepr { Source(Key), Lens { read: Key /*memo*/, write: Key /*Callback<T>*/ } }` (move today's
  `BindingRepr` logic into `Signal`'s `get/with/set/update/project`). `binding(get, set) -> Signal<T>`,
  `bind!(s.a.b) -> Signal<U>`, `Signal::optional() -> Signal<Option<T>>`. Delete `Binding`, `IntoBinding`,
  `IntoSelection` impls for `Binding` (keep `Signal<Option<K>>`/`Signal<Vec<K>>`). Controls take
  `value: Signal<T>` (concrete, no generic). `Signal::project` becomes `#[doc(hidden)]` (macro target); guides use `bind!` only.
  `ListSource for Signal<Vec<T>>` rows are `Signal<T>`; readonly sources keep `Value<T>`.
- breaking: yes   kpi: -   size: M
- confidence: high

### API-03 Event-handler and callback-prop shapes are inconsistent; `on_press` allocates per node
- kind: API / INEFF
- where: `crates/kivact-core/src/view/element_data.rs:422-472` (`on_key<F,R>` vs `on_press(impl Into<Callback<()>>)`);
  `crates/kivact-runtime/src/reactive/handles.rs:744-759` (`From<F> for Callback<()>` inserts an arena object);
  `crates/kivact-macros/src/expand.rs:217-223` (`Callback<()>` prop takes `Into<Callback<()>>`, `Callback<E>` prop
  takes only `impl Fn(E) -> R`); `custom-components.md:95` ("forward a parent's `on_pick: Callback<i32>` as
  `picker(move |n| on_pick.call(n))`"); `crates/kivact-ui/src/text_input.rs:181,227,239` (three shapes on one builder).
- evidence: a nullary handler accepts a `Callback` handle, a payload handler does not; every `.on_press(closure)`
  creates an owner-registered `CallbackFn` (`insert_object`) and then wraps it in a second closure (`set_press(move || f.call(()))`).
- fix: one sealed trait in `kivact-runtime/src/reactive/handler.rs`:
  `pub trait IntoHandler<E, M>: Sealed<M> { fn into_handler(self) -> Handler<E>; }` implemented for
  `F: Fn() -> R` (E = (), M = `Nullary`), `F: Fn(E) -> R` (M = `Unary`), `Callback<E>` (M = `Handle`), `R: HandlerResult`.
  `Handler<E>` = `Rc<dyn Fn(E) -> Result<Handled, UiError>>` (no arena object). Every `on_*` on `Element`, every
  component builder, and macro `Callback<E>` props take `impl IntoHandler<E, M>`. `callback(f)` stays the only way to
  make a shareable `Callback<E>`. Delete `From<F> for Callback<()>`.
- breaking: yes   kpi: K2 (one arena object + Rc per pressable node), K1   size: M
- confidence: high (shape), medium (allocation cost magnitude)

### API-04 Context returns a `Value` of a constant; five `provide_*` with different rules
- kind: API / DUP / CONTRA
- where: `crates/kivact-core/src/view/boundary.rs:71-85` (`provide(value: T)` stores `owner.constant`),
  `:102,142,179` (`provide_theme/locale/messages` take `impl IntoValue<_>`), `:220` (`provide_direction` takes a
  constant), `:230` (`use_direction`); `crates/kivact-runtime/src/reactive/setup.rs:188-212` (`use_context -> Value<T>`);
  `crates/kivact-core/src/view/owner.rs:141` (`use_app_state -> T`); `coming-from-react-or-solid.md:327` (`session.get().user.get()`).
- evidence: `provide` can only hold a constant, so the `Value<T>` it returns never changes; the user writes `.get()`
  twice and pays a clone per read. `use_app_state` (same concept, App scope) returns `T`.
- fix: `use_context::<T: Clone>() -> T`, `try_use_context::<T>() -> Option<T>` (clone of the provided value;
  reactivity comes from handles inside `T`, as in Solid). `provide(value: T, child)` unchanged.
  `provide_direction(dir: impl IntoValue<ReadingDirection>, child)` like the other three. Keep the four specialised
  providers (they install non-context state: theme ctx, layered messages) but implement them through one private
  `fn provide_with(owner_setup: impl FnOnce(Owner), child)` to delete the four copied bodies (boundary.rs:102-215).
- breaking: yes   kpi: K3 (minor)   size: S
- confidence: high

### API-05 Prelude is ≈150 names, including kit-level types and conversion traits
- kind: API
- where: `crates/kivact/src/prelude.rs:1-56`
- evidence: exports `IntoBinding, IntoText, IntoValue, ItemKey, PropValue, constant_value, IntoSelection, Selection,
  EditPhase, Modifiers, RowHeight, ScrollAxes, WindowMode, WindowPlacement, ThemeColor, ThemeMode, use_tokens,
  DataColumn, PageItem, StepStatus, StepperItem, breadcrumb, pagination, stepper, navigation_menu, toggle_group,
  calendar, date_range_input, time_input, table_head, table_footer, item_row, …`. The guide spends a section
  (Rust point 7, coming-from-react-or-solid.md:520-540) on prelude names (`list`, `field`, `message`, `window`)
  shadowing locals.
- fix: prelude = the list in section 4.2 (≈75 names). Everything else is reached through its module
  (`kivact::ui::data_table`, `kivact::state::Machine`, …). Docs examples add explicit `use` lines.
- breaking: yes   kpi: -   size: S
- confidence: high

### API-06 Own prop types need `constant_value!(T)` in the defining crate
- kind: API (Rust-ism)
- where: `crates/kivact-runtime/src/reactive/props.rs:499-516`; `custom-components.md:36,99-142`; 62 uses in
  `kivact-ui*`/`interaction`; `crates/kivact-core/src/state/__private.rs:1-3` + `crates/kivact-interaction/src/scalar/range.rs:13-42`
  (manual `IntoValue` impls via `__private::Prop`).
- evidence: "Without `constant_value!(Tone)`, `badge("Saved", Tone::Info)` fails to compile." The blanket
  `impl<F: Fn() -> T> IntoValue<T> for F` makes `impl<T> IntoValue<T> for T` incoherent, so the macro is structural.
  `__private` claims to be "Macro expansion support only" but the macro (`$crate::Prop`) never uses it; only
  interaction's hand-written impls do.
- fix: add `#[derive(Prop)]` to `kivact-macros` (emits `impl IntoValue<Self> for Self { fn into_prop(self) -> Prop<Self> { Prop::constant(self) } }`
  through the resolved crate path of API-01.4; no `IntoReactive` impl, so constants stay rejected by `show`). Users write
  `#[derive(Clone, PartialEq, Prop)]`. `constant_value!` stays `#[doc(hidden)]` for `kivact-runtime`/`kivact-types`
  primitives only. Replace interaction's manual `__private::Prop` impls with `constant_value!`/`From`; rename
  `__private` to what it is (`kivact::custom::Prop`, hidden) or delete it once unused.
- breaking: yes   kpi: -   size: S
- confidence: high

### API-07 Documentation references APIs that do not exist
- kind: CONTRA
- where / evidence / fix:
  - `docs/for-kivact-user/en/how-to-guide/forms.md:95` "`Signal::field` makes a `Binding`" — no `field` method exists
    (`grep "pub fn field"` in runtime: none); the code below uses `project`. Fix: say `bind!(draft.name)` and use it in the snippet.
  - `crates/kivact-runtime/src/reactive/props.rs:352` "Use the public facade's `kivact::view::Reactive`" — it is
    `kivact::custom::Reactive` (`crates/kivact-core/src/custom.rs`, macro `expand.rs:236`). Fix the path.
  - `crates/kivact-core/src/view/element_data.rs:248-252` doc: `let t = &use_theme(); style().padding(t.space().md)`
    — `use_theme()` returns `Value<Theme>`, which has no `space()`. Fix: `use_theme().with(|t| style().padding(t.space().md))`.
  - `crates/kivact-core/src/lib.rs:10` "created by … or by `Type::new`" — `Signal/Value/Var/Callback` have no `new`. Delete the clause.
  - `crates/kivact-core/src/state.rs:12` comment "Macro expansion support only" for `__private` (see API-06).
  - `README.md:33-35` teaches `scope(move || view)` as the component boundary; the guides teach `#[component]`. Rewrite per API-01.
- breaking: no   kpi: -   size: S
- confidence: high

### API-08 Async has two display paths; tutorial uses the one that needs `.into_view()`
- kind: DUP / API
- where: `docs/for-kivact-user/en/getting-started/07-async.md:139-160` (`dynamic(move || saved.state(), …match… .into_view())`)
  vs `examples/kivact/async/src/main.rs:26` and `http-json.md:33` (`async_view(user, text)`); `crates/kivact-ui/src/async_view.rs:16-53`.
- evidence: the tutorial teaches a `match` with four `.into_view()` arms for exactly what `async_view(..).loading(..).fallback(..)`
  does. `async_view` lives in `kivact::ui` while `resource/action` live in `kivact::task`.
- fix: tutorial step 7 uses `async_view(saved, move |tasks| tasks_screen(dark, tasks)).loading(|| …).fallback(|err, retry| …)`.
  `async_view.loading/.fallback` take `impl IntoViewFn<V>` / `Fn(UiError, Retry) -> V` like `show`/`error_boundary`.
  Keep `AsyncState` + `dynamic` documented only as the escape hatch in async-and-tasks.md.
- breaking: no   kpi: -   size: S
- confidence: high

### API-09 Process artefacts and Rust noise in user-facing examples
- kind: PROCESS
- where: `examples/kivact/component/src/main.rs:33,35` (two 150-char `// xtask-allow(state-fields): …` lines on
  `let first = signal(true)`); `#![warn(redundant_imports)]` in all 5 examples + template (`examples/kivact/*/src/main.rs:2`,
  `templates/basic/src/main.rs:6`) and catalogs; `//! Role:` headers in every example and test module;
  `crates/kivact-interaction/src/list/data.rs:305-308` (`xtask-allow` on public struct fields, visible on go-to-definition).
- evidence: a user copying the component example copies a lint justification for an internal rule; the state-fields
  rule fires on two independent local signals of an app, which is not an implicit state machine.
- fix: delete both `xtask-allow` lines; restrict the xtask state-fields check to `crates/` (exclude `examples/`,
  `templates/`, `docs/`). Move `redundant_imports = "warn"` into `[workspace.lints.rust]` and delete the per-file
  attributes. Drop `//! Role:` from examples/templates (exclude them from that xtask rule).
- breaking: no   kpi: -   size: S
- confidence: high

### API-10 Same item at several paths; colliding names
- kind: DUP / API
- where: `Owner`: `crates/kivact-core/src/state.rs:4` and `custom.rs` (`pub use crate::view::owner::Owner`);
  `BorderImage`, `BorderImageRepeat`: `kivact_style::facade` (style) and `crates/kivact-core/src/draw.rs` (draw);
  `Snap`: `crates/kivact-ui/src/lib.rs:265` and `kivact-interaction/src/lib.rs:153`; `ComboboxMode`: ui `lib.rs` and
  interaction; `Visibility`: two different enums (`kivact_types::layout::Visibility` in `style`,
  `kivact_types::embed::Visibility` in `embed`); `ViewTransform` (interaction zoom-pan, = `kivact_types::ViewTransform`)
  vs `draw::Transform`; `Direction` (flex) vs `ReadingDirection`.
- evidence: rustdoc shows each twice; users cannot tell which import is canonical.
- fix: one path each: `Owner` → `custom` only; paint/filter/transform/border-image types → `style` only, canvas
  types (`Draw, Path, Stroke, Layer, Text*`, `Offscreen`) → `draw` only; `Snap`, `ComboboxMode` → `interaction`
  only (ui re-export deleted); rename `embed::Visibility` → `embed::HostVisibility`; rename interaction
  `ViewTransform` → `ZoomPan` (it is the zoom-pan state, not a CSS transform).
- breaking: yes   kpi: -   size: S
- confidence: high

### API-11 Public items with no plausible user
- kind: DEAD
- where / fix:
  - `Callback::to_fn` (`handles.rs:571`, only user: `dev/tests/kivact-public-api/tests/e2e/runtime.rs:210`),
    `Callback::bind` (`:551`, only that test), `Callback::call_if_alive` (`:528`, only `element_data.rs:325`) → delete
    `to_fn`/`bind`, make `call_if_alive` `pub(crate)`-equivalent (`#[doc(hidden)]` since core is another crate).
  - `Notifier`/`notifier()` in `kivact::state` (layer tool with `notify_from_render`, `trigger.rs:20-80`) → move to `custom`.
  - `IntoReactive` in `kivact::state` (only a bound of `show/show_some/dynamic`) → `#[doc(hidden)]`.
  - `PropValue`, `ItemKey`, `IntoSelection`, `Selection`, `IntoText`, `IntoValue`, `IntoBinding` in prelude → remove (API-05).
  - `UiThreadOnly` in `custom` (marker; no external use) → private.
  - `kivact_core::launch::{AppOptions, Launch, ClientMode, TestTiming, Inspect, LaunchHandle}` (`launch.rs`) — public only
    for `kivact-test` → `#[doc(hidden)] pub mod __test` (or move the harness contract into `kivact-test`); facade
    `kivact::launch` (GPU preference only, `crates/kivact/src/launch.rs`) → move the five types to `kivact::platform`.
  - `App` mirrors every `AppOptions` setter (`crates/kivact-core/src/app.rs:15-110` vs `launch/settings.rs:46-160`) → `App` keeps the setters, `AppOptions` becomes the private field type.
- breaking: yes   kpi: K6 (negligible)   size: S
- confidence: high

### API-12 `use_theme().get()` clones the whole theme on every style recompute
- kind: INEFF
- where: `crates/kivact-style/src/theme/value.rs:205-218` (`Theme` owns `Extensions` Vec, `Palette`, `Typography`,
  `IconSet` by value); guides `custom-components.md:401`, `theming.md:58,116,447` teach `use_theme().get()` inside style closures.
- evidence: every `style(move || …use_theme().get()…)` re-run clones the extension Vec and nested structs.
- fix: `pub struct Theme(Arc<ThemeData>)` with copy-on-write builder methods (`Arc::make_mut`); `get()` becomes a refcount
  bump, so the JS-natural `.get()` stays cheap. No API change.
- breaking: no   kpi: K3, K5   size: S
- confidence: medium (clone cost not measured)

### API-13 Inherent builder methods shadow `Element` methods with different semantics
- kind: API / CONTRA
- where: `crates/kivact-ui/src/card.rs:58,71` (`on_press`, `disabled`), `stepper.rs:80`, `avatar.rs:87`, `badge.rs:45`,
  `chat_message.rs:79`, `item_row.rs:103`, `navigation_link.rs:37` (9 shadowing methods).
- evidence: `Element::disabled` documents that components read it with `accessible_disabled`
  (element_data.rs:135-145); `Card::disabled` stores its own `Value<bool>` instead. Generic code `fn f(e: impl Element)
  { e.disabled(true) }` takes the other path.
- fix: delete the inherent `disabled`/`on_press`; builders read `root.accessible_disabled()` and the press action at
  `into_view` (as `Element` docs prescribe). xtask check: no inherent method on an `Element` implementor may share a
  name with an `Element` method.
- breaking: no (same call syntax)   kpi: -   size: S
- confidence: medium (did not trace every builder's behaviour difference)

### API-14 `text_area` lacks `#[track_caller]`
- kind: BUG
- where: `crates/kivact-ui/src/text_input.rs:60-72` (`text_input` has it, `text_area` does not; both call
  `into_text_value()`, which locates `SetupOnly` errors at `Location::caller()`).
- fix: add `#[track_caller]` to `text_area`; xtask: every pub fn in `kivact-ui` that calls `into_value`/`into_text_value`/`into_binding` carries it.
- breaking: no   kpi: -   size: S
- confidence: high

### API-15 `Signal::set` on a disposed signal is ignored, `update` panics
- kind: CONTRA
- where: `crates/kivact-runtime/src/reactive/handles.rs:182-218` ("`set` and `notify` ignore it instead").
- evidence: two writes of one handle have opposite disposal semantics; a JS developer swapping `set(x)` for
  `update(|v| *v = x)` changes crash behaviour.
- fix: both record the same located `WriteAfterDispose` diagnostic and return without panicking. `update<R>` becomes
  `-> Option<R>` (`None` when disposed, `f` not run). Docs state "writes to a removed component's state are ignored and reported".
- breaking: yes (`update` return type)   kpi: -   size: S
- confidence: high

### API-16 Child-factory parameters use four different closure shapes
- kind: API
- where: `boundary.rs:50` (`scope: FnOnce`), `:71` (`provide: FnOnce`), `flow/branch.rs:168,183` (`IntoViewFn`),
  `:345` (`show_some: Fn(Value<T>)`), `flow/error_boundary.rs:52,64`, `ui/async_view.rs:30,35` (`Fn()`), `window_view.rs:67`.
- evidence: a `ViewFn` prop can be passed to `show` but not to `async_view(..).loading(..)` or `provide`.
- fix: every "content built on open" parameter takes `impl IntoViewFn<V>` (nullary) or `impl Fn(Arg) -> V`
  (payload); `provide`/`scope` keep `FnOnce` (run exactly once). Document the rule in `view/descriptor/factory.rs`.
- breaking: yes (minor)   kpi: -   size: S
- confidence: high

### API-17 A labelled field repeats its label
- kind: API
- where: `docs/for-kivact-user/en/how-to-guide/forms.md:24` (`field("Name", text_input("Name", name))`),
  `crates/kivact-ui/src/field.rs:60`, `text_input.rs:61` (label = invisible accessible name, required).
- evidence: `field` renders the visible label and names the control, but the control already required a name; JS
  developers expect `<label>` + `<input>` once.
- fix: see open question Q2 (requires an a11y-policy decision).
- breaking: yes   kpi: -   size: S
- confidence: high

### API-18 `#[component]` recognises prop kinds by the last path segment
- kind: BUG (latent)
- where: `crates/kivact-macros/src/parse.rs:540-570`
- evidence: `type Title = Value<String>; fn c(title: Title)` becomes `PropKind::Other` (`impl Into<Title>`, no
  constant/closure acceptance); a user type named `Callback<X>` or `Binding<X>` is captured as the kivact kind.
  Signature types are never type-checked as written (the struct field types are re-derived), which is also why the
  component example compiles while `ViewFn` is not imported.
- fix: document "props use the literal names `Value`, `Signal`, `Callback`, `View`, `ViewFn`, `Vec<View>`; aliases are
  plain props", and make the macro emit a `const _: fn() = || { let _: ::kivact::state::Value<T>; };`-style assertion
  per recognised prop so a foreign `Value`/`Callback` type fails with a clear error.
- breaking: no   kpi: -   size: S
- confidence: high

### API-19 `kivact::ui` mixes app components and kit contracts in one flat namespace
- kind: STRUCT
- where: `crates/kivact-ui/src/lib.rs` (≈366 names: `button` next to `DataTableRowStatusPaint`, `ButtonGroupMemberState`,
  `*Recipes`, `*Tokens`, `*State`, `provide_control_size`, `use_input_group_state`).
- fix: `kivact::ui` = component functions, their builder structs and app-facing enums (`ButtonVariant`, `ControlSize`, …);
  `kivact::ui::recipe` = `*Recipes`, `*Tokens`, `*State`, `UiRecipes`, `Recipe`, `Look`, `Variant`. Same split for
  interaction: app-facing data (`ListItem`, `MenuEntry`, `Toast`, `Tone`) at `kivact::interaction`, `use_*` + `*Api`
  behaviours stay (kit authors). Rename `toaster()`/`announcer()` → `use_toaster()`/`use_announcer()` (they read an
  App-wide service, like `use_context`).
- breaking: yes   kpi: -   size: M (paths only)
- confidence: high

## 3. Target structure for this area

Crates are unchanged in this area (the facade/macros/core boundaries are real); changes are module layout:

```
kivact (facade)
  App, Result, UiError, ErrorKind
  component (attr), Prop (derive), bind! (macro)
  prelude            ≈75 names (4.2)
  state              Signal, Value, Var, Callback, signal, memo, var, callback, binding, effect, effect_on,
                     on_mount, on_cleanup, untrack, provide, use_context, try_use_context, use_app_state,
                     selector/Selector, Machine/MachineCx/MachineHandle/Step/machine
  view               elements, flow (show, show_some, dynamic, for_each, for_index, virtual_list, error_boundary),
                     IntoView, IntoViewFn, Element, View, ViewFn, Node, events, NodeRef, overlay, window
  style              Style, style(), colors, tokens, Theme(Arc), provide_theme, use_theme, use_tokens,
                     paints, filters, Transform, BorderImage   (single home for paint vocabulary)
  draw               Draw, Path, Stroke, Layer, Text*, Offscreen   (canvas only)
  task               resource, action, spawn, blocking, sleep, timeout, on_tokio, Resource, Action, AsyncState, Retry
  motion, i18n, diagnostics, geometry   (unchanged)
  platform           + GpuPreference/GpuStart/GpuPower/GpuBackend/AdapterMatch (from deleted `launch`)
  ui                 components; ui::recipe for kit contracts
  interaction        app data types; use_* behaviours for kit authors
  custom             kit-author contract: Owner, Owned, Part, primitive, Reactive, Notifier, scope, raise, IME, editing
  test / embed / gpu_interop (features)
```

Deleted: `kivact::launch`; `Binding`, `IntoBinding`; `From<F> for Callback<()>`; `Callback::{bind,to_fn}`;
`#[prop(optional)]`; `state::__private` (after API-06); duplicate re-exports of API-10; `kivact_core::launch` public
surface (→ hidden test module). `constant_value!` becomes hidden (derive replaces it for users).

## 4. Public API

### 4.1 Before / after (written the way a SolidJS developer expects)

**Reusable component with optional callback, children, caller styling** (API-01, API-03)
```rust
// today: Option + #[prop(optional)] twice; result cannot be styled or put in field()
#[component]
fn card(title: Value<String>, #[prop(optional)] on_close: Option<Callback<()>>, content: ViewFn) -> impl IntoView { .. }
card("First", || text("..")).on_close(move || first.set(false))            // no .style(..)

// proposed
#[component]
fn card(title: Value<String>, on_close: Option<Callback<()>>, content: ViewFn) -> impl Element {
    let open = signal(true);
    column((
        row((text(title), on_close.map(|close| button("Close").on_press(close)))),
        show(open, content),
    ))
}
card("First", || text("..")).on_close(move || first.set(false)).style(style().width(320))
```

**A no-props helper** (API-01)
```rust
// today: choose between `fn header() -> impl Element`, `-> impl IntoView`, or `scope(move || ..)`
fn click_counter() -> impl IntoView { scope(move || { let count = signal(0); .. }) }
// proposed: one shape, always its own setup owner
#[component]
fn click_counter() -> impl Element { let count = signal(0); .. }
```

**Two-way field binding in a list row** (API-02)
```rust
// today
fn task_row(tasks: Signal<Vec<Task>>, task: Binding<Task>) -> impl Element {
    let id = task.get_untracked().id;
    let done = bind!(task.done);          // or task.project(|t| &t.done, |t| &mut t.done)
    ..
}
let plan = draft.project(|p| &p.plan, |p| &mut p.plan).optional();
// proposed
#[component]
fn task_row(tasks: Signal<Vec<Task>>, task: Signal<Task>) -> impl Element {
    let id = task.get().id;               // setup is untracked; no get_untracked needed
    checkbox(move || task.get().title, bind!(task.done)); ..
}
let plan = bind!(draft.plan).optional();  // Signal<Option<Plan>>
```

**Context** (API-04)
```rust
// today
let session = use_context::<Session>();
text(move || format!("Signed in as {}", session.get().user.get()))
// proposed (Solid: const session = useContext(SessionCtx))
let session = use_context::<Session>();
text(move || format!("Signed in as {}", session.user.get()))
```

**Forwarding a payload callback** (API-03)
```rust
// today
#[component] fn picker(on_pick: Callback<i32>) -> impl IntoView { .. }
picker(move |n| on_pick.call(n))
// proposed: closures and Callback handles are accepted everywhere
picker(on_pick)
text_input("Search", query).on_key(on_shortcut)          // Callback<KeyEvent>
```

**Async screen** (API-08)
```rust
// today (tutorial)
dynamic(move || saved.state(), move |state| match state {
    AsyncState::Idle | AsyncState::Pending => text("Loading tasks...").role(Role::Status).into_view(),
    AsyncState::Failed(error) => column((..)).into_view(),
    AsyncState::Ready(tasks) => tasks_screen(dark, tasks).into_view(),
})
// proposed (Solid: <Suspense fallback><ErrorBoundary>)
async_view(saved, move |tasks| tasks_screen(dark, tasks))
    .loading(|| text("Loading tasks...").role(Role::Status))
    .fallback(|error, retry| column((text(format!("Error: {error}")), button("Try again").on_press(move || retry.run()))))
```

**Own prop type** (API-06)
```rust
// today
#[derive(Clone, Copy, PartialEq)] pub enum Tone { Info, Warning }
constant_value!(Tone);
// proposed
#[derive(Clone, Copy, PartialEq, Prop)] pub enum Tone { Info, Warning }
```

**Example file** (API-09)
```rust
// today
#![warn(redundant_imports)]
// xtask-allow(state-fields): independent: whether the First card is shown, cleared only by …
let first = signal(true);
// proposed
let first = signal(true);
```

Rust-isms that remain by necessity and are documented once (coming-from-react-or-solid.md "Rust points"):
`move` on stored closures, `-> impl IntoView/Element`, `#[derive(Clone, PartialEq)]` on data, `.into_view()` in
heterogeneous `match` arms inside `dynamic`, `use_context::<T>()`, `signal(None::<T>)`. Removed from user code by this
plan: `.clone()` before closures (already absent), `impl IntoText`/`.into_text_value()`, `Binding`, `Option<Callback<()>>`
with `#[prop(optional)]`, `move |n| cb.call(n)` re-wrapping, `constant_value!`, `get_untracked` in setup,
`scope(move || ..)`, `xtask-allow` comments. Also simplify `http-json.md:24-31`: `on_tokio` returns
`Result<T, UiError>` via `?` so the `Ok::<_, UiError>(user)` turbofish disappears (make `on_tokio`'s closure output
`Result<T, E> where E: Into<UiError>`).

### 4.2 Target prelude (complete list)

`App, UiError, component, Prop, bind,`
`Signal, Value, Var, Callback, signal, memo, var, callback, binding, effect, effect_on, on_mount, on_cleanup, untrack, provide, use_context,`
`IntoView, Element, View, ViewFn, Role, Key, KeyEvent, PointerEvent, Handled, CheckedState, InputPurpose, NodeRef, node_ref,`
`node, text, column, row, stack, grid, scroll, spacer, image, icon, window,`
`show, show_some, dynamic, for_each, for_index, virtual_list, error_boundary,`
`Style, style, Color, ColorToken, Theme, Align, Justify, pct, rgb, use_theme, provide_theme,`
`Resource, Action, ActionMode, AsyncState, resource, action, spawn,`
`button, text_input, text_area, checkbox, switch, select, radio_group, slider, field, dialog, menu, tabs, tooltip,`
`list, async_view, toast_region, use_toaster, ListItem, MenuEntry, Toast, ButtonVariant, message`.

### 4.3 Names to delete / merge / rename / move

| Item | Action |
|---|---|
| `Binding<T>`, `IntoBinding<T>` | merge into `Signal<T>` (lens repr) |
| `binding(get, set)` | keep, returns `Signal<T>` |
| `Signal::project`, `Binding::project` | `#[doc(hidden)]` (target of `bind!`) |
| `#[prop(optional)]` | delete (`Option<T>` means optional) |
| `constant_value!` | hidden; users use `#[derive(Prop)]` |
| `From<F> for Callback<()>`, `Callback::{bind, to_fn}` | delete; `call_if_alive` hidden |
| `use_context`, `try_use_context` | return `T` / `Option<T>` |
| `scope` | move to `kivact::custom` |
| `Notifier`, `notifier`, `Owner` (state path), `UiThreadOnly` | `custom` only / private |
| `IntoReactive`, `PropValue`, `ItemKey` | hidden (diagnostic-only bounds) |
| `kivact::launch` | delete; types to `kivact::platform` |
| `style::BorderImage*` vs `draw::BorderImage*`, paint/filter/`Transform` in `draw` | `style` only |
| `ui::Snap`, `ui::ComboboxMode` | delete re-export (interaction path) |
| `embed::Visibility` | `embed::HostVisibility` |
| `interaction::ViewTransform` | `interaction::ZoomPan` |
| `toaster`, `announcer` | `use_toaster`, `use_announcer` |
| `*Recipes`, `*Tokens`, `*State`, `Look`, `Variant`, `Recipe` in `kivact::ui` | `kivact::ui::recipe` |
| `kivact_core::launch::{AppOptions, Launch, ClientMode, TestTiming, Inspect, LaunchHandle}` | `#[doc(hidden)] __test` |
| `Signal::update` | returns `Option<R>`; disposed write = diagnostic, not panic |
| inherent `disabled`/`on_press` on 9 ui builders | delete (use `Element`'s) |

Resulting concept count for form + list + async + reusable component ≈ 33 (from ≈ 52): one writable handle,
one handler shape, one component shape, one async view, no conversion traits in user code.

## 5. Open questions for the owner

- Q1. Rename `Value<T>`? It collides with `serde_json::Value` in the HTTP guide and says nothing about read-only-ness.
  Candidates: `Memo<T>` (wrong for constants/props), `Read<T>`, `Accessor<T>` (Solid). The rename is mechanical but touches
  thousands of lines; this audit keeps `Value<T>` unless the owner wants it.
- Q2. Accessible name of inputs (API-17): (a) keep the required first argument and let `field(label, control)` take a
  closure `field("Name", |name| text_input(name, value))`; or (b) make the name optional (`text_input(value).label("Name")`),
  have `field` supply it, and report a located diagnostic at mount for a focusable control with no accessible name
  (a11y enforced at run time and in `kivact::test`, not by the signature). (b) is what JS developers expect; it weakens the
  compile-time guarantee.
- Q3. Should `#[component]` become mandatory for `kivact-ui` and the five kits (API-01, L-size migration of 169
  builders), or stay the app path while kits keep hand-written `Element` builders? "One way" says mandatory; it needs
  the `-> impl Element` expansion and crate-path resolution first.
- Q4. `action(ActionMode::Latest, f)` makes the mode mandatory and nullary actions run with `.run(())`. Keep (explicit),
  or `action(f)` defaulting to `Latest` plus `queued_action(f)`? Not decided here because both are defensible.
