# kivact-interaction / kivact-ui audit (prefix UI)

## 0. Summary

1. **Per-item reactive cost is O(N) or O(N·S) where O(1) is available** (UI-02). Every list, combobox, menu and data-table item or cell builds 4 to 16 memos. Some do a linear `find` over all items, and others call `contains` on the selection Vec. The core `selector` (Solid `createSelector`) is never used. In a data table, one selection change re-runs R×C memos, each O(S). This is the main data-heavy use case (K5, K2).
2. **The "attached part state" machinery is dead weight** (UI-01). There are 23 `Signal<Option<Value<S>>>` slots, the per-collection `HashMap<K, Value<State>>` maps (list, combobox, menu, table rows, table cells) and 34 `*_state()` getters, and each getter allocates a new memo per call. Outside interaction there is exactly one caller (`text_input.rs:277`). Each item mount writes a HashMap signal.
3. **The UiRecipes/UiTokens monolith links every component into every app** (UI-03). The table holds 80 components and about 300 recipe closures. It mixes flat recipes with sub-tables, and its look code is spread over 46 side files (`look_*`, `*_values`, `*_tokens`, `group_*`, `toolbar_look`, about 3.8k lines) under five placement conventions (UI-14) (K6, K1).
4. **The interaction/ui boundary leaks both ways** (UI-04). ui holds 20 `impl Machine` (gestures, drafts, swipe, typeahead-like logic). interaction renders views (`Announcer::outlet`, `while_open`, `Toast::icon`) and owns policies and enums used only by ui (`SwitchDrag`, `ToastSwipe`, `DrawerPolicy`, `SelectPolicy`, `ComboboxMode`).
5. **Implicit state machines remain in the chain that is supposed to be the cleanest** (UI-05..UI-11). The combobox open state has two owners, synced by effects. Popup close has two code paths. The carousel syncs index and scroll in both directions with two effects. Typeahead has three owners and a `Cell<Binding>` injected into a Copy handle. There are unit-state "machines", host-disabled registries, and configuration stored in machine state.

**Target shape.** Keep two crates, and make the boundary checkable:
- interaction owns every input-driven state (`impl Machine`) and returns no views except a single `surface()` portal helper.
- ui owns only builders, recipe state and default looks.

Every collection keeps one key index plus `selector`s. Each item creates at most two memos, with no per-item maps and no `*_state()` getters (`Part::state()` is the only way). Each component is one module, `<name>/mod.rs` plus `<name>/look.rs`, with one `ThemeExtension` per component (`ButtonTheme` = recipes + tokens). `UiRecipes` and `UiTokens` are deleted. `.variant()` becomes a single `Element` method, and the open/close API is one uniform vocabulary.

## 1. Current structure map

| crate / module | responsibility | LOC | depended on by |
|---|---|---|---|
| `kivact-interaction` | headless behaviors: Api handles, Parts, Machines, InteractionPolicy (a ThemeExtension) | 25,445 | `kivact` (facade `kivact::interaction`), `kivact-ui`; the kits only through the facade |
| ├ menu (+menubar, menu_policy) | menu sessions, levels, press-drag, submenu corridor | 2,911 + 260 | ui menu; kits basic/soft |
| ├ table | `use_data_table_model` (filter/sort/page) + `use_data_table` (grid focus) | 2,204 | ui data_table |
| ├ scalar, range_scalar | slider, spin button, touch scrub | 2,052 + 219 | ui slider/number_input; kits |
| ├ list, roving, selection | listbox, roving tab stop, typeahead | 2,014 + 419 | ui list/radio/select/menu/tabs/tree; kits |
| ├ drag, drag_settle, drag_velocity | pointer drag session + snap math | 1,332 | ui + 13 kit files |
| ├ combobox, palette, select | filtered list, command palette, select | 1,003 + 534 + 982 | ui |
| ├ popup, disclosure, dialog, tooltip, delayed, hover_corridor, hover_card, navigation_menu | the opening chain | ~2,600 | ui; kits |
| ├ calendar, text_edit, toaster, zoom_pan, space_pan, splitter, tabs, toggle, tree, … | other behaviors | ~6,000 | ui; kits |
| `kivact-ui` | ~80 public constructors, builders, recipe state, default looks, recipes, tokens | 34,291 | `kivact` facade |
| ├ date/calendar family (calendar, date_input, date_range_input, date_time_input, time_input, date_wheel, date_sheet…) | | 4,202 | 6 kits |
| ├ data_table + column types | | 4,592 | kits |
| ├ recipes.rs + tokens.rs + values.rs + variants.rs + layer.rs + 46 side look/values/tokens files | theming | 6,321 | kits (every theme) |
| ├ chat_*, code_*, typing_indicator, composer_history | Soft/Pop domain | 2,060 | 2–4 kits |
| ├ pagination, breadcrumb, stepper, navigation_* | | 2,423 | kits |
| ├ combobox/select/selection_field | | 2,052 | kits |

Where the boundary between the two crates is wrong:
- **Fake in places.** ui contains behavior: 20 Machines in `alert.rs`, `button.rs`, `calendar/navigation.rs`, `composer_history.rs`, `data_table/filters.rs`, `data_table/row_drag.rs`, `date_input/machine.rs`, `date_range_draft.rs`, `date_sheet.rs`, `date_time_model.rs`, `drawer_swipe.rs`, `empty_state_changes.rs`, `number_input.rs`, `pagination_draft.rs`, `progress.rs`, `select/machine.rs`, `selection_field.rs`, `switch_drag/machine.rs`, `time_input/machine.rs` and `toast_swipe/machine.rs`. interaction contains views and ui-only vocabulary.
- **Missing.** No module owns "item index / per-key selection". Each collection behavior re-implements key→item lookup, selection membership and the slot maps.
- **Real.** The headless public API is the contract the kits build on (P-07, strategy). It is worth keeping as a crate once the leaks are fixed.

## 2. Findings

### UI-01 Part-state slots, item-state maps and `*_state()` getters are unused overhead
- kind: DEAD / INEFF
- where:
  - `kivact-interaction/src/part_slot.rs:11` (23 callers)
  - `disclosure.rs:55-57,91-113`, `popup/api.rs:104-106,216-238`, `tooltip/api.rs:131-133,226-248`, `combobox/handle.rs:114-117,324-358,729-741`
  - `list/state.rs:37-39`, `list/parts.rs:192-211`, `menu/level.rs:25-27`, `table/cells.rs:47-60`
  - 34 `pub fn *_state(self)` getters across the crate
