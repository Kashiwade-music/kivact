# 付属書 C：kivact-core・kivact-macros・facade・kivact-offscreen・App/platform/embed/view の公開 API

版 1.0（2026-11-18）。基点 `2b8df5ac`。本書の行番号は書かない。場所は関数名・型名・特徴のある 1 行で示す。

## 1. 範囲と前提

### 1.1 この付属書が扱う所見

| 区分 | 所見 |
|---|---|
| runtime-core.md | RC-09、RC-10、RC-11、RC-12、RC-13、RC-18、RC-19、RC-20、RC-21、RC-22、RC-27、RC-28、RC-29、RC-31（core の半分）、RC-36、RC-40（core の `Held` 以外。下の注）、RC-41（core の小項目） |
| api.md | API-01、API-04（R が実装。4.1 節 R-REQ-12）、API-05、API-07、API-08、API-09、API-10（`Owner`・`EmbedVisibility`）、API-11（launch・`AppOptions`・prelude）、API-13（4 章の割当て）、API-14、API-16、API-18、API-19（facade の path） |
| 利用者 | USR-05（`commit_focused_edit`）、USR-12（font を theme に）、USR-15（embed）、USR-16（`target_size`）、USR-17（`spacer`） |
| 設計書 | 5.1（`#[component]`）、5.4、5.6、5.8（`Offscreen` だけ）、5.11（prelude）、5.12（本付属書の名前）、6.2（PERF-9・PERF-10 の core 側）、7.2、7.9 |

注：RC-40 のうち `xtask-allow(state-fields)` の印の削除は付属書 T（W1）が行う。`OwnerEntry`・`Pending.turn_requested`・`ext: OnceCell` は runtime なので付属書 R。`virtual_list` の `Held` は RC-23（付属書 R）と同じファイルなので R の RC-23 の unit が行う（4.4 節）。

### 1.2 前提（他の付属書が先に終えていること）

- W1（付属書 T）：xtask の `state` 規則（設計書 6.1）、e2e の 4 分割（`e2e_core`・`e2e_ui`・`e2e_kits`・`e2e_native`）、`kivact-devtools-client` の protocol への統合、`visible_to` の既定（dev-dependency は対象外）、例・雛形を `//! Role:` 規則の対象外にすること。
- W1a（付属書 S）：`kivact_types::cell::{Deferred, SharedDeferred}`。
- 付属書 E：4.2 節の engine の要求。E-REQ-5 は E08（W4）、E-REQ-6 と `TreeChanges` の window は E30（W4）、E-REQ-1〜4 は E31（W4b）。
- 付属書 S の S09（W2a）：4.3 節の `Theme::with_fonts`・`Theme::fonts`（追加だけで非破壊）。S17（W6b の鎖）：`StyleProperty` の表（C19）。
- W5a の鎖（付属書 R が C より先。順は R05、R06、R08、R09 → R20〜R25 → R26〜R31 → C07〜C18。4.1 節）：runtime の名前（`IntoHandler`、`Handler`、`Callback::from_handler`、`Trigger`、`custom::Prop`、`Owner::setup`・`running`、`end_idle`）、`use_context -> T` と `provide` の `T`、`launch/turn.rs`、DEC-16 の形の macro の handler の props。

### 1.3 この付属書が後の付属書に渡すもの

- `#[component]` が `-> impl Element` を作れる（W5a の C09）。付属書 U（W7a〜W8）と K（W9〜W10）はこれで builder を移す。
- `Element::variant`、`Variant`、`Variants`、`Node::variant_style`・`Node::base_variant_style`（W5a の C10）。付属書 U が `Look<S>` をこの上に作る。
- `PointerEvent::target_size()`（W5a の C15）。付属書 U が `DragUpdate::target_size()` を作る。
- `kivact::draw::Offscreen`（新 crate `kivact-offscreen`、W2b の C02、署名は W5a の C17）。付属書 K の `kivact-plot` が使う。
- prelude（W5a の C18）。付属書 S・U は名前を作ったときに prelude の行を足す（4.3・4.5 節）。

## 2. unit の一覧

| ID | wave | lane | 前提の unit | 所有するファイル・directory（要約。正確な一覧は各 unit の「所有」） | 所見 | 規模 |
|---|---|---|---|---|---|---|
| C01 | W2b | C | T（W1）、S04（W2a） | `kivact-core/src/view/devtools_*`、`view/inspect_mode.rs`、`devtools/`、`view.rs`、`devtools.rs`、`view/tree.rs`、`view/state.rs`、`view/boundary.rs`、`view/root.rs`、`view/mount/style.rs`、`launch/runtime.rs`、`launch/inspect.rs` | RC-27、RC-10（hook）、RC-41（`is_running`） | M |
| C02 | W2b | C | T（W1）、S02・S07（W2a） | 新 `crates/kivact-offscreen/`、`Cargo.toml`（workspace）、`kivact-core/Cargo.toml`、`kivact-core/src/draw.rs`、`kivact-core/src/draw/offscreen.rs`（削除）、`crates/kivact/Cargo.toml`、`crates/kivact/src/{lib.rs,draw.rs}`、`dev/xtask/policy/workspace.toml` | RC-29（crate） | S |
| C03 | W3a | C | C01 | `view/state.rs`、`view/requests.rs`、`view/snapshots.rs`、`view/foreign_window.rs`、`client/dispatch/{platform.rs,delivery.rs}`、`view/mount/resolve.rs`、`view/node_ref.rs`、`view/elements/builders.rs`、`platform/app_handle.rs`、`platform/services.rs`（`EffectWaiter` の置換だけ）、`gpu_interop/`、`launch/runtime.rs` | RC-21、RC-36、RC-41（小項目） | M |
| C04 | W3a | C | C01 | `view/mount/style.rs`、`view/mount/create.rs`、`view/boundary.rs`、`view/root.rs`、`view/tree.rs`、`view/node_modifiers.rs`、`diagnostics/why.rs`、`devtools/node.rs` | RC-22、RC-41（`bind_states`・`Leaving.on_exit`） | M |
| C05 | W4 | C | C03、C04 | `view/state.rs`、`view/window_view.rs`、`view/foreign.rs`、`view/foreign_window.rs`、`view/root.rs`、`view/hover.rs`、`view/tree.rs`、`launch/runtime.rs`、`launch/close.rs`、`client/dispatch/env.rs`、`platform/env.rs`、`custom/node.rs` | RC-19、RC-41（`EnvFields`・dead field） | L |
| C06 | W4b | C | C01、C05、E08・E30（W4）、R04（`reported_since`） | `kivact-core/src/devtools/`、`view/devtools_hooks.rs`、`view/mount/style.rs`、`launch/inspect.rs`、`kivact-devtools-agent/src/{source.rs,agent.rs}`、`kivact-test/src/screen/state.rs` | RC-09、RC-10（残り）、RC-28 | L |
| C07 | W5a 鎖 | 鎖 | R31、C05、E31（W4b） | `view/tree.rs`、`view/text_edits.rs`（削除）、`view/state.rs`、`client/state.rs`、`client/dispatch/{env.rs,platform.rs,text.rs}`、`launch/{runtime.rs,turn.rs}`、`custom/node.rs`、`platform/env.rs`、`view/window_view.rs` | RC-20 | M |
| C08 | W5a 鎖 | 鎖 | C07 | `kivact-core/src/{state.rs,view/boundary.rs,view/owner.rs,view/root.rs（1 行）,custom.rs}` と `use_direction`・`state::Owner` の全呼出し元 | RC-18、RC-31（core の残り）、API-10（`Owner`） | M（掃引） |
| C09 | W5a 鎖 | 鎖 | C08 | `crates/kivact-macros/`、`Cargo.toml`（workspace の依存）、`kivact-core/src/view/{boundary.rs,elements/builders.rs,view.rs}`、`kivact-core/src/custom.rs`、`crates/kivact/src/lib.rs`、`kivact-ui`・`kivact-interaction` の `Cargo.toml`、policy の 1 行、`#[component]`・`scope`・`constant_value!` の利用者（R26・R27 の macro の上に書く） | API-01、API-18、API-06（derive）、API-07（core の doc）、5.1 規則 9（型引数） | L |
| C10 | W5a 鎖 | 鎖 | C09 | `kivact-core/src/view/{variant.rs（新）,element_data.rs,style_layer.rs,mount/parts.rs,mount/style.rs,mount/create.rs,tree.rs,view.rs}`、`kivact-ui/src/variants.rs`、`kivact-interaction`（`Tone` の impl 1 行）、ui・kit の `impl Variant for` | 5.4（`Element::variant`） | M |
| C11 | W5a 鎖 | 鎖 | C10 | `kivact-core/src/{app.rs,launch.rs,launch/*,lib.rs,platform.rs,embed/open.rs,embed/embedded.rs,platform/app_handle.rs}`、`crates/kivact/src/{lib.rs,launch.rs}`、`crates/kivact-test/src/`、`kivact-engine/src/embed.rs`、呼出し元 | RC-12、RC-13、API-11（launch） | L |
| C12 | W5a 鎖 | 鎖 | C11 | `kivact-core/src/{platform/close_request.rs,launch/close.rs,client/dispatch/env.rs,view/state.rs,launch/runtime.rs}`、`dev/tests/kivact-public-api/tests/*/{close_request.rs,windows.rs}` | RC-11 | M |
| C13 | W5a 鎖 | 鎖 | C12 | `kivact-core/src/embed*`、`kivact-types/src/embed.rs`、`kivact-engine/src/embed.rs`、`arvane/src/embed.rs`、`arvane-shell/src/child.rs`、`arvane-native/src/{port/embed/*,port/win32/*,port/x11/*,wait.rs}`、`dev/tests/kivact-public-api/tests/*/embed/` | USR-15、API-10（`EmbedVisibility`）、RC-12（retention） | M |
| C14 | W5a 鎖 | 鎖 | C13、S09（W2a。`Theme::with_fonts`） | `kivact-core/src/{app.rs,launch/settings.rs,launch/runtime.rs,view/root.rs,view/boundary.rs,view/state.rs,view/requests.rs,platform/env.rs}`、`kivact-test/src/`（options、render、driver、gpu_tests）、`App::font`・`App::fonts`・`RenderOptions::font`・`TestFonts::Only` の呼出し元 | USR-12 | M |
| C15 | W5a 鎖 | 鎖 | C14 | `kivact-core/src/view/{focus_commit.rs（新）,input.rs,view.rs,state.rs,requests.rs,elements/container.rs,mount/part_pointer.rs}`、`client/dispatch/{pointer.rs,text.rs}`、`launch/turn.rs`、`dev/tests/kivact-public-api/tests/*/{text_editing.rs,input_platform.rs,layout.rs}` | USR-05、USR-16（core 側）、USR-17（spacer） | M |
| C16 | W5a 鎖 | 鎖 | C15 | `kivact-core/src/view/{flow/error_boundary.rs,window_view.rs,element_data.rs,descriptor/factory.rs}`、`kivact-core/src/lib.rs`、`kivact-ui/src/{async_view.rs,text_input.rs}` と呼出し元 | API-16、API-08（署名）、API-14、API-07 | S |
| C17 | W5a 鎖 | 鎖 | C16 | `crates/kivact-offscreen/src/lib.rs`、`Offscreen::render` の全呼出し元 | RC-29（署名） | S |
| C18 | W5a 鎖 | 鎖 | C17（C の鎖の最後） | `crates/kivact/src/{prelude.rs,lib.rs}`、prelude の全利用者、`examples/kivact/*`、`templates/basic` | API-05、API-09、API-19（facade） | L（掃引） |
| C19 | W6b 鎖 | 鎖 | S17（宣言の表） | `kivact-core/src/devtools/{apply.rs,styles.rs,node.rs}` | RC-27（parser） | S |

wave と lane は作業指示書 4 章（SCHEDULE）の配置に合わせた：C01・C02 は W2b（W2a の S02・S04・S07・R04 と file が重なるため。INC-23）、C03・C04 は W3a、C05 は W4、C06 は W4b（E08・E30 の後。INC-26）、C07〜C18 は W5a の鎖、C19 は W6b の鎖（S17 の直後）。同じ sub-wave の C の 2 つの unit は同じファイルを持たない（5 章）。

---

## 3. unit の手順

## C01：DevTools の hook を一つの trait object にし、DevTools の状態を `devtools/` へ移す

- **wave / lane / 前提**：W2b / C / T（W1 の印の削除と e2e の分割）、S04（W2a。`launch/runtime.rs` に `HostEvent::StatesChanged` の腕を足してある。C01 はその腕を残す）
- **所有**：
  - `crates/kivact-core/src/view/devtools_hooks.rs`
  - `crates/kivact-core/src/view/devtools_state.rs` → `crates/kivact-core/src/devtools/state.rs`（移動）
  - `crates/kivact-core/src/view/devtools_apply.rs` → `crates/kivact-core/src/devtools/apply.rs`（移動）
  - `crates/kivact-core/src/view/inspect_mode.rs` → `crates/kivact-core/src/devtools/inspect_mode.rs`（移動）
  - 新 `crates/kivact-core/src/devtools/hooks.rs`
  - `crates/kivact-core/src/view.rs`、`crates/kivact-core/src/devtools.rs`、`crates/kivact-core/src/devtools/*.rs`（path の書換え）
  - `crates/kivact-core/src/view/{tree.rs,state.rs,boundary.rs,root.rs,mount/style.rs}`
  - `crates/kivact-core/src/launch/{runtime.rs,inspect.rs}`
- **目的**：view が DevTools の型と cfg を知らないようにする（RC-27）。hook は `Option<Rc<dyn DevtoolsHooks>>` を一度だけ読み、DevTools が無い App では `if let` 一つで終わる（RC-10 の hook の部分）。`provide_locale` にも locale の emulation を掛ける（RC-27）。`is_running` の stub を消す（RC-41）。
- **手順**：
  1. ファイルを移す（Git Bash）：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-core/src" || exit 1
     git mv view/devtools_state.rs devtools/state.rs
     git mv view/devtools_apply.rs devtools/apply.rs
     git mv view/inspect_mode.rs devtools/inspect_mode.rs
     rg -l "view::devtools_state|view::devtools_apply|view::inspect_mode" . | xargs perl -pi -e 's/crate::view::devtools_state/crate::devtools::state/g; s/crate::view::devtools_apply/crate::devtools::apply/g; s/crate::view::inspect_mode/crate::devtools::inspect_mode/g'
     ```
     `devtools/state.rs` の `use super::inspect_mode;` はそのまま正しい（同じ directory になった）。
  2. `view.rs`：次の 6 行（cfg 3 組）を消す。
     ```rust
     #[cfg(feature = "devtools")]
     pub(crate) mod devtools_apply;
     #[cfg(feature = "devtools")]
     pub(crate) mod devtools_state;
     #[cfg(feature = "devtools")]
     pub(crate) mod inspect_mode;
     ```
     `pub(crate) mod devtools_hooks;` は残す（cfg なし）。
  3. `devtools.rs`：`mod` の並びに次を足す（`pub(crate) mod start;` の前後どちらでもよいが、アルファベット順に置く）。
     ```rust
     pub(crate) mod apply;
     mod hooks;
     pub(crate) mod inspect_mode;
     pub(crate) mod state;
     ```
  4. `view/devtools_hooks.rs` を全部、次の内容に置き換える。cfg は一つも無い。
     ```rust
     //! Role: The one seam through which DevTools observes and shapes the view.
     //!
     //! The view knows only the trait [`DevtoolsHooks`]; the DevTools target
     //! (`crate::devtools`, feature `devtools`) implements it. An App without
     //! DevTools holds `None`, so every hook below costs one `if let`.

     use std::panic::Location;
     use std::rc::Rc;

     use kivact_engine::api::Prop;
     use kivact_engine::contract::HostEvent;
     use kivact_runtime::NodeId as EffectId;
     use kivact_runtime::{Handled, Owner, Runtime, Signal, Value};
     use kivact_style::internal::ForcedStates;
     use kivact_style::{Style, Theme};
     use kivact_types::{Capabilities, FocusTarget, NodeId, WindowId};

     use crate::i18n::Locale;
     use crate::view::state::ViewState;

     /// What DevTools observes and shapes in the view. Implemented by the
     /// DevTools inspector only.
     pub(crate) trait DevtoolsHooks {
         /// A prop of `id` was written (`changed`: the engine's value changed).
         fn prop_written(&self, id: NodeId, prop: Prop, writer: Option<EffectId>, changed: bool);
         /// An engine event the DevTools turn handles (keys, pick, injection).
         fn host_event(&self, event: &HostEvent);
         /// A pointer, wheel, key, IME or file event reached `window`.
         fn input_event(&self, rt: &Runtime, event: &HostEvent, window: WindowId);
         /// A handler of `node` ran for `event` (`None`: it failed).
         fn handler_ran(&self, node: NodeId, event: &str, outcome: Option<Handled>);
         /// The focus of `window` moved.
         fn focus_changed(&self, rt: &Runtime, window: WindowId, target: Option<FocusTarget>, visible: bool);
         /// The core sent a programmatic focus request for `target`.
         fn focus_requested(&self, target: FocusTarget);
         /// The host answered a focus request.
         fn focus_result(&self);
         /// `theme` with the DevTools token overrides applied.
         fn wrap_theme(&self, owner: Owner, theme: Value<Theme>) -> Value<Theme>;
         /// `locale` with the emulated locale applied.
         fn wrap_locale(&self, owner: Owner, locale: Value<Locale>) -> Value<Locale>;
         /// The host capabilities with the emulated touch support applied.
         fn wrap_capabilities(&self, owner: Owner, host: Signal<Capabilities>) -> Value<Capabilities>;
         /// `text` in the pseudo-locale, when one is on.
         fn pseudo(&self, text: String) -> String;
         /// The overrides and forced states of node `id`.
         fn style_hook(&self, rt: &Rc<Runtime>, owner: Owner, id: NodeId, at: &'static Location<'static>) -> StyleHook;
     }

     /// The DevTools hooks of `rt`'s App; `None` without DevTools.
     #[inline]
     pub(crate) fn hooks(rt: &Runtime) -> Option<&dyn DevtoolsHooks> {
         ViewState::of(rt).devtools.as_deref()
     }

     /// Queue an engine event for the DevTools turn.
     pub(crate) fn host_event(rt: &Runtime, event: &HostEvent) {
         if let Some(h) = hooks(rt) {
             h.host_event(event);
         }
     }

     /// Add an input event to the DevTools input log.
     pub(crate) fn input_event(rt: &Runtime, event: &HostEvent, window: WindowId) {
         if let Some(h) = hooks(rt) {
             h.input_event(rt, event, window);
         }
     }

     /// Add one handler run to the last input log entry.
     pub(crate) fn handler_ran(rt: &Runtime, node: NodeId, event: &str, outcome: Option<Handled>) {
         if let Some(h) = hooks(rt) {
             h.handler_ran(node, event, outcome);
         }
     }

     /// Add the focus move of `window` to the recorded history.
     pub(crate) fn focus_changed(rt: &Runtime, window: WindowId, target: Option<FocusTarget>, visible: bool) {
         if let Some(h) = hooks(rt) {
             h.focus_changed(rt, window, target, visible);
         }
     }

     /// Remember the target of the programmatic focus request just sent.
     pub(crate) fn focus_requested(rt: &Runtime, target: FocusTarget) {
         if let Some(h) = hooks(rt) {
             h.focus_requested(target);
         }
     }

     /// Drop a stale programmatic focus request.
     pub(crate) fn focus_result(rt: &Runtime) {
         if let Some(h) = hooks(rt) {
             h.focus_result();
         }
     }

     /// `theme` as DevTools shapes it.
     pub(crate) fn wrap_theme(rt: &Runtime, owner: Owner, theme: Value<Theme>) -> Value<Theme> {
         match hooks(rt) {
             Some(h) => h.wrap_theme(owner, theme),
             None => theme,
         }
     }

     /// `locale` as DevTools shapes it.
     pub(crate) fn wrap_locale(rt: &Runtime, owner: Owner, locale: Value<Locale>) -> Value<Locale> {
         match hooks(rt) {
             Some(h) => h.wrap_locale(owner, locale),
             None => locale,
         }
     }

     /// Apply the pseudo-locale to a user-visible text.
     pub(crate) fn pseudo(text: String) -> String {
         let Some(rt) = Runtime::current() else {
             return text;
         };
         match hooks(&rt) {
             Some(h) => h.pseudo(text),
             None => text,
         }
     }

     /// The style overrides and forced states of one node, read by its style
     /// effect. `Off` for every node of an App without DevTools.
     #[derive(Clone)]
     pub(crate) enum StyleHook {
         Off,
         On {
             /// The parsed overrides of the node (tracked: it reads the
             /// DevTools override signal), merged over its style.
             overrides: Rc<dyn Fn() -> Option<Style>>,
             forced: Value<ForcedStates>,
         },
     }

     impl StyleHook {
         /// The hook of node `id`.
         pub(crate) fn new(rt: &Rc<Runtime>, owner: Owner, id: NodeId, at: &'static Location<'static>) -> StyleHook {
             match hooks(rt) {
                 Some(h) => h.style_hook(rt, owner, id, at),
                 None => StyleHook::Off,
             }
         }

         /// Apply the node's overrides over `style`.
         pub(crate) fn apply(&self, style: Style) -> Style {
             match self {
                 StyleHook::Off => style,
                 StyleHook::On { overrides, .. } => match overrides() {
                     Some(over) => style.merge(&over),
                     None => style,
                 },
             }
         }

         /// The states a tool forces on the node.
         pub(crate) fn forced(&self) -> ForcedStates {
             match self {
                 StyleHook::Off => ForcedStates::default(),
                 StyleHook::On { forced, .. } => forced.get(),
             }
         }
     }
     ```
     消すもの（旧ファイルにあり、新しい内容に無いもの）：`prop_written`（自由関数）、`is_running`、`wrap_capabilities`（自由関数）、`input_description`（次の手順で `devtools/hooks.rs` へ移す）、`#[cfg(not(feature = "devtools"))]` の stub すべて。
  5. 新しい `devtools/hooks.rs` を作る。旧 `view/devtools_hooks.rs` の `#[cfg(feature = "devtools")]` 側の本体を、`impl DevtoolsHooks for Inspector` の method に移す。規則：
     - 各関数の先頭の `let DevtoolsState::Running(i) = DevtoolsState::of(rt) else { return ...; };` を消し、`i` を `self` に読み替える（`i.hooks.x` → `self.hooks.x`、`i.session.x` → `self.session.x`）。
     - `prop_written` は `rt` を使わない（旧本体の `Runtime::current()` と `DevtoolsState::of` を消す）。
     - `wrap_capabilities` の本体：
       ```rust
       fn wrap_capabilities(&self, owner: Owner, host: Signal<Capabilities>) -> Value<Capabilities> {
           let emulation = self.hooks.emulation;
           owner.with_setup(|| {
               kivact_runtime::memo(move || {
                   let mut capabilities = host.get();
                   if let Some(touch) = emulation.get().touch {
                       capabilities.touch = touch;
                   }
                   capabilities
               })
           })
       }
       ```
     - `style_hook` の本体：旧 `StyleHook::new` の devtools 側。`Style` は `PartialEq` を実装しない（`kivact-style/src/sheet/fields.rs` の `#[derive(Clone, Debug, Default)] pub struct Style`）ので memo にしない。解析した `Style` を返す closure を渡し、node の style effect がそれを呼ぶ（読みは effect に追跡される）：
       ```rust
       fn style_hook(&self, rt: &Rc<Runtime>, owner: Owner, id: NodeId, at: &'static Location<'static>) -> StyleHook {
           let sites = super::apply::call_sites(rt, owner, Some(at));
           let overrides = self.hooks.overrides;
           let forced = self.hooks.forced;
           StyleHook::On {
               overrides: Rc::new(move || {
                   let pairs = super::apply::node_overrides(&overrides.get(), id, &sites);
                   let mut merged: Option<Style> = None;
                   for (property, value) in pairs {
                       if let Ok(over) = super::apply::override_style(&property, &value) {
                           merged = Some(match merged {
                               Some(style) => style.merge(&over),
                               None => over,
                           });
                       }
                   }
                   merged
               }),
               forced: owner.with_setup(|| {
                   kivact_runtime::memo(move || forced.with(|all| all.get(&id).copied().unwrap_or_default()))
               }),
           }
       }
       ```
       `ForcedStates` は `Copy + PartialEq + Default`（旧コードが memo にしている）。
     - `input_description` は `devtools/hooks.rs` の private 関数にする。
     - `input_event` の `injected` の判定（`i.session.pending` を走査する部分）はこの unit では変えない（C06 が RC-28 で消す）。
     - `DevtoolsState::Running(Box<Inspector>)` を `DevtoolsState::Running(Rc<Inspector>)` に変える（`devtools/state.rs`）。`from_slot` の `DevtoolsState::Running(Box::new(Inspector { .. }))` を `DevtoolsState::Running(Rc::new(Inspector { .. }))` にする。
     - `devtools/state.rs` に次を足す：
       ```rust
       impl DevtoolsState {
           /// The hooks the view calls; `None` while DevTools is off.
           pub(crate) fn hooks(&self) -> Option<Rc<dyn crate::view::devtools_hooks::DevtoolsHooks>> {
               match self {
                   DevtoolsState::Off => None,
                   DevtoolsState::Running(i) => Some(i.clone() as Rc<dyn crate::view::devtools_hooks::DevtoolsHooks>),
               }
           }
       }
       ```
  6. `view/state.rs`：
     - `ViewState` に field を足す：`pub(crate) devtools: Option<Rc<dyn crate::view::devtools_hooks::DevtoolsHooks>>,`（doc：`/// The DevTools hooks; None without DevTools. Fixed at construction.`）。
     - `ViewState::new` の引数に `devtools: Option<Rc<dyn crate::view::devtools_hooks::DevtoolsHooks>>` を足す（`engine` の次）。本体の `let capabilities = crate::view::devtools_hooks::wrap_capabilities(owner, host_capabilities);` を次にする：
       ```rust
       let capabilities = match &devtools {
           Some(h) => h.wrap_capabilities(owner, host_capabilities),
           None => host_capabilities.into(),
       };
       ```
       `tree: RefCell::new(ViewNodes::new(engine, owner))` を `tree: RefCell::new(ViewNodes::new(engine, owner, devtools.clone()))` にし、構造体の初期化に `devtools,` を足す。
     - `host_capabilities` の `#[allow(dead_code, reason = "the capabilities memo is the only reader ...")]` はそのまま（C05 が field ごと消す）。
  7. `view/tree.rs`：
     - `ViewNodes` に `devtools: Option<Rc<dyn crate::view::devtools_hooks::DevtoolsHooks>>,` を足し、`ViewNodes::new(engine: Writer, owner: Owner)` を `ViewNodes::new(engine: Writer, owner: Owner, devtools: Option<Rc<dyn crate::view::devtools_hooks::DevtoolsHooks>>)` にする。
     - `update` の `crate::view::devtools_hooks::prop_written(id, prop, writer, changed);` を次にする：
       ```rust
       if let Some(h) = &self.devtools {
           h.prop_written(id, prop, writer, changed);
       }
       ```
     - `set_style_layers` と `CoreRare.style_layers` の `#[cfg(feature = "devtools")]` を外す（常に compile する。値を入れるのは DevTools がある App だけ）。
  8. `view/mount/style.rs` の `bind_style`：
     ```rust
     #[cfg(feature = "devtools")]
     if crate::view::devtools_hooks::is_running(cx.rt) {
         crate::view::state::ViewState::tree_mut(cx.rt).set_style_layers(id, layers.clone());
     }
     ```
     を次にする（cfg を消す）：
     ```rust
     if crate::view::devtools_hooks::hooks(cx.rt).is_some() {
         crate::view::state::ViewState::tree_mut(cx.rt).set_style_layers(id, layers.clone());
     }
     ```
  9. theme と locale の wrap を一か所にする（RC-27 の「install_locale と install_theme が一か所で wrap する」）。`view/boundary.rs`：
     - 次の関数を足す：
       ```rust
       /// Provide `theme` to `owner`'s subtree, as DevTools shapes it.
       pub(crate) fn install_theme(owner: Owner, theme: Value<Theme>) {
           let rt = owner.runtime();
           let theme = crate::view::devtools_hooks::wrap_theme(&rt, owner, theme);
           kivact_style::internal::install_theme(owner, theme);
       }
       ```
     - 既存の `install_locale(owner, locale)` の本体を次にする：
       ```rust
       pub(crate) fn install_locale(owner: Owner, locale: Value<Locale>) {
           let rt = owner.runtime();
           let locale = crate::view::devtools_hooks::wrap_locale(&rt, owner, locale);
           owner.provide(LocaleCtx(locale));
       }
       ```
     - `provide_theme` の本体の 2 行
       ```rust
       let value = crate::view::devtools_hooks::wrap_theme(owner, value);
       kivact_style::internal::install_theme(owner, value);
       ```
       を `install_theme(owner, value);` の 1 行にする。
     - `provide_locale` はすでに `install_locale` を呼ぶので変えない（これで wrap される）。
     - `owner.runtime()` は `kivact_runtime::Owner::runtime(self) -> Rc<Runtime>`（`#[track_caller]`）。
  10. `view/root.rs` の `install_app_contexts`：
     - `let locale = crate::view::devtools_hooks::wrap_locale(owner, locale);` を消す（`install_locale` が wrap する）。
     - `let theme = crate::view::devtools_hooks::wrap_theme(owner, theme);` と `kivact_style::internal::install_theme(owner, theme);` の 2 行を `crate::view::boundary::install_theme(owner, theme);` にする。
  11. `launch/runtime.rs` の `Launch::start` の `Runtime::new(..., |_rt, root| { ... })` の closure：
     ```rust
     |_rt, root| {
         #[cfg(feature = "devtools")]
         let devtools = crate::devtools::state::DevtoolsState::from_slot(
             root,
             slot.take().expect("invariant: the build runs once"),
             at,
         );
         #[cfg(feature = "devtools")]
         let hooks = devtools.hooks();
         #[cfg(not(feature = "devtools"))]
         let hooks = None;
         let view = ViewState::new(root, info, writer.clone(), hooks, at);
         let motion_env = install_main_window(&view, root, info, options, at);
         let extensions = Extensions::new().with(view).with(motion_env);
         // （既存の with(...) の列。DevtoolsState を作っていた with(...) は次の 2 行に置き換える）
         #[cfg(feature = "devtools")]
         let extensions = extensions.with(devtools);
         // （以下は既存のまま）
     }
     ```
     旧 `extensions.with(crate::view::devtools_state::DevtoolsState::from_slot(root, slot.take()..., at))` を消す。手順 1 の perl で path は `crate::devtools::state` になっている。
     同じファイルの `start_mount` の末尾の 6 行（`#[cfg(feature = "devtools")] if let crate::devtools::state::DevtoolsState::Running(i) = ... { let keys = crate::devtools::start::engine_keys(i.agent.borrow().config()); ViewState::tree(rt).engine().set_devtools_keys(keys); }`）を次の 2 行にし、元の本体を `devtools/start.rs` の新しい関数に移す（C06 が launch を触らずに中身を変えられるように）：
     ```rust
     #[cfg(feature = "devtools")]
     crate::devtools::start::after_mount(rt);
     ```
     ```rust
     // devtools/start.rs
     /// The DevTools work after the App's root mounted: the engine learns the
     /// configured key chords.
     pub(crate) fn after_mount(rt: &kivact_runtime::Runtime) {
         if let crate::devtools::state::DevtoolsState::Running(i) = crate::devtools::state::DevtoolsState::of(rt) {
             let keys = engine_keys(i.agent.borrow().config());
             crate::view::state::ViewState::tree(rt).engine().set_devtools_keys(keys);
         }
     }
     ```
  12. compile error が残る箇所（`DevtoolsState::Running(i)` の `i` が `&Box<Inspector>` から `&Rc<Inspector>` に変わる、`hooks(rt)` の引数）を直す。`client/dispatch/*` の `crate::view::devtools_hooks::{handler_ran,input_event,focus_changed,focus_result,host_event}` と `view/requests.rs` の `focus_requested`、`view/boundary.rs` の `pseudo` は自由関数の名前と引数が同じなので変えない。
- **消すもの**：`is_running`、自由関数 `prop_written`・`wrap_capabilities`、`devtools_hooks.rs` の cfg の stub 全部（約 20 組）、`view.rs` の 3 つの cfg 付き `mod`。
- **状態の閉じ方**：`ViewState.devtools`・`ViewNodes.devtools` は constructor だけが書く（6.1 節「constructor だけで書く field は状態ではない」）。`StyleHook` は enum（`Off`/`On`）。印は要らない。
- **試験**：足す試験は無い。既存の `e2e_native` の `devtools.rs`（`a_devtools_client_inspects_drives_and_reviews_a_test_app`）と、locale emulation の試験がそのまま通ること。
- **確認（tier a）**：
  ```sh
  cargo fmt -p kivact-core
  cargo check -p kivact-core --all-targets
  cargo check -p kivact-core --all-targets --features devtools
  cargo clippy -p kivact-core --all-targets --features devtools -- -D warnings
  cargo clippy -p kivact-core --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --features devtools --test e2e_native -- devtools
  ```
- **完了条件**：`rg -n 'cfg\(not\(feature = "devtools"\)\)' crates/kivact-core/src/view` が 0 件。`rg -n "devtools_state|devtools_apply|inspect_mode" crates/kivact-core/src/view` が 0 件（`devtools_hooks.rs` の中を除き 0 件）。`rg -n "fn is_running|fn wrap_capabilities" crates/kivact-core/src/view` が 0 件。上のコマンドが通る。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C02：`kivact-offscreen` crate を作り、`Offscreen` を core から出す

