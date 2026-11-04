# Foundation v3 decisions

| Date | Unit | Question / derivation | Decision and reason |
|---|---|---|---|
| 2026-11-04 | P-0 | Design documents exist only on the starting docs branch | Created integration from main and fast-forwarded the docs branch before unit worktrees. |
| 2026-11-04 | All | Work order requires author approval for design gaps | User explicitly delegates all judgments; orchestrator decides using API consistency, React/Solid accessibility, simple internals, and maintainability. |
| 2026-11-04 | P-0 | First baseline compilation exhausted disk space | Remove only this run's generated target/debug cache and disable incremental/debug symbols for checks; keep runtime debug assertions and optimize-level policy unchanged. |
| 2026-11-04 | P-0 | Baseline GPU read_back aborted with heap corruption after 5/6 tests | Preserve baseline failure; rerun the unchanged full suite with RUST_TEST_THREADS=1 to check whether concurrent GPU tests cause the failure. No source/test suppression. |
| 2026-11-04 | CP0 onward | Unchanged GPU read_back succeeds 6/6 with serial test execution after default parallel heap corruption | Use RUST_TEST_THREADS=1 for checkpoint E2E. Preserve the full test selection, keep window-lock, and retain the baseline concurrency failure as a known existing issue. |
| 2026-11-04 | P-0 | Baseline xtask waited after all architecture outputs were written | Volta node shim left an ELK grandchild holding stderr open. Verified and stopped only that worker; xtask returned 0. Set XTASK_NODE to the bundled node executable for later gates; do not modify vendor. |

## P-1 状態の全数調査（2026-11-04）

対象は `crates/`、`examples/`、`templates/` の Rust source（test と doctest も検索に含む）。付録 A の既存 169 件のうち SM-C27 の解消後だけを現契約に合わせて主担当判断で訂正し、新規 34 件を SM-R901、SM-A901、SM-I901〜904、SM-U901〜905、SM-K901〜923 として追加した。コードは変更しない。P-0 checker の全 crate report と clippy の最終結果は後段に記録する。

判定は 13.1 の 8 種類と、単独 bool を認める 18.14 を併用する。二つ以上の field があるだけで option の束・入力の record を状態機械と扱わず、更新する場所と利用する場所で依存を確認する。既存項目と同じ field/遷移は追加件数に含めない。unit は同 module の既存項目を優先し、該当しなければ同 crate が主な crate になる最初の状態機械 unit（kit は K-1、ui は U-1）へ割り当てる。後の unit が必要なら主担当が担当移管を記録する。

- `PieState` の公開 enum 化は主担当が確定した。`Closed / Open { mode: Click | Held, center, hot }` は閉じた状態の held/hot を排除する。ユーザーが判断を委ねた基準に従う設計決定であり、公開型の変更を見落とした扱いにはしない（SM-K901）。
- `arvane-asset/src/store/worker.rs:73` の jobs/thread の Option は Drop と panic の unwind で所有値を take するため。呼び出せる通常の Worker は両方 Some、stop は worker thread へ送る独立した終了信号である。通常操作から観測できる追加 phase を作らず除外する（切り離した thread の通知は既存 SM-R20）。
- `arvane-shell/src/clicks.rs:45` は click/tap の履歴を `ClickCounter::count` だけで更新する、意味の閉じた名前付き counter（S-9）。mouse の last と contact の tap は独立した pointer kind の履歴で、同時に存在してよい。
- `kivact-engine/src/measure/leaf.rs:402` の LeafText.inherited は一回の leaf 測定中の lazy cache（S-9）。host 起動の前後ではなく `get_or_init` で同じ computed text を再利用する。
- `kivact-reactive/src/data.rs:127` の MemoCell.value は memo の計算結果 cache（S-9）。eq の Option は比較 policy であり初期化 phase ではない。
- `kivact-engine/src/tree/child_order.rs:13` の Editing は Order::Packed/Editing が所有権を移しており、links と packed に同じ列が共存しない（13.1(7)の対象外）。Link の previous/next と Children の first/last は topology の record。
- `kivact-style/src/path/trim.rs:16` の Contour は path の幾何 record。closed と closing_edge は入力の閉じ方を記録し、Contour の構築が一つの関数に閉じる。継続的な lifecycle を持たない。
- `arvane-asset/src/store/asset_store.rs:39` の frame/tick/generation と `begin_frame` の契約は cache の LRU と保護範囲（S-9）。load/check の所属は既に State/Residency/Recheck enum で表す。
- `kivact-core/src/view/flow/error_boundary.rs:128-166` は Shown::Child/Fallback が明示の phase。current は直前の branch を take してから新しい branch を保存する mount 処理の所有 slot で、同時所属や独立した変更元はない。
- `kivact-core` の内部 render effect が tree/registry を更新する箇所は render の出力を作る入口であり、別の利用者向け signal の派生状態を後追いで直す effect と区別する。part の disabled/hidden の複製は既存 SM-C15、region の revision は SM-C16。
- `kivact-ui-creative/src/editors/shared/owned_edit.rs` は EditSession<(owner,snapshot)> 一つで既に状態を所有し、独立した active flag がない。gesture origin は既存 SM-K30。
- `kivact-ui-creative/src/viewport_surface.rs:226-243` の Shown::Gpu/Pixels と fit transform は memo による派生値。resize の前回値だけを SM-K918 に追加した。
- `kivact-ui-soft/src/typeahead.rs:162-183` の resource + sleep は resource が cancellation と stale result を所有する非同期処理であり、caller の Option<Task> timer ではない。cached は選択済み label の保持用で、SM-K919 の LabelCache<K> に保持規則を閉じる。options は resource 状態から直接 memo で読む。
- `kivact-ui-soft/src/typeahead.rs:487-518` の token_notices は Binding の差分を外部の announcer へ渡す通知 adapter。previous は差分計算の履歴で、label は翻訳 message の callback 内の入力、独立の表示 phase ではない。
- `examples/` の effect の signal 書込みは 18.20 の effect-writes の対象外である利用者 code。独立した demo の dialog の open・theme・editing などは同時に成立できる。fake async の sleep は server を模擬する処理で lifecycle の内部機械ではない。
- `templates/basic/src/main.rs:14` の memo は count の純粋な算出。templates に mutable phase の候補なし。
- `static` の OnceLock（icons、recipes、module mapping、native input message id、test display）は process 単位の値/cache で、18.20 の disallowed_types の明示例外に当たる。instance の後付けとは分ける。
- tests 内の Rc<Cell<Option<T>>> は setup の外へ handle/result を渡す試験 harness の mailbox。test の input/control flags と統計は別の事実で、製品の状態を構成しない。製品側の test host の lifecycle は既存 SM-A15。
- debug_assert の有限値、寸法、unique id、slot/table の整合、tree の親子関係の検査は値/topology の不変条件で phase の同期ではない。frame/turn/layout の順序の検査は既存 SM-R08、SM-C05/C06、SM-E04/E28/E29、SM-A01 に対応。
- doc の first/before/after は大半が描画順、focus/keyboard の仕様、引数の意味、callback の実行順である。setup owner は既存 SM-R08、mount/commit は SM-R05/SM-C06、GPU の submit/wait は既存 SM-A14、layout compute/reset は SM-E28 に対応する。

### 検索の件数

文字列一致は field 以外（引数、import、doc、enum variant）も含む。複数検索に一致する行は重複する。以下の台帳では同じ理由で判断する行を path ごとの line 列にまとめ、struct は型ごとに一行にした。

| 検索 | 一致数 |
|---|---:|
| `Cell<bool>` | 21 |
| `Cell<Option` | 59 |
| `RefCell<Option` | 28 |
| `OnceCell` | 21 |
| `OnceLock` | 40 |
| `Signal<bool>` | 69 |
| `Var<bool>` | 1 |
| `Signal<u64>` | 6 |
| `Var<u64>` | 1 |
| `: bool,` | 724 |
| `debug_assert` | 33 |
| `sleep` | 40 |
| doc の指定語（行） | 2089 |
| 二つ以上の bool/Option/counter field の struct 候補 | 505 |

### struct の判定台帳

`対応` は既存/新規の項目で処理する場所（同じ file の option record も含むため、すべてを新規機械と数えない）。`除外` は 18.14 の役割による判定。field 名を併記して検索を再現できるようにする。