- evidence:
  - Each getter is `memo(move || slot.get().map(|s| s.get()).unwrap_or(...))`, so it allocates a new memo on every call.
  - Every item and cell mount does `item_states.update(|m| m.insert(key, state))` plus a cleanup.
  - Only one external caller exists in ui, kits and examples: `kivact-ui/src/text_input.rs:277` (`editing.control_state()`). All other uses are in `dev/tests/.../interaction_toggle.rs` and `interaction_slider.rs`.
- fix:
  - Delete `part_slot.rs`, every `*_state` slot field, every `item_states`, `row_states` and `cell_states` map, and all 34 `*_state()` getters.
  - The only way to read a part's state is `let p = api.trigger(); p.state()` (already on `Part`).
  - For `text_input.rs:277`, keep the `Part` returned by `editing.control()` and read `.state()`.
  - Rewrite the two e2e tests to use `Part::state()`.
- breaking: yes   kpi: K2, K5   size: M   confidence: high

### UI-02 Per-item memos are O(N) / O(S); the core `selector` is never used
- kind: INEFF
- where:
  - `list/parts.rs:47-90`: `selected(state.selection).contains(key)` clones the selection Vec per item. `name` does `items.iter().find`. `set_size` is identical in every item.
  - `combobox/handle.rs:706-721`: `options.iter().find` per item, and `is_active` per item.
  - `menu/level.rs:349-372,559-572`: `item_entry` finds and clones the `MenuEntry` per item, plus 9 more memos per item on top of 6 in `list::base_item`.
  - `table/cells.rs:36-40,79,96`: `selected.with(|s| s.contains(key))` per row and per cell.
  - `kivact-runtime/src/reactive/selector.rs` (unused by interaction and ui).
- evidence: A selection change in a 1,000×10 data table re-runs 10,000+ cell memos, each O(S). Mounting N options in a combobox, list or menu is O(N²), because N items each find over N.
- fix:
  - Add `crate::collection::Index<K>`, a once-per-collection memo of `HashMap<K, usize>` plus `Vec<(K, enabled, label)>`. Item memos read `index.get(&key)` in O(1).
  - Active key: `selector(active)`.
  - Selection membership: `selector_set(selected)`, a selector over a `HashSet<K>` memo; add it to core next to `selector` if it is missing.
  - `set_size` becomes one memo shared by all items.
  - Per-item budget: at most 2 memos (state + name). `disabled` and `position` read the index directly inside the `part(..)` state closure.
  - Apply the same pattern to list, combobox, menu, select, tabs, tree, table rows/cells and palette.
- breaking: no   kpi: K5, K2, K3   size: M   confidence: high

### UI-03 The monolithic UiRecipes/UiTokens default links every component's look into every app
- kind: INEFF / STRUCT
- where:
  - `kivact-ui/src/recipes.rs:135-225`: the macro `default_table()` builds every field, and `Default::default()` builds every sub-table.
  - `recipes.rs:648-742`: UiRecipes holds 34 flat recipes and 48 sub-tables.
  - `recipes.rs:744`: `fallback` returns the full default.
  - `tokens.rs:13-43`
  - `layer.rs:23`: `pick(UiRecipes::default_table())` runs on every recipe-styled node.
- evidence:
  - Any app that draws one button reaches `UiRecipes::default_table()`, which references every `look_*` function, so the linker keeps all of them (chat, calendar, code block…).
  - UiTokens also builds every component's tokens.
  - `PartialEq` compares about 300 `Arc` pointers on every theme comparison.
- fix:
  - Delete `UiRecipes` and `UiTokens`.
  - Each component gets one `ThemeExtension`, `<Component>Theme` (for example `ButtonTheme`, `SelectTheme`), generated by the existing `recipe_table!` with a new `tokens { .. }` section that holds the component's former Tokens fields.
  - The component reads `theme.ext::<ButtonTheme>()`, and `fallback` builds only that component's defaults.
  - Kits install `theme.with(ButtonTheme::default().root(..))`. This is the same number of lines as today's `UiRecipes::default().button(..)`.
  - `theme_conformance` iterates a list of component themes.
- breaking: yes   kpi: K6, K1   size: L   confidence: high (link reachability read through the macro); the K6 magnitude is not measured

### UI-04 Behavior lives in ui and views/ui-only vocabulary live in interaction
- kind: STRUCT
- where:
  - **Machines in ui:**
    - `switch_drag/machine.rs` (369 lines)
    - `toast_swipe/machine.rs`, `drawer_swipe.rs`
    - `select/machine.rs` (sheet tap)
    - `date_input/machine.rs`, `time_input/machine.rs`, `date_range_draft.rs`, `date_time_model.rs`, `date_sheet.rs`
    - `pagination_draft.rs`, `composer_history.rs`, `calendar/navigation.rs`
    - `data_table/filters.rs`, `data_table/row_drag.rs`, `number_input.rs:237`, `selection_field.rs`
  - **Views in interaction:** `announcer.rs:49` (`outlet() -> View`), `menu/handle.rs:496` and `palette/handle.rs:497` (`while_open`), `toaster/data.rs:133` (`Toast::icon: ViewFn`).
  - **ui-only vocabulary in interaction:**
    - `interaction_policy.rs:99-196`: `SwitchDrag(pub bool)`, `ToastSwipe`, `SelectPolicy.sheet_slop`, `SliderTouchMotion`
    - `surface_policy.rs` (`DrawerPolicy`, `HoverCardTiming`)
    - `combobox/handle.rs:39` (`ComboboxMode`, used only by `kivact-ui/src/combobox/*`)