- **wave / lane / 前提**：W2b / C / T（W1）、S02・S07（W2a。root と core・kivact の `Cargo.toml`、`workspace.toml` を先に編集してある）
- **所有**：新 `crates/kivact-offscreen/{Cargo.toml,src/lib.rs}`、`Cargo.toml`（workspace の `members` 1 行）、`crates/kivact-core/Cargo.toml`、`crates/kivact-core/src/draw.rs`、`crates/kivact-core/src/draw/offscreen.rs`（削除）、`crates/kivact/Cargo.toml`、`crates/kivact/src/lib.rs`、新 `crates/kivact/src/draw.rs`、`dev/xtask/policy/workspace.toml`（`visible_to` の行）
- **目的**：入力が Theme・font・描画の closure、出力が `Pixels` か SVG 文字列という葉を crate にし、core から kivact-svg への依存を消す（RC-29）。署名の変更は C17（W5）。
- **手順**：
  1. `crates/kivact-offscreen/Cargo.toml`：
     ```toml
     [package]
     name = "kivact-offscreen"
     authors.workspace = true
     version.workspace = true
     edition.workspace = true
     license.workspace = true
     repository.workspace = true
     description = "Kivact's windowless renderer: draw with `Draw` into RGBA pixels or an SVG 1.1 document"

     [dependencies]
     kivact-types = { path = "../kivact-types", version = "0.1.0" }
     kivact-runtime = { path = "../kivact-runtime", version = "0.1.0" }
     kivact-style = { path = "../kivact-style", version = "0.1.0" }
     kivact-engine = { path = "../kivact-engine", version = "0.1.0" }
     # The SVG 1.1 writer of `Offscreen::render_svg` (it owns the PNG encoding).
     kivact-svg = { path = "../kivact-svg", version = "0.1.0" }

     [lints]
     workspace = true
     ```
  2. `git mv crates/kivact-core/src/draw/offscreen.rs crates/kivact-offscreen/src/lib.rs`。`lib.rs` の先頭の `//! Role: Fixed-resolution images and SVG documents drawn with `Draw`.` はそのまま残し、2 行目に `#![forbid(unsafe_code)]` を足す（module doc の後）。
  3. `lib.rs` の `use` を書き換える：
     ```rust
     use kivact_engine::api::{OffscreenError, OffscreenRenderer, TextMeasurer};
     use kivact_runtime::{ErrorKind, Result, UiError};
     use kivact_style::Theme;
     use kivact_style::asset::Asset;
     use kivact_style::draw::{Draw, Recording};
     use kivact_svg::{Resources, SvgError};
     use kivact_types::{ComputedTextStyle, DrawCmd, DrawList, Pixels, Size, TextRun};
     use std::panic::Location;
     use std::sync::Arc;
     ```
     （旧 `use crate::draw::Draw;`、`use crate::geometry::Size;`、`use crate::{Result, UiError};`、`use kivact_runtime::ErrorKind;` を消す。）
  4. doc の例 3 か所（`Offscreen` の型 doc の `no_run` と `compile_fail`、`render_svg` の doc）の `# use kivact_core as kivact;` と `kivact::...` を次にする：
     ```rust
     /// use kivact_offscreen::Offscreen;
     /// use kivact_types::Rect;
     /// use kivact_style::rgb;
     ```
     `compile_fail` の例は `requires_send::<kivact_offscreen::Offscreen>();`。`Ok::<(), kivact::UiError>(())` は `Ok::<(), kivact_runtime::UiError>(())`。
  5. `crates/kivact-core/src/draw.rs` の末尾の 3 行
     ```rust
     mod offscreen;
     #[doc(inline)]
     pub use offscreen::Offscreen;
     ```
     を消す。module doc は変えない。
  6. `crates/kivact-core/Cargo.toml` の `kivact-svg = ...` の行とその上の comment 1 行を消す。
  7. `Cargo.toml`（workspace）の `members` に `"crates/kivact-offscreen",` を `"crates/kivact-svg",` の次に足す。
  8. facade：`crates/kivact/Cargo.toml` の `[dependencies]` に `kivact-offscreen = { path = "../kivact-offscreen", version = "0.1.0" }` を足す（`kivact-macros` の次）。`crates/kivact/src/lib.rs` の `pub use kivact_core::{custom, diagnostics, draw, geometry, ...}` から `draw,` を消し、次を足す：
     ```rust
     /// Drawing tools: canvases, local element layers and windowless rendering ([`draw::Offscreen`]).
     pub mod draw;
     ```
     新 `crates/kivact/src/draw.rs`：
     ```rust
     //! Role: The facade's drawing vocabulary: the core's painter types and the windowless renderer.

     #[doc(inline)]
     pub use kivact_core::draw::*;
     #[doc(inline)]
     pub use kivact_offscreen::Offscreen;
     ```
  9. `dev/xtask/policy/workspace.toml` の `[visible_to]`：
     - `kivact-svg = [...]` の行を `kivact-svg = ["kivact-offscreen"]` にし、その上の comment を `# ----- SVG export: the SVG 1.1 writer and its PNG encoding are confined to kivact-svg, which kivact-offscreen (Offscreen::render_svg) uses.` にする。
     - 次の 2 行を足す（`kivact-svg` の行の次）：
       ```toml
       # ----- Windowless rendering: a leaf the facade re-exports as kivact::draw::Offscreen.
       kivact-offscreen = ["kivact"]
       ```
     - `kivact-runtime`、`kivact-style`、`kivact-engine` の行の配列に `"kivact-offscreen"` を足す（offscreen は error の型 `kivact_runtime::{ErrorKind, Result, UiError}` を使うので kivact-runtime に依存する。他の runtime の API は使わない。INC-35：設計書 4.2 の `kivact-offscreen →` の行に `kivact-runtime` を足す）。
- **消すもの**：`crates/kivact-core/src/draw/offscreen.rs`（移動）、core の `kivact-svg` 依存。
- **状態の閉じ方**：新しい状態の型は無い（`Offscreen { engine, theme }` は移動だけ）。
- **試験**：`dev/tests/kivact-public-api/tests/offscreen.rs`（`a_size_outside_what_windows_draw_is_invalid_and_never_calls_paint`、`the_image_is_exactly_sized_transparent_and_draw_knows_its_size`、`rects_and_text_are_drawn_where_they_are_placed`）と `e2e_core` の `offscreen`・`offscreen_svg` が変更なしで通る（path は `kivact::draw::Offscreen` のまま）。
- **確認（tier a）**：
  ```sh
  cargo fmt -p kivact-offscreen -p kivact-core -p kivact
  cargo check -p kivact-offscreen --all-targets
  cargo check -p kivact-core --all-targets
  cargo check -p kivact --all-targets
  cargo check -p kivact-ui-basic --all-targets
  cargo clippy -p kivact-offscreen -p kivact-core -p kivact --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-offscreen
  cargo xtask check --no-write --crate kivact-core
  cargo test -p kivact-offscreen --doc
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test offscreen
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- offscreen
  ```
- **完了条件**：`rg -n "kivact-svg|kivact_svg" crates/kivact-core` が 0 件。`rg -n "Offscreen" crates/kivact-core/src` が 0 件。上のコマンドが通る。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C03：host への要求の帳簿を型付きの 2 つの表にし、小さな矛盾と dead code を消す

- **wave / lane / 前提**：W3a / C / C01
- **所有**：`crates/kivact-core/src/view/{state.rs,requests.rs,snapshots.rs,foreign_window.rs,mount/resolve.rs,node_ref.rs,elements/builders.rs}`、`crates/kivact-core/src/client/dispatch/{platform.rs,delivery.rs}`、`crates/kivact-core/src/platform/{app_handle.rs,services.rs,env.rs}`（services は置換、env は doc 1 行）、`crates/kivact-core/src/gpu_interop.rs`、`crates/kivact-core/src/gpu_interop/import/registry.rs`、`crates/kivact-core/src/launch/runtime.rs`
- **目的**：focus 要求の `order`・map・`RequestStage` の同期と、3 つの待ち手の map をやめる（RC-21）。`initial_render` を毎 turn 書かない、`current_window` の暗黙の MAIN を消す（RC-36）。RC-41 の小項目（`report_session` の偽 handler、未使用の引数 `_kind`・`_at`、`n.at` の二度書き、`take_request` の毎 turn の lock、`gpu_interop::State` の常時の作成）。
- **手順**：
  1. `view/state.rs`：次の型を消し、新しい型に置き換える。
     消す：`FocusRequests`、`FocusRequest`、`RequestStage`、`EffectWaiter`、`SnapshotWaiter`、`ForeignSizeRequest`、`Requests` の field `effects`・`foreign_sizes`・`snapshots`。
     足す：
     ```rust
     /// Focus requests: those waiting for the next commit, in order, and those
     /// the host has not answered yet.
     #[derive(Default)]
     pub(crate) struct FocusRequests {
         pub(crate) queued: Vec<(kivact_types::RequestId, FocusRequest)>,
         pub(crate) sent: HashMap<kivact_types::RequestId, FocusReply>,
     }

     /// One focus request waiting for the commit.
     pub(crate) struct FocusRequest {
         pub(crate) target: kivact_types::FocusTarget,
         pub(crate) reply: FocusReply,
         /// The requested focus indicator.
         pub(crate) visible: bool,
     }

     /// Who waits for a host answer, by request id.
     pub(crate) enum Waiter {
         /// Report a failure to the owner's error chain (the default sink
         /// without an owner), located at the call; a `Failed` answer is an
         /// error of kind `failed`.
         Report {
             owner: Option<Owner>,
             at: &'static Location<'static>,
             failed: ErrorKind,
         },
         /// Complete a future with the platform answer.
         Future(Completion<Result<PlatformResult, UiError>>),
         /// A size request of a foreign window's native child.
         ForeignSize {
             window: WindowId,
             at: &'static Location<'static>,
             slot: Completion<Result<(), UiError>>,
         },
         /// A CPU capture answered at the settled host boundary.
         Snapshot {
             owner: Owner,
             at: &'static Location<'static>,
             reply: Completion<Result<kivact_types::Pixels, UiError>>,
         },
     }
     ```
     `Requests` は次の field を持つ：`next_request: u64`、`focus: FocusRequests`、`waiters: HashMap<u64, Waiter>`、`pending_activations: Vec<kivact_types::FocusTarget>`（付属書 R の RC-02 が W3 以降に消す。この unit では残す）、`pointer_releases: Vec<(NodeId, kivact_types::PointerId)>`。
  2. `ViewState::resolve_foreign_size(&self, id, result) -> bool` を消し、その本体の `PlatformResult` から `Result<(), UiError>` への変換を `requests.rs` の private 関数 `fn foreign_size_outcome(result: PlatformResult, at: &'static Location<'static>) -> Result<(), UiError>` にする（本体は旧関数の `let outcome = match result { ... };` をそのまま使う）。
  3. `ViewState::close_foreign_sizes(&self, window)` の本体を次にする：
     ```rust
     pub(crate) fn close_foreign_sizes(&self, window: WindowId) {
         let drained: Vec<Waiter> = {
             let mut requests = self.requests.borrow_mut();
             let ids: Vec<u64> = requests
                 .waiters
                 .iter()
                 .filter(|(_, waiter)| matches!(waiter, Waiter::ForeignSize { window: w, .. } if *w == window))
                 .map(|(id, _)| *id)
                 .collect();
             ids.into_iter().filter_map(|id| requests.waiters.remove(&id)).collect()
         };
         for waiter in drained {
             if let Waiter::ForeignSize { at, slot, .. } = waiter {
                 slot.complete(Err(UiError::new(
                     ErrorKind::Closed,
                     "the foreign window closed before the size request was answered",
                     Some(at),
                 )));
             }
         }
     }
     ```
  4. `ViewState::release_capacity`：`requests.focus.order.shrink_to_fit(); requests.focus.requests.shrink_to_fit(); requests.effects...; foreign_sizes...; snapshots...` を `requests.focus.queued.shrink_to_fit(); requests.focus.sent.shrink_to_fit(); requests.waiters.shrink_to_fit();` にする（`pending_activations` と `pointer_releases` の行は残す）。
  5. `view/requests.rs`：
     - `request_focus`：`req.focus.order.push(id); req.focus.requests.insert(id, FocusRequest { .. stage: RequestStage::Queued })` を `req.focus.queued.push((id, FocusRequest { target: node, reply, visible }));` にする。
     - `request_effect(rt, req: HostRequest, waiter: EffectWaiter)` を `request_effect(rt, req: HostRequest, waiter: Waiter)` にし、`r.effects.insert(id, waiter)` を `r.waiters.insert(id, waiter)` にする。`register_app_fonts` の `EffectWaiter::Report {..}` を `Waiter::Report {..}` にする。
     - `resolve_effect` を次にする：
       ```rust
       pub(crate) fn resolve_effect(rt: &Runtime, id: u64, result: PlatformResult) {
           let Some(waiter) = ViewState::of(rt).requests.borrow_mut().waiters.remove(&id) else {
               return;
           };
           match waiter {
               Waiter::Report { owner, at, failed } => {
                   let err = match result {
                       PlatformResult::Failed(m) => Some(UiError::new(failed, m.to_string(), Some(at))),
                       other => platform_result(other)
                           .err()
                           .map(|e| UiError::new(e.kind().clone(), e.message().to_string(), Some(at))),
                   };
                   if let Some(e) = err {
                       rt.report(owner, e);
                   }
               }
               Waiter::Future(slot) => slot.complete(Ok(result)),
               Waiter::ForeignSize { at, slot, .. } => slot.complete(foreign_size_outcome(result, at)),
               Waiter::Snapshot { .. } => {
                   unreachable!("invariant: a snapshot is answered by SnapshotResult, not EffectResult")
               }
           }
       }
       ```
       （旧本体の先頭の `if ViewState::of(rt).resolve_foreign_size(id, result.clone()) { return; }` は消える。`result.clone()` も消える。）
     - `on_focus_result`：
       ```rust
       pub(crate) fn on_focus_result(rt: &Runtime, id: u64, result: Result<(), String>) {
           let reply = ViewState::of(rt).requests.borrow_mut().focus.sent.remove(&RequestId(id));
           if let Some(reply) = reply {
               let result = result.map_err(|reason| UiError::new(ErrorKind::Platform, reason, None));
               resolve_focus_reply(rt, reply, FocusResult::Answered(result));
           }
       }
       ```
     - `flush_focus`：`order` の代わりに `queued` を取り出す。
       ```rust
       pub(crate) fn flush_focus(rt: &Runtime) {
           let queued = std::mem::take(&mut ViewState::of(rt).requests.borrow_mut().focus.queued);
           let exists = |target: kivact_types::FocusTarget| ViewState::tree(rt).contains(target.node);
           let explicit = queued
               .iter()
               .any(|(_, request)| matches!(request.reply, FocusReply::Report(_)) && exists(request.target));
           for (id, request) in queued {
               let restore = !matches!(request.reply, FocusReply::Report(_));
               let owner_gone = matches!(&request.reply, FocusReply::RestoreAndReport(completion) if !completion.owner.is_alive());
               if restore && (explicit || owner_gone || !can_take_focus(rt, request.target)) {
                   resolve_focus_reply(rt, request.reply, FocusResult::Canceled);
                   continue;
               }
               if exists(request.target) {
                   let FocusRequest { target, reply, visible } = request;
                   ViewState::of(rt).requests.borrow_mut().focus.sent.insert(id, reply);
                   ViewState::tree(rt).engine().focus(target, id, visible);
                   crate::view::devtools_hooks::focus_requested(rt, target);
               } else {
                   resolve_focus_reply(
                       rt,
                       request.reply,
                       FocusResult::Answered(Err(UiError::new(ErrorKind::Platform, "focus: node was removed", None))),
                   );
               }
           }
       }
       ```
  6. `view/snapshots.rs`：`requests.snapshots.insert(id, SnapshotWaiter { owner, at, reply })` を `requests.waiters.insert(id, Waiter::Snapshot { owner, at, reply })` にする。答えを受ける関数（`requests.snapshots.remove(&id)` を含む）を次の形にする：
     ```rust
     let waiter = ViewState::of(rt).requests.borrow_mut().waiters.remove(&id);
     let Some(Waiter::Snapshot { owner, at, reply }) = waiter else {
         debug_assert!(waiter.is_none(), "invariant: a SnapshotResult answers a snapshot request");
         return;
     };
     ```
     （その後の処理は旧本体のまま。`SnapshotWaiter { owner, at, reply }` の分解を上の束縛に置き換える。）
  7. `view/foreign_window.rs`：`requests.foreign_sizes.insert(id, ForeignSizeRequest { window, at, slot })` を `requests.waiters.insert(id, Waiter::ForeignSize { window, at, slot })` にする。
  8. `client/dispatch/*` と `platform/services.rs` の `EffectWaiter::` を `Waiter::` に置き換える：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-core/src" || exit 1
     rg -l "EffectWaiter" . | xargs perl -pi -e 's/\bEffectWaiter\b/Waiter/g'
     ```
     `platform/services.rs` は所有に無いが、この置換だけは同じ unit の機械的な追従として行う。
  9. RC-36：
     - `launch/runtime.rs` の `Launch::turn` の `ViewState::of(&rt).initial_render.set(false);` を消し、`start_mount` の `setup.finish_setup();` の次の行に `rt.enter(|| ViewState::of(rt).initial_render.set(false));` を足す。
     - `view/state.rs` の `initial_render` の doc を `/// True until the first turn after the App's setup (written once, by start_mount).` にする。`platform/env.rs` の `is_initial_render` の doc の「after the first host revision」を「until the first turn after the App's setup」にする（`platform/env.rs` はこの 1 行だけ編集してよい）。
     - `current_window` の本体を次にする：
       ```rust
       pub(crate) fn current_window(owner: Owner) -> WindowId {
           owner
               .context::<WindowScope>()
               .expect("invariant: every owner is under a WindowScope")
               .0
       }
       ```
       doc を `/// The window of `owner`'s subtree. The App's root provides the main window's scope.` にする。
  10. RC-41 の小項目：
     - `view/mount/resolve.rs`：選択（selection）が無いときに `report_session` を真にするためだけに入れている何もしない handler を消し、`report_session` の計算を `let report_session = selection.is_some() || draw_with_layout.is_some();` にする（変数名は既存のもの。`selection` と `draw_with_layout` の実際の束縛名に合わせる）。同じファイルの未使用の引数 `_kind` を関数の引数と全呼出し元から消す。
     - `view/node_ref.rs`：未使用の引数 `_at` を引数と全呼出し元から消す。
     - `view/elements/builders.rs`：`n.at = Some(..)` の直後に `n.at = None` とする関数（`at` を二度書く関数）を、引数 `at: Option<&'static Location<'static>>` を受けて一度だけ書く形にし、呼出し元を直す。
  11. `platform/app_handle.rs`（RC-41）：`AppLink` に `requested: std::sync::atomic::AtomicBool` を足す。
     - `close()` で状態を `CloseRequested(..)` にする分岐と、`attach()` で `ClosePending` から `CloseRequested(..)` にする分岐で、lock を離す前に `self.requested.store(true, Ordering::Release)` を呼ぶ。
     - `take_request(&self, accept: bool) -> bool` の先頭に：
       ```rust
       if !self.requested.swap(false, Ordering::AcqRel) {
           return false;
       }
       ```
       を足す（その後は既存の lock の処理）。
     - `AppHandle::new` の `AppLink { state: .., }` に `requested: AtomicBool::new(false),` を足す。
  12. `gpu_interop`（RC-41）：`gpu_interop::State` を launch で常に作るのをやめる。
     - `launch/runtime.rs` の `let extensions = extensions.with(crate::gpu_interop::State::new(root, at));` を消す。
     - `gpu_interop/import/registry.rs` の `State::of(rt)`（`rt.ext::<State>()` を呼ぶ関数）を、`ViewState` が持つ `gpu_interop: kivact_types::cell::Deferred<crate::gpu_interop::State>` から `get_or_init(|| State::new(rt.root(), Location::caller()))` で取る形にする。`ViewState` に field `pub(crate) gpu_interop: kivact_types::cell::Deferred<crate::gpu_interop::State>,`（doc：`/// GPU interop state, made on the first adapter query or import.`）を足し、`ViewState::new` で `gpu_interop: kivact_types::cell::Deferred::new(),` とする。`State::new(root, at)` の署名は変えない。`State::of` を呼ぶ全箇所は `ViewState::of(rt).gpu_interop.get_or_init(|| State::new(rt.root(), Location::caller()))` を返す 1 つの関数 `pub(crate) fn state(rt: &Runtime) -> &State` を通す（`registry.rs` に置き、`#[track_caller]` を付ける）。
- **消すもの**：`RequestStage`、`FocusRequests.order`、`EffectWaiter`、`SnapshotWaiter`、`ForeignSizeRequest`、`Requests.{effects,foreign_sizes,snapshots}`、`ViewState::resolve_foreign_size`、`Launch::turn` の `initial_render.set(false)`、`report_session` のための偽 handler、引数 `_kind`・`_at`、`gpu_interop::State` の常時の install。
- **状態の閉じ方**：`FocusRequests`（`queued` と `sent` は別の段階の別の要求で、同じ事実を二度持たない。`Vec` と `HashMap` は 6.1 の対象の field 型ではない）。`Waiter` は enum。`AppLink` は `Mutex<LinkState>` と `AtomicBool` を持つ。対象の field は `AtomicBool` 一つなので印は要らない。`FocusRequest.visible` は bool 一つ。
- **試験**：足す試験は無い。`crates/kivact-core/src/platform/app_handle.rs` の単体試験と、`e2e_core` の `focus`・`app_handle`・`windows`、`e2e_native` の `foreign_window`・`gpu_interop` が通ること。
- **確認（tier a）**：
  ```sh
  cargo fmt -p kivact-core
  cargo check -p kivact-core --all-targets
  cargo check -p kivact-core --all-targets --features gpu-interop,devtools,embed
  cargo clippy -p kivact-core --all-targets --features gpu-interop,devtools,embed -- -D warnings
  cargo xtask check --no-write --crate kivact-core
  KIVACT_TEST_MIRROR=off cargo test -p kivact-core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- focus app_handle windows
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- foreign_window
  ```
- **完了条件**：`rg -n "RequestStage|EffectWaiter|SnapshotWaiter|ForeignSizeRequest|resolve_foreign_size" crates/kivact-core` が 0 件。`rg -n "initial_render.set" crates/kivact-core/src/launch` が `start_mount` の 1 件だけ。`rg -n "map_or\(WindowId::MAIN" crates/kivact-core` が 0 件。上のコマンドが通る。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C04：mount の費用を下げる（style の memo、境界の関数一つ、provenance の側表）

- **wave / lane / 前提**：W3a / C / C01
- **所有**：`crates/kivact-core/src/view/{mount/style.rs,mount/create.rs,boundary.rs,root.rs,tree.rs,node_modifiers.rs}`、`crates/kivact-core/src/diagnostics/why.rs`、`crates/kivact-core/src/devtools/node.rs`（`origins` を読む行だけ）
- **目的**：RC-22。style の層の closure を状態の変化ごとに呼び直さない。固定の theme は constant。境界（`scope`・`provide`・`provide_theme`・`provide_locale`・`provide_messages`）の写しを一つの関数にする。node の provenance（`origins`・`semantic_origins`）は記録中か debug build だけ（PERF-9）。`later_callers` を O(M) にする。RC-41 の `bind_states`（何もしない）と `Leaving.on_exit`（常に `Some`）を消す。
- **手順**：
  1. `view/mount/style.rs`：
     - `bind_states` を消す。呼出し元（`view/mount/create.rs` か `view/mount/resolve.rs` の `bind_states(cx, id, disabled, selected, at)`）を次にする：
       ```rust
       let states = States {
           disabled: disabled.map(InputProp::into_fn),
           selected: selected.map(InputProp::into_fn),
       };
       ```
       `States` の field を `pub(super)` にする。
     - `bind_style` の effect を、層の合成の memo と compile の effect の 2 段にする。`let theme = cx.theme();` の次に：
       `Style` は `PartialEq` を実装しないので、memo の値は `Rc` の同一性で比べる型に包む。`mount/style.rs` に置く：
       ```rust
       /// The merged layers of one node. Each recomputation is a new value, so
       /// equality is identity: the memo notifies only when it ran again.
       #[derive(Clone)]
       struct Merged(Rc<Style>);
       impl PartialEq for Merged {
           fn eq(&self, other: &Self) -> bool {
               Rc::ptr_eq(&self.0, &other.0)
           }
       }
       ```
       `bind_style` の `let theme = cx.theme();` の次に：
       ```rust
       // The layers' closures run again only when what they read changes, not
       // when the disabled or selected state or the forced states change.
       let merge_hook = hook.clone();
       let merged: Value<Merged> = cx.owner.with_setup(|| {
           kivact_runtime::memo(move || {
               kivact_runtime::pure(|| Merged(Rc::new(merge_hook.apply(merge_layers(&layers)))))
           })
       });
       ```
       effect の中の `let s = hook.apply(merge_layers(&layers));` を消し、`theme_sig.with(|theme| merged.with(|s| { let state = Interactive { .. }; kivact_style::internal::compile_style(&s.0, theme, state) }))` にする（`forced: hook.forced()` は effect の中の `hook` を使う）。`Value` の import を足す。`hook` は C01 で `Clone`（`Copy` ではない）になっている。
     - effect の中の 4 つの `tree.update(.., resolved.x.clone(), writer)` を、`reports.report(&rt, owner, id, &resolved, at)` と `feed_local_paint(..)` を先に呼んでから、`resolved` を分解して move する形にする：
       ```rust
       feed_local_paint(&mcx, id, &local_paint, &resolved, &reports, at);
       reports.report(&rt, owner, id, &resolved, at);
       let kivact_style::CompiledStyle { mut layout, paint, text, states, .. } = resolved;
       if let Some(fix) = fix_layout {
           fix(&mut layout);
       }
       let writer = rt.running_effect();
       let mut tree = ViewState::tree_mut(&rt);
       tree.update(id, PropValue::Layout(layout), writer);
       tree.update(id, PropValue::Paint(paint), writer);
       tree.update(id, PropValue::TextStyle(text), writer);
       tree.update(id, PropValue::States(states.unwrap_or_default()), writer);
       ```
       `CompiledStyle` の field 名が違う場合は、旧コードで読んでいた `resolved.layout`・`resolved.paint`・`resolved.text`・`resolved.states` の名前に合わせる。
  2. `view/root.rs` の `install_app_contexts`：固定の theme を signal に包むのをやめる。
     ```rust
     let theme = match theme.fixed() {
         Some(theme) => owner.constant(theme, at),
         None => owner.with_setup(|| theme.into_value(owner, at)),
     };
     ```
     その上の comment（「The root publishes a theme source even for a fixed startup value ...」）を `// A fixed theme is a constant; DevTools makes it reactive through its wrap.` にする。
  3. `view/boundary.rs`：5 つの境界の本体の写しを一つの関数にする。
     ```rust
     /// The one owner boundary: a child owner of the mounting owner, `install`
     /// run in it, then `build` set up in it and mounted under a fragment.
     fn boundary<V: IntoView>(
         at: &'static Location<'static>,
         install: impl FnOnce(Owner) + 'static,
         build: impl FnOnce() -> V + 'static,
     ) -> View {
         view_fn(move |cx| {
             let owner = cx.owner.child(at);
             install(owner);
             let fragment = cx.fragment();
             let child = mount_in(cx.rt, owner, cx.window, move || build().into_view());
             cx.append(fragment, child);
             fragment
         })
     }
     ```
     `scope` は `boundary(at, |_| {}, build)`、`provide` は `boundary(at, move |owner| { let value = owner.constant(value, at); owner.provide(value); }, child)`（C08 がこの中身を変える）、`provide_theme` は `boundary(at, move |owner| install_theme(owner, theme.into_value(owner, at)), child)`、`provide_locale` は `boundary(at, move |owner| install_locale(owner, locale.into_value(owner, at)), child)`、`provide_messages` は `boundary(at, move |owner| install_layered_messages(owner, messages.into_value(owner, at), at), child)`。各関数の `let theme = theme.into_prop();` などの前処理（`view_fn` の外で `into_prop` する行）は残す。
  4. provenance の側表（PERF-9）：`view/tree.rs`
     - `CoreNode.origins: Vec<(Prop, EffectId)>` と `CoreRare.semantic_origins` を消す。
     - `ViewNodes` に次を足す：
       ```rust
       /// Which effect last set each dynamic prop and where semantics were
       /// declared, kept only while a recorder runs (DevTools, a test trace)
       /// or in debug builds.
       provenance: IdMap<Provenance>,
       ```
       ```rust
       /// The diagnostic origins of one node (`diagnostics::why`, DevTools).
       #[derive(Default)]
       pub(crate) struct Provenance {
           pub(crate) origins: Vec<(Prop, EffectId)>,
           pub(crate) semantic_origins: Vec<&'static Location<'static>>,
       }
       ```
     - `fn records_provenance(&self) -> bool { cfg!(debug_assertions) || self.devtools.is_some() }`。（test の trace は debug build で走るので debug の条件で足りる。）
     - `update` の `if let Some(writer) = writer { match node.origins... }` を `if let Some(writer) = writer && self.records_provenance() { let p = self.provenance.entry(id).or_default(); match p.origins.iter_mut().find(..) { .. } }` にする。
     - `semantic_origins` を書く関数（`semantic_origins` に push する箇所）と読む関数 `CoreNode::semantic_origins(&self)` を `ViewNodes::semantic_origins(&self, id) -> &[&'static Location<'static>]`（`self.provenance.get(&id).map_or(&[], |p| &p.semantic_origins)`）に移し、書く側も `records_provenance()` が真のときだけ書く。
     - `remove` で `self.provenance.remove(id)` を足す。`release_capacity` の `node.origins.shrink_to_fit()` を消し `self.provenance.shrink_to_fit()` を足す。
     - `diagnostics/why.rs` と `devtools/node.rs` の `node.origins` の読み出しを `tree.provenance(id).map_or(&[][..], |p| &p.origins)` にする（`pub(crate) fn provenance(&self, id: NodeId) -> Option<&Provenance>` を `ViewNodes` に足す）。
     - `const _: () = assert!(std::mem::size_of::<CoreNode>() <= 64, ...)` は残す（小さくなる）。
  5. `view/tree.rs` の `Leaving.on_exit: Option<..>`（常に `Some` の field）を `Option` でない型にし、`Some(..)` で作る箇所と `if let Some(on_exit)` で読む箇所を直す。
  6. `view/node_modifiers.rs` の `resolve_modifiers`：`later_callers` を後ろからの累積で作る（O(M·P)）。
     ```rust
     let mut later_callers: Vec<Vec<crate::view::semantics_data::SemanticsPatch>> = vec![Vec::new(); modifiers.len()];
     let mut suffix: Vec<crate::view::semantics_data::SemanticsPatch> = final_caller.semantics.clone();
     for index in (0..modifiers.len()).rev() {
         later_callers[index] = suffix.clone();
         if let Modifier::Caller(c) = &modifiers[index] {
             let mut with_this: Vec<_> = c.semantics.clone();
             with_this.extend(suffix);
             suffix = with_this;
         }
     }
     ```
     （旧の `(0..modifiers.len()).map(|index| modifiers[index + 1..]...collect())` を置き換える。順序は旧と同じ：index より後の Caller の semantics を前から順に、その後に `final_caller.semantics`。）
- **消すもの**：`bind_states`、`CoreNode.origins`、`CoreRare.semantic_origins`、5 つの境界の写し、root の固定 theme の signal、`Leaving.on_exit` の `Option`。
- **状態の閉じ方**：`Provenance` は `Vec` 2 つで、対象の field 型を持たない。`ViewNodes.provenance` は `IdMap`（対象外）。
- **試験**：足す試験は無い。`e2e_core` の `diagnostics`（why の試験）、`flows`、`setup_functions`、`i18n` が通ること。
- **確認（tier a）**：
  ```sh
  cargo fmt -p kivact-core
  cargo check -p kivact-core --all-targets --features devtools
  cargo clippy -p kivact-core --all-targets --features devtools -- -D warnings
  cargo xtask check --no-write --crate kivact-core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- diagnostics flows setup_functions i18n style_paint
  ```
- **完了条件**：`rg -n "fn bind_states|\.origins\b" crates/kivact-core/src/view/mount` が 0 件。`rg -n "owner.signal\(theme" crates/kivact-core/src/view/root.rs` が 0 件。`boundary.rs` の `view_fn(` が `boundary` 関数の 1 件だけ。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C05：window の記録を一つにする（main を含む）と、環境の field を使う時に作る