| 場所・型 | field | 判断と理由 |
|---|---|---|
| `crates/arvane/src/embed.rs:15` `EmbedOptions` | `scale`, `on_resize_request`, `prefer_gpu` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/arvane/src/env.rs:30` `Env` | `renderer`, `test_headless` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/arvane/src/window.rs:340` `Shell` | `prefer_gpu`, `observer` | 対応: SM-A10（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane/src/test_display/shell.rs:18` `VirtualShell` | `reduced_motion`, `dark`, `locale`, `dialogs`, `fail_next`, `redraw`, `min_size`, `ime`, `ime_text`, `fullscreen`, `soft_keyboard`, `frame`, `gpu`, `real`, `occluded`, `cpu_frame` | 対応: SM-A15（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane/src/test_display/virtual_window.rs:41` `VirtualWindowOptions` | `reduced_motion`, `dark`, `locale`, `gpu`, `renderer` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/arvane/tests/e2e/embed.rs:13` `Seen` | `waker`, `timer`, `wheel_settings` | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `crates/arvane-asset/src/store/entry.rs:117` `VariantKey` | `size`, `crop`, `tint` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/arvane-asset/src/store/worker.rs:73` `Worker` | `jobs`, `thread` | 除外: 上記の型別判断（所有権移動、履歴、cache、topology） |
| `crates/arvane-native/src/runtime.rs:86` `WinitWindow` | `subclass`, `monitor` | 対応: SM-A07、SM-A10（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane-native/src/runtime.rs:513` `Runner` | `cursor`, `shift` | 対応: SM-A07、SM-A10（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane-native/src/test_display.rs:112` `Pending` | `frame`, `accessibility`, `title`, `resize`, `gpu`, `scheduled` | 対応: SM-A15（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane-native/src/test_display.rs:377` `Entry` | `gpu`, `frame`, `owner` | 対応: SM-A15（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane-native/src/embed/windows/state.rs:59` `State` | `on_resize_request`, `in_callback`, `draining`, `pending_surrogate`, `frame`, `uia` | 対応: SM-A01、SM-A02、SM-A03。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/arvane-native/src/embed/x11/ime.rs:50` `App` | `allowed`, `focused` | 対応: SM-A06。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/arvane-native/src/embed/x11/ime.rs:65` `Connection` | `protocol_window`, `server_window`, `pending`, `flight` | 対応: SM-A06。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/arvane-native/src/embed/x11/window.rs:18` `Window` | `opening`, `on_resize_request`, `redraw` | 対応: SM-A01、SM-A05。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/arvane-native/src/native_input/state.rs:33` `NativeInput` | `text`, `reconvert`, `pointer`, `file_pointer`, `subclass_installed`, `reduced_motion`, `locale`, `env_changed`, `waker` | 対応: SM-A08（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane-native/src/native_input/state.rs:71` `ContactNote` | `canceled`, `tilt` | 対応: SM-A08（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane-render/src/paint.rs:44` `PatternFill` | `repeat_x`, `repeat_y` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/arvane-render/src/draw/canvas.rs:58` `GlyphRun` | `hint`, `cached` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/arvane-render/src/gpu/state.rs:27` `GpuEnv` | `backends`, `power` | 対応: SM-A12（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane-render/src/gpu/state.rs:69` `GpuContext` | `interop`, `offscreen` | 対応: SM-A12（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane-render/src/gpu/state.rs:147` `GpuSurface` | `raster`, `keep` | 対応: SM-A12（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/arvane-shell/src/child.rs:61` `ChildOptions` | `scale`, `on_resize_request` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/arvane-shell/src/clicks.rs:45` `ClickCounter` | `last`, `pressed`, `tap` | 除外: 上記の型別判断（所有権移動、履歴、cache、topology） |
| `crates/arvane-shell/src/ime.rs:16` `ShellImeText` | `password`, `composition` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/arvane-shell/src/services.rs:7` `ShellFileDialog` | `directory`, `file_name` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/arvane-text/src/cache.rs:82` `SpanBits` | `font_size`, `weight`, `color`, `family`, `italic`, `letter_spacing`, `word_spacing`, `kerning`, `features`, `decoration`, `highlight` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/arvane-text/src/cache.rs:102` `RichKey` | `decoration`, `overflow`, `max_lines`, `capsize` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/arvane-text/src/cache.rs:118` `TextKey` | `color`, `wrap`, `family`, `italic`, `max_width`, `carets`, `rich` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/arvane-text/src/cache.rs:235` `Cache` | `families`, `font_names` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/arvane-text/src/layouts.rs:29` `SpanKey` | `font_size`, `weight`, `family`, `italic`, `letter_spacing`, `word_spacing`, `kerning`, `features` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/arvane-text/src/layouts.rs:87` `Key` | `wrap`, `family`, `italic`, `rich` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-core/src/app.rs:14` `App` | `locale`, `window_size`, `min_window_size`, `threads` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/embed.rs:33` `Parent` | `window`, `scale`, `on_resize_request` | 対応: SM-C04、SM-C28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/client/link.rs:26` `HostLink` | `now_ns`, `in_turn` | 対応: SM-R04（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-core/src/client/state.rs:33` `HostState` | `turns`, `stale_events` | 対応: SM-C02、SM-C25。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/gpu_interop/share/app_side.rs:25` `SharedImage` | `current`, `pixels` | 対応: SM-C26。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/platform/services.rs:41` `FileDialog` | `directory`, `file_name` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/primitive/measurement.rs:22` `Constraints` | `known_width`, `known_height` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/test/options.rs:36` `RenderOptions` | `locale`, `host_locale`, `reduced_motion`, `dark`, `gpu`, `title` | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `crates/kivact-core/src/test/screen/host_sync_tests.rs:340` `Probe` | `armed`, `drops`, `reentered`, `reader` | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `crates/kivact-core/src/test/screen/model.rs:17` `Screen` | `artifacts_written`, `last_contact` | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `crates/kivact-core/src/test/screen/results.rs:228` `LayoutViolation` | `role`, `test_id` | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `crates/kivact-core/src/test/screen/results.rs:296` `SemanticNode` | `role`, `value`, `test_id`, `focused`, `disabled`, `selected`, `checked`, `at` | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `crates/kivact-core/src/view/common.rs:26` `Common` | `exit_complete`, `test_id`, `role`, `name`, `description`, `value`, `selected`, `disabled`, `hidden`, `invalid`, `semantics`, `checked`, `indeterminate`, `expanded`, `focusable`, `focusable_when_disabled`, `roving_focus`, `hit_testable`, `focus_on_press`, `text_editing`, `ime`, `part_region_events`, `part_modal`, `draw`, `draw_over`, `intrinsic`, `image_fit`, `monochrome`, `at` | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/common.rs:210` `Registry` | `revision`, `changed` | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/common.rs:242` `Capture` | `target`, `fallback` | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/common.rs:257` `Constraints` | `disabled`, `hidden`, `focusable`, `eligible`, `caller_disabled`, `caller_hidden` | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/common.rs:267` `SemanticSources` | `value_min`, `value_max`, `value_now`, `live_region`, `sort`, `orientation`, `active_descendant`, `level`, `set_size`, `pos_in_set`, `multiselectable`, `busy`, `required`, `read_only`, `has_popup`, `current_item`, `text` | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/editing_data.rs:82` `TextEditing` | `on_edit`, `on_selection` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/editing_data.rs:89` `EditorProps` | `behavior`, `highlight`, `forward_keys`, `multiline`, `line_layout`, `report_text`, `password`, `read_only`, `max_length`, `placeholder` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/input.rs:123` `KeyEvent` | `origin`, `region`, `repeat`, `text`, `target_role`, `pointer` | 除外: role が決まった入力/出力/API の record（18.14） |
| `crates/kivact-core/src/view/input.rs:171` `PointerEvent` | `pressure`, `tilt`, `is_primary` | 除外: role が決まった入力/出力/API の record（18.14） |
| `crates/kivact-core/src/view/overlays.rs:140` `Overlay` | `viewport_margin`, `keep_anchor_clear`, `modal` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-core/src/view/overlays.rs:502` `KeyRoute` | `stop`, `activates` | 対応: SM-C22。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/part.rs:75` `PartState` | `pressed`, `hovered`, `focused`, `focus_visible`, `disabled`, `hidden` | 対応: SM-C15。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/part.rs:111` `InputState` | `pointer_pressed`, `key_pressed`, `hovered`, `disabled`, `hidden` | 対応: SM-C15。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/part_data.rs:16` `PartMount` | `disabled`, `hidden`, `focusable`, `eligible` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/part_data.rs:31` `PartConstraints` | `disabled`, `hidden`, `focusable` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/state.rs:61` `NodeRefState` | `bound`, `observes_layout`, `rect_value` | 対応: SM-C08、SM-C09、SM-C10、SM-C14、SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/state.rs:143` `FocusKeys` | `node`, `visible` | 対応: SM-C08、SM-C09、SM-C10、SM-C14、SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/state.rs:204` `Started` | `min_window_size`, `adapter` | 対応: SM-C08、SM-C09、SM-C10、SM-C14、SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/tree.rs:65` `Handlers` | `activate`, `secondary_context`, `text`, `selection`, `image_error`, `scroll`, `layout` | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/tree.rs:145` `PartActions` | `activate`, `text` | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/tree.rs:209` `NodeObservation` | `rect`, `text_truncated` | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/tree.rs:226` `TNode` | `kind`, `parent`, `at`, `handlers`, `exit_complete`, `observation` | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/tree.rs:281` `Tree` | `removes`, `visibility_revision`, `live_region_revision` | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/flow/branch.rs:69` `Show` | `when`, `fallback` | 対応: SM-C19。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/flow/branch.rs:168` `ShowSome` | `value`, `fallback` | 対応: SM-C19。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/flow/error_boundary.rs:35` `ErrorBoundary` | `fallback`, `on_error` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/flow/virtual_list.rs:61` `VirtualList` | `scroll_to`, `keep_focused` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-core/src/view/mount/create.rs:17` `FirstWrite` | `exit_complete`, `test_id`, `role`, `focusable`, `focusable_when_disabled`, `roving_focus`, `hit_testable`, `intrinsic`, `observe_layout`, `editing` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/mount/draw.rs:17` `DrawInput` | `below`, `over`, `local_paint`, `regions` | 除外: role が決まった入力/出力/API の record（18.14） |
| `crates/kivact-core/src/view/mount/facts.rs:11` `NodeFacts` | `focus_stop`, `local_paint` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/mount/parts.rs:126` `ConstraintSources` | `disabled`, `hidden`, `focusable` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/mount/part_input.rs:38` `Selected` | `observed`, `target`, `own_canvas` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/mount/resolve.rs:32` `Resolved` | `disabled`, `selected`, `draw`, `late_semantics`, `ime`, `attached` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/mount/semantics.rs:169` `SemanticProps` | `name`, `description`, `value`, `checked`, `indeterminate`, `expanded`, `invalid`, `hidden`, `image_fit`, `monochrome`, `focus_on_press`, `sources` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-core/src/view/mount/style.rs:25` `States` | `disabled`, `selected` | 対応: SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/mount/style.rs:70` `NodeStyle` | `focusable`, `disabled`, `selected`, `fix_layout`, `local_paint` | 対応: SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-editing/src/behavior.rs:39` `EditContext` | `multiline`, `read_only`, `password`, `layout` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-editing/src/edit_state.rs:47` `EditState` | `max_length`, `damage` | 対応: SM-E32（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-engine/src/embed.rs:19` `EmbedOptions` | `window_size`, `scale`, `on_resize_request` | 対応: SM-R20、SM-E02、SM-A01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-engine/src/mutation.rs:96` `Mutation` | `style`, `measure`, `text`, `placed`, `restack`, `paint`, `states`, `read_only`, `subtree`, `local_text` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-engine/src/a11y/state.rs:10` `Metadata` | `parents`, `modal` | 対応: SM-E23、SM-E25。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/a11y/state.rs:42` `A11yState` | `root`, `empty_root`, `focus` | 対応: SM-E23、SM-E25。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/check/boxes.rs:68` `Axes` | `x`, `y` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-engine/src/host/pump.rs:45` `Host` | `next_wake`, `embedded_closing`, `reduced_motion_override`, `suspended`, `soft_keyboard`, `accessibility_noted`, `adapter` | 対応: SM-E01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-engine/src/host/testing/test_host.rs:23` `Counts` | `turns`, `frame_ticks` | 対応: SM-A15（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-engine/src/host/testing/test_options.rs:27` `TestHostOptions` | `reduced_motion`, `locale`, `inline_assets`, `gpu` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-engine/src/measure/editor.rs:6` `EditorNode` | `lines`, `measure_lines`, `pending_controlled`, `reported_session` | 対応: SM-E19（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-engine/src/measure/state.rs:14` `MeasureColumns` | `natural`, `extents`, `computed_text` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-engine/src/measure/state.rs:47` `TextExtents` | `single`, `wrapped` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-engine/src/model/aggregate.rs:12` `Model` | `editors`, `window_request` | 対応: SM-E32（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-engine/src/model/changes.rs:112` `Changes` | `needs_layout`, `paint`, `restack`, `fonts`, `a11y_structure`, `a11y_focus`, `layout_changed`, `viewport_chain` | 対応: SM-E24、SM-E27。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/changes.rs:132` `Dispatched` | `paint`, `restack`, `fonts`, `a11y_structure`, `a11y_focus` | 対応: SM-E24、SM-E27。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/lifecycle.rs:47` `RemovedPayload` | `editor`, `extents` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-engine/src/model/edit/state.rs:12` `InputSource` | `transform`, `password`, `read_only` | 対応: SM-E20、SM-E32。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/edit/state.rs:27` `SessionSource` | `area`, `transform` | 対応: SM-E20、SM-E32。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/focus/state.rs:9` `Focused` | `visible`, `reveal_at_commit` | 対応: SM-E23。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/pointer/state.rs:114` `Route` | `capture`, `press`, `touch` | 対応: SM-E10、SM-E11、SM-E12、SM-E13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/pointer/state.rs:205` `PointerSource` | `pressure`, `tilt`, `primary` | 対応: SM-E10、SM-E11、SM-E12、SM-E13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/state_motion/state.rs:50` `StateAnimation` | `colors_text`, `colors_base_text` | 対応: SM-E31。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/paint/state.rs:21` `PaintState` | `ink`, `look`, `needs_redraw` | 対応: SM-E08、SM-E09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/paint/state.rs:79` `Look` | `recolor`, `clip`, `rounded` | 対応: SM-E08、SM-E09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/paint/state.rs:115` `Step` | `transform`, `clips`, `rounded`, `own`, `group`, `matrix`, `parent_matrix` | 対応: SM-E08、SM-E09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/paint/state.rs:160` `PaintList` | `stale`, `built_for` | 対応: SM-E08、SM-E09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/tree/child_order.rs:7` `Link` | `previous`, `next` | 除外: 上記の型別判断（所有権移動、履歴、cache、topology） |
| `crates/kivact-engine/src/tree/child_order.rs:13` `Editing` | `first`, `last` | 除外: 上記の型別判断（所有権移動、履歴、cache、topology） |
| `crates/kivact-engine/src/tree/child_order.rs:187` `Children` | `first`, `last` | 除外: 上記の型別判断（所有権移動、履歴、cache、topology） |
| `crates/kivact-engine/src/tree/node.rs:22` `HostNode` | `scenes`, `sent_layout`, `sent_scroll` | 対応: SM-E30、SM-E31。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/tree/node.rs:57` `Place` | `transformed`, `z_states`, `stacks`, `sticky`, `fixed` | 対応: SM-E30、SM-E31。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/tests/host_locale.rs:15` `Told` | `at_start`, `changes` | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `crates/kivact-gpu-share/src/common.rs:10` `Identity` | `luid`, `device_uuid`, `driver_uuid` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-gpu-share/src/pool.rs:113` `Slots` | `drawn`, `last` | 対応: SM-A14（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-interaction/src/combobox.rs:93` `Combobox` | `popup`, `active` | 対応: SM-I06。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/delayed.rs:66` `Delayed` | `shown`, `wanted` | 対応: SM-I03。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/dialog.rs:75` `Dismiss` | `escape`, `outside` | 対応: SM-I05。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/dialog.rs:94` `DialogSettings` | `dismiss`, `initial_focus` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-interaction/src/disclosure.rs:51` `Disclosure` | `open`, `shown` | 対応: SM-I01。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/disclosure.rs:62` `DisclosureSettings` | `disabled`, `leaving` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-interaction/src/drag.rs:181` `DragSettings` | `start`, `update`, `end`, `edit` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-interaction/src/drag.rs:252` `Drag` | `active_source`, `sources`, `disabled` | 対応: SM-I12。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/edit.rs:130` `EditSession` | `initial`, `observer` | 対応: SM-I11。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/follow_end.rs:92` `FollowState` | `locked`, `geometry`, `expected`, `revision`, `history_pending` | 対応: SM-I23。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/palette.rs:64` `Command` | `group`, `hint`, `icon`, `disabled` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/palette.rs:265` `CommandPalette` | `open`, `close_on` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/popup.rs:112` `PopupSettings` | `anchor`, `initial_focus`, `focus_return`, `anchor_starts_focus_range` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-interaction/src/popup.rs:121` `Opening` | `opener`, `closed_by`, `returned` | 対応: SM-I02。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/roving.rs:25` `Member` | `eligible`, `disabled`, `hidden` | 対応: SM-I16。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/roving.rs:44` `Observation` | `current`, `active`, `adopt`, `reject_refused` | 対応: SM-I16。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/roving.rs:61` `Roving` | `requested`, `observed`, `restore_from` | 対応: SM-I16。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/select.rs:110` `SelectApi` | `selected`, `preview`, `arrow_open`, `retain_trigger_focus`, `arrow_edge`, `hovered` | 除外: role が決まった入力/出力/API の record（18.14） |
| `crates/kivact-interaction/src/splitter.rs:122` `SplitterSettings` | `key_step`, `shift_step_factor`, `reset`, `scale`, `disabled` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-interaction/src/toaster.rs:60` `ToastFocus` | `within`, `previous` | 対応: SM-I22。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/toaster.rs:67` `ToastOptions` | `key`, `on_hide` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-interaction/src/toaster.rs:113` `Toast` | `body`, `duration`, `action`, `icon` | 対応: SM-I22。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/toaster.rs:285` `Toaster` | `next`, `limit`, `holds` | 対応: SM-I22。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/toggle.rs:92` `Core` | `checked`, `disabled` | 対応: SM-I24。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/tooltip.rs:73` `Hub` | `hidden`, `next` | 対応: SM-I04。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/tree.rs:42` `TreeNode` | `parent`, `disabled` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/tree.rs:324` `TreeItemState` | `selected`, `active` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/zoom_pan.rs:142` `ZoomPan` | `gesturing`, `armed` | 対応: SM-I19、SM-I28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/calendar/handle.rs:34` `CalendarDayState` | `selected`, `edge`, `disabled` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/calendar/policy.rs:23` `CalendarPolicy` | `disabled`, `week_start`, `initial_focus`, `select_on_move`, `on_commit` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/list/data.rs:37` `ListItem` | `disabled`, `icon`, `description` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/list/data.rs:238` `ListPolicy` | `axis`, `typeahead` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/list/data.rs:301` `ListItemState` | `selected`, `active` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/list/state.rs:21` `Pending` | `origin`, `origin_ref` | 対応: SM-I14、SM-I26。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/list/state.rs:41` `State` | `selectable`, `anchor_value`, `anchor`, `active`, `retained_focus`, `leave`, `scroll`, `pending` | 対応: SM-I14、SM-I26。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/menu/level.rs:51` `Level` | `shown`, `aiming`, `retain_disabled_focus` | 対応: SM-I10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/menu/press_state.rs:46` `Press` | `gesture`, `highlight` | 対応: SM-I09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/menu/state.rs:55` `MenuItemState` | `pressed_over`, `check`, `expanded`, `destructive` | 対応: SM-I08。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/menu/state.rs:141` `ItemEntry` | `disabled`, `disabled_reason`, `icon`, `description`, `destructive`, `keep_open` | 対応: SM-I08。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/menu/state.rs:619` `Session` | `opened`, `origin`, `focus_key`, `want`, `press` | 対応: SM-I08。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/scalar/edit_state.rs:16` `Shared` | `touch`, `disabled`, `key_step`, `thumb_focus` | 対応: SM-I17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/scalar/edit_state.rs:113` `Edit` | `thumb_rect`, `bounds` | 対応: SM-I17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/scalar/touch.rs:25` `Contact` | `anchor`, `moved` | 対応: SM-I18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/scalar/touch.rs:45` `Touch` | `contact`, `lifted`, `coast` | 対応: SM-I18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/scalar/track.rs:14` `Target` | `rect`, `bounds` | 対応: SM-I901（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-interaction/src/scalar/track.rs:82` `Geometry` | `rect`, `rtl` | 対応: SM-I901（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-interaction/src/scalar/track.rs:130` `Pointer` | `geometry`, `attached`, `grab` | 対応: SM-I901（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-interaction/src/table/data.rs:12` `Header` | `sortable`, `width`, `min_width` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/table/data.rs:79` `TableHeaderState` | `sort`, `sortable`, `sort_rank` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/table/data.rs:90` `TableRowState` | `selected`, `active` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/table/data.rs:100` `TableCellState` | `selected`, `active` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/table/model.rs:62` `ColumnDef` | `width`, `min_width`, `compare`, `matches`, `matches_value`, `visible` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/table/model.rs:284` `TableModel` | `sort`, `query`, `column_queries`, `arrangement`, `typed_queries`, `paging` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-interaction/src/table/state.rs:19` `State` | `label`, `sort`, `multi_sort`, `active` | 対応: SM-I26、SM-I27。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/text_edit/handle.rs:315` `Data` | `forward_keys`, `password`, `read_only`, `max_length`, `placeholder`, `submit`, `composition`, `mounted`, `next_control`, `was_composing` | 対応: SM-I13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-layout/src/columns.rs:41` `Counted` | `unfiltered`, `at` | 対応: SM-E28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-layout/src/columns.rs:92` `NodeLayoutValues` | `final_layout`, `performed` | 対応: SM-E28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-layout/src/pass_state.rs:26` `LayoutState` | `changed`, `fixed_cursor` | 対応: SM-E28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-layout/src/tree.rs:51` `LayoutPass` | `final_layout`, `performed` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-layout/src/style/node.rs:34` `NodeStyle` | `aspect_ratio`, `align_items`, `align_self`, `align_content`, `justify_content`, `justify_self`, `grid` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-lower/src/context.rs:47` `Context` | `error`, `dropped`, `failures` | 対応: SM-E33（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-lower/src/glyphs.rs:26` `Colors` | `glyphs`, `decorations` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-motion/src/animated.rs:19` `AnimCell` | `running`, `completed` | 対応: SM-R25（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-motion/src/state.rs:18` `Frames` | `requested`, `last_tick` | 対応: SM-R22、SM-R23、SM-R24、SM-C23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-motion/src/state.rs:41` `MotionEnv` | `reduced_motion`, `surface_active` | 対応: SM-R22、SM-R23、SM-R24、SM-C23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-motion/src/state.rs:60` `MotionState` | `env`, `applied_offset_ns` | 対応: SM-R22、SM-R23、SM-R24、SM-C23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-motion/src/state.rs:294` `LoopCell` | `rate`, `run`, `frames` | 対応: SM-R22、SM-R23、SM-R24、SM-C23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-reactive/src/arena.rs:33` `Entry` | `value`, `tomb` | 対応: SM-R901。value/tomb の排他と free/retired を明示する |
| `crates/kivact-reactive/src/data.rs:47` `Runtime` | `seq`, `write_epoch`, `root_owner`, `later`, `turn_requested`, `writer`, `stab_epoch` | 対応: SM-R01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-reactive/src/data.rs:127` `MemoCell` | `value`, `f`, `eq` | 対応: SM-R01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-reactive/src/data.rs:274` `OwnerEntry` | `parent`, `on_error`, `disposing` | 対応: SM-R01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-reactive/src/data.rs:311` `Ctx` | `owner`, `setup`, `pure`, `effect`, `tracking` | 対応: SM-R01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-reactive/src/error.rs:87` `Inner` | `at`, `source` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-reactive/src/host.rs:18` `Pending` | `user`, `render`, `later`, `turn_requested` | 対応: SM-R10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-style/src/grid.rs:262` `GridStyle` | `columns`, `rows`, `auto_columns`, `auto_rows`, `flow`, `column`, `row`, `justify_items`, `justify_self` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/resolve.rs:58` `CompiledStyle` | `retained_paint`, `text`, `states` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/resolve.rs:194` `Interactive` | `focusable`, `disabled`, `selected`, `root` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/sheet.rs:161` `Sides` | `top`, `right`, `bottom`, `left` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/sheet.rs:179` `Style` | `local_paint`, `row_gap`, `column_gap`, `width`, `height`, `min_width`, `min_height`, `max_width`, `max_height`, `grow`, `shrink`, `basis`, `aspect_ratio`, `direction`, `align`, `align_self`, `justify`, `align_content`, `grid_box`, `grid`, `wrap`, `position`, `overflow`, `scroll_axes`, `background`, `border_width`, `border_color`, `border_style`, `ring`, `radius`, `shadows`, `opacity`, `translate`, `rotate`, `scale`, `transform`, `transform_origin`, `scrollbar`, `cursor`, `z_index`, `clip`, `filter`, `backdrop_filter`, `blend`, `mask`, `clip_path`, `font_size`, `weight`, `color`, `line_height`, `text_wrap`, `text_align`, `font_family`, `italic`, `text`, `hover`, `group`, `group_hover`, `group_focus`, `pressed`, `focus`, `focus_visible`, `disabled`, `selected`, `enter`, `exit` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/sheet.rs:1098` `ScrollbarStyle` | `width`, `colors` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/text.rs:216` `TextExtras` | `letter_spacing`, `word_spacing`, `kerning`, `features`, `tabular_nums`, `decoration_line`, `decoration_color`, `decoration_thickness`, `underline_offset`, `overflow`, `line_clamp`, `capsize`, `transform`, `shadows`, `caret_color`, `selection_color`, `placeholder_color` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/text.rs:355` `SpanStyle` | `font_size`, `weight`, `color`, `family`, `italic`, `letter_spacing`, `word_spacing`, `kerning`, `features`, `tabular_nums`, `decoration_line`, `decoration_color`, `decoration_thickness`, `underline_offset`, `background` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/draw/image.rs:48` `ImageOptions` | `source`, `pixelated` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-style/src/draw/layer.rs:34` `Layer` | `area`, `transform`, `clip_rect`, `clip`, `mask`, `shadow` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/draw/recorder.rs:151` `RegionSemantics` | `description`, `value`, `checked`, `selected`, `expanded`, `invalid`, `properties`, `disabled`, `hidden`, `focusable`, `focus_on_press` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/draw/text_run.rs:29` `TextStyle` | `family`, `italic`, `line_height`, `tabular_nums`, `decoration_color` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/paint/gradient.rs:59` `Gradient` | `stops`, `repeating` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-style/src/path/trim.rs:16` `Contour` | `closed`, `closing_edge` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-task/src/action.rs:189` `ActionCell` | `latest`, `last`, `next_handler` | 対応: SM-R13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-task/src/latest.rs:20` `LatestRun` | `current`, `next_id` | 対応: SM-R14（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-task/src/state.rs:58` `TaskState` | `blocking_threads`, `pool` | 対応: SM-R17、SM-R18、SM-R21、SM-C23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-text/src/records.rs:22` `Key` | `multiline`, `wrap_width` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-text/src/records.rs:68` `Kept` | `colors`, `painted`, `run_id`, `run` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-text/src/source.rs:7` `Source` | `damage`, `password`, `multiline`, `wrap_width` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-types/src/geometry.rs:155` `IntrinsicSize` | `min`, `preferred`, `max` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/gpu_interop.rs:57` `GpuPreference` | `backend`, `adapter` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/gpu_interop.rs:71` `GpuAdapter` | `luid`, `device_uuid`, `driver_uuid` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/input.rs:50` `Modifiers` | `shift`, `ctrl`, `alt`, `meta` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/platform.rs:154` `Capabilities` | `touch`, `pen`, `soft_keyboard`, `safe_area`, `memory_warnings`, `ime_reconversion`, `native_accessibility`, `gpu` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/protocol/effect.rs:57` `Layer` | `clip`, `filter`, `mask` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/protocol/effect.rs:94` `BoxShadow` | `inset`, `knockout` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/protocol/paint.rs:205` `Pattern` | `repeat_x`, `repeat_y` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/protocol/semantics.rs:20` `SemanticsNode` | `value`, `disabled`, `hidden`, `focusable`, `checked`, `selected`, `expanded`, `test_id`, `properties` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/protocol/semantics.rs:117` `SemanticsProperties` | `text`, `value_min`, `value_max`, `value_now`, `live_region`, `active_descendant`, `level`, `set_size`, `pos_in_set`, `multiselectable`, `busy`, `required`, `has_popup`, `current_item`, `invalid`, `read_only`, `protected` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/protocol/text.rs:40` `TextDecoration` | `color`, `thickness` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/protocol/text.rs:55` `TextStyle` | `family`, `italic`, `max_lines`, `capsize` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/protocol/text.rs:137` `SpanStyle` | `font_size`, `weight`, `color`, `family`, `italic`, `letter_spacing`, `word_spacing`, `kerning`, `features`, `decoration`, `highlight` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/protocol/window.rs:11` `PointerInput` | `pressure`, `tilt` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-types/src/protocol/window.rs:65` `Environment` | `reduced_motion`, `dark`, `locale` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-ui/src/button.rs:79` `Button` | `icon`, `trailing`, `loading`, `expanded`, `hide_label` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/button.rs:94` `Settings` | `size_explicit`, `leading_icon`, `trailing_icon` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/command_palette.rs:24` `CommandPalette` | `open`, `filter`, `close_on`, `placeholder`, `empty`, `footer` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/data_column.rs:47` `DataColumn` | `filter_field`, `cell`, `header_menu`, `numeric`, `resizable`, `flex`, `min_width` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/dialog.rs:46` `Dialog` | `open`, `title_hidden`, `align`, `width`, `description`, `initial_focus`, `dismiss`, `closable`, `media`, `content`, `footer` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/disclosure.rs:21` `Disclosure` | `open`, `keep_mounted`, `animated` | 対応: SM-U06（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui/src/field.rs:22` `Field` | `hint`, `required`, `error` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/list.rs:18` `List` | `selected`, `multiple`, `on_edit`, `item` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/menu.rs:38` `Menu` | `offset`, `align`, `at_least_trigger_width`, `item` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/menu.rs:256` `Level` | `align`, `at_least_anchor_width`, `context`, `sub` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/progress.rs:21` `ProgressMotion` | `animate_initial`, `reduced_period_ms` | 対応: SM-U905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui/src/progress.rs:53` `Progress` | `value`, `fixed_indeterminate`, `moving` | 対応: SM-U905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui/src/radio_group.rs:23` `RadioGroup` | `value`, `layout`, `on_edit`, `item` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/select.rs:31` `Select` | `selected`, `placeholder`, `preview`, `icon`, `option`, `arrow_open`, `retain_trigger_focus`, `touch_sheet` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/slider.rs:50` `Slider` | `range`, `step`, `on_edit` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/slider.rs:198` `Geometry` | `track`, `thumb`, `vertical`, `rtl` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/split.rs:29` `Split` | `grip`, `on_edit` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/split.rs:161` `Group` | `grip`, `on_edit`, `disabled` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/splitter.rs:16` `Splitter` | `start`, `end`, `on_edit` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/switch_drag.rs:25` `Contact` | `started_on`, `visually_on`, `armed`, `flipped`, `horizontal`, `vertical`, `touch` | 対応: SM-U01。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui/src/switch_drag.rs:92` `Gesture` | `value`, `contact`, `visual`, `generation` | 対応: SM-U01。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui/src/tabs.rs:29` `Tabs` | `item`, `content`, `keep_mounted` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/text_draft.rs:16` `Draft` | `on_edit`, `return_focus`, `ended` | 対応: SM-U05。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui/src/text_input.rs:32` `TextInput` | `invalid`, `commit`, `on_edit`, `return_focus`, `ended`, `multiline` | 対応: SM-U901（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui/src/toast_swipe.rs:67` `Swipe` | `rect`, `held`, `contact`, `blocked` | 対応: SM-U02。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui/src/toggle_button.rs:93` `Toggle` | `pressed`, `icon`, `hide_label` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui/src/tooltip.rs:16` `Tooltip` | `enabled`, `align`, `content` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/values.rs:101` `ButtonState` | `loading`, `expanded`, `leading_icon`, `trailing_icon`, `icon_only` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/values.rs:149` `ToggleButtonState` | `pressed`, `leading_icon`, `icon_only` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/values.rs:229` `SwitchDragState` | `offset`, `visually_on`, `touch`, `tracking`, `lifted`, `released` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/values.rs:242` `CheckState` | `invalid`, `text`, `switch_drag` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/values.rs:273` `DialogState` | `open`, `media` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/values.rs:298` `SelectTriggerState` | `open`, `placeholder` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/values.rs:345` `DataTableRowState` | `color`, `height` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/data_table/rows.rs:31` `Grid` | `tree`, `filters`, `indexed`, `selection`, `row_selectable`, `sticky`, `widths`, `actions`, `row_context_menu`, `cells`, `details`, `expanded_rows`, `reorder`, `row_height`, `row_status`, `row_color` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/data_table/rows.rs:63` `StickyColumns` | `rtl`, `overflow` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/data_table/row_status.rs:13` `RowStatus` | `icon`, `color` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/data_table/sticky_paint.rs:9` `DataTableStickyPaint` | `shadow_transition`, `overlay_transition` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/data_table/types.rs:34` `DataTableContext` | `selection`, `sort`, `filter`, `column_filters`, `typed_column_filters`, `page`, `page_size`, `has_more` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui/src/data_table/types.rs:98` `DataTable` | `grouping`, `tree`, `tree_expand_all`, `column_order`, `selection`, `row_selectable`, `sort`, `multi_sort`, `page`, `has_more`, `page_sizes`, `row_index`, `filter`, `column_filters`, `typed_column_filters`, `sticky`, `widths`, `resize_range`, `row_label`, `actions`, `row_context_menu`, `reorder`, `toolbar`, `footer`, `empty`, `details`, `expanded_rows`, `cells`, `row_height`, `row_status`, `row_color` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-basic/src/alert.rs:21` `Alert` | `icon`, `description`, `action` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/attachment.rs:54` `Attachment` | `media`, `description`, `actions`, `on_press` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/attachment.rs:71` `Cfg` | `vertical`, `has_media`, `actionable` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-basic/src/avatar.rs:44` `Avatar` | `image`, `badge` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/badge.rs:33` `Badge` | `icon`, `trailing_icon`, `on_press` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/button_group.rs:12` `ButtonGroup` | `rtl`, `children` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/card.rs:32` `CardHeader` | `description`, `action`, `bordered` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-basic/src/card.rs:97` `Card` | `header`, `image`, `content`, `footer` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/chat.rs:70` `Bubble` | `reactions`, `on_press` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/chat.rs:308` `Message` | `avatar`, `header`, `footer` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/combobox.rs:47` `Combobox` | `placeholder`, `empty`, `clear`, `matcher` | 対応: SM-K16（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-basic/src/date_picker.rs:19` `Picker` | `placeholder`, `format` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-basic/src/date_picker.rs:118` `DatePicker` | `value`, `calendar` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/date_picker.rs:204` `DateRangePicker` | `value`, `calendar` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/disclosure.rs:13` `Collapsible` | `open`, `content` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-basic/src/drawer.rs:29` `Drawer` | `open`, `description`, `content`, `footer` | 対応: SM-K28（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-basic/src/empty.rs:22` `EmptyState` | `media`, `description`, `content` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/field.rs:23` `FieldSet` | `legend`, `description` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/input_group.rs:32` `Around` | `block_start`, `block_end` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-basic/src/input_group.rs:68` `InputGroup` | `start`, `end`, `block_start`, `block_end`, `anchor`, `rtl` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/item.rs:50` `Item` | `rtl`, `media`, `description`, `actions`, `header`, `footer`, `on_press` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/native_select.rs:31` `NativeSelect` | `value`, `placeholder` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/navigation_menu.rs:354` `NavLink` | `description`, `icon`, `on_press` | 対応: SM-K12（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-basic/src/pagination.rs:81` `PaginationLink` | `shown`, `active`, `on_press` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/questionnaire.rs:69` `QuestionnaireStep` | `description`, `optional` | 対応: SM-K914（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-basic/src/table.rs:140` `TableCell` | `header`, `width` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/trace.rs:22` `TraceLine` | `icon`, `description` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/calendar/view.rs:30` `Calendar` | `month`, `disabled`, `outside`, `week_start`, `on_commit`, `autofocus` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/chart/model.rs:100` `Frame` | `horizontal`, `grid`, `axis` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-basic/src/chart/plot.rs:10` `Options` | `horizontal`, `grid`, `axis` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/chart/view.rs:38` `Chart` | `tooltip`, `legend` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/code_block/view.rs:56` `CodeBlock` | `title`, `copy`, `line_numbers`, `footer`, `max_height` | 対応: SM-K24（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:28` `SidebarGroup` | `action`, `content` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:166` `SidebarMenuButton` | `icon`, `active`, `badge`, `action`, `on_press`, `sub` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:180` `ButtonCfg` | `cx`, `active`, `has_action` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-basic/src/sidebar/menu.rs:479` `SidebarMenuSubButton` | `active`, `on_press` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/sidebar/panel.rs:20` `Sidebar` | `header`, `content`, `footer`, `rail` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-basic/src/sidebar/state.rs:55` `SidebarCx` | `open`, `shown` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-basic/tests/consumer/data.rs:78` `Handles` | `sort`, `page`, `visible`, `acted` | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `crates/kivact-ui-creative/src/feedback.rs:254` `WindowOptions` | `title`, `modal`, `icon` | 対応: SM-K30（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/numeric.rs:46` `Numeric` | `logarithmic`, `timecode`, `label_visible`, `accent`, `bipolar`, `horizontal`, `edit`, `text_parser`, `mixed` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-creative/src/outliner_data.rs:10` `TreeItem` | `parent`, `visible`, `selectable`, `rendered`, `placeholder` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/outliner_data.rs:64` `TreeDrop` | `target`, `after` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/picker.rs:53` `PickerItem` | `section`, `description`, `shortcut`, `icon`, `enabled` | 対応: SM-K905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/picker.rs:175` `Parts` | `pick`, `preview`, `cancel` | 対応: SM-K905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/picker.rs:416` `Outcomes` | `pick`, `preview`, `cancel` | 対応: SM-K905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/picker.rs:474` `Session` | `active`, `column` | 対応: SM-K905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/pie.rs:36` `PieState` | `open`, `held`, `hot` | 対応: SM-K901（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/property_row.rs:17` `PropertyRow` | `leading`, `trailing` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-creative/src/range.rs:39` `Range` | `interval`, `movable`, `on_edit` | 対応: SM-K904（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/viewport_surface.rs:61` `ViewportSurface` | `image`, `transform`, `resize`, `pointer`, `overlay`, `pixelated`, `on_edit`, `gpu_frame` | 対応: SM-K918（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/audio/meter.rs:11` `Meter` | `horizontal`, `segmented` | 対応: SM-K908（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/audio/transport.rs:13` `TransportState` | `playing`, `recording`, `cycling` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/audio/view_state.rs:5` `ChannelState` | `mute`, `solo` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/color_wheel/state.rs:20` `State` | `palette`, `drag`, `blocked` | 対応: SM-K902（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/color_wheel/view.rs:34` `ColorWheel` | `alpha`, `fields`, `swatches`, `edit` | 対応: SM-K903（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/dock/data.rs:23` `Shared` | `next_rect`, `on_close` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/dock/data.rs:53` `DockCx` | `on_edit`, `drag`, `root` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/dock/model.rs:53` `DockTab` | `icon`, `closable` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/dock/view.rs:41` `Dock` | `on_close`, `on_edit` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-creative/src/editors/arrangement/gesture.rs:41` `State` | `on_edit`, `cycle`, `cycling`, `snap`, `active_auto` | 対応: SM-K30（CanvasGesture の payload と drag 開始値の写し）。node の focus/data の取消しは SM-K906 |
| `crates/kivact-ui-creative/src/editors/arrangement/model.rs:53` `TimelineTrack` | `muted`, `solo`, `armed`, `automation_read` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/editors/arrangement/view.rs:20` `Arrangement` | `snap`, `add_track`, `on_edit` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-creative/src/editors/graph/gesture.rs:72` `State` | `on_edit`, `snap`, `crate`, `active`, `lane_activate`, `item_activate`, `child_focus` | 対応: SM-K30（CanvasGesture の payload と drag 開始値の写し）。node の focus/data の取消しは SM-K906 |
| `crates/kivact-ui-creative/src/editors/graph/view.rs:21` `GraphEditor` | `lanes`, `snap`, `on_edit` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-creative/src/editors/node/gesture.rs:50` `Info` | `checked`, `selected`, `range` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/editors/node/gesture.rs:112` `State` | `on_edit`, `selected_link`, `snap`, `active`, `blocked` | 対応: SM-K30（CanvasGesture の payload と drag 開始値の写し）。node の focus/data の取消しは SM-K906 |
| `crates/kivact-ui-creative/src/editors/node/model.rs:34` `NodeData` | `preview`, `muted` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/editors/node/view.rs:22` `NodeEditor` | `snap`, `edit` | 対応: SM-K906（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/src/editors/shared/gesture.rs:13` `Pressed` | `target`, `pending`, `double`, `moved` | 対応: SM-K30。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-creative/src/editors/shared/lane.rs:12` `LaneExtensions` | `lane_activate`, `item_activate`, `menu` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/editors/shared/model.rs:20` `EditorContext` | `item`, `lane` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/editors/shared/region_parts.rs:20` `RegionInfo` | `checked`, `selected` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/editors/time/gesture.rs:30` `State` | `on_edit`, `crate`, `snap`, `active`, `child_focus`, `lane_activate`, `item_activate` | 対応: SM-K30（CanvasGesture の payload と drag 開始値の写し）。node の focus/data の取消しは SM-K906 |
| `crates/kivact-ui-creative/src/editors/time/view.rs:22` `TimeEditor` | `lanes`, `snap`, `on_edit` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-creative/src/gradient/state.rs:23` `State` | `selection`, `drag`, `blocked`, `valid` | 対応: SM-K902。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-creative/src/gradient/view.rs:32` `GradientEditor` | `selection`, `edit` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-creative/src/outliner/view.rs:50` `Outliner` | `on_rename`, `on_restriction`, `restrictions`, `activation`, `item_drop`, `active`, `reveal_active`, `row_height` | 対応: SM-K11、SM-K917（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-creative/tests/consumer/arrangement_edit.rs:13` `Model` | `cycling`, `disabled`, `visible`, `present` | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `crates/kivact-ui-creative/tests/consumer/graph_domain.rs:11` `Model` | `snap`, `disabled`, `visible`, `present` | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `crates/kivact-ui-game/src/feedback.rs:134` `Alert` | `actions`, `on_dismiss` | 対応: SM-K33（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-game/src/layout.rs:14` `TreeEntry` | `folder`, `initially_expanded` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-pop/src/display.rs:146` `Avatar` | `halo`, `online` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-pop/src/display.rs:566` `ListItem` | `glyph`, `trailing` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-pop/src/timeline.rs:13` `TimelineItem` | `title_content`, `glyph` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-pop/src/date/calendar.rs:60` `Calendar` | `events`, `on_activate`, `week_starts_on`, `year_navigation`, `year_month_menus`, `month_title`, `range`, `weekend` | 対応: SM-K912（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-pop/src/date/input.rs:47` `DateInput` | `valid`, `calendar` | 対応: SM-K20（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-pop/src/date/range_input.rs:50` `DateRangeInput` | `valid`, `calendar` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-pop/src/date/text.rs:67` `DateText` | `shown`, `checked` | 対応: SM-K913。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/alert_dialog.rs:11` `AlertDialog` | `open`, `action`, `cancel`, `loading` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/app_shell.rs:47` `MobileNavConfig` | `has_toggle`, `open`, `on_change`, `content`, `breakpoint` | 対応: SM-K915（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/app_shell.rs:111` `AppShell` | `top`, `side`, `banner` | 対応: SM-K915（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/avatar.rs:40` `Avatar` | `name`, `alt`, `src`, `fallback_src`, `status`, `press`, `open` | 対応: SM-K32（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/avatar_group.rs:15` `AvatarGroupContext` | `active`, `entries` | 対応: SM-K10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/avatar_group_overflow.rs:16` `AvatarGroupOverflow` | `content`, `press` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/avatar_status_dot.rs:61` `AvatarStatusDot` | `label`, `icon` | 対応: SM-K916（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/banner.rs:35` `Banner` | `description`, `icon`, `end`, `content`, `collapsible`, `open`, `dismissable`, `dismiss`, `dismiss_label` | 対応: SM-K22（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/bottom_sheet.rs:53` `BottomSheet` | `open`, `scrim`, `final_focus`, `content` | 対応: SM-K28（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/breadcrumbs.rs:115` `BreadcrumbItem` | `current`, `icon`, `action`, `menu` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/button_group.rs:17` `GroupMember` | `solid`, `keep_size` | 対応: SM-K10（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/calendar.rs:16` `Calendar` | `month`, `displayed_month`, `touch_months`, `disabled`, `min`, `max`, `max_span`, `variable_rows`, `outside`, `week_start`, `on_commit`, `autofocus` | 対応: SM-K912（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/card.rs:32` `Card` | `padding`, `height`, `disabled` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/carousel.rs:25` `Options` | `looping`, `snap` | 対応: SM-K01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/carousel.rs:177` `Carousel` | `buttons`, `edge_fade`, `handle` | 対応: SM-K01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/citation.rs:13` `CitationSource` | `title`, `url`, `image`, `icon` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/code_block.rs:131` `CodeBlock` | `title`, `language_label`, `line_numbers`, `diff`, `copy_button`, `on_copy`, `end_content`, `wrap`, `max_height`, `collapsible`, `expanded`, `syntax_theme` | 対応: SM-K24（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/collapsible.rs:35` `CollapsibleConfig` | `enabled`, `default_open`, `open`, `value` | 対応: SM-K03、SM-K04（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/collapsible.rs:54` `CollapsibleState` | `is_enabled`, `is_open`, `open` | 対応: SM-K03、SM-K04（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/collapsible.rs:83` `Collapsible` | `open`, `disabled`, `content`, `chevron`, `key`, `keep_mounted`, `animate_layout` | 対応: SM-K03、SM-K04（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/container_reveal.rs:23` `RevealContent` | `inverted`, `preserve_layout`, `force`, `container_force` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/date_input.rs:13` `DateInput` | `value`, `configure` | 対応: SM-K18（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/date_range_input.rs:41` `DateRangeInput` | `value`, `configure`, `presets`, `min`, `max`, `max_span` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/date_time_input.rs:19` `DateTimeInput` | `value`, `seconds` | 対応: SM-K19（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/dialog_header.rs:18` `DialogHeader` | `subtitle`, `start`, `end`, `close`, `compensation`, `divider` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/divider.rs:16` `Divider` | `label`, `full_bleed` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/empty_state.rs:14` `EmptyState` | `description`, `icon`, `actions`, `compact` | 対応: SM-K23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/feedback.rs:215` `StatusDot` | `icon`, `pulsing` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/file_input.rs:16` `FileInput` | `max_bytes`, `multiple`, `dropzone` | 対応: SM-K31（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/grid.rs:25` `Grid` | `row_gap`, `column_gap`, `row_height` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/hover_card.rs:22` `HoverCard` | `label`, `enabled`, `open`, `default_open`, `on_open_change` | 対応: SM-K14（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/indicator.rs:22` `Indicator` | `disabled`, `children` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/layout.rs:6` `LayoutInset` | `outer_x`, `outer_y`, `inner_x`, `inner_y`, `own_outer_x`, `own_outer_y` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/layout.rs:47` `Layout` | `header`, `footer`, `start`, `end`, `content`, `start_resize`, `end_resize`, `padding`, `content_width`, `default_dividers` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/layout.rs:133` `LayoutSlots` | `root_ref`, `header_divider`, `footer_divider`, `has_start`, `has_end`, `content_width` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/layout.rs:262` `LayoutRegion` | `divider`, `padding`, `padding_bottom`, `scrollable`, `resizable`, `width` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/lightbox.rs:32` `Lightbox` | `open`, `index`, `zoom` | 対応: SM-K05（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/lightbox.rs:64` `LightboxHandle` | `open`, `index` | 対応: SM-K05（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/link.rs:28` `Link` | `label`, `press`, `new_tab_label`, `underline`, `disabled`, `external`, `standalone`, `size`, `weight`, `as_button` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/markdown.rs:55` `Options` | `inline`, `on_link`, `width`, `heading_links`, `soft_breaks`, `autolink`, `render_code`, `render_image`, `render_heading` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/markdown.rs:730` `Marks` | `strong`, `italic`, `deleted`, `code` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/markdown_parser.rs:15` `ListEntry` | `number`, `checked` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/mobile_nav.rs:18` `AppShellMobile` | `mobile`, `open`, `toggle`, `drawer`, `auto_toggle` | 対応: SM-K09（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/mobile_nav.rs:71` `MobileNav` | `open`, `label`, `header`, `header_view` | 対応: SM-K09（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/mobile_nav.rs:276` `MobileNavToggle` | `label`, `icon` | 対応: SM-K09（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/month_swipe.rs:25` `MonthSwipe` | `contact`, `suppress_choice` | 対応: SM-K28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/more_menu.rs:11` `MoreMenu` | `icon`, `disabled` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/nav_heading.rs:23` `NavHeadingMenuItem` | `description`, `icon`, `disabled`, `action` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/nav_heading.rs:306` `SideNavHeading` | `superheading`, `subheading`, `icon`, `end`, `menu`, `heading_action`, `super_action`, `sub_action` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/outline.rs:50` `Outline` | `active`, `viewport`, `content`, `scroll_on_press`, `change`, `start`, `end` | 対応: SM-K26（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/outline.rs:144` `Navigation` | `viewport`, `scroll_on_press`, `controlled`, `pending`, `start`, `end` | 対応: SM-K26（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/pagination.rs:28` `Pagination` | `page_size`, `total_items`, `total_pages`, `has_more`, `disabled`, `page_label`, `first_last`, `on_change` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/pagination.rs:130` `Model` | `total`, `has_more`, `disabled`, `change` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/resizable.rs:38` `ResizableConfig` | `container`, `collapsible`, `default_collapsed`, `collapsed` | 対応: SM-K06（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/resizable.rs:204` `ResizeHandle` | `reversed`, `divider`, `always_visible`, `overlay`, `grip` | 対応: SM-K06（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/rich_blocks.rs:20` `Editor` | `focus_after_mount`, `target`, `composing`, `whole` | 対応: SM-K909（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/rich_text.rs:176` `RichTextEditor` | `read_only`, `block_editing` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/scrollable_area.rs:31` `ScrollableArea` | `full_bleed`, `bleed_top`, `bleed_bottom`, `force_focusable` | 対応: SM-K07（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/scrollable_area.rs:161` `AxisScroll` | `is_scrollable`, `at_start`, `at_end` | 対応: SM-K07（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/scrollable_area.rs:216` `ScrollOverflow` | `start`, `end`, `any` | 対応: SM-K07（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/section.rs:44` `Section` | `bleed_top`, `bleed_bottom` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/segmented_control.rs:34` `SegmentedControl` | `icon_only`, `disabled`, `disabled_reason` | 対応: SM-K27（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/side_nav.rs:29` `SideNavCx` | `collapsed`, `popup` | 対応: SM-K13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/side_nav.rs:50` `SideNavResize` | `default_collapsed`, `collapsed` | 対応: SM-K13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/side_nav.rs:70` `SideNav` | `header`, `top`, `footer`, `icons`, `collapse`, `resize`, `collapse_button` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/side_nav.rs:341` `SideNavCollapseButton` | `binding`, `label` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/side_nav.rs:427` `SideNavItem` | `icon`, `selected_icon`, `selected`, `disabled`, `end`, `actions`, `children`, `action` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/side_nav.rs:878` `SideNavSection` | `subtitle`, `end`, `header_hidden` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_chat.rs:42` `ChatMessage` | `avatar`, `metadata`, `name` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_chat.rs:176` `ChatBubble` | `group`, `width`, `name`, `metadata` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_chat.rs:341` `ChatMetadata` | `timestamp`, `footer`, `status` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_chat.rs:517` `ChatMessageList` | `empty`, `is_empty`, `loading_top`, `streaming`, `gap` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_chat.rs:627` `ChatLayout` | `composer`, `scroll_button`, `viewport`, `history_revision` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_chat_composer.rs:36` `ChatSendButton` | `stop_shown`, `disabled`, `on_send`, `on_stop`, `send_icon`, `stop_icon` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_chat_composer.rs:187` `ChatComposer` | `on_submit`, `on_stop`, `stop_shown`, `disabled`, `drawer`, `header_actions`, `header_context`, `footer_actions`, `send_actions`, `status`, `history` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_chat_composer.rs:485` `ChatComposerDrawer` | `count`, `collapsed`, `summary` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_chat_composer.rs:591` `DictationState` | `supported`, `listening` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/soft_chat_composer.rs:598` `ChatDictationButton` | `on_toggle`, `hide_when_unsupported` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_chat_tools.rs:31` `ToolCall` | `target`, `duration`, `error` | 対応: SM-K08（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/soft_chat_tools.rs:81` `ChatToolCalls` | `label`, `expanded`, `detail` | 対応: SM-K08（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/soft_list.rs:57` `Item` | `label_view`, `description`, `description_view`, `start`, `end`, `marker`, `on_press`, `delegate`, `disabled`, `selected`, `highlighted`, `list_item` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_list.rs:418` `List` | `header`, `dividers` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_metadata.rs:164` `MetadataList` | `position`, `label_width`, `max_items`, `expanded`, `title` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_overflow.rs:35` `OverflowList` | `item_width`, `on_overflow_change` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_table.rs:50` `TableRow` | `dividers`, `hover`, `striped` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_table.rs:121` `TableCell` | `header`, `width`, `divider` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_table.rs:231` `TableSelectionToolbar` | `selection_label`, `actions` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/soft_tree.rs:31` `TreeList` | `selection`, `activation`, `header`, `renderer`, `animated` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/spinner.rs:33` `Spinner` | `label`, `named`, `ink` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/stack.rs:51` `Padding` | `all`, `x`, `y`, `start`, `end`, `top`, `bottom` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/stack.rs:219` `Stack` | `scrollable`, `horizontal`, `vertical` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/stepper.rs:66` `Step` | `description`, `content`, `end`, `disabled`, `optional`, `density` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/stepper.rs:207` `StepperCx` | `compact`, `vertical`, `action` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/stepper.rs:347` `Summary` | `description`, `disabled` | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `crates/kivact-ui-soft/src/tab_list.rs:48` `Tab` | `icon`, `selected_icon`, `label_hidden`, `end`, `disabled`, `panel` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/tab_list.rs:108` `TabMenuOption` | `icon`, `disabled` | 対応: SM-K10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/tab_list.rs:178` `TabList` | `full_bleed`, `content` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/tab_list.rs:260` `FocusMember` | `selected`, `disabled` | 対応: SM-K10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/tab_list.rs:267` `FocusTrack` | `focused_within`, `roving` | 対応: SM-K10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/text.rs:102` `Text` | `heading`, `size`, `color`, `weight`, `wrap`, `strikethrough`, `tabular_numbers`, `capsize` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/thumbnail.rs:28` `Thumbnail` | `source`, `alt`, `label`, `press`, `remove`, `loading`, `disabled` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/timestamp.rs:145` `TimestampEntry` | `zone`, `label`, `copyable` | 対応: SM-K910。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/timestamp.rs:171` `Timestamp` | `details`, `show_zone`, `live` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/time_input.rs:50` `TimeInput` | `value`, `seconds` | 対応: SM-K911（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/toggle_button_group.rs:83` `ToggleButtonGroup` | `disabled`, `icon_only`, `pressed_icon` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/token.rs:52` `Token` | `icon`, `end`, `press`, `remove`, `description`, `label_hidden`, `disabled`, `as_link` | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `crates/kivact-ui-soft/src/toolbar.rs:32` `Toolbar` | `start`, `center`, `end` | 対応: SM-K25（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/top_nav.rs:34` `Slots` | `heading`, `start`, `center`, `end` | 対応: SM-K13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/top_nav.rs:175` `TopNavItem` | `content`, `icon`, `icon_only`, `selected`, `disabled`, `action` | 対応: SM-K13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/top_nav.rs:436` `TopNavMegaMenu` | `items`, `featured`, `on_change` | 対応: SM-K13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/top_nav.rs:590` `TopNavMegaMenuItem` | `description`, `icon`, `action` | 対応: SM-K13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/top_nav.rs:724` `TopNavMegaMenuFeaturedCard` | `description`, `image`, `content`, `link` | 対応: SM-K13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-ui-soft/src/typeahead.rs:26` `Typeahead` | `chip_labels`, `chips`, `search`, `exclude_selected` | 対応: SM-K16、SM-K919（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `crates/kivact-wire/src/drawing.rs:17` `DrawingLayer` | `clip`, `mask` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/events.rs:26` `PointerEvent` | `pressure`, `tilt`, `is_primary` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/events.rs:63` `HostEnv` | `reduced_motion`, `locale`, `fullscreen`, `surface_active` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/grid.rs:12` `GridProps` | `container`, `justify_self` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/host.rs:29` `TurnOutput` | `next_wake_ns`, `exit` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/measure.rs:34` `MeasureRequest` | `known_width`, `known_height` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:44` `NodeProperties` | `editing`, `ime`, `layout`, `paint`, `text`, `states`, `semantics` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:85` `NodeInput` | `disabled`, `hidden`, `focusable`, `focusable_when_disabled`, `roving_focus`, `listen_pointer`, `focus_on_press`, `listen_wheel`, `listen_file`, `hit_testable` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:119` `NodeSemantics` | `role`, `name`, `description`, `value`, `selected`, `checked`, `expanded`, `invalid`, `accessibility`, `actions`, `test_id` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:146` `BoxData` | `intrinsic`, `measure` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:155` `TextData` | `content`, `spans` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:164` `EditingData` | `behavior`, `colors`, `forward_keys`, `value`, `password`, `read_only`, `max_length`, `placeholder`, `multiline`, `line_layout`, `report_text`, `report_session` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:193` `EditLineLayout` | `wrap`, `rows` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:200` `ImageData` | `image`, `fit`, `monochrome`, `intrinsic` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:276` `OverlayFit` | `keep_anchor_clear`, `rtl` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:307` `OverlayProps` | `fit`, `modal` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/node.rs:345` `FileDialogRequest` | `directory`, `file_name` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/paint.rs:48` `FillLayer` | `repeat_x`, `repeat_y` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/paint.rs:314` `Effects` | `mask`, `clip_path` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/scene.rs:16` `TextRun` | `family`, `italic`, `line_height` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/scene.rs:114` `Region` | `focusable`, `focus_on_press`, `disabled`, `hidden`, `description`, `checked`, `selected`, `expanded`, `invalid`, `properties`, `shape`, `value` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/semantics.rs:16` `SemanticsProperties` | `value_min`, `value_max`, `value_now`, `live_region`, `active_descendant`, `level`, `set_size`, `pos_in_set`, `multiselectable`, `busy`, `required`, `read_only`, `has_popup`, `current_item`, `text` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/style.rs:13` `LayoutProps` | `align_items`, `align_self`, `justify`, `align_content`, `wrap`, `hidden`, `aspect_ratio`, `grid` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/style.rs:171` `PaintProps` | `transition`, `background`, `border`, `ring`, `scrollbar`, `clip`, `effects` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/style.rs:218` `PaintDelta` | `background`, `border`, `ring`, `radius`, `opacity`, `shadows`, `transform`, `z_index`, `cursor`, `scrollbar`, `effects`, `text_color` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/style.rs:237` `StateStyles` | `hover`, `pressed`, `focus`, `focus_visible`, `group_hover`, `group_focus`, `group`, `enter`, `exit` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/style.rs:297` `ScrollbarDelta` | `width`, `colors` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/text.rs:54` `TextDecoration` | `color`, `thickness` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/text.rs:90` `TextProps` | `font_size`, `weight`, `color`, `line_height`, `wrap`, `align`, `family`, `italic`, `letter_spacing`, `word_spacing`, `kerning`, `features`, `tabular_nums`, `decoration_lines`, `decoration_color`, `decoration_thickness`, `underline_offset`, `overflow`, `max_lines`, `capsize`, `transform`, `shadows`, `caret_color`, `selection_color`, `placeholder_color` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/text.rs:227` `ComputedText` | `line_height`, `family`, `italic`, `more` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/text.rs:255` `TextMore` | `max_lines`, `capsize`, `caret_color`, `selection_color`, `placeholder_color` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `crates/kivact-wire/src/text.rs:330` `SpanProps` | `font_size`, `weight`, `color`, `family`, `italic`, `letter_spacing`, `word_spacing`, `kerning`, `features`, `decoration`, `background` | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `examples/kivact-ui-creative/src/creative/audio_values.rs:11` `TransportState` | `playing`, `recording`, `cycling` | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |
| `examples/kivact-ui-game/src/game/frame.rs:12` `NativeWindow` | `menu_bar`, `toolbar`, `status_bar` | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |
| `examples/kivact-ui-game/src/game/state.rs:6` `Demo` | `create`, `delete`, `drawer`, `palette` | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |
| `examples/kivact-ui-pop/src/pop/messaging.rs:19` `ChatEntry` | `mine`, `typing` | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |
| `examples/kivact-ui-pop/src/pop/mission.rs:9` `Mission` | `progress_label`, `action_label`, `locked` | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |
| `examples/kivact-ui-soft/src/model.rs:7` `CatalogState` | `dark`, `selected` | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |

### 文字列検索の全 path 台帳

指定された wrapper/Signal/Var/bool の全一致行。型の判定は上の台帳と付録 A、引数/import/doc は field ではないため除外。

| path | line | 判断の入口 |
|---|---|---|
| `crates/arvane-asset/src/fit.rs` | 27 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-asset/src/font.rs` | 210 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-asset/src/store/asset_store.rs` | 174, 572 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-native/src/a11y/conversion.rs` | 92 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-native/src/a11y/native.rs` | 77, 102 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-native/src/embed/windows/create.rs` | 206 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-native/src/embed/windows/state.rs` | 46, 49, 67, 69, 72, 77, 85 | 対応: SM-A01、SM-A02、SM-A03。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/arvane-native/src/embed/windows/test_parent.rs` | 679 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-native/src/embed/x11/dnd.rs` | 42, 407 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-native/src/embed/x11/event.rs` | 16 | 対応: SM-A05。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/arvane-native/src/embed/x11/ime.rs` | 53, 54, 110, 480 | 対応: SM-A06。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/arvane-native/src/embed/x11/test_parent.rs` | 26, 740 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-native/src/embed/x11/window.rs` | 21, 26 | 対応: SM-A01、SM-A05。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/arvane-native/src/module.rs` | 14, 18 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-native/src/native_input/state.rs` | 35, 40, 43, 44, 48, 50, 54, 57, 60, 77 | 対応: SM-A08 |
| `crates/arvane-native/src/native_input/windows.rs` | 516 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-native/src/runtime.rs` | 525 | 対応: SM-A07、SM-A10 |
| `crates/arvane-native/src/test_display.rs` | 47, 115, 119, 121, 169, 382 | 対応: SM-A15 |
| `crates/arvane-native/src/wheel.rs` | 7 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-render/src/draw/canvas.rs` | 64, 71 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-render/src/draw/painter.rs` | 46 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-render/src/gpu/encode.rs` | 101, 143, 203, 544 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-render/src/gpu/external.rs` | 92 | 対応: SM-A14 |
| `crates/arvane-render/src/gpu/state.rs` | 153, 171 | 対応: SM-A12 |
| `crates/arvane-render/src/gpu/surface.rs` | 376, 391 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-render/src/gradient.rs` | 27 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-render/src/paint.rs` | 50, 52 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-render/src/validate/scene.rs` | 239 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-shell/src/a11y/shared.rs` | 22, 151, 269 | 対応: SM-A13 |
| `crates/arvane-shell/src/events.rs` | 62 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-shell/src/ime.rs` | 18 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-shell/src/mouse.rs` | 52 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-shell/src/types.rs` | 11 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-text/src/cache.rs` | 95, 108, 110, 125, 128, 131, 142 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-text/src/layouts.rs` | 92, 94 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-text/src/output.rs` | 283 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane-text/src/text_engine.rs` | 246, 317 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane/src/embed.rs` | 23 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane/src/env.rs` | 36 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane/src/presentation.rs` | 26 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane/src/presenter/frame_policy.rs` | 21, 348 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane/src/presenter/gpu.rs` | 232 | 対応: SM-A11 |
| `crates/arvane/src/presenter/test_frames.rs` | 69 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane/src/test_display/shell.rs` | 25, 26, 38, 40, 42, 44, 47, 49, 52, 63, 68 | 対応: SM-A15 |
| `crates/arvane/src/test_display/virtual_window.rs` | 51, 53, 61 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/arvane/src/window.rs` | 342, 403, 406, 425 | 対応: SM-A10 |
| `crates/arvane/tests/e2e/embed.rs` | 18 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/client/dispatch/env.rs` | 51 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/client/dispatch/text.rs` | 56 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/client/link.rs` | 16, 19, 30, 32 | 対応: SM-R04 |
| `crates/kivact-core/src/primitive/node.rs` | 17 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/test/options.rs` | 52, 54, 68 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/test/screen/driver.rs` | 33, 50, 113 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/test/screen/gpu_tests.rs` | 307, 540 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/test/screen/host_sync_tests.rs` | 184, 292, 341, 344 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/test/screen/model.rs` | 28 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/test/screen/platform_tests.rs` | 23 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/test/screen/real_window_tests.rs` | 139, 171, 220 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/test/screen/results.rs` | 301, 302, 303 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/test/screen/scene_cache_tests.rs` | 18, 161, 168 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/view/common.rs` | 63, 81, 85, 175, 186, 216, 217 | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/editing_data.rs` | 94, 96 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/view/flow/branch.rs` | 308 | 対応: SM-C19。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/flow/error_boundary.rs` | 145 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/view/flow/virtual_list.rs` | 68, 442 | 対応: SM-C18、SM-C20。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/ime_session.rs` | 59 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/view/input.rs` | 132, 201 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/view/mount/create.rs` | 23, 32 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/view/mount/editing.rs` | 18 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/view/mount/facts.rs` | 15, 22 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/view/mount/part_input.rs` | 42, 47 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-core/src/view/mount/style.rs` | 75 | 対応: SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/node_ref.rs` | 245 | 対応: SM-C14。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/overlays.rs` | 146, 147, 510 | 対応: SM-C22。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/part.rs` | 78, 81, 83, 86, 88, 90, 112, 113, 114, 115, 116 | 対応: SM-C15。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/requests.rs` | 45, 205 | 対応: SM-C10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/state.rs` | 10, 67, 86, 148, 225, 233, 250 | 対応: SM-C08、SM-C09、SM-C10、SM-C14、SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/sync_hook.rs` | 35 | 対応: SM-C21。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-core/src/view/tree.rs` | 194, 211, 289 | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-editing/src/behavior.rs` | 16, 24, 41, 43, 45, 53, 54, 55 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-editing/src/edit_state.rs` | 56 | 対応: SM-E32 |
| `crates/kivact-engine/src/a11y/build/node.rs` | 259, 429 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/a11y/publish.rs` | 19 | 対応: SM-E25。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/check/boxes.rs` | 69, 70 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/check/findings.rs` | 29, 81 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/check/lint/changed_texts.rs` | 41 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/check/shaped_lines.rs` | 19, 46 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/host/pump.rs` | 68, 71, 76 | 対応: SM-E01 |
| `crates/kivact-engine/src/host/testing/test_options.rs` | 45, 52, 57 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/host/write/writer.rs` | 125, 165 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/measure/editor.rs` | 11, 13 | 対応: SM-E19 |
| `crates/kivact-engine/src/measure/leaf.rs` | 319, 404 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/measure/state.rs` | 15, 16, 17 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/model/apply.rs` | 102, 365, 407 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/model/changes.rs` | 118, 119, 120, 121, 122, 125, 126, 128, 135, 136, 137, 138, 141 | 対応: SM-E24、SM-E27。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/edit/state.rs` | 19, 20 | 対応: SM-E20、SM-E32。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/focus/navigation.rs` | 186, 229, 231 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/model/focus/state.rs` | 12, 14 | 対応: SM-E23。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/frame/ticks.rs` | 15 | 対応: SM-E04。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/keys.rs` | 121, 287 | 対応: SM-E16。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/pointer/shell.rs` | 154 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/model/pointer/state.rs` | 219, 240 | 対応: SM-E10、SM-E11、SM-E12、SM-E13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/model/scrollbar.rs` | 23 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/model/state_motion/state.rs` | 57, 59 | 対応: SM-E31。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/mutation.rs` | 98, 100, 103, 105, 107, 109, 111, 113, 115, 117 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/paint/frame_scene.rs` | 221 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/paint/list/invalidation.rs` | 25, 49, 95, 96, 97 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-engine/src/paint/state.rs` | 22, 30, 94, 98, 127, 130, 132, 136, 171, 200 | 対応: SM-E08、SM-E09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/tree/node.rs` | 59, 63, 67, 71, 73 | 対応: SM-E30、SM-E31。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-engine/src/tree/node_tree.rs` | 219 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-gpu-share/src/d3d12/pool.rs` | 149 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-gpu-share/src/import/device.rs` | 12 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-gpu-share/src/import/dx12.rs` | 88 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-gpu-share/src/import/vulkan.rs` | 242, 412 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-gpu-share/src/pool.rs` | 35 | 対応: SM-A14 |
| `crates/kivact-gpu-share/src/vulkan/pool.rs` | 171 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-gpu-share/src/vulkan_raw.rs` | 81 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-gpu-share/src/wgpu/pool.rs` | 208 | 対応: SM-A14 |
| `crates/kivact-interaction/src/calendar/handle.rs` | 38, 40, 44 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-interaction/src/calendar/policy.rs` | 28 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-interaction/src/delayed.rs` | 168 | 対応: SM-I03。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/dialog.rs` | 77, 79, 158 | 対応: SM-I05。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/disclosure.rs` | 24 | 対応: SM-I01。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/drag.rs` | 103, 113, 125, 220, 256, 577 | 対応: SM-I12。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/follow_end.rs` | 93, 98 | 対応: SM-I23。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/hover_corridor.rs` | 129 | 対応: SM-I21。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/list/data.rs` | 40, 248, 303, 304 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-interaction/src/list/input.rs` | 154, 182, 213 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-interaction/src/list/parts.rs` | 38 | 対応: SM-I14。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/list/state.rs` | 58, 61, 62, 64 | 対応: SM-I14、SM-I26。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/list/typeahead.rs` | 26 | 対応: SM-I15。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/menu/level.rs` | 62 | 対応: SM-I10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/menu/order.rs` | 119 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-interaction/src/menu/press.rs` | 141 | 対応: SM-I09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/menu/state.rs` | 63, 66, 129, 148, 155, 158 | 対応: SM-I08。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/menubar.rs` | 60 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-interaction/src/palette.rs` | 80, 242 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-interaction/src/popup.rs` | 54, 116, 130 | 対応: SM-I02。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/roving.rs` | 51, 55, 69, 71 | 対応: SM-I16。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/scalar/touch.rs` | 31 | 対応: SM-I18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/scalar/track.rs` | 87, 134 | 対応: SM-I901 |
| `crates/kivact-interaction/src/select.rs` | 120, 121 | 対応: SM-I07。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/space_pan.rs` | 23 | 対応: SM-I20。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/splitter.rs` | 32 | 対応: SM-I28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/table/data.rs` | 15, 83, 92, 95, 103, 105 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-interaction/src/tabs.rs` | 45 | 対応: SM-I28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/text_edit/handle.rs` | 325, 326, 328, 330 | 対応: SM-I13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/toaster.rs` | 56, 263, 290, 465 | 対応: SM-I22。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/tooltip.rs` | 80 | 対応: SM-I04。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-interaction/src/tree.rs` | 48, 144, 319, 326, 327 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-layout/src/columns.rs` | 32, 37, 46 | 対応: SM-E28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-layout/src/pass_state.rs` | 28, 32 | 対応: SM-E28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-layout/src/placement.rs` | 77 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-layout/src/style/convert.rs` | 107 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-layout/src/traversal.rs` | 36 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-layout/src/tree.rs` | 651 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-lower/src/border.rs` | 200, 201 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-lower/src/context.rs` | 54 | 対応: SM-E33 |
| `crates/kivact-lower/src/glyphs.rs` | 103 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-motion/src/animated.rs` | 24, 33 | 対応: SM-R25 |
| `crates/kivact-motion/src/state.rs` | 63, 300 | 対応: SM-R22、SM-R23、SM-R24、SM-C23 |
| `crates/kivact-reactive/src/data.rs` | 20, 60, 68, 74, 77, 128, 191, 290, 314 | 対応: SM-R01 |
| `crates/kivact-reactive/src/frame.rs` | 44, 45 | 対応: SM-R08。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-reactive/src/graph.rs` | 200, 234 | 対応: SM-R01、SM-R07。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-reactive/src/handles.rs` | 32, 351 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-reactive/src/host.rs` | 20, 22, 24, 26 | 対応: SM-R10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-reactive/src/runtime.rs` | 4, 28 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-reactive/src/selector.rs` | 21 | 対応: SM-R12。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-style/src/draw/image.rs` | 55 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/draw/recorder.rs` | 165, 169, 174, 176, 178 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/draw/text_run.rs` | 44, 63 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/effect.rs` | 36, 204 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/paint/gradient.rs` | 24, 62 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/path/commands.rs` | 13, 59, 77, 104, 105 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/path/geometry.rs` | 19 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/path/trim.rs` | 19, 22 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/resolve.rs` | 198, 201, 203, 206, 455 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/sheet.rs` | 249 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/text.rs` | 273 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-style/src/theme/value.rs` | 45 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-task/src/action.rs` | 199 | 対応: SM-R13 |
| `crates/kivact-task/src/latest.rs` | 21 | 対応: SM-R14 |
| `crates/kivact-task/src/spawn.rs` | 3, 50 | 対応: SM-R17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-task/src/state.rs` | 8, 50, 74 | 対応: SM-R17、SM-R18、SM-R21、SM-C23 |
| `crates/kivact-text/src/chunks.rs` | 20, 21 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-text/src/records.rs` | 25 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-text/src/source.rs` | 19, 21 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/gpu_interop.rs` | 292 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/input.rs` | 52, 54, 56, 58 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/platform.rs` | 156, 158, 160, 162, 164, 167, 170, 174 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/protocol/effect.rs` | 107, 109 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/protocol/paint.rs` | 211, 213 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/protocol/scene.rs` | 49, 68 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/protocol/semantics.rs` | 45, 47, 49, 53, 152, 154, 160, 162, 164 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/protocol/shape.rs` | 35, 37, 525, 526 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/protocol/shaped.rs` | 133 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/protocol/text.rs` | 74, 92, 161 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-types/src/protocol/window.rs` | 71, 73 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/alert_dialog.rs` | 21 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/attachment.rs` | 73, 74, 75 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/badge.rs` | 150 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/button_group.rs` | 15 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/calendar/panel.rs` | 157 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/calendar/view.rs` | 40 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/chart/cartesian.rs` | 27 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/chart/model.rs` | 104, 105, 106 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/chart/plot.rs` | 12 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/choice.rs` | 33, 34, 35 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/drawer.rs` | 44 | 対応: SM-K28 |
| `crates/kivact-ui-basic/src/icons.rs` | 278 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/input_group.rs` | 33, 34, 40, 77, 153, 179 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/input_otp.rs` | 218 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/item.rs` | 53 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/navigation_menu.rs` | 233 | 対応: SM-K12 |
| `crates/kivact-ui-basic/src/questionnaire.rs` | 73 | 対応: SM-K914 |
| `crates/kivact-ui-basic/src/recipes.rs` | 24, 227 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/recipes_menu.rs` | 26 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/sidebar/menu.rs` | 184 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/src/table.rs` | 142, 150 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/tests/consumer/navigation.rs` | 68 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-basic/tests/consumer/navigation_layout.rs` | 25, 109, 299 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/audio/transport.rs` | 14, 15, 16 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/color_wheel/fields.rs` | 49 | 対応: SM-K903。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-creative/src/color_wheel/state.rs` | 15, 27 | 対応: SM-K902 |
| `crates/kivact-ui-creative/src/dock/chrome.rs` | 442 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/dock/geometry.rs` | 202 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/dock/items.rs` | 13, 14, 19, 25 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/dock/model.rs` | 58 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/arrangement/model.rs` | 58, 59, 60, 61, 118 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/graph/gesture.rs` | 35, 45 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/node/gesture.rs` | 57 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/node/layout.rs` | 16 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/node/model.rs` | 47 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/node/paint.rs` | 17 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/shared/context.rs` | 11, 33, 34 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/shared/gesture.rs` | 20, 21 | 対応: SM-K30。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-creative/src/editors/shared/lane.rs` | 88 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/shared/model.rs` | 35 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/shared/region_parts.rs` | 25, 26 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/shared/texts.rs` | 63, 72 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/editors/time/model.rs` | 15 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/feedback.rs` | 258 | 対応: SM-K30 |
| `crates/kivact-ui-creative/src/gradient/state.rs` | 29 | 対応: SM-K902。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-creative/src/icons.rs` | 310 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/numeric.rs` | 756 | 対応: SM-K30。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-creative/src/outliner/controls.rs` | 39 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/outliner_data.rs` | 16, 17, 18, 22, 87 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/src/panel_group.rs` | 34 | 対応: SM-K30。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-creative/src/picker.rs` | 69 | 対応: SM-K905 |
| `crates/kivact-ui-creative/src/pie.rs` | 37, 38 | 対応: SM-K901 |
| `crates/kivact-ui-creative/src/recipes.rs` | 23 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/tests/consumer/arrangement_edit.rs` | 19, 25, 26, 27 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-creative/tests/consumer/graph_domain.rs` | 17, 19, 20, 21 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-game/src/display.rs` | 460 | 対応: SM-K33。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-game/src/icons.rs` | 143 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-game/src/input.rs` | 128 | 対応: SM-K31 |
| `crates/kivact-ui-game/src/layout.rs` | 19, 21, 67 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-game/src/recipes.rs` | 20, 49, 156, 157 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-game/src/theme.rs` | 19 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-pop/src/button.rs` | 37 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-pop/src/choice.rs` | 97, 111, 112 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-pop/src/date/text.rs` | 76 | 対応: SM-K913。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-pop/src/icons.rs` | 225 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-pop/src/messaging.rs` | 226 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-pop/src/notifications.rs` | 25 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-pop/src/recipes.rs` | 34 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/app_shell.rs` | 48 | 対応: SM-K915 |
| `crates/kivact-ui-soft/src/banner.rs` | 45, 47 | 対応: SM-K22 |
| `crates/kivact-ui-soft/src/bottom_sheet.rs` | 59 | 対応: SM-K28 |
| `crates/kivact-ui-soft/src/button_group.rs` | 19, 20, 146, 147, 204 | 対応: SM-K10 |
| `crates/kivact-ui-soft/src/calendar.rs` | 28, 29, 32 | 対応: SM-K912 |
| `crates/kivact-ui-soft/src/carousel.rs` | 26, 27, 183, 184 | 対応: SM-K01 |
| `crates/kivact-ui-soft/src/code.rs` | 21 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/code_block.rs` | 136, 139, 140, 145, 147, 247, 495, 546, 548, 549 | 対応: SM-K24 |
| `crates/kivact-ui-soft/src/collapsible.rs` | 36, 37, 55, 92, 93, 333 | 対応: SM-K03、SM-K04 |
| `crates/kivact-ui-soft/src/container_reveal.rs` | 24, 25 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/date_time_input.rs` | 24 | 対応: SM-K19 |
| `crates/kivact-ui-soft/src/date_touch.rs` | 80, 96, 298, 313 | 対応: SM-K29 |
| `crates/kivact-ui-soft/src/dialog_header.rs` | 25 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/divider.rs` | 21 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/feedback.rs` | 219 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/field_status.rs` | 29 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/file_input.rs` | 22, 23 | 対応: SM-K31 |
| `crates/kivact-ui-soft/src/hover_card.rs` | 31 | 対応: SM-K14 |
| `crates/kivact-ui-soft/src/icons.rs` | 3, 104 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/input_group.rs` | 25, 26, 27 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/layout.rs` | 59, 137, 138, 269 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/lightbox.rs` | 36, 65 | 対応: SM-K05 |
| `crates/kivact-ui-soft/src/link.rs` | 34, 35, 36, 37, 43 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/markdown.rs` | 56, 64, 65, 528, 731, 732, 733, 734 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/markdown_parser.rs` | 28 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/month_swipe.rs` | 34, 42 | 対応: SM-K28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/nav_heading.rs` | 27 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/outline.rs` | 58, 147 | 対応: SM-K26 |
| `crates/kivact-ui-soft/src/recipes.rs` | 3, 6 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/resizable.rs` | 44, 47, 153, 208, 209, 210, 211 | 対応: SM-K06 |
| `crates/kivact-ui-soft/src/rich_blocks.rs` | 26, 27, 34, 69, 294, 295, 335 | 対応: SM-K909 |
| `crates/kivact-ui-soft/src/rich_text.rs` | 45, 46, 180, 181 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/scrollable_area.rs` | 36, 37, 38, 175, 235 | 対応: SM-K07 |
| `crates/kivact-ui-soft/src/section.rs` | 50, 51 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/segmented_control.rs` | 38, 39 | 対応: SM-K27 |
| `crates/kivact-ui-soft/src/side_nav.rs` | 54, 79, 827, 828, 829, 884 | 対応: SM-K13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/soft_chat.rs` | 632 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/soft_chat_composer.rs` | 204, 592, 593, 603 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/soft_list.rs` | 76, 412, 423 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/soft_table.rs` | 52, 53, 54, 123, 130 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/soft_tree.rs` | 42 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/spinner.rs` | 38 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/stack.rs` | 227, 399 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/stepper.rs` | 74, 210, 387, 427, 705 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/tab_list.rs` | 54, 112, 175, 186, 598, 716 | 対応: SM-K10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/text.rs` | 115, 116 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/time_input.rs` | 55 | 対応: SM-K911 |
| `crates/kivact-ui-soft/src/timestamp.rs` | 149, 177, 180, 181, 396, 429, 430 | 対応: SM-K910。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui-soft/src/toggle_button_group.rs` | 90 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/token.rs` | 62, 63, 64 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui-soft/src/toolbar.rs` | 267 | 対応: SM-K25 |
| `crates/kivact-ui-soft/src/top_nav.rs` | 180, 289, 290, 291, 292 | 対応: SM-K13 |
| `crates/kivact-ui-soft/src/typeahead.rs` | 37 | 対応: SM-K16、SM-K919 |
| `crates/kivact-ui/src/button.rs` | 88, 90, 96, 97, 98 | 対応: SM-U905。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui/src/command_palette.rs` | 54 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/data_column.rs` | 53, 54 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/data_table/filters.rs` | 156, 243 | 対応: SM-U904 |
| `crates/kivact-ui/src/data_table/group_state.rs` | 8 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/data_table/rows.rs` | 65, 169, 672 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/data_table/sticky_paint.rs` | 33 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/data_table/tree.rs` | 27, 73 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/data_table/types.rs` | 105 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/dialog.rs` | 75, 129 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/disclosure.rs` | 26, 27 | 対応: SM-U06 |
| `crates/kivact-ui/src/menu.rs` | 44, 175, 176, 261, 262, 264 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/progress.rs` | 23, 56 | 対応: SM-U905 |
| `crates/kivact-ui/src/recipes.rs` | 62, 99 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/select.rs` | 42, 43 | 対応: SM-U03。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui/src/slider.rs` | 201, 202 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/switch_drag.rs` | 28, 29, 30, 31, 32, 33, 34, 97 | 対応: SM-U01。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui/src/tabs.rs` | 38 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/text_input.rs` | 13, 36, 46, 48, 84, 86 | 対応: SM-U901 |
| `crates/kivact-ui/src/toast.rs` | 76, 117 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-ui/src/toast_swipe.rs` | 56, 70, 80 | 対応: SM-U02。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `crates/kivact-ui/src/values.rs` | 109, 118, 121, 125, 153, 157, 161, 217, 224, 233, 234, 235, 236, 237, 251, 275, 279, 302, 305, 333, 365, 373 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-wire/src/events.rs` | 47, 67, 83, 87, 109, 184, 231 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-wire/src/host.rs` | 35 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-wire/src/motion.rs` | 68 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-wire/src/node.rs` | 80, 114, 194, 268, 278, 279, 319 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-wire/src/paint.rs` | 54, 55, 138, 156, 291, 307 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-wire/src/scene.rs` | 27, 50, 59, 66, 117, 120, 121, 126, 130 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-wire/src/style.rs` | 27, 68, 116, 190, 253 | 除外: 上の role/値/引数/import/doc の判断 |
| `crates/kivact-wire/src/text.rs` | 239, 268 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-basic/src/shell.rs` | 65 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-creative/src/creative/audio_values.rs` | 14, 15, 16 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-creative/src/creative/controls.rs` | 231 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-creative/src/creative/editors.rs` | 722 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-creative/src/creative/feedback.rs` | 24, 420 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-creative/src/creative/layout.rs` | 721 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-creative/src/creative/media_pool.rs` | 62 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-creative/src/creative/shell.rs` | 395, 397, 398, 787 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-game/src/game/state.rs` | 7, 8, 9, 10 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-pop/src/pop/components.rs` | 862, 910 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-pop/src/pop/data.rs` | 320 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-pop/src/pop/messaging.rs` | 22, 23 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-pop/src/pop/mission.rs` | 27 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-pop/src/pop/patterns.rs` | 195, 196, 197, 198 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-pop/src/pop/shell.rs` | 753 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact-ui-soft/src/model.rs` | 8 | 除外: 上の role/値/引数/import/doc の判断 |
| `examples/kivact/list/src/main.rs` | 10 | 除外: 上の role/値/引数/import/doc の判断 |

### doc の全 path 台帳

指定語の doc 一致行。状態の順序の要求は上の既存項目へ対応し、それ以外は描画/値/利用者の操作/公開 callback の説明として除外。

| path | line |
|---|---|
| `crates/arvane-asset/src/font.rs` | 4, 215 |
| `crates/arvane-asset/src/format.rs` | 68 |
| `crates/arvane-asset/src/lib.rs` | 38 |
| `crates/arvane-asset/src/pixels.rs` | 22 |
| `crates/arvane-asset/src/raster.rs` | 9, 12 |
| `crates/arvane-asset/src/read.rs` | 41 |
| `crates/arvane-asset/src/store.rs` | 12, 25, 26, 27 |
| `crates/arvane-asset/src/store/asset_store.rs` | 43, 88, 92, 102, 103, 428 |
| `crates/arvane-asset/src/store/entry.rs` | 18, 37, 57, 163 |
| `crates/arvane-asset/src/store/evict.rs` | 3, 7, 8, 11, 14, 18, 55, 113, 146 |
| `crates/arvane-asset/src/store/job.rs` | 13, 17 |
| `crates/arvane-asset/src/store/worker.rs` | 4, 9, 31, 133 |
| `crates/arvane-asset/src/svg.rs` | 21 |
| `crates/arvane-asset/src/svg/document.rs` | 14 |
| `crates/arvane-asset/src/svg/nesting.rs` | 3, 7, 21, 55, 63 |
| `crates/arvane-native/src/a11y/conversion.rs` | 28, 31, 123, 386 |
| `crates/arvane-native/src/a11y/native.rs` | 11, 65, 194, 202, 221 |
| `crates/arvane-native/src/contacts.rs` | 5, 18 |
| `crates/arvane-native/src/cpu.rs` | 8, 23 |
| `crates/arvane-native/src/display.rs` | 6, 8, 9 |
| `crates/arvane-native/src/display/placement.rs` | 1, 6, 66, 67 |
| `crates/arvane-native/src/embed/child.rs` | 11, 53, 55, 56, 141, 296 |
| `crates/arvane-native/src/embed/session.rs` | 3, 4, 5, 90 |
| `crates/arvane-native/src/embed/windows.rs` | 5 |
| `crates/arvane-native/src/embed/windows/create.rs` | 202 |
| `crates/arvane-native/src/embed/windows/proc.rs` | 26, 200 |
| `crates/arvane-native/src/embed/windows/state.rs` | 23, 45, 84 |
| `crates/arvane-native/src/embed/x11/conn.rs` | 50 |
| `crates/arvane-native/src/embed/x11/dnd.rs` | 39 |
| `crates/arvane-native/src/embed/x11/ime.rs` | 7, 113, 192 |
| `crates/arvane-native/src/embed/x11/keyboard.rs` | 48, 51, 106, 236 |
| `crates/arvane-native/src/embed/x11/thread.rs` | 61 |
| `crates/arvane-native/src/lib.rs` | 23, 28, 53 |
| `crates/arvane-native/src/module.rs` | 22 |
| `crates/arvane-native/src/native_input.rs` | 43, 50, 53, 56 |
| `crates/arvane-native/src/native_input/windows.rs` | 25, 29, 155, 156, 396 |
| `crates/arvane-native/src/open.rs` | 6, 15, 81 |
| `crates/arvane-native/src/runtime.rs` | 4, 15, 18, 28, 38, 78, 88, 93, 166, 208, 271, 272, 273, 394, 437, 501, 506, 533, 540 |
| `crates/arvane-native/src/test_display.rs` | 3, 5, 55, 99, 172, 225, 330 |
| `crates/arvane-render/src/backdrop.rs` | 4, 30 |
| `crates/arvane-render/src/cpu.rs` | 110 |
| `crates/arvane-render/src/derived.rs` | 55 |
| `crates/arvane-render/src/draw.rs` | 27, 35 |
| `crates/arvane-render/src/draw/effects.rs` | 18, 330 |
| `crates/arvane-render/src/draw/painter.rs` | 82, 89, 107 |
| `crates/arvane-render/src/gpu.rs` | 19, 37, 38 |
| `crates/arvane-render/src/gpu/context.rs` | 147, 245, 275, 276 |
| `crates/arvane-render/src/gpu/encode.rs` | 10, 16, 18, 20, 26, 28, 34, 50, 51, 267, 579, 581 |
| `crates/arvane-render/src/gpu/external.rs` | 19, 21, 261 |
| `crates/arvane-render/src/gpu/shared_images.rs` | 9 |
| `crates/arvane-render/src/gpu/state.rs` | 16, 70, 71, 167, 170 |
| `crates/arvane-render/src/gpu/surface.rs` | 11, 21, 23, 61, 358 |
| `crates/arvane-render/src/gradient.rs` | 21 |
| `crates/arvane-render/src/ink.rs` | 17 |
| `crates/arvane-render/src/validate.rs` | 1, 5 |
| `crates/arvane-render/src/validate/scene.rs` | 1, 215 |
| `crates/arvane-shell/src/a11y.rs` | 5 |
| `crates/arvane-shell/src/a11y/error.rs` | 8 |
| `crates/arvane-shell/src/a11y/handle.rs` | 11, 32, 59, 65 |
| `crates/arvane-shell/src/a11y/runs.rs` | 87 |
| `crates/arvane-shell/src/a11y/shared.rs` | 69, 128, 130, 132, 166, 176, 193, 200, 205, 224, 249, 262, 264, 315, 363 |
| `crates/arvane-shell/src/a11y/state.rs` | 6, 78, 91, 249 |
| `crates/arvane-shell/src/a11y/validation.rs` | 3, 11, 12 |
| `crates/arvane-shell/src/child.rs` | 8, 10, 55 |
| `crates/arvane-shell/src/clicks.rs` | 9, 84 |
| `crates/arvane-shell/src/contacts.rs` | 4, 9, 13, 63 |
| `crates/arvane-shell/src/events.rs` | 19, 27 |
| `crates/arvane-shell/src/ime.rs` | 36 |
| `crates/arvane-shell/src/lib.rs` | 22 |
| `crates/arvane-shell/src/mouse.rs` | 23, 47 |
| `crates/arvane-shell/src/services.rs` | 5 |
| `crates/arvane-shell/src/types.rs` | 61, 64, 67, 69, 87 |
| `crates/arvane-shell/src/window.rs` | 60, 67, 150, 161, 176, 181, 187, 198, 200 |
| `crates/arvane-text/src/build.rs` | 151, 152, 168, 171 |
| `crates/arvane-text/src/cache.rs` | 100 |
| `crates/arvane-text/src/inline.rs` | 16 |
| `crates/arvane-text/src/layouts.rs` | 17, 72, 100 |
| `crates/arvane-text/src/lib.rs` | 14 |
| `crates/arvane-text/src/output.rs` | 10, 14, 122, 159 |
| `crates/arvane-text/src/text_engine.rs` | 26, 39, 104, 122, 488, 489, 513, 532, 533 |
| `crates/arvane-text/src/transform.rs` | 1, 3, 74, 87 |
| `crates/arvane/src/engine.rs` | 68, 90, 106, 187 |
| `crates/arvane/src/env.rs` | 17 |
| `crates/arvane/src/presentation.rs` | 35, 40, 44 |
| `crates/arvane/src/presenter.rs` | 11, 32 |
| `crates/arvane/src/presenter/frame.rs` | 30, 58 |
| `crates/arvane/src/presenter/frame_policy.rs` | 17, 37, 125, 134, 175, 327 |
| `crates/arvane/src/presenter/gpu.rs` | 6, 9, 10, 16, 17, 61, 83, 287, 354, 371 |
| `crates/arvane/src/presenter/health.rs` | 5, 8, 12, 31 |
| `crates/arvane/src/presenter/test_frames.rs` | 29, 39 |
| `crates/arvane/src/shutdown.rs` | 1, 5 |
| `crates/arvane/src/test_display/shell.rs` | 74 |
| `crates/arvane/src/test_display/virtual_window.rs` | 206, 227, 243, 295, 301, 315, 321, 327, 333, 340, 341, 350, 361, 496 |
| `crates/arvane/src/window.rs` | 69, 70, 112, 153, 159, 162, 338, 387, 426 |
| `crates/arvane/tests/e2e/assets.rs` | 33 |
| `crates/arvane/tests/e2e/layers.rs` | 4 |
| `crates/arvane/tests/e2e/main.rs` | 29 |
| `crates/arvane/tests/e2e/present.rs` | 55 |
| `crates/arvane/tests/e2e/support.rs` | 17 |
| `crates/kivact-core/src/app.rs` | 20, 77, 81, 91, 100, 133, 139, 240 |
| `crates/kivact-core/src/client.rs` | 20, 25, 27, 28, 30, 34 |
| `crates/kivact-core/src/client/app_client.rs` | 3, 5, 53, 62, 63, 187, 268, 364 |
| `crates/kivact-core/src/client/dispatch.rs` | 14 |
| `crates/kivact-core/src/client/dispatch/common.rs` | 28, 53 |
| `crates/kivact-core/src/client/link.rs` | 10, 11, 12, 44 |
| `crates/kivact-core/src/client/phases.rs` | 13, 15, 33, 61 |
| `crates/kivact-core/src/client/settle.rs` | 1, 10, 11 |
| `crates/kivact-core/src/client/state.rs` | 23, 24, 61 |
| `crates/kivact-core/src/control.rs` | 16 |
| `crates/kivact-core/src/control/container.rs` | 15, 174 |
| `crates/kivact-core/src/control/node.rs` | 8, 45, 63, 68, 77, 97, 343 |
| `crates/kivact-core/src/control/text.rs` | 50 |
| `crates/kivact-core/src/diagnostics/errors.rs` | 6 |
| `crates/kivact-core/src/diagnostics/why.rs` | 27 |
| `crates/kivact-core/src/draw.rs` | 2 |
| `crates/kivact-core/src/draw/offscreen.rs` | 5, 13, 26, 41 |
| `crates/kivact-core/src/embed.rs` | 31, 67, 94, 95, 99, 106, 107, 130, 171, 180 |
| `crates/kivact-core/src/gpu_interop.rs` | 82 |
| `crates/kivact-core/src/gpu_interop/import.rs` | 12, 24 |
| `crates/kivact-core/src/gpu_interop/import/image.rs` | 75, 97, 100, 152, 160, 162 |
| `crates/kivact-core/src/gpu_interop/import/inbox.rs` | 13, 15, 68, 94 |
| `crates/kivact-core/src/gpu_interop/import/registry.rs` | 90, 123, 128 |
| `crates/kivact-core/src/gpu_interop/share.rs` | 13, 18 |
| `crates/kivact-core/src/gpu_interop/share/app_side.rs` | 27, 63, 138, 168 |
| `crates/kivact-core/src/gpu_interop/share/link.rs` | 10, 42, 102, 120, 165, 181, 239, 322, 381, 389, 418, 437 |
| `crates/kivact-core/src/platform/env.rs` | 25, 32, 34, 85 |
| `crates/kivact-core/src/platform/lifecycle.rs` | 21, 24, 26 |
| `crates/kivact-core/src/platform/services.rs` | 63 |
| `crates/kivact-core/src/primitive.rs` | 24, 47, 82, 85, 105 |
| `crates/kivact-core/src/primitive/measured_leaf.rs` | 47, 50, 52, 57, 96 |
| `crates/kivact-core/src/primitive/measurement.rs` | 177, 188, 203 |
| `crates/kivact-core/src/primitive/node.rs` | 82, 90, 143 |
| `crates/kivact-core/src/primitive/node_ext.rs` | 31, 37, 38, 62, 244, 361, 375, 389, 436, 441, 454, 474, 480 |
| `crates/kivact-core/src/primitive/text_editing.rs` | 70, 73, 84, 87, 107, 116, 117, 119, 139, 142, 176, 209, 210, 222, 224, 225 |
| `crates/kivact-core/src/test.rs` | 13, 41 |
| `crates/kivact-core/src/test/parent.rs` | 25 |
| `crates/kivact-core/src/test/screen/artifacts.rs` | 6, 67 |
| `crates/kivact-core/src/test/screen/asset_tests.rs` | 3, 4 |
| `crates/kivact-core/src/test/screen/controls.rs` | 78, 133 |
| `crates/kivact-core/src/test/screen/driver.rs` | 4, 24, 127, 128, 232 |
| `crates/kivact-core/src/test/screen/element.rs` | 192 |
| `crates/kivact-core/src/test/screen/gpu_tests.rs` | 354 |
| `crates/kivact-core/src/test/screen/host_sync_tests.rs` | 211, 212 |
| `crates/kivact-core/src/test/screen/input.rs` | 10, 177, 193, 201, 220, 331, 413, 414, 470, 503, 510, 654, 770 |
| `crates/kivact-core/src/test/screen/layout.rs` | 23, 26 |
| `crates/kivact-core/src/test/screen/query.rs` | 144 |
| `crates/kivact-core/src/test/screen/results.rs` | 137, 143, 148, 244, 250 |
| `crates/kivact-core/src/test/screen/structure.rs` | 3 |
| `crates/kivact-core/src/test/screen/window_tests.rs` | 4 |
| `crates/kivact-core/src/view.rs` | 74, 77 |
| `crates/kivact-core/src/view/boundary.rs` | 13 |
| `crates/kivact-core/src/view/common.rs` | 35, 38, 60, 77, 109, 183, 244 |
| `crates/kivact-core/src/view/descriptor.rs` | 3 |
| `crates/kivact-core/src/view/editing_data.rs` | 15, 16, 24, 26, 32, 43 |
| `crates/kivact-core/src/view/exit_layout.rs` | 6 |
| `crates/kivact-core/src/view/flow/branch.rs` | 40, 104, 206, 287 |
| `crates/kivact-core/src/view/flow/error_boundary.rs` | 14, 61 |
| `crates/kivact-core/src/view/flow/virtual_list.rs` | 24, 56, 117, 174, 180, 512 |
| `crates/kivact-core/src/view/focus.rs` | 13 |
| `crates/kivact-core/src/view/handler_layers.rs` | 6, 10, 86, 97, 111, 123, 183 |
| `crates/kivact-core/src/view/hover.rs` | 10, 15, 30 |
| `crates/kivact-core/src/view/input.rs` | 18, 19, 20, 49, 53, 61, 135, 147, 183, 198 |
| `crates/kivact-core/src/view/modifiers.rs` | 19, 23, 52, 61, 70, 79, 88 |
| `crates/kivact-core/src/view/mount.rs` | 54, 60, 66 |
| `crates/kivact-core/src/view/mount/attachments.rs` | 24 |
| `crates/kivact-core/src/view/mount/create.rs` | 1, 16, 36 |
| `crates/kivact-core/src/view/mount/facts.rs` | 1, 10 |
| `crates/kivact-core/src/view/mount/part_input.rs` | 142 |
| `crates/kivact-core/src/view/mount/parts.rs` | 15, 20, 133 |
| `crates/kivact-core/src/view/mount/regions.rs` | 76, 240 |
| `crates/kivact-core/src/view/mount/resolve.rs` | 4 |
| `crates/kivact-core/src/view/mount/semantics.rs` | 81 |
| `crates/kivact-core/src/view/mount/style.rs` | 80 |
| `crates/kivact-core/src/view/node.rs` | 171, 204, 205, 256, 332, 339, 346, 366, 380, 394 |
| `crates/kivact-core/src/view/node_ref.rs` | 266, 283, 285, 291, 321, 386, 461, 480, 578 |
| `crates/kivact-core/src/view/overlays.rs` | 15, 19, 27, 75, 77, 103, 301, 340, 503 |
| `crates/kivact-core/src/view/owner.rs` | 72, 80, 141 |
| `crates/kivact-core/src/view/part.rs` | 14, 15, 16, 17, 18, 19, 26, 96, 137, 246, 247, 254, 316, 319, 348, 351, 646, 670, 673, 695 |
| `crates/kivact-core/src/view/requests.rs` | 4, 21, 26, 27, 92, 93, 211, 212 |
| `crates/kivact-core/src/view/root.rs` | 9 |
| `crates/kivact-core/src/view/span.rs` | 39 |
| `crates/kivact-core/src/view/state.rs` | 77, 119, 120, 187, 190, 203, 216, 364 |
| `crates/kivact-core/src/view/sync_hook.rs` | 13, 19, 45 |
| `crates/kivact-core/src/view/tree.rs` | 29, 117, 128, 142, 155, 217, 233, 360, 372, 393, 708, 725, 788, 834, 859, 930 |
| `crates/kivact-editing/src/behavior.rs` | 38, 42, 44, 107, 135, 141, 158 |
| `crates/kivact-editing/src/change.rs` | 5, 9, 36 |
| `crates/kivact-editing/src/edit_state.rs` | 60, 68, 142, 176, 186, 308, 331, 355, 448 |
| `crates/kivact-editing/src/geometry.rs` | 10, 16, 54, 63, 90 |
| `crates/kivact-editing/src/journal.rs` | 48, 100 |
| `crates/kivact-editing/src/lib.rs` | 12, 22 |
| `crates/kivact-editing/src/line_colors.rs` | 26, 38, 65, 77 |
| `crates/kivact-editing/src/text_layout.rs` | 7 |
| `crates/kivact-engine/src/a11y.rs` | 31, 32, 41 |
| `crates/kivact-engine/src/a11y/build.rs` | 25 |
| `crates/kivact-engine/src/a11y/build/node.rs` | 366 |
| `crates/kivact-engine/src/a11y/build/relations.rs` | 7 |
| `crates/kivact-engine/src/a11y/publish.rs` | 12, 14 |
| `crates/kivact-engine/src/a11y/state.rs` | 4, 162 |
| `crates/kivact-engine/src/a11y/text.rs` | 19, 20 |
| `crates/kivact-engine/src/bridge.rs` | 6 |
| `crates/kivact-engine/src/check.rs` | 8, 33, 40, 49 |
| `crates/kivact-engine/src/check/boxes.rs` | 9, 117 |
| `crates/kivact-engine/src/check/findings.rs` | 210 |
| `crates/kivact-engine/src/check/lint.rs` | 3, 8, 17 |
| `crates/kivact-engine/src/check/lint/changed_texts.rs` | 34 |
| `crates/kivact-engine/src/frame_log.rs` | 19, 20, 24 |
| `crates/kivact-engine/src/host.rs` | 9, 17, 18, 33, 41, 44, 46, 59, 72, 74 |
| `crates/kivact-engine/src/host/client.rs` | 38, 40, 44, 46, 58, 59 |
| `crates/kivact-engine/src/host/events.rs` | 15, 17 |
| `crates/kivact-engine/src/host/launch.rs` | 13, 19, 21 |
| `crates/kivact-engine/src/host/options.rs` | 4, 8, 10 |
| `crates/kivact-engine/src/host/pump.rs` | 13, 20, 23, 58, 67, 97, 270, 408 |
| `crates/kivact-engine/src/host/testing.rs` | 44 |
| `crates/kivact-engine/src/host/testing/self_check.rs` | 8 |
| `crates/kivact-engine/src/host/testing/test_host.rs` | 58, 174, 192, 461, 517, 666 |
| `crates/kivact-engine/src/host/write.rs` | 9, 11, 27 |
| `crates/kivact-engine/src/host/write/reader.rs` | 111, 122, 131, 160 |
| `crates/kivact-engine/src/host/write/typed.rs` | 10, 11, 14, 20, 68, 126, 294 |
| `crates/kivact-engine/src/host/write/writer.rs` | 90, 111, 256, 276, 285, 302, 313 |
| `crates/kivact-engine/src/lib.rs` | 53, 54 |
| `crates/kivact-engine/src/measure/editor.rs` | 9, 12 |
| `crates/kivact-engine/src/measure/inherited_text.rs` | 40 |
| `crates/kivact-engine/src/measure/leaf.rs` | 32, 33, 34, 49, 56, 61, 62, 191, 329, 331, 354 |
| `crates/kivact-engine/src/measure/state.rs` | 5 |
| `crates/kivact-engine/src/model.rs` | 14, 35, 36, 41, 49 |
| `crates/kivact-engine/src/model/aggregate.rs` | 24, 35, 37, 39, 48, 113 |
| `crates/kivact-engine/src/model/apply.rs` | 27, 43, 57, 222, 358 |
| `crates/kivact-engine/src/model/changes.rs` | 30, 31, 181 |
| `crates/kivact-engine/src/model/dismiss.rs` | 13, 15 |
| `crates/kivact-engine/src/model/edit.rs` | 15, 20, 36, 38 |
| `crates/kivact-engine/src/model/edit/inputs.rs` | 63 |
| `crates/kivact-engine/src/model/edit/session.rs` | 16, 147 |
| `crates/kivact-engine/src/model/events.rs` | 23 |
| `crates/kivact-engine/src/model/focus.rs` | 26, 29 |
| `crates/kivact-engine/src/model/focus/navigation.rs` | 10, 222, 323, 337, 359 |
| `crates/kivact-engine/src/model/focus/state.rs` | 18 |
| `crates/kivact-engine/src/model/frame.rs` | 8, 9, 21 |
| `crates/kivact-engine/src/model/frame/state.rs` | 97 |
| `crates/kivact-engine/src/model/frame/ticks.rs` | 38, 108, 167 |
| `crates/kivact-engine/src/model/interact/caret.rs` | 1, 5, 6 |
| `crates/kivact-engine/src/model/interact/geometry.rs` | 26, 31, 52, 166, 298, 345, 346, 457, 458, 502 |
| `crates/kivact-engine/src/model/interact/sticky.rs` | 19, 20, 25 |
| `crates/kivact-engine/src/model/interact/sync.rs` | 6, 8, 9, 28, 70 |
| `crates/kivact-engine/src/model/interact/text_geometry.rs` | 102 |
| `crates/kivact-engine/src/model/interact/visual.rs` | 42 |
| `crates/kivact-engine/src/model/keys.rs` | 10, 46, 47, 51, 191 |
| `crates/kivact-engine/src/model/modal.rs` | 12, 19 |
| `crates/kivact-engine/src/model/modal/confinement.rs` | 75, 102 |
| `crates/kivact-engine/src/model/modal/state.rs` | 3, 21 |
| `crates/kivact-engine/src/model/pointer.rs` | 22, 28, 36, 37, 44, 45, 54, 55 |
| `crates/kivact-engine/src/model/pointer/hover.rs` | 90, 91, 100, 101 |
| `crates/kivact-engine/src/model/pointer/machine.rs` | 16, 273, 501 |
| `crates/kivact-engine/src/model/pointer/shell.rs` | 12, 17, 19, 20, 24, 247, 250 |
| `crates/kivact-engine/src/model/pointer/state.rs` | 55, 97, 101, 115, 116, 214, 218 |
| `crates/kivact-engine/src/model/pointer/touch.rs` | 5, 140 |
| `crates/kivact-engine/src/model/presence.rs` | 1, 4, 6, 14, 15, 24, 25 |
| `crates/kivact-engine/src/model/presence/state.rs` | 7 |
| `crates/kivact-engine/src/model/presence/transitions.rs` | 1, 9 |
| `crates/kivact-engine/src/model/state_motion.rs` | 14 |
| `crates/kivact-engine/src/model/state_motion/state.rs` | 38 |
| `crates/kivact-engine/src/model/state_motion/steps.rs` | 9, 65 |
| `crates/kivact-engine/src/model/states.rs` | 27, 31, 85, 90 |
| `crates/kivact-engine/src/model/transform.rs` | 24, 35, 42, 43, 247 |
| `crates/kivact-engine/src/mutation.rs` | 112 |
| `crates/kivact-engine/src/native.rs` | 5 |
| `crates/kivact-engine/src/offscreen.rs` | 12, 21, 22, 59, 118 |
| `crates/kivact-engine/src/paint.rs` | 38 |
| `crates/kivact-engine/src/paint/frame_scene.rs` | 91, 250, 304 |
| `crates/kivact-engine/src/paint/list.rs` | 9, 11, 15, 31, 44, 46, 55, 61, 65, 84 |
| `crates/kivact-engine/src/paint/list/index.rs` | 16, 18 |
| `crates/kivact-engine/src/paint/list/invalidation.rs` | 16, 40 |
| `crates/kivact-engine/src/paint/list/walk.rs` | 12 |
| `crates/kivact-engine/src/paint/state.rs` | 20, 34, 54, 153, 170, 221 |
| `crates/kivact-engine/src/paint/text.rs` | 40 |
| `crates/kivact-engine/src/text_measure.rs` | 52 |
| `crates/kivact-engine/src/tree.rs` | 4, 6, 9, 10 |
| `crates/kivact-engine/src/tree/node.rs` | 46, 70, 287 |
| `crates/kivact-engine/src/tree/node_tree.rs` | 87, 140, 253, 303, 313, 314, 315 |
| `crates/kivact-engine/src/tree/store.rs` | 8, 32, 102, 126 |
| `crates/kivact-gpu-share/src/d3d12.rs` | 23 |
| `crates/kivact-gpu-share/src/d3d12/pool.rs` | 29, 68, 104, 105, 133, 136 |
| `crates/kivact-gpu-share/src/d3d12/read_back.rs` | 30, 38 |
| `crates/kivact-gpu-share/src/import.rs` | 16, 21, 22, 27, 41 |
| `crates/kivact-gpu-share/src/import/dx12.rs` | 5, 14, 16 |
| `crates/kivact-gpu-share/src/import/pool.rs` | 137, 150, 160, 254 |
| `crates/kivact-gpu-share/src/import/vulkan.rs` | 10, 21, 26, 27, 84, 386 |
| `crates/kivact-gpu-share/src/lib.rs` | 15, 72, 73, 81, 139, 149, 156 |
| `crates/kivact-gpu-share/src/pool.rs` | 99, 106 |
| `crates/kivact-gpu-share/src/vulkan.rs` | 15 |
| `crates/kivact-gpu-share/src/vulkan/pool.rs` | 25, 66, 67, 107, 108, 116, 117, 144, 147, 560 |
| `crates/kivact-gpu-share/src/vulkan/read_back.rs` | 63, 71 |
| `crates/kivact-gpu-share/src/wgpu.rs` | 27, 28 |
| `crates/kivact-gpu-share/src/wgpu/d3d12.rs` | 11, 145 |
| `crates/kivact-gpu-share/src/wgpu/pool.rs` | 102, 106, 146, 147, 150, 152, 207, 258 |
| `crates/kivact-gpu-share/src/wgpu/read_back.rs` | 13 |
| `crates/kivact-i18n/src/date.rs` | 38, 49, 88, 148, 153, 181, 232, 246, 247, 277 |
| `crates/kivact-i18n/src/locale.rs` | 37, 42, 43, 102 |
| `crates/kivact-interaction/src/calendar.rs` | 3, 10, 14, 15, 18, 26, 28 |
| `crates/kivact-interaction/src/calendar/handle.rs` | 20, 60, 86, 151 |
| `crates/kivact-interaction/src/calendar/policy.rs` | 21, 65, 72 |
| `crates/kivact-interaction/src/calendar/state.rs` | 12, 17, 164 |
| `crates/kivact-interaction/src/combobox.rs` | 53, 54, 55, 60, 275, 345 |
| `crates/kivact-interaction/src/delayed.rs` | 3, 4, 6, 7, 35, 48, 49, 76, 135, 148, 149, 210 |
| `crates/kivact-interaction/src/dialog.rs` | 18, 19, 129, 132, 177, 256 |
| `crates/kivact-interaction/src/disclosure.rs` | 3, 23, 93, 149 |
| `crates/kivact-interaction/src/drag.rs` | 7, 20, 21, 180, 230, 236, 240, 250, 251, 337, 338, 341, 342, 379, 436, 438, 612, 617, 621, 623, 672 |
| `crates/kivact-interaction/src/drag_settle.rs` | 36 |
| `crates/kivact-interaction/src/edit.rs` | 7, 11, 35, 37, 38, 91, 93, 226 |
| `crates/kivact-interaction/src/focus_return.rs` | 96 |
| `crates/kivact-interaction/src/follow_end.rs` | 22, 42 |
| `crates/kivact-interaction/src/hover_corridor.rs` | 18 |
| `crates/kivact-interaction/src/list.rs` | 23, 24, 25, 41, 58 |
| `crates/kivact-interaction/src/list/data.rs` | 109, 179 |
| `crates/kivact-interaction/src/list/handle.rs` | 18, 57 |
| `crates/kivact-interaction/src/list/order.rs` | 16 |
| `crates/kivact-interaction/src/list/selection.rs` | 19 |
| `crates/kivact-interaction/src/list/typeahead.rs` | 18, 23, 32, 59, 63 |
| `crates/kivact-interaction/src/menu.rs` | 22, 27, 28, 36 |
| `crates/kivact-interaction/src/menu/handle.rs` | 36, 42, 45, 48, 159, 202, 204, 251, 255, 257 |
| `crates/kivact-interaction/src/menu/order.rs` | 115 |
| `crates/kivact-interaction/src/menu/session.rs` | 112, 113, 198 |
| `crates/kivact-interaction/src/menu/state.rs` | 74, 126, 150, 210, 224, 274, 629 |
| `crates/kivact-interaction/src/menubar.rs` | 17, 153 |
| `crates/kivact-interaction/src/palette.rs` | 42, 219, 221, 420 |
| `crates/kivact-interaction/src/policy/grid.rs` | 22, 41, 45, 56, 76 |
| `crates/kivact-interaction/src/policy/scalar.rs` | 8, 10, 14, 35, 79 |
| `crates/kivact-interaction/src/popup.rs` | 8, 13, 67, 69, 81, 151, 160, 244, 294, 322 |
| `crates/kivact-interaction/src/range_scalar.rs` | 154 |
| `crates/kivact-interaction/src/scalar.rs` | 18, 37, 38, 42 |
| `crates/kivact-interaction/src/scalar/handle.rs` | 35, 46, 138, 154, 156, 227, 235, 236, 271 |
| `crates/kivact-interaction/src/scalar/touch.rs` | 16 |
| `crates/kivact-interaction/src/select.rs` | 12, 44, 47, 54, 59, 73, 172, 173, 181, 198, 204, 274, 345 |
| `crates/kivact-interaction/src/splitter.rs` | 271, 274, 275 |
| `crates/kivact-interaction/src/table.rs` | 27 |
| `crates/kivact-interaction/src/table/data.rs` | 26, 28, 65 |
| `crates/kivact-interaction/src/table/handle.rs` | 31, 45, 46, 100, 163 |
| `crates/kivact-interaction/src/table/model.rs` | 499, 519, 526, 527 |
| `crates/kivact-interaction/src/tabs.rs` | 9, 11 |
| `crates/kivact-interaction/src/text_edit/handle.rs` | 18, 102, 106, 109, 163 |
| `crates/kivact-interaction/src/toaster.rs` | 79, 95, 120, 162, 190, 197, 198, 234, 242, 286, 288, 296, 331, 388, 479, 490, 505, 580, 581 |
| `crates/kivact-interaction/src/toggle.rs` | 62, 104, 116, 203, 220 |
| `crates/kivact-interaction/src/tone.rs` | 15 |
| `crates/kivact-interaction/src/tooltip.rs` | 10, 13, 51, 54, 70, 74, 84, 159, 167, 174, 177, 211, 279, 287 |
| `crates/kivact-interaction/src/tree.rs` | 155, 209, 212, 355, 374, 549, 628, 637 |
| `crates/kivact-interaction/src/zoom_pan.rs` | 316, 318, 324 |
| `crates/kivact-layout/src/borrowed_tree.rs` | 18, 22, 23, 24, 40, 42, 57 |
| `crates/kivact-layout/src/columns.rs` | 28, 121, 129, 131, 157, 170, 171, 172, 192 |
| `crates/kivact-layout/src/compare.rs` | 35, 44, 46, 47, 58, 86, 88, 111 |
| `crates/kivact-layout/src/image.rs` | 16, 25 |
| `crates/kivact-layout/src/lib.rs` | 9, 16, 21, 25, 26 |
| `crates/kivact-layout/src/pass_state.rs` | 11, 34, 39, 53, 61 |
| `crates/kivact-layout/src/placement.rs` | 74, 75 |
| `crates/kivact-layout/src/tree.rs` | 18, 19, 23, 25, 27, 29, 329, 330, 364, 374, 375, 412, 558, 559, 560 |
| `crates/kivact-lower/src/assets.rs` | 21, 124, 216, 248 |
| `crates/kivact-lower/src/border.rs` | 5, 13, 194 |
| `crates/kivact-lower/src/context.rs` | 4, 6, 10, 21, 24, 25, 33, 35, 45, 69, 133 |
| `crates/kivact-lower/src/conversion.rs` | 9, 14, 19 |
| `crates/kivact-lower/src/effects.rs` | 12, 14, 15, 17, 32, 71, 146, 150 |
| `crates/kivact-lower/src/fill.rs` | 11, 17, 18, 20, 35, 87, 154 |
| `crates/kivact-lower/src/glyphs.rs` | 94, 95, 186 |
| `crates/kivact-lower/src/lib.rs` | 11 |
| `crates/kivact-lower/src/sanitize.rs` | 4, 197 |
| `crates/kivact-lower/src/scene.rs` | 9 |
| `crates/kivact-lower/src/scene_cache.rs` | 9 |
| `crates/kivact-lower/src/shape.rs` | 30 |
| `crates/kivact-motion/src/animated.rs` | 59 |
| `crates/kivact-motion/src/frame_loop.rs` | 23, 87 |
| `crates/kivact-motion/src/lib.rs` | 14, 21, 22, 27, 42, 47 |
| `crates/kivact-motion/src/state.rs` | 256, 285, 289, 313 |
| `crates/kivact-motion/src/transition.rs` | 61, 68 |
| `crates/kivact-motion/src/values.rs` | 115 |
| `crates/kivact-reactive/src/data.rs` | 15, 16, 96, 366, 383, 385, 415, 427, 490 |
| `crates/kivact-reactive/src/diag.rs` | 68, 91, 216, 221 |
| `crates/kivact-reactive/src/driver.rs` | 11, 33 |
| `crates/kivact-reactive/src/error.rs` | 29 |
| `crates/kivact-reactive/src/ext.rs` | 7 |
| `crates/kivact-reactive/src/graph.rs` | 16, 19, 26, 145, 598, 614 |
| `crates/kivact-reactive/src/handles.rs` | 181, 183, 236, 744 |
| `crates/kivact-reactive/src/host.rs` | 37, 91, 165 |
| `crates/kivact-reactive/src/mounts.rs` | 3, 10 |
| `crates/kivact-reactive/src/object.rs` | 11, 27, 117 |
| `crates/kivact-reactive/src/owner.rs` | 25, 32, 45, 192, 194, 325 |
| `crates/kivact-reactive/src/owner_handle.rs` | 113, 114, 159, 171, 183, 216, 217, 294 |
| `crates/kivact-reactive/src/props.rs` | 34, 142 |
| `crates/kivact-reactive/src/report.rs` | 59 |
| `crates/kivact-reactive/src/scope.rs` | 29 |
| `crates/kivact-reactive/src/setup.rs` | 11, 20, 112, 145, 159, 197, 211, 222 |
| `crates/kivact-reactive/src/trigger.rs` | 4, 63 |
| `crates/kivact-reactive/src/turn.rs` | 4, 6, 32, 63, 65, 107, 130 |
| `crates/kivact-style/src/draw.rs` | 2, 31, 32 |
| `crates/kivact-style/src/draw/image.rs` | 10, 15, 81, 123, 176 |
| `crates/kivact-style/src/draw/layer.rs` | 13, 53, 108 |
| `crates/kivact-style/src/draw/paragraph.rs` | 11, 13, 120, 154, 269 |
| `crates/kivact-style/src/draw/recorder.rs` | 58, 76, 96, 99, 147 |
| `crates/kivact-style/src/draw/regions.rs` | 11 |
| `crates/kivact-style/src/draw/text_run.rs` | 51, 54, 186 |
| `crates/kivact-style/src/effect.rs` | 15 |
| `crates/kivact-style/src/grid.rs` | 112 |
| `crates/kivact-style/src/paint.rs` | 11, 16 |
| `crates/kivact-style/src/paint/background.rs` | 9 |
| `crates/kivact-style/src/paint/canvas_gradient.rs` | 61 |
| `crates/kivact-style/src/paint/gradient.rs` | 43, 121, 122, 130, 139 |
| `crates/kivact-style/src/paint/layer.rs` | 65, 94, 155 |
| `crates/kivact-style/src/path.rs` | 12 |
| `crates/kivact-style/src/path/geometry.rs` | 18, 36, 230, 253 |
| `crates/kivact-style/src/path/trim.rs` | 113, 117, 120 |
| `crates/kivact-style/src/resolve.rs` | 23, 377, 814, 820, 822 |
| `crates/kivact-style/src/sheet.rs` | 19, 69, 71, 130, 153, 393, 431, 440, 441, 445, 446, 453, 511, 551, 764, 885, 887, 892, 894, 927, 1136, 1141, 1184, 1204, 1230 |
| `crates/kivact-style/src/text.rs` | 31, 32, 44, 63, 269, 366, 368 |
| `crates/kivact-style/src/theme/value.rs` | 154, 307, 605 |
| `crates/kivact-style/src/transform.rs` | 20, 57, 165 |
| `crates/kivact-task/src/action.rs` | 20, 54, 55, 198, 241, 475, 578, 603, 623, 685, 686, 720 |
| `crates/kivact-task/src/blocking.rs` | 179, 180, 181 |
| `crates/kivact-task/src/completion.rs` | 11 |
| `crates/kivact-task/src/drain.rs` | 3, 4, 15 |
| `crates/kivact-task/src/executor.rs` | 8 |
| `crates/kivact-task/src/lib.rs` | 5 |
| `crates/kivact-task/src/resource.rs` | 120, 121 |
| `crates/kivact-task/src/state.rs` | 46, 49, 56, 65, 72, 117, 188, 221 |
| `crates/kivact-task/src/time.rs` | 77, 115, 116 |
| `crates/kivact-task/src/worker_pool.rs` | 103 |
| `crates/kivact-text/src/chunks.rs` | 14, 211, 299 |
| `crates/kivact-text/src/edit_lines.rs` | 18, 31 |
| `crates/kivact-text/src/excerpt.rs` | 18, 46, 59 |
| `crates/kivact-text/src/lib.rs` | 3, 4, 12, 16, 23, 43, 44, 46, 57, 58, 59, 69, 75, 76, 84, 85, 86, 88, 92, 97 |
| `crates/kivact-text/src/pieces.rs` | 5, 11, 27, 150 |
| `crates/kivact-text/src/records.rs` | 91, 113, 115, 120, 201 |
| `crates/kivact-text/src/refresh.rs` | 14, 39 |
| `crates/kivact-text/src/shaping.rs` | 13, 25, 101, 160, 185, 296, 457, 488, 492, 516, 517 |
| `crates/kivact-text/src/source.rs` | 17 |
| `crates/kivact-types/src/gpu_interop.rs` | 82, 83, 84, 129, 237, 248, 255, 256, 257, 260, 269, 270, 275 |
| `crates/kivact-types/src/layout.rs` | 49 |
| `crates/kivact-types/src/lib.rs` | 23 |
| `crates/kivact-types/src/motion.rs` | 16, 53 |
| `crates/kivact-types/src/paint.rs` | 1, 3 |
| `crates/kivact-types/src/protocol/effect.rs` | 11, 89, 115 |
| `crates/kivact-types/src/protocol/image.rs` | 15, 17 |
| `crates/kivact-types/src/protocol/paint.rs` | 56, 120 |
| `crates/kivact-types/src/protocol/scene.rs` | 22, 144, 157, 178, 213, 224 |
| `crates/kivact-types/src/protocol/semantics.rs` | 2, 33, 41, 79 |
| `crates/kivact-types/src/protocol/shape.rs` | 28, 30, 49, 51, 85 |
| `crates/kivact-types/src/protocol/shaped.rs` | 24, 49, 77, 90, 92, 93, 107, 143, 144 |
| `crates/kivact-types/src/protocol/text.rs` | 69, 75, 77, 80, 91, 148, 150, 168 |
| `crates/kivact-types/src/protocol/transform.rs` | 10 |
| `crates/kivact-types/src/protocol/window.rs` | 22, 38 |
| `crates/kivact-types/src/semantics_text.rs` | 11, 22, 27, 28, 30, 77 |
| `crates/kivact-types/src/shape.rs` | 59 |
| `crates/kivact-types/src/text.rs` | 44, 46, 48, 95, 145, 151 |
| `crates/kivact-ui-basic/src/alert_dialog.rs` | 9 |
| `crates/kivact-ui-basic/src/attachment.rs` | 132 |
| `crates/kivact-ui-basic/src/avatar.rs` | 54, 191 |
| `crates/kivact-ui-basic/src/badge.rs` | 75, 81 |
| `crates/kivact-ui-basic/src/button_group.rs` | 28, 29, 146, 147 |
| `crates/kivact-ui-basic/src/calendar/state.rs` | 21 |
| `crates/kivact-ui-basic/src/calendar/view.rs` | 87, 95, 120, 128 |
| `crates/kivact-ui-basic/src/carousel.rs` | 24, 93 |
| `crates/kivact-ui-basic/src/chart/radar.rs` | 47 |
| `crates/kivact-ui-basic/src/chart/view.rs` | 52 |
| `crates/kivact-ui-basic/src/children.rs` | 12, 19 |
| `crates/kivact-ui-basic/src/choice.rs` | 128 |
| `crates/kivact-ui-basic/src/code_block/rust.rs` | 6, 7 |
| `crates/kivact-ui-basic/src/code_block/view.rs` | 131, 296 |
| `crates/kivact-ui-basic/src/date_picker.rs` | 114, 201 |
| `crates/kivact-ui-basic/src/empty.rs` | 42 |
| `crates/kivact-ui-basic/src/hover_card.rs` | 25, 27, 76, 83 |
| `crates/kivact-ui-basic/src/input_group.rs` | 109, 115 |
| `crates/kivact-ui-basic/src/input_otp.rs` | 212 |
| `crates/kivact-ui-basic/src/item.rs` | 112 |
| `crates/kivact-ui-basic/src/line_tabs.rs` | 68, 115 |
| `crates/kivact-ui-basic/src/marker.rs` | 58 |
| `crates/kivact-ui-basic/src/menubar.rs` | 20 |
| `crates/kivact-ui-basic/src/navigation_menu.rs` | 120, 127, 382 |
| `crates/kivact-ui-basic/src/pagination.rs` | 221, 261, 290 |
| `crates/kivact-ui-basic/src/primitives.rs` | 59 |
| `crates/kivact-ui-basic/src/questionnaire.rs` | 111, 128 |
| `crates/kivact-ui-basic/src/recipes.rs` | 139 |
| `crates/kivact-ui-basic/src/recipes_controls.rs` | 298, 318 |
| `crates/kivact-ui-basic/src/recipes_menu.rs` | 162 |
| `crates/kivact-ui-basic/src/sidebar/menu.rs` | 209 |
| `crates/kivact-ui-basic/src/sidebar/state.rs` | 4, 84 |
| `crates/kivact-ui-basic/src/sidebar/trigger.rs` | 21, 36 |
| `crates/kivact-ui-basic/src/slot.rs` | 1 |
| `crates/kivact-ui-basic/src/theme.rs` | 13 |
| `crates/kivact-ui-basic/src/trace.rs` | 55, 61 |
| `crates/kivact-ui-creative/src/bezier/data.rs` | 7, 9 |
| `crates/kivact-ui-creative/src/bezier/view.rs` | 23, 56 |
| `crates/kivact-ui-creative/src/color_wheel/view.rs` | 32 |
| `crates/kivact-ui-creative/src/dock/chrome.rs` | 28 |
| `crates/kivact-ui-creative/src/dock/geometry.rs` | 128 |
| `crates/kivact-ui-creative/src/dock/items.rs` | 31 |
| `crates/kivact-ui-creative/src/dock/model.rs` | 41, 70, 89, 94, 99, 142, 335, 341, 394, 518 |
| `crates/kivact-ui-creative/src/dock/view.rs` | 93, 110 |
| `crates/kivact-ui-creative/src/editors/arrangement/input.rs` | 16, 116 |
| `crates/kivact-ui-creative/src/editors/arrangement/layout.rs` | 58 |
| `crates/kivact-ui-creative/src/editors/arrangement/view_state.rs` | 12 |
| `crates/kivact-ui-creative/src/editors/graph/input.rs` | 13, 81 |
| `crates/kivact-ui-creative/src/editors/graph/view.rs` | 20 |
| `crates/kivact-ui-creative/src/editors/node/input.rs` | 67, 212 |
| `crates/kivact-ui-creative/src/editors/shared/lane.rs` | 69, 71 |
| `crates/kivact-ui-creative/src/editors/shared/model.rs` | 88 |
| `crates/kivact-ui-creative/src/editors/shared/ruler.rs` | 19 |
| `crates/kivact-ui-creative/src/editors/time/input.rs` | 13, 68 |
| `crates/kivact-ui-creative/src/feedback.rs` | 85, 132, 344, 411 |
| `crates/kivact-ui-creative/src/gradient/view.rs` | 30 |
| `crates/kivact-ui-creative/src/numeric.rs` | 182 |
| `crates/kivact-ui-creative/src/outliner/controls.rs` | 59 |
| `crates/kivact-ui-creative/src/outliner/drop_plan.rs` | 1 |
| `crates/kivact-ui-creative/src/outliner/view.rs` | 46, 154 |
| `crates/kivact-ui-creative/src/outliner_data.rs` | 6, 14, 59, 60, 69, 70 |
| `crates/kivact-ui-creative/src/panel_group.rs` | 22, 29 |
| `crates/kivact-ui-creative/src/picker.rs` | 32, 61, 65, 93, 105, 126, 138, 142, 145, 148, 149, 210, 212, 247, 248, 268, 269, 288 |
| `crates/kivact-ui-creative/src/property_row.rs` | 35, 70 |
| `crates/kivact-ui-creative/src/range.rs` | 50, 63 |
| `crates/kivact-ui-creative/src/recipes_controls.rs` | 221 |
| `crates/kivact-ui-creative/src/slot.rs` | 1 |
| `crates/kivact-ui-creative/src/tabs.rs` | 14 |
| `crates/kivact-ui-creative/src/viewport_surface.rs` | 29, 166, 167 |
| `crates/kivact-ui-creative/tests/consumer/gpu_interop.rs` | 98, 124, 173 |
| `crates/kivact-ui-creative/tests/consumer/table_rows.rs` | 21 |
| `crates/kivact-ui-game/src/choice.rs` | 51, 90, 91 |
| `crates/kivact-ui-game/src/input.rs` | 135 |
| `crates/kivact-ui-game/src/layout.rs` | 70 |
| `crates/kivact-ui-game/src/navigation.rs` | 10 |
| `crates/kivact-ui-game/src/recipes.rs` | 825 |
| `crates/kivact-ui-game/src/recipes_small.rs` | 93 |
| `crates/kivact-ui-game/src/slot.rs` | 1 |
| `crates/kivact-ui-pop/src/date/calendar.rs` | 34, 35, 45, 114, 142 |
| `crates/kivact-ui-pop/src/date/field.rs` | 94 |
| `crates/kivact-ui-pop/src/date/input.rs` | 18, 21 |
| `crates/kivact-ui-pop/src/date/range_input.rs` | 17, 18, 28, 29, 30, 31, 32, 33, 86 |
| `crates/kivact-ui-pop/src/date/text.rs` | 73 |
| `crates/kivact-ui-pop/src/form.rs` | 33 |
| `crates/kivact-ui-pop/src/recipes_extra.rs` | 191 |
| `crates/kivact-ui-pop/src/resize.rs` | 24 |
| `crates/kivact-ui-pop/src/slot.rs` | 1 |
| `crates/kivact-ui-soft/src/calendar.rs` | 66 |
| `crates/kivact-ui-soft/src/code_block.rs` | 216, 221 |
| `crates/kivact-ui-soft/src/color.rs` | 425 |
| `crates/kivact-ui-soft/src/file_input.rs` | 1 |
| `crates/kivact-ui-soft/src/markdown.rs` | 109 |
| `crates/kivact-ui-soft/src/soft_overflow.rs` | 99 |
| `crates/kivact-ui-soft/src/text.rs` | 201 |
| `crates/kivact-ui-soft/src/typeahead.rs` | 80 |
| `crates/kivact-ui-soft/tests/empty_state_announcement.rs` | 1 |
| `crates/kivact-ui/src/button.rs` | 41, 44, 204, 212, 240 |
| `crates/kivact-ui/src/command_palette.rs` | 93 |
| `crates/kivact-ui/src/data_column.rs` | 22, 233 |
| `crates/kivact-ui/src/data_table/builder.rs` | 33, 36, 148, 202, 342, 377, 455 |
| `crates/kivact-ui/src/data_table/rows.rs` | 415 |
| `crates/kivact-ui/src/dialog.rs` | 12, 116, 195 |
| `crates/kivact-ui/src/disclosure.rs` | 113 |
| `crates/kivact-ui/src/field.rs` | 100 |
| `crates/kivact-ui/src/helpers.rs` | 7 |
| `crates/kivact-ui/src/layer.rs` | 1 |
| `crates/kivact-ui/src/lib.rs` | 3, 8, 29, 33, 38, 72 |
| `crates/kivact-ui/src/list.rs` | 77 |
| `crates/kivact-ui/src/look_palette.rs` | 40 |
| `crates/kivact-ui/src/menu.rs` | 8, 13, 58, 84, 270 |
| `crates/kivact-ui/src/number_input.rs` | 20, 103 |
| `crates/kivact-ui/src/progress.rs` | 22, 123 |
| `crates/kivact-ui/src/radio_group.rs` | 112 |
| `crates/kivact-ui/src/recipes.rs` | 225, 299, 454, 473 |
| `crates/kivact-ui/src/select.rs` | 83 |
| `crates/kivact-ui/src/slider.rs` | 71, 130 |
| `crates/kivact-ui/src/split.rs` | 5, 25, 53, 54, 61, 130 |
| `crates/kivact-ui/src/splitter.rs` | 36, 79 |
| `crates/kivact-ui/src/tabs.rs` | 46, 75 |
| `crates/kivact-ui/src/tag_input.rs` | 39, 41, 100 |
| `crates/kivact-ui/src/text_draft.rs` | 13, 71 |
| `crates/kivact-ui/src/text_input.rs` | 44, 160, 177, 180, 181, 237 |
| `crates/kivact-ui/src/toast.rs` | 62, 86 |
| `crates/kivact-ui/src/toggle.rs` | 65 |
| `crates/kivact-ui/src/toggle_button.rs` | 144, 167 |
| `crates/kivact-ui/src/tooltip.rs` | 49, 156 |
| `crates/kivact-ui/src/values.rs` | 8, 50, 232, 288, 338 |
| `crates/kivact-ui/src/variants.rs` | 9, 112 |
| `crates/kivact-wire/src/check.rs` | 11, 80, 82 |
| `crates/kivact-wire/src/events.rs` | 23, 36, 68, 161, 162, 167, 198, 257, 272, 283, 292, 309, 318, 319 |
| `crates/kivact-wire/src/ids.rs` | 14, 15 |
| `crates/kivact-wire/src/measure.rs` | 5, 9, 11, 143, 155 |
| `crates/kivact-wire/src/motion.rs` | 18, 61 |
| `crates/kivact-wire/src/node.rs` | 42, 384 |
| `crates/kivact-wire/src/paint.rs` | 41, 51 |
| `crates/kivact-wire/src/paint_vec.rs` | 6 |
| `crates/kivact-wire/src/scene.rs` | 4, 31, 33 |
| `crates/kivact-wire/src/semantics.rs` | 14 |
| `crates/kivact-wire/src/style.rs` | 179, 254, 259, 260 |
| `crates/kivact-wire/src/text.rs` | 109, 258, 270 |
| `examples/kivact-ui-basic/src/model.rs` | 25 |
| `examples/kivact-ui-basic/tests/catalog.rs` | 55 |
| `examples/kivact-ui-game/src/game/slot.rs` | 1 |

### effect/memo・sleep・debug_assert の台帳

call の括弧の範囲を追い、内部の `.set/.update/.replace` も補助検索した。nested handler の書込みと memo 本体の書込みは区別する。`Draft::show` のような helper 経由も個別の調査に含む。

| 場所 | 検索 | 判断と理由 |
|---|---|---|
| `crates/arvane/src/test_display/shell.rs`: 119 | `sleep` | 対応: SM-A15 |
| `crates/arvane-asset/src/pixels.rs`: 50 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/arvane-asset/src/store/worker.rs`: 157 | `sleep` | 除外: worker 終了を待つ bounded polling。UI の phase/timer の実装でない |
| `crates/arvane-native/src/wait.rs`: 33 | `sleep` | 除外: worker 終了を待つ bounded polling。UI の phase/timer の実装でない |
| `crates/arvane-native/src/embed/session.rs`: 41 | `debug_assert` | 対応: SM-A01 |
| `crates/arvane-native/src/embed/windows/test_parent.rs`: 264 | `sleep` | 除外: harness の待機/不変条件の検証 |
| `crates/arvane-native/src/embed/windows/window.rs`: 75, 76 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/arvane-native/src/embed/x11/test_parent.rs`: 309, 411, 708 | `sleep` | 除外: harness の待機/不変条件の検証 |
| `crates/arvane-native/src/embed/x11/thread.rs`: 267 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/arvane-native/src/embed/x11/window.rs`: 78, 98 | `debug_assert` | 対応: SM-A01、SM-A05 |
| `crates/kivact-core/src/client/app_client.rs`: 257 | `debug_assert` | 対応: SM-R04、SM-R19、SM-C01 |
| `crates/kivact-core/src/client/link.rs`: 47 | `debug_assert` | 対応: SM-R04、SM-C05 |
| `crates/kivact-core/src/control/text_binding.rs:28` | `effect`: `tree.update`, `tree.update`, `reported.replace` | 対応: SM-C18。同じ遷移の再登録をしない |
| `crates/kivact-core/src/test/screen/artifacts.rs`: 256 | `sleep` | 除外: harness の待機/不変条件の検証 |
| `crates/kivact-core/src/test/screen/real_window_tests.rs`: 123, 203 | `sleep` | 除外: harness の待機/不変条件の検証 |
| `crates/kivact-core/src/view/node.rs`: 108 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/kivact-core/src/view/part.rs:173` | `effect`: `input.disabled.set`, `input.hidden.set`, `previously_focused.replace` | 対応: SM-C15。同じ遷移の再登録をしない |
| `crates/kivact-core/src/view/tree.rs`: 623 | `debug_assert` | 対応: SM-C07、SM-C11、SM-C12、SM-C13 |
| `crates/kivact-core/src/view/flow/branch.rs:236` | `effect`: `sig.update` | 対応: SM-C19。同じ遷移の再登録をしない |
| `crates/kivact-core/src/view/flow/error_boundary.rs:150` | `effect`: `shown.set` | 除外: 内部の render/mount effect が tree の出力を作る。別の derived signal の同期でない |
| `crates/kivact-core/src/view/flow/list.rs:107` | `effect`: `sig.update` | 除外: 内部の render/mount effect が tree の出力を作る。別の derived signal の同期でない |
| `crates/kivact-core/src/view/flow/list.rs:266` | `effect`: `sig.update` | 除外: 内部の render/mount effect が tree の出力を作る。別の derived signal の同期でない |
| `crates/kivact-core/src/view/flow/virtual_list.rs:309` | `effect`: `warned.replace` | 対応: SM-C18、SM-C20。同じ遷移の再登録をしない |
| `crates/kivact-core/src/view/mount/draw.rs:50` | `effect`: `reported.replace` | 対応: SM-C18。同じ遷移の再登録をしない |
| `crates/kivact-core/src/view/mount/region_collection.rs:37` | `effect`: `signal.update`, `registry.update` | 除外: 内部の render/mount effect が tree の出力を作る。別の derived signal の同期でない |
| `crates/kivact-core/src/view/mount/style.rs:116` | `effect`: `reported_leaf_paint.replace`, `tree.update`, `tree.update`, `tree.update`, `tree.update`, `reported_states.replace`, `reported.replace`, `reported_text.replace` | 対応: SM-C18。同じ遷移の再登録をしない |
| `crates/kivact-engine/src/text_measure.rs`: 40, 61, 90 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/kivact-engine/src/check/shaped_lines.rs`: 50 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/kivact-engine/src/host/write/writer.rs`: 42, 95, 96, 101, 102 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/kivact-engine/src/model/apply.rs`: 146, 438 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/kivact-engine/src/model/lifecycle.rs`: 9 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/kivact-engine/src/model/frame/ticks.rs`: 82 | `debug_assert` | 対応: SM-E04 |
| `crates/kivact-engine/src/tree/child_order.rs`: 36, 93, 94 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/kivact-engine/src/tree/node_tree.rs`: 189 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/kivact-engine/src/tree/store.rs`: 113, 114, 115 | `debug_assert` | 除外: 値・寸法・unique id・slot/topology の整合の検証。状態の同期でない |
| `crates/kivact-interaction/src/combobox.rs:156` | `effect`: `active.set` | 対応: SM-I06。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/combobox.rs:163` | `effect`: `active.set` | 対応: SM-I06。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/combobox.rs:224` | `effect`: `open.set` | 対応: SM-I06。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/delayed.rs`: 217 | `sleep` | 対応: SM-I03 |
| `crates/kivact-interaction/src/follow_end.rs:69` | `effect`: `observer.set` | 対応: SM-I23。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/hover_corridor.rs:82` | `effect`: `path.set`, `path.set`, `path.set`, `path.set`, `holding.set`, `progress.update`, `timer.set`, `path.set`, `timer_slot.set`, `holding.set` | 対応: SM-I21。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/hover_corridor.rs`: 116 | `sleep` | 対応: SM-I21 |
| `crates/kivact-interaction/src/popup.rs:403` | `effect`: `anchor_seen.set`, `entered.set` | 対応: SM-I02。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/range_scalar.rs:104` | `effect_on`: `low.set`, `high.set` | 対応: SM-I902。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/select.rs:147` | `effect`: `hovered.set` | 対応: SM-I07。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/select.rs:590` | `effect`: `reported.set` | 対応: SM-I07。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/toaster.rs:417` | `effect`: `self.holds.set`, `self.holds.set` | 対応: SM-I22。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/toaster.rs:447` | `effect`: `previous.set` | 対応: SM-I22。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/toaster.rs:565` | `effect`: `engaged.set` | 対応: SM-I22。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/toaster.rs:587` | `effect`: `left.set` | 対応: SM-I22。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/toaster.rs`: 594 | `sleep` | 対応: SM-I22 |
| `crates/kivact-interaction/src/calendar/state.rs:70` | `effect`: `pick.set` | 対応: SM-I26、SM-I903。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/list/state.rs:96` | `memo`: `position.set` | 対応: SM-I14、SM-I26。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/menu/level.rs:224` | `effect`: `session.want.set` | 対応: SM-I10。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/menu/level.rs:376` | `effect`: `session.focus_key.set` | 対応: SM-I10。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/menu/level.rs:494` | `effect`: `rects.update` | 対応: SM-I10。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/menu/press.rs`: 51 | `sleep` | 対応: SM-I09 |
| `crates/kivact-interaction/src/menu/session.rs:56` | `effect`: `path.set`, `want.set`, `focus_key.set`, `opened.set` | 対応: SM-I08。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/menu/session.rs:68` | `effect`: `want.set` | 対応: SM-I08。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/menu/session.rs:81` | `effect`: `path.update`, `focus_key.set` | 対応: SM-I08。同じ遷移の再登録をしない |
| `crates/kivact-interaction/src/scalar/range.rs:88` | `effect_on`: `binding.set` | 対応: SM-I902。同じ遷移の再登録をしない |
| `crates/kivact-motion/src/animated.rs:263` | `effect`: `first_cell.replace` | 対応: SM-R25。同じ遷移の再登録をしない |
| `crates/kivact-motion/src/animated.rs:381` | `effect`: `seen.replace` | 対応: SM-R25。同じ遷移の再登録をしない |
| `crates/kivact-reactive/src/frame.rs`: 183, 191 | `debug_assert` | 対応: SM-R08 |
| `crates/kivact-task/src/resource.rs:68` | `effect`: `cell.state.update`, `cell.latest.update`, `cell.state.update` | 対応: SM-R15。同じ遷移の再登録をしない |
| `crates/kivact-task/src/time.rs`: 79 | `sleep` | 対応: SM-R16 |
| `crates/kivact-task/src/worker_pool.rs`: 111 | `sleep` | 除外: worker 終了を待つ bounded polling。UI の phase/timer の実装でない |
| `crates/kivact-ui/src/button.rs`: 323 | `sleep` | 対応: SM-U905 |
| `crates/kivact-ui/src/number_input.rs:200` | `effect_on`: `typed.set` | 対応: SM-U902。同じ遷移の再登録をしない |
| `crates/kivact-ui/src/number_input.rs:233` | `effect_on`: `typed.set` | 対応: SM-U902。同じ遷移の再登録をしない |
| `crates/kivact-ui/src/progress.rs:186` | `effect`: `phase.set` | 対応: SM-U905。同じ遷移の再登録をしない |
| `crates/kivact-ui/src/select.rs:234` | `effect_on`: `sheet_open.set` | 対応: SM-U03。同じ遷移の再登録をしない |
| `crates/kivact-ui/src/switch_drag.rs:118` | `effect`: `gesture.contact.set`, `gesture.visual.set`, `gesture.generation.update` | 対応: SM-U01。同じ遷移の再登録をしない |
| `crates/kivact-ui/src/switch_drag.rs`: 289 | `sleep` | 対応: SM-U01 |
| `crates/kivact-ui/src/toast_rows.rs:24` | `effect`: `next_generation.set`, `live_slots.set`, `slots.update` | 対応: SM-U903。同じ遷移の再登録をしない |
| `crates/kivact-ui/src/toast_rows.rs:77` | `effect`: `height.set` | 対応: SM-U903。同じ遷移の再登録をしない |
| `crates/kivact-ui/src/toast_rows.rs:84` | `effect`: `slots.update` | 対応: SM-U903。同じ遷移の再登録をしない |
| `crates/kivact-ui/src/tree.rs:164` | `memo`: `previous_open.set` | 対応: SM-U04。同じ遷移の再登録をしない |
| `crates/kivact-ui/src/tree.rs:179` | `effect`: `last_height.set` | 対応: SM-U04。同じ遷移の再登録をしない |
| `crates/kivact-ui/src/data_table/filters.rs:249` | `effect`: `raw.set`, `valid.set`, `sent.set` | 対応: SM-U904。同じ遷移の再登録をしない |
| `crates/kivact-ui-basic/src/combobox.rs:386` | `effect`: `query.set` | 対応: SM-K16。同じ遷移の再登録をしない |
| `crates/kivact-ui-basic/src/navigation_menu.rs:261` | `effect_on`: `open.set` | 対応: SM-K12。同じ遷移の再登録をしない |
| `crates/kivact-ui-basic/src/questionnaire.rs:452` | `effect`: `phase.set` | 対応: SM-K914。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/picker.rs:554` | `effect`: `active_column.set` | 対応: SM-K905。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/range.rs:207` | `effect`: `active.set` | 対応: SM-K30、SM-K904。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/range.rs:214` | `effect`: `active.set`, `active.set`, `moved.set` | 対応: SM-K30、SM-K904。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/viewport_surface.rs:247` | `effect`: `previous.set` | 対応: SM-K918。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/audio/meter.rs:89` | `effect`: `clipped.set` | 対応: SM-K908。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/color_wheel/fields.rs:174` | `effect`: `selected.set` | 対応: SM-K903。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/color_wheel/view.rs:134` | `effect`: `state.hue.set` | 対応: SM-K902、SM-K903。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/color_wheel/view.rs:191` | `effect`: `state.blocked.set` | 対応: SM-K902、SM-K903。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/color_wheel/view.rs:200` | `effect`: `active.set` | 対応: SM-K902、SM-K903。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/editors/node/view.rs:196` | `effect`: `state.mode.update` | 対応: SM-K906。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/editors/shared/targets.rs:82` | `memo`: `table.update` | 対応: SM-K907。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/gradient/view.rs:155` | `effect`: `state.blocked.set` | 対応: SM-K30、SM-K902。同じ遷移の再登録をしない |
| `crates/kivact-ui-creative/src/outliner/view.rs:245` | `effect_on`: `pending.set`, `expanded.set`, `pending.set` | 対応: SM-K11、SM-K917。同じ遷移の再登録をしない |
| `crates/kivact-ui-game/src/display.rs:462` | `effect`: `revealed.set` | 対応: SM-K33。同じ遷移の再登録をしない |
| `crates/kivact-ui-game/src/surface.rs:123` | `effect`: `time.set` | 対応: SM-K33。同じ遷移の再登録をしない |
| `crates/kivact-ui-pop/src/date/calendar.rs:194` | `effect`: `visible.set` | 対応: SM-K912。同じ遷移の再登録をしない |
| `crates/kivact-ui-pop/src/date/input.rs:110` | `effect`: `valid.set` | 対応: SM-K20。同じ遷移の再登録をしない |
| `crates/kivact-ui-pop/src/date/range_input.rs:129` | `effect`: `valid.set` | 対応: SM-K20。同じ遷移の再登録をしない |
| `crates/kivact-ui-pop/src/date/text.rs:90` | `effect_on`: `checked.set` | 対応: SM-K913。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/app_shell.rs:246` | `effect_on`: `skip_requested.set` | 対応: SM-K915。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/avatar_group.rs:33` | `effect`: `active.set` | 対応: SM-K10。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/avatar_status_dot.rs:114` | `effect`: `cx.label.set` | 対応: SM-K916。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/banner.rs:143` | `effect`: `previous.set` | 対応: SM-K22。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/calendar.rs:315` | `effect`: `previous_month.replace`, `announcement.set` | 対応: SM-K912。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/calendar.rs:350` | `effect`: `local.set`, `pending.set` | 対応: SM-K912。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/carousel.rs:311` | `effect_on`: `last.replace` | 対応: SM-K01。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/carousel.rs`: 316 | `sleep` | 対応: SM-K01 |
| `crates/kivact-ui-soft/src/code_block.rs`: 529 | `sleep` | 対応: SM-K24 |
| `crates/kivact-ui-soft/src/date_input.rs:124` | `effect`: `draft.set`, `invalid.set` | 対応: SM-K18。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/empty_state.rs:92` | `effect`: `first.update`, `spoken.set`, `has_notice.update`, `spoken.set`, `has_notice.set` | 対応: SM-K23。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/hover_card.rs:117` | `effect`: `visited.set`, `initially_open.set` | 対応: SM-K14。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/hover_card.rs:154` | `effect`: `previous.set` | 対応: SM-K14。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/hover_card.rs:202` | `effect`: `dismissed.set` | 対応: SM-K14。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/month_swipe.rs`: 147 | `sleep` | 対応: SM-K28 |
| `crates/kivact-ui-soft/src/outline.rs`: 205 | `sleep` | 対応: SM-K10、SM-K26 |
| `crates/kivact-ui-soft/src/pagination.rs:637` | `effect_on`: `draft.set` | 除外: 入力/出力の adapter、cache、または nested handler の書込み。独立した phase の同期でない |
| `crates/kivact-ui-soft/src/rich_blocks.rs:46` | `effect`: `document.set`, `editor.whole.set` | 対応: SM-K909。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/rich_blocks.rs:105` | `effect`: `self.target.set`, `self.composing.set` | 対応: SM-K909。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/segmented_control.rs:117` | `effect`: `focused.set`, `focused.set` | 対応: SM-K25、SM-K27。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/soft_chat_tools.rs:510` | `effect`: `visible.set`, `visible.set`, `task.set`, `visible.set`, `visible.set`, `task_slot.set` | 対応: SM-K08。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/soft_chat_tools.rs`: 537 | `sleep` | 対応: SM-K08 |
| `crates/kivact-ui-soft/src/soft_overflow.rs:279` | `effect_on`: `widths_cache.update` | 対応: SM-K07。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/soft_overflow.rs:294` | `effect_on`: `widths_cache.update` | 対応: SM-K07。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/timer.rs:94` | `effect`: `display.set`, `display.set` | 対応: SM-K910。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/timer.rs`: 104 | `sleep` | 対応: SM-K910 |
| `crates/kivact-ui-soft/src/timestamp.rs:289` | `effect`: `display.set` | 対応: SM-K910。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/timestamp.rs`: 324 | `sleep` | 対応: SM-K910 |
| `crates/kivact-ui-soft/src/time_input.rs:131` | `effect`: `draft.set`, `invalid.set` | 対応: SM-K911。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/toolbar.rs:272` | `effect`: `seen.set`, `anchor.set`, `anchor.set`, `anchor.set` | 対応: SM-K25。同じ遷移の再登録をしない |
| `crates/kivact-ui-soft/src/toolbar.rs`: 279 | `sleep` | 対応: SM-K25 |
| `crates/kivact-ui-soft/src/typeahead.rs:186` | `effect`: `cached.set` | 対応: SM-K919。検索完了でLabelCache<K>の選択済みlabel保持規則を更新する。optionsはresource状態の直接memoを維持 |
| `crates/kivact-ui-soft/src/typeahead.rs:234` | `effect`: `sheet_open.set`, `editing.set`, `query.set` | 対応: SM-K16（open/filteringの遷移）。label cacheのSM-K919とは別 |
| `crates/kivact-ui-soft/src/typeahead.rs:246` | `effect`: `open.set` | 対応: SM-K16（open/filteringの遷移）。label cacheのSM-K919とは別 |
| `crates/kivact-ui-soft/src/typeahead.rs:497` | `effect_on`: `previous.replace`, `label.set` | 除外: token_noticesの翻訳messageへのcallback内の引数と差分履歴（上記の型別判断）。LabelCacheとは別 |
| `crates/kivact-ui-soft/src/typeahead.rs`: 178 | `sleep` | 除外: resourceがdebounceとcancellationを所有する非同期処理。LabelCacheの候補状態ではない |
| `crates/kivact-ui-soft/tests/search.rs`: 34 | `sleep` | 除外: harness の待機/不変条件の検証 |
| `examples/kivact/async/src/main.rs`: 11 | `sleep` | 除外: demo の非同期処理の模擬。利用者 code |
| `examples/kivact-ui-basic/src/docs/button.rs`: 139 | `sleep` | 除外: demo の非同期処理の模擬。利用者 code |
| `examples/kivact-ui-creative/src/creative/audio_samples.rs:219` | `effect`: `integrated.set` | 除外: 利用者 code（18.20）。demo の入力/外部結果の adapter |
| `examples/kivact-ui-game/src/game/controls.rs`: 119, 130, 412 | `sleep` | 除外: demo の非同期処理の模擬。利用者 code |
| `examples/kivact-ui-game/src/game/shell.rs:156` | `effect`: `current.set`, `current.set` | 除外: 利用者 code（18.20）。demo の入力/外部結果の adapter |
| `examples/kivact-ui-game/src/game/system.rs:352` | `effect`: `title.set` | 除外: 利用者 code（18.20）。demo の入力/外部結果の adapter |
| `examples/kivact-ui-game/src/game/system.rs:415` | `effect`: `confirmation.set` | 除外: 利用者 code（18.20）。demo の入力/外部結果の adapter |
| `examples/kivact-ui-pop/src/pop/components.rs`: 403, 782 | `sleep` | 除外: demo の非同期処理の模擬。利用者 code |
| `examples/kivact-ui-pop/src/pop/data.rs`: 44 | `sleep` | 除外: demo の非同期処理の模擬。利用者 code |
| `examples/kivact-ui-pop/src/pop/messaging.rs`: 50, 59, 211 | `sleep` | 除外: demo の非同期処理の模擬。利用者 code |
| `examples/kivact-ui-pop/src/pop/shell.rs:152` | `effect`: `selected.set` | 除外: 利用者 code（18.20）。demo の入力/外部結果の adapter |
| `examples/kivact-ui-pop/src/pop/shell.rs`: 89 | `sleep` | 除外: demo の非同期処理の模擬。利用者 code |