- evidence: `lib.rs:7` says "A behavior draws nothing". A kit that wants a swipeable switch or toast cannot reuse the ui machines, because they are private to ui. This contradicts strategy line 79 ("drag/取消しは interaction と ui で組み").
- fix:
  - **Rule 1** (xtask-checkable): ui contains no `impl Machine` driven by input events. Input-driven machines move to interaction, each as `use_<name>` returning a Copy Api with Parts:
    - `use_switch_drag`
    - `use_swipe_dismiss`, for toast and drawer together: the same gesture with different policy
    - `use_sheet_tap`
    - `use_date_segments`, for date_input, time_input and date_time
    - `use_draft<T>`: a generic "edit a draft, commit or cancel to a Binding" machine that replaces `date_range_draft`, `pagination_draft`, `date_time_model` and the number_input typed draft
    - `use_history` (composer_history)
  - Presentation-only timers (button busy reveal, progress motion, alert collapse, `empty_state_changes`) stay in ui.
  - **Rule 2:** interaction returns no `View` except `<Api>::surface(placement, content)` (UI-24). `Announcer::outlet` becomes a ui component `announcer_outlet()`. `Toast::icon` becomes `Toast::icon_name: Option<&'static str>` (an IconSet name), and ui draws it.
  - **Rule 3:** move `ComboboxMode` to ui.
  - **Rule 4:** InteractionPolicy keeps only policies of interaction behaviors. After the moves in Rule 1 this holds, so delete nothing else. Replace `SwitchDrag(pub bool)` and `MenuPress(pub bool)` with plain `bool` fields `switch.drag` and `menu.press`.
- breaking: yes   kpi: -   size: L   confidence: high

### UI-05 Combobox: the open state has two owners, query config sits in machine state, and copies diverge
- kind: STATE / BUG
- where:
  - `combobox/machine.rs:5-38,117-137`
  - `combobox/handle.rs:163-181,376-417,421-429,463-499,789-794`
  - `palette/handle.rs:336`
  - `kivact-ui/src/combobox/render.rs:449,565-578`
- evidence:
  - **Two owners of "open".** `ComboMode::{Closed, Browsing, Filtering}` and the caller's `Binding<bool>` both hold it. They are synced by `Effect::WriteOpen` and by `feed(open, OpenRequested)` ("the machine is the source of truth" vs the caller's binding).
  - **Inline mode fakes it.** It keeps `ComboMode::Closed` while `is_open()` returns `true`. ui passes a fake `binding(|| true, |_| {})` (`render.rs:566`).
  - **Config inside state.** `Query::Local(String) | Shared(Binding<String>)` is construction-time configuration stored in state, with a hand-written `PartialEq`.
  - **The reset comment is wrong.** The comment at `machine.rs:50` says "The close transition's reset", but Close does not reset the query. `follow_open` (an effect) does it for the palette.
  - **Copies diverge.** `with_input_ref(mut self)` makes a copy whose `input()` binds another ref, while the `FocusLeft` watcher installed by `popup_with_focus_transfer` watches the original ref. Whether this works depends on call order.
- fix:
  - New signature: `use_combobox(items, selected, query: impl IntoBinding<String>)`. The query is always the caller's binding, so `Query` and `query_binding` are deleted.
  - Machine state: `enum ComboState<K> { Closed, Open { surface: Surface, filtering: bool, highlight: Option<K> } }`.
  - For a popup, `open` is derived as `binding(move || matches!(state, Open{..}), |v| send(if v {Open} else {Close}))`. The caller observes it through `on_open_change` (one handler, Radix-style), and `.popup(open)` is deleted.
  - Inline mode is a constructor flag: `ComboboxOptions { inline: bool }`.
  - Close resets the highlight and writes `""` to the query inside the transition. Delete `follow_open`, `Event::Reset` and `Event::Share`.
  - Replace `with_input_ref` with `input_for(node_ref)` (a second input part on the same handle). Each input part registers its own focus watcher at attach.
- breaking: yes   kpi: K5   size: M   confidence: high

### UI-06 Popup: two close paths, two sources for "modal", overlapping focus-return options
- kind: STATE / DUP
- where: `popup/api.rs:163-177` (effect `WriteClose`) vs `popup/api.rs:358-384` (`finish_close`), plus `:119-121`, `:171-173` and `:301-315`
- evidence:
  - `hide()` goes through `Event::Close` → `Effect::WriteClose` → `open.set(false)` → `Event::Accepted`.
  - A dismissal goes through `finish_close`, which sets `open` itself and sends `Accepted`.
  - Both compute `return_target(policy, reason, opener, anchor, settings.modal || content.is_modal())`.
  - `modal` is both a private setter flag (`PopupApi::modal`, set by dialog) and the engine's `NodeRef::is_modal()`.
  - `return_focus_target: Option<Value<NodeRef>>` silently overrides `focus_return: Option<FocusReturn>`.
  - The machine state `ReturningFocus` is entered and immediately left again in the same dispatch through `Effect::Unmounted` → `Event::Unmount`, an effect that re-sends an event to reach `Closed`.
- fix:
  - Single close path: dismissal calls `self.machine.send(Event::Close(reason))`. Delete `finish_close`.
  - Modal is only `content_ref().is_modal()`. Delete `PopupOptions.modal` and `PopupApi::modal`.
  - Replace `return_focus_target` with `FocusReturn::anchor_value(Value<NodeRef>)`, a variant of `FocusReturnTarget`.
  - The Unmount transition goes directly to `Closed` with `Effect::Return(opener)` when it restores. Delete `Effect::Unmounted` and `State::ReturningFocus`.
- breaking: no (internal)   kpi: -   size: S   confidence: high

### UI-07 Carousel: two effects sync index ↔ scroll in both directions
- kind: STATE
- where: `kivact-ui/src/carousel.rs:253-322` (`follow_index`, `follow_scroll`, including the dead `let _ = offset;`)
- evidence: One effect writes the scroll offset from `shown` when `navigation` is Idle. The other writes the index (`api.scrolling(at)`) from the scroll offset. Correctness depends on the machine's Idle gate breaking the loop.
- fix:
  - Move this into `interaction::carousel` (UI-04).
  - The machine takes `Event::ScrollObserved(nearest_index)` (fed with `feed(scroll_offset, ..)`) and `Event::GoTo(i)`, and emits `Effect::ScrollTo(i, behavior)`.
  - ui keeps one `feed` and one effect handler. No effect writes a signal.
- breaking: no   kpi: K3   size: S   confidence: high

### UI-08 Typeahead has three owners, injected through a `Cell<Binding>`
- kind: STATE / DUP
- where:
  - `list/state.rs:56,103-106`, `list/handle.rs:271,315-320` (`shared_typeahead` replaces the Cell after creation)
  - `list/machine.rs:64,100-107`, `select/machine.rs:16,…`, `menu/machine.rs:14,176,300-301`, `menu/session.rs:246-256`