- **wave / lane / 前提**：W4 / C / C03、C04
- **所有**：`crates/kivact-core/src/view/{state.rs,window_view.rs,foreign.rs,foreign_window.rs,root.rs,hover.rs,tree.rs}`、`crates/kivact-core/src/launch/{runtime.rs,close.rs}`、`crates/kivact-core/src/client/dispatch/env.rs`、`crates/kivact-core/src/platform/env.rs`、`crates/kivact-core/src/custom/node.rs`（`PointerObservation` を読む箇所があれば）
- **目的**：RC-19。一つの window の宣言を 3 か所に持つのをやめ、`ForeignWindows` の写しの表と `PointerObservation::Plain` を消す。main window も同じ `open_window_record` で作る。RC-41：`EnvState::new` が 10 個の memo を先に作るのをやめ、型付きの遅延 getter にする。dead field `host_capabilities`・`title`・`mode` を消す。
- **手順**：
  1. `view/state.rs` の `WindowRecord` を次にする（`#[derive(Clone)]` は残す）：
     ```rust
     /// One open window: its declaration, environment, focus source and the
     /// pointer sources of the window.
     #[derive(Clone)]
     pub(crate) struct WindowRecord {
         /// The owner that declared the window (the root owner for the main one).
         pub(crate) declared_in: Owner,
         /// The declaration: title, open binding, sizes, mode, placement,
         /// dismiss handler and the foreign hooks. Shared with the window slot.
         pub(crate) decl: Rc<WindowDecl>,
         pub(crate) env: Rc<EnvState>,
         pub(crate) focus: FocusKeys,
         /// The node under the pointer, as the host reports it.
         pub(crate) hover: Signal<Option<FocusTarget>>,
         /// The mouse position in the window.
         pub(crate) mouse: Signal<Option<crate::geometry::Point>>,
     }

     /// What declares a window. The main window's is built by the launch from
     /// the App's settings.
     pub(crate) struct WindowDecl {
         pub(crate) title: Value<String>,
         pub(crate) open: Binding<bool>,
         pub(crate) size: kivact_types::Size,
         pub(crate) min_size: Option<kivact_types::Size>,
         pub(crate) mode: WindowMode,
         pub(crate) placement: kivact_types::WindowPlacement,
         pub(crate) dismiss: Option<Handler<DismissReason>>,
         /// The native child machinery of a `foreign_window`.
         pub(crate) foreign: Option<Rc<super::foreign::ForeignHooks>>,
     }
     ```
     （`Binding` は付属書 R が W5 で `Signal` に統合する。この unit の時点では `Binding` のまま書く。）
     `PointerObservation` を消す。`ViewState::{hover, set_hover, mouse, set_mouse}` を消し、使う側は `with_window(window, |r| r.hover)` で signal を得る。
  2. `WindowDecl` の field は builder（`Window`）が値渡しで作り、作った後は読むだけなので状態ではない（6.1 節）。`min_size`・`dismiss`・`foreign` の 3 つの `Option` は「constructor だけで書く」の例外に当たる。印は要らない。
  3. 一つの作成関数を `view/window_view.rs` に置く：
     ```rust
     /// Record `window`, declared by `declared_in` with `decl`: its owner
     /// scope, environment, focus and pointer sources. Every window, the main
     /// one included, is recorded here.
     pub(crate) fn open_window_record(
         rt: &Rc<Runtime>,
         window: WindowId,
         declared_in: Owner,
         owner: Owner,
         decl: Rc<WindowDecl>,
         env: Environment,
         surface_active: bool,
         at: &'static Location<'static>,
     ) -> WindowRecord {
         owner.provide(WindowScope(window));
         WindowRecord {
             declared_in,
             decl,
             env: Rc::new(EnvState::new(owner, env, surface_active, at)),
             focus: FocusKeys::new(owner, at),
             hover: owner.signal(None, at),
             mouse: owner.signal(None, at),
         }
     }
     ```
     `WindowSlot::begin` の `state.install_window(id, WindowRecord { .. })` と `owner.provide(WindowScope(id));` を、`state.install_window(id, open_window_record(rt, id, self.parent, owner, self.decl.clone(), parent_env.environment(), parent_env.surface_active(), self.at));` にする。`WindowSlot` の field `title, open, size, min_size, mode, placement, dismiss` を `decl: Rc<WindowDecl>` 一つにし、`content: SlotContent` は残す。`SlotContent::Foreign(hooks)` は `decl.foreign` と同じ `Rc` を指すので、`SlotContent` を `enum SlotContent { Views(Option<ViewFn<()>>), Foreign }` にし、hooks は `decl.foreign` から読む。
  4. `launch/runtime.rs` の `install_main_window` を `open_window_record` を使う形にする：
     ```rust
     let decl = Rc::new(WindowDecl {
         title: root.signal(options.settings.title.clone(), at).into_value(),
         open: root.signal(true, at).into_binding(),
         size: options.settings.window_size.unwrap_or(kivact_types::defaults::DEFAULT_WINDOW_SIZE),
         min_size: options.settings.min_window_size,
         mode: WindowMode::Normal,
         placement: kivact_types::WindowPlacement::Auto,
         dismiss: None,
         foreign: None,
     });
     let record = crate::view::window_view::open_window_record(rt, WindowId::MAIN, root, root, decl, info.env.clone(), info.surface_active, at);
     let env = record.env.clone();
     view.install_window(WindowId::MAIN, record);
     ```
     `install_main_window` は `rt: &Rc<Runtime>` を受け取る（Runtime の build closure の `_rt` 引数を使う。名前を `rt` にする）。`start_mount` の `root_owner.provide(WindowScope(WindowId::MAIN));` を消す（`open_window_record` が provide する）。
  5. foreign window の hooks を縮める（`view/foreign.rs`）：`ForeignHooks` から `title, open, size, min_size, mode, placement, dismiss` を消し、残りを `{ resizable: Value<bool>, ready: Option<Handler<ForeignChild>>, resize: Option<Handler<SurfaceSize>>, destroy: Option<Box<dyn Fn()>>, phase: Cell<ForeignPhase> }` にする（`ready`・`resize`・`destroy` の `RefCell<Option<..>>` は builder が作った後に書かれないので `Option` にする）。`ForeignHooks::ready`・`resized` の `self.ready.borrow().clone()` を `self.ready.clone()` にする。`view/foreign_window.rs` の `into_view` で `ForeignHooks { .. }` を作る箇所と `foreign_window_view(hooks, at)` を、`WindowDecl` と縮めた `ForeignHooks` を作って `Window { decl, content: SlotContent::Foreign, at }.into_view()` を返す形にする。`foreign_window_view` を消す。
  6. `ForeignWindows` を消す（`view/foreign.rs` の型と `register`・`close_all_for_exit`、`launch/runtime.rs` の `extensions.with(crate::view::foreign::ForeignWindows::default())`、`mount_foreign` の `ForeignWindows::register(rt, hooks);`）。代わりに `view/state.rs` に：
     ```rust
     impl ViewState {
         /// Write `false` to the `open` binding of every foreign window, as the
         /// App's exit turn does. Owners are disposed by the next stabilization,
         /// so `on_destroy` runs while the child is still alive.
         pub(crate) fn close_foreign_for_exit(&self) {
             let open: Vec<Rc<WindowDecl>> = self
                 .windows
                 .borrow()
                 .values()
                 .filter(|record| record.decl.foreign.is_some())
                 .map(|record| record.decl.clone())
                 .collect();
             for decl in open {
                 let released = decl
                     .foreign
                     .as_ref()
                     .is_some_and(|hooks| hooks.phase.get() == super::foreign::ForeignPhase::Released);
                 if !released && decl.open.get_untracked() {
                     decl.open.set(false);
                 }
             }
         }
     }
     ```
     `launch/close.rs` と `client/dispatch/env.rs` の `crate::view::foreign::ForeignWindows::close_all_for_exit(rt);` を `ViewState::of(rt).close_foreign_for_exit();` にする（C12 がこの 2 か所を `begin_exit` 一つにまとめる）。
  7. `view/hover.rs` の `install`（root の mount 時に hover の source を入れる関数）を消す。hover は `WindowRecord.hover` が最初から signal。`view/root.rs` の `mount_window_root` の `crate::view::hover::install(rt, window, owner, at);` を消す。`hover.rs` の `PointerObservation` の分岐（`Plain` を `Reactive` に昇格する処理）を、`hover` signal を直接 `set`/`get` する形にする。`client/dispatch/*` の hover・mouse の報告は `ViewState::of(rt).with_window(window, |r| r.hover).map(|s| s.set(target))` の形にする（`rt.enter(|| ..)` で包んでいた箇所はそのまま包む）。
  8. 前もって作る main の root をやめる（`view/tree.rs`）：`ViewNodes::new` の中で main window の root node を作る処理を消す。`mount_window_root` の `let existing = ViewState::tree(rt).root(window); let root = match existing { Some(root) => root, None => ViewState::tree_mut(rt).create_root(window, owner) };` を `let root = ViewState::tree_mut(rt).create_root(window, owner);` にする。main window の root は `start_mount` の `mount_window_root(rt, WindowId::MAIN, ...)` が作る（既存の呼出し）。
  9. `EnvState` の遅延（RC-41）：`view/state.rs` の `EnvState` を次にする。
     ```rust
     pub(crate) struct EnvState {
         owner: Owner,
         env: Signal<Environment>,
         pub(crate) surface_active: Signal<bool>,
         fields: EnvFields,
     }

     /// One memo per environment field, made when a reader first asks.
     #[derive(Default)]
     struct EnvFields {
         fullscreen: Deferred<Value<bool>>,
         reduced_motion: Deferred<Value<bool>>,
         scale_factor: Deferred<Value<f64>>,
         safe_area: Deferred<Value<kivact_types::Insets>>,
         window_size: Deferred<Value<kivact_types::Size>>,
         pointer: Deferred<Value<kivact_types::PointerSettings>>,
         keyboard_inset: Deferred<Value<f64>>,
         theme_hint: Deferred<Value<kivact_types::ThemeHint>>,
         locale: Deferred<Value<Option<String>>>,
         frame_period_ns: Deferred<Value<u64>>,
     }
     ```
     各 field の型は `Environment` の同名 field の型に合わせる（`kivact_types::Environment` の定義を開いて写す。型が上と違えば `Environment` の型を使う）。`Deferred` は `kivact_types::cell::Deferred`。
     `EnvState::field(name, f)` を消し、field ごとの getter を足す：
     ```rust
     impl EnvState {
         fn memo_of<T: kivact_runtime::PropValue>(&self, slot: &Deferred<Value<T>>, f: fn(&Environment) -> T) -> Value<T> {
             let env = self.env;
             *slot.get_or_init(|| self.owner.with_setup(|| kivact_runtime::memo(move || env.with(f))))
         }
         pub(crate) fn fullscreen(&self) -> Value<bool> { self.memo_of(&self.fields.fullscreen, |e| e.fullscreen) }
         pub(crate) fn reduced_motion_value(&self) -> Value<bool> { self.memo_of(&self.fields.reduced_motion, |e| e.reduced_motion) }
         pub(crate) fn scale_factor(&self) -> Value<f64> { self.memo_of(&self.fields.scale_factor, |e| e.scale_factor) }
         pub(crate) fn safe_area(&self) -> Value<kivact_types::Insets> { self.memo_of(&self.fields.safe_area, |e| e.safe_area) }
         pub(crate) fn window_size(&self) -> Value<kivact_types::Size> { self.memo_of(&self.fields.window_size, |e| e.window_size) }
         pub(crate) fn pointer(&self) -> Value<kivact_types::PointerSettings> { self.memo_of(&self.fields.pointer, |e| e.pointer) }
         pub(crate) fn keyboard_inset(&self) -> Value<f64> { self.memo_of(&self.fields.keyboard_inset, |e| e.keyboard_inset) }
         pub(crate) fn theme_hint(&self) -> Value<kivact_types::ThemeHint> { self.memo_of(&self.fields.theme_hint, |e| e.theme_hint) }
         pub(crate) fn locale(&self) -> Value<Option<String>> { self.memo_of(&self.fields.locale, |e| e.locale.clone()) }
         pub(crate) fn frame_period_ns(&self) -> Value<u64> { self.memo_of(&self.fields.frame_period_ns, |e| e.frame_period_ns) }
     }
     ```
     `EnvState::new` の 10 行の `state.field(..)` を消す。`env.field("is_motion_reduced", |e| e.reduced_motion)` などの呼出し（`launch/runtime.rs` の `MotionEnv`、`view/root.rs` の `env.field("locale", ..)`、`platform/env.rs` の各 accessor）を対応する getter に置き換える（`field("is_fullscreen", ..)` → `fullscreen()`、`"is_motion_reduced"` → `reduced_motion_value()`、`"scale_factor"` → `scale_factor()`、`"safe_area"` → `safe_area()`、`"window_size"` → `window_size()`、`"pointer_settings"` → `pointer()`、`"keyboard_inset"` → `keyboard_inset()`、`"theme_hint"` → `theme_hint()`、`"locale"` → `locale()`、`"frame_period_ns"` → `frame_period_ns()`）。`MotionEnv` の `reduced_motion` は runtime が最初の commit で読むので、main window では起動時に作られる（それ以外は使う時）。
     `ThemeHint` は付属書 S が W6 で `ThemeMode` に置き換える。この unit では今の名前で書く。
  10. dead field：`ViewState.host_capabilities` を消し（`ViewState::new` の `let host_capabilities = owner.signal(..)` は `capabilities` を作るためにだけ残し、field にしない）、host から capabilities が変わる event があればその書き込み先を `capabilities` の元の signal にする。書き込みが無ければ（`rg -n "host_capabilities" crates/kivact-core` で `state.rs` 以外に 0 件）field を消すだけでよい。`WindowRecord.title`・`WindowRecord.mode` は手順 1 で `WindowDecl` に移り、DevTools の `list_windows` などが読む（`rg -n "record.title|record.mode|\.title\b" crates/kivact-core/src/devtools` で読む箇所を `record.decl.title` に直す）。
- **消すもの**：`PointerObservation`、`ForeignWindows`、`foreign_window_view`、`hover::install`、`ViewNodes::new` の main の root、`EnvState::field`、`EnvState.fields: RefCell<Vec<(&str, Box<dyn Any>)>>`、`ViewState::{hover,set_hover,mouse,set_mouse}`、`ViewState.host_capabilities`、`ForeignHooks` の 7 つの宣言の field。
- **状態の閉じ方**：`WindowRecord` は signal を 2 つ（`hover`・`mouse`）持ち、`&mut self` の method を持たない。内部可変の field（signal）は構築の後に書かれる。対象になるので、`WindowRecord` の上に次の 1 行を置く：`// xtask-allow(state): hover と mouse は host の報告の別々の事実（指している node と指の位置）で、どの組合せも起きる`。`EnvFields` は `Deferred` だけ（6.1 の対象の型を持たない）。`ForeignHooks` は `Option` 3 つ（builder が作った後は読むだけ）と `Cell<ForeignPhase>`（enum）。対象の field は `Cell` 一つなので印は要らない。
- **試験**：足す試験は無い。`e2e_core` の `windows`（全 10 本）・`hover`・`environment`・`lifecycle`、`e2e_native` の `foreign_window`、`dev/tests/kivact-public-api/tests/foreign_window_run.rs`（`--features embed`）が通ること。
- **確認（tier a）**：
  ```sh
  cargo fmt -p kivact-core
  cargo check -p kivact-core --all-targets --features devtools,embed,gpu-interop
  cargo clippy -p kivact-core --all-targets --features devtools,embed,gpu-interop -- -D warnings
  cargo xtask check --no-write --crate kivact-core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- windows hover environment lifecycle
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- foreign_window
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --features embed --test foreign_window_run
  ```
- **完了条件**：`rg -n "PointerObservation|ForeignWindows|foreign_window_view|fn field\(" crates/kivact-core/src` が 0 件。`rg -n "WindowRecord \{" crates/kivact-core/src` が `open_window_record` の 1 件だけ。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C06：DevTools の agent を UI thread の外で起動し、木の通知・find・剪定を engine の変更記録で行う、入力の出所を型で持つ

- **wave / lane / 前提**：W4b / C / C01、C05、付属書 E の E08（E-REQ-5 の変更記録。W4）と E30（E-REQ-6 の `InputOrigin` と window 付きの `TreeChanges`。W4）、付属書 R の R04（`Runtime::reported_since`。W2a）
- **所有**：`crates/kivact-core/src/devtools/`（全ファイル）、`crates/kivact-core/src/view/devtools_hooks.rs`、`crates/kivact-core/src/view/mount/style.rs`（`StyleHook::new` を呼ぶ 1 か所）、`crates/kivact-core/src/launch/inspect.rs`、`crates/kivact-devtools-agent/src/{source.rs,agent.rs}`、`crates/kivact-test/src/screen/state.rs`（`launch/runtime.rs` は持たない。C01 が launch から呼ぶ関数を `agent_slot`・`from_slot`・`after_mount` の 3 つに絞ってあるので、その名前と呼び方を変えずに中身だけを変える）
- **目的**：PERF-10（起動の critical path で agent を起動しない）と RC-10 の残り（agent の非同期起動、node ごとの style hook の memo 2 つをやめて App 全体の値一つ）。RC-09（毎 turn の O(木) の作業）。RC-28（注入された入力の判定を推測しない）。
- **手順**：
  1. agent を `Send` にする：`crates/kivact-devtools-agent/src/source.rs` の `Rc<str>` をすべて `Arc<str>`（`use std::sync::Arc;`、`Rc::clone` → `Arc::clone`）にする。`agent.rs` の `pub struct Agent` の後に：
     ```rust
     // The App starts the agent on a background thread and moves it to the UI thread.
     const _: fn() = || {
         fn send<T: Send>() {}
         send::<Agent>();
     };
     ```
  2. `devtools/state.rs` の `AgentSlot` を次にする：
     ```rust
     /// The agent of an App whose launch allows DevTools.
     pub(crate) enum AgentSlot {
         /// Starting on a background thread; the first turn after it is ready promotes it.
         Starting(std::sync::mpsc::Receiver<std::io::Result<kivact_devtools_agent::Agent>>),
         Ready(kivact_devtools_agent::Agent),
         /// It could not start (already warned).
         Failed,
     }
     ```
     `DevtoolsState::from_slot(root, slot, at)` の名前は変えず、引数 `slot` の型を `Option<AgentStart>` にする（`None`：DevTools は off）。`Inspector.agent: RefCell<Agent>` を `agent: RefCell<AgentSlot>` にする。`AgentStart` は手順 3。launch の `let mut slot = Some(crate::devtools::start::agent_slot(..));` と `from_slot(root, slot.take().expect(..), at)` は型がそのまま流れるので変えない。
  3. `devtools/start.rs` の `agent_slot(title, allowed, can_inject, waker) -> AgentSlot` を、同じ名前で、型 `AgentStart` と戻り `Option<AgentStart>` の形にする：
     ```rust
     /// What a launch that allows DevTools asks for.
     pub(crate) struct AgentStart {
         pub(crate) receiver: std::sync::mpsc::Receiver<std::io::Result<Agent>>,
         pub(crate) can_inject: bool,
     }

     /// Start the agent on a background thread when the launch and the
     /// environment allow it; `None` otherwise.
     pub(crate) fn agent_slot(title: &str, allowed: bool, can_inject: bool, waker: ThreadWaker) -> Option<AgentStart> {
         if !allowed || std::env::var("KIVACT_DEVTOOLS").as_deref() == Ok("0") || is_devtools_app() {
             return None;
         }
         let app = /* 既存の p::AppInfo { .. } の組立てをそのまま */;
         let (sender, receiver) = std::sync::mpsc::channel();
         let spawned = std::thread::Builder::new()
             .name("kivact-devtools-start".to_owned())
             .spawn(move || {
                 let wake = waker.clone();
                 let started = Agent::start(AgentConfig { app }, Box::new(move || wake.wake()));
                 let _ = sender.send(started);
                 waker.wake();
             });
         match spawned {
             Ok(_) => Some(AgentStart { receiver, can_inject }),
             Err(error) => {
                 tracing::warn!(target: "kivact::devtools", "cannot start the DevTools agent: {error}");
                 None
             }
         }
     }
     ```
     `std::env::current_exe()`・`current_dir()` を読む `AppInfo` の組立ては thread の外のまま（速い）。`is_devtools_app` は変えない。
     `Inspector` に `can_inject: bool` は残す（`AgentStart.can_inject` から入れる）。
  4. 昇格（promotion）：`devtools/turn.rs` の `take_incoming(rt)` を次にする：
     ```rust
     pub(crate) fn take_incoming(rt: &Runtime) -> Vec<Incoming> {
         let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
             return Vec::new();
         };
         i.promote(rt);
         match &mut *i.agent.borrow_mut() {
             AgentSlot::Ready(agent) => agent.take_incoming(),
             AgentSlot::Starting(_) | AgentSlot::Failed => Vec::new(),
         }
     }
     ```
     `Inspector::promote(&self, rt: &Runtime)`（`devtools/state.rs`）：
     ```rust
     pub(crate) fn promote(&self, rt: &Runtime) {
         let started = match &*self.agent.borrow() {
             AgentSlot::Starting(receiver) => match receiver.try_recv() {
                 Ok(started) => started,
                 Err(std::sync::mpsc::TryRecvError::Empty) => return,
                 Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                     Err(std::io::Error::other("the DevTools start thread ended"))
                 }
             },
             AgentSlot::Ready(_) | AgentSlot::Failed => return,
         };
         let next = match started {
             Ok(agent) => {
                 if !cfg!(debug_assertions) {
                     tracing::warn!(target: "kivact::devtools", "the DevTools agent is enabled in a release build");
                 }
                 let keys = super::start::engine_keys(agent.config());
                 let tree = crate::view::state::ViewState::tree(rt);
                 tree.engine().set_devtools_keys(keys);
                 tree.reader().record_tree_changes(true);
                 drop(tree);
                 AgentSlot::Ready(agent)
             }
             Err(error) => {
                 tracing::warn!(target: "kivact::devtools", "cannot start the DevTools agent: {error}");
                 AgentSlot::Failed
             }
         };
         *self.agent.borrow_mut() = next;
     }
     ```
     （`Reader::record_tree_changes` は E08 の名前。E-REQ-5。）
     `devtools/start.rs` の `after_mount(rt)`（C01 が作った、launch が root の mount の後に呼ぶ関数）の本体を `if let DevtoolsState::Running(i) = DevtoolsState::of(rt) { i.promote(rt); }` にする（engine の key の設定は昇格のときに移った）。`launch/runtime.rs` は編集しない。
  5. agent を使う箇所（`rg -n "\.agent\.borrow" crates/kivact-core/src/devtools crates/kivact-core/src/launch`）を次の 2 つの method で書き直す（`devtools/state.rs` に置く）：
     ```rust
     impl Inspector {
         /// The ready agent; `None` while it starts or after it failed.
         pub(crate) fn agent(&self) -> Option<std::cell::RefMut<'_, kivact_devtools_agent::Agent>> {
             std::cell::RefMut::filter_map(self.agent.borrow_mut(), |slot| match slot {
                 AgentSlot::Ready(agent) => Some(agent),
                 AgentSlot::Starting(_) | AgentSlot::Failed => None,
             })
             .ok()
         }
         /// The ready agent, for a call that only a ready agent delivers.
         pub(crate) fn ready_agent(&self) -> std::cell::RefMut<'_, kivact_devtools_agent::Agent> {
             self.agent().expect("invariant: calls arrive only from a ready agent")
         }
     }
     ```
     `wants(topic)` は `self.agent().is_some_and(|a| a.wants(topic))`、`notify(n)` は `if let Some(a) = self.agent() { a.notify(n) }`。呼出し（call）を処理する経路（`tree.rs` の `get_app_info`、`node.rs`、`review.rs`、`perf.rs`、`updates.rs` の `name_hint`）は `i.ready_agent()`。`turn.rs` の `run_events` の `open_devtools()` は `if let Some(mut a) = i.agent() { .. }`（agent が無い間の key は何もしない）。
  6. `launch/inspect.rs`：
     ```rust
     #[cfg(feature = "devtools")]
     pub fn devtools_target(&self) -> Option<std::path::PathBuf> {
         let crate::devtools::state::DevtoolsState::Running(inspector) =
             crate::devtools::state::DevtoolsState::of(self.rt())
         else {
             return None;
         };
         inspector.promote(self.rt());
         inspector.agent().map(|agent| agent.discovery_path().to_owned())
     }

     /// Whether the DevTools agent is still starting.
     #[cfg(feature = "devtools")]
     pub fn devtools_starting(&self) -> bool {
         matches!(
             crate::devtools::state::DevtoolsState::of(self.rt()),
             crate::devtools::state::DevtoolsState::Running(i)
                 if matches!(&*i.agent.borrow(), crate::devtools::state::AgentSlot::Starting(_))
         )
     }
     ```
  7. `crates/kivact-test/src/screen/state.rs` の `devtools_target`：
     ```rust
     #[cfg(feature = "devtools")]
     pub fn devtools_target(&self) -> Option<std::path::PathBuf> {
         let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
         loop {
             if let Some(path) = self.inspect().devtools_target() {
                 return Some(path);
             }
             if !self.inspect().devtools_starting() || std::time::Instant::now() >= deadline {
                 return None;
             }
             std::thread::sleep(std::time::Duration::from_millis(5));
             self.run_until_idle();
         }
     }
     ```
     doc に「The agent starts in the background; this waits for it (5 s of real time at most).」を足す。
  8. App 全体の style override（RC-10）：node ごとに `node_overrides` の走査と文字列の解析をするのをやめ、App に一つの解析済みの表を持つ。`Style` は `PartialEq` を実装しないので、表は signal ではなく `Rc<RefCell<..>>` と変化の通知（`kivact_runtime::Notifier`。W5 で付属書 R が `Trigger` に改名し、その unit が追従する）で持つ。
     - `view/devtools_hooks.rs` に置く（view から見える型）：
       ```rust
       /// The DevTools style overrides, parsed once when they change: by node and
       /// by call site (file, line, column). The node overrides win.
       #[derive(Default)]
       pub(crate) struct NodeOverrides {
           pub(crate) by_node: std::collections::BTreeMap<NodeId, Style>,
           pub(crate) by_site: Vec<(String, u32, u32, Style)>,
       }
       ```
     - `devtools/state.rs` の `HookData` に `pub(crate) node_overrides: Rc<RefCell<NodeOverrides>>,` と `pub(crate) node_overrides_changed: kivact_runtime::Notifier,` を足す（`from_slot` で `Rc::default()` と `root.notifier(at)`）。
     - `devtools/styles.rs`：override の一覧（`i.hooks.overrides`）を書く全箇所の直後で `rebuild_node_overrides(i)` を呼ぶ。
       ```rust
       /// Parse the override list once into the table the style hooks read.
       fn rebuild_node_overrides(i: &Inspector) {
           let mut table = NodeOverrides::default();
           for entry in i.hooks.overrides.get_untracked() {
               let Ok(style) = super::apply::override_style(&entry.property, &entry.value) else {
                   continue;
               };
               match entry.scope {
                   p::OverrideScope::CallSite { location } => {
                       table.by_site.push((location.file, location.line, location.column, style));
                   }
                   p::OverrideScope::Node { node } => {
                       let id = NodeId(node);
                       let merged = match table.by_node.remove(&id) {
                           Some(before) => before.merge(&style),
                           None => style,
                       };
                       table.by_node.insert(id, merged);
                   }
               }
           }
           *i.hooks.node_overrides.borrow_mut() = table;
           i.hooks.node_overrides_changed.notify();
       }
       ```
       `p::OverrideScope` が上の 2 つ以外の variant を持つ場合（`rg -n "enum OverrideScope" -A12 crates/kivact-devtools-protocol/src` で見る）、旧 `node_overrides` の `_ => {}` と同じく何もしない arm `_ => {}` を足す。`NodeId(node)` の構築は旧コードの `*node == id.0` の比較に合わせる。
     - `view/devtools_hooks.rs` の `StyleHook` を次にする（C01 の `On` を置き換える）：
       ```rust
       #[derive(Clone)]
       pub(crate) enum StyleHook {
           Off,
           /// The node and its call sites; its overrides are looked up in the
           /// App's one table, and only when the table is not empty.
           Node {
               id: NodeId,
               sites: Rc<[&'static Location<'static>]>,
               table: Rc<std::cell::RefCell<NodeOverrides>>,
               changed: kivact_runtime::Notifier,
               forced: Value<ForcedStates>,
           },
       }

       impl StyleHook {
           /// The hook of node `id`.
           pub(crate) fn new(rt: &Rc<Runtime>, owner: Owner, id: NodeId, at: &'static Location<'static>) -> StyleHook {
               match hooks(rt) {
                   Some(h) => h.style_hook(rt, owner, id, at),
                   None => StyleHook::Off,
               }
           }

           /// Apply the node's overrides over `style`.
           pub(crate) fn apply(&self, style: Style) -> Style {
               let StyleHook::Node { id, sites, table, changed, .. } = self else {
                   return style;
               };
               changed.track();
               let table = table.borrow();
               if table.by_node.is_empty() && table.by_site.is_empty() {
                   return style;
               }
               let mut style = style;
               for (file, line, column, over) in &table.by_site {
                   if sites.iter().any(|s| s.file() == file && s.line() == *line && s.column() == *column) {
                       style = style.merge(over);
                   }
               }
               if let Some(over) = table.by_node.get(id) {
                   style = style.merge(over);
               }
               style
           }

           /// The states a tool forces on the node.
           pub(crate) fn forced(&self) -> ForcedStates {
               match self {
                   StyleHook::Off => ForcedStates::default(),
                   StyleHook::Node { forced, .. } => forced.get(),
               }
           }
       }
       ```
       （順序は旧 `node_overrides` と同じ：call site の override が先、node の override が後で勝つ。）
     - `devtools/hooks.rs` の `style_hook`：
       ```rust
       fn style_hook(&self, rt: &Rc<Runtime>, owner: Owner, id: NodeId, at: &'static Location<'static>) -> StyleHook {
           let forced = self.hooks.forced;
           StyleHook::Node {
               id,
               sites: Rc::from(super::apply::call_sites(rt, owner, Some(at))),
               table: self.hooks.node_overrides.clone(),
               changed: self.hooks.node_overrides_changed,
               forced: owner.with_setup(|| {
                   kivact_runtime::memo(move || forced.with(|all| all.get(&id).copied().unwrap_or_default()))
               }),
           }
       }
       ```
       node ごとの memo は `forced` の 1 つだけになり、override の解析は override の一覧が変わったときに一度だけになる。
     - `apply::node_overrides` と `same_location` を消す。`HookData.overrides: Signal<Vec<p::StyleOverride>>` は protocol の一覧（`list_overrides` の答え）として残す。
  9. 変更記録で木を通知する（RC-09）：`devtools/turn.rs` の `after_turn` を次にする：
     ```rust
     pub(crate) fn after_turn(rt: &Runtime) {
         let DevtoolsState::Running(i) = DevtoolsState::of(rt) else {
             return;
         };
         if i.agent().is_none() {
             return;
         }
         let changes = ViewState::tree(rt).reader().take_tree_changes();
         let closed = tree::notify_windows_changed(rt);
         sweep_closed_windows(rt, i, &closed);
         tree::notify_tree_changes(rt, &changes);
         prune_removed(i, &changes.removed);
         let records = ViewState::tree(rt).reader().take_engine_records();
         query::after_turn(rt, !changes.upserted.is_empty() || !changes.removed.is_empty());
         updates::after_turn(rt, &records);
         overlay::after_turn(rt, &records);
         console::after_turn(rt);
     }
     ```
     - `tree::notify_windows_changed(rt) -> Vec<WindowId>`：開いている window の一覧を前回（`sent_windows`）と比べ、閉じた window の id を返す（Windows の購読が無くても一覧の比較はする。比べる値は `reader().windows()` の `Vec<WindowId>`。`sent_windows` の型を `RefCell<Vec<WindowId>>` にし、購読があるときだけ `WindowsChanged` を送る）。
     - `sweep_closed_windows(rt, i, closed: &[WindowId])`：`closed` が空なら何もしない。`pending` の答えは `closed.contains(&window)` のものだけ。`frames`・`focus_history`・`sent_overlays` の剪定も `closed` の window だけ。`i.session.sent_tree.borrow_mut().retain(..)` を消す。
     - `tree::notify_tree_changes(rt, changes: &TreeChanges)`：購読（`p::Topic::Tree`）が無ければ何もしない。あれば `changes.upserted` の node だけ `tree_node(rt, id)` で `ElementInfo` を作り、`changes.removed` と合わせて window ごとに `TreeChanged` を送る（`current_nodes` と `sent_tree` の比較を消す）。`removed` の window は `ElementInfo` が無いので、E-REQ-5 の `TreeChanges.removed: Vec<(WindowId, NodeId)>` の window を使う。
     - `prune_removed(i, removed)`（`turn.rs` に置く）：`i.session.update_counts` と `i.hooks.forced`（`forced.update(|m| { for (_, id) in removed { m.remove(id); } })`、何も消えないときは書かない）から消えた node を除く。
     - `query::after_turn(rt, tree_changed: bool)`：`tree_changed` が偽のときは期限の判定だけをする（`search` を呼ばない）。真のときは今までどおり `search` する。
     - `SessionData.sent_tree` を消す。
  10. 誤りの通知を O(新しいもの) にする（RC-09）：`devtools/console.rs` の `after_turn` の `rt.reported()` を `rt.reported_since(since)`（R-REQ-9）にする。`errors_sent: RefCell<Sent<u64>>` を `errors_sent: Cell<u64>` にし、`get().copied().unwrap_or(0)` を `get()`、`update(last)` を `set(last)` にする。
  11. 入力の出所（RC-28）：`devtools/hooks.rs` の `input_event` の
     ```rust
     let injected = i.session.pending.borrow().iter().any(|pending| { matches!(pending, Pending::Inject { .. }) ... });
     let source = if injected { p::InputSource::Injected } else { p::InputSource::Platform };
     ```
     を、event が持つ出所（E-REQ-6）で決める：
     ```rust
     let source = match input_origin(event) {
         Some(kivact_engine::contract::InputOrigin::Injected(_)) => p::InputSource::Injected,
         _ => p::InputSource::Platform,
     };
     ```
     `fn input_origin(event: &HostEvent) -> Option<InputOrigin>` は `input_description` の隣に置き、`HostEvent::Pointer(route) | Wheel(route) | Key(route) | File(route)` の `route.origin` を返す（`Ime` は `None`）。
- **消すもの**：`AgentSlot::{Off, Running}`、`SessionData.sent_tree`、`current_nodes`、`apply::node_overrides`、`StyleHook::On`、`errors_sent` の `Sent`、`input_event` の `pending` の走査、`after_mount` の中の engine の key の設定（昇格へ移る）。
- **状態の閉じ方**：`AgentSlot` は enum。`Inspector` は既存の `HookData`・`SessionData`（DevTools の記録。field の多くは独立な記録の表）で、名前が `Data` で終わる。W1 の T の新しい `state` 検査で対象になる場合、`HookData` の上に `// xtask-allow(state): DevTools の記録の表の集まり。各表は別の購読・記録の設定が書き、互いの値から導かれない`、`SessionData` の上に `// xtask-allow(state): DevTools の session の表の集まり。待ちの呼出し・frame・overlay・snapshot は別々の要求が書く` を置く。`NodeOverrides` は値（`&mut self` の method なし）。
- **試験**：`e2e_native` の `devtools.rs` の `a_devtools_client_inspects_drives_and_reviews_a_test_app` が通る（agent が後から起動しても `devtools_target` が待つ）。足す試験は無い。
- **確認（tier a）**：
  ```sh
  cargo fmt -p kivact-core -p kivact-devtools-agent -p kivact-test
  cargo check -p kivact-devtools-agent --all-targets
  cargo check -p kivact-core --all-targets --features devtools
  cargo check -p kivact-test --all-targets --features devtools
  cargo clippy -p kivact-core -p kivact-devtools-agent -p kivact-test --all-targets --features kivact-core/devtools -- -D warnings
  cargo xtask check --no-write --crate kivact-core
  cargo xtask check --no-write --crate kivact-devtools-agent
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --features devtools --test e2e_native -- devtools
  ```
- **完了条件**：`rg -n "sent_tree|current_nodes|AgentSlot::Running|AgentSlot::Off" crates/kivact-core/src` が 0 件。`rg -n "Rc<str>" crates/kivact-devtools-agent/src` が 0 件。上のコマンドが通る。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C07：engine の事実を core に写して同期させるのをやめる