### 補助照合: Option と State/Phase/Mode の同居

二つの Option の検索に加え、Option が一つでも State/Phase/Mode 名の field と同居する struct を全文から抽出した（35 候補）。State という名前でも struct/値の場合があるため、enum かどうかを利用側で区別する。

| 場所 | 判断と理由 |
|---|---|
| `arvane-render/src/draw/painter.rs:122` Painter.index/Mode | 除外: index は frame が backdrop を含む場合の入力索引、Drawing/Replaying のどちらでも保持できる。Mode の phase は既に enum、hidden は group の nesting の深さで event counter ではない |
| `kivact-core/src/gpu_interop/share/link.rs:119` Open.phase/pixels | 対応: SM-C27。pixels は request の mode と独立して新しい CPU picture を take する mailbox（source の doc も whatever the request と明記） |
| `kivact-engine/src/host/pump.rs:45` Host | 対応: SM-E01/E02、その他の adapter/IME は既存の SM-E22/E26 |
| `kivact-engine/src/model/pointer/state.rs:13` Pointer | 対応: SM-E10〜E15。file_target は file drag の独立した routing target |
| `kivact-engine/src/model/aggregate.rs:12` Model | 対応: SM-E27/E32。layout/paint/a11y はそれぞれ独立の所有者 |
| `kivact-interaction/src/list/state.rs:41` State | 対応: SM-I14/I28 |
| `kivact-interaction/src/list/data.rs:238` ListPolicy | 除外: axis は設定、mode は一覧操作の設定。lifecycle の phase ではない |
| `kivact-interaction/src/menu/state.rs:55` MenuItemState、`table/data.rs:79` TableHeaderState | 除外: recipe に渡す snapshot record。PartState 自体も入力の値であり、この struct がその phase を変更しない |
| `kivact-ui/src/values.rs:101,242,345` recipe state | 除外: PartState/CheckedState/TableRowState を含む描画用の snapshot record |
| `kivact-style/src/draw/layer.rs:34` Layer、`kivact-types/src/protocol/effect.rs:57` Layer、`kivact-wire/src/drawing.rs:17` DrawingLayer/paint.rs:314 Effects | 除外: BlendMode と optional clipping/mask は描画の入力 record。Mode は描画設定で lifecycle ではない |
| `kivact-style/src/draw/recorder.rs:151` RegionSemantics、`kivact-wire/src/scene.rs:114` Region | 除外: CheckedState はチェックの入力値。semantics の optional properties は独立した公開 record |
| `arvane/src/test_display/virtual_window.rs:41`、`kivact-core/src/test/options.rs:36`、`kivact-engine/src/host/testing/test_options.rs:27` | 除外: WindowMode と locale/renderer の設定の束（18.14） |
| `kivact-core/src/view/input.rs:171`、`kivact-types/src/protocol/window.rs:11`、`kivact-wire/src/events.rs:26`、`kivact-ui-creative/src/viewport_surface.rs:31` | 除外: PointerPhase/pressure/tilt/region は入力 event の値。phase により破棄する可変な内部 field ではない |
| `kivact-ui-creative/src/audio/mixer_strip.rs:14`、`editors/{arrangement,graph,node,time}/view.rs`、`outliner/view.rs:50` | 除外: builder の入力。ChannelState/各 editor State は caller が渡す model の値、optional callback/renderer と phase 依存を持たない。動作中の node/outliner の同期は SM-K906/K917 |
| `kivact-ui-soft/src/app_shell.rs:111` | 除外: MobileNavMode と optional slot は構造の option。動作中の mobile は SM-K09、skip link は SM-K915 |
| `kivact-ui-soft/src/collapsible.rs:299` GroupCx | 対応: SM-K04。GroupMode と選択の不変条件 |
| `kivact-ui-soft/src/resizable.rs:204` ResizeHandle | 除外: optional grip と ResizableState は builder の表示入力。操作中の PanelSize は SM-K06 |
| `kivact-ui-creative/src/audio/transport.rs:13` TransportState、`audio/view_state.rs:5` ChannelState、`examples/kivact-ui-creative/src/creative/audio_values.rs:11` | 除外: caller 所有の音声 engine の値。play/record/cycle、mute/solo 等は kit が関連付けて phase を推定しない独立した control。stop で record arm を保持することも認める |
| `kivact-ui-soft/src/soft_chat_composer.rs:591` DictationState | 除外: caller 提供の capability/listening の表示入力。kit は speech engine を所有せず内部の開始/終了の規則を作らない |