- evidence: list, select and menu each keep a `Typeahead` field and each schedule `TYPEAHEAD_WINDOW` expiry. select and menu then overwrite the list's `typed` Cell. A mutable `Cell` inside an `Rc<State>` behind a Copy handle is hidden mutable state.
- fix:
  - Typeahead lives only in the list machine.
  - select and menu read and reset it through `ListApi::typeahead()` / `ListApi::clear_typeahead()` (pub(crate)).
  - Delete `shared_typeahead`, `State.typed: Cell<..>`, and the `Typeahead` fields and `Expire` events in the select and menu machines.
- breaking: no   kpi: -   size: S   confidence: high

### UI-09 Machines without state, machines with String state, and effects feeding machines
- kind: STATE / PROCESS
- where:
  - `scalar/focus_machine.rs` (`type State = ()`), `scalar/range_machine.rs:19` (`State = ()`)
  - `kivact-ui/src/calendar/view.rs:388-403`: the month-navigation machine whose state is the announcement `String`, fed by an `effect`
  - `delayed/api.rs:103`: `effect(move || state.send(Event::Wanted(wanted.get())))`
  - `kivact-ui/src/date_input/view.rs:179`, `date_range_input.rs:429`: `effect(|| send(External(value.get())))`
- evidence: The "everything is a Machine" rule produced machines that are plain conditionals. Some sites use `feed`, others use `effect` + `send`, for the same job.
- fix:
  - Delete the unit-state machines. Inline `if !any_focused { drag.cancel(); touch.finish(true) }` in the blur handler.
  - Calendar: the announcement is a `memo` of the displayed month and locale; there is no machine.
  - Replace every `effect(move || m.send(E(v.get())))` with `m.feed(v, E)`.
  - xtask: forbid `machine(()` and `type State = ()`.
- breaking: no   kpi: K2   size: S   confidence: high

### UI-10 List: configuration stored in machine state (`Focus::Physical | Retained`)
- kind: STATE
- where: `list/state.rs:27,92-97,148-150`, `list/parts.rs:150-156`, `list/handle.rs:246-251`
- evidence: The `retained_focus: Value<bool>` option is read untracked at each item mount and sent as `Event::MountFocus` into machine state. The option and the machine field both hold the fact.
- fix: Make `retained_focus: bool` a construction-time field of `ListOptions`. Delete `machine::Focus`, `Event::MountFocus` and `retain_focus()`. `retains_focus()` reads the option.
- breaking: no (pub(crate))   kpi: -   size: S   confidence: high

### UI-11 The disabled state comes from two sources (Api setter + per-host registry)
- kind: STATE / DUP
- where: `disclosure.rs:58-61,119-168,179-186`, `scalar/edit_state.rs:28-31,48-52,79-88`, `scalar/handle.rs`
- evidence:
  - `settings.disabled` and `host_disabled: Signal<Vec<Value<bool>>>` (filled from `host.accessible_disabled()` at each attach, pruned in cleanup) are OR-ed together.
  - `DisclosureApi::is_disabled` reads `host_disabled` untracked, while `base_trigger` reads it tracked: two code paths that disagree on reactivity.
- fix:
  - The one source is the node's `.disabled(..)` (the core doc already says "this one definition disables the component's own input too").
  - Delete `DisclosureApi::disabled`, `ScalarApi::disabled` (and `TooltipApi::disabled` → node `.disabled`), and the `host_disabled` registries.
  - Each part reads `host.accessible_disabled()` at attach into a part-local `Value<bool>`.
- breaking: yes   kpi: K2   size: S   confidence: medium (scalar paths not read end to end)

### UI-12 The opening/closing vocabulary is not uniform, and the lib docs contradict the code
- kind: API / CONTRA
- where:
  - `kivact-interaction/src/lib.rs:17-33`
  - `disclosure.rs:133` (only `hide`), `popup/api.rs:351` (`hide(reason)`), `dialog.rs` (no hide)
  - `inline_edit.rs:193-197` (`show(key)`, `is_open(&K) -> bool`, not a Value), `navigation_menu.rs:263-272` (`show(index)`)
  - `menu/handle.rs:126-141` (`use_menu` vs `use_menu_with_open`), `combobox/handle.rs:376` (`.popup(open)` setter)
  - `hover_card.rs`, `navigation_menu.rs:231` (`on_open_change`) vs a Binding elsewhere
- evidence:
  - The lib doc promises `show()`, `hide()` and `toggle()` on every surface. It also says "a tooltip and a dialog add their own rules to a popup", but tooltip does not use popup at all (`tooltip/api.rs` uses `DelayedApi` only).
  - The open state is supplied in three ways: a positional Binding, a `_with_open` constructor, or a setter.
  - A popover can be built twice: `use_popup` or `use_dialog(..).mode(DialogMode::Popover)`.
- fix, for every surface Api (Disclosure, Popup, Dialog, Menu, Select, Combobox popup, Palette, HoverCard, NavigationMenu):
  - Constructor `use_x(.., open: impl IntoBinding<bool>)`, with `open` always positional and required. ui creates a local signal when the user gives none.
  - Commands `show()`, `hide()` and `toggle()` with no arguments. `PopupApi::hide(reason)` becomes `pub(crate) close(reason)`.
  - `is_open() -> Value<bool>`.
  - Delete `use_menu_with_open` (`use_menu(items, open)`), `on_open_change` (observe the binding), `DialogMode::Popover` (use `use_popup`) and the private `DialogOptionsMode`.
  - `inline_edit`: `editing() -> Value<Option<K>>` plus `edit(key)` and `cancel()`.
  - Rewrite `lib.rs:17-33` to describe what remains.
- breaking: yes   kpi: -   size: M   confidence: high

### UI-13 Component constructors and handler names deviate
- kind: API
- where:
  - `tag_input.rs:59` (`tags: impl IntoValue<Vec<String>>` plus `on_add`/`on_remove`: an uncontrolled value with edit callbacks, the only such component)
  - `stepper.rs:122,153` (`current: IntoValue<usize>` plus `on_step_press(usize)`)
  - `avatar.rs:87-99` (`on_open` = `on_press` + `Role::Link`; each call attaches another `use_button()` part)
  - `calendar/view.rs:174,280` (`on_activate` + `on_commit`), `text_input.rs:227` (`on_draft_commit`)
  - `hover_card.rs:119`, `navigation_menu.rs:48` (`on_open_change`)
  - `heading(value, level: u8)`, `hover_card(content_fn)` vs `tooltip(label)`, `input_group` vs `input_group_with`
  - 14 builders with `size`, but only `Button` has `default_size`