- **wave / lane / 前提**：W5a の鎖 / 鎖 / 付属書 R の R31（R の鎖の最後）、C05、付属書 E の E31（E-REQ-1〜E-REQ-4。W4b）
- **所有**：`crates/kivact-core/src/view/{tree.rs,text_edits.rs（削除）,state.rs,window_view.rs,node_ref.rs}`、`crates/kivact-core/src/client/{state.rs,dispatch/env.rs,dispatch/platform.rs,dispatch/delivery.rs}`、`crates/kivact-core/src/launch/{runtime.rs,turn.rs}`（turn の関数は R06 が `launch/turn.rs` に移してある）、`crates/kivact-core/src/custom/node.rs`、`crates/kivact-core/src/platform/env.rs`、`crates/kivact-core/src/devtools/{input.rs,query.rs}`（`is_visible` を読む 2 か所）
- **目的**：RC-20。engine が持つ事実（未配送の `TextChanged`、node の最後の layout 報告、node の可視と live region、font の版）を core が counter・側表・App 全体の notifier で写すのをやめる。asset の失敗の一度だけの報告を node と一緒に消える印にする。起動時と環境の変化ごとの font の列挙をやめる（PERF-10）。
- **手順**：
  1. 未配送の文字の編集（E-REQ-1）：engine が `Writer::set(id, PropValue::InputValue(..))` を未配送の `TextChanged` がある間は無視する。core の写しを消す：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-core/src" || exit 1
     git rm view/text_edits.rs
     ```
     `view.rs` の `pub(crate) mod text_edits;` を消す。`view/tree.rs` の `future_text_edits` field・`TextEditsInFlight`・`update` の `if prop == Prop::InputValue && self.future_text_edits ... { return false; }`・`remove` の `self.future_text_edits.borrow_mut().0.remove(id);`・`release_capacity` の `future_text_edits` の行を消す。turn の関数（`launch/turn.rs` の `Launch::turn`。R06 が移した）の `let edits = TextEditsAhead::begin(...);` と `edits.deliver(event);` と `use crate::view::text_edits::TextEditsAhead;` を消す。
  2. layout の種（E-REQ-2）：`view/tree.rs` の `LayoutSeed`、`ViewNodes.layout_seeds`、`note_layout`、`remove` の `self.layout_seeds.remove(id);` を消す。`observe_node` の `let (measured, text_truncated) = self.layout_seeds.remove(&id).unwrap_or_default();` を次にする：
     ```rust
     let (measured, text_truncated) = match self.reader().layout_report(id) {
         Some(report) => (
             report.layout_rect.map(|layout_rect| Measured { rect: report.rect, layout_rect }),
             report.text_truncated,
         ),
         None => (None, false),
     };
     ```
     `client/dispatch/env.rs` の `layout_changed` の `Some(None) => { ... tree.note_layout(..) ... None }` の分岐を `Some(None) => None,` にする（engine が報告を持つ）。
  3. 可視と live region（E-REQ-3）：
     - `view/tree.rs` の `NodeObservation` に `pub(crate) flags: Signal<kivact_engine::contract::NodeFlags>,` を足す。`observe_node` で `flags: node.owner.signal(self.reader().node_flags(id), at),` を作り、作った直後に `self.engine().watch_flags(id, true);` を呼ぶ。
     - `ViewNodes.visibility_revision`、`live_region_revision`、`visibility_revision()`、`live_region_revision()`、`notify_visibility`、`is_live_region`、`update` の `hides`・`was_hidden`・`tracks_live`・`was_live` の計算と 2 つの `notify_from_render` を消す。`is_visible(&self, id)` と `is_in_live_region(&self, id)`（DevTools と custom が使う、その場の祖先の走査）は `self.reader().node_flags(id).visible` と `.in_live_region` を返す 1 行にする。
     - `client/dispatch/delivery.rs` の event の `match` で、付属書 E の E31 が足した何もしない腕（`HostEvent::NodeFlagsChanged { .. }`）を `HostEvent::NodeFlagsChanged { node, flags } => env::node_flags_changed(rt, *node, *flags),` にする（E31 が別の core の file に腕を置いていれば、その腕をこの形にする）。`client/dispatch/env.rs`：
       ```rust
       /// The engine reports a new visibility or live-region fact of a watched node.
       pub(super) fn node_flags_changed(rt: &Runtime, node: NodeId, flags: kivact_engine::contract::NodeFlags) {
           let observation = ViewState::tree(rt).get(node).and_then(crate::view::tree::CoreNode::observation);
           if let Some(observation) = observation {
               rt.enter(|| observation.flags.set(flags));
           }
       }
       ```
     - `custom/node.rs` の `is_visible` と `is_in_live_region`：
       ```rust
       #[track_caller]
       fn is_visible(&self) -> bool {
           self.node_flags(Location::caller()).is_some_and(|flags| flags.visible)
       }
       #[track_caller]
       fn is_in_live_region(&self) -> bool {
           self.node_flags(Location::caller()).is_some_and(|flags| flags.in_live_region)
       }
       ```
       同じ `impl` に private の helper（`NodeRefExt` の trait ではなく `NodeRef` の inherent `pub(crate)` method として `view/node_ref.rs` に置く）：
       ```rust
       /// The engine's visibility facts of the bound node (tracked).
       pub(crate) fn node_flags(&self, at: &'static Location<'static>) -> Option<kivact_engine::contract::NodeFlags> {
           self.require_observation(RefObservation::Sources, at);
           let target = self.target_value(at)?;
           observation(self.app, target).map(|o| o.flags.get())
       }
       ```
       `target_value(&self, at) -> Option<FocusTarget>` は `view/node_ref.rs` の既存の `pub(crate)` method（`custom/node.rs` の `tracked_node` が使うもの）。`RefObservation::Sources` は `view/ref_data.rs` の既存の値。`observe_node` が memo の中で signal を作ることが runtime に拒まれる（`WriteInPureContext` などの panic）場合は STOP する。
  4. asset の失敗（CoreRare の印）：
     - `view/tree.rs` の `CoreRare` に `pub(crate) asset_failures: kivact_types::cell::ReportOnceSet<String>,` を足す。`ViewNodes` に：
       ```rust
       /// Whether the failure `message` of `id`'s asset is new for this node
       /// (reported once per node; forgotten with the node).
       pub(crate) fn note_asset_failure(&mut self, id: NodeId, message: &str) -> bool {
           match self.nodes.0.get_mut(&id) {
               Some(node) => node.rare_mut().asset_failures.note(message.to_owned()),
               None => false,
           }
       }
       ```
     - `client/dispatch/platform.rs` の `asset_failed` の `let first = HostState::of(rt).asset_failures.borrow_mut().insert((node, message.to_owned()));` を `let first = ViewState::tree_mut(rt).note_asset_failure(node, message);` にし、`memory_warning` の `HostState::of(rt).asset_failures.borrow_mut().shrink_to_fit();` の文を消す。
     - `client/state.rs` の `asset_failures` field を消す。`view/state.rs` の `ViewState.removed` と `impl super::tree::Removed { fn commit(..) }` を消す。`Removed` の `ids` field を消し、`Removed::commit(rt)` を呼んでいた箇所（`rg -n "\.commit\(rt\)|\.commit\(&rt\)" crates/kivact-core/src/view`）を `drop(removed);`（tree の borrow を離した後）にする。turn の `commit` 関数の `let removed = std::mem::take(&mut *ViewState::of(rt).removed.borrow_mut()); if !removed.is_empty() { host.asset_failures... }` を消す。
     - `CoreRare` は既に `Option<Box<..>>` の中なので、`CoreNode` の大きさの assert は変わらない。
  5. font の一覧を使う時に作る（E-REQ-4）：
     - `view/state.rs` の `ViewState.fonts_value: Signal<Vec<String>>` を `pub(crate) fonts: kivact_types::cell::Deferred<FontsValue>,` にし、次を足す：
       ```rust
       // xtask-allow(state): generation は families を最後に読んだ font の版。families は generation の射影で、refresh_fonts だけが両方を同じ遷移で書く
       pub(crate) struct FontsValue {
           pub(crate) families: Signal<Vec<String>>,
           generation: std::cell::Cell<u64>,
       }

       impl ViewState {
           /// The families of the App's fonts, read from the engine on first use.
           pub(crate) fn font_families(&self, rt: &Runtime, at: &'static Location<'static>) -> Value<Vec<String>> {
               let fonts = self.fonts.get_or_init(|| {
                   let reader = self.tree.borrow().reader();
                   FontsValue {
                       families: rt.root().signal(reader.font_families(), at),
                       generation: std::cell::Cell::new(reader.font_generation()),
                   }
               });
               fonts.families.into()
           }

           /// Write the families again when the engine's font database changed
           /// since they were read. Nothing happens before the first read.
           pub(crate) fn refresh_fonts(&self, rt: &Runtime) {
               let Some(fonts) = self.fonts.get() else { return };
               let reader = self.tree.borrow().reader();
               let generation = reader.font_generation();
               if generation != fonts.generation.get() {
                   fonts.generation.set(generation);
                   let families = reader.font_families();
                   rt.enter(|| fonts.families.set(families));
               }
           }
       }
       ```
       `ViewState::new` の `fonts_value: owner.signal(Vec::new(), at),` を `fonts: kivact_types::cell::Deferred::new(),` にする。
     - `launch/runtime.rs` の `start_mount` の 2 行（`let families = ViewState::tree(rt).reader().font_families();` と `rt.enter(|| ViewState::of(rt).fonts_value.set(families));`）を消す。
     - `client/dispatch/env.rs` の `env_changed` の `let families = ...; ViewState::of(rt).fonts_value.set(families);` の 2 行を消し、`rt.enter(..)` の後に `ViewState::of(rt).refresh_fonts(rt);` を足す。`client/dispatch/platform.rs` の `effect_result` の `requests::resolve_effect(..)` の後にも `ViewState::of(rt).refresh_fonts(rt);` を足す（font の登録の答えで版が変わる）。
     - `platform/env.rs` の `font_families()`：
       ```rust
       #[track_caller]
       pub fn font_families() -> Value<Vec<String>> {
           let at = Location::caller();
           let rt = Runtime::current().expect("invariant: font_families requires a running App");
           ViewState::of(&rt).font_families(&rt, at)
       }
       ```
       doc の「This value follows changes to the font database.」は残す。
  6. `kivact_runtime::Notifier::notify_from_render` の core の呼出しがこれで無くなる（`rg -n notify_from_render crates/kivact-core` が `view/tree.rs` の `scene_revision` と `mount/style.rs` の `source.revision` だけになる）。この 2 つは残る（RC-20 の対象ではない）。付属書 R は method を残す（4.1 節）。
- **消すもの**：`view/text_edits.rs`（`TextEditsAhead`）、`TextEditsInFlight`、`future_text_edits`、`LayoutSeed`、`layout_seeds`、`note_layout`、`visibility_revision`、`live_region_revision`、`notify_visibility`、`ViewState.removed`、`Removed.ids`、`Removed::commit`、`HostState.asset_failures`、`ViewState.fonts_value`、起動時の `font_families()`。
- **状態の閉じ方**：`FontsValue` に上の印 1 行（Signal と Cell の 2 つ）。`NodeObservation` は signal を既に複数持つ（C05 の時点で T の新しい検査の印があればそのまま）。`flags` は engine の報告の写しではなく engine が書く唯一の通知先（engine が事実の持ち主）なので、`NodeObservation` の印の文面に「flags は engine の NodeFlagsChanged だけが書く」を足す。印が無い場合は `NodeObservation` の上に `// xtask-allow(state): 各 signal は engine の別々の報告（layout、truncation、scroll、image、flags）だけが書き、互いに導かれない` を置く。
- **試験**：`e2e_core` の `text_editing`（`ime_composition_commits_cancels_swallows_keys_and_ends_on_blur` を含む）、`custom_text_field`、`layout`、`images_animated`、`assets`、`launch_fonts`、`e2e_ui` の `interaction_toast`（live region）と roving focus を使う `apg_toolbar`・`apg_radio_group` が通ること。足す試験は無い。
- **確認（tier a）**：
  ```sh
  cargo fmt -p kivact-core
  cargo check -p kivact-core --all-targets --features devtools,embed,gpu-interop
  cargo clippy -p kivact-core --all-targets --features devtools,embed,gpu-interop -- -D warnings
  cargo check -p kivact-test -p kivact-interaction -p kivact-ui --all-targets
  cargo xtask check --no-write --crate kivact-core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- text_editing custom_text_field layout images_animated assets launch_fonts
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- interaction_toast apg_toolbar apg_radio_group
  ```
- **完了条件**：`rg -n "TextEditsAhead|TextEditsInFlight|layout_seeds|note_layout|visibility_revision|live_region_revision|asset_failures|fonts_value" crates/kivact-core` が `CoreRare.asset_failures` と `note_asset_failure` の 2 行を除いて 0 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C08：`use_direction` は値、`Owner` は `custom` だけ

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C07（付属書 R の R20 が `use_context::<T>() -> T`・`try_use_context::<T>() -> Option<T>` に、R26 が core の `provide` を `T` をそのまま置く形にし、R26〜R31 の掃引が呼出し元の `.get()` を消してある。4.1 節 R-REQ-12）
- **所有**：`crates/kivact-core/src/{state.rs,custom.rs,view/boundary.rs,view/owner.rs,view/root.rs（1 行）,view/element_data.rs,view/overlays.rs}`、`use_direction`・`kivact::state::Owner` の全呼出し元（`crates/`、`dev/`、`examples/`、`templates/`）（呼出し元の `dev/` は `dev/tests/kivact-public-api`・`dev/tests/kivact-kit-tests`・`dev/tests/gpu-interop-producer`・`dev/bench` を含む。W5 では kit・catalog・`kivact-kit-tests` も compile するので直す。INC-33）
- **目的**：RC-18：`use_direction() -> ReadingDirection`、root が `Ltr` を provide し、呼ぶたびに constant を作る fallback を消す。API-10：`Owner` の path を `kivact::custom` だけにする。`Owner::store`・`store_with` を `#[doc(hidden)]`（RC-31 の core の半分の残り。context の置き場の統一（API-04）は付属書 R が済ませた）。
- **手順**：
  1. RC-18：`crates/kivact-core/src/view/boundary.rs`
     ```rust
     /// Build `child` with `dir` as the reading direction of its subtree.
     #[track_caller]
     pub fn provide_direction<V: IntoView>(dir: ReadingDirection, child: impl FnOnce() -> V + 'static) -> View {
         provide(dir, child)
     }

     /// The reading direction provided above the current owner (the App's root
     /// provides `Ltr`). Setup, handlers and effects.
     #[track_caller]
     pub fn use_direction() -> ReadingDirection {
         let owner = Owner::running("use_direction", Location::caller());
         owner
             .context::<ReadingDirection>()
             .expect("invariant: the App's root provides the reading direction")
     }
     ```
     （`Owner::running` は付属書 R の R25 が `Owner::ambient` から改名した runtime の関数。`provide_direction` は `ReadingDirection` の定数を受けるまま（`impl IntoValue` にしない。12 章 API-16）。）`provide` は R26 の後 `T` をそのまま置くので、`provide(dir, ..)` の `ReadingDirection` は `owner.context::<ReadingDirection>()` で読める。
     `view/root.rs` の `install_app_contexts` の `crate::view::boundary::install_locale(owner, locale);` の次の行に `owner.provide(crate::view::ReadingDirection::Ltr);` を足す（`view/root.rs` はこの 1 行だけ編集してよい）。
  2. 呼出し元の掃引（use_direction）：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     rg -l "use_direction\(\)\.get\(\)" crates dev examples templates | xargs perl -pi -e 's/use_direction\(\)\.get\(\)/use_direction()/g'
     ```
     残りの compile error は次のとおり機械的に直す：`let x = use_direction();` の後の `x.get()` → `x`、`x.get_untracked()` → `x`、`Value<ReadingDirection>` 型の field・引数 → `ReadingDirection`、`move || x.get() == ReadingDirection::Rtl` のような closure はそのまま（`x` は `Copy`）。`crate::view::use_direction().get()` も perl で直る。
  3. `crates/kivact-core/src/state.rs`：`pub use crate::view::owner::{Owner, use_app_state};` を `pub use crate::view::owner::use_app_state;` にする。`rg -n "state::Owner|state::\{[^}]*\bOwner\b" crates dev examples templates` の箇所を `custom::Owner` に直す。
  4. `crates/kivact-core/src/view/owner.rs` の `Owner::store` と `Owner::store_with` の doc の上に `#[doc(hidden)]` と `// xtask-allow(internal-contract): custom components store their objects under their owner` を付ける（`rg -n "internal-contract" dev/xtask/src` が 0 件なら印は付けず `#[doc(hidden)]` だけにする。1 件以上なら印も付ける）。core の 0 引数の `Owner::current()` は改名しない（付属書 R の 4.1 節）。
- **消すもの**：`use_direction` の `owner.constant` の fallback、`use_direction` の `Value` の包み、`kivact::state::Owner`。
- **状態の閉じ方**：新しい状態の型は無い。
- **試験**：足す：`e2e_core` の `setup_functions.rs` に `use_direction_is_ltr_at_the_root_and_follows_provide_direction`（root で `use_direction() == ReadingDirection::Ltr`、`provide_direction(ReadingDirection::Rtl, || ..)` の中で `Rtl`、その中の button の `on_press` の handler でも `Rtl`）。`use_context` の試験の書換えは付属書 R の R31。
- **確認（tier a）**：W5a の鎖の共通の確認（4.6 節の `W5_CHECK`）と：
  ```sh
  cargo xtask check --no-write --crate kivact-core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- setup_functions
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- apg_toolbar apg_slider apg_tabs apg_radio_group
  ```