### helper 経由の書込みと SM-C27 の訂正

- effect の直接 `.set` だけでなく全 kit の effect を列挙して helper を追った結果、SM-I904（palette.reset）と SM-K920〜923（cancel/commit/open）を追加した。この補助照合時点では32件を追加。後述のcheckerでSM-R901を足し、さらに NativeInput の構築を SM-A901 として追加し、最終の追加は34件、付録Aは計203件。
- `editors/{arrangement,time,graph}/view.rs` の `state.cancel()` は SM-K906 の同じ focus/data の取消しに含める。drag origin と CanvasGesture 自体は既存 SM-K30。
- `combobox.rs:488` の row.scroll_into_view、`tab_list.rs:425` の scroll_into_view は外部の node への effect で内部 signal の派生状態を複製しないため除外。
- `picker.rs:586` の outcomes.preview はすでに PickerPhase::Open { previewed } / Settled の明示状態で重複除去し、追加の bool/Option の組合せを作らないため除外。
- `calendar/state.rs:131` と `table/state.rs:144` の reconcile は Roving::reconcile に入り、requested/observed/restore_from の処理は既存 SM-I16。新規機械として重複登録しない。
- **SM-C27 の訂正（主担当決定）**: `gpu_interop/share/link.rs:124-127` は pixels を whatever the request の新しい CPU picture と明記する。公開 publish_pixels の GPU request 中の受理を保存するため、Phase::Pixels(Pixels) 案をやめ、Open が phase と独立した Mailbox<Pixels> を持つ形へ訂正する。Mailbox は V-4/S-9 の共通型。C-3 は振る舞い不変の unit なので、phase に閉じ込めて受理を制限する変更は行わない。