- evidence: Handlers with the same meaning have five names (`on_press`, `on_activate`, `on_step_press`, `on_open`, `on_commit`). Controlled values are given in two ways (Binding vs Value + callbacks).
- fix, using these rules:
  - (a) Every value the user edits is a positional `impl IntoBinding<T>`: `tag_input(label, tags: IntoBinding<Vec<String>>)`, `stepper(items, current: IntoBinding<usize>)`. Delete `on_add`, `on_remove` and `on_step_press`.
  - (b) A press with no payload is `on_press(impl Fn())`. A choice of a keyed item is `on_activate(impl Fn(K))`. Delete `Avatar::on_open`; use `.on_press(..).role(Role::Link)` and attach the button part once in `avatar()` when a press handler exists.
  - (c) Commit or draft is `on_commit(impl Fn(T))` everywhere: rename `on_draft_commit` to `on_commit`.
  - (d) `input_group_with` is deleted, and `input_group(label, children)` accepts a closure through `IntoChildren`.
- breaking: yes   kpi: -   size: M   confidence: high

### UI-14 Boilerplate and five conventions for where a component's look lives
- kind: DUP / STRUCT
- where:
  - 68 identical `pub fn variant(mut self, v: impl Variant)` methods (with copied doc comments), 63 `impl Element for`, 70 `impl IntoView for`, 119 `variants: Variants` fields
  - Look code is spread over:
    - `look.rs`, `look_extra.rs` (select, menu, dialog, tooltip, tabs, field, disclosure and hover card together)
    - `look_controls.rs`, `look_collections.rs`, `look_static_collections.rs`, `look_feedback.rs`, `look_indicators.rs`, `look_navigation.rs`, `look_metrics.rs`, `look_chat.rs`, `look_files.rs`, `look_palette.rs`, `look_input.rs`, `look_radio.rs`, `look_calendar.rs`
    - `toolbar_look.rs`, `group_look.rs`, `combobox/look.rs`
  - State types are spread over `values.rs`, `component_values.rs`, `collection_values.rs`, `navigation_values.rs`, `indicator_values.rs`, `chat_values.rs`, `code_values.rs`, `table_values.rs`, `tree_values.rs`, `kbd_values.rs`, `hover_card_values.rs`, `disclosure_values.rs`, `carousel_values.rs`, `calendar_values.rs` and `group_types.rs`.
  - Tokens types are spread over 12 `*_tokens.rs` files, `group_types.rs` and `collection_values.rs`.
  - `UiTokens` has `pagination`, `breadcrumb` and `navigation_menu` all of type `NavigationTokens`, and `date_input`/`time_input` both of type `DateInputTokens`.
  - Recipe state types differ by component: `()`, `usize`, `Orientation`, `PartState`, interaction types (`PopupState`, `ListItemState`, `NavOpen`, `DropzoneState`) or ui types.
  - The `UiRecipes` docs (`recipes.rs:15-20`) list 20 of the 48 tables.
- evidence: Adding Rating (architecture.md:428-442) touches 8 runtime files across 2 crates, and the 46 side files (about 3.8k lines) exist only to hold look code apart from the component.
- fix, one pattern that a weak implementer can apply component by component:
  ```
  crates/kivact-ui/src/<name>/mod.rs   pub fn <name>(..) -> <Name>; pub struct <Name> { root: Node, .. }; setters; impl Element; impl IntoView
  crates/kivact-ui/src/<name>/look.rs  pub struct <Name>State (recipe state); recipe_table!{ <Name>Theme { part(..): <Name>State = fn, .. } tokens { .. } }; default fns
  crates/kivact-ui/src/lib.rs          mod <name>; pub use <name>::{<name>, <Name>, <Name>State, <Name>Theme};
  ```
  - Every part of a component takes the same `<Name>State`; a part-specific fact is a field or an accessor.
  - Delete every `look_*.rs`, `*_values.rs`, `*_tokens.rs`, `group_*.rs` and `toolbar_look.rs`, moving their contents into the owning component's `look.rs`. Shared helpers (`height`, `font_size`, `gap`, `tinted`) go to `look_common.rs`.
  - `.variant(..)` becomes one method on the core `Element` trait. Variants are stored on the node, and `with_recipe_style` reads them from there. This deletes all 68 copies and the `variants` fields. It is a cross-area decision with the core auditor.
- breaking: yes   kpi: K6 (smaller code)   size: L   confidence: high

### UI-15 The standard-component scope contradicts strategy
- kind: CONTRA
- where: `docs/for-kivact-developer/strategy.md:78-79` ("標準部品 中核（少数）"; calendar and timeline are kit domain) and `:99` ("二つ以上の公式 kit… → 中核") vs `kivact-ui/src/lib.rs` (about 80 constructors, including chat_message/thread/composer, typing_indicator, code_block, carousel, stepper, the calendar and date family at 4.2k lines, and avatar_group)
- evidence: The "2+ kits → core" rule moved every shared kit component into the core crate. The `kivact-ui/src/lib.rs` doc table lists 25 components, and the prelude exports `selection_field` (used by one example and one soft test) and `stepper`.
- fix: The owner decides the scope (§5 Q1). Independent of that decision, remove `selection_field` and `stepper` from the prelude and rewrite the lib.rs table from the real list.
- breaking: yes (prelude)   kpi: K6   size: S/L   confidence: high

### UI-16 Hard-coded English strings in components
- kind: BUG
- where: `kivact-ui/src/combobox/render.rs:600` (`"Clear"`), `:618` (`"Toggle options"`), `async_view.rs:60` (`"Retry"`), `carousel.rs:34` (`.name("Carousel")`)
- evidence: Every other component names itself with `message("…", "…")`.
- fix: `message("combobox.clear", "Clear")`, `message("combobox.toggle", "Toggle options")`, `message("async_view.retry", "Retry")`, `message("carousel.label", "Carousel")`, plus entries in the i18n catalogs.
- breaking: no   kpi: -   size: S   confidence: high