- **完了条件**：`rg -n "use_direction\(\)\.get" crates dev examples templates` が 0 件。`rg -n "fn use_direction\(\) -> Value" crates` が 0 件。`rg -n "pub use crate::view::owner::\{Owner" crates/kivact-core` が 0 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C09：`#[component]` を 5.1 節の契約どおりに作り直す（`impl Element`、`Node::absorb`、path の解決、`Option`、`Callback` は `Handler`、`#[derive(Prop)]`）

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C08（付属書 R の R26・R27 が `kivact-macros` を先に直してある。手順 0）
- **所有**：`crates/kivact-macros/`（全ファイル）、`crates/kivact-core/src/{custom.rs,custom/measured_leaf.rs → view/measured_leaf.rs,custom/measurement.rs}`、`Cargo.toml`（workspace の `[workspace.dependencies]` に 1 行）、`crates/kivact-ui/Cargo.toml`・`crates/kivact-interaction/Cargo.toml`（`kivact-macros` の依存）、`dev/xtask/policy/workspace.toml`（`kivact-macros` の `visible_to` の 1 行）、`constant_value!` の呼出し元（kivact-interaction、kivact-ui、5 つの kit、`examples/`、`dev/`）、`crates/kivact-core/src/view/{boundary.rs,elements/builders.rs,element_data.rs}`、`crates/kivact-core/src/{view.rs,custom.rs,lib.rs}`、`crates/kivact/src/lib.rs`、`#[component]` の利用者（`examples/kivact/component`、`templates/basic`、`dev/tests/kivact-public-api/src/lib.rs`、`dev/tests/kivact-public-api/tests/*/component_macro.rs`）、`scope(` の利用者（`crates/kivact-ui-soft`、`examples/kivact-ui-soft`、`dev/tests`、`dev/bench`）（呼出し元の `dev/` は `dev/tests/kivact-public-api`・`dev/tests/kivact-kit-tests`・`dev/tests/gpu-interop-producer`・`dev/bench` を含む。W5 では kit・catalog・`kivact-kit-tests` も compile するので直す。INC-33）
- **目的**：API-01（部品の書き方を一つに）、API-18（props の種類を字面で決め、実在を確かめる）、RC-33（`Callback<E>` の props は `Handler<E>` で持つ）、API-06（`#[derive(Prop)]`）、5.1 規則 9（型引数を持つ部品）。`scope` を `kivact::custom` へ（5.1 規則 8）。API-07 の core の doc の誤り（`lib.rs` の「or by `Type::new`」）。
- **手順**：
  0. **付属書 R が済ませてあること**（W5a の鎖で C09 より前の R26・R27。この unit はその状態の上に書く）：
     - `parse.rs`：`PropKind::Binding` は `PropKind::Signal(Type)`（`"Signal"` の字面）になっている（R26）。
     - `expand.rs`（R26）：`Callback<()>` の props は `impl ::kivact::state::IntoHandler<(), M<Name>>`（`fn m_ident` が作る `M` + UpperCamel の generic）で受け、`::kivact::custom::Handler<()>` で持つ。`Callback<E>`（E が `()` でない）の props は `impl ::std::ops::Fn(E) -> R<Name> + 'static`（`fn r_ident` の `R` + UpperCamel の generic、`R<Name>: ::kivact::view::HandlerResult`）で受け、`::kivact::custom::Handler::from_fn(#input)` で `::kivact::custom::Handler<E>` として持つ（設計書 DEC-16：payload のある handler は closure の引数の型推論のため `Fn(E)`）。scope の中で `::kivact::state::Callback::from_handler` で `Callback<E>` にする。
     - `expand.rs`（R27）：`Value<String>` は `::kivact::custom::Prop::from_text(#input)`、`Value<T>` は `::kivact::custom::Prop::new(#input)` で `::kivact::custom::Prop<..>` として持つ（`custom::Reactive` は消えている）。
     - `kivact::custom::{Handler, Prop}`（hidden）、`kivact::state::IntoHandler`、`Callback::from_handler` がある（R26・R27）。
     - R が**していない**こと（C09 がする）：path の解決（`::kivact::` の直書きのまま）、`-> impl Element` と `absorb`、`Option` だけの任意（`#[prop(optional)]` はまだ有効）、型引数（5.1 規則 9）、生成の generic の予約名、API-18 の確認、`#[derive(Prop)]` と `constant_value!` の置換、`scope` の移動。下の手順はこの状態から書き換える（R26 の `m_ident`・`r_ident`・`v_ident` は手順 4 の名前の規則に置き換わる）。
  1. 依存：`Cargo.toml`（workspace）の `[workspace.dependencies]` に `proc-macro-crate = "3.5.0"` を足す（`Cargo.lock` に 3.5.0 がある）。`crates/kivact-macros/Cargo.toml` の `[dependencies]` に `proc-macro-crate = { workspace = true }` を足す。
  2. 新 `crates/kivact-macros/src/root.rs`：
     ```rust
     //! Role: Find the path the generated code names Kivact by.

     use proc_macro_crate::{FoundCrate, crate_name};
     use proc_macro2::{Span, TokenStream};
     use quote::quote;

     /// `::kivact_core` in a crate that depends on kivact-core (kivact-ui and
     /// kivact-interaction), else `::kivact` (apps and kits); an error naming
     /// the missing dependency otherwise. kivact-core is checked first: those
     /// two crates also have `kivact` as a dev-dependency, which proc-macro-crate
     /// does not tell apart, and no other crate may depend on kivact-core
     /// (`visible_to`).
     pub(crate) fn root() -> syn::Result<TokenStream> {
         for name in ["kivact-core", "kivact"] {
             match crate_name(name) {
                 Ok(FoundCrate::Name(found)) => {
                     let ident = syn::Ident::new(&found, Span::call_site());
                     return Ok(quote! { ::#ident });
                 }
                 Ok(FoundCrate::Itself) => {
                     let ident = syn::Ident::new(&name.replace('-', "_"), Span::call_site());
                     return Ok(quote! { ::#ident });
                 }
                 Err(_) => {}
             }
         }
         Err(syn::Error::new(
             Span::call_site(),
             "#[component] needs the `kivact` crate as a dependency",
         ))
     }
     ```
     （5.1 規則 6 のとおり kivact-core を先に調べる。）
  3. `parse.rs` を次の model にする（既存の関数を直す。`upper_camel`・`check_signature` の大部分・`type_arguments`・`is_named`・`is_unit` は残す）：
     ```rust
     pub(crate) struct Model {
         pub(crate) vis: Visibility,
         pub(crate) name: Ident,
         pub(crate) struct_name: Ident,
         pub(crate) docs: Vec<Attribute>,
         pub(crate) attrs: Vec<Attribute>,
         pub(crate) deprecated: Vec<Attribute>,
         pub(crate) props: Vec<Prop>,
         pub(crate) body: Block,
         /// `-> impl Element` (stylable) or `-> impl IntoView`.
         pub(crate) returns: Returns,
         /// The function's type parameters and where-clause (5.1 rule 9), each
         /// type parameter with `'static` added to its bounds.
         pub(crate) generics: syn::Generics,
         /// The type parameters no prop's type names, held by `__phantom`.
         pub(crate) phantom: Vec<Ident>,
     }

     pub(crate) enum Returns {
         Element,
         IntoView,
     }

     pub(crate) struct Prop {
         pub(crate) name: Ident,
         pub(crate) kind: PropKind,
         pub(crate) required: Required,
         pub(crate) mutable: bool,
         /// The type as written (for the existence check of API-18).
         pub(crate) written: Type,
     }

     pub(crate) enum Required {
         /// A positional argument of the builder function.
         Positional,
         /// `Option<..>`: a setter, absent by default.
         Optional,
         /// `#[prop(default = expr)]`: a setter with a default.
         Default(Expr),
     }

     pub(crate) enum PropKind {
         /// `Value<String>`: text, takes `IntoText`, kept as `Prop<String>`.
         Text,
         /// `Value<T>`: takes `IntoValue<T>`, kept as `Prop<T>`.
         Value(Type),
         /// `Signal<T>`: takes exactly a `Signal<T>`.
         Signal(Type),
         /// `Callback<()>`: takes `IntoHandler<(), M>` (a closure or a `Callback<()>`), kept as `Handler<()>`.
         Callback0,
         /// `Callback<E>` (E is not `()`): takes `Fn(E) -> R` (DEC-16), kept as `Handler<E>`.
         Callback(Type),
         /// `View`: takes `IntoView`.
         View,
         /// `Vec<View>`: takes `IntoChildren`.
         Children,
         /// `ViewFn`: takes `IntoViewFn<V>`, kept as `ViewFn<()>`.
         ViewFn,
         /// A type parameter of the component itself (`control: C`): takes `C`
         /// as it is (rule 3: wrapping it in `Into` stops inference).
         Param(Type),
         /// Any other type `T`: takes `impl Into<T>`.
         Other(Type),
     }
     ```
     - `returns_into_view` を `fn returns(output: &ReturnType) -> Option<Returns>` にする：`impl` の bound の最後の segment が `Element` なら `Returns::Element`、`IntoView` なら `Returns::IntoView`、それ以外は `None`（error：`"a #[component] function returns `impl Element` or `impl IntoView`"`）。
     - `required(attrs, ty)`：`optional` を受けない。`#[prop(..)]` は `default = expr` だけを受け、他は error `"#[prop] accepts only `default = <expr>`; an optional prop is written `Option<T>`"`。型が `Option<X>`（最後の segment が `Option`）なら `(Required::Optional, X)`。`Option<X>` に `default` を付けたら error `"an `Option<..>` prop is optional already; remove `#[prop(default = ..)]`"`。それ以外は `default` があれば `Default`、無ければ `Positional`。
     - `kind(ty, params)`（`params` は部品の型引数の ident の一覧。`kind` を呼ぶ前に手順の「型引数」の `generics` を作る）：型が `Type::Path` で、`qself` が無く、segment が 1 つで型引数（`<..>`）を持たず、その ident が `params` のどれかと等しければ `Param(ty)`（設計書 5.1 規則 3、12 章 API-1。`Option<C>` も `Required::Optional` の `Param`）。それ以外は最後の segment の字面で決める：`"Value"`（中が字面 `String` なら `Text`）、`"Signal"`、`"Callback"`、`"View"`、`"Vec"`（中が字面 `View`）、`"ViewFn"`、それ以外は `Other`。`"Callback"` は中が `()` なら `Callback0`、それ以外は `Callback(inner)`（R26 の形のまま）。型別名は `Other`（字面で決めるため自動でそうなる）。
     - `Prop.written` には `Option` を外す前の内側の型（`Option<Value<String>>` なら `Value<String>`）を入れる。
     - 型引数（5.1 規則 9）：`check_signature` の `if !sig.generics.params.is_empty() || sig.generics.where_clause.is_some() { return Err(..) }` を消し、次の検査にする（新しい関数 `fn generics(sig: &syn::Signature) -> syn::Result<syn::Generics>`）：
       - `GenericParam::Lifetime` があれば error `"a #[component] takes type parameters only; remove the lifetime parameter"`（span はその引数）。
       - `GenericParam::Const` があれば error `"a #[component] takes type parameters only; remove the const parameter"`。
       - 型引数の名前が予約の接頭辞 `__Kv` で始まれば error `"type parameter names starting with `__Kv` are reserved for #[component]"`（4.8 節）。
       - prop の名前が `__` で始まれば error `"prop names starting with `__` are reserved for #[component]"`（生成する field と局所変数 `__root`・`__phantom`・`__body`・`__default`・`__n` のため）。
       - 各 `GenericParam::Type` の bounds に `'static` を足した `Generics` を返す（`tp.bounds.push(syn::parse_quote!('static))`。既に `'static` があっても足す。重複は rustc が受ける）。where-clause はそのまま持つ。
     - 引数の属性：`#[prop(default = ..)]` 以外の属性を持つ引数は error `"a #[component] prop takes only #[prop(default = ..)]"`（span はその属性）。設計書 5.1 規則 10 の `#[caller] caller: Caller` は付属書 U の U25 がこの検査に `caller` を足して受ける（C09 の時点では error でよい。U25 の手順が parser を広げる）。
     - `phantom`：各型引数の ident が、どの prop の `written`（`Option` を外す前の型も含め、prop の型全体）の token にも現れなければ `phantom` に入れる。判定は `quote!(#ty).into_iter()` を再帰で辿り（`proc_macro2::TokenTree::Group` の中も）、`TokenTree::Ident` が型引数の ident と等しいものがあるか、で行う。
  4. `expand.rs` を全部書き直す。生成するものの形（`root` は手順 2 の path。以下 `R` と書く）：
     ```rust
     // props: title: Value<String>, children: Vec<View>, on_close: Option<Callback<()>>, #[prop(default = 1)] level: u8 ; returns impl Element
     #[must_use]
     pub struct Card {
         title: R::custom::Prop<::std::string::String>,
         children: ::std::vec::Vec<R::view::View>,
         on_close: ::std::option::Option<R::custom::Handler<()>>,
         level: u8,
         __root: R::view::Node,          // returns impl Element のときだけ
     }

     #[track_caller]
     pub fn card(title: impl R::state::IntoText, children: impl R::view::IntoChildren) -> Card {
         Card {
             title: R::custom::Prop::from_text(title),
             children: R::view::IntoChildren::into_views(children),
             on_close: ::std::option::Option::None,
             level: { let __default = 1; ::std::convert::Into::into(__default) },
             __root: R::view::node(),
         }
     }

     impl Card {
         #[track_caller]
         pub fn on_close<__KvMOnClose>(mut self, on_close: impl R::state::IntoHandler<(), __KvMOnClose>) -> Self {
             self.on_close = ::std::option::Option::Some(R::state::IntoHandler::into_handler(on_close));
             self
         }
         #[track_caller]
         pub fn level(mut self, level: impl ::std::convert::Into<u8>) -> Self {
             self.level = ::std::convert::Into::into(level);
             self
         }
     }

     impl R::view::Element for Card {
         fn node_mut(&mut self) -> &mut R::view::Node {
             &mut self.__root
         }
     }

     impl R::view::IntoView for Card {
         fn into_view(self) -> R::view::View {
             let Card { title, children, on_close, level, __root } = self;
             R::custom::scope(move || {
                 let title: R::state::Value<::std::string::String> = R::state::IntoValue::into_value(title);
                 let on_close: ::std::option::Option<R::state::Callback<()>> = on_close.map(R::state::Callback::from_handler);
                 let __body = (move || { /* the function's statements */ })();
                 R::view::Element::with_node(__body, move |__n| __n.absorb(__root))
             })
         }
     }

     // API-18: each recognised prop names the Kivact type it stands for.
     // A non-capturing closure coerces to this fn pointer only when both sides are
     // the same types, so a foreign `Value` or a missing import is a type error here.
     const _: fn(
         ::std::marker::PhantomData<(Value<String>, Vec<View>, Callback<()>)>,
     ) -> ::std::marker::PhantomData<(R::state::Value<::std::string::String>, ::std::vec::Vec<R::view::View>, R::state::Callback<()>)> = |written| written;
     ```
     規則（kind ごと）：

     | kind | 位置引数・setter の型 | 保存の型 | 保存の式 | scope の中の変換 |
     |---|---|---|---|---|
     | Text | `impl R::state::IntoText` | `R::custom::Prop<String>` | `R::custom::Prop::from_text(x)` | `let x: R::state::Value<String> = R::state::IntoValue::into_value(x);` |
     | Value(T) | `impl R::state::IntoValue<T>` | `R::custom::Prop<T>` | `R::custom::Prop::new(x)` | `let x: R::state::Value<T> = R::state::IntoValue::into_value(x);` |
     | Signal(T) | `R::state::Signal<T>` | 同じ | `x` | なし |
     | Callback0 | `impl R::state::IntoHandler<(), M_x>`（`M_x` は prop ごとの generic。名前は `__KvM` + `upper_camel(name)`。4.8 節） | `R::custom::Handler<()>` | `R::state::IntoHandler::into_handler(x)` | `let x: R::state::Callback<()> = R::state::Callback::from_handler(x);` |
     | Callback(E) | `impl ::std::ops::Fn(E) -> R_x + 'static`（`R_x: R::view::HandlerResult` は prop ごとの generic。名前は `__KvR` + `upper_camel(name)`。DEC-16。`Callback<E>` の handle をそのまま渡すことはできず、呼び出し側は `move \|e\| cb.call(e)` と書く） | `R::custom::Handler<E>` | `R::custom::Handler::from_fn(x)` | `let x: R::state::Callback<E> = R::state::Callback::from_handler(x);` |
     | View | `impl R::view::IntoView` | `R::view::View` | `R::view::IntoView::into_view(x)` | なし |
     | Children | `impl R::view::IntoChildren` | `Vec<R::view::View>` | `R::view::IntoChildren::into_views(x)` | なし |
     | ViewFn | `impl R::view::IntoViewFn<V_x>`（`V_x: R::view::IntoView` の generic。名前は `__KvV` + `upper_camel(name)`。4.8 節） | `R::view::ViewFn<()>` | `R::view::IntoViewFn::into_view_fn(x)` | なし |
     | Param(C) | `C`（部品の型引数そのもの。`impl Into<C>` で包まない：包むと `field("Name", text_input(v))` の `C` が推論できない。scratchpad の rustc の試し（`p1.rs`・`p2.rs`）で、`control: C` は注釈なしで推論され、`control: impl Into<C>` は E0283 になることを確かめた） | `C` | `x` | なし |
     | Other(T) | `impl ::std::convert::Into<T>` | `T` | `::std::convert::Into::into(x)` | なし |

     - Optional（`Option<X>`）：field の型は `Option<保存の型>`、constructor では `None`、setter は上の入力型を取り `Some(保存の式)` を入れる、scope の中の変換は `x.map(|x| 変換の式)`（変換が無い kind は何もしない）。
     - Default(expr)：constructor で `{ let __default = expr; 保存の式(__default) }`。ただし Callback の default は、closure の引数の型を推論させるため `Handler::from_fn` の `Fn` の bound を直接通す（入れ子の `fn` は外の型引数を使えないので使わない）：Callback0 は `{ let __default = expr; R::custom::Handler::<()>::from_fn(move |()| __default()) }`、Callback(E) は `R::custom::Handler::<E>::from_fn(expr)`（R26 と同じ式）。
     - 型引数（5.1 規則 9）：`Model.generics` を `split_for_impl()` で分け、
       - struct：`pub struct Card #impl_generics #where_clause { ..fields.., __phantom: ::std::marker::PhantomData<fn() -> (#(#phantom,)*)> }`（`__phantom` は `phantom` が空でないときだけ。field の宣言と初期化 `__phantom: ::std::marker::PhantomData` と、`into_view` の分解 `__phantom: _` も同じ条件）。
       - constructor：`pub fn card<#user_params, #generated_params>(..) -> Card #ty_generics #where_clause`。利用者の型引数（bound 付き）が先、生成する `__KvM*`・`__KvR*`・`__KvV*` が後。
       - setter の `impl`、`impl Element`、`impl IntoView`：`impl #impl_generics Card #ty_generics #where_clause`。
       - 例：
         ```rust
         // #[component] pub fn choice<K: Clone + PartialEq>(items: Value<Vec<K>>, value: Signal<Option<K>>, on_pick: Option<Callback<K>>) -> impl Element
         pub struct Choice<K: Clone + PartialEq + 'static> {
             items: R::custom::Prop<Vec<K>>,
             value: R::state::Signal<Option<K>>,
             on_pick: ::std::option::Option<R::custom::Handler<K>>,
             __root: R::view::Node,
         }
         #[track_caller]
         pub fn choice<K: Clone + PartialEq + 'static>(items: impl R::state::IntoValue<Vec<K>>, value: R::state::Signal<Option<K>>) -> Choice<K> { .. }
         impl<K: Clone + PartialEq + 'static> Choice<K> {
             #[track_caller]
             pub fn on_pick<__KvROnPick: R::view::HandlerResult>(mut self, on_pick: impl ::std::ops::Fn(K) -> __KvROnPick + 'static) -> Self { .. }
         }
         ```
     - `Returns::IntoView` のとき：`__root` field、`node()` の初期化、`impl Element`、`absorb` を出さない。scope の最後は `__body`。
     - `Callback0`・`Callback(E)` の props の `Callback::from_handler` は、scope の中で本体の statements の**前に**一度だけ呼ぶ（その部品の owner に一度だけ登録する。設計書 5.1 規則 7、12 章 API-15。子に渡すときまで遅らせることはしない）。上の生成の形の `let on_close = on_close.map(R::state::Callback::from_handler);` がそれで、`__body` の closure の前に置く。
     - 呼出しの結果は保つ（12 章 API-14）：`Callback0` の保存は `IntoHandler::into_handler` の `Handler<()>`、`Callback(E)` の保存は `Handler::from_fn(f)`（`HandlerResult::into_outcome` で `Result<Handled, UiError>` にする）。本体に渡す `Callback<E>` は `Callback::from_handler` で同じ `Handler` を包み、本体が結果の要る所（子の `on_key` など）へ渡すときは `move |e| cb.invoke(e)`（付属書 R の `#[doc(hidden)] Callback::invoke(&self, e) -> Result<Handled, UiError>`）で呼ぶので、inline の closure と同じ `Handled` が返る。`Callback<()>` の props を子の `on_press` などへ渡すときは `IntoHandler<(), Handle> for Callback<()>`（R が `invoke` で実装）が結果を保つ。公開の `Callback::call(e)` は `()` を返す（利用者が手で転送する `move |e| cb.call(e)` は結果を捨てる。設計どおり）。4.1 節 R-REQ-15。
     - `mut` の prop は今までどおり scope の中で `#[allow(unused_mut)] let mut x = x;` を変換の後に出す。
     - 関数の statements は `(move || { #(#statements)* })()` の closure の中に置く（本体の `return` がこの closure から返るため）。
     - API-18 の確認：認識した kind（Other と Param 以外。Param は利用者の型引数そのもので、確かめる Kivact の型が無い）の prop ごとに、`Prop.written` を左、`R` で名指した型（Text は `R::state::Value<String>`、Value(T) は `R::state::Value<T>`、Signal(T) は `R::state::Signal<T>`、Callback0 は `R::state::Callback<()>`、Callback(E) は `R::state::Callback<E>`、View は `R::view::View`、Children は `Vec<R::view::View>`、ViewFn は `R::view::ViewFn`）を右にした tuple を、上の `fn(..) -> .. = |written| written` の形で 1 つにまとめる。型が利用者の型引数を含むことがある（`Value<Vec<K>>`）ので、item の `const _` ではなく **constructor の本体の先頭** に `let _: fn(::std::marker::PhantomData<(..written..)>) -> ::std::marker::PhantomData<(..kivact..)> = |written| written;` として置く（constructor の中では型引数が見える）。tuple が空なら出さない。上の生成例の `const _` の形は型引数の無い部品の場合の見え方で、生成は常にこの `let _` の形にする。利用者が `Value` などを import していない、または別の `Value` 型を書いた場合、ここが型の不一致の error になる（API-18 の「foreign な型ははっきりした error で失敗」）。
     - `#[track_caller]`、`docs`、`attrs`、`deprecated` の付け方は今の `expand.rs` のまま（constructor に `attrs`・`deprecated`・`docs`、生成する各 item に `attrs`）。
     - 1 関数 80 行の上限があるので、`expand.rs` は `field`、`constructor`、`setter`、`element_impl`、`into_view`、`check`、`input_type`、`keep_type`、`store`、`convert` の関数に分ける。1000 行を超えるなら `expand/kinds.rs`（kind の表）と `expand.rs`（組立て）の 2 ファイルにする。
  5. `#[derive(Prop)]`（API-06）：`lib.rs` に足す。
     ```rust
     /// Let a type of your own be a prop constant: `#[derive(Clone, PartialEq, Prop)]`.
     /// It implements `IntoValue<Self>` for the type, so `badge("Saved", Tone::Info)` compiles.
     #[proc_macro_derive(Prop)]
     pub fn derive_prop(item: TokenStream) -> TokenStream {
         let input = syn::parse_macro_input!(item as syn::DeriveInput);
         let root = match root::root() {
             Ok(root) => root,
             Err(error) => return error.to_compile_error().into(),
         };
         let name = &input.ident;
         // Keep the type's own where-clause and add `Self: 'static` (a prop value
         // lives in the App's arena; 12 章 API-6).
         let mut generics = input.generics.clone();
         generics.make_where_clause().predicates.push(syn::parse_quote!(Self: 'static));
         let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
         quote! {
             impl #impl_generics #root::state::IntoValue<#name #ty_generics> for #name #ty_generics #where_clause {
                 fn into_prop(self) -> #root::custom::Prop<#name #ty_generics> {
                     #root::custom::Prop::constant(self)
                 }
             }
         }
         .into()
     }
     ```
     （`where Self: 'static` を足した形が generic の struct で compile することは scratchpad の `p1.rs`（`impl<T: Clone> IntoValue<Pair<T>> for Pair<T> where T: PartialEq, Self: 'static`）で確かめた。）
     `lib.rs` の `component` の doc を「Required arguments are positional; `Option<..>` arguments and `#[prop(default = ..)]` arguments are setters. Return `impl Element` to let callers style the component.」にする。`mod root;` を足す。
  6. `Node::absorb`（core）：`crates/kivact-core/src/view/elements/builders.rs` の `impl Node` に足す。
     ```rust
     /// Move the modifiers a caller wrote on a `#[component]` (collected in an
     /// empty `node()`) onto the root its body built: the caller's style layers
     /// after the body's, its handlers before the body's (as `prepend_key`), and
     /// its name, description, disabled, test id, variants and node refs over
     /// the body's (a node ref is added, so the body's own refs stay bound).
     /// Generated code calls it; write `.style(..)` and friends instead.
     #[doc(hidden)]
     pub fn absorb(mut self, caller: Node) -> Node {
         let Node { common, children, modifiers, .. } = caller;
         debug_assert!(children.is_empty(), "a #[component] caller adds no children to its root");
         self.modifiers.push(crate::view::element_data::Modifier::Caller(Box::new(std::mem::take(&mut self.common))));
         self.modifiers.extend(modifiers);
         self.common = common;
         self
     }
     ```
     `resolve_modifiers` が `Modifier::Caller` の段を順に `apply_caller` で重ねる（style の層は後ろに、handler は `merge_caller` が前に、semantic patch は後のものが勝つ）ので、規則 2 の順になる。`Modifier` を `pub(crate)` から見えるように（既に `pub(crate)`）。
  7. `scope` を `kivact::custom` へ（5.1 規則 8）：`crates/kivact-core/src/view.rs` の `pub use boundary::{ReadingDirection, provide_direction, scope, use_direction};` から `scope,` を消す。`crates/kivact-core/src/custom.rs` に `pub use crate::view::boundary::scope;` を足す。利用者の掃引：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     rg -l "view::\{[^}]*\bscope\b|view::scope\b" crates dev examples templates --type rust | xargs perl -0pi -e 's/view::scope\b/custom::scope/g'
     rg -n "\bscope\(" crates/kivact-ui-soft examples dev templates --type rust
     ```
     `use kivact::view::{.., scope, ..}` の形の import は、perl の後に残る `scope` を手で `use kivact::custom::scope;` の行に移す。prelude から `scope` を使っていた箇所（prelude の `scope` は C18 で消えるが、この unit で `prelude.rs` の `scope,` を消す）には `use kivact::custom::scope;` を足す。
  8. `#[derive(Prop)]` への置換（付属書 R の 4.7 節の依頼）：
     - `crates/kivact-ui/Cargo.toml` と `crates/kivact-interaction/Cargo.toml` の `[dependencies]` に `kivact-macros = { path = "../kivact-macros", version = "0.1.0" }` を足す（ui・interaction の `#[derive(Prop)]` と、付属書 U の W7〜W8 の `#[component]` のため）。`dev/xtask/policy/workspace.toml` の `kivact-macros = ["kivact"]` を `kivact-macros = ["kivact", "kivact-ui", "kivact-interaction"]` にする。
     - 置換の対象：
       ```sh
       ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
       rg -n "constant_value!" crates/kivact-interaction crates/kivact-ui crates/kivact-ui-basic crates/kivact-ui-pop crates/kivact-ui-game crates/kivact-ui-creative crates/kivact-ui-soft examples dev templates --type rust
       ```
       （約 62 か所。付属書 R の 4.7 節の場所、すなわち runtime・i18n・style・core は対象外。）各呼出し `…::constant_value!(A, B, ..);` について、型 `A`・`B` .. の定義（`rg -n "pub(\(crate\))? (enum|struct) A\b" <その crate>/src`）の `#[derive(..)]` に `Prop` を足し、呼出しの文を消す。derive の import：kivact-ui・kivact-interaction は `use kivact_macros::Prop;`、kit・例・試験は `use kivact::Prop;`（prelude を使うファイルは C18 の後 prelude に入るが、この unit では明示の `use` を足す）。型の定義が別の crate にある場合（orphan の規則で derive を足せない）は STOP する。
     - 置換の後：`rg -n "constant_value!" crates/kivact-interaction crates/kivact-ui crates/kivact-ui-* examples dev templates` が 0 件。
  9. API-07：`crates/kivact-core/src/lib.rs` の module doc の「created by the setup functions (`signal`, `memo`, `effect`, ... like Solid's `createSignal`, `createMemo`, `createEffect`) or by `Type::new`.」から「 or by `Type::new`」を消す。`crates/kivact-core/src/view/element_data.rs` の `Element::style` の doc の例 `.style(move || { let t = &use_theme(); style().padding(t.space().md) })` を `.style(move || use_theme().with(|t| style().padding(t.space().md)))` にする（付属書 S が W6 で theme 変数の例に書き換える）。
  9b. 手書きの `impl Element`・`impl IntoView` を `view` module だけにする（設計書 6.3、xtask の `hand-element` 検査。INC-14）：`crates/kivact-core/src/custom/measured_leaf.rs` の `Primitive`（`impl Element for Primitive`・`impl IntoView for Primitive`）は `custom` にある唯一の手書きの要素。
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-core/src" || exit 1
     git mv custom/measured_leaf.rs view/measured_leaf.rs
     ```
     - `view/measured_leaf.rs` の `use super::measurement::{Constraints, Measurement, constraints, sanitize};` を `use crate::custom::measurement::{Constraints, Measurement, constraints, sanitize};` にする。`custom.rs` の `mod measurement;` を `pub(crate) mod measurement;` にする（`custom/measurement.rs` の `pub(super) fn constraints` と `pub(super) fn sanitize` を `pub(crate)` にする）。
     - `custom.rs` の `mod measured_leaf;` を消し、`pub use measured_leaf::{Primitive, primitive};` を `pub use crate::view::measured_leaf::{Primitive, primitive};` にする。`view.rs` に `pub(crate) mod measured_leaf;` を足す（`kivact::view` から再公開はしない。公開の path は `kivact::custom::{Primitive, primitive}` のまま）。
  10. 利用者の移行：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     rg -l "#\[prop\(optional\)\]" examples templates dev crates --type rust | xargs perl -pi -e 's/#\[prop\(optional\)\]\s*//g'
     ```
     - `dev/tests/kivact-public-api/tests/*/component_macro.rs`：payload のある `Callback<E>` の props は closure だけを受ける（DEC-16）。`payload_callback_infers_the_argument`（`picker().on_pick(|n| ..)`）と `default_payload_callback_infers_the_argument` は closure の形のまま通る。`Callback<E>` の handle を payload の prop にそのまま渡している箇所（`rg -n "on_pick\(on_pick\)|\(on_[a-z_]+\)" dev/tests/kivact-public-api/tests/*/component_macro.rs` で探す）は `move |e| cb.call(e)` の形に書き換える（期待値は変えない）。`Callback<()>` の props は handle も closure も受ける。既存の 10 本の試験の期待値は変えない。`#[component] fn card(..) -> impl IntoView` のうち 1 つ（`card`）を `-> impl Element` にした新しい試験を足す（次の「試験」）。
     - `dev/tests/kivact-public-api/src/lib.rs` の doc test：`#[prop(optional)]` を使う「rejects」の例は、`#[prop(optional)]` が error になる例として残し、期待を「`#[prop] accepts only default`」に変える（`compile_fail` のまま）。「`-> impl IntoView` 以外は error」の例の文言を「`impl Element` or `impl IntoView`」にする。新しい `compile_fail` の例を足す：`Option<..>` に `#[prop(default = ..)]`、import していない `Value<String>`（`use kivact::prelude::*;` なしで `kivact::component` だけを使う）、lifetime 引数（`fn label<'a>(text: Value<String>) -> impl Element`）、const 引数（`fn dots<const N: usize>() -> impl Element`）、予約の接頭辞の型引数（`fn x<__KvT: Clone>(v: Value<__KvT>) -> impl Element`）。「generic は error」の既存の `compile_fail` の例は消す（型引数は受ける）。
     - `examples/kivact/component/src/main.rs`、`templates/basic/src/main.rs`：部品の関数を `-> impl Element` にする（root に `.style(..)` を付けられる形）。
- **消すもの**：`#[prop(optional)]`、R26 の生成の generic の名前 `M<Name>`・`R<Name>`・`V<Name>`（`__KvM`・`__KvR`・`__KvV` に）、kivact-interaction・kivact-ui・kit・例・試験の `constant_value!`、`returns_into_view`、型引数を拒む検査（`#[component] functions cannot be generic`）、`kivact::view::scope`、macro の `::kivact::` の直書き、`Box<dyn FnOnce() -> Callback<E>>` の保存。
- **状態の閉じ方**：`Model`・`Prop` は値（parse の結果）。`Required`・`PropKind`・`Returns` は enum。
- **試験**：`component_macro.rs` に足す（`e2e_core`）：
  - `an_element_component_takes_the_callers_style_and_handlers`：`#[component] fn tile(label: Value<String>) -> impl Element { node().child(text(label)).style(style().width(10)).on_key(..body の handler..) }` を `tile("A").style(style().width(40)).test_id("tile").on_key(..caller..)` で mount し、幅が 40、test id で取れる、caller の handler が body の handler より先に呼ばれる（両方が `Handled::No` を返し、呼ばれた順を `Var<Vec<&str>>` に記録して `["caller", "body"]`）ことを確かめる。
  - `an_element_component_fits_in_field`：`field("Name", my_input())`（`my_input` は `#[component] -> impl Element`）が compile し、label で取れる。
  - `a_payload_callback_prop_infers_its_closure_parameter_in_a_generic_component`：下の `choice` の `on_pick(move |k| log.update(|l| l.push(k)))`（`k` の型を書かない）が compile し、「2」を押すと `log == [2]`（DEC-16 の推論を型引数の上でも確かめる）。
  - `a_generic_component_takes_its_type_parameter_from_its_props`：`#[component] fn choice<K: Clone + PartialEq + std::fmt::Display>(items: Value<Vec<K>>, value: Signal<Option<K>>, on_pick: Option<Callback<K>>) -> impl Element`（本体は items を `for_each` で button にし、押すと `value.set(Some(k))` と `on_pick` を呼ぶ）を `choice(vec![1u32, 2, 3], picked).on_pick(move |k: u32| log.update(|l| l.push(k)))` で mount し、"2" を押すと `picked == Some(2)`、`log == [2]`。`K` は props から推論される（turbofish なし）。
  - `a_type_parameter_no_prop_names_is_kept_in_a_phantom_field`：`#[component] fn zero_of<T: Default + std::fmt::Display + Clone + PartialEq>(label: Value<String>) -> impl Element { let v = signal(T::default()); text(move || format!("{} {}", label.get(), v.get())) }` を `zero_of::<u8>("n")` で mount し、text が `"n 0"`。（callback と `ViewFn` の props が無い部品なので turbofish が書ける。4.8 節。）
  - `a_component_taking_its_type_parameter_as_a_prop_infers_it`（12 章 API-1）：`#[component] fn labelled<C: Element>(label: Value<String>, control: C) -> impl Element { column((text(label), control)) }` を `labelled("Name", button("Go"))`（`C = Button`）と `labelled("Box", node().test_id("n"))`（`C = Node`）で mount する（型注釈なし。設計書 5.1 規則 3 の `field("Name", text_input(v))` と同じ形）。button と node が label の下に出る。
  - `a_callback_prop_keeps_the_handled_result`（12 章 API-14。props の経路を確かめる。手で転送する `call` は試さない）：`#[component] fn keys(on_key_prop: Callback<KeyEvent>) -> impl Element { node().focusable(true).test_id("k").on_key(move |e| on_key_prop.invoke(e)) }` を、`.on_key(move |_| { parent_saw.set(true); Handled::No })` を持つ column の中に置く。(a) `keys(|_| Handled::Yes)`："k" に focus して Enter を押すと `parent_saw == false`。(b) `keys(|_| Handled::No)`：`true`。(c) `Callback<()>` の prop：`#[component] fn press_through(on_go: Callback<()>) -> impl Element { node().focusable(true).test_id("p").on_key(move |_| on_go.invoke(())) }` に共有の `let go = callback(|_: ()| Handled::Yes);` を `press_through(go)` で渡し、"p" で Enter を押すと `parent_saw == false`（共有の `Callback<()>` の props の経路でも `Handled` が保たれる）。
  - `derive_prop_on_a_generic_type`（12 章 API-6）：`#[derive(Clone, Debug, PartialEq, Prop)] struct Pair<T: Clone + PartialEq + std::fmt::Debug> where T: Default { a: T }` を `#[component] fn shows_pair(pair: Value<Pair<u8>>) -> impl Element { text(move || format!("{}", pair.get().a)) }` に定数 `Pair { a: 7 }` で渡して `"7"`。
  - `a_generic_component_with_a_where_clause`：`#[component] fn shown<T>(value: Value<T>) -> impl Element where T: Clone + PartialEq + std::fmt::Display { text(move || value.get().to_string()) }` を `shown(3.5f64)` と `shown(move || n.get())`（`n: Signal<i32>`）で mount し、`"3.5"` と `n` の値が出て、`n.set(..)` で変わる。
- **確認（tier a）**：
  ```sh
  cargo fmt -p kivact-macros -p kivact-core -p kivact
  cargo check -p kivact-macros --all-targets
  cargo clippy -p kivact-macros --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-macros
  cargo xtask check --no-write --strict --only hand-element --crate kivact-core
  # 4.6 節の W5_CHECK
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- component_macro
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --doc
  ```
- **完了条件**：`rg -n "prop\(optional\)" crates dev examples templates` が 0 件（`kivact-macros` の error の文言を除く）。`rg -n "::kivact::" crates/kivact-macros/src` が 0 件（`#root` を使う）。`rg -n "pub use boundary::\{[^}]*scope" crates/kivact-core/src/view.rs` が 0 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C10：`Element::variant` と `Variant`・`Variants` を core に置く

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C09
- **所有**：新 `crates/kivact-core/src/view/variant.rs`、`crates/kivact-core/src/view/{view.rs の mod 行,element_data.rs,style_layer.rs,mount/parts.rs,mount/style.rs,mount/create.rs,tree.rs,node_modifiers.rs}`、`crates/kivact-ui/src/{variants.rs,lib.rs}`、`crates/kivact-interaction/src/` の `Tone` の定義のあるファイル（impl 1 行）、`impl Variant for` を持つ ui・kit のファイル（`Clone` の derive を足すだけ）
- **目的**：設計書 5.4：`Element` に `fn variant<V: Variant>(self, v: impl IntoValue<V>) -> Self`。variant は node が `Option<Box<Variants>>` で持つ（MEM-2）。ui と kit の 68 個の写しは付属書 U・K が消す（この unit は core の受け口だけを作る）。
- **手順**：
  1. 新 `view/variant.rs`：
     ```rust
     //! Role: Variants: the looks a component takes with `.variant(..)`, kept on its root node.

     use std::any::{Any, TypeId};
     use std::rc::Rc;

     use kivact_runtime::Prop;

     /// A look a component takes with `.variant(value)` and its recipes read: a
     /// value of the theme's or kit's own type. Implemented explicitly
     /// (`impl Variant for Pill {}`); not sealed, so a kit defines its own.
     pub trait Variant: Clone + PartialEq + std::fmt::Debug + 'static {}

     /// The variant values of one node, one per type (a later value of a type
     /// replaces the earlier one). A value may be reactive.
     #[derive(Clone, Default)]
     pub struct Variants(Rc<Vec<(TypeId, Rc<dyn Any>)>>);

     impl Variants {
         /// Set the value of type `V`, replacing an earlier one.
         pub(crate) fn set<V: Variant>(&mut self, value: Prop<V>) {
             let entries = Rc::make_mut(&mut self.0);
             entries.retain(|(ty, _)| *ty != TypeId::of::<V>());
             entries.push((TypeId::of::<V>(), Rc::new(value)));
         }

         /// Merge `later` over these values (a later value of a type wins).
         pub(crate) fn merge(&mut self, later: &Variants) {
             let entries = Rc::make_mut(&mut self.0);
             for (ty, value) in later.0.iter() {
                 entries.retain(|(existing, _)| existing != ty);
                 entries.push((*ty, value.clone()));
             }
         }

         /// The value of type `V` the component was given, if any. Tracked when
         /// the value is reactive.
         #[track_caller]
         pub fn get<V: Variant>(&self) -> Option<V> {
             self.0
                 .iter()
                 .find(|(ty, _)| *ty == TypeId::of::<V>())
                 .and_then(|(_, value)| value.downcast_ref::<Prop<V>>())
                 .map(Prop::get)
         }
     }
     ```
     `view.rs` に `mod variant;` と次を足す（設計書 5.9：`Variant` の公開の path は `kivact::ui::theme::Variant` だけ。facade は `kivact_core::view` を丸ごと再公開するので、core の再公開を hidden にする。INC-22）：
     ```rust
     // xtask-allow(internal-contract): the public path of Variant and Variants is kivact::ui::theme (annex U re-exports them)
     #[doc(hidden)]
     pub use variant::{Variant, Variants};
     ```
     （`internal-contract` の印は C08 の手順 4 と同じ規則で付けるか決める。）
  2. `view/element_data.rs`：
     - `Common` に `// xtask-allow` 無しで `pub(crate) variants: Option<Box<crate::view::Variants>>,` を足す（`Common` は builder の値。constructor と値渡しの builder だけが書く）。
     - `Element` trait に：
       ```rust
       /// Choose a look of the component: a value of a variant type its theme's
       /// recipes know (`ButtonVariant::Ghost`, a kit's own enum). One value per
       /// type; a later one replaces an earlier one. A theme that does not know the
       /// type draws the plain look. It may be reactive (`move || ..`, a `Signal`).
       #[track_caller]
       fn variant<V: crate::view::Variant>(mut self, v: impl IntoValue<V>) -> Self {
           self.node_mut().common.variants.get_or_insert_with(Default::default).set(v.into_prop());
           self
       }
       ```
  3. `view/mount/parts.rs` の `Common::apply_caller`：`replace_fields!(..)` の後に：
     ```rust
     if let Some(later) = later.variants.take() {
         self.variants.get_or_insert_with(Default::default).merge(&later);
     }
     ```
  4. variant を読む style の層（recipe が mount の時の最終の variant を受け取る）：`view/style_layer.rs` の `StyleLayer.f: Rc<dyn Fn() -> Style>` を `f: LayerFn` にする。
     ```rust
     /// What a style layer computes from.
     #[derive(Clone)]
     pub(crate) enum LayerFn {
         /// A constant or reactive style (`.style(..)`).
         Plain(Rc<dyn Fn() -> Style>),
         /// A recipe that reads the node's final variants.
         Variants(Rc<dyn Fn(&crate::view::Variants) -> Style>),
     }

     impl LayerFn {
         pub(crate) fn style(&self, variants: &crate::view::Variants) -> Style {
             match self {
                 LayerFn::Plain(f) => f(),
                 LayerFn::Variants(f) => f(variants),
             }
         }
     }
     ```
     `element_data.rs` の `prepend_style`・`push_style`・`push_recipe_style`・`prepend_recipe_style` は `f: LayerFn::Plain(s.into_fn())` にする。`Node` に 2 つの method を足す（`view/node_modifiers.rs` の `base_recipe_style` の次）：
     ```rust
     /// Add a recipe style layer that reads the node's final variants (after the
     /// caller's `.variant(..)`). DevTools shows `name`.
     #[doc(hidden)]
     #[track_caller]
     pub fn variant_style(mut self, name: crate::view::RecipeName, f: impl Fn(&crate::view::Variants) -> Style + 'static) -> Self {
         self.common.style.push(StyleLayer { f: LayerFn::Variants(Rc::new(f)), at: Some(Location::caller()), origin: LayerOrigin::Recipe(name) });
         self
     }

     /// The base-layer form of [`Node::variant_style`]: before every caller style.
     #[doc(hidden)]
     #[track_caller]
     pub fn base_variant_style(mut self, name: crate::view::RecipeName, f: impl Fn(&crate::view::Variants) -> Style + 'static) -> Self {
         let index = self.common.base_count();
         self.common.style.insert(index, StyleLayer { f: LayerFn::Variants(Rc::new(f)), at: Some(Location::caller()), origin: LayerOrigin::BaseRecipe(name) });
         self
     }
     ```
     （付属書 U の `Look<S>` はこの 2 つで recipe を node に付ける。）
  5. mount：`view/mount/style.rs` の `merge_layers(layers: &[StyleLayer]) -> Style` を `merge_layers(layers: &[StyleLayer], variants: &Variants) -> Style`（`(layer.f)()` → `layer.f.style(variants)`）にする。`NodeStyle` に `pub variants: Variants,` を足す（`Default` で空）。`bind_node_style` を呼ぶ箇所（`view/mount/create.rs` の node を作る関数。`rg -n "bind_node_style\(" crates/kivact-core/src/view/mount`）で、`common.variants.take().map(|v| *v).unwrap_or_default()` を `NodeStyle.variants` に渡す（`bind_node_style` の引数に `variants: Variants` を足す）。`bind_style` の `merged` の memo（C04）で `merge_layers(&layers, &variants)` を使う。
  6. DevTools：`view/tree.rs` の `set_style_layers(id, layers)` の呼出し（C01 の `mount/style.rs`）では、`LayerFn::Variants` の層をその node の `variants` で包んだ `LayerFn::Plain` にしてから渡す（`layers.iter().map(|l| match &l.f { LayerFn::Variants(f) => { let (f, v) = (f.clone(), variants.clone()); StyleLayer { f: LayerFn::Plain(Rc::new(move || f(&v))), ..l.clone() } } LayerFn::Plain(_) => l.clone() }).collect()`）。`devtools/` の `(layer.f)()` の読み出しは `layer.f.style(&Variants::default())` にする（ここに来る層はすべて `Plain`）。
  7. ui の trait を core のものにする：`crates/kivact-ui/src/variants.rs` の `pub trait Variant: PartialEq + std::fmt::Debug + 'static {}` を消し、`pub use kivact_core::view::Variant;` にする。`impl Variant for kivact_interaction::Tone {}` を消し、`kivact-interaction` の `Tone` の定義の直後に `impl kivact_core::view::Variant for Tone {}` を足す。`Variant` に `Clone` が加わったので、`impl Variant for X {}`（`rg -n "impl (crate::|kivact::ui::|kivact_ui::)?Variant for" crates examples dev`）の `X` が `Clone` でなければ `#[derive(Clone, ..)]` を足す。ui の `Extensions::with<T: Variant>` などの内部は変えない（付属書 U が W7〜W8 に `Variants` へ移す）。
- **消すもの**：ui の `trait Variant` の定義、ui の `impl Variant for Tone`。
- **状態の閉じ方**：`Variants` は `Rc<Vec<..>>`（値）。`Common.variants` は builder の値の field。`LayerFn` は enum。
- **試験**：`e2e_core` の `components.rs` に `element_variant_reaches_a_variant_recipe_and_follows_a_signal` を足す：`#[derive(Clone, Copy, Debug, PartialEq, Prop)] enum Size { Small, Large } impl Variant for Size {}`、`node().base_variant_style(Rc::new(|| "size"), |v| match v.get::<Size>() { Some(Size::Large) => style().width(80), _ => style().width(20) })` を返す `#[component] fn sized() -> impl Element` を `sized().variant(move || size.get())` で mount し、幅が 20、`size.set(Size::Large)` の後 80。
- **確認（tier a）**：4.6 節の `W5_CHECK` と：
  ```sh
  cargo xtask check --no-write --crate kivact-core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- components
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_
  ```
- **完了条件**：`rg -n "pub trait Variant" crates` が `kivact-core/src/view/variant.rs` の 1 件だけ。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C11：`App` を唯一の builder にし（`!Clone`）、launch の内部を隠し、GPU の希望の型を `platform` へ

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C10
- **所有**：`crates/kivact-core/src/{app.rs,launch.rs,launch/*,lib.rs,platform.rs,platform/app_handle.rs,embed/open.rs,embed/embedded.rs}`、`crates/kivact/src/{lib.rs,launch.rs（削除）}`、`crates/kivact-test/src/`（`options.rs`、`facts.rs`、`screen/{driver.rs,model.rs,state.rs}`、`kivact-test` の中の `launch::` を使う全ファイル）、`crates/kivact-engine/src/embed.rs`（`retain_module` の 2 か所）、`launch::Gpu*` の呼出し元（`dev/tests/kivact-public-api/tests/*/{gpu_interop.rs,drawable_export_gpu.rs}`）（呼出し元の `dev/` は `dev/tests/kivact-public-api`・`dev/tests/kivact-kit-tests`・`dev/tests/gpu-interop-producer`・`dev/bench` を含む。W5 では kit・catalog・`kivact-kit-tests` も compile するので直す。INC-33）
- **目的**：RC-13：`AppOptions` を消し `App` が値を持つ。`App: !Clone`、二度目の run の panic の経路を消す。`Launch`・`Inspect`・`LaunchHandle` を `kivact_core::__test`（`#[doc(hidden)]`）へ。RC-12：`ClientMode` を `LaunchPolicy` にし、場合分けを一つの値に寄せる。閉じる途中の embedded App は turn を回さない。module の保持は engine の `EmbeddedHost` だけ。`devtools_in_tests` を消す。API-11：`kivact::launch` を消し、GPU の 5 型を `kivact::platform` へ。
- **手順**：
  1. `launch/settings.rs`：`AppOptions` を消し、その field と method を `App` に移す（`app.rs`）。
     ```rust
     /// The App: its window, theme, locale, texts and run. Build it with
     /// `App::new(title)` and the setters, then `run` it once.
     pub struct App {
         theme: kivact_runtime::Prop<Theme>,
         pub(crate) settings: AppSettings,
     }
     ```
     `#[derive(Clone)]` は付けない。`App::new`・`theme`・`locale`・`messages`・`window_size`・`min_window_size`・`font`・`fonts`（C14 が消す）・`blocking_threads`・`gpu_preference`・`handle`・`run` は `App` の上で直接 field を書く（旧 `self.options = self.options.x(..)` の転送を消し、`AppOptions` の本体をそこに置く）。`AppOptions::{check_window_sizes, title, host_options, window_title, into_embed_settings, into_view_settings}` は `App` の `pub(crate)` method にする（`title` は `pub(crate) fn retitle(mut self, title: impl Into<String>) -> Self`）。`AppSettings::with_theme(self, theme) -> AppOptions` は `-> App`。`AppSettings.devtools_in_tests` と `AppOptions::devtools_in_tests` を消す。`launch/settings.rs` には `AppSettings` だけが残る。
     `gpu_preference(mut self, preference: crate::platform::GpuPreference)`、doc の `crate::launch::GpuStart` を `crate::platform::GpuStart` にする。
  2. 新 `launch/policy.rs`：
     ```rust
     //! Role: What differs between a window run, an embedded run and a test run.

     use std::rc::Rc;

     /// How a launch treats closing, DevTools and test timing. Built once by
     /// `App::run`, `App::embed` and kivact-test.
     pub struct LaunchPolicy {
         pub(crate) close: ClosePolicy,
         /// Whether the DevTools agent may start.
         pub(crate) devtools: bool,
         pub(crate) test: Option<TestRun>,
     }

     /// Who may end the run.
     pub(crate) enum ClosePolicy {
         /// `AppHandle::close` and the main window's close end it.
         Accept,
         /// The embedder ends it by dropping `Embedded`; `AppHandle::close` is
         /// reported as unsupported. The signal tells that the host is closing.
         Embedded(kivact_engine::contract::CloseSignal),
     }

     /// A kivact-test run: its turn timing reporter and the test's name (the
     /// trace file is named after it).
     pub(crate) struct TestRun {
         pub(crate) timing: Rc<dyn Fn(u64)>,
         pub(crate) name: String,
     }

     impl LaunchPolicy {
         pub(crate) fn window() -> Self {
             Self { close: ClosePolicy::Accept, devtools: true, test: None }
         }
         pub(crate) fn embedded(closing: kivact_engine::contract::CloseSignal) -> Self {
             Self { close: ClosePolicy::Embedded(closing), devtools: true, test: None }
         }
         /// A kivact-test run.
         pub fn test(name: String, timing: Rc<dyn Fn(u64)>, devtools: bool) -> Self {
             Self { close: ClosePolicy::Accept, devtools, test: Some(TestRun { timing, name }) }
         }
         /// A window run without a test harness (kivact-test's own launch tests).
         pub fn plain_window() -> Self {
             Self::window()
         }
         pub(crate) fn is_embedded(&self) -> bool {
             matches!(self.close, ClosePolicy::Embedded(_))
         }
         /// Whether the embedded host asked to close (never for other runs).
         pub(crate) fn closing(&self) -> bool {
             matches!(&self.close, ClosePolicy::Embedded(closing) if closing.is_set())
         }
     }
     ```
  3. `launch/runtime.rs` と `launch/turn.rs`（付属書 R の R06 が turn の関数 `Launch::turn` をここへ移してある）：`ClientMode`、`TestTiming`、`ClientMode::devtools_allowed` を消す。`Launch.mode: ClientMode` を `policy: LaunchPolicy` にする。各場合分けを次にする：
     - `start`：`#[cfg(debug_assertions)] let trace_name = self.policy.test.as_ref().map(|t| t.name.clone());`。DevTools の起動の引数 `self.mode.devtools_allowed(options.settings.devtools_in_tests)` を `self.policy.devtools`、`!matches!(self.mode, ClientMode::Embedded(_))` を `!self.policy.is_embedded()` にする。
     - `turn`（`launch/turn.rs`）：先頭（`rt.advance_clock(now_ns);` の前）に
       ```rust
       // A closing embedded App runs no turn: its host is tearing it down.
       if self.policy.closing() {
           return output(rt.end_idle());
       }
       ```
       を置き（`Runtime::end_idle` は R06。4.1 節 R-REQ-8）、`for event in events` の中の `if matches!(&self.mode, ClientMode::Embedded(closing) if closing.is_set()) { break; }` を消す。`let embedded = matches!(self.mode, ClientMode::Embedded(_)); super::close::take_close_request(&rt, embedded);` を `super::close::take_close_request(&rt, self.policy.is_embedded());` にする。
     - `Drop for Launch` の最後の `if matches!(self.mode, ClientMode::Embedded(_)) && ... { kivact_engine::embed::retain_module(); }` を消す。
     - `shared_client(shared, timing)` の `timing` は `policy.test.as_ref().map(|t| t.timing.clone())` から渡す（`inspect.rs` の `into_engine_client`）。
     - `Launch::new(options: AppOptions, mode: ClientMode, root)` を `Launch::new(app: App, policy: LaunchPolicy, root)` にし、`LaunchState::Waiting { options: Box<App>, .. }` にする（`options.settings`・`options.handle()` などの読み出しは `App` の同名の field と method）。
  4. `embed/embedded.rs` の `struct Retention` と `Embedded._retention` field を消す（`Embedded::new` の初期化からも）。`crates/kivact-engine/src/embed.rs`：`impl Drop for EmbeddedHost` の `self.closing.close();` の後と、`embed_native` が `Err` を返す直前に、次を置く：
     ```rust
     if kivact_types::process::DetachedThreads::any() {
         retain_module();
     }
     ```
     （engine の `pub use arvane_native::retain_module;` は既にある。）
  5. `launch.rs` を次にする：
     ```rust
     //! Role: Shared application launch and read-only inspection boundary.
     mod close;
     mod inspect;
     mod policy;
     mod runtime;
     mod settings;
     pub(crate) use inspect::{Inspect, LaunchHandle};
     pub(crate) use policy::{ClosePolicy, LaunchPolicy};
     pub(crate) use runtime::{Launch, run_native};
     pub(crate) use settings::AppSettings;
     ```
     GPU の 5 型の re-export を `platform.rs` に移す：
     ```rust
     /// An adapter's identity, as the graphics APIs report it.
     #[doc(inline)]
     pub use kivact_types::AdapterMatch;
     /// A graphics API the window's GPU device can use.
     #[doc(inline)]
     pub use kivact_types::GpuBackend;
     /// Which kind of adapter an App prefers ([`App::gpu_preference`](crate::App::gpu_preference)).
     #[doc(inline)]
     pub use kivact_types::GpuPower;
     /// What an App wishes of the window's GPU device ([`App::gpu_preference`](crate::App::gpu_preference)).
     #[doc(inline)]
     pub use kivact_types::GpuPreference;
     /// When the window's GPU device is started (`GpuPreference::start`).
     #[doc(inline)]
     pub use kivact_types::gpu::GpuStart;
     ```
  6. `lib.rs`：`pub mod launch;` を `mod launch;` にし、次を足す：
     ```rust
     // xtask-allow(internal-contract): kivact-test drives an App through the same launch as App::run
     #[doc(hidden)]
     pub mod __test {
         //! Role: The launch contract kivact-test drives; not an App API.
         pub use crate::launch::{Inspect, LaunchHandle, LaunchPolicy};
         pub use crate::app::test_launch::{app_theme_has_fonts, check_window_sizes, host_options, launch, retitle, window_title};
     }
     ```
     （`internal-contract` の印は C08 の手順 7 と同じ規則で付けるか決める。）`app.rs` の末尾に：
     ```rust
     pub(crate) mod test_launch {
         //! Role: The functions `kivact_core::__test` exposes over a private App.
         use super::App;
         use crate::launch::{Launch, LaunchHandle, LaunchPolicy};
         use crate::view::View;

         /// Launch `app` under `policy` with `root`, returning the engine client and
         /// the inspection handle.
         #[track_caller]
         pub fn launch(app: App, policy: LaunchPolicy, root: impl FnOnce() -> View + 'static) -> (kivact_engine::api::Client, LaunchHandle) {
             Launch::new(app, policy, root).into_engine_client()
         }
         pub fn host_options(app: &App) -> kivact_engine::api::HostOptions { app.host_options() }
         pub fn window_title(app: &App) -> &str { app.window_title() }
         #[track_caller]
         pub fn check_window_sizes(app: &App) -> crate::Result<()> { app.check_window_sizes() }
         pub fn retitle(app: App, title: impl Into<String>) -> App { app.retitle(title) }
         /// Whether the App's fixed theme carries fonts (C14 fills it).
         pub fn app_theme_has_fonts(app: &App) -> bool { let _ = app; false }
     }
     ```
     （`app_theme_has_fonts` は C14 で本体を書く。この unit では `false` を返す。）
  7. `App::run`：
     ```rust
     #[track_caller]
     pub fn run<V: IntoView>(self, root: impl FnOnce() -> V + 'static) -> Result<()> {
         self.check_window_sizes()?;
         if self.settings.handle.begin() == Begin::Skip {
             return Ok(());
         }
         let options = self.host_options();
         let (client, _) = crate::launch::Launch::new(self, crate::launch::LaunchPolicy::window(), move || root().into_view())
             .into_engine_client();
         crate::launch::run_native(client, options)
     }
     ```
     `embed/open.rs` の `Launch::new(.., ClientMode::Embedded(closing), ..)` を `Launch::new(.., LaunchPolicy::embedded(closing), ..)` にし、`AppOptions` の読み出しを `App` に直す。
  8. `platform/app_handle.rs`：`App` が `Clone` でなく `run(self)` が消費するので、一つの handle が二度 `begin` されることは無い。`AppLink::begin` を次にする：
     ```rust
     #[track_caller]
     fn begin(&self) -> Begin {
         let mut state = self.lock();
         match state.clone() {
             LinkState::Unstarted => Begin::Run,
             LinkState::ClosePending => {
                 *state = LinkState::Ended;
                 Begin::Skip
             }
             LinkState::Running(_) | LinkState::CloseRequested(_) | LinkState::Exiting | LinkState::Ended => {
                 drop(state);
                 unreachable!("invariant: App::run consumes the App, so its handle begins once")
             }
         }
     }
     ```
     `app_handle.rs` の試験のうち `#[should_panic(expected = "one AppHandle drives one run")]` の試験を消す。`app.rs` の `handle_tests::a_handle_cannot_drive_a_second_run`（`app.clone()` を使う）を消す。`App::handle` の doc の「Clones of this `App` share it」を消す。
  9. facade：`crates/kivact/src/launch.rs` を消し、`lib.rs` の `/// What an App wishes ...` と `pub mod launch;` を消す。
  10. kivact-test：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-test" || exit 1
     rg -l "kivact_core::launch::" src | xargs perl -pi -e 's/kivact_core::launch::\{?Inspect\}?/kivact_core::__test::Inspect/g; s/kivact_core::launch::\{Inspect, LaunchHandle\}/kivact_core::__test::{Inspect, LaunchHandle}/g; s/kivact_core::launch::GpuPreference/kivact_core::platform::GpuPreference/g'
     ```
     - `options.rs`：`RenderOptions` の `#[derive(Clone)]` を消す。`app: kivact_core::launch::AppOptions` を `app: kivact_core::App` にし、`devtools: bool` field を足す（`Default` で `false`、`pub fn devtools(mut self) -> Self { self.devtools = true; self }`）。`AppOptions::new(..)` を `kivact_core::App::new(..)`、`self.app.title(title)` を `kivact_core::__test::retitle(self.app, title)`、`app.check_window_sizes()` を `kivact_core::__test::check_window_sizes(&app)`、`app.window_title()` を `kivact_core::__test::window_title(&app)`、`app.host_options()` を `kivact_core::__test::host_options(&app)` にする。`into_host` は `(kivact_core::App, bool /*devtools*/, TestHostOptions)` を返す。`Debug` の `self.app.window_title()` なども同じく直す。
     - `screen/driver.rs`：`Launch::new(app, ClientMode::Test { timing: TestTiming(Rc::new(..)), name: name.clone() }, root).into_engine_client()` を `kivact_core::__test::launch(app, kivact_core::__test::LaunchPolicy::test(name.clone(), Rc::new(..), devtools), root)` にする。
     - `kivact-test` の中の `ClientMode::Window` と `Launch::new(AppOptions::new(..), ..)` を使う自己試験（`rg -n "ClientMode|AppOptions" crates/kivact-test dev/tests` で見つかる箇所。W1 で `dev/tests` へ移っていればそこ）は `kivact_core::__test::launch(kivact_core::App::new(..), kivact_core::__test::LaunchPolicy::plain_window(), ..)` にする。
  11. GPU の型の呼出し元：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     rg -l "kivact::launch::" crates dev examples templates --type rust | xargs perl -pi -e 's/kivact::launch::/kivact::platform::/g'
     rg -l "crate::launch::Gpu" crates/kivact-core | xargs perl -pi -e 's/crate::launch::Gpu/crate::platform::Gpu/g'
     ```
- **消すもの**：`AppOptions`、`ClientMode`、`TestTiming`、`Launch` と `Inspect`・`LaunchHandle` の公開の path、`kivact::launch`（file `crates/kivact/src/launch.rs`）、`kivact_core::launch` の公開、`AppSettings.devtools_in_tests`、`Retention`、`Launch` の `Drop` の `retain_module`、`App: Clone`、`AppLink::begin` の二度目の run の panic、試験 `a_handle_cannot_drive_a_second_run` と `app_handle.rs` の `should_panic` の試験。
- **状態の閉じ方**：`LaunchPolicy` は名前が `Policy` で終わる（対象外）。`ClosePolicy` は enum。`App` は値渡しの builder（6.1 の「値渡しの builder だけで書く field は状態ではない」）。`RenderOptions` は名前が `Options` で終わる。
- **試験**：`app.rs` の `window_size_tests`（`app.options.check_window_sizes()` を `app.check_window_sizes()`、`app.options.host_options()` を `app.host_options()` に直す）と `closing_before_the_run_returns_without_opening_a_window`。`e2e_core` の `app_handle`、`launch_window`、`launch_platform`、`e2e_native` の `embed`、`gpu_interop`、`devtools`。
- **確認（tier a）**：4.6 節の `W5_CHECK` と：
  ```sh
  cargo xtask check --no-write --crate kivact-core
  cargo xtask check --no-write --crate kivact-test
  KIVACT_TEST_MIRROR=off cargo test -p kivact-core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-test
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- app_handle launch_window launch_platform
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --features embed,devtools --test e2e_native -- embed devtools
  ```
- **完了条件**：`rg -nw "AppOptions|ClientMode|TestTiming|devtools_in_tests" crates dev examples templates` が 0 件。`rg -n "kivact::launch|pub mod launch" crates dev examples templates` が 0 件。`rg -n "derive\(Clone\)\]\s*pub struct App\b" crates/kivact-core` が 0 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C12：`on_close_request` を呼んだ owner の window に効かせ、終了の経路を一つにする

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C11
- **所有**：`crates/kivact-core/src/{platform/close_request.rs,platform.rs,launch/close.rs,launch/runtime.rs,client/dispatch/env.rs,view/state.rs,view/window_view.rs}`、`dev/tests/kivact-public-api/tests/*/{close_request.rs,windows.rs}`、`on_close_request` の呼出し元
- **目的**：RC-11。`platform::on_close_request(handler: impl IntoHandler<(), M>)` は setup の中だけで呼べ、呼んだ owner の window（main、`window()`、foreign window）に効く。main 以外の window では handler が閉じるのを止めなければ `on_dismiss(DismissReason::WindowClose)` が続く。終了の始め（`request_exit` と foreign window の close）を `begin_exit` 一つにする。
- **手順**：
  1. `launch/close.rs` に：
     ```rust
     /// Start the App's exit: the runtime drains, and every foreign window's
     /// `open` binding is written false so its `on_destroy` runs while its
     /// child still exists. The one exit path of `AppHandle::close` and of a
     /// close request on the main window.
     pub(crate) fn begin_exit(rt: &Runtime) {
         rt.request_exit();
         crate::view::state::ViewState::of(rt).close_foreign_for_exit();
     }
     ```
     `take_close_request` の `rt.request_exit(); ...close_foreign_for_exit();` を `begin_exit(rt);` にする。`launch.rs` に `pub(crate) use close::begin_exit;` を足す。
  2. `view/state.rs` の `WindowRecord` に `pub(crate) close_handlers: Rc<RefCell<kivact_types::cell::Registry<crate::platform::close_request::Registration>>>,` を足し、C05 の `open_window_record` で `close_handlers: Rc::default(),` にする。`WindowRecord` の印（C05）に「close_handlers は登録の表（owner が登録し、owner の cleanup が消す）」を足す。
  3. `platform/close_request.rs` を次にする：
     ```rust
     //! Role: Owner-scoped handlers that may veto closing the window they were registered in.

     use std::panic::Location;
     use std::rc::Rc;

     use kivact_runtime::{Handler, IntoHandler, Owner};

     /// One `on_close_request` handler and the owner it runs under.
     pub(crate) struct Registration {
         pub(crate) owner: Owner,
         pub(crate) handler: Handler<()>,
     }

     /// Ask before the window this component is in closes: the handler runs when
     /// the user asks to close that window (title bar button, Alt+F4) and keeps
     /// it open if it returns `Handled::Yes` or `Handled::PreventDefault`, or
     /// fails (the error is reported). Return `()` or `Handled::No` to let it
     /// close. In the main window a close ends the App; in a `window` or
     /// `foreign_window` it then runs the window's `on_dismiss`. Every live
     /// handler of the window runs, in registration order.
     ///
     /// Call it in setup; it is removed when the calling component unmounts.
     /// Closing with [`AppHandle::close`](crate::platform::AppHandle::close)
     /// and an embedded App never run it.
     #[track_caller]
     pub fn on_close_request<M>(handler: impl IntoHandler<(), M>) {
         let at = Location::caller();
         let owner = Owner::setup("on_close_request", at);
         let window = crate::view::state::current_window(owner);
         let registration = Rc::new(Registration { owner, handler: handler.into_handler() });
         let rt = owner.runtime();
         let handlers = crate::view::state::ViewState::of(&rt)
             .with_window(window, |record| record.close_handlers.clone())
             .expect("invariant: the window of a mounting owner is recorded");
         handlers.borrow_mut().insert(&registration);
         owner.on_cleanup(move || drop(registration));
     }
     ```
     （`Owner::setup` は付属書 R の R25 が `Owner::current` から改名した、setup だけの owner の取得。`IntoHandler`・`into_handler` は R21 の名前。）doc の例の `on_close_request(move || ..)` はそのまま compile する。`CloseRequestState`・`registrations` を消す。`launch/runtime.rs` の `.with(crate::platform::close_request::CloseRequestState::default())` を消す（付属書 R の RC-04 で `AppState` になっていれば、その field を消す）。
  4. `client/dispatch/env.rs` の `close_requested`：
     ```rust
     pub(super) fn close_requested(rt: &Runtime, window: WindowId) {
         let Some(record) = ViewState::of(rt).window_record(window) else {
             return;
         };
         if keeps_open(rt, &record) {
             return;
         }
         if window == WindowId::MAIN {
             crate::launch::begin_exit(rt);
             return;
         }
         let close = match &record.decl.dismiss {
             None => true,
             Some(handler) => {
                 run_handler(rt, Some(record.declared_in), || handler.call(DismissReason::WindowClose)) == Some(Handled::No)
             }
         };
         if close {
             record.decl.open.set(false);
         }
     }

     /// Run every live close handler of `record`'s window in registration order,
     /// each under its owner. `true` when one answers `Yes` or `PreventDefault`,
     /// or fails (its error is reported). Every handler runs either way.
     fn keeps_open(rt: &Runtime, record: &crate::view::state::WindowRecord) -> bool {
         let registrations = record.close_handlers.borrow_mut().live();
         let mut keeps_open = false;
         for registration in registrations {
             if !registration.owner.is_alive() {
                 continue;
             }
             let answer = run_handler(rt, Some(registration.owner), || registration.handler.call(()));
             keeps_open |= answer != Some(Handled::No);
         }
         keeps_open
     }
     ```
     `keeps_main_open` と `use crate::platform::close_request;` を消す。
  5. 呼出し元：`rg -n "on_close_request\(" crates dev examples templates docs --type rust`。setup の外（handler・effect の中）で呼んでいる箇所があれば、setup に移す（e2e の `close_request.rs` はすべて setup の中）。
- **消すもの**：`CloseRequestState`、`registrations`、`keeps_main_open`、`client/dispatch/env.rs` と `launch/close.rs` の 2 つの `request_exit` + foreign の close の写し。
- **状態の閉じ方**：`WindowRecord.close_handlers` は登録の表（C05 の印に 1 文足す）。
- **試験**：`e2e_core` の `close_request.rs` の 7 本はそのまま通る。`windows.rs` に足す：
  - `a_close_request_handler_in_a_window_keeps_only_that_window_open`：main の root と `window("Editor", open).content(|| { on_close_request(|| Handled::Yes); text("editor") })` を mount。`screen.window("Editor").request_close()` が `false` で window は開いたまま、`screen.request_close()`（main）は App を終える（main には handler が無い）。
  - `a_window_close_handler_that_allows_runs_the_dismiss_handler`：window の中の `on_close_request(|| Handled::No)` と `.on_dismiss(|_| Handled::PreventDefault)`：`request_close()` は `false`（dismiss が止める）。`on_close_request(|| ())` と `.on_dismiss(|_| ())`：`true` で閉じる。
  - `on_close_request_outside_setup_is_a_setup_only_error`：button の `on_press` の中で `on_close_request(|| ())` を呼ぶと、`screen.errors()` に `SetupOnly` の error が 1 つ（位置は呼んだ行）。
  - `close_request_handled_is_the_same_inline_shared_and_through_a_prop`（`e2e_core` の `close_request.rs`。T15 で `close_request` は `e2e_core`。12 章 API-14 の window の経路。R-REQ-16）：三つの window をそれぞれ `window(..).content(..)` で開き、content の setup で (a) `on_close_request(|| Handled::Yes)`（inline）、(b) `let keep = callback(move |_: ()| Handled::Yes); on_close_request(keep);`（共有の `Callback<()>`）、(c) `#[component] fn guard(on_close: Callback<()>) -> impl IntoView { on_close_request(on_close); text("guard") }` を `guard(|| Handled::Yes)` で置く（`#[component]` の `Callback<()>` の prop を通す）。三つとも `screen.window(..).request_close()` が `false` で window が開いたまま。同じ形で `Handled::No` を返す三つは `true` で閉じる。
- **確認（tier a）**：4.6 節の `W5_CHECK` と：
  ```sh
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- close_request windows app_handle
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --features embed --test foreign_window_run
  ```
- **完了条件**：`rg -n "CloseRequestState|keeps_main_open" crates` が 0 件。`rg -n "request_exit\(\)" crates/kivact-core/src` が `begin_exit` の 1 件だけ。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C13：embed を全 OS で compile、停止は `EmbedNotice::Stopped(EmbedStatus)`、`EmbedVisibility`

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C12
- **所有**：`crates/kivact-core/src/{embed.rs,embed/*}`、`crates/kivact-types/src/{embed.rs,lib.rs}`、`crates/kivact-engine/src/embed.rs`、`crates/kivact-engine/src/contract/host.rs`（`HostError` を読むだけ。変えない）、`crates/arvane/src/embed.rs`、`crates/arvane-shell/src/child.rs`、`crates/arvane-native/src/{wait.rs,port/embed/*,port/win32/{create.rs,window.rs},port/x11/{conn.rs,test_parent.rs,thread.rs,window.rs}}`、`crates/kivact-test/src/parent.rs`、`dev/tests/kivact-public-api/tests/*/embed/*`
- **目的**：USR-15、API-10、ST-21。`kivact::embed` は全 OS で compile。未対応の環境の `App::embed` は `Err(ErrorKind::Unsupported)`。停止は `EmbedNotice::Stopped(EmbedStatus)` で一度だけ知らせる。`embed::Visibility` を `EmbedVisibility` に改名。
- **手順**：
  1. 改名（型を kivact-types で改名し、使う全ファイルを直す）：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates" || exit 1
     perl -pi -e 's/\bpub enum Visibility\b/pub enum EmbedVisibility/; s/\bVisibility\b/EmbedVisibility/g' kivact-types/src/embed.rs
     FILES="kivact-core/src/embed.rs kivact-core/src/embed/embedded.rs kivact-core/src/embed/parent.rs kivact-engine/src/embed.rs arvane/src/embed.rs arvane-shell/src/child.rs arvane-native/src/port/embed/child.rs arvane-native/src/port/embed/resize.rs arvane-native/src/port/embed/surface.rs arvane-native/src/port/win32/create.rs arvane-native/src/port/win32/window.rs arvane-native/src/port/x11/conn.rs arvane-native/src/port/x11/test_parent.rs arvane-native/src/port/x11/thread.rs arvane-native/src/port/x11/window.rs"
     for f in $FILES; do rg -q "\bVisibility\b" $f && perl -pi -e 's/embed::Visibility\b/embed::EmbedVisibility/g; s/\bVisibility::(Shown|Hidden)\b/EmbedVisibility::$1/g' $f; done
     rg -n "\bVisibility\b" $FILES
     ```
     最後の `rg` に残る `Visibility`（型の位置の `: Visibility`、`-> Visibility`、`use ..::{.., Visibility}`）を一つずつ `EmbedVisibility` にする。ただし style の `kivact_types::Visibility`（`use kivact_types::Visibility` や `layout::Visibility`）を使う行は変えない（このファイル群では `kivact-engine/src/embed.rs` 以外に style の `Visibility` は無い。engine の `embed.rs` で両方が出たら、`embed::` の付く方だけを変える）。`kivact-types/src/lib.rs` の `pub use embed::{EmbedOptions, ResizeAnswer, UnhandledKeys};` に `EmbedVisibility` を足す。
     `kivact-core/src/embed.rs` の `pub use kivact_types::embed::{ResizeAnswer, UnhandledKeys, Visibility};` は `pub use kivact_types::embed::{EmbedVisibility, ResizeAnswer, UnhandledKeys};` になる。
  2. 停止の通知に理由を載せる（arvane-shell）：`crates/arvane-shell/src/child.rs`：
     ```rust
     #[derive(Clone, Debug, PartialEq)]
     pub enum EmbedNotice {
         ScaleChanged(f64),
         /// The child stopped: the host destroyed the parent, a platform failure
         /// stopped the App, or it panicked. Sent once, when the child's stop
         /// status leaves running, with the stop at that moment.
         Stopped(ChildStop),
     }
     ```
     `NoticeSink::stopped(&self)` を `stopped(&self, stop: ChildStop)`（`self.send(EmbedNotice::Stopped(stop))`）にする。単体試験 `notices_reach_the_callback_in_order` の `NoticeSink::stopped()` の呼出しを `stopped(ChildStop::Closed)` に、期待の `EmbedNotice::Stopped` を `EmbedNotice::Stopped(ChildStop::Closed)` にする。
     `crates/arvane-native/src/port/embed/child.rs` の 4 か所（`keep_panic`、`request_close`、`window_gone`、`fail`）の
     ```rust
     if left_running {
         self.notices.stopped();
     }
     ```
     を
     ```rust
     if left_running {
         self.notices.stopped(self.stop().expect("invariant: the child left running"));
     }
     ```
     にする（`stop()` は lock を取り直す。上の block で lock は離れている）。同じファイルの試験の `NoticeSink` の利用は compile error に従って `stopped(..)` の引数を足す。
  3. core の通知の型（`crates/kivact-core/src/embed/notice.rs`）を core の enum にする：
     ```rust
     //! Role: What an embedded App tells its host.

     use super::EmbedStatus;

     /// A push notice of an embedded App (`Parent::on_notice`). It runs on the
     /// thread that observed the change; it must not wait or panic.
     #[derive(Clone, Debug, PartialEq)]
     pub enum EmbedNotice {
         /// The applied scale changed for a reason other than the host's
         /// `set_scale` (an operating-system scale change). Keep the logical
         /// size by asking for `logical size * scale`.
         ScaleChanged(f64),
         /// The App stopped, once: closed by the host, failed, or panicked.
         /// [`Embedded::status`](super::Embedded::status) keeps reporting it.
         Stopped(EmbedStatus),
     }
     ```
     `embed/status.rs` に、`Embedded::status` の `match self.host.stop() { .. }` の本体を移した関数を置く：
     ```rust
     /// The status of a child whose engine stop is `stop`.
     pub(crate) fn status_of(stop: Option<kivact_engine::embed::ChildStop>, panic: &kivact_engine::embed::PanicSlot) -> EmbedStatus {
         match stop {
             None => EmbedStatus::Running,
             Some(kivact_engine::embed::ChildStop::Closed) => EmbedStatus::Closed,
             Some(kivact_engine::embed::ChildStop::Failed(error)) => {
                 EmbedStatus::Failed(UiError::new(ErrorKind::Platform, error.to_string(), None))
             }
             Some(kivact_engine::embed::ChildStop::Panicked) => EmbedStatus::Panicked(UiError::new(
                 ErrorKind::Panicked,
                 panic.message().unwrap_or_else(|| "the embedded App panicked".to_owned()),
                 None,
             )),
         }
     }
     ```
     `Embedded::status` は `status_of(self.host.stop(), &self.panic)`。
     `embed/open.rs` の `ChildHooks { on_notice: parent.on_notice, .. }` を、利用者の callback を包む closure にする：
     ```rust
     let user = parent.on_notice.clone();
     let notice_panic = panic.clone();
     let on_notice: Option<Arc<dyn Fn(kivact_engine::embed::EmbedNotice) + Send + Sync>> = user.map(|user| {
         Arc::new(move |notice: kivact_engine::embed::EmbedNotice| match notice {
             kivact_engine::embed::EmbedNotice::ScaleChanged(scale) => user(EmbedNotice::ScaleChanged(scale)),
             kivact_engine::embed::EmbedNotice::Stopped(stop) => {
                 user(EmbedNotice::Stopped(super::status::status_of(Some(stop), &notice_panic)))
             }
         }) as Arc<dyn Fn(kivact_engine::embed::EmbedNotice) + Send + Sync>
     });
     ```
     `Parent.on_notice` の型は `Option<Arc<dyn Fn(EmbedNotice) + Send + Sync>>`（core の `EmbedNotice`）。`PanicSlot: Clone + Send + Sync`（既に `ChildHooks.panic` に clone を渡している）。
  4. 未対応の環境は `Unsupported`：`crates/kivact-engine/src/embed.rs` の `embed_native` の戻りを `Result<EmbeddedHost, EmbedOpenError>` にする：
     ```rust
     /// Why an embedded host did not open.
     #[derive(Debug)]
     pub enum EmbedOpenError {
         /// This platform or build cannot embed a window.
         Unsupported(String),
         /// The platform failed to open it.
         Failed(String),
     }
     ```
     本体の `.map_err(|e| HostError::new(e.to_string()))` を `.map_err(|e| if e.is_unsupported() { EmbedOpenError::Unsupported(e.to_string()) } else { EmbedOpenError::Failed(e.to_string()) })` にする（`ShellError::is_unsupported` は既存）。`crates/kivact-core/src/embed/open.rs` の `Err(error) => Err(UiError::new(ErrorKind::Platform, error.message, None))` を：
     ```rust
     Err(EmbedOpenError::Unsupported(message)) => Err(UiError::new(ErrorKind::Unsupported, message, Some(at))),
     Err(EmbedOpenError::Failed(message)) => Err(UiError::new(ErrorKind::Platform, message, Some(at))),
     ```
     doc の「an unsupported OS return `ErrorKind::Platform`」を「an unsupported OS or a build without the feature returns `ErrorKind::Unsupported`」にする。
  5. 全 OS で warning なし：`crates/arvane-native/src/wait.rs` の `#[cfg(feature = "embed")] pub(crate) const SHUTDOWN_WAIT` を `#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]` にする。
  6. 利用者：`dev/tests/kivact-public-api/tests/*/embed/panic_boundary.rs` の `if notice == EmbedNotice::Stopped {` を `if matches!(notice, EmbedNotice::Stopped(_)) {` にする。`kivact-test/src/parent.rs` と e2e の embed の他のファイルで `EmbedNotice::Stopped` を比べる箇所も同様。`Visibility` を使う箇所は手順 1 の名前に直す。
- **消すもの**：`kivact_types::embed::Visibility`（改名）、`EmbedNotice::Stopped`（引数なし）、`NoticeSink::stopped()`（引数なし）、`HostError` による embed の失敗の運搬。
- **状態の閉じ方**：新しい状態の型は無い（enum だけ）。
- **試験**：`e2e_native` の `embed/contract.rs` に足す：`stopped_notice_carries_the_status_once`：`Parent::on_notice` で受けた notice を `Arc<Mutex<Vec<EmbedNotice>>>` に記録し、`parent_destruction_stops_the_app_without_an_error` と同じ手順で親を壊した後、`Stopped(EmbedStatus::Closed)` がちょうど 1 つ。`panic_boundary.rs` の panic の scenario で `Stopped(EmbedStatus::Panicked(_))` を受ける（既存の期待を `matches!` で強める）。未対応の OS の試験は置かない（CI は Linux だけ）。
- **確認（tier a）**：4.6 節の `W5_CHECK` と：
  ```sh
  cargo check -p arvane-native --all-targets --features embed
  cargo check -p arvane-shell --all-targets
  cargo clippy -p arvane-native -p arvane-shell -p kivact-engine --all-targets --features arvane-native/embed -- -D warnings
  rustup target list --installed | rg -q x86_64-apple-darwin && cargo check -p kivact --features embed --target x86_64-apple-darwin
  cargo test -p arvane-shell
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --features embed --test e2e_native -- embed
  ```
- **完了条件**：`rg -n "embed::Visibility\b|EmbedNotice::Stopped\b[^(]" crates dev` が 0 件。`rg -n "ErrorKind::Platform, error.message" crates/kivact-core/src/embed` が 0 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C14：font は theme が持つ（`App::font`・`App::fonts`・`TestFonts::Only` を消す）

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C13、付属書 S の S09（W2a。`Theme::with_fonts`・`Theme::fonts`。4.3 節 S-REQ-1）
- **所有**：`crates/kivact-core/src/{app.rs,launch/settings.rs,launch/runtime.rs,view/root.rs,view/boundary.rs,view/state.rs,view/requests.rs,platform/env.rs}`、`crates/kivact-test/src/{options.rs,lib.rs,render.rs,screen/driver.rs,screen/gpu_tests.rs}`（`gpu_tests.rs` は W1 で `dev/tests` へ移っていればその path）、`crates/kivact-types/src/asset.rs`（doc の例 1 行）、`App::font`・`App::fonts` の呼出し元（`crates/kivact-devtools/src/main.rs`、`examples/kivact-ui-{basic,pop,game,soft,creative}/src/main.rs`）、`RenderOptions::font` と `TestFonts::Only` の呼出し元（`examples/kivact-ui-*/tests/*.rs`、`dev/tests/kivact-kit-tests/tests/e2e_kits/theme_conformance.rs`、`dev/tests/kivact-public-api/tests/*/{text_style.rs,offscreen.rs,text_effects.rs,images_animated.rs,launch_fonts.rs}`）（呼出し元の `dev/` は `dev/tests/kivact-public-api`・`dev/tests/kivact-kit-tests`・`dev/tests/gpu-interop-producer`・`dev/bench` を含む。W5 では kit・catalog・`kivact-kit-tests` も compile するので直す。INC-33）
- **目的**：USR-12、設計書 5.6。font は theme が持つ（12 章 API-13）：`Theme::with_fonts(impl IntoIterator<Item = impl Into<Asset>>)` の `Asset`（path・`Vec<u8>`・共有 bytes・static bytes。`'static` は要らない）を theme が持ち、その font を、`App::theme(t)`・`provide_theme(..)`・`kivact::test::RenderOptions::{with_theme, theme}` が登録する。`App::font`・`App::fonts`・`RenderOptions::font`・`TestFonts::Only` を消す。
- **手順**：
  1. 登録の一か所：`view/requests.rs` に：
     ```rust
     /// Register the fonts of `theme` the App has not registered yet (by asset
     /// identity: the path, the static address and length, or the shared
     /// allocation), located at `at`. The first layout after this call sees
     /// them (the engine registers a font when it is written). A file that is
     /// not a usable font is reported as `ErrorKind::Asset` at `at`.
     pub(crate) fn register_theme_fonts(rt: &Runtime, theme: &kivact_style::Theme, at: &'static Location<'static>) {
         let fonts: Vec<Asset> = {
             let mut seen = ViewState::of(rt).registered_fonts.borrow_mut();
             theme.fonts().iter().filter(|asset| seen.insert((*asset).clone())).cloned().collect()
         };
         for font in fonts {
             let waiter = Waiter::Report { owner: None, at, failed: ErrorKind::Asset };
             request_effect(rt, HostRequest::RegisterFont(font), waiter);
         }
     }
     ```
     `register_app_fonts` を消す（その本体はここに入った）。`ViewState` に `pub(crate) registered_fonts: RefCell<std::collections::HashSet<Asset>>,`（doc：`/// The theme fonts already registered, by asset identity.`）を足す（`Asset` は `Eq + Hash` で、等しさは同一性：path、static の address と長さ、共有の allocation）。
  2. App の theme：
     - `app.rs` の `App::theme` に `#[track_caller]` を付ける。`launch/settings.rs` の `AppSettings` に `pub(crate) theme_at: &'static Location<'static>,` を足し（`App::new` に `#[track_caller]` を付けて `Location::caller()`、`App::theme` で `Location::caller()` に上書き）、`AppSettings.fonts: Vec<(Asset, &'static Location<'static>)>` を消す。
     - `launch/runtime.rs` の `start_mount` の `requests::register_app_fonts(rt, std::mem::take(&mut options.settings.fonts));` と、その上の comment 2 行を消す。
     - `view/root.rs` の `install_app_contexts` の引数 `settings` の組に `theme_at` を足し（`App::into_view_settings` が返す組に `self.settings.theme_at` を足す）、theme の行の前に：
       ```rust
       if let Some(fixed) = theme.fixed() {
           crate::view::requests::register_theme_fonts(rt, &fixed, theme_at);
       }
       ```
       theme が reactive のとき（`fixed()` が `None`）は手順 3 の effect が登録する。
  3. theme の切替え：`view/boundary.rs` の `install_theme(owner, theme)`（C01）を、theme が変わるたびに font を登録する形にする：
     ```rust
     pub(crate) fn install_theme(owner: Owner, theme: Value<Theme>, at: &'static Location<'static>) {
         let rt = owner.runtime();
         let weak = Rc::downgrade(&rt);
         owner.effect(
             kivact_runtime::EffectKind::Render,
             move || {
                 let Some(rt) = weak.upgrade() else { return };
                 theme.with(|t| crate::view::requests::register_theme_fonts(&rt, t, at));
             },
             at,
         );
         let theme = crate::view::devtools_hooks::wrap_theme(&rt, owner, theme);
         kivact_style::internal::install_theme(owner, theme);
     }
     ```
     （`owner.effect(kind, f, at)` は `window_view.rs` が使う既存の runtime の API。）呼出し元（`provide_theme` と `install_app_contexts`）に `at` を渡す（`provide_theme` は自分の `Location::caller()`、root は `theme_at`）。登録済みの font は手順 1 が飛ばすので、固定の theme の二重登録は起きない。
  4. `App::font` と `App::fonts` を消す（`app.rs`。C11 で `AppOptions` から `App` に移った 2 つの method）。doc の参照を直す：`platform/env.rs` の `font_families` の doc の `[`crate::App::font`]` を `` `Theme::with_fonts` ``、`view/requests.rs` の「Register the App's fonts (`App::font`)」の doc は手順 1 で消える、`crates/kivact-types/src/asset.rs` の doc の例 `App::new("Editor").font(include_bytes!(..));` を `Theme::default().with_fonts([include_bytes!("../assets/Inter-Regular.ttf")]);` にする。呼出し元：
     - `crates/kivact-devtools/src/main.rs` の `.fonts(kivact_devtools::fonts::fonts().iter().copied())` を消し、その App の `.theme(X)` の `X` を `X.with_fonts(kivact_devtools::fonts::fonts().iter().copied())` にする（`.theme(..)` が無ければ `.theme(kivact::style::Theme::default().with_fonts(kivact_devtools::fonts::fonts().iter().copied()))` を足す。`&'static [u8]` は `Into<Asset>`）。
     - `examples/kivact-ui-basic/src/main.rs`、`examples/kivact-ui-pop/src/main.rs`、`examples/kivact-ui-game/src/main.rs`（`.fonts(fonts().iter().copied())`）と `examples/kivact-ui-soft/src/main.rs`、`examples/kivact-ui-creative/src/main.rs`（`for &font in fonts() { app = app.font(font); }` の loop）も同じ形にする（`fonts()` は各 crate の `fonts.rs` の `-> &'static [&'static [u8]]` なので `.with_fonts(fonts().iter().copied())`。loop と `let mut app` の `mut` を消す）。
  5. kivact-test：
     - `options.rs`：`RenderOptions::font` を消す。`TestFonts::Only` を消す（`#[default] Bundled` と `System` が残る）。`RenderOptions::theme` と `RenderOptions::with_theme` に `#[track_caller]` を付ける（登録の失敗が利用者の試験の行に located になる）。
     - `into_host(self, fonts: Vec<(Asset, &Location)>)` の引数 `fonts` と、`for (font, _) in fonts { app = app.font(font); }` を消す。App の theme が font を持ち（`kivact_core::__test::app_theme_has_fonts(&app)`）`self.fonts` が `Bundled` のとき、engine の test host の font を `kivact_engine::testing::TestFonts::Only(Vec::new())`（font の無い host）にする。theme の font は App が最初の layout の前に登録する（手順 2）ので、旧 `TestFonts::Only(fonts)` と同じく theme の font だけで形を作る。壊れた font を host の作成に渡さないので、host の作成は失敗しない。`System` のときは `System` のまま。
     - `screen/driver.rs` の `Screen::start(options, fonts, root, at)` の引数 `fonts` を消し、`render.rs` の `Screen::start(options, Vec::new(), ..)` と `gpu_tests.rs` の 3 か所の `Screen::start(options, Vec::new(), ..)` から `Vec::new(),` を消す。
     - C11 の `app_theme_has_fonts` の本体を次にする：
       ```rust
       /// Whether the App's fixed theme carries fonts (the test host then starts without any).
       pub fn app_theme_has_fonts(app: &App) -> bool {
           app.theme.fixed().is_some_and(|theme| !theme.fonts().is_empty())
       }
       ```
     - engine の `kivact_engine::testing::TestFonts::Only` は engine の試験の API として残す（kivact-test の公開から消えるだけ）。`lib.rs` の doc の「with `TestFonts::Only` (the bundled font has no CJK)」を「with a theme that has the fonts (`Theme::with_fonts`)」にする。
  6. `TestFonts::Only` と `RenderOptions::font` の呼出し元：
     - 形 A：`RenderOptions::with_theme(T)` … `.fonts(TestFonts::Only(fonts().iter().map(|&bytes| bytes.into()).collect()))` → `RenderOptions::with_theme(T.with_fonts(fonts().iter().copied()))` にし `.fonts(..)` を消す。
     - 形 B：`RenderOptions::from_env().fonts(TestFonts::Only(fonts()...))` → `RenderOptions::from_env().theme(kivact::style::Theme::default().with_fonts(fonts().iter().copied()))`。同じ式に `.theme(T)` が既にあれば `T.with_fonts(fonts().iter().copied())` にする。
     - 形 C：`.fonts(TestFonts::Only(vec![kivact::view::Asset::from(INTER)]))`（`text_style.rs`、`theme_conformance.rs`、`offscreen.rs`）と `.font(Asset::from(INTER))`（`text_effects.rs`、`images_animated.rs`）→ 形 B と同じく theme に `.with_fonts([INTER])` を付ける（`TUFFY` は `[TUFFY]`）。
     - `use kivact::test::{.., TestFonts, ..}` と `use kivact::view::Asset` の未使用の import を消す。
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     rg -n "TestFonts::Only|RenderOptions::[a-z_]+\(\)[^;]*\.font\(|\.font\(Asset|app\.font\(|\.fonts\(fonts" crates dev examples templates --type rust
     ```
     が 0 件になるまで直す（engine の `testing` の中を除く）。
  7. `launch_fonts.rs`（`e2e_core`）の 2 本は `App::font` の契約の試験なので、同じ契約を theme の font で確かめる試験に書き換える（期待は緩めない）：
     - helper：
       ```rust
       /// A screen whose App theme carries `fonts`.
       #[track_caller]
       fn screen<V: IntoView>(fonts: Vec<Asset>, root: impl FnOnce() -> V + 'static) -> Screen {
           kivact::test::render_with(RenderOptions::default().theme(Theme::default().with_fonts(fonts)), root)
       }
       static NOT_A_FONT: &[u8] = b"wOFF\0\x01\0\0";
       /// Tuffy's file in the repository (the same bytes as `TUFFY`).
       const TUFFY_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../crates/arvane/assets/fonts/Tuffy.ttf");
       ```
     - `app_fonts_register_before_the_first_layout` → `theme_fonts_register_before_the_first_layout`：`screen(vec![Asset::from(TUFFY)], root)` と `screen(Vec::new(), root)` で、旧と同じ 3 つの assert（`with` に asset の error なし、`without` に 1 つ、幅が違う）。
     - `a_rejected_app_font_is_located_at_its_call` → `a_rejected_theme_font_is_located_at_the_theme_call`：`let line = line!() + 1; let s = screen(vec![Asset::from(NOT_A_FONT)], || text("x"));` の後、旧と同じ 3 つの assert（error が 1 つ、"TTF or OTF" を含む、location の行が `line`）。`screen` は `#[track_caller]`、`RenderOptions::theme` と `App::theme` も `#[track_caller]` なので、位置は試験の行になる。
     - 新しい試験 `a_switched_theme_registers_its_fonts`：`let theme = signal(Theme::default());` で `provide_theme(theme, || text("WWWW").test_id("t").style(style().font_family("Tuffy")))` を mount し、`platform::font_families()` に `"Tuffy"` が無い。`theme.set(Theme::default().with_fonts([TUFFY]))` と `run_until_idle` の後に現れ、幅が変わる。
     - 新しい試験 `theme_fonts_from_owned_bytes_and_a_path_register`（12 章 API-13：`'static` でない font）：`screen(vec![Asset::from(std::fs::read(TUFFY_PATH).expect("the font file is readable"))], root)` と `screen(vec![Asset::from(std::path::PathBuf::from(TUFFY_PATH))], root)` の両方で、`theme_fonts_register_before_the_first_layout` と同じく asset の error が無く、`"Tuffy"` の text の幅が font なしの screen と違う。
- **消すもの**：`App::font`、`App::fonts`、`AppSettings.fonts`、`register_app_fonts`、`RenderOptions::font`、`kivact::test::TestFonts::Only`、`Screen::start` の `fonts` 引数、`into_host` の `fonts` 引数、試験 `app_fonts_register_before_the_first_layout`・`a_rejected_app_font_is_located_at_its_call`（同じ契約の theme の試験に置き換え）。
- **状態の閉じ方**：`ViewState.registered_fonts` は「見た」集合（単調に増える表。6.1 の対象の型ではない）。`AppSettings.theme_at` は値の field。
- **試験**：手順 7 の 4 本。`text_style`、`text_effects`、`images_animated`、`offscreen` の既存の試験が期待を変えずに通る。
- **確認（tier a）**：4.6 節の `W5_CHECK` と：
  ```sh
  cargo xtask check --no-write --crate kivact-core
  cargo xtask check --no-write --crate kivact-test
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- launch_fonts text_style text_effects images_animated offscreen
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits -- theme_conformance
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-basic-catalog --test catalog
  ```
- **完了条件**：`rg -n "TestFonts::Only|fn font\(mut self|fn fonts\(mut self|register_app_fonts" crates/kivact-core crates/kivact-test crates/kivact dev examples templates` が 0 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C15：`commit_focused_edit`、`PointerEvent::target_size`、`spacer` の最小の大きさ

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C14
- **所有**：新 `crates/kivact-core/src/view/focus_commit.rs`、`crates/kivact-core/src/view/{view.rs,input.rs,state.rs,requests.rs,elements/container.rs,mount/part_pointer.rs}`、`crates/kivact-core/src/client/dispatch/{pointer.rs,text.rs}`、`crates/kivact-core/src/launch/turn.rs`（R06 が移した turn の関数）、`view/window_view.rs`（window を消すときに待ちを終える 1 か所）、`dev/tests/kivact-public-api/tests/*/{text_editing.rs,input_platform.rs,layout.rs}`（試験を足す）
- **目的**：USR-05：focus を持つ control の「focus を失ったときの確定」を実行し、その結果が state に書かれたら完了する future。USR-16：handler を付けた node の大きさ（`pos` と同じ座標系）。USR-17：`spacer()` は `min_width(0).min_height(0)` を含む。
- **手順（USR-05）**：仕組み：今の確定は、engine の `FocusChanged { node: None/他 }` の配送で `FocusKeys.focus` が変わり、`text_edit/draft.rs` の `effect_on(focused, ..)`（user effect）が同じ turn の user effect の段で `Drafting::commit` を走らせる、というもの。engine の `Writer::blur(window)` は `change_focus(None)` を通るので、IME の未確定の文字を先に `TextChanged` として出し、その後に `FocusChanged { node: None }` を出す。よって「blur を送り、`FocusChanged` が配送された turn の user effect が終わった後」に future を終える。同じ turn の中で終えると、`settle_async` が user effect より先に task を進めて確定前の state を見る（delivery の途中で `Completion` を完了すると、その turn の `settle_async` で task が起きる）。そこで配送の turn では印だけを付け、**次の turn の始め**に完了する。
  1. `view/state.rs` に：
     ```rust
     /// One `commit_focused_edit` call: the window whose focus it removes.
     pub(crate) struct BlurRequest {
         pub(crate) window: WindowId,
         pub(crate) stage: BlurStage,
         pub(crate) done: Completion<()>,
     }

     #[derive(Clone, Copy, PartialEq, Eq)]
     pub(crate) enum BlurStage {
         /// Waiting for the next commit, which sends the blur to the engine.
         Queued,
         /// Sent; waiting for the engine's `FocusChanged` of the window.
         Sent,
         /// Delivered: the next turn completes it, after this turn's user
         /// effects committed the edit.
         Delivered,
     }
     ```
     `Requests` に `pub(crate) blurs: Vec<BlurRequest>,` を足す。
  2. 新 `view/focus_commit.rs`：
     ```rust
     //! Role: Commit the focused control's edit as leaving it would, and wait for the commit.

     use std::future::Future;
     use std::panic::Location;

     use kivact_runtime::{Completion, Owner, require_effectful};

     use crate::view::state::{BlurRequest, BlurStage, ViewState, current_window};

     /// Run the focused control's "commit on focus loss" (a text input's draft
     /// goes to its value, a number field parses and clamps) and finish once
     /// that commit is written to the state. The control loses the focus; focus
     /// it again with `NodeRef::focus` when the app wants it back. Nothing is
     /// focused in this window: it finishes at once.
     ///
     /// Call it from a handler or a task, for example a Save shortcut:
     /// `spawn(async move { commit_focused_edit().await; save(model.get()) })`.
     #[track_caller]
     pub fn commit_focused_edit() -> impl Future<Output = ()> + 'static {
         let at = Location::caller();
         require_effectful("commit_focused_edit");
         let owner = Owner::running("commit_focused_edit", at);
         let rt = owner.runtime();
         let window = current_window(owner);
         let done = Completion::new();
         let state = ViewState::of(&rt);
         let focused = state.focus_keys(window).is_some_and(|keys| keys.focus.get_untracked().is_some());
         if focused {
             state.requests.borrow_mut().blurs.push(BlurRequest { window, stage: BlurStage::Queued, done: done.clone() });
             rt.request_turn();
         } else {
             done.complete(());
         }
         async move { done.await }
     }
     ```
     `Completion::clone` が無ければ、`BlurRequest.done` に入れる値と future の値を `Completion::new()` の戻り（`kivact_runtime::Completion` の API に従う。完了する側と待つ側の 2 つの handle を返す形なら、待つ側を future に、完了する側を `BlurRequest` に置く）。`Owner::running` は R25 が `Owner::ambient` から改名した名前。
     `view.rs` に `mod focus_commit;` と `#[doc(inline)] pub use focus_commit::commit_focused_edit;` を足す。
  3. 送る：`view/requests.rs` の `flush_focus` の末尾（commit の中）に：
     ```rust
     let queued: Vec<WindowId> = {
         let mut requests = ViewState::of(rt).requests.borrow_mut();
         let mut windows = Vec::new();
         for blur in requests.blurs.iter_mut().filter(|b| b.stage == BlurStage::Queued) {
             blur.stage = BlurStage::Sent;
             if !windows.contains(&blur.window) {
                 windows.push(blur.window);
             }
         }
         windows
     };
     for window in queued {
         let focused = ViewState::of(rt).focus_keys(window).is_some_and(|k| k.focus.get_untracked().is_some());
         if focused {
             ViewState::tree(rt).engine().blur(window);
         } else {
             mark_blur_delivered(rt, window);
         }
     }
     ```
     ```rust
     /// The focus of `window` left: its blur requests complete on the next turn.
     pub(crate) fn mark_blur_delivered(rt: &Runtime, window: WindowId) {
         let mut any = false;
         for blur in ViewState::of(rt).requests.borrow_mut().blurs.iter_mut() {
             if blur.window == window && blur.stage == BlurStage::Sent {
                 blur.stage = BlurStage::Delivered;
                 any = true;
             }
         }
         if any {
             rt.request_turn();
         }
     }

     /// Complete the blur requests delivered in an earlier turn. `true` when one
     /// completed (a task woke, so the turn is not idle).
     pub(crate) fn complete_delivered_blurs(rt: &Runtime) -> bool {
         let delivered: Vec<BlurRequest> = {
             let mut requests = ViewState::of(rt).requests.borrow_mut();
             let (done, keep): (Vec<_>, Vec<_>) = std::mem::take(&mut requests.blurs)
                 .into_iter()
                 .partition(|b| b.stage == BlurStage::Delivered);
             requests.blurs = keep;
             done
         };
         let woke = !delivered.is_empty();
         for blur in delivered {
             blur.done.complete(());
         }
         woke
     }
     ```
     （`Writer::blur(&self, window: WindowId)` は engine の既存の公開 method。core はこれまで呼んでいない。）
  4. 配送：`client/dispatch/text.rs` の `focus_changed(rt, window, target, visible)` の末尾に `if target.is_none() { crate::view::requests::mark_blur_delivered(rt, window); }` を足す。
  5. 次の turn の始め：turn の関数で、閉じる要求を取る行（`take_close_request(..)`）の直後、idle の判定の前に：
     ```rust
     // A blur delivered last turn: its user effects have committed the edit.
     let woke = crate::view::requests::complete_delivered_blurs(&rt);
     ```
     を置き、idle の条件に `!woke &&` を足す（R06 の idle の判定の前に論理和として足す：`let idle = !woke && ..`）。
  6. window が閉じたとき：`view/window_view.rs` の `WindowSlot::close` の `ViewState::of(rt).remove_window(id);` の次に、その window の `blurs` をすべて `Delivered` にする（`mark_blur_delivered` を `Sent` に限らず全段に効く `pub(crate) fn release_blurs(rt, window)` として `requests.rs` に置き、ここで呼ぶ）。
- **手順（USR-16）**：
  7. `view/input.rs` の `PointerEvent` に field `pub(crate) target_size: Size,`（`crate::geometry::Size`）を足し、method を足す：
     ```rust
     /// The size of the node whose handler receives this event, in the
     /// coordinates of [`pos`](Self::pos) (its border box, before its transform).
     pub fn target_size(&self) -> Size {
         self.target_size
     }
     ```
     手書きの `PartialEq` は `routed` だけを比べたまま。
  8. `view/mount/part_pointer.rs` の `PointerEvent::from_routed(routed)` を `from_routed(routed, target_size: Size)` にする（struct literal に `target_size`）。`client/dispatch/pointer.rs` の `pointer` の `let e = PointerEvent::from_routed(routed);` を：
     ```rust
     let target_size = ViewState::tree(rt)
         .reader()
         .box_model(node)
         .map(|b| Size::new(b.border.width, b.border.height))
         .unwrap_or_default();
     let e = PointerEvent::from_routed(routed, target_size);
     ```
     にする（`node` は engine が選んだ受け手で、`pos` はその node の座標）。`event_for(..)` と `Capture.last`・`Gestures.last` の保存は event を丸ごと写すので変えない。
  9. `DragUpdate::target_size()` は付属書 U（4.5 節 U-REQ-2）。
- **手順（USR-17）**：
  10. `view/elements/container.rs` の `spacer`：
      ```rust
      /// Flexible empty space: it grows, and shrinks to nothing.
      #[track_caller]
      pub fn spacer() -> Node {
          node().style(style().grow(1).min_width(0).min_height(0))
      }
      ```
      （W6 で付属書 S が `grow` を `flex_grow` に改名する。）
- **消すもの**：なし。
- **状態の閉じ方**：`BlurRequest` は `BlurStage`（enum）と `Completion`。`Requests` は C03 の形に `Vec` を一つ足すだけ（`Vec` は対象の型ではない）。
- **試験**：
  - `e2e_core` の `text_editing.rs`：`commit_focused_edit_commits_a_draft_and_resolves_after_the_value_is_written`：`text_input(name).on_draft_commit(move |v| name.set(v))`（W5 の時点の名前。U が `on_commit` に改名する）と、`Ctrl+S` の `on_key` で `spawn(async move { commit_focused_edit().await; saved.set(name.get()) })`。field に focus して "Ada" を入力し（commit 前は `name` が空）、`Ctrl+S` を押して `run_until_idle` の後、`saved.get() == "Ada"`。IME の未確定の文字（`screen` の IME の操作で preedit "x"）の後でも同じく確定される。
  - 同じファイル：`commit_focused_edit_without_focus_resolves_at_once`：focus なしで呼ぶと次の `run_until_idle` で完了する。
  - `e2e_core` の `input_platform.rs`：`pointer_events_know_the_size_of_their_target`：幅 120・高さ 40 の node の `on_pointer` で受けた event の `target_size()` が `Size::new(120.0, 40.0)`、子の node の上を押しても（子は listener でない）同じ。
  - `e2e_core` の `layout.rs`：`a_spacer_shrinks_to_nothing_in_a_full_row`：幅 100 の `row((node().style(style().width(100)), spacer()))` で spacer の幅が 0。
- **確認（tier a）**：4.6 節の `W5_CHECK` と：
  ```sh
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- text_editing input_platform layout
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- ui_field interaction_drag_edit
  ```
- **完了条件**：上の 4 本の新しい試験が通る。`rg -n "pub fn commit_focused_edit|pub fn target_size" crates/kivact-core/src/view` が 2 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C16：開くときに作る content の引数をそろえ、小さな API の誤りを直す

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C15
- **所有**：`crates/kivact-core/src/view/{flow/error_boundary.rs,descriptor/factory.rs}`、`crates/kivact-ui/src/{async_view.rs,text_input.rs}`、呼出し元
- **目的**：API-16：開いたときに作る content は引数なしなら `impl IntoViewFn<V>`、引数ありなら `impl Fn(Arg) -> V`。`provide`・`scope` は `FnOnce`。API-08：`async_view(source, ready).loading(f).fallback(f)` の `loading` が `IntoViewFn` を受ける。API-14：`text_area` に `#[track_caller]`。
- **手順**：
  1. `crates/kivact-ui/src/async_view.rs`：`pub fn loading<V: IntoView>(mut self, f: impl Fn() -> V + 'static) -> Self` を `pub fn loading<V: IntoView>(mut self, f: impl kivact_core::view::IntoViewFn<V>) -> Self` にし、本体を `let f = f.into_view_fn(); self.loading = Rc::new(move || f.call(()));` にする（`ViewFn<()>::call(())` は `window_view.rs` が使う既存の API）。`fallback` は `Fn(UiError, Callback<()>) -> V`（付属書 R の RC-35 で `Retry` が `Callback<()>` になっている）のまま。
  2. `crates/kivact-core/src/view/flow/error_boundary.rs` の `fallback` は `Fn(UiError, Callback<()>) -> V`（R の後の形）。変更なし。`error_boundary(child: impl IntoViewFn<V>)` は既に正しい。
  3. `crates/kivact-core/src/view/descriptor/factory.rs` の module doc に規則を書く：`//! Content built when something opens takes `impl IntoViewFn<V>` (no argument) or `impl Fn(Arg) -> V` (an argument). `provide` and `scope` run their child once and take `FnOnce`.`
  4. `crates/kivact-ui/src/text_input.rs` の `pub fn text_area(` の前に `#[track_caller]` を付ける。
- **消すもの**：`AsyncView::loading` の `Fn() -> V`。
- **状態の閉じ方**：なし。
- **試験**：`e2e_ui` の `async_tests.rs`（`e2e_core` に分かれていればそちら）の `loading` を使う試験が通る。足す試験は無い。
- **確認（tier a）**：4.6 節の `W5_CHECK` と `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- async_tests`。
- **完了条件**：`rg -n "fn loading<V: IntoView>\(mut self, f: impl Fn\(\)" crates` が 0 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C17：`Offscreen::render(f64, f64)` と `.scale(f64)`

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C16
- **所有**：`crates/kivact-offscreen/src/lib.rs`、`Offscreen::render` の呼出し元（`crates/kivact-ui-basic/src/plot/paint.rs`、`dev/tests/kivact-public-api/tests/offscreen.rs`、`dev/tests/kivact-public-api/tests/*/{offscreen.rs,offscreen_svg.rs}`、`docs` を除く全 crate）（呼出し元の `dev/` は `dev/tests/kivact-public-api`・`dev/tests/kivact-kit-tests`・`dev/tests/gpu-interop-producer`・`dev/bench` を含む。W5 では kit・catalog・`kivact-kit-tests` も compile するので直す。INC-33）
- **目的**：RC-29：`render(u32, u32)` と `render_svg(f64, f64)` の不一致をやめ、両方とも論理の大きさ `(width: f64, height: f64)`、画素の密度は `.scale(f64)`。
- **手順**：
  1. `Offscreen` に field `scale: f64` を足し（`new` で `1.0`）、builder を足す：
     ```rust
     /// Pixels per logical pixel of the images [`render`](Self::render) makes
     /// (default 1). The drawing keeps its logical coordinates; a scale of 2
     /// makes an image twice as wide and high. `render_svg` ignores it.
     pub fn scale(mut self, scale: f64) -> Self {
         self.scale = scale;
         self
     }
     ```
  2. `render` の署名と本体：
     ```rust
     #[track_caller]
     pub fn render(&mut self, width: f64, height: f64, paint: impl FnOnce(&mut Draw, Size)) -> Result<Pixels> {
         let at = Location::caller();
         let valid = width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0
             && self.scale.is_finite() && self.scale > 0.0;
         let pixels = (
             (width * self.scale).round().min(f64::from(u32::MAX)) as u32,
             (height * self.scale).round().min(f64::from(u32::MAX)) as u32,
         );
         if !valid || !OffscreenRenderer::accepts_size(pixels.0, pixels.1) {
             return Err(failed(
                 ErrorKind::Message,
                 format!("invalid or excessive image size {width}x{height} at scale {}", self.scale),
                 at,
             ));
         }
         let size = Size::new(width, height);
         let scale = self.scale;
         let Recorded { scene, inherited, .. } = self.record(size, |draw, size| {
             if scale == 1.0 {
                 paint(draw, size);
             } else {
                 let layer = kivact_style::draw::Layer::new().transform(kivact_style::transform::Transform::default().scale(scale));
                 draw.layer(layer, |draw| paint(draw, size));
             }
         }, at)?;
         let image = self.engine.render(&Arc::new(scene), pixels, &inherited).map_err(|error| match error {
             OffscreenError::Dropped(_) => rejected(at),
             OffscreenError::Failed(message) => failed(ErrorKind::Asset, message.to_string(), at),
         })?;
         Ok(Pixels { width: image.width, height: image.height, rgba: image.rgba.into() })
     }
     ```
     `Draw::layer` の transform は canvas の原点のまわり（`kivact-style/src/transform.rs` の module doc「about the canvas origin」）。`Transform::default().scale(..)` の名前が違えば `kivact_style::transform::Transform` の `scale` を持つ constructor に合わせる。型の doc の「Coordinates are output pixels at scale 1」を「Coordinates are logical pixels; [`scale`](Self::scale) sets the pixels per logical pixel」にし、doc の例の `render(640, 360, ..)` を `render(640.0, 360.0, ..)` にする。
  3. 呼出し元：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     rg -n "\.render\(\s*[0-9a-z_]+\s*,\s*[0-9a-z_]+\s*,\s*\|" crates dev examples templates --type rust
     ```
     の各箇所で、`u32` の引数を `f64` にする（整数の literal は `640.0` に、`u32` の変数は `f64::from(x)` に）。`render(w * 2, h * 2, ..)` のように画素を倍にしていた箇所は `.scale(2.0)` と論理の大きさにする。
- **消すもの**：`render(u32, u32)`。
- **状態の閉じ方**：`Offscreen { engine, theme, scale }` に新しい対象の field は無い（`f64` は数えない）。
- **試験**：`dev/tests/kivact-public-api/tests/offscreen.rs` の 3 本を新しい署名で書き換える（期待値は同じ）。足す：`a_scale_of_two_doubles_the_pixels_and_keeps_the_drawing`：`Offscreen::new().scale(2.0).render(10.0, 5.0, |d, size| { assert_eq!(size, Size::new(10.0, 5.0)); d.fill_rect(Rect::new(0.0, 0.0, 5.0, 5.0), rgb(0xff0000)) })` の `Pixels` が 20×10 で、(9, 9) が赤、(11, 1) が透明。
- **確認（tier a）**：4.6 節の `W5_CHECK` と：
  ```sh
  cargo test -p kivact-offscreen --doc
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test offscreen
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- offscreen
  ```
- **完了条件**：`rg -n "width: u32,\s*$" crates/kivact-offscreen/src/lib.rs` が 0 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C18：prelude を 5.11 節の一覧にし、全利用者に import を足す。例と雛形の工程の痕跡を消す

- **wave / lane / 前提**：W5a の鎖 / 鎖 / C17（C の鎖の最後。この後に付属書 E・A の破壊的な unit と掃引が続く）
- **所有**：`crates/kivact/src/{prelude.rs,lib.rs}`、prelude の全利用者（`crates/`、`dev/`、`examples/`、`templates/`）、`examples/kivact/*/src/main.rs`、`templates/basic/src/main.rs`（呼出し元の `dev/` は `dev/tests/kivact-public-api`・`dev/tests/kivact-kit-tests`・`dev/tests/gpu-interop-producer`・`dev/bench` を含む。W5 では kit・catalog・`kivact-kit-tests` も compile するので直す。INC-33）
- **目的**：API-05（prelude を 5.11 節の一覧に）、API-09（例・雛形の `xtask-allow` と `#![warn(redundant_imports)]` と `//! Role:` を消す）、API-19 の facade の path（`kivact::ui`・`kivact::interaction` は crate の再公開のまま。prelude に kit 用の名前を入れない）。
- **手順**：
  1. `crates/kivact/src/prelude.rs` を次にする（W5 の時点で存在する名前だけ。後の wave で足す名前は 4.3・4.5 節）：
     ```rust
     //! Role: Everyday names: what the introductory guides use without a path.
     //!
     //! Everything else is imported from its module (`kivact::ui::data_table`,
     //! `kivact::state::Machine`, ...). No name is exported from two modules.

     pub use crate::{App, Prop, UiError, bind, component};

     pub use crate::state::{
         Callback, Signal, Trigger, Value, Var, binding, callback, effect, effect_on, memo,
         on_cleanup, on_mount, provide, signal, trigger, untrack, use_context, var,
     };

     pub use crate::view::{
         CheckedState, Element, Handled, IntoView, Key, KeyEvent, MouseButton, NodeRef,
         PointerEvent, PointerPhase, Role, View, ViewFn, WheelEvent, canvas, column, dynamic,
         error_boundary, for_each, for_index, grid, icon, image, node, node_ref, row, scroll, show,
         show_some, spacer, stack, text, virtual_list, window,
     };

     pub use crate::style::{
         Align, Color, Cursor, Justify, Overflow, Position, Style, TextAlign, Theme, ThemeMode, em,
         pct, provide_theme, rgb, style, use_theme,
     };

     pub use crate::draw::{Draw, Path, Shadow, Stroke};

     pub use crate::geometry::{Point, Rect, Size};

     pub use crate::task::{Action, AsyncState, Resource, action, resource, sleep, spawn};

     pub use crate::ui::{
         ButtonVariant, ControlSize, async_view, button, checkbox, dialog, field, list, menu,
         radio_group, select, slider, switch, tabs, text_area, text_input, toast_region, tooltip,
     };

     pub use crate::interaction::{ListItem, MenuEntry, Toast};

     pub use crate::i18n::message;
     ```
     facade の `lib.rs`：`pub use kivact_macros::component;` を `pub use kivact_macros::{Prop, component};` にし、`bind` を root に出す：`pub use kivact_core::state::bind;`（`bind!` は runtime の `#[macro_export]`。`kivact::bind` で呼べるようにする）。facade の module doc の例（`fn counter(initial: i32) -> impl IntoView`）を `#[component] fn counter(initial: i32) -> impl Element` にする。
     名前が W5 の時点で別の path にある場合（`provide_theme` が `kivact::style` に無く `kivact::view` にある、`Shadow` が `kivact::draw` に無い、`ListItem` が `kivact::interaction` に無い、など）は、実在する path から出す（`rg -n "pub use.*\bNAME\b" crates/kivact-core/src crates/kivact-ui/src crates/kivact-interaction/src` で探す）。同じ名前を二つの path から出さない。
  2. 利用者に import を足す。次の表の名前は prelude から消えた。prelude を使うファイルで compile error「cannot find (type|function|value|macro|trait) `NAME`」が出たら、右の path の `use` をそのファイルに足す。
     | 名前 | 足す `use` |
     |---|---|
     | `IntoValue`、`IntoText`、`ItemKey`、`PropValue`、`constant_value` | `kivact::state::NAME` |
     | `EditPhase`、`InputPurpose`、`Modifiers`、`Node`、`Orientation`、`Placement`、`RowHeight`、`ScrollAxes`、`WindowMode`、`WindowPlacement`、`drawable` | `kivact::view::NAME` |
     | `scope` | `kivact::custom::scope` |
     | `ColorToken`、`ThemeColor`、`use_tokens` | `kivact::style::NAME`（W6 で付属書 S が消す） |
     | `Easing`、`Spring`、`Transition`、`animated`、`frame_loop`、`timeline` | `kivact::motion::NAME` |
     | `Transform` | `kivact::draw::Transform`（W6 で付属書 S が `kivact::style` へ移す） |
     | `blocking` | `kivact::task::blocking` |
     | `DialogMode`、`DismissOn`、`IntoSelection`、`Selection`、`Tone`、`toaster` | `kivact::interaction::NAME` |
     | `DataColumn`、`PageItem`、`StepStatus`、`StepperItem`、`breadcrumb`、`button_group`、`combobox`、`data_table`、`navigation_menu`、`pagination`、`selection_field`、`stepper`、`toggle_group`、`toolbar`、`calendar`、`date_input`、`date_range_input`、`date_time_input`、`time_input`、`item_list`、`item_row`、`table`、`table_body`、`table_cell`、`table_footer`、`table_head`、`table_header`、`table_row`、`tree` | `kivact::ui::NAME` |
     手順（package ごと）：
     ```sh
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
     for p in kivact-ui-basic kivact-ui-pop kivact-ui-game kivact-ui-creative kivact-ui-soft kivact-ui-basic-catalog kivact-ui-pop-catalog kivact-ui-game-catalog kivact-ui-creative-catalog kivact-ui-soft-catalog kivact-devtools kivact-public-api-tests kivact-bench kivact-template-basic kivact-example-counter kivact-example-form kivact-example-list kivact-example-async kivact-example-component; do
       cargo check -p $p --all-targets --message-format short 2>&1 | rg "cannot find (type|function|value|macro|trait|struct|enum) \`(\w+)\`" -o -r '$2' | sort -u > /tmp/missing-$p.txt
       echo "$p: $(tr '\n' ' ' < /tmp/missing-$p.txt)"
     done
     ```
     （`/tmp` の代わりに作業者の scratchpad を使う。）出た名前を表の path で、error の出たファイルに `use` する。prelude と同じ名前を持つ自分の item（例えば kit の `button`）は glob より強いので変えない。glob どうしの名前の衝突（`ambiguous`）が出たら、そのファイルで明示の `use` を足して解く。
     `crates/kivact-interaction`・`crates/kivact-style`・`crates/kivact-ui` の `kivact::prelude::*` は doc の例の中だけ。`cargo test -p <crate> --doc` は W5 の確認に入れず（`kivact-docs` と同じく W11 で付属書 G が直す）、ただし `cargo check` で compile する `src` の doc test は無いので壊れない。
  3. API-09：`examples/kivact/{counter,form,list,async,component}/src/main.rs` と `templates/basic/src/main.rs` から `#![warn(redundant_imports)]` の行、`// xtask-allow(...)` の行、先頭の `//! Role: ...` の module doc を消す（T が W1 で例・雛形を Role 規則の対象外にし、`redundant_imports` を `[workspace.lints.rust]` に置いている前提。そうでなければ STOP）。例と雛形の部品を `#[component] fn x(..) -> impl Element`（root に modifier を付けさせないものは `-> impl IntoView`）にそろえる（`component` 以外の例は、`fn app() -> impl IntoView` の root の関数だけを持つならそのまま）。
- **消すもの**：prelude の 5.11 節に無い名前すべて、例と雛形の `#![warn(redundant_imports)]`・`xtask-allow`・`//! Role:`。
- **状態の閉じ方**：なし。
- **試験**：足す試験は無い。`e2e_core` の全体を一度（鎖の最後の C の unit なので）：`dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core`。
- **確認（tier a）**：4.6 節の `W5_CHECK` と上の `e2e_core`。`cargo xtask check --no-write --crate kivact`。
- **完了条件**：`crates/kivact/src/prelude.rs` の名前が 5.11 節の一覧から `icon_button`、`use_toaster`、`color`、`space`、`radius`、`font` を除いたものと一致する（4.3・4.5 節の unit が足す）。`rg -n "warn\(redundant_imports\)|xtask-allow" examples/kivact templates` が 0 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

## C19：DevTools の style の parser を消し、kivact-style の property の表を使う

- **wave / lane / 前提**：W6b の鎖 / 鎖 / 付属書 S の S17（4.3 節 S-REQ-2 の `kivact_style::internal::{StyleProperty, style_property}` を作る宣言の表の unit）の直後
- **所有**：`crates/kivact-core/src/devtools/{apply.rs,styles.rs,node.rs,hooks.rs}`
- **目的**：RC-27：DevTools が kivact-style の `Debug` の出力を読み戻す二つ目の parser（`devtools/apply.rs` の `override_layout`・`override_paint`・`override_text`・`length`・`number`・`color`・`display`・`text_align`・`editable_value` と `*_text` の関数群）を消し、kivact-style の一つの property の表を使う。
- **手順**：
  1. `devtools/apply.rs` の `override_style(property, value) -> Result<Style, String>` の本体を：
     ```rust
     pub(crate) fn override_style(property: &str, value: &str) -> Result<Style, String> {
         let property_def = kivact_style::internal::style_property(property)
             .ok_or_else(|| format!("unknown property: {property}"))?;
         (property_def.parse)(value).map_err(|message| format!("{property}: {message}"))
     }
     ```
     `editable_value(property, described)` を消し、その呼出し元（`devtools/styles.rs` か `devtools/node.rs` の style の一覧を作る関数。`rg -n editable_value crates/kivact-core/src/devtools`）を、style の値から直接 `kivact_style::internal::style_property(name).and_then(|p| (p.describe)(style))` を読む形にする（`describe_style` の `Debug` の文字列を介さない）。
  2. `override_layout`、`override_paint`、`override_text`、`with_property`、`length`、`number`、`color`、`display`、`text_align`、`number_text`、`split_group`、`one_value`、`length_text`、`display_text`、`text_align_text`、`color_text`、`token_name`、`background_text` を消す。`apply_token_overrides`・`token_list`・`pseudo_locale`・`call_sites` は残す（W6 の付属書 S の theme 変数に合わせた `apply_token_overrides` の書換えは S の掃引の unit が行う）。
- **消すもの**：上の関数群。
- **状態の閉じ方**：なし。
- **試験**：`e2e_native` の `devtools.rs` の style override の段（`a_devtools_client_inspects_drives_and_reviews_a_test_app` の中）が通る。
- **確認（tier a）**：
  ```sh
  cargo check -p kivact-core --all-targets --features devtools
  cargo clippy -p kivact-core --all-targets --features devtools -- -D warnings
  cargo xtask check --no-write --crate kivact-core
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --features devtools --test e2e_native -- devtools
  ```
- **完了条件**：`rg -n "fn (override_layout|editable_value|color_text|background_text)" crates/kivact-core` が 0 件。（試験の command はどれも 1 件以上の試験を走らせる。0 件は失敗。4.6 節）

---

## 4. 他の付属書との接点

### 4.1 付属書 R（runtime）に求めるもの・R から受けるもの

W5a の鎖の順（付属書 R の 4.1・4.4 節と合わせたもの）：**R05、R06、R08、R09（core の `AppState`・turn・activation。W5a の鎖の先頭）→ R20〜R25（runtime の API）→ R26〜R31（core・macro・facade と全 crate の掃引、e2e）→ C07、C08、C09、C10、C11、C12、C13、C14、C15、C16、C17、C18 → 付属書 E・A の破壊的な unit と掃引**。W6 の鎖では C19 が付属書 S の宣言の表の unit の直後。

C の W5 の unit は R の鎖の後に走り、次が R の unit で入っていることを前提にする。無ければ STOP。各行の「答え」は付属書 R の 4.1 節の答え。

| ID | 名前・署名 | 答え（付属書 R の unit） | C の利用者 |
|---|---|---|---|
| R-REQ-1 | `kivact_runtime::IntoHandler<(), M>`（sealed。設計書 DEC-16：payload の無い handler、すなわち `Fn() -> R` と `Callback<()>` だけ）と `fn into_handler(self) -> Handler<()>`。facade の path は `kivact::state::IntoHandler`。payload のある handler（`on_key` など、`#[component]` の `Callback<E>` の props で E が `()` でないもの）は `impl Fn(E) -> R + 'static` で受ける | R21 | C09（`Callback<()>` の props）、C12（`on_close_request`） |
| R-REQ-2 | `kivact_runtime::Handler<E>`（`Handler::from_fn`）。facade の path は `kivact::custom::Handler`（`#[doc(hidden)]`） | R21、R26 | C09 |
| R-REQ-3 | `#[doc(hidden)] pub fn Callback::<E>::from_handler(handler: Handler<E>) -> Callback<E>` | R21 | C09 |
| R-REQ-4 | `kivact::custom::Prop<T>`（`#[doc(hidden)]`。`constant`・`new(impl IntoValue<T>)`・`from_text(impl IntoText)`）と `IntoValue::into_prop`（`#[doc(hidden)]`） | R24、R27 | C09（保存の型、`#[derive(Prop)]`） |
| R-REQ-5 | `custom::Reactive`、`IntoReactive`、`state::__private` を消してあること | R24、R27 | C09 |
| R-REQ-6 | `Notifier`→`Trigger`、`notifier()`→`trigger()`、`owner.notifier(at)`→`owner.trigger(at)` の改名。`Trigger::{track, notify, notify_from_render}`。`notify_from_render` は残す | R23 | C06（`node_overrides_changed`。W4 の C06 は `Notifier` で書き、R23〜R31 の掃引が `Trigger` に直す）、C07、C18 |
| R-REQ-7 | runtime の `Owner::current`→`Owner::setup`、`Owner::ambient`→`Owner::running` の改名と、全 crate の呼出し元の掃引（core の 0 引数の `kivact::custom::Owner::current()` は改名しない） | R25、R26〜R31 | C08（`Owner::running`）、C12（`Owner::setup`）、C15（`Owner::running`） |
| R-REQ-8 | `Runtime::end_idle() -> TurnReport` | R06 | C11 |
| R-REQ-9 | `Runtime::reported_since(since: u64) -> Vec<Reported>` | R04（W2a） | C06（W4b） |
| R-REQ-10 | core の turn（RC-01〜RC-03）・activation（RC-02）・`Extensions`（RC-04）の unit を W2〜W4 に置かない | 受け入れ：R05、R06、R08、R09 を W5a の鎖の先頭に置いた。turn の関数は R06 が `launch/turn.rs` に移す（C11・C15 はそのファイルを直す） | C01、C03、C05、C11、C15 |
| R-REQ-11 | `ViewState::of(rt)`・`HostState::of(rt)` の名前と戻りの型を変えない | R05 | 全 C |
| R-REQ-12 | **解決済み**：`use_context::<T>() -> T`・`try_use_context::<T>() -> Option<T>` は R20（runtime）、core の `provide` が `T` をそのまま置く 1 行は R26、呼出し元の `.get()` の削除は R26〜R31 の掃引が行う。C08 は `use_direction`・`provide_direction`・`Owner` の path・`Owner::store` の hidden だけを行う | R20、R26〜R31 | C08 |
| R-REQ-13 | `virtual_list` の `Held`（RC-40） | R10（W3） | — |
| R-REQ-14 | `Retry` を `Callback<()>` にし、`AsyncSnapshot` を消し、`kivact-ui/src/async_view.rs` を直す | R29 | C16 |
| R-REQ-15 | 12 章 API-14（親の決定）：公開の `Callback::call(e)` は `()` を返すまま。`#[doc(hidden)] pub fn Callback::<E>::invoke(&self, e: E) -> Result<Handled, UiError>` が handler の結果を保つ。`IntoHandler<(), Handle> for Callback<()>` は `invoke` で実装する。macro の `Callback` props の本体側の利用（`a_callback_prop_keeps_the_handled_result`）は `invoke` を使う | R21・R26（12 章に合わせた改訂） | C09 |
| R-REQ-16 | 12 章 API-14 の window の経路の E2E（`on_close_request` に inline の closure・共有の `Callback<()>`・`#[component]` の `Callback<()>` の prop を渡して同じ `Handled`）は C12 の `close_request_handled_is_the_same_inline_shared_and_through_a_prop` が持つ。`IntoHandler<(), Handle> for Callback<()>` が `invoke` で結果を保つこと（R-REQ-15）に依る | R21（受け手） | C12 |

R から C へ（付属書 R の 4.4 節）：
- R26 は `kivact-macros` の handler の props を DEC-16 の形にする（C09 の手順 0 に、R26・R27 が済ませた macro の状態と C09 が変えることを書いた）。R31 は e2e の `component_macro` に `callback_prop_closure_infers_its_parameter` を足す。C09 はこの試験を変えない。
- 付属書 R の 4.7 節の場所（runtime・i18n・style・core）の外の `constant_value!` は、C09 の手順 8 が `#[derive(Prop)]` に置き換える。

### 4.2 付属書 E（engine）に求めるもの

| ID | 名前・署名（kivact-engine） | 意味 | C の利用者・wave |
|---|---|---|---|
| E-REQ-1（E31、W4b） | `Writer::set(id, PropValue::InputValue(..)) -> bool` が、model に client へまだ配送していない `TextChanged` がその node にある間は値を書かず `false` を返す | 古い値の echo が新しい native の編集を上書きしない（core の `TextEditsAhead` の置き換え） | C07（W5a） |
| E-REQ-2（E31） | `pub struct LayoutReport { pub rect: Rect, pub layout_rect: Option<Rect>, pub text_truncated: bool }` と `Reader::layout_report(&self, id: NodeId) -> Option<LayoutReport>`：その node の最後の `HostEvent::LayoutChanged` と同じ値 | `observe_node` の種 | C07 |
| E-REQ-3（E31） | `pub struct NodeFlags { pub visible: bool, pub in_live_region: bool }`（`Clone, Copy, PartialEq, Eq, Debug, Default`、`kivact_engine::contract`）、`Writer::watch_flags(&self, id: NodeId, watch: bool)`、`Reader::node_flags(&self, id: NodeId) -> NodeFlags`、`HostEvent::NodeFlagsChanged { node: NodeId, flags: NodeFlags }`：見張っている node の祖先を含めた hidden と live region の事実が commit で変わったときに出す | `NodeRefExt::is_visible`・`is_in_live_region` の O(変化) 化 | C07 |
| E-REQ-4（E31） | `Reader::font_generation(&self) -> u64`：font の database が変わるたびに増える | `platform::font_families()` の書換えを版の変化だけに | C07 |
| E-REQ-5（E08、W4。window 付きの形は E30） | `Reader::record_tree_changes(&self, on: bool)` と `Reader::take_tree_changes(&self) -> TreeChanges`（`pub struct TreeChanges { pub upserted: Vec<(WindowId, NodeId)>, pub removed: Vec<(WindowId, NodeId)> }`）：記録中だけ、作った・prop が変わった・親が変わった node と消えた node を集める（設計書 7.2 Q3） | DevTools の木の通知・find・剪定 | C06（W4b） |
| E-REQ-6（E30、W4） | `pub enum InputOrigin { Platform, Injected(RequestId) }`（`kivact_engine::contract`）と `Routed<T>` の field `pub origin: InputOrigin`。engine の注入の経路（DevTools の `inject_input`）が `Injected` を入れる | DevTools の入力の出所（RC-28） | C06（W4b）。`Routed { .. }` の literal（engine の 7 か所、kivact-test の `host_sync_tests.rs` の 2 か所）は E の unit が直す |
| E-REQ-7 | C13 は W5a の鎖で `kivact-engine/src/embed.rs`（`EmbedOpenError`、`retain_module` の 2 か所）を、C11 は同じファイルの `retain_module` を編集する。E は W5 の間このファイルを編集しない | — | C11、C13 |

engine の `Reader::box_model` は `#[doc(hidden)]` の DevTools 用の API だが、C15 が core の pointer の配送で使う（`target_size`）。E が `box_model` を消す・名前を変える場合は C15 の手順の 1 行を直すこと。

### 4.3 付属書 S（style・types）に求めるもの・S が C の後にすること

| ID | 内容 |
|---|---|
| S-REQ-1 | `impl Theme { pub fn with_fonts(self, fonts: impl IntoIterator<Item = impl Into<Asset>>) -> Theme; pub fn fonts(&self) -> &[Asset] }`（12 章 API-13。theme が `Asset` を持つ。`'static` は要らない）：`with_fonts` は既存の font に足す（二度呼べば両方）。付属書 S の S09（W2a）が作る |
| S-REQ-2 | S17（W6b の鎖。宣言の表）が `kivact_style::internal::{StyleProperty, style_property}` を作る：`pub struct StyleProperty { pub name: &'static str, pub describe: fn(&Style) -> Option<String>, pub parse: fn(&str) -> Result<Style, String> }`、`pub fn style_property(name: &str) -> Option<&'static StyleProperty>`。`describe` は DevTools の override の書式（`12`、`50%`、`auto`、`#2563eb`、theme 変数の名前）。C19 がその直後に使う |
| S-REQ-3 | W6 の鎖で S が prelude（`crates/kivact/src/prelude.rs`）の `pub use crate::style::{..}` に `color, font, radius, space` を足し、`pub use crate::draw::{Draw, Path, Shadow, Stroke};` の `Shadow` を `crate::style::Shadow` へ移す（5.8 節：`Shadow` は style だけ）。`ThemeHint`→`ThemeMode` の掃引は core の `view/state.rs` の `EnvFields.theme_hint`（C05）も直す |
| S-REQ-4 | W6 の S の掃引は C の次のファイルを直す：`kivact-core/src/devtools/apply.rs`（`Palette`・`ColorToken`・`Spacing`・`Radii` の token の override）、`view/root.rs`（root の `ColorToken::Surface`）、`view/elements/container.rs`（`spacer` の `grow`）、`kivact-offscreen/src/lib.rs`（`self.theme.text().family`） |
| S-REQ-5 | `kivact_types::embed::Visibility` の改名（`EmbedVisibility`、ST-21 の embed の部分）は C13 が行う。S は同じ型を別の名前にしない |

### 4.4 付属書 T（tooling）に求めるもの

| ID | 内容 |
|---|---|
| T-REQ-1 | W1 の `visible_to` の書き直しで `kivact-offscreen` の行を書かない（C02 が W2 で足す）。`kivact-svg` の行は今のまま残す（C02 が変える） |
| T-REQ-2 | API-13 の xtask 検査（`Element` を実装する型に、`Element` の method と同じ名前の固有 method を置かない）と API-14 の検査（`kivact-ui` の `pub fn` で `into_value`・`into_text_value` を呼ぶものは `#[track_caller]`）は T が作る |
| T-REQ-3 | 例・雛形・docs を `//! Role:` の規則の対象外にし、`redundant_imports = "warn"` を `[workspace.lints.rust]` に置く（W1）。C18 が例と雛形の行を消す |
| T-REQ-4 | `internal-contract` の印の規則を変えるなら W1 で。C08・C11 は `rg -n "internal-contract" dev/xtask/src` で規則の有無を見て印を付けるか決める |

### 4.5 付属書 U（interaction・ui）・K（kit）・G（docs）との接点

| ID | 内容 |
|---|---|
| U-REQ-1 | C09 の後、ui の部品は `#[component] fn x(..) -> impl Element` で書ける（macro は `::kivact_core` を名指す）。U は W7〜W8 で ui の builder を移し、`Card::disabled` などの `Element` と同名の固有 method（API-13 の 9 個：`card.rs` の `on_press`・`disabled`、`stepper.rs`、`avatar.rs`、`badge.rs`、`chat_message.rs`、`item_row.rs`、`navigation_link.rs`）を builder と一緒に消す |
| U-REQ-2 | `DragUpdate::target_size() -> Size`（USR-16 の interaction の部分）。C15 の `PointerEvent::target_size()` から取り、Escape と修飾キーの経路（`PointerEvent` が無い）は `Active` に最後の大きさを持って使う |
| U-REQ-3 | `icon_button`（USR-17）を作る unit が prelude の `pub use crate::ui::{..}` に `icon_button` を足す。`toaster`→`use_toaster` の改名の unit が prelude に `use_toaster`（`kivact::interaction` から。ui から出すなら ui から一つだけ）を足す |
| U-REQ-4 | `Look<S>` は C10 の `Variants`・`Node::variant_style`・`Node::base_variant_style` の上に作る。ui の `Extensions`・`Variants`（ui の内部）は U が消す。variant の enum は `#[derive(Prop)]`（C09）を付け、`impl Variant for X {}` を持つ |
| U-REQ-5 | `on_draft_commit`→`on_commit` の改名の unit は C15 の新しい試験 `commit_focused_edit_commits_a_draft_and_resolves_after_the_value_is_written` の 1 行を直す |
| U-REQ-6 | `async_view` の `loading` は C16 で `IntoViewFn`。U が `async_view` を `#[component]` にするとき、この型を変えない |
| K-REQ-1 | `kivact-plot` は `kivact::draw::Offscreen`（C17 の後の `render(f64, f64)`・`.scale(f64)`）を使う。`kivact-ui-basic/src/plot/paint.rs` の呼出しは C17 が直してある |
| K-REQ-2 | kit の catalog の `main.rs` は C14 で `Theme::with_fonts(fonts().iter().copied())` を使う形になっている |
| G-REQ-1 | 利用者の docs（`docs/for-kivact-user`）の `#[prop(optional)]`、`scope`、`use_context().get()`、`use_direction().get()`、`App::font`、`App::fonts`、`RenderOptions::font`、`TestFonts::Only`、`kivact::launch`、`EmbedNotice::Stopped`、`embed::Visibility`、`Offscreen::render(u32, u32)`、prelude の名前、`README.md` の `scope` の段落（API-07）は W11 の付属書 G が直す。`architecture.md` の「core の AppClient」（RC-41）は G が「`launch` の turn の関数（付属書 R の `launch/turn.rs`）」に直す |

### 4.6 W5a の鎖の共通の確認（`W5_CHECK`）

試験の command の規則（12 章 PLAN-01・PLAN-07）：`cargo test` は binary ごとに 1 行（`--test e2e_core`・`e2e_ui`・`e2e_native`・`e2e_kits` の module を一つの command に混ぜない。module の binary は付属書 T の T15 の表のとおり）。window を開く package の試験は `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test ...` の形で、window-lock は 1 回だけ。各 unit の「完了条件」には、その unit の試験の command が**どれも 1 件以上の試験を走らせた**ことを含む（出力の `running N tests` の `N` が 0 の command があれば失敗）。

C08〜C18 の「確認」は、次の一式と、unit ごとの試験のコマンドからなる。`kivact-docs` は W5 では compile しない（設計書 9 章）。

```sh
ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT" || exit 1
W5_PACKAGES="kivact-runtime kivact-i18n kivact-style kivact-core kivact-macros kivact-offscreen kivact-interaction kivact-ui kivact kivact-ui-basic kivact-ui-pop kivact-ui-game kivact-ui-creative kivact-ui-soft kivact-ui-basic-catalog kivact-ui-pop-catalog kivact-ui-game-catalog kivact-ui-creative-catalog kivact-ui-soft-catalog kivact-devtools kivact-devtools-mcp kivact-test kivact-public-api-tests kivact-kit-tests kivact-bench kivact-template-basic kivact-example-counter kivact-example-form kivact-example-list kivact-example-async kivact-example-component"
cargo fmt --all
for p in $W5_PACKAGES; do cargo check -p $p --all-targets || break; done
cargo check -p kivact --all-targets --features embed,devtools,test,gpu-interop,tokio
cargo check -p kivact-public-api-tests --all-targets --features embed,devtools
cargo check -p kivact-kit-tests --all-targets
for p in kivact-core kivact-macros kivact kivact-test kivact-offscreen; do cargo clippy -p $p --all-targets -- -D warnings || break; done
cargo clippy -p kivact-core --all-targets --features devtools,embed,gpu-interop -- -D warnings
```

`dev/bench` の probe（`kv_startup`・`kv_stress`・`kv_latency`・`kv_heap`）が compile error になったら、同じ unit で直す（測る画面と操作は変えない。設計書 2 章）。

### 4.7 設計判断の結果（決定済み）

- **D-1（5.1 規則 6 の順）**：解決済み。設計書 5.1 規則 6 が「kivact-core を先に調べる」に直った。C09 はそのとおり。
- **D-2（absorb の node_ref）**：解決済み。設計書 5.1 規則 2 が「node_ref は本体の ref に追加する」に直った。C09 の `absorb` はそのとおり。
- **D-3（`App::font`）**：解決済み（覆された）。設計書 5.6 が `App::fonts` と `App::font` の両方を消すに直った。C14 が両方を消す。
- **D-4（`commit_focused_edit` の後の focus）**：決定済み（採用）。C15 は focus を戻さない（control は focus を失う）。戻すと draft が新しく始まり、確定の直後に「編集中」になる。**推奨**：戻さない。アプリが必要なら `NodeRef::focus()` を呼ぶ（doc に書いた）。

### 4.8 `#[component]` の型引数と、macro が生成する名前（付属書 U・K・G 向け）

- 利用者の型引数（5.1 規則 9）は、生成する struct・constructor・setter の `impl`・`impl Element`・`impl IntoView` にそのまま（bound と where-clause ごと）付き、各型引数に `'static` が足される。どの prop の型にも現れない型引数は隠した field `__phantom: PhantomData<fn() -> (..)>` が持つ。lifetime 引数と const 引数は compile error。
- macro が足す generic は prop ごとの 3 種類だけ：`Callback<()>` の props の `__KvM<PropName>`（`IntoHandler` の marker）、`Callback<E>`（E が `()` でない）の props の `__KvR<PropName>`（`Fn(E) -> R` の `R: HandlerResult`。DEC-16）、`ViewFn` の props の `__KvV<PropName>`（`V: IntoView`）。`<PropName>` は prop の名前の UpperCamel（`on_pick` → `__KvMOnPick`）。constructor では利用者の型引数が先、生成の generic が後。setter の generic は setter の method にだけ付く。 部品の型引数そのものの prop（`control: C`。5.1 規則 3）は `C` のまま受け、生成の generic を足さず、`__phantom` にも入らない。
- 予約：型引数の名前の接頭辞 `__Kv`（使うと compile error）、prop の名前の接頭辞 `__`（生成の `__root`・`__phantom`・`__body`・`__default`・`__n` のため。使うと compile error）。これで利用者の型引数（`K`、`V`、`T`、`M` など）と生成の名前は衝突しない。
- turbofish：constructor が生成の generic を持つ（必須の `Callback`・`ViewFn` の props がある）部品では、利用者の型引数を `x::<K>(..)` で書けない（Rust は明示の generic 引数をすべて要求する）。型引数は props から推論させる。必須の `Callback`・`ViewFn` の props が無い部品では turbofish が書ける（`impl Trait` の引数は明示の generic 引数の数に入らない）。付属書 U の 49 の generic な constructor（`select<K>`、`list<K>`、`data_table<R, K>`、`async_view<T, V>`、`field<C>` など）は、型引数が props の型に現れれば推論で足りる。props の型に現れない型引数（`__phantom` が持つもの）を持ち、かつ生成の generic も持つ部品は、呼び出し側が型を書けないので作らない（U・K が部品を移すときの規則）。

## 5. 自己点検

### 5.1 採用した所見と unit

| 所見 | unit |
|---|---|
| RC-09 | C06 |
| RC-10 | C01（hook の一つの trait object、`if let` 一つ、`wrap_capabilities` の常時 memo の廃止）、C06（agent の非同期起動、App 全体の style override） |
| RC-11 | C12 |
| RC-12 | C11（`LaunchPolicy`、閉じる途中は turn なし、`devtools_in_tests` の削除）、C11・C13（module の保持を engine の `EmbeddedHost` だけに） |
| RC-13 | C11 |
| RC-18 | C08 |
| RC-19 | C05 |
| RC-20 | C07 |
| RC-21 | C03 |
| RC-22 | C04 |
| RC-27 | C01（移動と trait、locale の wrap）、C19（二つ目の parser） |
| RC-28 | C06 |
| RC-29 | C02（crate）、C17（署名） |
| RC-31（core） | R26（`provide`）、C08（`Owner` の path と `store` の hidden） |
| RC-36 | C03 |
| RC-40 | T（印）、R（`Held`、`OwnerEntry`、`Pending.turn_requested`、`ext`）。core の誤った印の文面は W1 で T が印ごと消す |
| RC-41（core） | C01（`is_running`）、C03（`report_session`、`_kind`、`_at`、`n.at`、`take_request` の fast path、`gpu_interop::State` の遅延）、C04（`bind_states`、`Leaving.on_exit`）、C05（`EnvFields`、dead field `host_capabilities`・`title`・`mode`）、C07（`notify_from_render` の core の呼出し）、C11（`devtools_in_tests`）、G（`AppClient` の doc）、R（`Owner::ambient`・`current` の改名、`report_now`、`mount_revision`） |
| API-01 | C09（macro と absorb）、U・K（builder の移行） |
| API-04 | R（R20・R26〜R31。4.1 節 R-REQ-12） |
| API-05 | C18、S-REQ-3、U-REQ-3 |
| API-07 | C09（core の doc 2 か所）、G（README、docs）。runtime の `props.rs` の doc の誤り（`kivact::view::Reactive`）は R-REQ-5 で `Reactive` ごと消える |
| API-08 | C16（`loading` の型）、G（tutorial） |
| API-09 | C18、T-REQ-3 |
| API-10 | C08（`Owner`）、C13（`EmbedVisibility`）、S（`BorderImage`・CSS の値の型、`Direction`）、U（`Snap`・`ComboboxMode`・`ZoomPan`） |
| API-11 | C11（launch の型、`kivact::launch`、`AppOptions`）、C18（prelude）、R（`Callback::{to_fn,bind}`、`Notifier`、`IntoReactive`）。`UiThreadOnly` は設計書 5.2 のとおり `kivact::custom` に残す |
| API-13 | U-REQ-1（固有 method の削除）、T-REQ-2（検査） |
| API-14 | C16（`#[track_caller]`）、T-REQ-2（検査） |
| API-16 | C16 |
| API-18 | C09 |
| API-19（facade の path） | C18（prelude に kit 用の名前を入れない。`kivact::ui::theme` は U） |
| USR-05 | C15 |
| USR-12 | C14 |
| USR-15 | C13 |
| USR-16 | C15（core）、U-REQ-2（interaction） |
| USR-17 | C15（`spacer`）、U-REQ-3（`icon_button`） |
| 5.1 規則 9（型引数を持つ部品） | C09（4.8 節） |
| 12 章 API-1（型引数そのものの prop は `C`）、API-6（`#[derive(Prop)]` の `Self: 'static`）、API-14（`Handled` を保つ）、API-15（body の前に一度だけ登録） | C09 |
| 12 章 API-8（再び開く中身は `ViewFn`） | 本付属書の例に再び開く `Vec<View>` は無い（C09 の例の `children` は一度だけ mount する） |
| 12 章 API-13（font は theme の `Asset`） | C14、S-REQ-1 |
| 12 章 API-16（`provide_direction` は定数） | C08 |
| 設計書 5.1 規則 10（`#[caller]`） | C09 は `#[prop(..)]` 以外の引数の属性を error にし、付属書 U の U25 が `caller` を受けるよう parser を広げる |
| 12 章 PLAN-04・PLAN-07・PLAN-01 | 全 unit（固定の `cd` なし、binary ごとの command、window-lock 1 回、0 件は失敗。4.6 節） |
| 5.4（`Element::variant`） | C10 |
| 5.6（GPU の型、`App: !Clone`、`on_close_request`、`use_direction`、embed、font） | C11、C12、C08、C13、C14 |
| 5.8（`Offscreen`） | C02、C17 |
| 5.11 | C18 |
| 6.2 PERF-9 | C04（provenance）。PERF-10：C06（DevTools の agent）、C07（起動時の font の列挙） |

### 5.2 設計の判断を残していないか

- 各 unit の手順は、変える関数・型の名前と新しいコード（または機械的な置換のコマンド）を書いた。「必要なら」「検討」「例えば」を手順に書いていない。分岐は compile error・`rg` の結果で決まる機械的なもの（C08 の印の有無、C17 の `Transform` の constructor の名前）だけで、決め方を書いた。
- 判断が要るものは 4.7 節の D-4 の 1 つ（D-1〜D-3 は設計書の改訂で解決済み）。推奨の答えで unit を書いた。

### 5.3 同じ wave の二つの unit が同じファイルを持たないか

- W2b：C01（`view/devtools_*`、`view.rs`、`devtools/`、`view/{tree,state,boundary,root,mount/style}.rs`、`launch/{runtime,inspect}.rs`）と C02（新 crate、`Cargo.toml`、core の `Cargo.toml`・`draw.rs`・`draw/offscreen.rs`、facade、policy）。重なりなし。
- W3a：C03（`view/{state,requests,snapshots,foreign_window,mount/resolve,node_ref,elements/builders}.rs`、`client/dispatch/{platform,delivery}.rs`、`platform/app_handle.rs`、`app.rs` の試験なし、`gpu_interop/`、`launch/runtime.rs`）と C04（`view/{mount/style,mount/create,boundary,root,tree,node_modifiers}.rs`、`diagnostics/why.rs`、`devtools/node.rs`）。重なりなし。C03 の手順 8 の `platform/services.rs` の置換は C04 の所有と重ならない。
- W4 と W4b：C05（W4）と C06（W4b）は別の sub-wave。参考に file も分けてある：C05（`view/{state,window_view,foreign,foreign_window,root,hover,tree}.rs`、`launch/{runtime,close}.rs`、`client/dispatch/env.rs`、`platform/env.rs`、`custom/node.rs`）と C06（`devtools/`、`view/devtools_hooks.rs`、`view/mount/style.rs`、`launch/inspect.rs`、agent の 2 ファイル、kivact-test の `screen/state.rs`）。重なりなし（C01 が launch から呼ぶ DevTools の関数を `agent_slot`・`from_slot`・`after_mount` に絞ったので、C06 は `launch/runtime.rs` を持たない）。
- W5a・W6b：鎖（1 lane の直列）なので重なってよい。C11・C13 は engine の `embed.rs`、C13 は arvane の embed のファイルを持つ（E-REQ-7、4.2 節。A の lane は W5 の間これらを編集しない）。