### clippy `disallowed_types` の全 crate 調査

実行: `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=2 cargo clippy --workspace --all-targets --all-features --locked --message-format=json -- -W clippy::disallowed_types`。exit **0**、2m33s。warning は全 target/feature の重複込み **194** 行、source location と内容でまとめた **43** 件を下表ですべて判定した。未処理 **0**。これは Windows host で有効な cfg の型検査であり、X11 など host で無効な cfg は前段の全文検索で照合する。

生ログ: `C:\Users\030009~1\AppData\Local\Temp\kivact-p1-clippy.jsonl`（compiler-message JSON）、`C:\Users\030009~1\AppData\Local\Temp\kivact-p1-clippy.stderr`（cargo進行と終了）。scratch log なので削除後も下表に判定が残る。

| 場所 | 重複込み件数 | 判断と理由 |
|---|---:|---|
| `crates/arvane-native/src/module.rs:14` | 2 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/arvane-native/src/module.rs:18` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/arvane-native/src/native_input/windows.rs:516` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/arvane-native/src/test_display.rs:47` | 2 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/arvane-native/src/test_display.rs:169` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-core/src/client/link.rs:19` | 2 | 対応: SM-C05/SM-R19（host waker の後付け） |
| `crates/kivact-core/src/client/link.rs:30` | 2 | 対応: SM-C05/SM-R19（host waker の後付け） |
| `crates/kivact-core/src/test/screen/driver.rs:113` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-core/src/view/state.rs:10` | 2 | 対応: SM-C08（Started の後付け） |
| `crates/kivact-core/src/view/state.rs:225` | 2 | 対応: SM-C08（Started の後付け） |
| `crates/kivact-core/src/view/state.rs:250` | 2 | 対応: SM-C08（Started の後付け） |
| `crates/kivact-engine/src/measure/leaf.rs:319` | 2 | 除外: LeafText の一測定に閉じた inherited computed-text cache（S-9、上記の型別判断） |
| `crates/kivact-engine/src/measure/leaf.rs:404` | 2 | 除外: LeafText の一測定に閉じた inherited computed-text cache（S-9、上記の型別判断） |
| `crates/kivact-reactive/src/data.rs:20` | 2 | 対応: SM-R09（driver の後付け） |
| `crates/kivact-reactive/src/data.rs:68` | 2 | 対応: SM-R09（driver の後付け） |
| `crates/kivact-reactive/src/runtime.rs:4` | 2 | 対応: SM-R09（driver の後付け） |
| `crates/kivact-reactive/src/runtime.rs:28` | 2 | 対応: SM-R09（driver の後付け） |
| `crates/kivact-task/src/spawn.rs:3` | 2 | 対応: SM-R17（TaskCell の後付け） |
| `crates/kivact-task/src/spawn.rs:50` | 2 | 対応: SM-R17（TaskCell の後付け） |
| `crates/kivact-task/src/state.rs:8` | 2 | 対応: SM-R17（TaskCell の後付け） |
| `crates/kivact-task/src/state.rs:50` | 2 | 対応: SM-R17（TaskCell の後付け） |
| `crates/kivact-ui-basic/src/icons.rs:278` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-basic/src/recipes.rs:24` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-creative/src/icons.rs:310` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-creative/src/recipes.rs:23` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-game/src/icons.rs:143` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-game/src/recipes.rs:20` | 2 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-game/src/recipes.rs:49` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-pop/src/icons.rs:225` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-pop/src/recipes.rs:34` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-soft/src/icons.rs:3` | 2 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-soft/src/icons.rs:104` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-soft/src/recipes.rs:3` | 2 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui-soft/src/recipes.rs:6` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui/src/recipes.rs:62` | 2 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui/src/recipes.rs:99` | 80 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `crates/kivact-ui/src/text_input.rs:13` | 2 | 対応: SM-U901（instance の callback の後付け） |
| `crates/kivact-ui/src/text_input.rs:36` | 2 | 対応: SM-U901（instance の callback の後付け） |
| `crates/kivact-ui/src/text_input.rs:46` | 2 | 対応: SM-U901（instance の callback の後付け） |
| `crates/kivact-ui/src/text_input.rs:84` | 2 | 対応: SM-U901（instance の callback の後付け） |
| `crates/kivact-ui/src/text_input.rs:86` | 2 | 対応: SM-U901（instance の callback の後付け） |
| `examples/kivact-ui-basic/src/docs.rs:11` | 2 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |
| `examples/kivact-ui-basic/src/docs.rs:81` | 4 | 除外: static の process cache / recipe / icons / message id / test display（18.20）。import と構築の指摘も同じ利用場所に対応 |

- NativeInput.waker の照合: `runtime.rs:653-664` は default 後に Some を埋め、`embed/windows/window.rs:295` は default の None のまま使う。これは take mailbox ではないので既存 SM-A08 とは別に SM-A901 を追加した。Direct と EventLoop の構造の option を構築時に選ぶことで、embed に不要な waker を強制せず native の未設定状態をなくす。

### P-0 checker の最終 report（d76f849 取込み後）

P-0 の lib/bin root の補正を取り込み、専用の `target/p1-checker` で xtask を build（exit0）し直してから実行。各 workspace package を `--crate` で明示して policy の skip を上書きした。43 crate × 2 rules = **86** 呼出すべて exit **0**。`--report` なので、これは違反0という意味ではなく、調査出力の完了を意味する。state-fields **431** 行、effect-writes **90** 行、計 **521** 行を以下で一行ずつ分類し、未処理 **0**。所要173秒。

コマンドの形: `target/p1-checker/debug/xtask.exe check --only <state-fields|effect-writes> --crate <package> --report --no-write`。二つの read-only process までで実行。P-0 修正前の予備版も86呼出を完走したが、以下は修正後の結果だけを採用する。

生ログ: `C:\Users\030009~1\AppData\Local\Temp\kivact-p1-reports-final.json`（全出力行、全終了コード、expected=86、checker_commit=d76f849）。旧予備ログ `C:\Users\030009~1\AppData\Local\Temp\kivact-p1-reports.json` は最終の根拠に使わない。scratch log がなくても下表で全行を追える。

| package | state-fields | effect-writes | exit |
|---|---:|---:|---:|
| `arvane` | 3 | 0 | 0 / 0 |
| `arvane-asset` | 2 | 0 | 0 / 0 |
| `arvane-native` | 9 | 0 | 0 / 0 |
| `arvane-render` | 5 | 0 | 0 / 0 |
| `arvane-shell` | 3 | 0 | 0 / 0 |
| `arvane-text` | 6 | 0 | 0 / 0 |
| `gpu-interop-producer` | 1 | 0 | 0 / 0 |
| `kivact` | 0 | 0 | 0 / 0 |
| `kivact-core` | 43 | 0 | 0 / 0 |
| `kivact-docs` | 0 | 0 | 0 / 0 |
| `kivact-editing` | 2 | 0 | 0 / 0 |
| `kivact-engine` | 29 | 0 | 0 / 0 |
| `kivact-example-async` | 0 | 0 | 0 / 0 |
| `kivact-example-component` | 0 | 0 | 0 / 0 |
| `kivact-example-counter` | 0 | 0 | 0 / 0 |
| `kivact-example-form` | 0 | 0 | 0 / 0 |
| `kivact-example-list` | 0 | 0 | 0 / 0 |
| `kivact-gpu-share` | 2 | 0 | 0 / 0 |
| `kivact-i18n` | 0 | 0 | 0 / 0 |
| `kivact-interaction` | 50 | 22 | 0 / 0 |
| `kivact-layout` | 5 | 0 | 0 / 0 |
| `kivact-lower` | 2 | 0 | 0 / 0 |
| `kivact-motion` | 5 | 0 | 0 / 0 |
| `kivact-public-api-tests` | 0 | 0 | 0 / 0 |
| `kivact-reactive` | 7 | 0 | 0 / 0 |
| `kivact-style` | 13 | 0 | 0 / 0 |
| `kivact-task` | 3 | 0 | 0 / 0 |
| `kivact-template-basic` | 0 | 0 | 0 / 0 |
| `kivact-text` | 3 | 0 | 0 / 0 |
| `kivact-types` | 12 | 0 | 0 / 0 |
| `kivact-ui` | 37 | 13 | 0 / 0 |
| `kivact-ui-basic` | 33 | 2 | 0 / 0 |
| `kivact-ui-basic-catalog` | 0 | 0 | 0 / 0 |
| `kivact-ui-creative` | 34 | 11 | 0 / 0 |
| `kivact-ui-creative-catalog` | 1 | 0 | 0 / 0 |
| `kivact-ui-game` | 1 | 2 | 0 / 0 |
| `kivact-ui-game-catalog` | 1 | 0 | 0 / 0 |
| `kivact-ui-pop` | 2 | 1 | 0 / 0 |
| `kivact-ui-pop-catalog` | 2 | 0 | 0 / 0 |
| `kivact-ui-soft` | 83 | 39 | 0 / 0 |
| `kivact-ui-soft-catalog` | 1 | 0 | 0 / 0 |
| `kivact-wire` | 30 | 0 | 0 / 0 |
| `xtask` | 1 | 0 | 0 / 0 |