### UI-17 Text editing: Home/End go to the ends of the whole text, there is no goal column, and word logic sits only in the engine
- kind: BUG / STRUCT
- where: `kivact-interaction/src/text_edit/policy.rs:81-88` (Home/End), `:103-135` (vertical moves added in 2b8df5ac), `kivact-engine/src/text_input/word.rs` (added in 1f5d1cd6 for double/triple click)
- evidence:
  - In a multiline editor, Home and End jump to offset 0 and `len`, where the platform convention is line start and end.
  - Repeated Up/Down use the current caret x each time, so a short line in between loses the column.
  - Ctrl/Alt+Arrow and Ctrl+Backspace word moves are missing from the key policy, while word segmentation exists in the engine for clicks. Text-editing knowledge is split between `kivact-engine` and `kivact-interaction`.
- fix:
  - Home/End in multiline use `context.line_of` / `offset_at(line, 0 | f64::INFINITY)`. Ctrl+Home/End go to the ends of the text.
  - Add `EditState::goal_x: Option<f64>`, set by vertical moves and cleared by every other edit or move.
  - Expose word boundaries through `EditContext::word_at(offset)` (core contract, engine implementation) and add Ctrl+Left/Right/Backspace/Delete (Alt on macOS, from Capabilities) to `text_edit_policy`. Then there is one place for keys and one for segmentation.
- breaking: no   kpi: -   size: M   confidence: high (Home/End, goal column), medium (engine side)

### UI-18 Button reads the theme policy once at construction
- kind: BUG
- where: `kivact-ui/src/button.rs:122-131`
- evidence: `.focusable_when_disabled(matches!(use_theme().get().ext::<InteractionPolicy>().button.disabled_focus, Retain))` is evaluated once, so a later theme switch does not update it. `menu/level.rs:38` does the same ("read when the level is built").
- fix: `.focusable_when_disabled(move || matches!(use_theme().get().ext::<InteractionPolicy>().button.disabled_focus, Retain))`. This needs core `focusable_when_disabled(impl IntoValue<bool>)`; check the signature. Make the same change in the menu level.
- breaking: no   kpi: -   size: S   confidence: medium

### UI-19 One type exported under two paths, and two column vocabularies
- kind: CONTRA / API
- where:
  - `kivact-ui/src/carousel.rs:10` (`pub use kivact_interaction::Snap`), `kivact-ui/src/lib.rs:292` (`pub use kivact_interaction::ComboboxMode`)
  - `kivact-interaction/src/table/model.rs:60-140` (`ColumnDef::with_width`) vs `kivact-ui/src/column_values.rs:46-200` (`DataColumn::width`, wrapping `ColumnDef<R, ()>`)
  - `kivact-ui/src/data_column.rs` holds only `header_context_menu`, while `DataColumn` is defined in `column_values.rs`
- evidence: architecture.md:163 forbids exporting the same type from another module.
- fix:
  - `Snap` lives only in interaction and ui stops re-exporting it. `ComboboxMode` moves to ui (UI-04).
  - Rename `ColumnDef::with_width` to `width` so both column types use the same setter names.
  - Merge `data_column.rs` into `data_table/column.rs` together with `column_values.rs`.
- breaking: yes   kpi: -   size: S   confidence: high

### UI-20 Public items without users
- kind: DEAD
- where (zero uses outside the defining crate except as noted):
  - `use_hover_corridor`, `HoverCorridorApi` (menu-internal only)
  - `use_hover_surface`, `HoverSurfaceApi` (one soft consumer test)
  - `GridMove`, `DelayedPhase` with `DelayedApi::phase`, `HoverCardOpenedBy`
  - `DragVelocity`, `Announcer`, `filter_commands`, `flatten_tree_rows` (ui-internal only)
  - `kbd_combo`, `provide_control_size` (one e2e test each)
  - `selection_field` (one example and one test, yet in the prelude)
- fix:
  - Make `use_hover_corridor`, `use_hover_surface`, their Apis, `GridMove`, `DelayedPhase`, `DelayedApi::phase`, `HoverCardOpenedBy`, `DragVelocity`, `filter_commands` and `flatten_tree_rows` `pub(crate)`.
  - Delete `selection_field`: it is a select with a custom trigger. Fold its one option into `select(..).trigger(view)`.
  - Keep `kbd_combo` and `provide_control_size` only if the owner wants them (§5 Q1).
- breaking: yes   kpi: K6   size: S   confidence: high (usage counts by grep)

### UI-21 Sparkline (new) recomputes and clones per change and per paint, and bypasses the recipe-paint rule
- kind: INEFF / CONTRA
- where: `kivact-ui/src/sparkline.rs:50-51,141-158,172-235`, `recipes.rs:545-551`, `sparkline_tokens.rs`
- evidence:
  - `values.get()` clones the Vec three times per change (`hidden`, `draw`, `description`), and `Summary::of` runs three times.
  - `paint` clones `SparklineTokens` and allocates `Vec<Vec<Point>>` on every repaint.
  - The recipe state is `()` and only sizes the box. The line is drawn by the component from tokens, which contradicts `recipes.rs:32` ("Local drawing is the recipe's `Style::paint`"), and variants never reach the drawing (a `Tone` variant cannot color the line).
- fix:
  - One `memo` per instance: `geometry = memo(|| values.with(|v| (Summary::of(v), v.clone())))`; `hidden` and `description` read it.
  - Draw from `SparklineState { summary, values: Rc<[f64]> }` through the recipe: the default recipe returns `style().size(..).paint(draw_line(tokens, state))`, and the component adds no `.draw`.
  - Derive `Copy` on `SparklineTokens` and `SparklineFill`.
  - Draw runs directly into the path; do not allocate `runs`.
- breaking: yes (recipe state)   kpi: K4, K5   size: S   confidence: high

### UI-22 Eager per-instance cost: estimates read from the constructors
- kind: INEFF
- where:
  - `tooltip/api.rs:150-222`, `delayed/api.rs:83-110`, `combobox/handle.rs:155-233`, `drag/handle.rs:145-221`
  - `list/state.rs:67-145`, `menu/level.rs:336-414`, `table/cells.rs`, `button.rs:248-291`
- evidence (signals / memos / effects / machines; memos inside `part()`, `hovered()` and `focus_within()` are not counted):

  | instance | signals | memos | effects | machines |
  |---|---|---|---|---|
  | DisclosureApi | 4 | 0 (+1 per `*_state()` call) | 0 | 0 |
  | PopupApi | 7 | 0 | 1 per opening | 1 |
  | TooltipApi (every trigger, even if never hovered) | 3 | 4 + 3 observers | 1 | 1 |
  | ComboboxApi | 9 (4 OptionsCells) | 5 | 0 | 1 |
  | per combobox option | 0 | 4 | 1 | 0 (+2 map writes) |
  | ListApi | 5 | 6 | 1 | 1 |
  | per list item | 1 + var | 6 | 0 | 0 (+1 map write) |
  | per menu item | – | ~16 (list 6 + level 10) | 0 | 0 |
  | per data-table cell | 0 | 1–2 (O(S) each) | 0 | 0 (+1 map write) |
  | DragApi | 4 (3 option cells) | 1 | 0 | 1 |

- fix:
  - UI-01 and UI-02 remove most of this.
  - Merge all setup options of one Api into a single `OptionsCell<XOptions>`: DragApi `settings`, `modifiers` and `disabled` become one; ComboboxApi `source`, `debounce`, `search_status` and `mode` become one.
  - Tooltip: drop `phase` (UI-20) and create `engaged`, the delayed machine and `visible` lazily, on the first `trigger()` attach, inside `on_attach`.
- breaking: no   kpi: K2, K1   size: M   confidence: medium (counts from reading, not measured)

### UI-23 Process artefacts: `xtask-allow(state-fields)` on passive data, and doctest shims
- kind: PROCESS
- where:
  - 291 `xtask-allow` in these two crates. Most are `state-fields` comments on passive recipe-state structs (`values.rs:117-134,170-178,316-324,422-453,500-505`), on authoring options (`toaster/data.rs:115-123`, `menu/state.rs:190-204`, `table/model.rs:64-74`, `dialog.rs:59-63`), and on slot fields that UI-01 deletes.
  - 26 doctests carry a 10-line `# mod kivact { … }` facade shim.
- evidence: The comments repeat the field doc ("option: the entry's authored icon") and protect nothing. Recipe state and builder options are by definition passive.
- fix:
  - The `state-fields` check skips structs whose name ends in `State`, `Options`, `Tokens`, `Policy` or `Theme`, plus public authoring value types (`Toast`, `MenuEntry`, `ColumnDef`, `ListItem`).
  - Delete those comments.
  - Add `kivact` as a dev-dependency of kivact-interaction (ui already has it) and write doctests with `use kivact::…`, deleting the shims.
- breaking: no   kpi: -   size: M   confidence: high

### UI-24 Building a custom component from the parts: friction compared with Radix/Ark/Kobalte
- kind: API
- where: `popup/api.rs:84-99` (doc example), `kivact-ui-pop/src/notifications.rs:89-113`, `menu/handle.rs:496`, `palette/handle.rs:497`, `recipes.rs:173-180`
- evidence:
  - The shape `let p = use_popup(open); node().part(p.trigger())` maps well to Ark's `{...api.getTriggerProps()}`.
  - The friction points:
    - (a) The caller must create its own `anchor = node_ref()`, attach it to the trigger, and pass it to `overlay(anchor, ..)`, although the popup already owns `own_anchor` (pub(crate)).
    - (b) Every floating surface is `show(api.is_open(), move || overlay(anchor, placement, view).with_node(|n| n.part(api.content())))`, except menu and palette, which have their own `while_open`.
    - (c) The kit code needs `with_node(|root| root.with_node(|root| root.part(..)))`.
    - (d) A recipe override is a closure that returns a closure: `.button(|prev| move |theme, look| prev(theme, look).radius(0))`.
    - (e) `trigger_state()` must be called during setup, because it creates a memo, while `is_open()` may be called anywhere.
- fix:
  - Every surface Api gets `pub fn surface<V: IntoView>(self, placement: Placement, content: impl Fn() -> V + 'static) -> View`. It does the `show` / `overlay(anchor)` / `part(content())` wiring, with the anchor taken from the trigger part.
  - Delete `while_open` on menu and palette; their bodies become `surface`.
  - Change the recipe override signature to `F: Fn(&Theme, &Look<S>, Style) -> Style`, where the third argument is the previous recipe's result: `.button(|theme, look, base| base.radius(0))`. `recipe_table!` evaluates the previous recipe and passes it in.
  - UI-01 removes (e).
- breaking: yes   kpi: -   size: M   confidence: high

### UI-25 Smaller duplications
- kind: DUP
- where:
  - `delayed/api.rs:84-110` vs `hover_corridor/api.rs` vs `hover_card.rs:46`: the delayed-hover chains are fine, but `HoverCardTiming` (`surface_policy.rs:6`) duplicates `Delays` (`delayed/api.rs:20`) field for field.
  - `toaster/api.rs:153-162`: the `visible` memo clones the whole column state (`column.get()`), toasts and closures included, on every change.
  - `kivact-ui/src/combobox/render.rs:566`: a fake binding for inline mode.
- fix:
  - Make `HoverCardTiming` a type alias of, or a field of type, `Delays`.
  - `visible = memo(|| column.state().with(|s| s.queue.iter().take(s.limit).map(..).collect()))`.
  - Inline mode follows UI-05.
- breaking: minor   kpi: K5   size: S   confidence: high

## 3. Target structure for this area

```
kivact-interaction   (headless contract; kits and ui use only its public API)
  collection/        Index<K> (key→index, enabled, label), selector-based membership; used by list/combobox/menu/select/tabs/tree/table/palette
  list/ roving/      list + typeahead (only owner)
  open/              disclosure, popup (one close path), dialog (Modal/AlertModal/Drawer only), tooltip, delayed, hover_card, navigation_menu, menu(+menubar)
                     every surface Api: use_x(.., open), show/hide/toggle, is_open, trigger(), content(), surface(placement, f)
  gesture/           drag, drag_settle, swipe_dismiss (from ui toast_swipe+drawer_swipe), switch_drag, sheet_tap, zoom_pan, space_pan, splitter
  edit/              text_edit (+goal column, line Home/End, word keys), scalar (no unit machines), draft<T> (from ui drafts), date_segments, history
  combobox/ select/ palette/ table/ calendar/ toaster/ tabs/ toggle/ tree/ carousel (+scroll sync) / dropzone / copy_feedback / follow_end / inline_edit
  policy             InteractionPolicy (only policies of the behaviors above)
kivact-ui
  <component>/mod.rs + <component>/look.rs   (one per component; <Name>State + <Name>Theme = recipes + tokens)
  look_common.rs     shared sizing helpers
  layer.rs           with_recipe_style (reads variants from the node)
```