| checker | 場所・対象（実出力） | 判断と理由 |
|---|---|---|
| `state-fields` | `crates/arvane-asset/src/store/entry.rs:114` `arvane-asset::store::entry`: VariantKey combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/arvane-asset/src/store/worker.rs:73` `arvane-asset::store::worker`: Worker combines state fields | 除外: 上記の型別判断（所有権移動、履歴、cache、topology） |
| `state-fields` | `crates/arvane-native/src/embed/windows/state.rs:59` `arvane-native::embed::windows::state`: State combines state fields | 対応: SM-A01、SM-A02、SM-A03。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/arvane-native/src/embed/x11/ime.rs:49` `arvane-native::embed::x11::ime`: App combines state fields | 対応: SM-A06。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/arvane-native/src/embed/x11/ime.rs:65` `arvane-native::embed::x11::ime`: Connection combines state fields | 対応: SM-A06。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/arvane-native/src/embed/x11/window.rs:18` `arvane-native::embed::x11::window`: Window combines state fields | 対応: SM-A01、SM-A05。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/arvane-native/src/native_input/state.rs:68` `arvane-native::native_input::state`: ContactNote combines state fields | 対応: SM-A08（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/arvane-native/src/runtime.rs:513` `arvane-native::runtime`: Runner combines state fields | 対応: SM-A07、SM-A10（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/arvane-native/src/runtime.rs:85` `arvane-native::runtime`: WinitWindow combines state fields | 対応: SM-A07、SM-A10（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/arvane-native/src/test_display.rs:110` `arvane-native::test_display`: Pending combines state fields | 対応: SM-A15（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/arvane-native/src/test_display.rs:376` `arvane-native::test_display`: Entry combines state fields | 対応: SM-A15（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/arvane-render/src/draw/canvas.rs:57` `arvane-render::draw::canvas`: GlyphRun combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/arvane-render/src/gpu/state.rs:145` `arvane-render::gpu::state`: GpuSurface combines state fields | 対応: SM-A12（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/arvane-render/src/gpu/state.rs:19` `arvane-render::gpu::state`: GpuEnv combines state fields | 対応: SM-A12（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/arvane-render/src/gpu/state.rs:67` `arvane-render::gpu::state`: GpuContext combines state fields | 対応: SM-A12（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/arvane-render/src/paint.rs:43` `arvane-render::paint`: PatternFill combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/arvane-shell/src/clicks.rs:43` `arvane-shell::clicks`: ClickCounter combines state fields | 除外: 上記の型別判断（所有権移動、履歴、cache、topology） |
| `state-fields` | `crates/arvane-shell/src/ime.rs:5` `arvane-shell::ime`: ShellImeText combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/arvane-shell/src/services.rs:5` `arvane-shell::services`: ShellFileDialog combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/arvane-text/src/cache.rs:115` `arvane-text::cache`: TextKey combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/arvane-text/src/cache.rs:232` `arvane-text::cache`: Cache combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/arvane-text/src/cache.rs:79` `arvane-text::cache`: SpanBits combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/arvane-text/src/cache.rs:98` `arvane-text::cache`: RichKey combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/arvane-text/src/layouts.rs:27` `arvane-text::layouts`: SpanKey combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/arvane-text/src/layouts.rs:83` `arvane-text::layouts`: Key combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/arvane/src/env.rs:28` `arvane::env`: Env combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/arvane/src/test_display/shell.rs:18` `arvane::test_display::shell`: VirtualShell combines state fields | 対応: SM-A15（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/arvane/src/window.rs:336` `arvane::window`: Shell combines state fields | 対応: SM-A10（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `dev/tests/gpu-interop-producer/src/vulkan.rs:38` `gpu-interop-producer::vulkan`: Vulkan combines state fields | 除外: 試験producerのGPU resource。device/poolのOptionは失敗した構築とDropの所有権移動で、公開中の操作phaseではない |
| `state-fields` | `crates/kivact-core/src/app.rs:11` `kivact-core::app`: App combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/client/link.rs:24` `kivact-core::client::link`: HostLink combines state fields | 対応: SM-R04（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-core/src/client/state.rs:31` `kivact-core::client::state`: HostState combines state fields | 対応: SM-C02、SM-C25。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/embed.rs:29` `kivact-core::embed`: Parent combines state fields | 対応: SM-C04、SM-C28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/gpu_interop/share/app_side.rs:18` `kivact-core::gpu_interop::share::app_side`: SharedImage combines state fields | 対応: SM-C26。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/gpu_interop/share/link.rs:118` `kivact-core::gpu_interop::share::link`: Open combines state fields | 対応: SM-C27（独立したCPU pictureのMailbox、主担当訂正） |
| `state-fields` | `crates/kivact-core/src/platform/services.rs:37` `kivact-core::platform::services`: FileDialog combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/primitive/measurement.rs:17` `kivact-core::primitive::measurement`: Constraints combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/test/screen/model.rs:16` `kivact-core::test::screen::model`: Screen combines state fields | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `state-fields` | `crates/kivact-core/src/test/screen/results.rs:226` `kivact-core::test::screen::results`: LayoutViolation combines state fields | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `state-fields` | `crates/kivact-core/src/test/screen/results.rs:294` `kivact-core::test::screen::results`: SemanticNode combines state fields | 除外: 試験の入力・結果/handle の受け渡し。製品の phase を複製しない |
| `state-fields` | `crates/kivact-core/src/view/common.rs:209` `kivact-core::view::common`: Registry combines state fields | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/common.rs:24` `kivact-core::view::common`: Common combines state fields | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/common.rs:242` `kivact-core::view::common`: Capture combines state fields | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/common.rs:255` `kivact-core::view::common`: Constraints combines state fields | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/common.rs:266` `kivact-core::view::common`: SemanticSources combines state fields | 対応: SM-C16、SM-C17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/editing_data.rs:76` `kivact-core::view::editing_data`: TextEditing combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/editing_data.rs:88` `kivact-core::view::editing_data`: EditorProps combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/flow/branch.rs:167` `kivact-core::view::flow::branch`: ShowSome combines state fields | 対応: SM-C19。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/flow/branch.rs:68` `kivact-core::view::flow::branch`: Show combines state fields | 対応: SM-C19。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/flow/error_boundary.rs:33` `kivact-core::view::flow::error_boundary`: ErrorBoundary combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/flow/virtual_list.rs:60` `kivact-core::view::flow::virtual_list`: VirtualList combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-core/src/view/mount/create.rs:16` `kivact-core::view::mount::create`: FirstWrite combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/mount/facts.rs:10` `kivact-core::view::mount::facts`: NodeFacts combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/mount/part_input.rs:36` `kivact-core::view::mount::part_input`: Selected combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/mount/parts.rs:125` `kivact-core::view::mount::parts`: ConstraintSources combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/mount/resolve.rs:31` `kivact-core::view::mount::resolve`: Resolved combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/mount/semantics.rs:168` `kivact-core::view::mount::semantics`: SemanticProps combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/mount/style.rs:24` `kivact-core::view::mount::style`: States combines state fields | 対応: SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/mount/style.rs:68` `kivact-core::view::mount::style`: NodeStyle combines state fields | 対応: SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/overlays.rs:500` `kivact-core::view::overlays`: KeyRoute combines state fields | 対応: SM-C22。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/part.rs:110` `kivact-core::view::part`: InputState combines state fields | 対応: SM-C15。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/part.rs:73` `kivact-core::view::part`: PartState combines state fields | 対応: SM-C15。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/part_data.rs:11` `kivact-core::view::part_data`: PartMount combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/part_data.rs:29` `kivact-core::view::part_data`: PartConstraints combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-core/src/view/state.rs:141` `kivact-core::view::state`: FocusKeys combines state fields | 対応: SM-C08、SM-C09、SM-C10、SM-C14、SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/state.rs:202` `kivact-core::view::state`: Started combines state fields | 対応: SM-C08、SM-C09、SM-C10、SM-C14、SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/state.rs:59` `kivact-core::view::state`: NodeRefState combines state fields | 対応: SM-C08、SM-C09、SM-C10、SM-C14、SM-C18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/tree.rs:140` `kivact-core::view::tree`: PartActions combines state fields | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/tree.rs:207` `kivact-core::view::tree`: NodeObservation combines state fields | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/tree.rs:222` `kivact-core::view::tree`: TNode combines state fields | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/tree.rs:280` `kivact-core::view::tree`: Tree combines state fields | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-core/src/view/tree.rs:59` `kivact-core::view::tree`: Handlers combines state fields | 対応: SM-C07、SM-C11、SM-C12、SM-C13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-editing/src/behavior.rs:30` `kivact-editing::behavior`: EditContext combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-editing/src/edit_state.rs:38` `kivact-editing::edit_state`: EditState combines state fields | 対応: SM-E32（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-engine/src/a11y/state.rs:41` `kivact-engine::a11y::state`: A11yState combines state fields | 対応: SM-E23、SM-E25。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/a11y/state.rs:9` `kivact-engine::a11y::state`: Metadata combines state fields | 対応: SM-E23、SM-E25。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/check/boxes.rs:66` `kivact-engine::check::boxes`: Axes combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-engine/src/host/pump.rs:45` `kivact-engine::host::pump`: Host combines state fields | 対応: SM-E01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-engine/src/host/testing/test_host.rs:21` `kivact-engine::host::testing::test_host`: Counts combines state fields | 対応: SM-A15（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-engine/src/host/testing/test_host.rs:46` `kivact-engine::host::testing::test_host`: ArvaneTestHost combines state fields | 対応: SM-A15（test hostのwoken mailbox）。failureは最後のerrorを渡す独立した値 |
| `state-fields` | `crates/kivact-engine/src/measure/editor.rs:5` `kivact-engine::measure::editor`: EditorNode combines state fields | 対応: SM-E19（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-engine/src/measure/state.rs:13` `kivact-engine::measure::state`: MeasureColumns combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-engine/src/measure/state.rs:46` `kivact-engine::measure::state`: TextExtents combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-engine/src/model/aggregate.rs:11` `kivact-engine::model::aggregate`: Model combines state fields | 対応: SM-E32（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-engine/src/model/changes.rs:110` `kivact-engine::model::changes`: Changes combines state fields | 対応: SM-E24、SM-E27。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/model/changes.rs:131` `kivact-engine::model::changes`: Dispatched combines state fields | 対応: SM-E24、SM-E27。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/model/edit/state.rs:23` `kivact-engine::model::edit::state`: SessionSource combines state fields | 対応: SM-E20、SM-E32。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/model/edit/state.rs:8` `kivact-engine::model::edit::state`: InputSource combines state fields | 対応: SM-E20、SM-E32。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/model/focus/state.rs:7` `kivact-engine::model::focus::state`: Focused combines state fields | 対応: SM-E23。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/model/lifecycle.rs:47` `kivact-engine::model::lifecycle`: RemovedPayload combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-engine/src/model/pointer/state.rs:112` `kivact-engine::model::pointer::state`: Route combines state fields | 対応: SM-E10、SM-E11、SM-E12、SM-E13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/model/pointer/state.rs:203` `kivact-engine::model::pointer::state`: PointerSource combines state fields | 対応: SM-E10、SM-E11、SM-E12、SM-E13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/model/state_motion/state.rs:49` `kivact-engine::model::state_motion::state`: StateAnimation combines state fields | 対応: SM-E31。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/mutation.rs:95` `kivact-engine::mutation`: Mutation combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-engine/src/paint/state.rs:113` `kivact-engine::paint::state`: Step combines state fields | 対応: SM-E08、SM-E09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/paint/state.rs:158` `kivact-engine::paint::state`: PaintList combines state fields | 対応: SM-E08、SM-E09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/paint/state.rs:19` `kivact-engine::paint::state`: PaintState combines state fields | 対応: SM-E08、SM-E09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/paint/state.rs:75` `kivact-engine::paint::state`: Look combines state fields | 対応: SM-E08、SM-E09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/tree/child_order.rs:12` `kivact-engine::tree::child_order`: Editing combines state fields | 除外: 上記の型別判断（所有権移動、履歴、cache、topology） |
| `state-fields` | `crates/kivact-engine/src/tree/child_order.rs:187` `kivact-engine::tree::child_order`: Children combines state fields | 除外: 上記の型別判断（所有権移動、履歴、cache、topology） |
| `state-fields` | `crates/kivact-engine/src/tree/child_order.rs:6` `kivact-engine::tree::child_order`: Link combines state fields | 除外: 上記の型別判断（所有権移動、履歴、cache、topology） |
| `state-fields` | `crates/kivact-engine/src/tree/node.rs:20` `kivact-engine::tree::node`: HostNode combines state fields | 対応: SM-E30、SM-E31。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-engine/src/tree/node.rs:51` `kivact-engine::tree::node`: Place combines state fields | 対応: SM-E30、SM-E31。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-gpu-share/src/common.rs:8` `kivact-gpu-share::common`: Identity combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-gpu-share/src/pool.rs:104` `kivact-gpu-share::pool`: Slots combines state fields | 対応: SM-A14（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `effect-writes` | `crates/kivact-interaction/src/calendar/state.rs:76` `kivact-interaction::calendar::state`: Signal/Var write inside an effect closure | 対応: SM-I26、SM-I903（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/combobox.rs:159` `kivact-interaction::combobox`: Signal/Var write inside an effect closure | 対応: SM-I06（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/combobox.rs:174` `kivact-interaction::combobox`: Signal/Var write inside an effect closure | 対応: SM-I06（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/hover_corridor.rs:110` `kivact-interaction::hover_corridor`: Signal/Var write inside an effect closure | 対応: SM-I21（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/hover_corridor.rs:112` `kivact-interaction::hover_corridor`: Signal/Var write inside an effect closure | 対応: SM-I21（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/hover_corridor.rs:119` `kivact-interaction::hover_corridor`: Signal/Var write inside an effect closure | 対応: SM-I21（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/menu/level.rs:496` `kivact-interaction::menu::level`: Signal/Var write inside an effect closure | 除外: surface rects のgeometry cache更新（S-9）。sessionの開閉状態はSM-I08/I10 |
| `effect-writes` | `crates/kivact-interaction/src/menu/session.rs:61` `kivact-interaction::menu::session`: Signal/Var write inside an effect closure | 対応: SM-I08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/menu/session.rs:62` `kivact-interaction::menu::session`: Signal/Var write inside an effect closure | 対応: SM-I08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/menu/session.rs:63` `kivact-interaction::menu::session`: Signal/Var write inside an effect closure | 対応: SM-I08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/menu/session.rs:64` `kivact-interaction::menu::session`: Signal/Var write inside an effect closure | 対応: SM-I08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/menu/session.rs:74` `kivact-interaction::menu::session`: Signal/Var write inside an effect closure | 対応: SM-I08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/menu/session.rs:97` `kivact-interaction::menu::session`: Signal/Var write inside an effect closure | 対応: SM-I08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/menu/session.rs:98` `kivact-interaction::menu::session`: Signal/Var write inside an effect closure | 対応: SM-I08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/popup.rs:409` `kivact-interaction::popup`: Signal/Var write inside an effect closure | 対応: SM-I02（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/popup.rs:415` `kivact-interaction::popup`: Signal/Var write inside an effect closure | 対応: SM-I02（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/select.rs:149` `kivact-interaction::select`: Signal/Var write inside an effect closure | 対応: SM-I07（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/select.rs:594` `kivact-interaction::select`: Signal/Var write inside an effect closure | 対応: SM-I07（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/toaster.rs:419` `kivact-interaction::toaster`: Signal/Var write inside an effect closure | 対応: SM-I22（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/toaster.rs:420` `kivact-interaction::toaster`: Signal/Var write inside an effect closure | 対応: SM-I22（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/toaster.rs:451` `kivact-interaction::toaster`: Signal/Var write inside an effect closure | 対応: SM-I22（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-interaction/src/toaster.rs:565` `kivact-interaction::toaster`: Signal/Var write inside an effect closure | 対応: SM-I22（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `state-fields` | `crates/kivact-interaction/src/calendar/handle.rs:32` `kivact-interaction::calendar::handle`: CalendarDayState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/calendar/policy.rs:17` `kivact-interaction::calendar::policy`: CalendarPolicy combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/combobox.rs:39` `kivact-interaction::combobox`: Combobox combines state fields | 対応: SM-I06。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/delayed.rs:48` `kivact-interaction::delayed`: Delayed combines state fields | 対応: SM-I03。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/dialog.rs:72` `kivact-interaction::dialog`: Dismiss combines state fields | 対応: SM-I05。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/dialog.rs:91` `kivact-interaction::dialog`: DialogSettings combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-interaction/src/disclosure.rs:27` `kivact-interaction::disclosure`: Disclosure combines state fields | 対応: SM-I01。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/disclosure.rs:59` `kivact-interaction::disclosure`: DisclosureSettings combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-interaction/src/drag.rs:179` `kivact-interaction::drag`: DragSettings combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-interaction/src/drag.rs:225` `kivact-interaction::drag`: Drag combines state fields | 対応: SM-I12。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/edit.rs:85` `kivact-interaction::edit`: EditSession combines state fields | 対応: SM-I11。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/follow_end.rs:91` `kivact-interaction::follow_end`: FollowState combines state fields | 対応: SM-I23。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/list/data.rs:236` `kivact-interaction::list::data`: ListPolicy combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/list/data.rs:299` `kivact-interaction::list::data`: ListItemState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/list/state.rs:20` `kivact-interaction::list::state`: Pending combines state fields | 対応: SM-I14、SM-I26。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/list/state.rs:41` `kivact-interaction::list::state`: State combines state fields | 対応: SM-I14、SM-I26。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/menu/level.rs:50` `kivact-interaction::menu::level`: Level combines state fields | 対応: SM-I10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/menu/press_state.rs:45` `kivact-interaction::menu::press_state`: Press combines state fields | 対応: SM-I09。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/menu/state.rs:140` `kivact-interaction::menu::state`: ItemEntry combines state fields | 対応: SM-I08。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/menu/state.rs:53` `kivact-interaction::menu::state`: MenuItemState combines state fields | 対応: SM-I08。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/menu/state.rs:618` `kivact-interaction::menu::state`: Session combines state fields | 対応: SM-I08。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/palette.rs:197` `kivact-interaction::palette`: CommandPalette combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/popup.rs:110` `kivact-interaction::popup`: PopupSettings combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-interaction/src/popup.rs:119` `kivact-interaction::popup`: Opening combines state fields | 対応: SM-I02。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/roving.rs:23` `kivact-interaction::roving`: Member combines state fields | 対応: SM-I16。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/roving.rs:43` `kivact-interaction::roving`: Observation combines state fields | 対応: SM-I16。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/roving.rs:58` `kivact-interaction::roving`: Roving combines state fields | 対応: SM-I16。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/scalar/edit_state.rs:111` `kivact-interaction::scalar::edit_state`: Edit combines state fields | 対応: SM-I17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/scalar/edit_state.rs:12` `kivact-interaction::scalar::edit_state`: Shared combines state fields | 対応: SM-I17。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/scalar/touch.rs:24` `kivact-interaction::scalar::touch`: Contact combines state fields | 対応: SM-I18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/scalar/touch.rs:44` `kivact-interaction::scalar::touch`: Touch combines state fields | 対応: SM-I18。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/scalar/track.rs:12` `kivact-interaction::scalar::track`: Target combines state fields | 対応: SM-I901（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-interaction/src/scalar/track.rs:128` `kivact-interaction::scalar::track`: Pointer combines state fields | 対応: SM-I901（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-interaction/src/scalar/track.rs:80` `kivact-interaction::scalar::track`: Geometry combines state fields | 対応: SM-I901（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-interaction/src/splitter.rs:119` `kivact-interaction::splitter`: SplitterSettings combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-interaction/src/table/data.rs:7` `kivact-interaction::table::data`: Header combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/table/data.rs:77` `kivact-interaction::table::data`: TableHeaderState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/table/data.rs:88` `kivact-interaction::table::data`: TableRowState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/table/data.rs:98` `kivact-interaction::table::data`: TableCellState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/table/model.rs:241` `kivact-interaction::table::model`: TableModel combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/table/model.rs:53` `kivact-interaction::table::model`: ColumnDef combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/table/state.rs:19` `kivact-interaction::table::state`: State combines state fields | 対応: SM-I26、SM-I27。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/text_edit/handle.rs:315` `kivact-interaction::text_edit::handle`: Data combines state fields | 対応: SM-I13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/toaster.rs:232` `kivact-interaction::toaster`: Toaster combines state fields | 対応: SM-I22。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/toaster.rs:59` `kivact-interaction::toaster`: ToastFocus combines state fields | 対応: SM-I22。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/toaster.rs:92` `kivact-interaction::toaster`: Toast combines state fields | 対応: SM-I22。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/toggle.rs:90` `kivact-interaction::toggle`: Core combines state fields | 対応: SM-I24。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/tooltip.rs:69` `kivact-interaction::tooltip`: Hub combines state fields | 対応: SM-I04。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-interaction/src/tree.rs:322` `kivact-interaction::tree`: TreeItemState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-interaction/src/zoom_pan.rs:139` `kivact-interaction::zoom_pan`: ZoomPan combines state fields | 対応: SM-I19、SM-I28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-layout/src/columns.rs:39` `kivact-layout::columns`: Counted combines state fields | 対応: SM-E28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-layout/src/columns.rs:90` `kivact-layout::columns`: NodeLayoutValues combines state fields | 対応: SM-E28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-layout/src/pass_state.rs:21` `kivact-layout::pass_state`: LayoutState combines state fields | 対応: SM-E28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-layout/src/style/node.rs:32` `kivact-layout::style::node`: NodeStyle combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-layout/src/tree.rs:51` `kivact-layout::tree`: LayoutPass combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-lower/src/context.rs:44` `kivact-lower::context`: Context combines state fields | 対応: SM-E33（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-lower/src/glyphs.rs:25` `kivact-lower::glyphs`: Colors combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-motion/src/animated.rs:19` `kivact-motion::animated`: AnimCell combines state fields | 対応: SM-R25（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-motion/src/state.rs:16` `kivact-motion::state`: Frames combines state fields | 対応: SM-R22、SM-R23、SM-R24、SM-C23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-motion/src/state.rs:294` `kivact-motion::state`: LoopCell combines state fields | 対応: SM-R22、SM-R23、SM-R24、SM-C23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-motion/src/state.rs:38` `kivact-motion::state`: MotionEnv combines state fields | 対応: SM-R22、SM-R23、SM-R24、SM-C23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-motion/src/state.rs:50` `kivact-motion::state`: MotionState combines state fields | 対応: SM-R22、SM-R23、SM-R24、SM-C23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-reactive/src/arena.rs:33` `kivact-reactive::arena`: Entry combines state fields | 対応: SM-R901。value/tomb の排他と free/retired を明示する |
| `state-fields` | `crates/kivact-reactive/src/data.rs:127` `kivact-reactive::data`: MemoCell combines state fields | 対応: SM-R01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-reactive/src/data.rs:274` `kivact-reactive::data`: OwnerEntry combines state fields | 対応: SM-R01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-reactive/src/data.rs:309` `kivact-reactive::data`: Ctx combines state fields | 対応: SM-R01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-reactive/src/data.rs:45` `kivact-reactive::data`: Runtime combines state fields | 対応: SM-R01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-reactive/src/error.rs:87` `kivact-reactive::error`: Inner combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-reactive/src/host.rs:16` `kivact-reactive::host`: Pending combines state fields | 対応: SM-R10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-style/src/draw/layer.rs:9` `kivact-style::draw::layer`: Layer combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/draw/recorder.rs:145` `kivact-style::draw::recorder`: RegionSemantics combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/draw/text_run.rs:11` `kivact-style::draw::text_run`: TextStyle combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/grid.rs:260` `kivact-style::grid`: GridStyle combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/paint/gradient.rs:40` `kivact-style::paint::gradient`: Gradient combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/path/trim.rs:14` `kivact-style::path::trim`: Contour combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/resolve.rs:191` `kivact-style::resolve`: Interactive combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/resolve.rs:46` `kivact-style::resolve`: CompiledStyle combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/sheet.rs:1097` `kivact-style::sheet`: ScrollbarStyle combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/sheet.rs:160` `kivact-style::sheet`: Sides combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/sheet.rs:177` `kivact-style::sheet`: Style combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/text.rs:213` `kivact-style::text`: TextExtras combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-style/src/text.rs:350` `kivact-style::text`: SpanStyle combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-task/src/action.rs:189` `kivact-task::action`: ActionCell combines state fields | 対応: SM-R13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-task/src/latest.rs:18` `kivact-task::latest`: LatestRun combines state fields | 対応: SM-R14（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-task/src/state.rs:53` `kivact-task::state`: TaskState combines state fields | 対応: SM-R17、SM-R18、SM-R21、SM-C23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-text/src/records.rs:19` `kivact-text::records`: Key combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-text/src/records.rs:66` `kivact-text::records`: Kept combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-text/src/source.rs:5` `kivact-text::source`: Source combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-types/src/geometry.rs:153` `kivact-types::geometry`: IntrinsicSize combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/gpu_interop.rs:48` `kivact-types::gpu_interop`: GpuPreference combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/gpu_interop.rs:67` `kivact-types::gpu_interop`: GpuAdapter combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/protocol/effect.rs:44` `kivact-types::protocol::effect`: Layer combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/protocol/effect.rs:83` `kivact-types::protocol::effect`: BoxShadow combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/protocol/paint.rs:196` `kivact-types::protocol::paint`: Pattern combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/protocol/semantics.rs:115` `kivact-types::protocol::semantics`: SemanticsProperties combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/protocol/semantics.rs:18` `kivact-types::protocol::semantics`: SemanticsNode combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/protocol/text.rs:134` `kivact-types::protocol::text`: SpanStyle combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/protocol/text.rs:38` `kivact-types::protocol::text`: TextDecoration combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/protocol/text.rs:53` `kivact-types::protocol::text`: TextStyle combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-types/src/protocol/window.rs:61` `kivact-types::protocol::window`: Environment combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `effect-writes` | `crates/kivact-ui-basic/src/navigation_menu.rs:263` `kivact-ui-basic::navigation_menu`: Signal/Var write inside an effect closure | 対応: SM-K12（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-basic/src/questionnaire.rs:458` `kivact-ui-basic::questionnaire`: Signal/Var write inside an effect closure | 対応: SM-K914（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `state-fields` | `crates/kivact-ui-basic/src/alert.rs:20` `kivact-ui-basic::alert`: Alert combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/attachment.rs:53` `kivact-ui-basic::attachment`: Attachment combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/attachment.rs:69` `kivact-ui-basic::attachment`: Cfg combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-basic/src/avatar.rs:43` `kivact-ui-basic::avatar`: Avatar combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/badge.rs:32` `kivact-ui-basic::badge`: Badge combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/button_group.rs:11` `kivact-ui-basic::button_group`: ButtonGroup combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/calendar/view.rs:15` `kivact-ui-basic::calendar::view`: Calendar combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/card.rs:31` `kivact-ui-basic::card`: CardHeader combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-basic/src/card.rs:96` `kivact-ui-basic::card`: Card combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/chart/model.rs:99` `kivact-ui-basic::chart::model`: Frame combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-basic/src/chart/view.rs:32` `kivact-ui-basic::chart::view`: Chart combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/code_block/view.rs:55` `kivact-ui-basic::code_block::view`: CodeBlock combines state fields | 対応: SM-K24（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-basic/src/combobox.rs:39` `kivact-ui-basic::combobox`: Combobox combines state fields | 対応: SM-K16（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-basic/src/date_picker.rs:112` `kivact-ui-basic::date_picker`: DatePicker combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/date_picker.rs:18` `kivact-ui-basic::date_picker`: Picker combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-basic/src/date_picker.rs:198` `kivact-ui-basic::date_picker`: DateRangePicker combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/disclosure.rs:10` `kivact-ui-basic::disclosure`: Collapsible combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-basic/src/drawer.rs:24` `kivact-ui-basic::drawer`: Drawer combines state fields | 対応: SM-K28（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-basic/src/input_group.rs:29` `kivact-ui-basic::input_group`: Around combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-basic/src/input_group.rs:54` `kivact-ui-basic::input_group`: InputGroup combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/item.rs:49` `kivact-ui-basic::item`: Item combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/native_select.rs:26` `kivact-ui-basic::native_select`: NativeSelect combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/navigation_menu.rs:350` `kivact-ui-basic::navigation_menu`: NavLink combines state fields | 対応: SM-K12（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-basic/src/pagination.rs:80` `kivact-ui-basic::pagination`: PaginationLink combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/questionnaire.rs:67` `kivact-ui-basic::questionnaire`: QuestionnaireStep combines state fields | 対応: SM-K914（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-basic/src/sidebar/menu.rs:164` `kivact-ui-basic::sidebar::menu`: SidebarMenuButton combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/sidebar/menu.rs:178` `kivact-ui-basic::sidebar::menu`: ButtonCfg combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-basic/src/sidebar/menu.rs:27` `kivact-ui-basic::sidebar::menu`: SidebarGroup combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/sidebar/menu.rs:477` `kivact-ui-basic::sidebar::menu`: SidebarMenuSubButton combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/sidebar/panel.rs:16` `kivact-ui-basic::sidebar::panel`: Sidebar combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/sidebar/state.rs:53` `kivact-ui-basic::sidebar::state`: SidebarCx combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-basic/src/table.rs:139` `kivact-ui-basic::table`: TableCell combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-basic/src/trace.rs:20` `kivact-ui-basic::trace`: TraceLine combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `examples/kivact-ui-creative/src/creative/audio_values.rs:10` `kivact-ui-creative-catalog::creative::audio_values`: TransportState combines state fields | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |
| `effect-writes` | `crates/kivact-ui-creative/src/audio/meter.rs:91` `kivact-ui-creative::audio::meter`: Signal/Var write inside an effect closure | 対応: SM-K908（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-creative/src/color_wheel/fields.rs:177` `kivact-ui-creative::color_wheel::fields`: Signal/Var write inside an effect closure | 対応: SM-K903（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-creative/src/color_wheel/view.rs:203` `kivact-ui-creative::color_wheel::view`: Signal/Var write inside an effect closure | 対応: SM-K902、SM-K903（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-creative/src/outliner/view.rs:249` `kivact-ui-creative::outliner::view`: Signal/Var write inside an effect closure | 対応: SM-K11、SM-K917（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-creative/src/outliner/view.rs:276` `kivact-ui-creative::outliner::view`: Signal/Var write inside an effect closure | 対応: SM-K11、SM-K917（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-creative/src/picker.rs:560` `kivact-ui-creative::picker`: Signal/Var write inside an effect closure | 対応: SM-K905（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-creative/src/range.rs:209` `kivact-ui-creative::range`: Signal/Var write inside an effect closure | 対応: SM-K30、SM-K904（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-creative/src/range.rs:218` `kivact-ui-creative::range`: Signal/Var write inside an effect closure | 対応: SM-K30、SM-K904（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-creative/src/range.rs:221` `kivact-ui-creative::range`: Signal/Var write inside an effect closure | 対応: SM-K30、SM-K904（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-creative/src/range.rs:225` `kivact-ui-creative::range`: Signal/Var write inside an effect closure | 対応: SM-K30、SM-K904（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-creative/src/viewport_surface.rs:259` `kivact-ui-creative::viewport_surface`: Signal/Var write inside an effect closure | 対応: SM-K918（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `state-fields` | `crates/kivact-ui-creative/src/audio/meter.rs:10` `kivact-ui-creative::audio::meter`: Meter combines state fields | 対応: SM-K908（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-creative/src/audio/transport.rs:11` `kivact-ui-creative::audio::transport`: TransportState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/audio/view_state.rs:3` `kivact-ui-creative::audio::view_state`: ChannelState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/color_wheel/state.rs:19` `kivact-ui-creative::color_wheel::state`: State combines state fields | 対応: SM-K902（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-creative/src/color_wheel/view.rs:21` `kivact-ui-creative::color_wheel::view`: ColorWheel combines state fields | 対応: SM-K903（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-creative/src/dock/data.rs:23` `kivact-ui-creative::dock::data`: Shared combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/dock/data.rs:52` `kivact-ui-creative::dock::data`: DockCx combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/dock/model.rs:47` `kivact-ui-creative::dock::model`: DockTab combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/dock/view.rs:27` `kivact-ui-creative::dock::view`: Dock combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-creative/src/editors/arrangement/gesture.rs:40` `kivact-ui-creative::editors::arrangement::gesture`: State combines state fields | 対応: SM-K30（CanvasGesture の payload と drag 開始値の写し）。node の focus/data の取消しは SM-K906 |
| `state-fields` | `crates/kivact-ui-creative/src/editors/arrangement/model.rs:51` `kivact-ui-creative::editors::arrangement::model`: TimelineTrack combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/editors/graph/gesture.rs:71` `kivact-ui-creative::editors::graph::gesture`: State combines state fields | 対応: SM-K30（CanvasGesture の payload と drag 開始値の写し）。node の focus/data の取消しは SM-K906 |
| `state-fields` | `crates/kivact-ui-creative/src/editors/node/gesture.rs:111` `kivact-ui-creative::editors::node::gesture`: State combines state fields | 対応: SM-K30（CanvasGesture の payload と drag 開始値の写し）。node の focus/data の取消しは SM-K906 |
| `state-fields` | `crates/kivact-ui-creative/src/editors/node/gesture.rs:49` `kivact-ui-creative::editors::node::gesture`: Info combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/editors/node/model.rs:32` `kivact-ui-creative::editors::node::model`: NodeData combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/editors/shared/gesture.rs:12` `kivact-ui-creative::editors::shared::gesture`: Pressed combines state fields | 対応: SM-K30。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui-creative/src/editors/shared/lane.rs:11` `kivact-ui-creative::editors::shared::lane`: LaneExtensions combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/editors/shared/model.rs:18` `kivact-ui-creative::editors::shared::model`: EditorContext combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/editors/shared/region_parts.rs:19` `kivact-ui-creative::editors::shared::region_parts`: RegionInfo combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/editors/time/gesture.rs:30` `kivact-ui-creative::editors::time::gesture`: State combines state fields | 対応: SM-K30（CanvasGesture の payload と drag 開始値の写し）。node の focus/data の取消しは SM-K906 |
| `state-fields` | `crates/kivact-ui-creative/src/editors/time/view.rs:22` `kivact-ui-creative::editors::time::view`: TimeEditor combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/gradient/state.rs:22` `kivact-ui-creative::gradient::state`: State combines state fields | 対応: SM-K902。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui-creative/src/outliner/view.rs:29` `kivact-ui-creative::outliner::view`: Outliner combines state fields | 対応: SM-K11、SM-K917（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-creative/src/outliner_data.rs:57` `kivact-ui-creative::outliner_data`: TreeDrop combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/outliner_data.rs:6` `kivact-ui-creative::outliner_data`: TreeItem combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-creative/src/picker.rs:173` `kivact-ui-creative::picker`: Parts combines state fields | 対応: SM-K905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-creative/src/picker.rs:28` `kivact-ui-creative::picker`: PickerItem combines state fields | 対応: SM-K905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-creative/src/picker.rs:415` `kivact-ui-creative::picker`: Outcomes combines state fields | 対応: SM-K905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-creative/src/picker.rs:473` `kivact-ui-creative::picker`: Session combines state fields | 対応: SM-K905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-creative/src/pie.rs:34` `kivact-ui-creative::pie`: PieState combines state fields | 対応: SM-K901（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-creative/src/property_row.rs:15` `kivact-ui-creative::property_row`: PropertyRow combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-creative/src/range.rs:38` `kivact-ui-creative::range`: Range combines state fields | 対応: SM-K904（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-creative/src/viewport_surface.rs:29` `kivact-ui-creative::viewport_surface`: SurfacePointer combines state fields | 除外: pointer の入力 record。PointerPhase と optional region は caller に渡す値（18.14） |
| `state-fields` | `crates/kivact-ui-creative/src/viewport_surface.rs:60` `kivact-ui-creative::viewport_surface`: ViewportSurface combines state fields | 対応: SM-K918（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `examples/kivact-ui-game/src/game/state.rs:5` `kivact-ui-game-catalog::game::state`: Demo combines state fields | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |
| `effect-writes` | `crates/kivact-ui-game/src/display.rs:469` `kivact-ui-game::display`: Signal/Var write inside an effect closure | 対応: SM-K33（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-game/src/surface.rs:132` `kivact-ui-game::surface`: Signal/Var write inside an effect closure | 対応: SM-K33（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `state-fields` | `crates/kivact-ui-game/src/layout.rs:12` `kivact-ui-game::layout`: TreeEntry combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `examples/kivact-ui-pop/src/pop/messaging.rs:18` `kivact-ui-pop-catalog::pop::messaging`: ChatEntry combines state fields | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |
| `state-fields` | `examples/kivact-ui-pop/src/pop/mission.rs:7` `kivact-ui-pop-catalog::pop::mission`: Mission combines state fields | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |
| `effect-writes` | `crates/kivact-ui-pop/src/date/calendar.rs:197` `kivact-ui-pop::date::calendar`: Signal/Var write inside an effect closure | 対応: SM-K912（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `state-fields` | `crates/kivact-ui-pop/src/date/calendar.rs:32` `kivact-ui-pop::date::calendar`: Calendar combines state fields | 対応: SM-K912（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-pop/src/date/text.rs:65` `kivact-ui-pop::date::text`: DateText combines state fields | 対応: SM-K913。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `examples/kivact-ui-soft/src/model.rs:6` `kivact-ui-soft-catalog::model`: CatalogState combines state fields | 除外: 利用者の demo の値/入力。独立した表示・操作の事実 |
| `effect-writes` | `crates/kivact-ui-soft/src/app_shell.rs:248` `kivact-ui-soft::app_shell`: Signal/Var write inside an effect closure | 対応: SM-K915（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/banner.rs:146` `kivact-ui-soft::banner`: Signal/Var write inside an effect closure | 対応: SM-K22（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/calendar.rs:318` `kivact-ui-soft::calendar`: Signal/Var write inside an effect closure | 対応: SM-K912（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/calendar.rs:351` `kivact-ui-soft::calendar`: Signal/Var write inside an effect closure | 対応: SM-K912（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/calendar.rs:352` `kivact-ui-soft::calendar`: Signal/Var write inside an effect closure | 対応: SM-K912（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/date_input.rs:125` `kivact-ui-soft::date_input`: Signal/Var write inside an effect closure | 対応: SM-K18（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/date_input.rs:126` `kivact-ui-soft::date_input`: Signal/Var write inside an effect closure | 対応: SM-K18（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/empty_state.rs:108` `kivact-ui-soft::empty_state`: Signal/Var write inside an effect closure | 対応: SM-K23（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/empty_state.rs:112` `kivact-ui-soft::empty_state`: Signal/Var write inside an effect closure | 対応: SM-K23（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/empty_state.rs:115` `kivact-ui-soft::empty_state`: Signal/Var write inside an effect closure | 対応: SM-K23（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/empty_state.rs:96` `kivact-ui-soft::empty_state`: Signal/Var write inside an effect closure | 対応: SM-K23（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/empty_state.rs:99` `kivact-ui-soft::empty_state`: Signal/Var write inside an effect closure | 対応: SM-K23（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/hover_card.rs:119` `kivact-ui-soft::hover_card`: Signal/Var write inside an effect closure | 対応: SM-K14（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/hover_card.rs:121` `kivact-ui-soft::hover_card`: Signal/Var write inside an effect closure | 対応: SM-K14（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/hover_card.rs:157` `kivact-ui-soft::hover_card`: Signal/Var write inside an effect closure | 対応: SM-K14（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/hover_card.rs:204` `kivact-ui-soft::hover_card`: Signal/Var write inside an effect closure | 対応: SM-K14（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/pagination.rs:637` `kivact-ui-soft::pagination`: Signal/Var write inside an effect closure | 対応: SM-K922（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/rich_blocks.rs:107` `kivact-ui-soft::rich_blocks`: Signal/Var write inside an effect closure | 対応: SM-K909（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/rich_blocks.rs:108` `kivact-ui-soft::rich_blocks`: Signal/Var write inside an effect closure | 対応: SM-K909（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/rich_blocks.rs:50` `kivact-ui-soft::rich_blocks`: Signal/Var write inside an effect closure | 対応: SM-K909（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/segmented_control.rs:119` `kivact-ui-soft::segmented_control`: Signal/Var write inside an effect closure | 対応: SM-K25、SM-K27（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/segmented_control.rs:121` `kivact-ui-soft::segmented_control`: Signal/Var write inside an effect closure | 対応: SM-K25、SM-K27（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/soft_chat_tools.rs:518` `kivact-ui-soft::soft_chat_tools`: Signal/Var write inside an effect closure | 対応: SM-K08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/soft_chat_tools.rs:522` `kivact-ui-soft::soft_chat_tools`: Signal/Var write inside an effect closure | 対応: SM-K08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/soft_chat_tools.rs:540` `kivact-ui-soft::soft_chat_tools`: Signal/Var write inside an effect closure | 対応: SM-K08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/soft_chat_tools.rs:562` `kivact-ui-soft::soft_chat_tools`: Signal/Var write inside an effect closure | 対応: SM-K08（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/soft_overflow.rs:282` `kivact-ui-soft::soft_overflow`: Signal/Var write inside an effect closure | 除外: 測定widthのcache更新（S-9）。fitのphaseは既存SM-K07 |
| `effect-writes` | `crates/kivact-ui-soft/src/soft_overflow.rs:296` `kivact-ui-soft::soft_overflow`: Signal/Var write inside an effect closure | 除外: 測定widthのcache更新（S-9）。fitのphaseは既存SM-K07 |
| `effect-writes` | `crates/kivact-ui-soft/src/time_input.rs:132` `kivact-ui-soft::time_input`: Signal/Var write inside an effect closure | 対応: SM-K911（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/time_input.rs:133` `kivact-ui-soft::time_input`: Signal/Var write inside an effect closure | 対応: SM-K911（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/toolbar.rs:276` `kivact-ui-soft::toolbar`: Signal/Var write inside an effect closure | 対応: SM-K25（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/toolbar.rs:277` `kivact-ui-soft::toolbar`: Signal/Var write inside an effect closure | 対応: SM-K25（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/toolbar.rs:280` `kivact-ui-soft::toolbar`: Signal/Var write inside an effect closure | 対応: SM-K25（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/toolbar.rs:284` `kivact-ui-soft::toolbar`: Signal/Var write inside an effect closure | 対応: SM-K25（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui-soft/src/typeahead.rs:198` `kivact-ui-soft::typeahead`: Signal/Var write inside an effect closure | 対応: SM-K919。検索完了でLabelCache<K>の選択済みlabel保持規則を更新する。optionsはresource状態の直接memoを維持 |
| `effect-writes` | `crates/kivact-ui-soft/src/typeahead.rs:236` `kivact-ui-soft::typeahead`: Signal/Var write inside an effect closure | 対応: SM-K16（open/filteringの遷移）。label cacheのSM-K919とは別 |
| `effect-writes` | `crates/kivact-ui-soft/src/typeahead.rs:237` `kivact-ui-soft::typeahead`: Signal/Var write inside an effect closure | 対応: SM-K16（open/filteringの遷移）。label cacheのSM-K919とは別 |
| `effect-writes` | `crates/kivact-ui-soft/src/typeahead.rs:248` `kivact-ui-soft::typeahead`: Signal/Var write inside an effect closure | 対応: SM-K16（open/filteringの遷移）。label cacheのSM-K919とは別 |
| `effect-writes` | `crates/kivact-ui-soft/src/typeahead.rs:514` `kivact-ui-soft::typeahead`: Signal/Var write inside an effect closure | 除外: token_notices の翻訳messageへのcallback内の引数。独立した表示stateを後追いで保持しない（上記判断） |
| `state-fields` | `crates/kivact-ui-soft/src/alert_dialog.rs:11` `kivact-ui-soft::alert_dialog`: AlertDialog combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/app_shell.rs:111` `kivact-ui-soft::app_shell`: AppShell combines state fields | 対応: SM-K915（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/app_shell.rs:47` `kivact-ui-soft::app_shell`: MobileNavConfig combines state fields | 対応: SM-K915（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/avatar.rs:39` `kivact-ui-soft::avatar`: Avatar combines state fields | 対応: SM-K32（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/avatar_group.rs:14` `kivact-ui-soft::avatar_group`: AvatarGroupContext combines state fields | 対応: SM-K10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui-soft/src/avatar_group_overflow.rs:15` `kivact-ui-soft::avatar_group_overflow`: AvatarGroupOverflow combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/avatar_status_dot.rs:60` `kivact-ui-soft::avatar_status_dot`: AvatarStatusDot combines state fields | 対応: SM-K916（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/banner.rs:34` `kivact-ui-soft::banner`: Banner combines state fields | 対応: SM-K22（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/bottom_sheet.rs:53` `kivact-ui-soft::bottom_sheet`: BottomSheet combines state fields | 対応: SM-K28（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/breadcrumbs.rs:115` `kivact-ui-soft::breadcrumbs`: BreadcrumbItem combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/button_group.rs:16` `kivact-ui-soft::button_group`: GroupMember combines state fields | 対応: SM-K10（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/calendar.rs:16` `kivact-ui-soft::calendar`: Calendar combines state fields | 対応: SM-K912（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/card.rs:32` `kivact-ui-soft::card`: Card combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/carousel.rs:177` `kivact-ui-soft::carousel`: Carousel combines state fields | 対応: SM-K01（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/citation.rs:11` `kivact-ui-soft::citation`: CitationSource combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/code_block.rs:130` `kivact-ui-soft::code_block`: CodeBlock combines state fields | 対応: SM-K24（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/collapsible.rs:35` `kivact-ui-soft::collapsible`: CollapsibleConfig combines state fields | 対応: SM-K03、SM-K04（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/collapsible.rs:53` `kivact-ui-soft::collapsible`: CollapsibleState combines state fields | 対応: SM-K03、SM-K04（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/collapsible.rs:83` `kivact-ui-soft::collapsible`: Collapsible combines state fields | 対応: SM-K03、SM-K04（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/container_reveal.rs:22` `kivact-ui-soft::container_reveal`: RevealContent combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/dialog_header.rs:18` `kivact-ui-soft::dialog_header`: DialogHeader combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/divider.rs:16` `kivact-ui-soft::divider`: Divider combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/empty_state.rs:13` `kivact-ui-soft::empty_state`: EmptyState combines state fields | 対応: SM-K23（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/grid.rs:25` `kivact-ui-soft::grid`: Grid combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/hover_card.rs:22` `kivact-ui-soft::hover_card`: HoverCard combines state fields | 対応: SM-K14（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/layout.rs:132` `kivact-ui-soft::layout`: LayoutSlots combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/layout.rs:262` `kivact-ui-soft::layout`: LayoutRegion combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/layout.rs:4` `kivact-ui-soft::layout`: LayoutInset combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/layout.rs:46` `kivact-ui-soft::layout`: Layout combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/lightbox.rs:32` `kivact-ui-soft::lightbox`: Lightbox combines state fields | 対応: SM-K05（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/lightbox.rs:63` `kivact-ui-soft::lightbox`: LightboxHandle combines state fields | 対応: SM-K05（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/link.rs:27` `kivact-ui-soft::link`: Link combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/markdown.rs:729` `kivact-ui-soft::markdown`: Marks combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/markdown_parser.rs:14` `kivact-ui-soft::markdown_parser`: ListEntry combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/mobile_nav.rs:17` `kivact-ui-soft::mobile_nav`: AppShellMobile combines state fields | 対応: SM-K09（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/mobile_nav.rs:276` `kivact-ui-soft::mobile_nav`: MobileNavToggle combines state fields | 対応: SM-K09（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/mobile_nav.rs:71` `kivact-ui-soft::mobile_nav`: MobileNav combines state fields | 対応: SM-K09（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/month_swipe.rs:24` `kivact-ui-soft::month_swipe`: MonthSwipe combines state fields | 対応: SM-K28。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui-soft/src/more_menu.rs:10` `kivact-ui-soft::more_menu`: MoreMenu combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/nav_heading.rs:22` `kivact-ui-soft::nav_heading`: NavHeadingMenuItem combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/outline.rs:143` `kivact-ui-soft::outline`: Navigation combines state fields | 対応: SM-K26（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/outline.rs:50` `kivact-ui-soft::outline`: Outline combines state fields | 対応: SM-K26（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/pagination.rs:129` `kivact-ui-soft::pagination`: Model combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/pagination.rs:28` `kivact-ui-soft::pagination`: Pagination combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/resizable.rs:38` `kivact-ui-soft::resizable`: ResizableConfig combines state fields | 対応: SM-K06（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/rich_blocks.rs:19` `kivact-ui-soft::rich_blocks`: Editor combines state fields | 対応: SM-K909（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/scrollable_area.rs:159` `kivact-ui-soft::scrollable_area`: AxisScroll combines state fields | 対応: SM-K07（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/scrollable_area.rs:215` `kivact-ui-soft::scrollable_area`: ScrollOverflow combines state fields | 対応: SM-K07（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/scrollable_area.rs:31` `kivact-ui-soft::scrollable_area`: ScrollableArea combines state fields | 対応: SM-K07（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/section.rs:44` `kivact-ui-soft::section`: Section combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/segmented_control.rs:33` `kivact-ui-soft::segmented_control`: SegmentedControl combines state fields | 対応: SM-K27（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/side_nav.rs:28` `kivact-ui-soft::side_nav`: SideNavCx combines state fields | 対応: SM-K13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui-soft/src/side_nav.rs:341` `kivact-ui-soft::side_nav`: SideNavCollapseButton combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/side_nav.rs:427` `kivact-ui-soft::side_nav`: SideNavItem combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/side_nav.rs:50` `kivact-ui-soft::side_nav`: SideNavResize combines state fields | 対応: SM-K13。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui-soft/src/side_nav.rs:70` `kivact-ui-soft::side_nav`: SideNav combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/side_nav.rs:878` `kivact-ui-soft::side_nav`: SideNavSection combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/soft_chat_composer.rs:590` `kivact-ui-soft::soft_chat_composer`: DictationState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/soft_chat_tools.rs:30` `kivact-ui-soft::soft_chat_tools`: ToolCall combines state fields | 対応: SM-K08（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/spinner.rs:32` `kivact-ui-soft::spinner`: Spinner combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/stack.rs:218` `kivact-ui-soft::stack`: Stack combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/stack.rs:50` `kivact-ui-soft::stack`: Padding combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/stepper.rs:206` `kivact-ui-soft::stepper`: StepperCx combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/stepper.rs:346` `kivact-ui-soft::stepper`: Summary combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui-soft/src/stepper.rs:66` `kivact-ui-soft::stepper`: Step combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/tab_list.rs:107` `kivact-ui-soft::tab_list`: TabMenuOption combines state fields | 対応: SM-K10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui-soft/src/tab_list.rs:178` `kivact-ui-soft::tab_list`: TabList combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/tab_list.rs:260` `kivact-ui-soft::tab_list`: FocusMember combines state fields | 対応: SM-K10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui-soft/src/tab_list.rs:266` `kivact-ui-soft::tab_list`: FocusTrack combines state fields | 対応: SM-K10。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui-soft/src/tab_list.rs:48` `kivact-ui-soft::tab_list`: Tab combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/text.rs:101` `kivact-ui-soft::text`: Text combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/thumbnail.rs:27` `kivact-ui-soft::thumbnail`: Thumbnail combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/timestamp.rs:143` `kivact-ui-soft::timestamp`: TimestampEntry combines state fields | 対応: SM-K910。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui-soft/src/timestamp.rs:170` `kivact-ui-soft::timestamp`: Timestamp combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/toggle_button_group.rs:82` `kivact-ui-soft::toggle_button_group`: ToggleButtonGroup combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/token.rs:51` `kivact-ui-soft::token`: Token combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui-soft/src/toolbar.rs:31` `kivact-ui-soft::toolbar`: Toolbar combines state fields | 対応: SM-K25（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/top_nav.rs:175` `kivact-ui-soft::top_nav`: TopNavItem combines state fields | 対応: SM-K13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/top_nav.rs:33` `kivact-ui-soft::top_nav`: Slots combines state fields | 対応: SM-K13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/top_nav.rs:436` `kivact-ui-soft::top_nav`: TopNavMegaMenu combines state fields | 対応: SM-K13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/top_nav.rs:590` `kivact-ui-soft::top_nav`: TopNavMegaMenuItem combines state fields | 対応: SM-K13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/top_nav.rs:724` `kivact-ui-soft::top_nav`: TopNavMegaMenuFeaturedCard combines state fields | 対応: SM-K13（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui-soft/src/typeahead.rs:26` `kivact-ui-soft::typeahead`: Typeahead combines state fields | 対応: SM-K16、SM-K919（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `effect-writes` | `crates/kivact-ui/src/data_table/filters.rs:252` `kivact-ui::data_table::filters`: Signal/Var write inside an effect closure | 対応: SM-U904（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/data_table/filters.rs:253` `kivact-ui::data_table::filters`: Signal/Var write inside an effect closure | 対応: SM-U904（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/data_table/filters.rs:254` `kivact-ui::data_table::filters`: Signal/Var write inside an effect closure | 対応: SM-U904（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/number_input.rs:206` `kivact-ui::number_input`: Signal/Var write inside an effect closure | 対応: SM-U902（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/number_input.rs:235` `kivact-ui::number_input`: Signal/Var write inside an effect closure | 対応: SM-U902（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/progress.rs:194` `kivact-ui::progress`: Signal/Var write inside an effect closure | 対応: SM-U905（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/select.rs:236` `kivact-ui::select`: Signal/Var write inside an effect closure | 対応: SM-U03（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/toast_rows.rs:36` `kivact-ui::toast_rows`: Signal/Var write inside an effect closure | 対応: SM-U903（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/toast_rows.rs:44` `kivact-ui::toast_rows`: Signal/Var write inside an effect closure | 対応: SM-U903（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/toast_rows.rs:46` `kivact-ui::toast_rows`: Signal/Var write inside an effect closure | 対応: SM-U903（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/toast_rows.rs:81` `kivact-ui::toast_rows`: Signal/Var write inside an effect closure | 対応: SM-U903（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/toast_rows.rs:86` `kivact-ui::toast_rows`: Signal/Var write inside an effect closure | 対応: SM-U903（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `effect-writes` | `crates/kivact-ui/src/tree.rs:184` `kivact-ui::tree`: Signal/Var write inside an effect closure | 対応: SM-U04（同じfileの状態の遷移。新規との境界は各項目の不正欄） |
| `state-fields` | `crates/kivact-ui/src/button.rs:79` `kivact-ui::button`: Button combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui/src/button.rs:92` `kivact-ui::button`: Settings combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui/src/command_palette.rs:20` `kivact-ui::command_palette`: CommandPalette combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/data_table/row_status.rs:11` `kivact-ui::data_table::row_status`: RowStatus combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/data_table/rows.rs:30` `kivact-ui::data_table::rows`: Grid combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/data_table/rows.rs:62` `kivact-ui::data_table::rows`: StickyColumns combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/data_table/sticky_paint.rs:7` `kivact-ui::data_table::sticky_paint`: DataTableStickyPaint combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/data_table/types.rs:31` `kivact-ui::data_table::types`: DataTableContext combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/data_table/types.rs:97` `kivact-ui::data_table::types`: DataTable combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/dialog.rs:46` `kivact-ui::dialog`: Dialog combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/disclosure.rs:20` `kivact-ui::disclosure`: Disclosure combines state fields | 対応: SM-U06（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui/src/field.rs:18` `kivact-ui::field`: Field combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui/src/list.rs:16` `kivact-ui::list`: List combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui/src/menu.rs:255` `kivact-ui::menu`: Level combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/menu.rs:38` `kivact-ui::menu`: Menu combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/progress.rs:19` `kivact-ui::progress`: ProgressMotion combines state fields | 対応: SM-U905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui/src/progress.rs:52` `kivact-ui::progress`: Progress combines state fields | 対応: SM-U905（同じ file の状態。option/input の field 自体は依存のない record と区別） |
| `state-fields` | `crates/kivact-ui/src/radio_group.rs:21` `kivact-ui::radio_group`: RadioGroup combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui/src/select.rs:31` `kivact-ui::select`: Select combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui/src/slider.rs:196` `kivact-ui::slider`: Geometry combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/slider.rs:49` `kivact-ui::slider`: Slider combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui/src/split.rs:159` `kivact-ui::split`: Group combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/split.rs:28` `kivact-ui::split`: Split combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui/src/splitter.rs:15` `kivact-ui::splitter`: Splitter combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui/src/switch_drag.rs:24` `kivact-ui::switch_drag`: Contact combines state fields | 対応: SM-U01。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui/src/switch_drag.rs:91` `kivact-ui::switch_drag`: Gesture combines state fields | 対応: SM-U01。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui/src/tabs.rs:29` `kivact-ui::tabs`: Tabs combines state fields | 除外: builder/設定の option の束（18.14）。動作中の phase ではない |
| `state-fields` | `crates/kivact-ui/src/text_draft.rs:13` `kivact-ui::text_draft`: Draft combines state fields | 対応: SM-U05。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui/src/toast_swipe.rs:66` `kivact-ui::toast_swipe`: Swipe combines state fields | 対応: SM-U02。既存と同じ状態は再登録しない。独立した入力/option record は18.14で除外 |
| `state-fields` | `crates/kivact-ui/src/tooltip.rs:14` `kivact-ui::tooltip`: Tooltip combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/values.rs:147` `kivact-ui::values`: ToggleButtonState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/values.rs:227` `kivact-ui::values`: SwitchDragState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/values.rs:240` `kivact-ui::values`: CheckState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/values.rs:272` `kivact-ui::values`: DialogState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/values.rs:296` `kivact-ui::values`: SelectTriggerState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/values.rs:342` `kivact-ui::values`: DataTableRowState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-ui/src/values.rs:98` `kivact-ui::values`: ButtonState combines state fields | 除外: 設定/入力/出力または geometry の record。field は独立したデータで、利用可能性を別 field の lifecycle が決めない |
| `state-fields` | `crates/kivact-wire/src/drawing.rs:15` `kivact-wire::drawing`: DrawingLayer combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/events.rs:58` `kivact-wire::events`: HostEnv combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/grid.rs:10` `kivact-wire::grid`: GridProps combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/host.rs:26` `kivact-wire::host`: TurnOutput combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/measure.rs:32` `kivact-wire::measure`: MeasureRequest combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/node.rs:117` `kivact-wire::node`: NodeSemantics combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/node.rs:144` `kivact-wire::node`: BoxData combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/node.rs:153` `kivact-wire::node`: TextData combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/node.rs:162` `kivact-wire::node`: EditingData combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/node.rs:191` `kivact-wire::node`: EditLineLayout combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/node.rs:198` `kivact-wire::node`: ImageData combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/node.rs:274` `kivact-wire::node`: OverlayFit combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/node.rs:301` `kivact-wire::node`: OverlayProps combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/node.rs:343` `kivact-wire::node`: FileDialogRequest combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/node.rs:40` `kivact-wire::node`: NodeProperties combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/paint.rs:311` `kivact-wire::paint`: Effects combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/paint.rs:45` `kivact-wire::paint`: FillLayer combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/scene.rs:111` `kivact-wire::scene`: Region combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/scene.rs:15` `kivact-wire::scene`: TextRun combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/semantics.rs:7` `kivact-wire::semantics`: SemanticsProperties combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/style.rs:11` `kivact-wire::style`: LayoutProps combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/style.rs:170` `kivact-wire::style`: PaintProps combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/style.rs:215` `kivact-wire::style`: PaintDelta combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/style.rs:234` `kivact-wire::style`: StateStyles combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/style.rs:295` `kivact-wire::style`: ScrollbarDelta combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/text.rs:223` `kivact-wire::text`: ComputedText combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/text.rs:253` `kivact-wire::text`: TextMore combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/text.rs:327` `kivact-wire::text`: SpanProps combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/text.rs:52` `kivact-wire::text`: TextDecoration combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `crates/kivact-wire/src/text.rs:82` `kivact-wire::text`: TextProps combines state fields | 除外: 公開/内部の値・geometry・style の record、または名前付き text cache（18.14/S-9） |
| `state-fields` | `dev/xtask/src/check_source.rs:12` `xtask::check_source`: Source combines state fields | 除外: source targetの解析結果record。libraryはtargetの分類、Option targetはfixtureの匿名identityで、実行phaseではない |

### P-1 の結論と状態の点検表

- 指定検索の未処理 **0**、checker の未処理 **0/521**、clippy の未処理 **0/43**。新規 **34**（SM-R901、SM-A901、SM-I901〜904、SM-U901〜905、SM-K901〜923）で付録 A は **203** 件。既存では主担当が決定した SM-C27 の訂正だけを行った。
- NativeInput は接尾辞 Input のため checker のrole除外に当たるが、全文検索と構築側の照合で SM-A901 を発見した。checker だけを全数の根拠にしない。
- 1: bool は独立した事実/入力と、複数fieldでphaseを推定するものを区別し、後者を付録に対応。
- 2: Option の依存を型ごとに照合。arena の value/tomb の依存を SM-R901 に追加。
- 3: 派生値の保存/複製を列挙し、signalをmemoにするか単一snapshotへ移す形を記載。
- 4: effect の直接書込み90行とhelper経由のcancel/reset/commit/openを照合。Machineの遷移へ集める形を記載。
- 5: doc指定語2089行をpath/lineで記録し、既存Stage/型状態・callback順・geometryの説明に分類。
- 6: OnceCell/OnceLock/LazyLockとCell/RefCell Optionを照合。instanceの後付けは既存項目またはSM-U901/SM-A901へ、static/cacheは理由を記録。
- 7: event counterとgeneration/統計/深さを区別し、eventはTrigger、識別はGenerationの既存項目に対応。
- 8: S-9の除外はcache/counter/mailbox/統計の役割と更新箇所を記録。SM-C27はMailbox、SM-K918はSentとして明示する。
- 9: P-1は調査unitなのでコードの解消は未実装。追加の解消後は13.2/18.13に沿う型、unitを全件に指定。
- 10: 付録にない34項目を追記。公開PieStateは主担当判断を記録し、SM-C27の現契約との不一致を主担当決定で訂正。

P-1は文書のみの変更で、runtime/kitの実装を変更せず、新しいE2Eは追加しない。P-0 checkerの実装テスト28件はP-0担当が実施済み。本unitの確認は検索・report・clippyと文書の整合検査である。PROGRESS.mdへの追加件数/完了記録は主担当の所有範囲で行う。

### spot review による記述の訂正

- SM-U903: slots は全行、live_slots はその live な部分集合であり、正常な行は両方に入る。問題は二重所属ではなく、この重複した索引と mounted/height/animation の phase を複数 effect の更新順で一致させる構造。単一の行表と明示 phase の案は維持する。
- SM-K919: cached は候補 options ではなく、検索から消えた選択済み item の label を保持する cache。211〜224行の options は resource.state の Ready を直接読む memo。S-9 の LabelCache<K> が検索完了 event で既存と同じ保持規則を更新し、labels は source+cache の memo、options は resource 状態の直接 memo を維持する。現在の検索結果を別 controller に複製せず、callback/表示契約を変えない。
- SM-U905: progress の Running に開始時刻を追加しない。既存と同じ clock の絶対時刻から位相を計算し、再開/period変更で開始時刻相対へ変えない。停止中の扱いとbuttonの待ちの契約も保存する。
- 文書の正確化だけで、追加34件/総203件、検査結果と分類件数は変えない。指示どおりchecker/clippyを再実行しない。

## Orchestrator follow-up decisions

### W1 definition reconciliation

- V-2: `Semantics<R>` includes the semantic properties currently split across `NodeSemantics`, both property structs, and the protocol node. The transport node retains topology/geometry plus one `semantics` value. Existing wire optional booleans take precedence; protocol-only `protected` remains a bool.
- V-2 consumer reconciliation: engine also stores `Semantics<FocusTarget>`, since existing relations may identify a draw region. The illustrative `Semantics<NodeId>` would lose that information. Map the node/region identity together to `SemanticsId` at accessibility publication.
- V-3: `EmbedOptions` is a pure value (`size`, `scale`, `prefer_gpu`, `gpu_preference`). Resize callbacks and the existing panic slot belong to shell-owned `ChildHooks`, passed explicitly beside options through embedding entry points and `Backend::open_child`. Preserve callback and panic forwarding behavior. This corrects the old options' hidden closure/state boundary rather than adding types-to-shell dependencies.
- V-3: `FailureKind` adopts existing Arvane `ErrorKind` categories Content/Resource/Backend/Surface/Unavailable; no new arbitrary classification.
- V-3 failure consolidation: Arvane's old `Error { kind, message }` has exactly the new shared failure's meaning, so remove it and reexport `Failure` directly. A newtype around `Failure` would preserve the duplication without adding a boundary contract. Rendering/text adapter errors are classified into `Failure`; core's user diagnostics retain their separate source-location/diagnostic meaning.
- V-3/E-2 fonts: W1 introduces the font-generation field without changing the existing font-registration effect/event order. After E-1 relocates the font collection owner, E-2 must advance its generation on successful registration and publish the updated environment; a later native environment refresh must not reset that generation. This follow-up is explicitly added to E-2's work order.
- V-3 layout diagnostics: keep `LayoutViolation` as a diagnostic enrichment (creation location, text, role, name, test id) around one canonical `LayoutFinding`; do not copy the rule, amount, or geometry into parallel fields. Reexport shared `LayoutRule`, `LayoutLimit`, and `LayoutCheck`, retaining the ignore-test-id builder on the shared check.
- V-4: create helper definitions alongside the other definitions, then apply the cross-module f64 conversion after those merges and before grouped consumer migration. This avoids overlapping changes to the same types without changing the wave's final scope.
- V-4 window validation: retain the existing huge-size rejection (`1e300`) after removing implicit f32 narrowing. App and resize requests share a positive/finite/u32-extent predicate in the environment vocabulary; native backends still validate the scaled physical size. Do not delete the existing invalid-size fixture merely because its value is now finite in f64.
- V-4: `Generation` is a copyable opaque u64 token, starts at zero, and `next(&mut self) -> Self` checks overflow; `is_current(&self, token: Self)` compares tokens. `Sent::update` retains the appendix's `Option<&T>` contract. Detached-thread tracking remains a permanent process fact, matching all four existing sticky atomic flags.
- V-5: preserve unavailable pressure as `Option<f64>` and retain `is_primary`; the illustrative 5.4 struct omitted both distinctions. `Routed<T>` carries `input`, `target: Option<NodeId>`, `region: Option<RegionId>`, `local`, `captured`, and `activation`. A missing target preserves window-directed keyboard input. Pointer/wheel producers supply a target. Route facts are stored once. `Activation` wraps the existing adopted-activation boolean until C-4.
- V-5 IME: selection ranges are UTF-8 bytes relative to the last surrounding-text snapshot. Native conversion uses its existing snapshot to translate UTF-16 units. The engine retains the existing excerpt byte origin and adds it when applying a range to its full editor; custom sessions already supply their complete text. No redundant full-document byte offset is added to the native snapshot.
- V-6: computed text resolves inheritance while preserving relative length/current-color meanings. Keep `TextLen`, `LineHeight`, and `TextDecoration` values and resolve pixels at use; semantic absence (unlimited lines, absent caret override) may remain optional. A parent 10px font with 0.1em spacing and a child 20px font must still yield 2px spacing. Do not add a second raw-style snapshot.
- V-6 drawing: `DrawTextStyle` remains a drawing-options value because token colors and baseline anchors have meaning beyond a pure declared text style. Convert it once to canonical text-run values. Existing draw frontdoor names are retained until N-1 as the work order requires. `RgbaImage` and authored `Pixels` both reexport the canonical Arc-backed pixel type; remove their separate structs.

| Date | Unit | Question / derivation | Decision and reason |
|---|---|---|---|
| 2026-11-04 | V-1a | Shared accessibility node/run numeric identity space | Define TextRunId in types::ids and reexport it from types/wire; retain node/run collision rejection with explicit SemanticsId(run.id.0) conversions at validation/error boundaries. |
| 2026-11-04 | V-1b | BoxData.measure contains LeafMeasure, an Rc closure | Put BoxData in engine::contract, not types. This follows the closure boundary and avoids types-to-engine dependency cycles; correct the design/work order. |
| 2026-11-04 | V-1b | Grid repetition and animation repetition are distinct Repeat enums | Rename grid repetition to layout::GridRepeat; retain motion::Repeat and update all consumers without compatibility aliases. |
| 2026-11-04 | CP0 | Trace calls would introduce client-to-diagnostics module dependencies | Keep debug tracing implementation under client::trace; diagnostics documents the public diagnostic workflow without adding a dependency exception. |
| 2026-11-04 | CP0 / F-2 | Configuring disallowed_types enables default warnings before per-crate adoption | Use workspace allow during migration, overridden by each converted crate's explicit deny; F-2 replaces it with workspace deny. Keep P-1 report warnings explicit with -W. |
| 2026-11-04 | V-2 | SemanticsText is both declared and computed, contains float geometry, and cannot support Eq without changing existing equality | Derive PartialEq/Hash, not Eq, for Semantics<R>. Add value Hash to existing text snapshot types with canonical signed-zero hashes; preserve their PartialEq and NaN behavior. Finite::new canonicalizes signed zero. No second text property/transport. |
| 2026-11-04 | V-1b | Style recorder also consumes TextShaper, so engine-only placement would add a forbidden style-to-engine dependency | Keep one backend-neutral pure-query trait in types::text_shaping, depending only on shared values/std; LeafTextMeasurer and closure-owning LeafMeasure remain engine contracts. No duplicate trait or compatibility reexport. |
| 2026-11-04 | CP0 trace | Buffered tracing preserves 781/782 completed reference files byte-for-byte; remaining DX12 file has one extra empty GPU poll | Empty poll counts 4→5; after excluding only fully empty turns and normalizing their turn-number offset, all remaining bytes/order match. Keep raw logs and apply this narrowly documented comparison rule. |