Why the split stays: inputs and outputs are Copy Api handles, `Part<S>`, `Value`/`Binding` and policy data. Neither side reads the other's internals, and the crate has one reason to change: the behavior and semantics contract that kits and external components share (P-07). After this change it can be checked mechanically:
- interaction contains no `View` except in `surface()`.
- ui contains no input-driven `impl Machine` and no `effect` that writes a signal.

**Deleted:**
- `part_slot.rs` and all `*_state()` getters and item/row/cell state maps
- `UiRecipes`, `UiTokens`, the 46 side files
- `Query`/`query_binding`/`follow_open`/`Event::Reset`/`Share` (combobox)
- `finish_close`, `PopupOptions.modal`, `return_focus_target`, `State::ReturningFocus`
- `shared_typeahead`, the select/menu typeahead fields, `scalar/focus_machine.rs`, `scalar/range_machine.rs`
- `use_menu_with_open`, `DialogMode::Popover`, `while_open`, `Avatar::on_open`, `input_group_with`, `selection_field`, and the per-builder `variant` copies

## 4. Public API

**Custom popover (React/Solid developer), before:**
```rust
let open = signal(false);
let popup = use_popup(open);
let trigger_part = popup.trigger();
let anchor = node_ref();
let trigger = trigger.with_node(|root| root.with_node(|root| root.part(trigger_part)).node_ref(anchor));
let body = show(popup.is_open(), move || {
    overlay(anchor, Placement::Below, content()).with_node(|n| n.part(popup.content())).name("Options")
});
```
**After:**
```rust
let open = signal(false);
let popup = use_popup(open);
let trigger = button("Options").part(popup.trigger());
let body = popup.surface(Placement::Below, move || content().name("Options"));
```

**Recipe override, before:** `.button(|prev| move |theme, look| prev(theme, look).radius(0))` on `UiRecipes`
**After:** `theme.with(ButtonTheme::default().root(|_theme, _look, base| base.radius(0)))`

**Controlled values and handlers, before:**
`tag_input("Labels", tags).on_add(move |t| tags.update(|v| v.push(t))).on_remove(move |i| tags.update(|v| { v.remove(i); }))`
**After:** `tag_input("Labels", tags)` (Binding; `on_edit` remains for edit phases)

**Opening, before:** `use_menu(items)` / `use_menu_with_open(items, open)`, `popup.hide(PopupCloseReason::Programmatic)`, `disclosure.hide()` (no show), `combobox.popup(open)`
**After:** `use_menu(items, open)`, `popup.hide()`, `disclosure.show()/hide()/toggle()`, `use_combobox(items, selected, query)` + `ComboboxOptions { inline }`

**Delete:**
- `*_state()` (34 getters: `trigger_state`, `content_state`, `item_state`, `control_state`, `list_state`, `input_state`, `grid_state`, `cell_state`, `row_state`, `header_state`, `thumb_state`, `viewport_state`, `source_state`, `bar_state`)
- `use_menu_with_open`, `ComboboxApi::{popup, popup_with_focus_transfer, query_binding, with_input_ref}`
- `MenuApi::while_open`, `CommandPaletteApi::while_open`, `DialogMode::Popover`
- `DisclosureApi::disabled`, `ScalarApi::disabled`, `TooltipApi::disabled`
- `UiRecipes`, `UiTokens` and all `*Recipes`/`*Tokens` types (replaced by `<Name>Theme`), `ui::Snap` re-export, `ui::ComboboxMode` re-export (it moves to ui only)
- `Avatar::on_open`, `TagInput::{on_add, on_remove}`, `Stepper::on_step_press`, `HoverCard::on_open_change`, `NavigationMenu::on_open_change`, `input_group_with`, `selection_field`
- `SwitchDrag` and `MenuPress` newtypes (plain `bool`)

**Make pub(crate):** `use_hover_corridor`, `use_hover_surface`, `HoverCorridorApi`, `HoverSurfaceApi`, `GridMove`, `DelayedPhase`, `DelayedApi::phase`, `HoverCardOpenedBy`, `DragVelocity`, `filter_commands`, `flatten_tree_rows`

**Rename:**
- `on_draft_commit` → `on_commit`
- `ColumnDef::with_width` → `width`
- `inline_edit` `show(key)`/`is_open(&K)` → `edit(key)`/`editing()`
- `PopupApi::hide(reason)` → `pub(crate) close(reason)`

**Move:**
- `.variant()` → core `Element::variant` (cross-area)
- `ComboboxMode` → ui
- `Toast::icon: ViewFn` → `Toast::icon_name`
- `Announcer::outlet` → `ui::announcer_outlet()`

## 5. Open questions for the owner

1. **Scope of kivact-ui.** strategy.md says "標準部品 中核（少数）" and puts calendar and timeline in the kit domain, but the "2+ kits → core" rule has grown ui to about 80 components (chat, code_block, typing_indicator and the date family at 4.2k lines, carousel, stepper…). Choose one:
   - (a) Accept it and rewrite strategy.md.
   - (b) Split a `kivact-ui-domain` crate for the domain components (chat, code_block, typing_indicator, calendar/date/time, carousel, stepper, avatar_group). Its boundary: it depends only on the public kivact API, has one reason to change (domain features), and apps that do not use it do not compile it (K6).

   Recommendation: (b), with UI-03 done first. If the owner chooses (a), kbd_combo and provide_control_size stay.
2. **Should component Tokens fold into the per-component `<Name>Theme`, as UI-03 proposes, or stay separate?** Folding them removes UiTokens and the KIT-3 distinction; KIT-3 would be rewritten as "tokens are the plain-data fields of `<Name>Theme`".
3. **`.variant()` on core `Element`, with variants stored on the node.** This deletes 68 copies but adds an `Rc<[Extension]>` slot to each node that uses it, which bears on MEM-6. The alternative is a ui trait `Themed` with a default method, exported in the prelude.
4. **Open state for surfaces.** Should it be a required positional `open` binding everywhere (UI-12, the proposal), or an optional internal signal with an `.open(binding)` setter? The proposal keeps Copy handles free of post-construction option mutation.
