# 付属書 E：engine・layout・editing（kivact-engine、kivact-layout、kivact-editing）

版 1.0（2026-11-18）。本書は `REWORK_DESIGN.md`（以下「設計書」）の 6.2 節・7.3 節と監査報告 `rework/audit/engine.md`（EN-01〜EN-32）を、実装の unit に分けたものである。

## 1. 範囲と前提

### 1.1 範囲

| 所見 | 判定（設計書 7.3） | unit |
|---|---|---|
| EN-01 二度・三度クリックの選択が drag で崩れる | 採用 | E20 |
| EN-02 時刻の基準が三つ、一時停止を開始時刻のずらしで直す | 採用（Q2：App 全体の clock 一つ） | E01 |
| EN-03 毎 frame 全 node を lower する | 採用（Q4：draw cache は描画中の node だけ） | E12 |
| EN-04 paint list の全再構築が多すぎる | 採用 | E06（`set_scene` の分）、E13 |
| EN-05 focus 中の scene 書込みで Tab 順を全部作る | 採用 | E06 |
| EN-06 overlay・Contents・fixed で facts を全部作り直す | 採用 | E07 |
| EN-07 毎 frame 全 text input を caret へ scroll | 採用 | E10 |
| EN-08 layout pass ごとに全 observed を報告 | 採用 | E27 |
| EN-09 hit test が子を全部回る | 採用 | E24 |
| EN-10 a11y の Metadata で順序を何度も数える | 変更して採用（S の部分だけ） | E16 |
| EN-11 Tab 順が O(N) | 採用（Tab 押下の一覧は残す。4.7 の D-2） | E06 |
| EN-12 折返しの editor が毎回全行を作り直す、damage の持ち主が三つ | 採用（折返し行の splice は後続。4.7 の D-3） | E09 |
| EN-13 `ImeTarget` が派生値の保存 | 採用 | E05 |
| EN-14 `EditOutcome::changed`、`max_length` を状態に保存 | 採用 | E17（W5a の鎖） |
| EN-15 window の表が二つ、close を phase に写す | 採用 | E15 |
| EN-16 `DeferredRequest` が `HostRequest` の写し | 採用 | E19 |
| EN-17 motion run の構造体が三つ | 採用 | E02 |
| EN-18 書込み統計の HashSet、a11y の統計が 0 | 採用 | E22 |
| EN-19 frame log が二つ、launch の重複 | 採用 | E19 |
| EN-20 `KeyResult::Text` | 採用 | E05 |
| EN-21 単語・行の選択が engine の方針 | 採用 | E20 |
| EN-22 `LeafMeasure` が二つ | 採用 | E05 |
| EN-23 node あたり約 1.5 KB | 採用（`unrounded` は残す。4.7 の D-4） | E11、E18（W5a の鎖）、E26、E28 |
| EN-24 祖先の走査が毎段 hash | 採用 | E04 |
| EN-25 slot の列の持ち主が五つ | 採用 | E03 |
| EN-26 shell の入力状態の写し | 変更して採用（`PointerSource` の写しだけ消す。`Admission` は engine に残す） | E21 |
| EN-27 sync の重複 | 採用 | E25 |
| EN-28 試験だけの code が release にある | 採用 | E23、E29 |
| EN-29 状態の印の雑音と実装一つの trait | 採用 | E11、E22、E24 |
| EN-30 host の段階が二つの enum | 採用（実装しない。設計書 7.3 節と本付属書 4.7 の D-1） | — |
| EN-31 上下移動に goal column が無い | 採用 | E17（W5a の鎖） |
| EN-32 host だけの editing の型が公開 | 採用 | E09 |
| 付属書 C 向け：tree の変更記録 `Reader::take_tree_changes`（E-REQ-5、window 付き） | 設計書 7.2 Q3 | E08 |
| 付属書 C 向け：入力の出所 `InputOrigin`・`Routed::origin`（E-REQ-6） | RC-28 | E30 |
| 付属書 C 向け：InputValue の echo の抑止・`LayoutReport`・`NodeFlags`・`font_generation`（E-REQ-1〜4） | RC-20 | E31 |
| 付属書 A 向け：layer の `content_version`（AR-03）の engine 側 | 設計書 6.2 PERF-7 | E14 |
| EA-01 検証の描き直しと内容の変化を分ける | 採用（設計書 12 章） | E12（照合を別の出力に）、E14（持ち越しの規則と試験） |
| EA-02 scroll の部分 rewalk が transform・layer を増やす | 採用 | E13（`compact`）、E32（試験） |
| EA-03 部分 rewalk で外れた node の draw cache が残る | 採用 | E13（`forget_left_draws`）、E32（試験） |
| EA-11・EA-12・EA-16 `EditContext` の全文の走査・3 行の窓・cluster の境界 | 採用 | E17（W5a の鎖） |
| EA-13 self_check の参照の hit walk が索引を使う | 採用 | E24 |
| EA-14 照合が cache の paint を使う | 採用 | E12（`verify_draw`）、E32（試験） |

### 1.2 前提（他の付属書が先に済ませたこと）

- W1（付属書 T）：xtask の `state-fields` 検査は新しい `state` 検査に置き換わり、engine・layout・editing の `// xtask-allow(state-fields): ...` の行は消えている。本書の unit が足す状態の型には、本書が書く `// xtask-allow(state): ...` の一行を struct の直前に置く。`e2e` の test binary は `e2e_core`・`e2e_ui`・`e2e_kits`・`e2e_native` に分かれている。
- W2（付属書 S、lane A）：`kivact_types::render::*` は `arvane_protocol::*` になり、engine の `use` 行は書き換え済み。本書のコードで `RenderCmd`、`Composition`、`CompositeLayer`、`LayerKey`、`Paint`、`ShapedText`、`FrameStats` と書く型は、そのファイルの `use` 行が今指している path で使う（例：`rg -n "RenderCmd" crates/kivact-engine/src/lower/scene_out.rs | head -3`）。`prop_interp`（`PropInterp`、`PropBasis`、`apply_components`、`prop_interp`）は engine の中へ移っている。path は `rg -n "struct PropInterp" crates/kivact-engine/src` で確かめ、`src/model/state_motion/*.rs` が今使っている `use` 行をそのまま使う。
- W3a（付属書 S の S10）：`crates/kivact-types/src/lib.rs` の S10 の変更は済んでいる。E01 はその後の `lib.rs` に 1 行足す。
- W5b の鎖（付属書 A の A09）：`CompositeLayer::content_version` があり、engine の `into_layer`（`src/lower/scene_out.rs`）と `append_overlay_layer`（`src/host/devtools.rs`）は `src/lower/content_versions.rs` の `fresh_content_version()` を書いている（4.3 節）。E14 はその後の W6b。
- W6a（付属書 S の S13・S14）：S の鎖が engine・layout・editing の型と `use` を書き換えている。W6b 以降の本書の unit は、その後の名前と path で書く（手順の型名・関数名が見つからないときは `rg` で今の名前を探す。意味は変わっていない）。
- 監査報告の行番号は 2b8df5ac のもの。本書は関数名と、その関数の中の目印になる行で場所を示す。

### 1.3 wave・lane の使い方

- 配置は作業指示書 4 章の wave の表（SCHEDULE）のとおり：W3b（E1：E01 → E02、E2：E03 → E04）、W4（E1：E05 → E06 → E07 → E08 → E09 → E30、E2：E10 → E11）、W4b（lane E：E31）、W5a（鎖：… → C18 → E15 → E17 → E18 → E20、E1：E16、E2：E12 → E13）、W6b（E1：E19 → E21、E2：E14 → E32 → E24）、W7a（E1：E22 → E23 → E25 → E26）、W7b（E1：E27 → E28）、W8（E1：E29）。
- lane は wave ごとに、表の順に unit を一つずつ実行する（同じ lane の中は直列）。同じ wave の二つの lane は一つもファイルを共有しない（5.3 節）。
- W5a の鎖（1 lane）に置く unit は E15・E17・E18・E20。付属書 R と C の鎖の unit の後に、E15 → E17 → E18 → E20 の順で走らせる（E15 を E1 から鎖へ移したのは E17 の EA-11 の設計のため：E17 は E15 の後の `host/events.rs` と `host/testing/self_check.rs` を直す）。A の鎖（A05〜A10）は W5b で、本書の W5a の unit と同時に走らない。
- 本書の「`src/...`」は `crates/kivact-engine/src/...` を指す。

### 1.4 試験の実行の約束（全 unit 共通）

- コマンドはすべて worktree の root で実行する（PLAN-04）。手順の中で別の directory に入る所は `ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/..." || exit 1` で入り、`cd "$ROOT" || exit 1` で戻る。
- アプリを起動する試験は `dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test ...` の形で実行する。window-lock は一つのコマンドに一度だけ（作業指示書 0.6、PLAN-01・INC-15・INC-43）。
- E2E の module と test binary の対応は付属書 T の T15 の手順 7 の三つの列（と `e2e_kits` へ移す四つ）のとおり。本書の確認が使う module はすべて次の表にある（`e2e_core` の行は T15 の一つ目の列そのもの。PLAN-07）。
  | binary（package） | 本書が使う module |
  |---|---|
  | `e2e_core`（`kivact-public-api-tests`） | `a11y_incremental`、`app_handle`、`assets`、`async_tests`、`binding_update`、`borders_backgrounds`、`canvas`、`canvas_worker`、`close_request`、`color_spaces`、`component_macro`、`components`、`compositor`、`custom_text_field`、`diagnostics`、`disposal`、`drawables`、`effects_filters`、`environment`、`flows`、`focus`、`frame_sources`、`grid`、`hover`、`i18n`、`image_semantics`、`images_animated`、`input_platform`、`keyboard`、`keys_unhandled`、`landmark_roles`、`launch_fonts`、`launch_platform`、`launch_real_window`、`launch_support`、`launch_window`、`layout`、`layout_check`、`layout_scroll`、`layout_units`、`lifecycle`、`machine`、`motion`、`motion_easing`、`node_ref`、`offscreen`、`offscreen_svg`、`overlay`、`paint`、`parts`、`path_algebra`、`platform_paths`、`pointer`、`presence`、`presence_modes`、`primitive`、`relations`、`resource_reclaim`、`runtime`、`scroll_timeline`、`setup_functions`、`shaders`、`shared_layout`、`style_paint`、`svg_embedded_text`、`task`、`text_editing`、`text_effects`、`text_primitives`、`text_style`、`timeline`、`touch_scroll`、`transform`、`transform_3d`、`view`、`viewport`、`windows` |
  | `e2e_ui`（`kivact-public-api-tests`） | `apg_*`、`interaction_*`、`ui_*`（`ui_standard` を除く）、`metrics_and_description`、`reactive_props` |
  | `e2e_native`（`kivact-public-api-tests`） | `devtools`、`embed`、`foreign_window`、`gpu_on_demand` |
  | `e2e_kits`（`kivact-kit-tests`） | `ui_standard`、`theme_conformance` |
- 各 unit の「確認」の `E2E: a, b, c` は、列挙した module 名 `<m>` を一つずつ、上の表の binary の一つのコマンドで実行することを指す。一つのコマンドに二つの binary の module を混ぜない。
  ```
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_core -- <m>::
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_ui -- <m>::
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- <m>::
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-kit-tests --test e2e_kits -- <m>::
  ```
- **0 件は失敗**：filter が一つの試験も実行しなかったコマンド（cargo の出力が `running 0 tests`）は、その unit の確認の失敗とする。別の binary で探し直さない（module が表の binary に無ければ STOP）。engine の試験（`--test content_version` など）と `-p kivact-editing --lib -- <filter>` も同じ。

### 1.5 共通の確認（tier a）

各 unit の「確認」の `ENGINE-A` は次の列を指す。

```
cargo fmt -p kivact-engine
cargo check -p kivact-engine --all-targets
cargo check -p kivact-engine --all-targets --features testing
cargo clippy -p kivact-engine --all-targets --features testing -- -D warnings
cargo check -p kivact-core --all-targets
cargo xtask check --no-write --crate kivact-engine
dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-engine --features testing
```

`kivact-layout` を変える unit は次の `LAYOUT-A` も実行する。

```
cargo fmt -p kivact-layout
cargo clippy -p kivact-layout --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-layout
cargo test -p kivact-layout
```

`kivact-editing` を変える unit は次の `EDITING-A` も実行する。

```
cargo fmt -p kivact-editing
cargo clippy -p kivact-editing --all-targets -- -D warnings
cargo xtask check --no-write --crate kivact-editing
cargo test -p kivact-editing
```

### 1.6 他の付属書へ渡すもの

- 付属書 C：E-REQ-5 の `Reader::record_tree_changes(on)` と `Reader::take_tree_changes() -> TreeChanges`（window 付き。E08、W4）、E-REQ-6 の `InputOrigin` と `Routed::origin`（E30、W4）、E-REQ-1〜4（E31、W4b）。C06 の前提は E08・E30、C07 の前提は E31（4.2 節）。
- 付属書 A：layer ごとの `content_version` を engine が正しく上げること（E14、W6b。A09 の `fresh_content_version()` を置き換える）。field は付属書 A の A09（W5b）が足す（4.3 節）。
- 付属書 R・C・U・K：`kivact::custom` の editing の API の新しい形（E17・E20、W5a の鎖）。
- 付属書 G：architecture.md に書く engine の規則（4.6 節）。

## 2. unit の一覧

| ID | wave | lane | 前提の unit | 所有するファイル・directory（完全な一覧は各 unit の「所有」） | 所見 | 規模 |
|---|---|---|---|---|---|---|
| E01 | W3b | E1 | — (W3a の S10) | `crates/kivact-types/src/clock.rs`（新設）、`crates/kivact-types/src/lib.rs`（1 行）、`src/model/frame/*`、`src/model/app_model.rs`、`src/model.rs`、`src/host/{pump,events,window_changes}.rs`、motion の 4 directory | EN-02 | M |
| E02 | W3b | E1 | E01 | `src/model/motion_run.rs`（新設）、`src/model/{state_motion,presence,layout_motion}/*` | EN-17 | M |
| E03 | W3b | E2 | — | `src/model/{aggregate,lifecycle}.rs` と `editors`・`measure`・`paint.ink`・`paint.look` を読む約 35 ファイル、`crates/kivact-layout/src/{borrowed_tree,layout,columns,compare}.rs`、`src/tree/{store,layout_adapter}.rs` | EN-25 | L（機械的） |
| E04 | W3b | E2 | E03 | `src/tree/node_tree.rs`、`src/paint/list/walk.rs`、`src/model/interact/{geometry,sticky,sync}.rs`、`src/model/aggregate.rs` | EN-24 | M |
| E05 | W4 | E1 | — | `src/model/edit/{state,session,ime}.rs`、`src/model/focus/navigation.rs`、`src/model/keys.rs`、`src/host/events.rs`、`src/host/write/reader.rs`、`src/measure/state.rs` | EN-13、EN-20、EN-22 | S |
| E06 | W4 | E1 | E05 | `src/model/focus/navigation.rs`、`src/model/apply.rs`、`src/paint/state.rs` | EN-05、EN-11、EN-04（一部） | M |
| E07 | W4 | E1 | E06 | `src/model/apply.rs`、`src/tree/node_tree.rs` | EN-06 | S |
| E08 | W4 | E1 | E07 | `src/contract/devtools.rs`、`src/contract.rs`、`src/model/events.rs`、`src/model/lifecycle.rs`、`src/host/write/{reader,writer,typed}.rs` | 付属書 C | S |
| E09 | W4 | E1 | E08 | `crates/kivact-editing/src/*`、`src/text_input/*`、`src/text_input.rs`、`src/measure/{editor,input}.rs`、`src/model/keys.rs`、`src/model/edit/inputs.rs` | EN-12、EN-32 | M |
| E30 | W4 | E1 | E09 | `src/contract/routed.rs`、`src/contract.rs`、`src/model/frame/state.rs`、`src/model/app_model.rs`、`src/model/keys.rs`、`src/model/pointer/{machine,shell}.rs`、`src/host/events.rs`、`src/host/write/writer.rs`、`crates/kivact-test/src/screen/host_sync_tests.rs` | 付属書 C の E-REQ-6（RC-28） | S |
| E10 | W4 | E2 | — | `src/model/changes.rs`、`src/model/interact/{sync,text_geometry,caret,geometry}.rs`、`src/model/aggregate.rs`、`src/host/pump.rs` | EN-07 | S |
| E11 | W4 | E2 | E10 | `src/layer.rs`、`src/paint/list/{look,walk}.rs`、`src/model/interact/{hit,geometry}.rs` | EN-23（Look）、EN-29（Look・SubtreeFacts） | S |
| E31 | W4b | E | E30 | `src/host/write/{reader,writer,typed}.rs`、`src/contract.rs`、`src/contract/events.rs`、`src/model/app_model.rs`、`src/model/semantics.rs`、`src/host/pump.rs`、`crates/kivact-core/src/launch/runtime.rs`（1 行）、`crates/kivact-core/src/client/dispatch/delivery.rs`（1 腕） | 付属書 C の E-REQ-1〜4（RC-20） | M |
| E12 | W5a | E2 | — | `src/paint/{state,frame_scene,canvas,snapshot}.rs`、`src/paint/list/invalidation.rs`、`src/lower/scene_out.rs`、`src/model/{aggregate,states,scrollbar,transform}.rs`、`src/model/interact/{sync,visual,geometry,hit}.rs`、`src/model/pointer/hover.rs`、`src/model/frame/ticks.rs` | EN-03、EA-01・EA-14（照合） | L |
| E13 | W5a | E2 | E12 | `src/model/interact/sync.rs`、`src/model/changes.rs`、`src/model/frame/ticks.rs`、`src/model/state_motion/*`、`src/model/layout_motion/steps.rs`、`src/paint/list/{walk,invalidation}.rs`、`src/paint/{state,frame_scene}.rs` | EN-04、EA-02、EA-03 | L |
| E14 | W6b | E2 | E13、A09（W5b） | `src/lower/scene_out.rs`、`src/lower/content_versions.rs`、`src/host/testing/test_host.rs`、`crates/kivact-engine/{Cargo.toml,tests/content_version.rs}`、`src/paint/{frame_scene,state,canvas,snapshot}.rs`、`src/paint/list/walk.rs`（`paint_list`） | AR-03（engine 側）、EA-01 | S |
| E32 | W6b | E2 | E14 | `src/paint/state.rs`（thread-local と試験用の method）、`src/paint/list/invalidation.rs`（3 行）、`src/host/testing/test_host.rs`（method 二つ）、`src/host/testing.rs`（1 行）、`crates/kivact-engine/{Cargo.toml,tests/draw_cache.rs}` | EA-02・EA-03・EA-14 の試験（設計書 12 章） | S |
| E15 | W5a | 鎖 | R・C の鎖 | `src/host/*`（`window.rs` は `src/model/window.rs` へ移す）、`src/model/{app_model.rs}`、`src/model.rs`、`src/host.rs` | EN-15 | M |
| E16 | W5a | E1 | — | `src/a11y/build/{dirty,identity,node}.rs` | EN-10（S） | S |
| E17 | W5a | 鎖 | E15（R・C の鎖） | `crates/kivact-editing/{src/*,tests/geometry.rs}`、`src/model/{keys,apply}.rs`、`src/model/edit/*`、`src/model/pointer/machine.rs`、`src/text_input/shaping.rs`、`src/text_input.rs`、`src/measure/{editor,input}.rs`、`src/host/events.rs`（2 行）、`src/host/testing/self_check.rs`（2 行）、core・interaction・ui・ui-soft の editing の呼出し元、`dev/tests/kivact-public-api/tests/e2e_core/{text_editing,custom_text_field}.rs` | EN-14、EN-31、監査 4 章、EA-11、EA-12、EA-16 | M |
| E18 | W5a | 鎖 | E17 | `src/contract/{node.rs,contract.rs}`、`src/host/write/typed.rs`、`src/tree/node.rs`、`src/model/apply.rs`、`src/model/pointer/shell.rs`、`crates/kivact-core/src/view/tree.rs`、`crates/kivact-test/src/facts.rs` | EN-23（NodeInput） | S |
| E19 | W6b | E1 | — | `src/model/{aggregate,app_model}.rs`、`src/model.rs`、`src/host/write/writer.rs`、`src/host/{pump,launch}.rs`、`src/embed.rs`、`src/frame_log.rs`、`src/lib.rs` | EN-16、EN-19 | S |
| E20 | W5a | 鎖 | E18（E17） | `crates/kivact-editing/src/edit_state.rs`、`src/model/pointer/{machine,state,touch}.rs`、`src/text_input/word.rs`（削除）、`src/text_input.rs`、`src/model/edit/ime.rs`、`dev/tests/kivact-public-api/tests/e2e_core/text_editing.rs` | EN-01、EN-21 | S |
| E21 | W6b | E1 | E19 | `src/model/pointer/*` | EN-26（変更） | M |
| E22 | W7a | E1 | — | `src/mutation.rs`、`src/host/write.rs`、`src/host/write/{writer,reader,typed}.rs`、`src/a11y/{state,publish}.rs`、`src/a11y/build/{dirty,payload}.rs`、`src/model/{apply,states,changes,model.rs}`、`src/model/edit/inputs.rs`、`src/model/presence/transitions.rs`、`src/lib.rs` | EN-18、EN-29（Mutation） | M |
| E23 | W7a | E1 | E22 | `src/model/window.rs`、`src/host/foreign.rs`、`crates/kivact-layout/{Cargo.toml,src/*}`、`crates/kivact-engine/Cargo.toml`、`src/text_input/refresh.rs` | EN-28（devtools 以外） | S |
| E24 | W6b | E2 | E32 | `src/model/interact/hit.rs`（`src/paint/list/hit.rs` へ移す）、`src/paint/list/{hit_reach.rs（削除）,facts.rs（新設）}`、`src/paint.rs`、`src/paint/list.rs`、`src/paint/state.rs`、`src/paint/list/walk.rs`、`src/layer.rs`、`src/model/interact.rs`、`src/model/interact/geometry.rs`、`src/model/drawables/hit.rs`、`src/model/drawables.rs`、`src/lower/shape.rs`、`src/host/testing/test_host.rs`（method 一つ）、`crates/kivact-engine/{Cargo.toml,tests/hit_reference.rs}` | EN-09、EN-29（LayerView・HitInk）、EA-13 | M |
| E25 | W7a | E1 | E23 | `src/model/interact/sync.rs`、`src/model/changes.rs`、`src/tree/node_tree.rs`、`src/model/keys.rs`、`src/model/edit/ime.rs`、`src/model/pointer/shell.rs`、`src/model/apply.rs`、`src/paint/state.rs`、`src/lower/assets.rs` | EN-27 | S |
| E26 | W7a | E1 | E25 | `src/tree/{node,node_tree}.rs` と `scroll`・`shift` を読む約 20 ファイル | EN-23（HostNode） | M（機械的） |
| E27 | W7b | E1 | — | `src/model/changes.rs`、`src/model/interact/{geometry,sync}.rs`、`src/model/{states,apply}.rs`、`src/model/focus/navigation.rs`、`src/model/state_motion/steps.rs`、`src/model/pointer/machine.rs`、`src/model/drawables/request.rs`、`src/check/layout_reference.rs` | EN-08 | M |
| E28 | W7b | E1 | E27 | `crates/kivact-layout/src/{columns,tree,compare}.rs` | EN-23（layout） | S |
| E29 | W8 | E1 | — | `crates/kivact-engine/Cargo.toml`、`crates/kivact-core/Cargo.toml`（`devtools` の行だけ）、`src/{lib,check}.rs`、`src/host/{devtools,pump,window_changes,events}.rs`、`src/host/write/{reader,writer}.rs` | EN-28（devtools） | M |

## 3. unit の手順

## E01：App 全体の motion の時計（`kivact_types::clock::Clock`）

- **wave / lane / 前提**：W3b / E1 / なし（W3a の S10 の後の `kivact-types/src/lib.rs` に 1 行足す）
- **所有**：`crates/kivact-types/src/clock.rs`（新設）、`crates/kivact-types/src/lib.rs`（`pub mod clock;` の 1 行だけ）、`src/model/frame.rs`、`src/model/frame/state.rs`、`src/model/frame/ticks.rs`、`src/model/frame/clocks.rs`、`src/model/app_model.rs`、`src/model.rs`、`src/model/state_motion.rs`、`src/model/state_motion/steps.rs`、`src/model/state_motion/state.rs`（doc だけ）、`src/model/presence/size.rs`、`src/model/layout_motion/steps.rs`、`src/model/layout_motion/state.rs`（doc だけ）、`src/model/playback/tick.rs`、`src/model/playback/state.rs`（doc だけ）、`src/host/pump.rs`、`src/host/events.rs`、`src/host/window_changes.rs`
- **目的**：時刻の基準を「host の ns」と「App の motion の ns」の二つにし、motion の ns は App に一つの `Clock`（一時停止と DevTools の速さを持つ）だけが作る。開始時刻を後からずらす修理（`shift_*_times`）を消す（EN-02、設計書 7.3 Q2）。
- **手順**：
  1. `crates/kivact-types/src/clock.rs` を新しく作り、次をそのまま書く。
     ```rust
     //! Role: The App's motion time: host time less the stops, run at the DevTools' rate.
     //!
     //! Invariants:
     //! - Motion time stands still while the clock is stopped (no window can
     //!   draw) and goes on from where it stood when it resumes: the stopped
     //!   host time is summed in `stopped_ns`, never added to the start of a
     //!   run afterwards.
     //! - Motion time passes `scale` times as fast as running host time; a new
     //!   scale continues from the motion time of the moment it is set.
     //! - Motion time never decreases.

     use crate::motion::{MotionClock, TimeScale};

     /// Whether the App's motion time runs.
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum ClockPhase {
         /// It runs; `resumed_ns` (host ns) is when the last stop ended (0: none yet).
         Running {
             /// Host time (ns) the last stop ended at.
             resumed_ns: u64,
         },
         /// It stands still since `since_ns` (host ns): no window can draw.
         Stopped {
             /// Host time (ns) the stop began at.
             since_ns: u64,
             /// Host time (ns) the stop before it ended at.
             resumed_ns: u64,
         },
     }

     /// The App's one motion clock: host time (ns) less the time no window
     /// could draw, scaled by the DevTools' time scale.
     #[derive(Clone, Copy, Debug, PartialEq)]
     pub struct Clock {
         /// The DevTools' rate, anchored in running time (`running_ns`).
         scale: TimeScale,
         phase: ClockPhase,
         /// The host time (ns) every ended stop stood still, summed.
         stopped_ns: u64,
     }

     impl Clock {
         /// A running clock at rate 1 that never stopped.
         pub const fn new() -> Self {
             Clock {
                 scale: TimeScale {
                     anchor_host_ns: 0,
                     anchor_motion_ns: 0,
                     scale: 1.0,
                 },
                 phase: ClockPhase::Running { resumed_ns: 0 },
                 stopped_ns: 0,
             }
         }

         /// Whether it runs or stands still, and since when.
         pub fn phase(&self) -> ClockPhase {
             self.phase
         }

         /// The host time (ns) that ran up to `host_ns`: the stops left out.
         pub fn running_ns(&self, host_ns: u64) -> u64 {
             let at = match self.phase {
                 ClockPhase::Running { .. } => host_ns,
                 ClockPhase::Stopped { since_ns, .. } => since_ns.min(host_ns),
             };
             at.saturating_sub(self.stopped_ns)
         }

         /// The motion time (ns) at host time `host_ns`.
         pub fn motion_ns(&self, host_ns: u64) -> u64 {
             self.scale.motion_ns(self.running_ns(host_ns))
         }

         /// Motion ns per host ns now: the scale while it runs, 0 while it stands still.
         pub fn rate(&self) -> f64 {
             match self.phase {
                 ClockPhase::Running { .. } => self.scale.scale,
                 ClockPhase::Stopped { .. } => 0.0,
             }
         }

         /// Run at `scale` from host time `host_ns` on (the DevTools' time scale).
         pub fn set_scale(&mut self, host_ns: u64, scale: f64) {
             self.scale = self.scale.rescaled(self.running_ns(host_ns), scale);
         }

         /// Stand still from host time `host_ns` (nothing when already stopped).
         pub fn stop(&mut self, host_ns: u64) {
             if let ClockPhase::Running { resumed_ns } = self.phase {
                 self.phase = ClockPhase::Stopped {
                     since_ns: host_ns,
                     resumed_ns,
                 };
             }
         }

         /// Run again from host time `host_ns` (nothing when running).
         pub fn resume(&mut self, host_ns: u64) {
             if let ClockPhase::Stopped { since_ns, .. } = self.phase {
                 self.stopped_ns = self
                     .stopped_ns
                     .saturating_add(host_ns.saturating_sub(since_ns));
                 self.phase = ClockPhase::Running { resumed_ns: host_ns };
             }
         }

         /// The snapshot a `FrameTick` carries to the client.
         pub fn motion_clock(&self) -> MotionClock {
             let resumed_ns = match self.phase {
                 ClockPhase::Running { resumed_ns } | ClockPhase::Stopped { resumed_ns, .. } => {
                     resumed_ns
                 }
             };
             MotionClock {
                 offset_ns: self.stopped_ns,
                 resumed_ns,
             }
         }
     }

     impl Default for Clock {
         fn default() -> Self {
             Self::new()
         }
     }
     ```
  2. `crates/kivact-types/src/lib.rs`：`pub mod check;` の次の行に `pub mod clock;` を足す。root の `pub use` には足さない（`kivact_types::clock::Clock` で使う）。
  3. `src/model/frame/state.rs`：
     - `enum TickHistory` と `enum Clock` を消す。`use` 行から `TimeScale` を消す。
     - 次の型を足す（`FrameRequests` の後）。
       ```rust
       /// The instant of the event, turn or tick being handled, in both time bases.
       #[derive(Clone, Copy, Debug, Default, PartialEq)]
       pub(crate) struct Now {
           /// The host clock (ns): frame deadlines and wakes are in it.
           pub(crate) host_ns: u64,
           /// The App's motion time (ns) at `host_ns`: every motion record starts and steps in it.
           pub(crate) motion_ns: u64,
           /// Motion ns per host ns at `host_ns` (0 while the App clock stands still).
           pub(crate) rate: f64,
       }

       impl Now {
           /// The instant `host_ns` read on the App clock `clock`.
           pub(crate) fn at(host_ns: u64, clock: &kivact_types::clock::Clock) -> Self {
               Now {
                   host_ns,
                   motion_ns: clock.motion_ns(host_ns),
                   rate: clock.rate(),
               }
           }
       }
       ```
     - `struct Frame` の field を次にする（`now: u64`、`clock: Clock`、`last_frame: TickHistory`、`time_scale: TimeScale` を置き換える。他の field と doc はそのまま）。struct の直前に印を一行置く。
       ```rust
       // xtask-allow(state): drawable is the drawability last reported to the client (start_env/update_env only); last_frame is the host stamp of the last delivered tick (tick only); independent, all four combinations occur
       pub(crate) struct Frame {
           /// The instant being handled (`Frame::set_now`).
           pub(super) now: Now,
           /// Whether the window can draw, as last reported to the client.
           pub(super) drawable: bool,
           pub(super) frames: FrameRequests,
           pub(super) smooth_scrolls: BTreeMap<NodeId, SmoothScroll>,
           /// The host stamp (ns) of the last delivered tick (`None`: none yet).
           pub(super) last_frame: Option<u64>,
           pub(super) env: Environment,
           pub(super) shader_clocks: BTreeMap<NodeId, ClockNeed>,
       }
       ```
       `Default` の中は `now: Now::default(), drawable: true, last_frame: None` にし、`clock` と `time_scale` の行を消す。
     - method を次に置き換える（`start_theme`、`theme_hint`、`period_ns`、`reduced_motion`、`stop_smooth_scrolls` はそのまま）。
       ```rust
       /// Schedule motion now: a frame at the current host time; returns the motion time a run starting now starts at.
       pub(crate) fn request_motion_frame(&mut self) -> u64 {
           self.frames.0.insert(self.now.host_ns);
           self.now.motion_ns
       }

       /// The instant of the event or turn being handled.
       pub(crate) fn set_now(&mut self, now: Now) {
           self.now = now;
       }

       /// The motion time (ns) of the event or turn being handled.
       pub(crate) fn now(&self) -> u64 {
           self.now.motion_ns
       }

       /// The host time (ns) of the event or turn being handled.
       pub(crate) fn host_now(&self) -> u64 {
           self.now.host_ns
       }

       /// The host time (ns) at which motion time reaches `motion_ns`, read
       /// at the current instant: now when it already has; a display period
       /// from now while motion time stands still.
       pub(crate) fn deadline_after(&self, motion_ns: u64) -> u64 {
           if motion_ns <= self.now.motion_ns {
               return self.now.host_ns;
           }
           if self.now.rate > 0.0 {
               let wait = (motion_ns - self.now.motion_ns) as f64 / self.now.rate;
               self.now.host_ns.saturating_add(wait.ceil() as u64)
           } else {
               self.now.host_ns.saturating_add(self.period_ns())
           }
       }

       /// Whether there is a surface to draw on.
       pub(crate) fn surface_active(&self) -> bool {
           self.drawable
       }

       /// Ask for a tick at or after the host time `at_ns`.
       pub(crate) fn request(&mut self, at_ns: u64) {
           self.frames.0.insert(at_ns);
       }
       ```
  4. `src/model/frame/ticks.rs`：
     - `use super::state::{Clock, FrameRequests, SmoothScroll, TickHistory};` を `use super::state::{FrameRequests, Now, SmoothScroll};` にし、`use kivact_types::clock::Clock;` を足す。
     - `fn set_time_scale` を消す。
     - `scroll_smoothly` の `let last = match self.frame.last_frame { ... };` を `let last = self.frame.last_frame.unwrap_or(0);` にする。
     - `start_scroll_fling`：`last_ns: self.frame.now,` を `last_ns: self.frame.now(),` に、`.insert(self.frame.now.saturating_add(self.frame.period_ns()))` を `.insert(self.frame.host_now().saturating_add(self.frame.period_ns()))` にする。
     - `start_env` の本体を次にする。
       ```rust
       self.frame.drawable = surface_active;
       self.frame.env = env;
       ```
     - `update_env`：`match (self.frame.clock, surface_active) { ... }` の式全体を `self.frame.drawable = surface_active;` の一行に置き換える。
     - `retarget_frame`：`self.retarget_states(self.frame.now, ...)` を `self.retarget_states(self.frame.now(), ...)` に、`self.frame.frames.0.insert(self.frame.now);` を `self.frame.frames.0.insert(self.frame.host_now());` にする。
     - `tick` の署名と本体を次にする（`if self.motion.is_active() { ... }` の塊は E13 が消すので、ここでは残す）。
       ```rust
       pub(crate) fn tick(
           &mut self,
           now: u64,
           clock: &Clock,
           client: Option<&mut FrameRequests>,
       ) -> Option<FrameStamp> {
           self.frame.set_now(Now::at(now, clock));
           let source = client.is_some();
           let due = self
               .frame_deadline(client.as_deref())
               .filter(|t| self.frame.surface_active() && *t <= now)?;
           let period = self.frame.period_ns();
           let stamp = if now - due < period / 4 { due } else { now };
           self.frame.frames.0.retain(|t| *t > now);
           if let Some(client) = client {
               client.0.retain(|t| *t > now);
           }
           // (the `if self.motion.is_active() { ... }` block, unchanged)
           let motion = clock.motion_ns(stamp);
           let reduced = self.frame.reduced_motion();
           let sizes = self.step_sizes(motion, reduced);
           let layout = self.step_layout_motion(motion, reduced);
           self.step_playback(motion);
           self.step_shader_clocks(stamp);
           if self.step_states(motion, reduced) || sizes || layout {
               self.frame.frames.0.insert(stamp.saturating_add(period));
           }
           self.sync();
           self.tick_scrolls(stamp, motion, period);
           self.frame.last_frame = Some(stamp);
           source.then_some(FrameStamp {
               now_ns: stamp,
               estimated_present_ns: stamp.saturating_add(period),
           })
       }
       ```
     - `tick_scrolls(&mut self, stamp: u64, period: u64)` を `tick_scrolls(&mut self, stamp: u64, motion: u64, period: u64)` にする。中で `began_ns: stamp` を `began_ns: motion` に、`(start, target, stamp, easing)` を `(start, target, motion, easing)` に、`stamp.saturating_sub(began)` を `motion.saturating_sub(began)` に、`self.tick_fling(*id, velocity, last_ns, stamp)` を `self.tick_fling(*id, velocity, last_ns, motion)` にする。frame の要求 `stamp.saturating_add(period)` はそのまま（host の時刻）。
     - `frame_deadline`：`let TickHistory::Ticked { last_ns: last } = self.frame.last_frame else { return Some(requested); };` を `let Some(last) = self.frame.last_frame else { return Some(requested); };` にする。
  5. `src/model/frame/clocks.rs`：`self.request_clock_frames(self.frame.now());` を `self.request_clock_frames(self.frame.host_now());` にする。
  6. `src/model/playback/tick.rs`：`refresh_playback` と `step_playback` の `self.frame.request(at);`（2 か所）を `self.frame.request(self.frame.deadline_after(at));` にする（`next_switch` は motion の時刻を返す）。
  7. `src/model/state_motion/steps.rs`：`pub(crate) fn shift_state_times` を消す。`src/model/state_motion.rs` の doc の最後の項目（`While nothing can be drawn ... (Model::shift_state_times) ...`）を `- While no window can draw, the App clock stands still (kivact_types::clock::Clock), so the transitions go on from where they stood.` にする。`state_motion/state.rs` の `TrackPhase::Running` の doc の `(host ns)` を `(motion ns)` にする。
  8. `src/model/presence/size.rs`：`pub(crate) fn shift_size_times` を消す。
  9. `src/model/layout_motion/steps.rs`：`pub(crate) fn shift_layout_times` を消す。`layout_motion/state.rs` の `/// When the run started (host ns).` を `(motion ns)` にする。`playback/state.rs` の `Playing { start }` の doc の `(host ns)` を `(motion ns)` にする。
  10. `src/model/app_model.rs`：
      - `enum AppClock` を消す。`use kivact_types::{ExternalImageId, MotionClock, ...}` から `MotionClock` を消す。
      - `struct AppModel` の `motion: MotionClock` と `clock: AppClock` の二つの field を次の一つにする：
        ```rust
        /// The App's one motion clock (`kivact_types::clock::Clock`): it stops while no window can draw; `FrameTick.clock` carries its snapshot.
        pub(crate) clock: kivact_types::clock::Clock,
        ```
        `AppModel::new` の `motion: MotionClock::default(), clock: AppClock::Running,` を `clock: kivact_types::clock::Clock::new(),` にする。
      - `pub(crate) fn motion(&self) -> MotionClock` を消す。
      - 次の method を足す。
        ```rust
        /// Window `id` handles an event or a turn at host time `host_ns`.
        pub(crate) fn set_now(&mut self, id: WindowId, host_ns: u64) {
            let now = super::frame::state::Now::at(host_ns, &self.clock);
            if let Some(model) = self.windows.get_mut(&id) {
                model.frame.set_now(now);
            }
        }
        ```
  11. `src/model.rs`：`pub(crate) use app_model::{AppClock, AppModel, WindowChange};` を `pub(crate) use app_model::{AppModel, WindowChange};` にする。`src/model/frame.rs` の doc の二つ目の項目（`Without a surface no tick is delivered ... Model::update_env.`）を次にする：`- Without a surface no tick is delivered. Motion time is the App's one clock (kivact_types::clock::Clock): it stands still while no window can draw, and every tick carries its snapshot (MotionClock). Frame requests and deadlines are host time; every motion record is motion time (frame::state::Now).`
  12. `src/host/window_changes.rs`：
      - `use crate::model::{AppClock, WindowChange};` を `use crate::model::WindowChange;` にする。
      - `finish_motion` の `match app.clock { ... }` の式全体を次にする。
        ```rust
        if drawable {
            app.clock.resume(now);
        } else {
            app.clock.stop(now);
        }
        ```
      - `start_window` の中の `model.frame.set_now(now);`（`let model = app.model_mut(id).expect("invariant: a started window has a model");` の後）を消し、その `let model = ...` の行の直前に `app.set_now(id, now);` を置く。
  13. `src/host/pump.rs`：
      - `turn` の window の loop を次にする。
        ```rust
        for id in self.window_ids() {
            let drawable = self.drawable(id);
            let mut app = self.app.borrow_mut();
            app.set_now(id, now);
            if let Some(model) = app.model_mut(id) {
                model.set_surface(drawable);
            }
        }
        ```
      - `effects` の `DeferredRequest::TimeScale(scale) => { ... }` の腕の本体を次にする。
        ```rust
        if scale.is_finite() {
            let now = windows.now_ns();
            self.app.borrow_mut().clock.set_scale(now, scale);
        }
        PlatformResult::Done
        ```
      - `tick_windows`：`let clock = app.motion();` と `let AppModel { windows, client_frames, events, .. } = &mut *app;` を次にする。
        ```rust
        let AppModel {
            windows,
            client_frames,
            events,
            clock,
            ..
        } = &mut *app;
        let clock = *clock;
        ```
        loop の中の `model.tick(now, client)` を `model.tick(now, &clock, client)` に、`HostEvent::FrameTick { ..., clock, }` の `clock` を `clock: clock.motion_clock(),` にする。
  14. `src/host/events.rs`：`handle_window` の `if let Some(model) = self.app.borrow_mut().model_mut(id) { model.frame.set_now(now); if stops_smooth_scrolls(&event) { ... } }` を次にする。
      ```rust
      {
          let mut app = self.app.borrow_mut();
          app.set_now(id, now);
          if let Some(model) = app.model_mut(id)
              && stops_smooth_scrolls(&event)
          {
              model.frame.stop_smooth_scrolls();
          }
      }
      ```
  15. `rg -n "set_time_scale|shift_state_times|shift_size_times|shift_layout_times|AppClock|TickHistory|frame\.set_now\(now\)" crates/kivact-engine/src` が何も返さないことを確かめる。
- **消すもの**：`AppClock`、`AppModel::motion`（field と method）、`frame::state::Clock`、`TickHistory`、`Frame::time_scale`、`Model::set_time_scale`、`shift_state_times`、`shift_size_times`、`shift_layout_times`。
- **状態の閉じ方**：`Clock` は数の field が `stopped_ns` 一つだけで対象外（`phase` は enum）。`Now` は `&mut self` の method が無く対象外。`Frame` は `drawable: bool` と `last_frame: Option<u64>` を持つので、手順 3 の印の一行で閉じる。
- **試験**：足さない・直さない。
- **確認（tier a）**：
  ```
  cargo fmt -p kivact-types
  cargo clippy -p kivact-types --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-types
  ENGINE-A
  E2E: motion, motion_easing, presence, presence_modes, shared_layout, frame_sources, images_animated, touch_scroll, windows, timeline, scroll_timeline, environment
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- devtools::
  ```
- **完了条件**：手順 15 の `rg` が空。上の確認がすべて通る。特に `motion::paused_run_across_a_clock_stop_resumes_where_it_paused`、`windows::destroying_the_main_surface_does_not_stop_or_rewind_another_window_animation`、`touch_scroll::fling_decelerates_and_stops`、`images_animated::gif_frames_advance` が通る。

## E02：一つの `MotionRun`

- **wave / lane / 前提**：W3b / E1 / E01
- **所有**：`src/model/motion_run.rs`（新設）、`src/model.rs`（`mod` の 1 行）、`src/model/state_motion/state.rs`、`src/model/state_motion/steps.rs`、`src/model/presence/state.rs`、`src/model/presence/size.rs`、`src/model/layout_motion/state.rs`、`src/model/layout_motion/steps.rs`
- **目的**：状態の transition の track、presence の大きさ、layout motion の三つの run を一つの型 `MotionRun` にし、step を一か所に置く（EN-17）。
- **手順**：
  1. `src/model/motion_run.rs` を作る。
     ```rust
     //! Role: One run of a transition between two lists of numbers, timed in motion ns.

     use kivact_types::motion::{SpringRun, Transition};

     /// One run: from `from` to `to`, drawn at `current`, started at `start`
     /// (motion ns, `frame::state::Now`).
     #[derive(Debug)]
     pub(crate) struct MotionRun {
         pub(crate) from: Vec<f64>,
         pub(crate) to: Vec<f64>,
         /// The numbers drawn now.
         pub(crate) current: Vec<f64>,
         /// How it moves.
         pub(crate) timing: Transition,
         /// When it started (motion ns).
         pub(crate) start: u64,
         /// A spring's velocity, step time and iteration.
         pub(crate) spring: SpringRun,
     }

     impl MotionRun {
         /// A run from `from` to `to` starting at `now` (motion ns); `spring`
         /// keeps an interrupted spring's velocity.
         pub(crate) fn start(
             from: Vec<f64>,
             to: Vec<f64>,
             timing: Transition,
             mut spring: SpringRun,
             now: u64,
         ) -> Self {
             spring.start(now, &timing, &from, &to);
             MotionRun {
                 current: from.clone(),
                 from,
                 to,
                 timing,
                 start: now,
                 spring,
             }
         }

         /// Step to `now` (motion ns); `true` when the run ended (at `to`, or
         /// where an alternate count leaves it). A decorative run under
         /// reduced motion ends at `to` at once.
         pub(crate) fn step(&mut self, now: u64, reduced: bool) -> bool {
             if reduced && self.timing.decorative {
                 self.current.clone_from(&self.to);
                 return true;
             }
             self.timing.step(
                 &self.from,
                 &self.to,
                 self.start,
                 now,
                 &mut self.current,
                 &mut self.spring,
             )
         }
     }
     ```
  2. `src/model.rs`：model の子 module の宣言の並び（`mod layout_motion;` などがある所）に `pub(crate) mod motion_run;` を足す（並びは名前順）。
  3. `src/model/state_motion/state.rs`：
     - `use crate::model::motion_run::MotionRun;` を足す。
     - `struct PropTrack` から `pub(super) transition: Arc<Transition>,` と、その doc を消す。
     - `TrackPhase::Running { interp, current, start, spring }` を次にする（doc は `/// Moving: \`run\` interpolates the numbers of \`basis\`.`）。
       ```rust
       Running {
           /// The shape the numbers are written into.
           basis: PropBasis,
           /// The run.
           run: MotionRun,
       },
       ```
     - `TrackPhase::drawn` の `Running` の腕を `TrackPhase::Running { basis, run } => Some((basis, &run.current)),` にする。
     - 使わなくなった `use`（`Arc`、`Transition`、`SpringRun` のうち未使用のもの）を消す。
  4. `src/model/state_motion/steps.rs`：
     - `use crate::model::motion_run::MotionRun;` を足す。
     - `captured_track`：`let transition = transition_for(paint, prop)?;` を `transition_for(paint, prop)?;` にし、`PropTrack { prop, transition, phase: ... }` から `transition,` を消す。
     - `StateAnimation::capture` の `TrackPhase::Running { interp, current, spring, .. } => TrackPhase::Captured { interp, shown: current, spring },` を次にする。
       ```rust
       TrackPhase::Running { basis, run } => TrackPhase::Captured {
           interp: PropInterp {
               basis,
               from: run.from,
               to: run.to,
           },
           shown: run.current,
           spring: run.spring,
       },
       ```
     - `retarget` の `spring.start(now, &timing, &new.from, &new.to);` から `step_track(track, now, reduced); }` までを次にする（`instant` の計算は前の行のまま）。
       ```rust
       let run = MotionRun::start(new.from.clone(), new.to.clone(), timing, spring, now);
       track.phase = TrackPhase::Running {
           basis: new.basis,
           run,
       };
       if instant {
           step_track(track, now, reduced);
       }
       ```
       `instant` は `timing` を使うので、`let instant = matches!(timing.duration_ms(), Some(ms) if ms <= 0.0);` は `MotionRun::start` の前に置く（今の位置のまま）。
     - `step_track` の本体を次にする。
       ```rust
       let phase = std::mem::replace(&mut track.phase, TrackPhase::Settled);
       track.phase = match phase {
           TrackPhase::Running { basis, mut run } => {
               if !run.step(now, reduced) {
                   TrackPhase::Running { basis, run }
               } else if run.current == run.to {
                   TrackPhase::Settled
               } else {
                   TrackPhase::Held {
                       interp: PropInterp {
                           basis,
                           from: run.from,
                           to: run.to,
                       },
                       value: run.current,
                   }
               }
           }
           other => other,
       };
       ```
     - `PropInterp` の `use` は手順 3 の state.rs と同じ path を使う（1.2 節）。
  5. `src/model/presence/state.rs`：`struct SizeMotion` を次にする。
     ```rust
     /// The scalar motion of one presence axis: a run of one number from 0 (none) to 1 (its size).
     pub(crate) struct SizeMotion {
         pub(crate) axis: kivact_types::Orientation,
         pub(super) run: crate::model::motion_run::MotionRun,
     }
     ```
  6. `src/model/presence/size.rs`：
     - `start_size` の `let from = ... .map_or(if entering { 0.0 } else { 1.0 }, |m| m.current[0]);` の `m.current[0]` を `m.run.current[0]` にし、`self.presence.sizes.insert(id, SizeMotion { ... });` を次にする。
       ```rust
       let to = if entering { 1.0 } else { 0.0 };
       let start = self.frame.request_motion_frame();
       self.presence.sizes.insert(
           id,
           SizeMotion {
               axis,
               run: MotionRun::start(vec![from], vec![to], transition, SpringRun::default(), start),
           },
       );
       ```
       （`use crate::model::motion_run::MotionRun;` と `use kivact_types::motion::SpringRun;` を足す。）
     - `step_sizes` の `let done = (reduced && m.transition.decorative) || m.transition.step(...);` を `let done = m.run.step(now, reduced);` にし、続く `set_presence_scale` の式の `m.to` を `m.run.to[0]` に、`m.current[0]` を `m.run.current[0]` にする。
  7. `src/model/layout_motion/state.rs`：`struct LayoutAnimation` を消し、`runs: HashMap<NodeId, LayoutAnimation>` を `runs: HashMap<NodeId, crate::model::motion_run::MotionRun>` にする。使わなくなった `use kivact_types::motion::{SpringRun, Transition};` を消す。
  8. `src/model/layout_motion/steps.rs`：
     - `use super::state::LayoutAnimation;` を `use crate::model::motion_run::MotionRun;` にし、`fn run_map(run: &LayoutAnimation)` を `fn run_map(run: &MotionRun)` にする。
     - `retarget_run` の `let start = self.frame.request_motion_frame(); spring.start(start, &timing, &from, &to); self.layout_motion.runs.insert(id, LayoutAnimation { ... });` を次にする。
       ```rust
       let start = self.frame.request_motion_frame();
       self.layout_motion
           .runs
           .insert(id, MotionRun::start(from, to, timing, spring, start));
       ```
     - `step_layout_motion` の `retain` の閉包を `|id, a| tree.contains(*id) && !a.step(now, reduced)` にする。
  9. `rg -n "LayoutAnimation|\.transition\.step|m\.transition|track\.transition" crates/kivact-engine/src` が空であることを確かめる。
- **消すもの**：`LayoutAnimation`、`SizeMotion` の `transition`・`start`・`from`・`to`・`current`・`spring`、`PropTrack::transition`、`TrackPhase::Running` の `interp`・`current`・`start`・`spring`。
- **状態の閉じ方**：`MotionRun` の数の field は `start` 一つで対象外（`Vec` と `SpringRun` は数えない）。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: motion, motion_easing, presence, presence_modes, shared_layout, style_paint, interaction_show_hide`
- **完了条件**：手順 9 の `rg` が空。確認がすべて通る。

## E03：slot の列の持ち主を一つにする（`Slots`）

- **wave / lane / 前提**：W3b / E2 / なし
- **所有**：`src/model/aggregate.rs`、`src/model/lifecycle.rs`、`src/paint/state.rs`、`src/paint/list/invalidation.rs`、`src/model/interact/sync.rs`、`src/tree/store.rs`、`src/tree/layout_adapter.rs`、手順 3 の置換が触れるファイル（`rg -l "\b(self|host|model)\.editors\b" crates/kivact-engine/src` の結果と、手順 3 の measure の一覧と、`paint.ink`・`paint.look` の一覧）、`crates/kivact-layout/src/borrowed_tree.rs`、`crates/kivact-layout/src/layout.rs`、`crates/kivact-layout/src/columns.rs`、`crates/kivact-layout/src/compare.rs`、`crates/kivact-layout/src/layout/tests.rs`
- **目的**：node の slot ごとの列（measure、editor、ink、look）を一つの struct `Slots` が持ち、push と reset を一か所でする。kivact-layout の列は engine の番号を信じて push されるのをやめ、使う時に `slot_count()` まで伸ばす（EN-25）。
- **手順**：
  1. `src/model/aggregate.rs` に次を足す（`impl Model` の前）。`use std::cell::Cell;` が無ければ足す。
     ```rust
     /// The window's per-slot columns, one entry per structural slot of the
     /// node store: pushed together when a slot is made, reset together when
     /// it is freed (`Model::create_slot`, `Model::remove_slots`). The tree's
     /// flat-children cache grows by itself (`NodeTree`), and the layout's
     /// columns grow to the store's slot count when a pass reads them
     /// (`kivact_layout::Layout`).
     #[derive(Default)]
     pub(crate) struct Slots {
         /// What each slot measures and its resolved text style.
         pub(crate) measure: crate::measure::state::MeasureColumns,
         /// The native editor of each slot that edits text.
         pub(crate) editors: Vec<Option<Box<crate::measure::editor::EditorNode>>>,
         /// The ink each box last drew on (`paint::list::ink`).
         pub(crate) ink: Vec<Cell<Option<crate::paint::state::Ink>>>,
         /// The base paint's look of each node (`Model::derive`).
         pub(crate) look: Vec<Option<crate::layer::Look>>,
     }

     impl Slots {
         /// Append a fresh slot for a node of `kind`.
         pub(crate) fn push(&mut self, kind: &crate::contract::NodeKind) {
             self.measure.push(kind);
             self.editors.push(None);
             self.ink.push(Cell::new(None));
             self.look.push(None);
         }

         /// Reset `slot` before it is freed; returns the editor and the
         /// shaped text extents it held, for the caller to drop in order.
         pub(crate) fn reset(
             &mut self,
             slot: usize,
         ) -> (
             Option<Box<crate::measure::editor::EditorNode>>,
             Option<Box<crate::measure::state::TextExtents>>,
         ) {
             let editor = self.editors[slot].take();
             let extents = self.measure.take_slot(slot);
             self.ink[slot].set(None);
             self.look[slot] = None;
             (editor, extents)
         }
     }
     ```
  2. `struct Model` の `pub(crate) measure: crate::measure::state::MeasureColumns,` と `pub(crate) editors: Vec<Option<Box<...EditorNode>>>,` を消し、`pub(crate) tree: NodeTree,` の次に `pub(crate) slots: Slots,` を足す。`Model::new` の `measure: Default::default(),` と `editors: Vec::new(),` を消して `slots: Slots::default(),` を足す。
  3. 置換（Git Bash、repository の root で）：
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-engine/src" || exit 1
     rg -l '\b(self|host|model)\.editors\b' . | xargs perl -pi -e 's/\b(self|host|model)\.editors\b/$1.slots.editors/g'
     for f in check/layout_reference.rs a11y/text.rs model/lifecycle.rs model/apply.rs model/edit/inputs.rs model/interact/geometry.rs model/interact/sync.rs model/text.rs; do perl -pi -e 's/\bself\.measure\b(?!\s*\()/self.slots.measure/g' "$f"; done
     rg -l '\b(self|host|model)\.paint\.(ink|look)\b' . | xargs perl -pi -e 's/\b(self|host|model)\.paint\.(ink|look)\b/$1.slots.$2/g'
     cd "$ROOT" || exit 1
     ```
     `src/measure/leaf.rs` の `self.measure.measure(...)`（`EngineMeasure` の field）と `src/mutation.rs` の `self.measure || other.measure`（`Mutation` の field）は置換しない（上の一覧に入れていない）。
  4. `src/paint/state.rs`：`struct PaintState` から `ink` と `look` の field を消し、`PaintState::new` からその初期化を、`impl PaintState` から `push_slot` と `reset_slot` を消す。
  5. `src/paint/list/invalidation.rs`：`PaintState::outdate_paint` の署名に最後の引数 `ink: &[std::cell::Cell<Option<Ink>>]` を足し、本体の `list.invalidate(changes, tree, &self.ink);` を `list.invalidate(changes, tree, ink);` にする。`src/model/interact/sync.rs` の呼出し `self.paint.outdate_paint(&self.tree, &changes, redraw, changes.fonts, restack);` を `self.paint.outdate_paint(&self.tree, &changes, redraw, changes.fonts, restack, &self.slots.ink);` にする。`rg -n "forget_ink\(" crates/kivact-engine/src` で `&self.paint.ink` を渡している呼出しがあれば `&self.slots.ink` にする。
  6. `src/model/lifecycle.rs`：
     - `create_slot` の `if fresh { self.layout.push(); self.measure.push(&kind); self.paint.push_slot(); self.editors.push(None); } else { self.measure.set_measure(index.0, &kind, false); }`（手順 3 の後は `self.slots.measure`）を次にする。
       ```rust
       if fresh {
           self.slots.push(&kind);
       } else {
           // A reused slot measures the new node's kind, never the old one's.
           self.slots.measure.set_measure(index.0, &kind, false);
       }
       ```
     - `remove_slots` を次にする。
       ```rust
       pub(crate) fn remove_slots(&mut self, id: NodeId) -> Vec<NodeId> {
           let Self {
               tree,
               layout,
               slots,
               ..
           } = self;
           tree.remove_with(id, |index, node| {
               let (editor, extents) = slots.reset(index.0);
               // Own user-bearing fields before a reset can fail, as the old
               // removed HostNode did. Slot publication still follows all resets.
               let payload = RemovedPayload {
                   node,
                   editor,
                   extents,
               };
               layout.forget(index.0);
               payload
           })
       }
       ```
     - `impl Drop for Model` の本体を次にする。
       ```rust
       let Self { tree, slots, .. } = self;
       tree.nodes.drain_payloads(|index, node| {
           let extents = slots.measure.take_slot(index.0);
           let editor = slots.editors[index.0].take();
           drop(RemovedPayload {
               node,
               editor,
               extents,
           });
       });
       ```
  7. 手順 3 の置換で直らない残り：`cargo check -p kivact-engine --all-targets --features testing` の error を順に直す。直し方は次の三つだけ：(a) `let Self { ..., measure, ..., editors, .. } = self;` のような分解は `slots` を取り出し、中で `slots.measure`・`slots.editors` を使う。(b) `Measure::new(engine, &mut assets, &self.slots.measure, &self.slots.editors)` のように `&self.slots.X` を渡す。(c) 型名の path（`crate::paint::state::Ink`）の `use` を足す。
  8. `src/tree/store.rs`：`impl NodeStore` に次を足す。
     ```rust
     /// The number of slots (live or free): every slot index is below it.
     pub(crate) fn slot_count(&self) -> usize {
         self.entries.len()
     }
     ```
  9. `crates/kivact-layout/src/borrowed_tree.rs`：`trait LayoutTree` の `fn index_of` の前に次を足す。
     ```rust
     /// The number of structural slots (live or vacant): every slot is below it.
     fn slot_count(&self) -> usize;
     ```
     `src/tree/layout_adapter.rs` の `impl LayoutTree for NodeTree` に `#[inline] fn slot_count(&self) -> usize { self.nodes.slot_count() }` を足す。`rg -n "impl LayoutTree for|impl<.*> LayoutTree for" crates` で他の実装（kivact-layout の試験の tree など）を探し、同じ method を足す（その tree の slot の数を返す）。
  10. `crates/kivact-layout/src/columns.rs`：
      - `pub(crate) fn push(&mut self)` を `fn push_slot(&mut self)` にし（本体はそのまま）、その前に次を足す。
        ```rust
        /// Grow every column to hold `slots` structural slots (plus the viewport).
        pub(crate) fn ensure(&mut self, slots: usize) {
            while self.states.len() < slots + 1 {
                self.push_slot();
            }
        }
        ```
      - `pub(crate) fn reset(&mut self, slot: usize)` を `pub(crate) fn forget(&mut self, slot: usize)` にし、本体の先頭に `if slot + 1 >= self.states.len() { return; }` を置く（列がまだ伸びていない slot は何も持たない）。doc の「Allocate a column slot when ...」の段落を「Columns grow to the tree's slot count when a computation reads them (`ensure`); a freed slot is forgotten (`forget`).」にする。
  11. `crates/kivact-layout/src/layout.rs`：
      - `pub fn push(&mut self)` を消す。`pub fn reset(&mut self, slot: usize)` を `pub fn forget(&mut self, slot: usize) { self.columns.forget(slot); }` にし、doc を `/// Forget a freed zero-based structural slot (nothing when the columns never reached it).` にする。
      - `set_presence_scale`、`rebuild_style`、`rebuild_all`、`invalidate_chain`、`compute`、`compute_overlay` の本体の最初の行に `self.columns.ensure(tree.slot_count());` を置く（`compute` は `if tree.root().is_none()` の判定の前）。
      - `remeasures_same` は `self.columns.remeasures_same(...)` の前に `if slot + 1 >= self.columns.states.len() { return false; }` を置く（`states` が `pub(crate)` なのでそのまま読める）。
      - `struct Layout` の doc を `/// Derived layout and invalidation for the slots of one node store (see LayoutTree::slot_count).` にする。
  12. `crates/kivact-layout/src/compare.rs`：`compare_measurements` と `compare` の中で slot ごとの `Vec` を `[at]` で読む所を、`get(at)` で読んで無ければその slot を飛ばす（`continue`）形にする。
  13. `crates/kivact-layout/src/layout/tests.rs`：`layout.push();` の行（3 か所）を消す。`.reset(` があれば `.forget(` にする。
  14. `rg -n "\.push_slot\(|reset_slot|layout\.push\(\)|\bself\.editors\b|\bself\.measure\b" crates/kivact-engine/src` が、`src/measure/leaf.rs` の `self.measure.measure` と `src/model/aggregate.rs` の `Slots` の中だけを返すことを確かめる。
- **消すもの**：`Model::measure`、`Model::editors`、`PaintState::{ink, look, push_slot, reset_slot}`、`kivact_layout::Layout::{push, reset}`、`NodeLayoutColumns::{push, reset}`（`push_slot`・`forget` に替わる）。
- **状態の閉じ方**：`Slots` は `Vec` の列だけで対象外。
- **試験**：`crates/kivact-layout/src/layout/tests.rs` の `push` の呼出しを手順 13 のとおり消す（期待値は変えない）。
- **確認（tier a）**：`LAYOUT-A`、`ENGINE-A`、`E2E: layout, layout_scroll, text_editing, custom_text_field, images_animated, assets, resource_reclaim, disposal, presence, a11y_incremental, compositor`
- **完了条件**：手順 14 の `rg` の結果のとおり。確認がすべて通る。

## E04：祖先を link で辿る、描画の判定を walk で渡す、window の原点の cache

- **wave / lane / 前提**：W3b / E2 / E03
- **所有**：`src/tree/node_tree.rs`、`src/paint/list/walk.rs`、`src/model/interact/geometry.rs`、`src/model/interact/sticky.rs`、`src/model/interact/sync.rs`、`src/model/aggregate.rs`
- **目的**：祖先の走査を `NodeIndex` の link だけで行い（一段ごとに id の hash を引かない）、paint の walk は親から「描画される」を受け継ぎ、`rect()` の window 原点を幾何が変わるまで覚える（EN-24）。
- **手順**：
  1. `src/tree/node_tree.rs` の `impl NodeTree` に次を足す。
     ```rust
     /// `index` and its ancestors with their nodes, nearest first, by the
     /// links alone (no id lookups).
     pub(crate) fn ancestors(
         &self,
         index: NodeIndex,
     ) -> impl Iterator<Item = (NodeIndex, &HostNode)> + '_ {
         std::iter::successors(Some(index), move |i| self.nodes.links(*i).parent)
             .map(move |i| (i, self.nodes.at(i)))
     }
     ```
  2. 同じファイルの次の関数の本体を置き換える（署名はそのまま）。
     ```rust
     pub(crate) fn flat_parent(&self, node: &HostNode) -> Option<NodeId> {
         self.ancestors(node.index)
             .skip(1)
             .find(|(_, n)| !n.is_transparent())
             .map(|(i, _)| self.nodes.id_at(i))
     }

     pub(crate) fn chain(&self, node: Option<NodeId>) -> Vec<NodeId> {
         node.and_then(|id| self.nodes.index_of(id))
             .map_or_else(Vec::new, |start| {
                 self.ancestors(start)
                     .map(|(i, _)| self.nodes.id_at(i))
                     .collect()
             })
     }

     pub(crate) fn depth(&self, id: NodeId) -> usize {
         self.nodes
             .index_of(id)
             .map_or(0, |start| self.ancestors(start).count() - 1)
     }

     pub(crate) fn is_in_subtree(&self, node: NodeId, ancestor: NodeId) -> bool {
         if node == ancestor {
             return true;
         }
         let (Some(start), Some(target)) = (self.nodes.index_of(node), self.nodes.index_of(ancestor))
         else {
             return false;
         };
         self.ancestors(start).any(|(i, _)| i == target)
     }

     fn attached_where(&self, id: NodeId, keep: impl Fn(NodeId, &HostNode) -> bool) -> bool {
         let Some(start) = self.nodes.index_of(id) else {
             return false;
         };
         let mut last = start;
         for (i, n) in self.ancestors(start) {
             if !keep(self.nodes.id_at(i), n) {
                 return false;
             }
             last = i;
         }
         self.root == Some(self.nodes.id_at(last))
     }
     ```
     `is_in_subtree` の引数の `mut` は消す。
  3. `src/paint/list/walk.rs`：`Walk::node` の `if !host.tree().drawn(id) { return; }` を次にする（walk は描画される親からだけ子へ降り、根と layer は `Model::layers` が描画されるものだけを渡すので、自分の隠れだけを見ればよい）。
     ```rust
     // The walk reaches a node only from a drawn parent, or as a root or
     // layer `Model::layers` found drawn: its own visibility decides.
     if n.hidden() || n.visibility_hidden() {
         return;
     }
     debug_assert!(
         host.tree().drawn(id),
         "invariant: the paint walk reaches only drawn nodes"
     );
     ```
     同じファイルの `subtree_paint_list` の `walk.node(id, Point::default(), 1.0, clip, None);` を `if self.tree().drawn(id) { walk.node(id, Point::default(), 1.0, clip, None); }` にする（export の根は隠れた祖先の下にあり得る。前は `walk.node` の中の `drawn` が同じ判定をしていた）。
  4. `src/model/aggregate.rs`：`struct Model` に次の field を足し（`pub(crate) viewport: Size,` の次）、`Model::new` で `origins: Default::default(),` にする。
     ```rust
     /// The window origins `Model::rect` found since the geometry last
     /// changed (forgotten by `Model::forget_origins`).
     pub(crate) origins: std::cell::RefCell<kivact_types::IdMap<kivact_types::Point>>,
     ```
     `impl Model` に次を足す。
     ```rust
     /// Positions moved (a layout pass, a scroll, a sticky shift, a resize or
     /// a moved box): the remembered window origins are found again.
     pub(crate) fn forget_origins(&mut self) {
         self.origins.get_mut().clear();
     }
     ```
  5. `src/model/interact/geometry.rs`：`pub(crate) fn rect` を次の二つにする（元の loop を `window_origin` に移す）。
     ```rust
     pub(crate) fn rect(&self, id: NodeId) -> Option<Rect> {
         if self.tree.get(id)?.is_transparent() {
             return None;
         }
         let size = self.computed_layout(id)?.size;
         let kept = self.origins.borrow().get(&id).copied();
         let origin = match kept {
             Some(origin) => origin,
             None => {
                 let origin = self.window_origin(id)?;
                 self.origins.borrow_mut().insert(id, origin);
                 origin
             }
         };
         Some(Rect {
             x: origin.x,
             y: origin.y,
             width: size.width,
             height: size.height,
         })
     }

     /// The window position of `id`'s border box: its location moved by
     /// each box above it less its scroll, up to an overlay's origin or a
     /// fixed box.
     fn window_origin(&self, id: NodeId) -> Option<Point> {
         let (mut x, mut y) = (0.0, 0.0);
         let mut cur = id;
         loop {
             // (the body of the old `loop` in `rect`, unchanged)
         }
         Some(Point { x, y })
     }
     ```
     `window_origin` の loop の中身は、元の `rect` の `loop { let node = self.tree.get(cur)?; ... }` をそのまま写す。`self.origins.borrow()` は `window_origin` を呼ぶ前に手放す（上のとおり `kept` に写してから呼ぶ。`overlay_origin` が `rect` を呼び返すため）。
  6. `forget_origins` を呼ぶ所：
     - `src/model/interact/geometry.rs` の `pub(crate) fn layout(` の本体の先頭と最後の行の直前に `self.forget_origins();`。
     - 同じファイルの `set_scroll` の中、offset が変わって `n.scroll = o;` を書いた直後に `self.forget_origins();`（`n` の借用が終わってから。`changes.scrolled(id)` の行の前）。
     - `src/model/interact/sticky.rs`：`n.shift = shift;` を書く `if` の中で、その借用が終わった後に `self.forget_origins();`。
     - `src/model/interact/sync.rs`：`set_viewport` の `if self.viewport != window {` の中の先頭に `self.forget_origins();`。`sync()` の `let changes = self.changes.dispatch();` の次の行に `if changes.geometry != Geometry::Kept { self.forget_origins(); }`。
- **消すもの**：`attached_where`・`flat_parent`・`chain`・`depth`・`is_in_subtree` の id を引く loop（`parent_id` は他の呼出し元があるので残す）。
- **状態の閉じ方**：`Model` に `RefCell` の field が一つ増える。`Model` の印（W1 で付属書 T が置いた struct 単位の一行）にこの field の役を足す：印の文の最後に `; origins caches window origins until positions move (forget_origins)` を足す。印が無ければ、struct の直前に `// xtask-allow(state): the model of one window: each group's state lives in its own leaf module; origins caches window origins until positions move (forget_origins)` を置く。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: layout, layout_scroll, overlay, a11y_incremental, focus, hover, transform, transform_3d, compositor, drawables, interaction_tooltip`
- **完了条件**：`rg -n "fn attached_where" -A 12 crates/kivact-engine/src/tree/node_tree.rs` の本体に `parent_id` が無い。確認がすべて通る。

## E05：IME の対象を導出する、`KeyResult::Text` を消す、`LeafCache` に改名

- **wave / lane / 前提**：W4 / E1 / なし
- **所有**：`src/model/edit/state.rs`、`src/model/edit/session.rs`、`src/model/edit/ime.rs`、`src/model/focus/navigation.rs`、`src/model/keys.rs`、`src/host/events.rs`、`src/host/write/reader.rs`、`src/measure/state.rs`
- **目的**：`ImeTarget` を保存せず、使う時に focus・editor・props から O(1) で導く（EN-13）。`KeyResult::Text` を消し、文字は `handle_key` が自分で入れる（EN-20）。engine 内の列の enum `LeafMeasure` を `LeafCache` に改名する（EN-22）。
- **手順**：
  1. `src/model/edit/state.rs`：`struct Edits` の `pub(crate) target: ImeTarget,` を消す。doc の「and what the IME was last given」はそのまま。`ImeTarget` と `EditorSlot` の型は残す。
  2. `src/model/edit/session.rs`：`pub(crate) fn update_ime_target(&mut self)` を次の関数に置き換える。
     ```rust
     /// What the IME edits now: the focused native editor, the focused node
     /// with an IME session, or nothing (a region, a disabled node, a plain
     /// node). Derived from the focus and the node each time (O(1)).
     pub(crate) fn ime_target(&self) -> super::state::ImeTarget {
         use super::state::{EditorSlot, ImeTarget};
         self.focused()
             .filter(|f| f.region.is_none())
             .and_then(|f| self.tree.get(f.node).map(|n| (f.node, n)))
             .filter(|(_, n)| !n.disabled())
             .map_or(ImeTarget::None, |(id, n)| {
                 if self.slots.editors[n.index.0].is_some() {
                     ImeTarget::Native(EditorSlot(n.index))
                 } else if n.properties.ime.is_some() {
                     ImeTarget::Session(id)
                 } else {
                     ImeTarget::None
                 }
             })
     }
     ```
     `focused_session` の `match self.edits.target {` を `match self.ime_target() {` にする。
  3. `src/model/keys.rs`：`focused_input` の `match self.edits.target {` を `match self.ime_target() {` にする。
  4. `src/model/focus/navigation.rs`：`change_focus` の `self.update_ime_target();` の行と、`revalidate_focus` の最初の行 `self.update_ime_target();` を消す。
  5. `src/host/write/reader.rs`：`ime_state` の `match model.edits.target {` を `match model.ime_target() {` に、`if matches!(model.edits.target, ImeTarget::Native(_))` を `if matches!(model.ime_target(), ImeTarget::Native(_))` にする。
  6. `src/model/edit/ime.rs`：`ime_select` の最後の 3 行 `self.place_caret(id, start.min(end), false); self.place_caret(id, start.max(end), true); self.sync();` を次にする（一つの journal で選択する）。
     ```rust
     self.edit_input(id, |e| {
         e.move_to(start.min(end), false);
         e.move_to(start.max(end), true);
         false
     });
     self.sync();
     ```
  7. `src/model/keys.rs`：
     - `enum KeyResult` の `Text(String)` の variant とその doc を消す。
     - `handle_key` の `if let Some(t) = text && !command && !read_only && !focus_key && key != Key::Enter { return KeyResult::Text(t.to_string()); }` を次にする。
       ```rust
       if let Some(t) = text
           && !command
           && !read_only
           && !focus_key
           && key != Key::Enter
       {
           // Text for the focused text input: inserted here, as typed.
           self.insert_text(t);
           return KeyResult::Consumed;
       }
       ```
     - `key()` の `if let KeyResult::Text(text) = &result { self.insert_text(text); }` の 3 行を消す。
     - `KeyResult::Consumed` の doc を `/// Used by the host: a focus move, an edit (typed text included), or the IME's while composing.` にする。
  8. `src/host/events.rs`：`key_event` の `KeyResult::Reported | KeyResult::Consumed | KeyResult::Text(_) => {}` を `KeyResult::Reported | KeyResult::Consumed => {}` にする。
  9. `perl -pi -e 's/\bLeafMeasure\b/LeafCache/g' crates/kivact-engine/src/measure/state.rs` を実行する（このファイルは列の enum だけを指し、`contract::LeafMeasure` を指さない）。
  10. `rg -n "update_ime_target|edits\.target|KeyResult::Text" crates/kivact-engine/src` と `rg -n "\bLeafMeasure\b" crates/kivact-engine/src/measure/state.rs` が空であることを確かめる。
- **消すもの**：`Edits::target`、`Model::update_ime_target`、`KeyResult::Text`、`measure::state::LeafMeasure`（`LeafCache` に改名）。
- **状態の閉じ方**：新しい状態の型は無い。`Edits` は field が減るだけ。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: text_editing, custom_text_field, input_platform, keyboard, focus, components, launch_real_window`、`E2E: ui_chat_code, interaction_edit`
- **完了条件**：手順 10 の `rg` が空。確認がすべて通る。

## E06：Tab 順を作らずに次の focus 先を探す、`set_scene` の restack を絞る

- **wave / lane / 前提**：W4 / E1 / E05
- **所有**：`src/model/focus/navigation.rs`、`src/model/apply.rs`、`src/paint/state.rs`
- **目的**：focus の再検証・削除・scene の書込みで、Tab 順の全体（O(N)）を作らず、focus の位置から先へ必要なだけ進んで次の対象を探す（EN-05、EN-11）。`set_scene` は scene の layer・item の事実が変わったときか、node が今の list に自分で描かれていないときだけ list を作り直す（EN-04 の一部）。
- **手順**：
  1. `src/model/focus/navigation.rs` の `impl Model` に次の四つを足す（`focus_order` の後）。
     ```rust
     /// The targets of the node `id` alone, in order: the node, then its
     /// scene regions by their focus order (each region once).
     pub(in crate::model) fn node_targets(&self, id: NodeId) -> Vec<FocusTarget> {
         let mut targets = vec![FocusTarget {
             node: id,
             region: None,
         }];
         let Some(node) = self.tree.get(id) else {
             return targets;
         };
         let mut regions: Vec<_> = node
             .scenes()
             .iter()
             .flatten()
             .flat_map(|s| &s.regions)
             .collect();
         regions.sort_by_key(|r| r.focus_order);
         let mut seen = std::collections::HashSet::new();
         for region in regions {
             if seen.insert(region.id) {
                 targets.push(FocusTarget {
                     node: id,
                     region: Some(region.id),
                 });
             }
         }
         targets
     }

     /// The node after `id` in flat preorder (into its subtree first when
     /// `into`), or `None` after the last node.
     fn preorder_next(&self, id: NodeId, into: bool) -> Option<NodeId> {
         if into && let Some(first) = self.tree.flat_children(id).next() {
             return Some(first);
         }
         let mut at = id;
         loop {
             let parent = self.tree.get(at).and_then(|n| self.tree.flat_parent(n))?;
             let mut siblings = self.tree.flat_children(parent);
             if siblings.by_ref().any(|c| c == at)
                 && let Some(next) = siblings.next()
             {
                 return Some(next);
             }
             at = parent;
         }
     }

     /// The first target `accept` takes on the nodes after `start` in flat
     /// preorder (into its subtree when `into`), going on from the root once
     /// the tree ends and stopping at `start`. It walks only as far as the
     /// first accepted target.
     fn scan_from(
         &self,
         start: NodeId,
         into: bool,
         accept: impl Fn(FocusTarget) -> bool,
     ) -> Option<FocusTarget> {
         let root = self.tree.root()?;
         let attached = self.tree.contains(start);
         let mut next = if attached {
             self.preorder_next(start, into)
         } else {
             Some(root)
         };
         let mut wrapped = !attached;
         loop {
             let id = match next {
                 Some(id) => id,
                 None if !wrapped => {
                     wrapped = true;
                     root
                 }
                 None => return None,
             };
             if let Some(t) = self.node_targets(id).into_iter().find(|t| accept(*t)) {
                 return Some(t);
             }
             if id == start {
                 return None;
             }
             next = self.preorder_next(id, true);
         }
     }

     /// The first target after `from` that `accept` takes: the later targets
     /// of its node (all of them when `from` is not one), then the nodes
     /// after it, wrapping once.
     pub(in crate::model) fn next_target_after(
         &self,
         from: FocusTarget,
         accept: impl Fn(FocusTarget) -> bool,
     ) -> Option<FocusTarget> {
         if self.tree.contains(from.node) {
             let own = self.node_targets(from.node);
             let at = own.iter().position(|t| *t == from).map_or(0, |i| i + 1);
             if let Some(t) = own[at..].iter().copied().find(|t| accept(*t)) {
                 return Some(t);
             }
         }
         self.scan_from(from.node, true, accept)
     }
     ```
  2. `focus_order` の本体の loop の中の region の列挙（`let mut seen = ...` から `for region in regions { ... }` まで）を `targets.extend(self.node_targets(id).into_iter().skip(1));` にし、`targets.push(FocusTarget { node: id, region: None });` と `let Some(node) = ... else { continue; };` は `targets.extend(self.node_targets(id));` の一行にまとめる。doc を次にする：`/// Every declared target in node, region, then descendant order: a complete enumeration (PERF-1), used only by the Tab key's listing (\`tab_stops\`, \`roving_targets\`) and the DevTools' tab order, both paced by the user.`
  3. `next_tab_stop_outside` の本体を次にする。
     ```rust
     let modal = self.top_modal(Some(removed));
     self.scan_from(removed, false, |t| {
         !self.tree.is_in_subtree(t.node, removed)
             && self.is_focus_target(t)
             && modal.is_none_or(|m| self.tree.is_in_subtree(t.node, m))
     })
     ```
  4. `revalidate_focus` の `let order = self.focus_order();` から `self.move_focus_to(next);` までを次にする。
     ```rust
     let next = self.next_target_after(focus, |t| self.is_focus_target(t) && self.in_scope(t.node));
     self.move_focus_to(next);
     ```
  5. `src/paint/state.rs` の `impl PaintState` に次を足す。
     ```rust
     /// Whether the current list draws `id` itself (an `Open` step of its
     /// own); `false` while the list waits to be built again.
     pub(crate) fn lists_own(&self, id: NodeId) -> bool {
         self.list.state != ListState::Stale
             && self
                 .list
                 .entries
                 .iter()
                 .any(|e| matches!(e, Entry::Open(step) if step.id == id && step.own))
     }
     ```
  6. `src/model/apply.rs` の `set_scene` を次にする（`roots` の計算とその loop は今のまま中に残す）。
     ```rust
     pub(crate) fn set_scene(&mut self, id: NodeId, layer: SceneLayer, scene: Arc<DrawList>) {
         self.stale_fact(id);
         let focus = self.focused().filter(|focus| focus.node == id);
         let old_targets = if focus.is_some() {
             self.node_targets(id)
         } else {
             Vec::new()
         };
         let before = self.tree.get(id).and_then(|n| {
             n.scenes()[layer_index(layer)]
                 .as_deref()
                 .map(crate::lower::scene::scene_layers)
         });
         // A new layer or item in the scene changes the node's layer facts;
         // a node the list does not draw may show now: the list is built again.
         // Otherwise the write forgets the node's ink (WRITTEN) and it is
         // drawn again in place.
         if before != Some(crate::lower::scene::scene_layers(&scene)) || !self.paint.lists_own(id) {
             self.changes.restack = true;
         }
         if let Some(n) = self.tree.get_mut(id) {
             n.scenes[layer_index(layer)] = Some(scene);
             self.changes.mark(id, Dirty::WRITTEN);
             self.a11y.node(id);
         }
         // (the `roots` block, unchanged)
         if let Some(focus) = focus.filter(|f| !self.is_focus_target(*f)) {
             let start = old_targets.iter().position(|t| *t == focus).unwrap_or(0);
             let next = old_targets[start..]
                 .iter()
                 .chain(old_targets[..start].iter())
                 .copied()
                 .find(|t| self.is_focus_target(*t) && self.in_scope(t.node))
                 .or_else(|| {
                     self.next_target_after(focus, |t| self.is_focus_target(t) && self.in_scope(t.node))
                 });
             self.move_focus_to(next);
         }
     }
     ```
     `scene_layers` の path は `src/model/interact/geometry.rs` の `use crate::lower::scene::scene_layers;` と同じ。
  7. `rg -n "focus_order\(\)" crates/kivact-engine/src` の結果が `grouped_tab_stops` の中だけであることを確かめる。
- **消すもの**：`set_scene` の無条件の `self.changes.restack = true;`、`next_tab_stop_outside`・`revalidate_focus`・`set_scene` の中の `focus_order()` の呼出し。
- **状態の閉じ方**：新しい状態は無い。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: focus, windows, canvas, drawables, overlay, presence, lifecycle, disposal, interaction_dialog, interaction_menu, interaction_tabs, apg_tabs, apg_grid, apg_toolbar, apg_radio_group, apg_tree_view, apg_listbox, apg_menu`
- **完了条件**：手順 7 の `rg` のとおり。確認がすべて通る。

## E07：layer の根の変化で facts を全部作り直さない

- **wave / lane / 前提**：W4 / E1 / E06
- **所有**：`src/model/apply.rs`、`src/tree/node_tree.rs`
- **目的**：overlay の開閉、Contents の切替え、fixed の切替え、切り離された subtree の挿入で `StaleFacts::All` にしない。変わった node と、その親・subtree の facts だけを古いとする（EN-06）。`StaleFacts::All` は新しい根と paint の self-check だけに残る。
- **手順**：
  1. `src/tree/node_tree.rs`：`pub(crate) fn forget_flat(&mut self)` を消し、次を足す。
     ```rust
     /// `id` started or stopped being a Contents node, or a Contents node was
     /// hidden or shown: forget its own flat list and the lists of its
     /// ancestors up to its nearest box, which list its children.
     pub(crate) fn forget_flat_through(&self, id: NodeId) {
         let Some(start) = self.nodes.index_of(id) else {
             return;
         };
         let mut flat = self.flat.borrow_mut();
         for (index, n) in self.ancestors(start) {
             if let Some(entry) = flat.get_mut(index.0) {
                 *entry = None;
             }
             if index != start && !n.is_transparent() {
                 break;
             }
         }
     }
     ```
  2. `src/model/apply.rs` の `reflattened` の最初の 2 行 `self.tree.forget_flat(); self.paint.stale_all_facts();` を `self.tree.forget_flat_through(id);` にし、`if let Some(parent) = ... { ... }` の塊の中の先頭に `self.stale_fact(parent);` を足し、塊の後に `self.stale_fact(id);` を足す。
  3. `changed` の `if fixed != was_fixed { // A fixed box came or went ... self.paint.stale_all_facts(); ... }` の `self.paint.stale_all_facts();` を次にする。
     ```rust
     // It became or stopped being a layer root: its facts and its box
     // parent's facts are derived again.
     self.paint.forget_fact(id);
     self.stale_fact(id);
     if let Some(box_parent) = self.tree.get(id).and_then(|n| self.tree.flat_parent(n)) {
         self.stale_fact(box_parent);
     }
     ```
  4. `insert`：
     - `if self.tree.insert(parent, id, before).is_none() { // Not attached ... self.paint.stale_all_facts(); return; }` の中の `self.paint.stale_all_facts();` の行と、その上のコメント行を消す（切り離された subtree の facts は、それが付いた時に次の手順で古いとする）。
     - `if self.tree.get(id).is_some_and(|n| n.kind() == NodeType::Overlay) { // An overlay is a layer root of its own. self.paint.stale_all_facts(); }` の塊全体を次にする。
       ```rust
       // The attached subtree (built while detached, an overlay, or moved)
       // derives its facts again, its ancestors as they change.
       self.stale_subtree_facts(id);
       ```
  5. `remove_now` の `if was_overlay || (had_root && self.tree.root().is_none()) { self.paint.stale_all_facts(); }` の塊と、`had_root`・`was_overlay` の変数を消す（消えた node の facts は `forget_fact(gone)` で消えている。overlay も根も親の facts に入らない）。
  6. `rg -n "stale_all_facts|forget_flat\(\)" crates/kivact-engine/src` が `src/model/lifecycle.rs`（新しい根）と `src/paint/state.rs`（定義）と `src/paint/list/self_check.rs`（`StaleFacts::All` の代入）だけを返すことを確かめる。
- **消すもの**：`NodeTree::forget_flat`、`apply.rs` の `stale_all_facts()` の 4 か所の呼出し。
- **状態の閉じ方**：新しい状態は無い。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: overlay, compositor, layout, flows, view, transform_3d, effects_filters, drawables, interaction_tooltip, interaction_dialog, interaction_menu, ui_disclosure_surfaces`
- **完了条件**：手順 6 の `rg` のとおり。確認がすべて通る（debug build と `testing` の paint の self-check（`paint_self_check`）が facts の食い違いを報告しない）。

## E08：tree の変更記録（付属書 C 向け）

- **wave / lane / 前提**：W4 / E1 / E07
- **所有**：`src/contract/devtools.rs`、`src/contract.rs`、`src/model/events.rs`、`src/model/lifecycle.rs`、`src/host/write/reader.rs`、`src/host/write/writer.rs`、`src/host/write/typed.rs`
- **目的**：DevTools が tree を全部比べずに済むよう、client が書いた node と消えた node を、記録中だけ集める（設計書 7.2 Q3）。
- **手順**：
  1. `src/contract/devtools.rs` に次を足し、`src/contract.rs` の `pub use devtools::{ ... };` の並びに `TreeChanges` を足す。
     ```rust
     /// The nodes the client created or wrote (props, scenes, children) and
     /// the nodes destroyed since the last `Reader::take_tree_changes`, while
     /// `Reader::record_tree_changes` is on. Each list is in (window, id) order; a node
     /// destroyed after a write is only in `removed`.
     #[derive(Clone, Debug, Default, PartialEq, Eq)]
     pub struct TreeChanges {
         pub upserted: Vec<(kivact_types::WindowId, kivact_types::NodeId)>,
         pub removed: Vec<(kivact_types::WindowId, kivact_types::NodeId)>,
     }
     ```
  2. `src/model/events.rs`：`struct EngineLogState` に `pub(crate) tree: TreeJournal,` を足し、次の型を足す。
     ```rust
     /// The tree changes the DevTools records (see `contract::TreeChanges`).
     #[derive(Default)]
     pub(crate) enum TreeJournal {
         #[default]
         Off,
         Recording {
             upserted: BTreeSet<(WindowId, NodeId)>,
             removed: BTreeSet<(WindowId, NodeId)>,
         },
     }
     ```
     `impl EngineLog` に次を足す（`BTreeSet`、`NodeId`、`WindowId` の `use` を足す）。
     ```rust
     /// Start (keeping what is recorded) or stop recording the tree changes.
     pub(crate) fn record_tree(&self, on: bool) {
         let mut state = self.0.borrow_mut();
         let recording = matches!(state.tree, TreeJournal::Recording { .. });
         if on && !recording {
             state.tree = TreeJournal::Recording {
                 upserted: BTreeSet::new(),
                 removed: BTreeSet::new(),
             };
         } else if !on {
             state.tree = TreeJournal::Off;
         }
     }

     /// The client created or wrote these nodes of `window` (nothing unless recording).
     pub(crate) fn tree_upserted(&self, window: WindowId, ids: impl IntoIterator<Item = NodeId>) {
         if let TreeJournal::Recording { upserted, .. } = &mut self.0.borrow_mut().tree {
             upserted.extend(ids.into_iter().map(|id| (window, id)));
         }
     }

     /// `id` of `window` was destroyed (nothing unless recording).
     pub(crate) fn tree_removed(&self, window: WindowId, id: NodeId) {
         if let TreeJournal::Recording { upserted, removed } = &mut self.0.borrow_mut().tree {
             upserted.remove(&(window, id));
             removed.insert((window, id));
         }
     }

     /// The tree changes recorded since the last call (empty when not recording).
     pub(crate) fn take_tree(&self) -> crate::contract::TreeChanges {
         match &mut self.0.borrow_mut().tree {
             TreeJournal::Off => crate::contract::TreeChanges::default(),
             TreeJournal::Recording { upserted, removed } => crate::contract::TreeChanges {
                 upserted: std::mem::take(upserted).into_iter().collect(),
                 removed: std::mem::take(removed).into_iter().collect(),
             },
         }
     }
     ```
  3. `src/model/lifecycle.rs`：`remove_slots` が返す `Vec<NodeId>` を返す前に、各 id について `self.log.tree_removed(self.window, id)` を呼ぶ。`tree.remove_with(...)` の結果を `let gone = tree.remove_with(...);` に受け、分解の後（`slots` などの借用が終わってから）`for id in &gone { self.log.tree_removed(self.window, *id); } gone` で返す。分解の `let Self { tree, layout, slots, .. } = self;` は block `{ ... }` に入れ、block の値を `gone` にする。
  4. `src/host/write/writer.rs`：
     - `create`：`writes.create(id);` の後に `model.log.tree_upserted(model.window, [id]);`。
     - `insert`：`m.insert(parent, id, before, true); writes.structural(id);` の後に `m.log.tree_upserted(m.window, [parent, id]);`。
     - `move_before`：閉包の中の `model.move_before(id, before);` の前に `let parent = model.tree().get(id).and_then(|n| model.tree().parent_id(n));` を置き、`writes.structural(id);` の後に `model.log.tree_upserted(model.window, parent.into_iter().chain([id]));`。
     - `remove_with_exit`：閉包の中の `let removed = model.tree().subtree(id);` の前に `let parent = model.tree().get(id).and_then(|n| model.tree().parent_id(n));` を置き、`writes.structural(id);` の後に `model.log.tree_upserted(model.window, parent);`（`Option<NodeId>` は `IntoIterator`）。
     - `set_scene`：`writes.scene(id);` の後に `model.log.tree_upserted(model.window, [id]);`。
  5. `src/host/write/typed.rs`：`fn record` の `if changed { writes.field(id); }` を `if changed { writes.field(id); model.log.tree_upserted(model.window, [id]); }` にする。
  6. `src/host/write/reader.rs`：`take_engine_records` の後に次を足す。属性と前の行の注釈（W1 の後の internal-contract の書き方）は `take_engine_records` のものをそのまま写し、文だけ `the core's DevTools reads the tree changes` にする。
     ```rust
     #[doc(hidden)]
     /// Start (keeping what is recorded) or stop recording the tree changes.
     pub fn record_tree_changes(&self, on: bool) {
         self.app.borrow().log.record_tree(on);
     }

     #[doc(hidden)]
     /// The tree changes recorded since the last call (empty when not recording).
     pub fn take_tree_changes(&self) -> crate::contract::TreeChanges {
         self.app.borrow().log.take_tree()
     }
     ```
- **消すもの**：なし。
- **状態の閉じ方**：`TreeJournal` は enum で、記録中だけ集合を持つ（S-2）。`EngineLogState` は `Log<T>` と `TreeJournal` の enum だけを持ち対象外。
- **試験**：足さない（付属書 C が DevTools の試験でこの API を使う）。
- **確認（tier a）**：`ENGINE-A`、`E2E: lifecycle, disposal, flows`、`dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- devtools::`
- **完了条件**：`rg -n "pub fn take_tree_changes|pub fn record_tree_changes" crates/kivact-engine/src/host/write/reader.rs` が 2 行。確認がすべて通る。

## E09：damage を revision で引く、host だけの editing の型を `host` module へ

- **wave / lane / 前提**：W4 / E1 / E08
- **所有**：`crates/kivact-editing/src/lib.rs`、`crates/kivact-editing/src/edit_state.rs`、`crates/kivact-editing/src/editor.rs`、`crates/kivact-editing/src/change.rs`、`crates/kivact-editing/src/host.rs`（新設）、`src/text_input.rs`、`src/text_input/edit_lines.rs`、`src/text_input/refresh.rs`、`src/text_input/source.rs`、`src/text_input/chunks.rs`、`src/measure/editor.rs`、`src/measure/input.rs`、`src/model/keys.rs`、`src/model/edit/inputs.rs`、`src/model/edit/state.rs`
- **目的**：editing の変化の範囲（damage）を `EditState` が revision ごとの短い記録として持ち、どの line cache も自分の revision から問える（`damage_since`）。三つの持ち主（`EditState.damage`、`EditLines.damage`、`flush_damage`）を消し、幅を測る cache（`measure_lines`）にも damage を渡す（EN-12）。`Editor`・`Journal`・`Incoming`・`Taken`・`Damage` を `kivact_editing::host`（`#[doc(hidden)]`）に移す（EN-32）。折返しの行の splice は 要設計判断 D-3 のため本 unit ではしない。
- **手順**：
  1. `crates/kivact-editing/src/edit_state.rs`：
     - `use std::collections::VecDeque;` を足し、`const DAMAGE_LOG: usize = 64;` を足す（doc：`/// How many changes of the text \`EditState::damage_since\` can fold.`）。
     - `struct EditState` の `damage: Option<Damage>,` を `damage: VecDeque<Damage>,` にし、doc を `/// The last changes of the text, oldest first, each from the revision before it (\`Damage::base\`); at most \`DAMAGE_LOG\`.` にする。`EditState::new` の `damage: None` を `damage: VecDeque::new()` にする。
     - `fn damaged(&mut self, before: u64, range: Range<usize>, len: usize)` の本体を次にする。
       ```rust
       if self.damage.len() == DAMAGE_LOG {
           self.damage.pop_front();
       }
       self.damage.push_back(Damage {
           base: before,
           start: range.start,
           old_end: range.end,
           new_end: range.start + len,
       });
       ```
     - `pub(crate) fn take_damage(&mut self) -> Option<Damage>` を消し、次を足す。
       ```rust
       /// What changed in the text since revision `revision`, folded into one
       /// range; `None` when the record does not reach back to it (the
       /// caller splits the whole text again). Equal revisions give `None`
       /// too: nothing to splice.
       pub fn damage_since(&self, revision: u64) -> Option<Damage> {
           let first = self.damage.iter().position(|d| d.base == revision)?;
           let mut changes = self.damage.iter().skip(first);
           let mut folded = *changes.next()?;
           for d in changes {
               folded = folded.then(d.start..d.old_end, d.new_end - d.start);
           }
           Some(folded)
       }
       ```
     - text を変える method（`rg -n "self\.text\b" crates/kivact-editing/src/edit_state.rs` で一覧する）のうち、`splice`（`damaged` を呼ぶ）と `set_controlled`（`damaged` を呼ぶ）以外で text を書くもの（`take` の `Taken::Whole`、`revert`、`truncate`、`adopt`）を一つずつ読み、`self.splice(...)` を通らずに `self.text` を書いている所には、書いた直後に `self.damage.clear();` を置く（記録が届かず、cache は全体を作り直す）。
  2. `crates/kivact-editing/src/editor.rs`：`pub fn take_damage(&mut self) -> Option<Damage>` を消す。
  3. `crates/kivact-editing/src/host.rs` を新しく作る。
     ```rust
     //! Role: The editing machinery only the native host uses: the editor, its journal and the damage record.

     pub use crate::change::Damage;
     pub use crate::edit_state::Taken;
     pub use crate::editor::Editor;
     pub use crate::journal::{Incoming, Journal};
     ```
  4. `crates/kivact-editing/src/lib.rs`：
     - `pub use change::{Damage, TextChange};` を `pub use change::TextChange;` に、`pub use edit_state::{EditState, Taken};` を `pub use edit_state::EditState;` にし、`pub use editor::Editor;` と `pub use journal::{Incoming, Journal};` を消す。
     - module の宣言の後に次を足す。注釈の行は W1 の後の internal-contract の書き方に合わせる（`rg -n "doc\(hidden\)" -B1 crates/kivact-engine/src/host/write/reader.rs | head -4` の形を写し、文は `the native host (kivact-engine) drives the editor through these`）。
       ```rust
       #[doc(hidden)]
       pub mod host;
       ```
     - crate の doc の項目を直す：「such keys report `changed: false`, so the host sends no `TextChanged`」を「the host sends `TextChanged` only when the text changed」に、「Every change of the text, recorded or not, joins the state's damage ... (`text_lines`).」を「Every change of the text is kept in a short record by revision, so each line cache asks what changed since its own revision (`EditState::damage_since`).」にする。
  5. engine の `use` の置換：
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-engine/src" || exit 1
     perl -pi -e 's/use kivact_editing::\{EditState, Editor, Journal\};/use kivact_editing::EditState;\nuse kivact_editing::host::{Editor, Journal};/' measure/editor.rs
     perl -pi -e 's/use kivact_editing::\{EditState, Editor, Taken, TextSession\};/use kivact_editing::host::{Editor, Taken};\nuse kivact_editing::{EditState, TextSession};/' model/edit/inputs.rs
     perl -pi -e 's/use kivact_editing::Journal;/use kivact_editing::host::Journal;/' model/keys.rs
     rg -l 'kivact_editing::Damage' . | xargs perl -pi -e 's/kivact_editing::Damage/kivact_editing::host::Damage/g'
     cd "$ROOT" || exit 1
     ```
     残りは `rg -n "kivact_editing::(Editor|Journal|Incoming|Taken|Damage)\b" crates` で探し、`kivact_editing::host::` に直す。
  6. `src/text_input/edit_lines.rs`：`EditLines` の `pub(crate) damage: Option<kivact_editing::host::Damage>,` とその上の注釈、`EditLines::new` の `damage: None,`、`pub(crate) fn hold_damage` を消す。
  7. `src/measure/editor.rs`：`pub(crate) fn flush_damage(&mut self)` を消す。
  8. `rg -n "flush_damage" crates/kivact-engine/src` の 4 か所（`src/model/keys.rs` の `edit_editor_with` と `commit_composition`、`src/model/edit/inputs.rs` の `editing_changed` と `controlled_value`）の、`if let Some(editor) = self.editor_node_mut(id) { editor.flush_damage(); }` の塊（またはそれに当たる呼出し）を消す。
  9. `src/measure/input.rs` の `refresh_lines`：`damage: if measurement { None } else { slot.as_mut().and_then(|lines| lines.damage.take()) },` を次にする（どちらの cache も自分の revision から問う）。
     ```rust
     damage: slot.as_ref().and_then(|lines| edit.damage_since(lines.revision)),
     ```
  10. `src/text_input.rs` の doc の「(`Editor::take_damage`)」を「(`EditState::damage_since`, asked with the lines' own revision)」にする。
  11. `rg -n "take_damage|hold_damage|flush_damage|\.damage\.take\(\)" crates/kivact-engine/src crates/kivact-editing/src` が空であることを確かめる。
- **消すもの**：`EditState::take_damage`、`Editor::take_damage`、`EditLines::damage`、`EditLines::hold_damage`、`EditorNode::flush_damage`、crate root の `Damage`・`Taken`・`Editor`・`Incoming`・`Journal` の再公開。
- **状態の閉じ方**：`EditState` は `Option<Log>` と `VecDeque` の記録を持つ。`damage` は数の field ではなく、`log: Option<Log>` と合わせても対象は一つで印は要らない。`EditLines` は field が一つ減る（W1 の印があれば、その文から damage の説明を消す）。
- **試験**：足さない。`crates/kivact-editing` に `#[cfg(test)]` があり `take_damage` を使っていれば、`damage_since` で同じことを確かめる形に書き直す（期待値は同じ）。
- **確認（tier a）**：`EDITING-A`、`ENGINE-A`、`cargo check -p kivact-style --all-targets`、`E2E: text_editing, custom_text_field, input_platform, interaction_edit, ui_chat_code, ui_field`
- **完了条件**：手順 11 の `rg` が空。確認がすべて通る。

## E30：入力の出所（`InputOrigin`、付属書 C の E-REQ-6）

- **wave / lane / 前提**：W4 / E1 / E09
- **所有**：`src/contract/routed.rs`、`src/contract.rs`、`src/model/frame/state.rs`、`src/model/app_model.rs`、`src/model/keys.rs`、`src/model/pointer/machine.rs`、`src/model/pointer/shell.rs`、`src/host/events.rs`、`src/host/write/writer.rs`、`crates/kivact-test/src/screen/host_sync_tests.rs`
- **目的**：engine が client へ渡す入力の event（`Routed<T>`）に、それが platform の入力か、DevTools の `inject_input`（`HostRequest::Inject`）で注入された入力かを持たせる（RC-28、付属書 C の E-REQ-6。C06 が使う）。`TreeChanges` の window 付きの形（INC-02）は同じ lane の E08 が作ってある。
- **手順**：
  1. `src/contract/routed.rs`：次の型を足し、`struct Routed<T>` の最後に field を足す。`src/contract.rs` の `routed` の再公開の並びに `InputOrigin` を足す。
     ```rust
     /// Where an input event came from.
     #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
     pub enum InputOrigin {
         /// The platform's input (or the test driver's raw input).
         #[default]
         Platform,
         /// Injected by the request (the DevTools' `inject_input`, `HostRequest::Inject`).
         Injected(kivact_types::RequestId),
     }
     ```
     ```rust
     /// Whether the input came from the platform or was injected.
     pub origin: InputOrigin,
     ```
  2. `src/model/frame/state.rs`：`struct Frame` に field `pub(super) origin: InputOrigin,`（doc：`/// Where the shell event being handled came from (\`Frame::set_origin\`).`）を足し、`Default` で `InputOrigin::Platform` にする。method を足す。
     ```rust
     /// The shell event being handled came from `origin`.
     pub(crate) fn set_origin(&mut self, origin: crate::contract::InputOrigin) {
         self.origin = origin;
     }

     /// Where the shell event being handled came from.
     pub(crate) fn origin(&self) -> crate::contract::InputOrigin {
         self.origin
     }
     ```
  3. `Routed { .. }` の literal に `origin` を足す：engine の 7 か所（`src/model/keys.rs` の二つの `HostEvent::Key(crate::contract::Routed {`、`src/model/pointer/machine.rs` の `pointer_sample` の `crate::contract::Routed {` と wheel の `HostEvent::Wheel(crate::contract::Routed {`、`src/model/pointer/shell.rs` の三つの `HostEvent::File(crate::contract::Routed {`）は `origin: self.frame.origin(),`。`crates/kivact-test/src/screen/host_sync_tests.rs` の 2 か所は `origin: kivact_engine::contract::InputOrigin::Platform,`。`rg -n "Routed \{" crates` の結果がすべて `origin` を書いていることを確かめる。
  4. `src/model/app_model.rs`：`struct AppModel` に次を足し（`new` で `BTreeMap::new()`）、`remove_window` に `self.injecting.remove(&id);` を足す。
     ```rust
     /// The injections queued for each window, oldest first: the shell
     /// events of a window come from the first until the shell reports it
     /// delivered (`ShellEvent::Injected`).
     pub(crate) injecting: BTreeMap<WindowId, std::collections::VecDeque<RequestId>>,
     ```
  5. `src/host/write/writer.rs` の `Writer::effect` の `HostRequest::Inject { window, inputs } => { ... }` の腕：model の `effects` に積む行の後に `injecting.entry(window).or_default().push_back(request);` を足す（腕の先頭の `let AppModel { windows, writes, .. } = &mut *app;` の分解に `injecting` を足す）。
  6. `src/host/events.rs`：
     - `handle_window` の E01 の block（`app.set_now(id, now);` と `stops_smooth_scrolls`）を次にする。
       ```rust
       {
           let mut app = self.app.borrow_mut();
           app.set_now(id, now);
           let origin = app
               .injecting
               .get(&id)
               .and_then(|queue| queue.front().copied())
               .map_or(InputOrigin::Platform, InputOrigin::Injected);
           if let Some(model) = app.model_mut(id) {
               model.frame.set_origin(origin);
               if stops_smooth_scrolls(&event) {
                   model.frame.stop_smooth_scrolls();
               }
           }
       }
       ```
     - `ShellEvent::Injected { request } => { ... }` の腕の先頭に次を足す（届いた注入を列から外す）。
       ```rust
       {
           let mut app = self.app.borrow_mut();
           if let Some(queue) = app.injecting.get_mut(&id)
               && queue.front() == Some(&request)
           {
               queue.pop_front();
           }
           app.injecting.retain(|_, queue| !queue.is_empty());
       }
       ```
     - `use crate::contract::InputOrigin;` を足す。
- **消すもの**：なし。
- **状態の閉じ方**：`InputOrigin` は enum。`Frame` は enum の field が一つ増えるだけ（E01 の印のまま）。`AppModel` は map の field が一つ増える（数・bool・Option ではない）。
- **試験**：足さない（C06 の `devtools::a_devtools_client_inspects_drives_and_reviews_a_test_app` が注入の出所を確かめる）。
- **確認（tier a）**：`ENGINE-A`、`cargo check -p kivact-test --all-targets`、`dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-test`、`E2E: pointer, keyboard, text_editing, input_platform, touch_scroll`、`dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- devtools::`
- **完了条件**：`rg -n "pub origin: InputOrigin" crates/kivact-engine/src/contract/routed.rs` が 1 行。手順 3 の確かめのとおり。確認がすべて通る。

## E10：caret が動いた text input だけを scroll する

- **wave / lane / 前提**：W4 / E2 / なし
- **所有**：`src/model/changes.rs`、`src/model/interact/sync.rs`、`src/model/interact/text_geometry.rs`、`src/model/interact/caret.rs`、`src/model/interact/geometry.rs`、`src/model/aggregate.rs`、`src/host/pump.rs`
- **目的**：caret が動いた・大きさが変わった・text が変わった editor だけを `Dirty::CARET` で印し、`settle` の最後に一度だけ scroll する。`present` と `caret()` からは呼ばない（EN-07）。
- **手順**：
  1. `src/model/changes.rs`：
     - `impl Dirty` に `pub(crate) const CARET: Self = Self(1 << 8);` を足す（doc：`/// A text input's caret, text, size or style changed: it is scrolled to keep the caret in view (\`Model::scroll_inputs\`). Kept by \`dispatch\` until taken.`）。
     - `dispatch` の `dirty.retain(|id, d| { if d.contains(Dirty::MEASURE) { self.dirty.insert(*id, Dirty::MEASURE); d.remove(Dirty::MEASURE); } !d.is_empty() });` を次にする（MEASURE と CARET を残す）。
       ```rust
       let kept_bits = Dirty::MEASURE.union(Dirty::CARET);
       dirty.retain(|id, d| {
           let kept = Dirty(d.0 & kept_bits.0);
           if !kept.is_empty() {
               self.dirty.insert(*id, kept);
               d.remove(kept);
           }
           !d.is_empty()
       });
       ```
  2. `src/model/aggregate.rs` の `repaint_text`：`self.changes.mark(id, super::changes::Dirty::REDRAW);` を `self.changes.mark(id, super::changes::Dirty::REDRAW.union(super::changes::Dirty::CARET));` にする。
  3. `src/model/interact/sync.rs`：
     - `remeasure`：`self.changes.mark(id, Dirty::MEASURE);` の次に次を足す。
       ```rust
       if self
           .tree
           .get(id)
           .is_some_and(|n| self.slots.editors[n.index.0].is_some())
       {
           self.changes.mark(id, Dirty::CARET);
       }
       ```
     - `set_viewport` の `if self.viewport != window {` の中の最後に `for id in self.input_ids() { self.changes.mark(id, Dirty::CARET); }` を足す。
     - `settle` を次にする（中の layout の塊は今のまま）。
       ```rust
       pub(crate) fn settle(&mut self, engine: &mut arvane::Engine) -> Settled {
           self.sync();
           let settled = if self.needs_layout() {
               // (the existing layout block, ending with `self.report_images(&images);`)
               Settled::Updated
           } else {
               Settled::Kept
           };
           // The carets that moved are kept in view once the boxes are laid out.
           self.scroll_inputs(engine);
           settled
       }
       ```
  4. `src/model/interact/geometry.rs` の `layout`：`for change in &geometry { self.a11y.moved(change.id); ... }` の loop の中の先頭に次を足す。
     ```rust
     if self
         .tree
         .get(change.id)
         .is_some_and(|n| self.slots.editors[n.index.0].is_some())
     {
         self.changes.mark(change.id, Dirty::CARET);
     }
     ```
     （`Dirty` の `use` が無ければ足す。）
  5. `src/model/interact/text_geometry.rs` の `scroll_inputs` を次にする（中の幾何の計算は今のまま）。
     ```rust
     /// Scroll the text inputs whose caret, text or box changed
     /// (`Dirty::CARET`) so the caret stays inside the field: past the
     /// right edge less 2 px, or at the left edge.
     pub(crate) fn scroll_inputs(&mut self, engine: &mut arvane::Engine) {
         let mut moved = false;
         for id in self.changes.take(Dirty::CARET) {
             if !self.edits_input(id) {
                 continue;
             }
             // (the existing loop body, with `self.invalidate_paint();` replaced by
             //  `self.changes.mark(id, Dirty::REDRAW); moved = true;`)
         }
         if moved {
             self.sync();
         }
     }
     ```
     `edits_input` は次の method として同じファイルに足す。
     ```rust
     /// Whether `id` is a live text input with an editing state.
     fn edits_input(&self, id: NodeId) -> bool {
         self.tree
             .get(id)
             .is_some_and(|n| self.slots.editors[n.index.0].is_some())
     }
     ```
  6. `src/host/pump.rs` の `present`：`model.scroll_inputs(&mut self.engine.borrow_mut());` の行を消す。
  7. `src/model/interact/caret.rs` の `caret`：`self.scroll_inputs(engine);` の行を消す。
  8. `rg -n "scroll_inputs\(" crates/kivact-engine/src` が `sync.rs`（`settle`）と `text_geometry.rs`（定義）だけを返すことを確かめる。
- **消すもの**：`present` と `caret` の中の `scroll_inputs` の呼出し、`scroll_inputs` の中の `invalidate_paint()` の呼出し。
- **状態の閉じ方**：`Dirty` は bit の集合で対象外。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: text_editing, custom_text_field, input_platform, launch_real_window, transform_3d, components`、`E2E: ui_field, ui_chat_code, interaction_edit`
- **完了条件**：手順 8 の `rg` のとおり。確認がすべて通る。

## E11：`Look` と `SubtreeFacts` の bool を bit の集合にする

- **wave / lane / 前提**：W4 / E2 / E10
- **所有**：`src/layer.rs`、`src/paint/list/look.rs`、`src/paint/list/walk.rs`、`src/model/interact/hit.rs`、`src/model/interact/geometry.rs`
- **目的**：`Look` の 14 個の bool を `LookFlags(u16)` に、`SubtreeFacts` の 5 個の bool を `SubtreeFacts(u8)` の bit にし、node あたりの大きさを減らす（EN-23、EN-29）。MEM-6 の上限を締める。
- **手順**：
  1. `src/layer.rs`：
     - 次の型を足す。
       ```rust
       /// The yes/no facts of a look, one bit each (the old field names are its methods).
       #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
       pub(crate) struct LookFlags(u16);

       impl LookFlags {
           pub(crate) const GROUPED: Self = Self(1);
           pub(crate) const CLIP: Self = Self(1 << 1);
           pub(crate) const ROUNDED: Self = Self(1 << 2);
           pub(crate) const FILTERED: Self = Self(1 << 3);
           pub(crate) const MASKED: Self = Self(1 << 4);
           pub(crate) const BACKDROPPED: Self = Self(1 << 5);
           pub(crate) const BLENDED: Self = Self(1 << 6);
           pub(crate) const ISOLATED: Self = Self(1 << 7);
           pub(crate) const PRESERVES: Self = Self(1 << 8);
           pub(crate) const FLAT: Self = Self(1 << 9);
           pub(crate) const TRANSFORMED: Self = Self(1 << 10);
           pub(crate) const PERSPECTIVE: Self = Self(1 << 11);
           pub(crate) const BACKFACE_HIDDEN: Self = Self(1 << 12);
           pub(crate) const SHADER: Self = Self(1 << 13);

           /// Whether `flag` is set.
           pub(crate) const fn contains(self, flag: Self) -> bool {
               self.0 & flag.0 == flag.0
           }

           /// These flags with `flag` set when `on`.
           pub(crate) const fn with(self, flag: Self, on: bool) -> Self {
               if on { Self(self.0 | flag.0) } else { self }
           }
       }
       ```
     - `struct Look` の 14 個の bool の field（`grouped`、`clip`、`rounded`、`filtered`、`masked`、`backdropped`、`blended`、`isolated`、`preserves`、`flat`、`transformed`、`perspective`、`backface_hidden`、`shader`）と各 field の doc・印を消し、`pub(crate) flags: LookFlags,` を一つ足す（doc：`/// Its yes/no facts (see the methods named after them).`）。`opacity`、`overflow`、`border`、`fx`、`items`、`will_change` は残す。
     - `impl Look` に、消した field と同じ名前の method を 14 個足す。形は全部同じ：
       ```rust
       /// Whether its group composites its subtree (`effects::grouped`).
       pub(crate) fn grouped(&self) -> bool {
           self.flags.contains(LookFlags::GROUPED)
       }
       ```
       （各 method の doc は消した field の doc の文を使う。）
     - `Look::of` の struct literal の 14 個の bool の行を消し、`flags: LookFlags::default().with(LookFlags::GROUPED, effects::grouped(p)).with(LookFlags::CLIP, p.clip)...` の形で、消した行と同じ式を同じ順に `.with(..)` で並べた一つの式を `flags:` に入れる。
     - `const _: () = assert!(std::mem::size_of::<Look>() <= 72, ...)` を次にする。
       ```rust
       const _: () = assert!(
           std::mem::size_of::<Look>() <= 64,
           "Look grew: box the new rare field"
       );
       ```
       （根拠：f64 が 3 つ（24）＋ `Edges<f64>`（32）＋ `items: u8`（1）＋ `will_change`（1 byte の enum。今の 72 の上限が成り立つことから 1 byte）＋ `LookFlags`（2）＝ 60、整列 8 で 64。）
     - `struct SubtreeFacts` を次にする（5 個の bool の field と印を消す）。
       ```rust
       /// What a subtree holds that decides its layers, one bit each (the old
       /// field names are its methods).
       #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
       pub(crate) struct SubtreeFacts(u8);

       impl SubtreeFacts {
           pub(crate) const LAYER: Self = Self(1);
           pub(crate) const HAS_LAYER: Self = Self(1 << 1);
           pub(crate) const HAS_ITEM: Self = Self(1 << 2);
           pub(crate) const HAS_BACKDROP: Self = Self(1 << 3);
           pub(crate) const HAS_BLEND: Self = Self(1 << 4);

           /// These facts with `fact` set when `on`.
           pub(crate) const fn with(self, fact: Self, on: bool) -> Self {
               if on { Self(self.0 | fact.0) } else { self }
           }

           pub(crate) const fn layer(self) -> bool {
               self.0 & Self::LAYER.0 != 0
           }
           pub(crate) const fn has_layer(self) -> bool {
               self.0 & Self::HAS_LAYER.0 != 0
           }
           pub(crate) const fn has_item(self) -> bool {
               self.0 & Self::HAS_ITEM.0 != 0
           }
           pub(crate) const fn has_backdrop(self) -> bool {
               self.0 & Self::HAS_BACKDROP.0 != 0
           }
           pub(crate) const fn has_blend(self) -> bool {
               self.0 & Self::HAS_BLEND.0 != 0
           }
       }

       const _: () = assert!(
           std::mem::size_of::<SubtreeFacts>() <= 1,
           "SubtreeFacts grew: it is a set of bits"
       );
       ```
     - layer.rs の中の `SubtreeFacts { layer: a, has_layer: b, has_item: c, has_backdrop: d, has_blend: e }` の literal は `SubtreeFacts::default().with(SubtreeFacts::LAYER, a).with(SubtreeFacts::HAS_LAYER, b)...` にし、`x.layer` などの読みは `x.layer()` などにする。`Look` の field の読み `x.grouped` などは `x.grouped()` などにする。
  2. `src/paint/list/look.rs` の `stacking`：tuple を次にする（14 個の bool は `flags` 一つにまとまる。意味は同じ）。
     ```rust
     (
         self.opacity,
         self.overflow,
         self.border,
         self.fx,
         self.items,
         self.will_change,
         self.flags,
     )
     ```
  3. `cargo check -p kivact-engine --all-targets --features testing` の error を、`src/paint/list/walk.rs`、`src/model/interact/hit.rs`、`src/model/interact/geometry.rs` の中で手順 1 と同じ規則（field の読み → 同名の method の呼出し）で直す。他のファイルに error が出たら、そのファイルも同じ規則で直し、本 unit の所有に足したことを完了の報告に書く。
- **消すもの**：`Look` の 14 個の bool の field、`SubtreeFacts` の 5 個の bool の field と、その印（W1 の後に残っていれば）。
- **状態の閉じ方**：`LookFlags`・`SubtreeFacts` は bit の集合で、`Key`/`Bits` と同じく値の型（`&mut self` の method が無い）で対象外。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: compositor, effects_filters, transform_3d, transform, drawables, canvas, style_paint, paint, borders_backgrounds, shaders`
- **完了条件**：`rg -n "pub\(crate\) (grouped|clip|rounded|filtered|masked|backdropped|blended|isolated|preserves|flat|transformed|perspective|backface_hidden|shader): bool" crates/kivact-engine/src/layer.rs` が空。確認がすべて通る。

## E31：core が写していた engine の事実を engine が答える（付属書 C の E-REQ-1〜4）

- **wave / lane / 前提**：W4b / E / E30（W4 の E1 の最後）
- **所有**：`src/host/write/reader.rs`、`src/host/write/writer.rs`、`src/host/write/typed.rs`、`src/contract.rs`、`src/contract/events.rs`、`src/model/app_model.rs`、`src/model/semantics.rs`、`src/host/pump.rs`、`crates/kivact-core/src/launch/runtime.rs`（`deliver` の 1 行だけ）、`crates/kivact-core/src/client/dispatch/delivery.rs`（`HostEvent::NodeFlagsChanged` を無視する 1 腕だけ）
- **目的**：付属書 C 4.2 の表の署名のとおりに、(1) client がまだ受け取っていない `TextChanged` がある node への InputValue の echo を engine が書かない（E-REQ-1）、(2) node の最後の layout の報告を engine が答える（E-REQ-2）、(3) 見張っている node の可視と live region の事実を engine が commit ごとに比べて知らせる（E-REQ-3）、(4) font の版を engine が数える（E-REQ-4）。core の写し（`TextEditsAhead`、`layout_seeds`、可視の notifier、font の再列挙）は C07（W5a）が消す。
- **手順**：
  1. `src/contract/events.rs` に次の型と `HostEvent` の variant を足し、`src/contract.rs` の `events` の再公開の並びに `LayoutReport` と `NodeFlags` を足す。
     ```rust
     /// The last layout report of a node: the values of its last `HostEvent::LayoutChanged`.
     #[derive(Clone, Copy, Debug, PartialEq)]
     pub struct LayoutReport {
         /// Its window-space rect, as drawn.
         pub rect: kivact_types::Rect,
         /// Its window layout box (no transform), once laid out.
         pub layout_rect: Option<kivact_types::Rect>,
         /// Whether its text is cut off.
         pub text_truncated: bool,
     }

     /// The visibility facts of a node, its ancestors included.
     #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
     pub struct NodeFlags {
         /// No node from it up to the root is hidden by its `Hidden` input flag.
         pub visible: bool,
         /// It or an ancestor is a live region (an alert, status or log role, or a polite or assertive live region).
         pub in_live_region: bool,
     }
     ```
     `HostEvent` に（`LayoutChanged` の次に）：
     ```rust
     /// The visibility facts of a watched node (`Writer::watch_flags`) changed in a commit.
     NodeFlagsChanged {
         node: kivact_types::NodeId,
         flags: NodeFlags,
     },
     ```
     engine の中で `HostEvent` を `match` していて新しい variant で compile が止まる所は、何もしない腕（`HostEvent::NodeFlagsChanged { .. } => {}`、またはその `match` の既存の「その他」の腕に足す）にする。
  2. `src/model/semantics.rs` に次を足す。
     ```rust
     impl Model {
         /// The visibility facts of `id` (`contract::NodeFlags`); a missing node is neither.
         pub(crate) fn node_flags(&self, id: NodeId) -> crate::contract::NodeFlags {
             let Some(start) = self.tree.nodes.index_of(id) else {
                 return crate::contract::NodeFlags::default();
             };
             let mut flags = crate::contract::NodeFlags {
                 visible: true,
                 in_live_region: false,
             };
             for (_, n) in self.tree.ancestors(start) {
                 if n.properties.input.hidden == Some(true) {
                     flags.visible = false;
                 }
                 if n.properties.semantics.as_deref().is_some_and(is_live_region) {
                     flags.in_live_region = true;
                 }
             }
             flags
         }
     }

     /// Whether `semantics` make a live region.
     fn is_live_region(semantics: &kivact_types::Semantics<kivact_types::FocusTarget>) -> bool {
         matches!(
             semantics.role,
             Some(kivact_types::Role::Alert | kivact_types::Role::Status | kivact_types::Role::Log)
         ) || matches!(
             semantics.live_region,
             Some(kivact_types::LiveRegion::Polite | kivact_types::LiveRegion::Assertive)
         )
     }
     ```
     （`input.hidden` の読みは W5a の E18 が `flags.get(InputFlag::Hidden)` に直す：E18 手順 6 の `rg` がこの行も拾う。）
  3. `src/model/app_model.rs`：`struct AppModel` に次の三つを足し（`new` で `HashMap::new()`、`BTreeMap::new()`、`0`）、method を足す。`AppModel` の印の文は変えない（W1 の後の印は `legacy:` で、CP1 の状態の台帳に載り、付属書 L が閉じ方を書く。台帳の `AppModel` の行に `text_ahead`・`font_generation` も挙げる。W4b の E31）。
     ```rust
     /// The `TextChanged` events of each node handed to the client in the
     /// running turn and not yet dispatched (`Writer::deliver`).
     pub(crate) text_ahead: HashMap<NodeId, u32>,
     /// The watched nodes and the visibility facts last reported of each.
     pub(crate) watched: BTreeMap<NodeId, crate::contract::NodeFlags>,
     /// How many times the font database changed.
     pub(crate) font_generation: u64,
     ```
     ```rust
     /// The client is handed `events`: their text edits hold back the
     /// input-value echoes of their nodes until each is dispatched.
     pub(crate) fn hold_text(&mut self, events: &[HostEvent]) {
         for event in events {
             if let HostEvent::TextChanged { node, .. } = event {
                 *self.text_ahead.entry(*node).or_default() += 1;
             }
         }
     }

     /// The client dispatches `event` now.
     pub(crate) fn delivered(&mut self, event: &HostEvent) {
         if let HostEvent::TextChanged { node, .. } = event
             && let Some(count) = self.text_ahead.get_mut(node)
         {
             *count -= 1;
             if *count == 0 {
                 self.text_ahead.remove(node);
             }
         }
     }

     /// Report the watched nodes whose visibility facts changed.
     pub(crate) fn report_flags(&mut self) {
         let ids: Vec<NodeId> = self.watched.keys().copied().collect();
         for id in ids {
             let Some(window) = self.window_of(id) else {
                 self.watched.remove(&id);
                 continue;
             };
             let flags = self.model(window).map(|m| m.node_flags(id)).unwrap_or_default();
             if self.watched.get(&id) != Some(&flags) {
                 self.watched.insert(id, flags);
                 self.push_event(HostEvent::NodeFlagsChanged { node: id, flags });
             }
         }
     }
     ```
  4. `src/host/pump.rs` の `turn`：`let output = self.client.turn(events, boundaries, now);` の前の行に `self.app.borrow_mut().hold_text(events);` を、後の行に `self.app.borrow_mut().text_ahead.clear();` を置く（turn が途中で止まっても次の turn に残さない。前の core の `TextEditsAhead` の `Drop` と同じ）。
  5. `src/host/write/typed.rs` の `fn write_input_value`：本体の最初に次を置く（E-REQ-1）。
     ```rust
     // An echo of a value the client has not seen the native edits after:
     // writing it would undo them.
     if self.app.borrow().text_ahead.contains_key(&id) {
         return false;
     }
     ```
  6. `src/host/write/writer.rs`：
     - 次の method を足す。
       ```rust
       /// The client dispatches `event` now (before its handlers run): a text
       /// edit stops holding back its node's input-value echoes.
       pub fn deliver(&self, event: &HostEvent) {
           self.app.borrow_mut().delivered(event);
       }

       /// Watch (or stop watching) the visibility facts of `id`: a commit that
       /// changes them queues `HostEvent::NodeFlagsChanged`.
       pub fn watch_flags(&self, id: NodeId, watch: bool) {
           let mut app = self.app.borrow_mut();
           if watch {
               let flags = app
                   .window_of(id)
                   .and_then(|window| app.model(window))
                   .map(|model| model.node_flags(id))
                   .unwrap_or_default();
               app.watched.insert(id, flags);
           } else {
               app.watched.remove(&id);
           }
       }
       ```
     - `commit`：`for model in app.windows.values_mut() { model.sync(); }`（またはその時点の同じ loop）の後、`revision` を返す前に `app.report_flags();` を置く。
     - `register_font`：`model.fonts_added();` を呼ぶ window の loop の後（同じ分岐の中）に `app.font_generation += 1;` を置く。
  7. `src/host/write/reader.rs` に次を足す（属性は同じファイルの他の `#[doc(hidden)]` の method に合わせる）。
     ```rust
     /// The last layout report of `id` (the values of its last `HostEvent::LayoutChanged`).
     pub fn layout_report(&self, id: NodeId) -> Option<crate::contract::LayoutReport> {
         self.with_model(id, |model| {
             let n = model.tree.get(id)?;
             let sent = n.sent.as_ref()?.layout.get()?;
             Some(crate::contract::LayoutReport {
                 rect: sent.0,
                 layout_rect: sent.3,
                 text_truncated: sent.2 == crate::tree::node::Clipping::Yes,
             })
         })
         .flatten()
     }

     /// The visibility facts of `id` now.
     pub fn node_flags(&self, id: NodeId) -> crate::contract::NodeFlags {
         self.with_model(id, |model| model.node_flags(id))
             .unwrap_or_default()
     }

     /// How many times the font database changed (the families change with it).
     pub fn font_generation(&self) -> u64 {
         self.app.borrow().font_generation
     }
     ```
     （`sent.0`・`sent.2`・`sent.3` は `tree::node::LayoutReport(Rect, Size, Clipping, Option<Rect>)` の field。）
  8. kivact-core：
     - `src/launch/runtime.rs` の turn の event の loop の `edits.deliver(event);` の次の行に `crate::view::state::ViewState::tree(&rt).engine().deliver(event);` を足す（C07 は `edits.deliver(event);` を消すが、この行は残す。4.2 節）。
     - `src/client/dispatch/delivery.rs` の `HostEvent` の `match` に `HostEvent::NodeFlagsChanged { .. } => {}` の腕を足す（C07 がこの腕を `env::node_flags_changed` を呼ぶ腕に置き換える）。
- **消すもの**：なし（core の写しは C07 が消す）。
- **状態の閉じ方**：`AppModel` は W1 の `legacy:` の印のまま（手順 3。付属書 L が閉じる）。`NodeFlags` の二つの bool は値（`&mut self` の method が無い）で対象外。
- **試験**：足さない（C07 が core の写しを消した後、`text_editing`・`node_ref`・`layout`・`launch_fonts` の E2E が engine の答えを使う）。この unit の時点では core の写しも残っているので、既存の E2E が通ればよい。
- **確認（tier a）**：
  ```
  ENGINE-A
  cargo check -p kivact-core --all-targets --features devtools
  cargo clippy -p kivact-core --all-targets -- -D warnings
  E2E: text_editing, custom_text_field, input_platform, node_ref, layout, launch_fonts, a11y_incremental, components
  E2E: ui_field, interaction_edit
  ```
- **完了条件**：`rg -n "pub fn (deliver|watch_flags|layout_report|node_flags|font_generation)" crates/kivact-engine/src/host/write` が 5 行。確認がすべて通る。

## E12：node ごとの draw cache（PERF-6）

- **wave / lane / 前提**：W5a / E2 / なし（W3b・W4・W4b の E unit は済んでいる）
- **所有**：`src/paint/state.rs`、`src/paint/frame_scene.rs`、`src/paint/canvas.rs`、`src/paint/snapshot.rs`、`src/paint/list/invalidation.rs`、`src/lower/scene_out.rs`、`src/model/aggregate.rs`、`src/model/states.rs`、`src/model/interact/sync.rs`、`src/model/interact/visual.rs`、`src/model/interact/geometry.rs`、`src/model/interact/hit.rs`、`src/model/scrollbar.rs`、`src/model/transform.rs`、`src/model/pointer/hover.rs`、`src/model/frame/ticks.rs`
- **目的**：list に載った node ごとに、前の frame の描画（sanitize した paint と、node 自身が出した `RenderCmd` の列）を slot の列に残し、描き直しの印（`Dirty::WRITTEN|REDRAW|SCROLLED`）が無く、置き場（origin・alpha・clip・scale）が同じなら lower し直さずに複写する。描画中の node だけが持ち、list から外れた node の分は捨てる（EN-03、設計書 7.3 Q4）。debug と `testing` の照合は、kept の paint を使わず node の今の paint と箱から別の出力に描き直して比べ、frame の出力・kept の drawing・layer の版には混ぜない（EA-01・EA-14、設計書 12 章）。`effective_paint` の clone を避ける借用版と、cursor だけを読む関数を足す。
- **手順**：
  1. `src/paint/state.rs` に次を足す（`use std::rc::Rc;` と `use std::collections::HashSet;` と、`RenderCmd`・`PaintProps` の `use` を足す。`RenderCmd` は 1.2 節の path）。
     ```rust
     /// What one listed node drew in the last frame it was drawn, kept while
     /// nothing it is drawn from changes (`paint::frame_scene`). Only listed
     /// nodes keep one; the list's rebuild drops the others (`Model::evict_draws`).
     #[derive(Debug)]
     pub(crate) struct NodeDraw {
         /// Where and at which scale it was drawn.
         pub(super) key: DrawKey,
         /// Its paint as drawn: state deltas, transition and layout-run radii
         /// applied, sanitized (`interact::visual::paint`).
         pub(super) paint: Rc<kivact_types::PaintProps>,
         pub(super) recorded: Recorded,
     }

     /// The place and scale a node's drawing was made for: a step with
     /// another one draws it again.
     #[derive(Clone, Copy, Debug, PartialEq)]
     pub(crate) struct DrawKey {
         pub(super) origin: Point,
         pub(super) alpha: f64,
         pub(super) clip: Rect,
         pub(super) scale: f64,
     }

     impl DrawKey {
         /// The key of `step` drawn in `pass`.
         pub(crate) fn of(step: &Step, pass: PaintPass) -> Self {
             DrawKey {
                 origin: step.origin,
                 alpha: step.alpha,
                 clip: step.clip,
                 scale: pass.scale,
             }
         }
     }

     /// What a node's drawing left to replay.
     #[derive(Debug)]
     pub(crate) enum Recorded {
         /// Its own commands: the box (between the outer and inner openings),
         /// the content (after the inner openings) and, once its close was
         /// drawn, the draw-over scene and scrollbars.
         Commands {
             boxed: Vec<RenderCmd>,
             content: Vec<RenderCmd>,
             over: Option<Vec<RenderCmd>>,
         },
         /// It opens layers or items, or draws pixels (images, patterns): it
         /// is lowered every frame (its paint is still kept).
         Live,
     }

     impl Recorded {
         /// The record of a box and a content part (`None`: a part could not be recorded).
         pub(crate) fn of(boxed: Option<Vec<RenderCmd>>, content: Option<Vec<RenderCmd>>) -> Self {
             match (boxed, content) {
                 (Some(boxed), Some(content)) => Recorded::Commands {
                     boxed,
                     content,
                     over: None,
                 },
                 _ => Recorded::Live,
             }
         }
     }
     ```
     `struct PaintPass` に field を足す：
     ```rust
     /// Whether the walk reads and keeps the nodes' drawings (the window's
     /// frame only; exports and snapshots draw afresh).
     pub(crate) cache: bool,
     ```
     `struct PaintState` に field を足し、`PaintState::new` で `draw_owners: RefCell::default(),` にする：
     ```rust
     /// The nodes (and their slots) that keep a drawing in `Slots::draws`.
     pub(crate) draw_owners: RefCell<HashSet<(NodeId, usize)>>,
     ```
  2. `src/model/aggregate.rs` の `struct Slots` に `pub(crate) draws: Vec<Cell<Option<Box<crate::paint::state::NodeDraw>>>>,` を足し（doc：`/// The drawing each listed node made in the last frame (\`paint::frame_scene\`).`）、`push` に `self.draws.push(Cell::new(None));`、`reset` に `self.draws[slot].set(None);` を足す。
  3. `src/lower/scene_out.rs`：
     - 次を足す。
       ```rust
       /// A node's drawing being recorded for the draw cache.
       #[derive(Debug, Default)]
       struct Recording {
           commands: Vec<D>,
           /// A layer, an item, a drawable or pixels were drawn: it cannot be replayed.
           spoiled: bool,
       }

       /// Whether `command` draws pixels the asset store hands out for this
       /// frame (an image, or a pattern that may tile one): such a command
       /// is drawn afresh every frame.
       fn holds_pixels(command: &D) -> bool {
           // Every variant is named: a new one must say whether it holds pixels.
           match command {
               D::Image { .. } | D::ImageSampled { .. } => true,
               D::Rect { fill, .. }
               | D::RoundedRect { fill, .. }
               | D::RoundedRectCorners { fill, .. } => matches!(fill, Paint::Pattern(_)),
               D::FillShape { paint, .. } | D::StrokeShape { paint, .. } => {
                   matches!(paint, Paint::Pattern(_))
               }
               D::Glyphs { fill, .. } => matches!(fill, Some(Paint::Pattern(_))),
               // (add here every other variant of `RenderCmd`: those with a
               // `Paint` or `Option<Paint>` field return whether it is
               // `Paint::Pattern(_)`; the others return `false`. No `_` arm.)
           }
       }
       ```
       `match` の残りの腕は `RenderCmd` の定義（`rg -n "pub enum RenderCmd" -A 170` を arvane-protocol で）を読んで、全 variant を書く。`Paint` の `use` を足す。
     - `struct SceneOut` に `recording: Option<Recording>,` を足し、`Default` で `recording: None,` にする。struct の直前の印を次にする（W1 の印があれば置き換える）：`// xtask-allow(state): ordinal numbers the scene layers of the current scope; recording is the node drawing the draw cache records (begin_record..end_record); independent`
     - `impl SceneOut` に次を足す。
       ```rust
       /// Start recording what the next commands draw (one recording at a time).
       pub(crate) fn begin_record(&mut self) {
           debug_assert!(self.recording.is_none(), "invariant: one recording at a time");
           self.recording = Some(Recording::default());
       }

       /// The commands drawn since `begin_record`, or `None` when they cannot
       /// be replayed (see `Recording::spoiled`).
       pub(crate) fn end_record(&mut self) -> Option<Vec<D>> {
           let recording = self.recording.take()?;
           (!recording.spoiled).then_some(recording.commands)
       }

       /// Draw `commands` again, as recorded.
       pub(crate) fn replay(&mut self, commands: &[D]) {
           for command in commands {
               self.push(command.clone());
           }
       }

       /// What is being recorded cannot be replayed.
       fn spoil(&mut self) {
           if let Some(recording) = &mut self.recording {
               recording.spoiled = true;
           }
       }
       ```
     - `push` を次にする。
       ```rust
       pub(crate) fn push(&mut self, command: D) {
           if let Some(recording) = &mut self.recording {
               if holds_pixels(&command) {
                   recording.spoiled = true;
               } else {
                   recording.commands.push(command.clone());
               }
           }
           self.top().push(command);
       }
       ```
     - `next_scene_key`、`open_layer`、`close_layer`、`item`、`drawable` の本体の最初の行に `self.spoil();` を置く（`scope` は置かない：名前を決めるだけで何も描かず、各 scene の描画が自分で先に呼ぶ）。
  4. `PaintPass { .. }` を作る所に `cache` を足す：`src/paint/frame_scene.rs` の `composition` は `cache: true`、`src/paint/canvas.rs` と `src/paint/snapshot.rs` は `cache: false`（`..pass` で写す所はそのまま）。
  5. `src/paint/frame_scene.rs`：
     - `const VERIFY_DRAWS: bool = cfg!(all(debug_assertions, feature = "testing"));` を足す（doc：`/// In debug builds with the test driver every kept drawing is checked before it is replayed: its paint against the node's paint now, its commands against the ones that paint draws now into a scratch output. A change that did not forget a drawing panics, naming the node; the frame, the kept drawings and the layers' versions are the replay's.`）。
     - `fn placed(&self, id, o) -> (PaintProps, Rect, Rect)` を次に置き換える。
       ```rust
       /// The box and the canvas (inside the border) of `id` when its layout
       /// box starts at `o`, drawn with the paint `p`.
       fn boxes_at(&self, id: NodeId, o: Point, p: &PaintProps) -> (Rect, Rect) {
           let layout = self.computed_layout(id).unwrap_or_default();
           let bounds = Rect {
               x: o.x,
               y: o.y,
               width: layout.size.width,
               height: layout.size.height,
           };
           (bounds, visual::padding_box(bounds, p))
       }
       ```
     - 次の method を足す。
       ```rust
       /// The drawing kept for `step`'s node when it was made for this step
       /// and scale, taken out of its slot (`keep_draw` puts one back).
       fn take_draw(&self, step: &Step, pass: PaintPass) -> Option<Box<NodeDraw>> {
           if !pass.cache {
               return None;
           }
           let n = self.node(step.id)?;
           self.slots.draws[n.index.0]
               .take()
               .filter(|d| d.key == DrawKey::of(step, pass))
       }

       /// Keep `draw` for `id`'s next frame (`fresh`: it was made now).
       fn keep_draw(&self, id: NodeId, draw: Box<NodeDraw>, fresh: bool, pass: PaintPass) {
           if !pass.cache {
               return;
           }
           let Some(n) = self.node(id) else {
               return;
           };
           if fresh {
               self.paint.draw_owners.borrow_mut().insert((id, n.index.0));
           }
           self.slots.draws[n.index.0].set(Some(draw));
       }

       /// The node's own box: its focus ring around a clip path, then its
       /// shadows, background, border and ring (nothing unless it draws itself).
       fn draw_box(&self, place: &Place, pass: PaintPass, engine: &mut arvane::Engine, out: &mut SceneOut) {
           // (the body of the old `if step.own { ... }` block of paint_open,
           //  reading `place.step`, `place.paint`, `place.bounds` and
           //  `place.clip_path`, starting with `if !place.step.own { return; }`)
       }

       /// The node's canvas scene and content (text, input lines, image),
       /// inside its clips (nothing unless it draws itself).
       fn draw_content(
           &self,
           place: &Place,
           canvas: Rect,
           pass: PaintPass,
           engine: &mut arvane::Engine,
           out: &mut SceneOut,
       ) {
           // (the part of the old paint_open after `if !step.own { return; }`,
           //  starting with `if !place.step.own { return; }` and
           //  `let Some(n) = self.node(place.step.id) else { return; };`)
       }

       /// The node's draw-over scene and scrollbars.
       fn draw_over(&self, step: &Step, p: &PaintProps, pass: PaintPass, engine: &mut arvane::Engine, out: &mut SceneOut) {
           let id = step.id;
           if step.own
               && let Some(n) = self.node(id)
               && let Some(scene) = &n.scenes()[1]
           {
               let (_, canvas) = self.boxes_at(id, step.origin, p);
               self.paint_scene(id, 1, scene, canvas, &tint(step), engine, out, pass);
           }
           self.paint_bars(step, out);
       }

       /// Drop the drawings of the nodes the rebuilt list does not list.
       pub(super) fn evict_draws(&mut self) {
           let listed: HashSet<NodeId> = self
               .paint
               .list
               .entries
               .iter()
               .filter_map(|e| match e {
                   Entry::Open(step) => Some(step.id),
                   Entry::Close(_) => None,
               })
               .collect();
           let slots = &self.slots;
           self.paint.draw_owners.get_mut().retain(|(id, slot)| {
               let keep = listed.contains(id);
               if !keep {
                   slots.draws[*slot].set(None);
               }
               keep
           });
       }
       ```
       `Place` の `paint: &'a PaintProps` はそのまま。`draw_box` と `draw_content` の本体は元の `paint_open` の該当部分を写し、`id`・`n`・`p`・`bounds`・`tint`・`outline`・`clip_path` は `place` と `step` から作る（`let tint = tint(place.step); let outline = BoxShape::of(place.bounds, &place.paint.radius);`）。
     - `src/paint/frame_scene.rs` の `use` に `RenderCmd`（1.2 節の path）と `kivact_types::PaintProps` が無ければ足す。次の method を足す（`paint_open` と照合が同じ置き方を使う）。
       ```rust
       /// Where `step`'s node is drawn with `paint`: its place and its canvas
       /// (inside the border).
       fn place_at<'a>(&self, step: &'a Step, list: &'a PaintList, paint: &'a PaintProps) -> (Place<'a>, Rect) {
           let (bounds, canvas) = self.boxes_at(step.id, step.origin, paint);
           let clip_path = paint
               .effects
               .as_ref()
               .and_then(|e| e.clip_path.as_ref())
               .map(|s| shape::clip_path(s, bounds));
           let place = Place {
               step,
               list,
               paint,
               bounds,
               clip_path,
           };
           (place, canvas)
       }
       ```
     - `paint_open` を次にする。kept の drawing は、`Commands` なら再生し、`Live` なら（paint だけを保ったまま）描く。照合（`VERIFY_DRAWS`）は再生の前に別の出力で行い、frame の出力・kept の drawing・layer の版は照合の無い build と同じになる（EA-01・EA-14）。
       ```rust
       fn paint_open(
           &self,
           step: &Step,
           list: &PaintList,
           pass: PaintPass,
           engine: &mut arvane::Engine,
           out: &mut SceneOut,
       ) {
           let id = step.id;
           if self.node(id).is_none() {
               return;
           }
           let kept = self.take_draw(step, pass);
           if VERIFY_DRAWS && let Some(draw) = &kept {
               self.verify_draw(step, list, draw, pass, engine);
           }
           let paint = kept
               .as_ref()
               .map_or_else(|| Rc::new(visual::paint(self, id)), |d| Rc::clone(&d.paint));
           let (place, canvas) = self.place_at(step, list, &paint);
           let (outer, inner): (Vec<Opening>, Vec<Opening>) = step
               .opened()
               .into_iter()
               .partition(|kind| matches!(kind, Opening::Layer | Opening::Transform | Opening::Group));
           for kind in outer {
               self.open(kind, &place, pass, out);
           }
           let (draw, fresh) = match kept {
               Some(draw) => {
                   match &draw.recorded {
                       Recorded::Commands { boxed, content, .. } => {
                           out.replay(boxed);
                           for kind in inner {
                               self.open(kind, &place, pass, out);
                           }
                           out.replay(content);
                       }
                       Recorded::Live => {
                           self.draw_box(&place, pass, engine, out);
                           for kind in inner {
                               self.open(kind, &place, pass, out);
                           }
                           self.draw_content(&place, canvas, pass, engine, out);
                       }
                   }
                   (draw, false)
               }
               None => {
                   out.begin_record();
                   self.draw_box(&place, pass, engine, out);
                   let boxed = out.end_record();
                   for kind in inner {
                       self.open(kind, &place, pass, out);
                   }
                   out.begin_record();
                   self.draw_content(&place, canvas, pass, engine, out);
                   let content = out.end_record();
                   let draw = Box::new(NodeDraw {
                       key: DrawKey::of(step, pass),
                       paint: Rc::clone(&paint),
                       recorded: Recorded::of(boxed, content),
                   });
                   (draw, true)
               }
           };
           self.keep_draw(id, draw, fresh, pass);
       }
       ```
       関数が 80 行を超えるときは、`match kept` の二つの腕の中身をそれぞれ `fn replay_open`、`fn record_open` に分ける（引数は `place`、`inner`、`canvas`、`pass`、`engine`、`out` と各腕の値）。
     - 次の method を足す。照合は kept の paint と記録を使わない：node の今の paint（`visual::paint`）と今の箱から描き直し、別の `SceneOut` に記録して比べる（EA-14）。
       ```rust
       /// Check a kept drawing against one made now (`VERIFY_DRAWS`): its
       /// paint against the paint the node is drawn with now, and its box and
       /// content against the ones that paint and the node's boxes draw now,
       /// recorded in a scratch output. The check only reads: the frame, the
       /// kept drawing and the layers' versions are the replay's.
       fn verify_draw(&self, step: &Step, list: &PaintList, draw: &NodeDraw, pass: PaintPass, engine: &mut arvane::Engine) {
           let id = step.id;
           let paint = visual::paint(self, id);
           assert!(
               *draw.paint == paint,
               "the kept paint of node {id:?} is stale: a change it is drawn from did not forget it"
           );
           let Recorded::Commands { boxed, content, .. } = &draw.recorded else {
               return;
           };
           let (place, canvas) = self.place_at(step, list, &paint);
           let mut scratch = SceneOut::default();
           scratch.begin_record();
           self.draw_box(&place, pass, engine, &mut scratch);
           let now_boxed = scratch.end_record();
           scratch.begin_record();
           self.draw_content(&place, canvas, pass, engine, &mut scratch);
           let now_content = scratch.end_record();
           assert!(
               now_boxed.as_ref() == Some(boxed) && now_content.as_ref() == Some(content),
               "the kept drawing of node {id:?} is stale: a change it is drawn from did not forget it"
           );
       }
       ```
       （記録した命令は出力の transform・clip に依らない：opening は記録の外で描く。`Live` の drawing は毎 frame 描くので paint だけを比べる。）
     - `paint_close` を次にする。
       ```rust
       fn paint_close(&self, step: &Step, pass: PaintPass, engine: &mut arvane::Engine, out: &mut SceneOut) {
           let kept = self.take_draw(step, pass);
           match kept {
               Some(mut draw) => {
                   self.close_kept(step, &mut draw, pass, engine, out);
                   self.keep_draw(step.id, draw, false, pass);
               }
               None => {
                   let p = visual::paint(self, step.id);
                   self.draw_over(step, &p, pass, engine, out);
               }
           }
           for kind in step.opened().into_iter().rev() {
               match kind {
                   Opening::Layer => out.close_layer(),
                   Opening::Transform => out.push(D::PopTransform),
                   Opening::Group | Opening::ClipGroup => out.push(D::PopGroup),
                   Opening::Presence | Opening::Clip => out.push(D::PopClip),
               }
           }
       }

       /// Draw the kept node's draw-over part: replayed when recorded (checked
       /// first, `VERIFY_DRAWS`), drawn when live, else drawn and recorded.
       fn close_kept(&self, step: &Step, draw: &mut NodeDraw, pass: PaintPass, engine: &mut arvane::Engine, out: &mut SceneOut) {
           let paint = Rc::clone(&draw.paint);
           let spoiled = match &mut draw.recorded {
               Recorded::Commands { over: Some(over), .. } => {
                   if VERIFY_DRAWS {
                       self.verify_over(step, over, pass, engine);
                   }
                   out.replay(over);
                   false
               }
               Recorded::Live => {
                   self.draw_over(step, &paint, pass, engine, out);
                   false
               }
               Recorded::Commands { over, .. } => {
                   out.begin_record();
                   self.draw_over(step, &paint, pass, engine, out);
                   let fresh = out.end_record();
                   let spoiled = fresh.is_none();
                   *over = fresh;
                   spoiled
               }
           };
           if spoiled {
               draw.recorded = Recorded::Live;
           }
       }

       /// Check a kept draw-over part against the one the node's paint now
       /// draws, recorded in a scratch output (`VERIFY_DRAWS`).
       fn verify_over(&self, step: &Step, over: &[RenderCmd], pass: PaintPass, engine: &mut arvane::Engine) {
           let paint = visual::paint(self, step.id);
           let mut scratch = SceneOut::default();
           scratch.begin_record();
           self.draw_over(step, &paint, pass, engine, &mut scratch);
           assert!(
               scratch.end_record().as_deref() == Some(over),
               "the kept draw-over of node {:?} is stale",
               step.id
           );
       }
       ```
     - `composition` の `if self.paint_list() { self.refresh_shader_clocks(); self.refresh_playback(); }` を `if self.paint_list() { self.evict_draws(); self.refresh_shader_clocks(); self.refresh_playback(); }` にする。
  6. `src/paint/list/invalidation.rs`：`PaintState::outdate_paint` の引数の最後に `draws: &[std::cell::Cell<Option<Box<NodeDraw>>>]` を足し、本体の最後（`self.list = list;` の後）に次を足す。
     ```rust
     if fonts {
         for (_, slot) in self.draw_owners.get_mut().drain() {
             draws[slot].set(None);
         }
     }
     for id in changes.redrawn() {
         if let Some(n) = tree.get(id) {
             draws[n.index.0].set(None);
         }
     }
     ```
     `src/model/interact/sync.rs` の呼出しに `&self.slots.draws` を足す。
  7. `src/model/interact/sync.rs`：
     - 次の method を足す。
       ```rust
       /// Every kept drawing is drawn again (the host repaints everything:
       /// scale, surface, reduced motion).
       pub(crate) fn forget_draws(&mut self) {
           for (_, slot) in self.paint.draw_owners.get_mut().drain() {
               self.slots.draws[slot].set(None);
           }
       }

       /// `id` and every node below it draw again: an inherited text color
       /// changed with `id`'s state.
       fn forget_draws_below(&mut self, id: NodeId) {
           let mut stack = vec![id];
           while let Some(x) = stack.pop() {
               if let Some(n) = self.tree.get(x) {
                   self.slots.draws[n.index.0].set(None);
                   stack.extend(self.tree.children_ids(n));
               }
           }
       }

       /// Whether a state of `id` (or its transition) colors the text below it.
       fn colors_text_below(&self, id: NodeId) -> bool {
           self.node(id)
               .and_then(|n| n.states())
               .is_some_and(|s| s.deltas().any(|d| d.text_color.is_some()))
               || self.motion.get(id).is_some_and(|a| a.colors_text(|| true))
       }
       ```
     - `invalidate_paint` の本体を `self.changes.paint = true; self.forget_draws();` にする。
     - `sync` の `for id in &restyled { ... }` の loop の中の最後に `if self.colors_text_below(*id) { self.forget_draws_below(*id); }` を足す。
  8. `src/model/frame/ticks.rs` の `update_env`：`if env.reduced_motion != self.frame.env.reduced_motion { self.changes.restack = true; self.changes.paint = true; }` の中に `self.forget_draws();` を足す。
  9. `src/model/states.rs` に次を足す（`use std::borrow::Cow;`）。
     ```rust
     /// The paint `id` is drawn with now, borrowed when nothing changes it:
     /// no state delta applies, no transition runs and no layout run scales it.
     pub(crate) fn paint_now(&self, id: NodeId) -> Cow<'_, PaintProps> {
         if self.motion.get(id).is_none()
             && self.layout_motion_scale(id).is_none()
             && self.active_deltas(id).is_empty()
             && let Some(p) = self.tree.get(id).and_then(|n| n.paint())
         {
             return Cow::Borrowed(p);
         }
         Cow::Owned(self.effective_paint(id))
     }

     /// The cursor `id` names now: its paint's, with its active state
     /// deltas' (a cursor does not transition).
     pub(crate) fn cursor_now(&self, id: NodeId) -> kivact_types::Cursor {
         self.active_deltas(id)
             .iter()
             .rev()
             .find_map(|d| d.cursor)
             .or_else(|| self.tree.get(id).and_then(|n| n.paint()).map(|p| p.cursor))
             .unwrap_or(PaintProps::default().cursor)
     }
     ```
     （`PaintProps::cursor` の型が `kivact_types::Cursor` でなければ、`src/model/pointer/hover.rs` が `CursorKind` と呼んでいる型をそのまま返り値にする。）
  10. `effective_paint` の呼出しを借用版に替える：`src/model/interact/visual.rs` の `paint` は `let mut p = host.paint_now(id).into_owned();`、`src/model/interact/geometry.rs` の `Look::now` は `Look::of(&host.paint_now(id))`、`src/model/interact/hit.rs` の `self.effective_paint(id).backface` は `self.paint_now(id).backface`、`src/model/scrollbar.rs` と `src/model/transform.rs` の `self.effective_paint(..)` はすべて `self.paint_now(..)`、`src/model/pointer/hover.rs` の `let specified = self.effective_paint(id).cursor;` は `let specified = self.cursor_now(id);`。`src/host/write/reader.rs` の `effective_paint` はそのまま。
  11. `rg -n "fn placed\(|self\.placed\(" crates/kivact-engine/src` が空であることを確かめる。
- **消すもの**：`Model::placed`。
- **状態の閉じ方**：`NodeDraw` は enum の `Recorded` と値だけを持ち対象外。`Recording` は bool が一つで対象外。`SceneOut` は手順 3 の印。`PaintState` は `RefCell` の field が一つ増える：W1 の `PaintState` の印の文の最後に `; draw_owners lists the slots holding a kept drawing (keep_draw, evict_draws, forget_draws)` を足す（印が無ければ `// xtask-allow(state): the paint group's derived caches, each with one writer; draw_owners lists the slots holding a kept drawing (keep_draw, evict_draws, forget_draws)` を struct の直前に置く）。
- **試験**：本 unit では足さない（engine の試験の入口 `src/host/testing/test_host.rs` は W5a では鎖の E15 が持つ）。debug build と `testing`（E2E の build）では手順 5 の照合が、描き直しの印の漏れを panic で示す。照合が paint の無効化の漏れを必ず見つけること（fault injection）は E32（W6b）の `draw_cache::a_paint_change_that_does_not_forget_the_drawing_is_caught` が確かめる（EA-14）。
- **確認（tier a）**：`ENGINE-A`、`E2E: compositor, frame_sources, style_paint, paint, borders_backgrounds, text_effects, effects_filters, canvas, drawables, hover, motion, images_animated, text_editing, custom_text_field, input_platform, layout_scroll, touch_scroll, transform, transform_3d, a11y_incremental, offscreen, offscreen_svg, viewport, color_spaces, text_style, text_primitives, svg_embedded_text`、`E2E: ui_controls, ui_standard, ui_theming, interaction_tooltip, interaction_slider`
- **完了条件**：手順 11 の `rg` が空。確認がすべて通る（照合の panic が一つも出ない）。

## E13：restack を重なりの事実が変わったときだけにする（PERF-6）

- **wave / lane / 前提**：W5a / E2 / E12
- **所有**：`src/model/interact/sync.rs`、`src/model/changes.rs`、`src/model/frame/ticks.rs`、`src/model/state_motion/state.rs`、`src/model/state_motion/steps.rs`、`src/model/layout_motion/steps.rs`、`src/paint/list/walk.rs`、`src/paint/state.rs`、`src/paint/frame_scene.rs`
- **目的**：paint list の全再構築を、重なりの事実（stacking context、z、layer の事実、子の組）の変化だけに絞る（EN-04）。具体的に：(1) transition の毎 tick の restack をやめ、動く layer は list を作り直さず matrix だけを毎 frame 置き直す。(2) 状態の style の変化は、list を作ったときの look の重なりの事実と比べて違うときだけ restack する。(3) scroll は scroll container の subtree だけを歩き直して list に差し込む。歩き直しは置き換えた範囲の古い transform・layer を消し（list の vector は scroll を何度繰り返しても作り直した list と同じ長さ。EA-02）、list から外れた node の歩きの記録と draw cache を消す（EA-03。設計書 12 章）。
- **手順**：
  1. `src/paint/state.rs`：次の型を足し、`struct PaintList` に field を足す（`Default` の derive はそのまま）。
     ```rust
     /// A listed layer that moves every frame (a running opacity or transform
     /// transition, or a layout run): its matrix is set again before each
     /// frame without a new list (`Model::refresh_moving_layers`).
     #[derive(Clone, Copy, Debug, PartialEq)]
     pub(crate) struct MovingLayer {
         pub(super) id: NodeId,
         /// Its index in `PaintList::layers`.
         pub(super) index: u32,
         /// Its box's origin in its parent's space, as walked.
         pub(super) origin: Point,
         /// The parent's perspective it was walked with.
         pub(super) perspective: Option<Matrix4>,
     }

     /// Where a listed scroll container was walked from, to walk its subtree
     /// again after it scrolls (`Model::rewalk_scrolled`).
     #[derive(Clone, Copy, Debug, PartialEq)]
     pub(crate) struct ScrollerWalk {
         pub(super) origin: Point,
         pub(super) alpha: f64,
         pub(super) clip: Rect,
         pub(super) perspective: Option<Matrix4>,
     }
     ```
     `PaintList` の field：
     ```rust
     /// The listed layers that move every frame.
     pub(super) moving: Vec<MovingLayer>,
     /// The listed scroll containers and where they were walked from.
     pub(super) scrollers: HashMap<NodeId, ScrollerWalk>,
     /// The window origins of the layers (`Model::layers`) the list was built with.
     pub(super) layer_origins: Vec<(NodeId, Point)>,
     /// The look each listed node with state styles or a transition was walked with.
     pub(super) looks: HashMap<NodeId, Look>,
     /// The scroll containers whose subtrees are walked again before the next frame.
     pub(super) rewalk: BTreeSet<NodeId>,
     ```
     `impl PaintState` に次を足す。
     ```rust
     /// The look `id` was listed with, if it has state styles or a transition.
     pub(crate) fn listed_look(&self, id: NodeId) -> Option<Look> {
         self.list.looks.get(&id).copied()
     }

     /// The scroll containers `scrolled` moved their content: their subtrees
     /// are walked again before the next frame (a list built anew does it all).
     pub(crate) fn rewalk_after(&mut self, scrolled: impl IntoIterator<Item = NodeId>) {
         self.list.rewalk.extend(scrolled);
     }
     ```
  2. `src/paint/list/walk.rs`：
     - `struct Walk` に `moving: Vec<MovingLayer>,`、`scrollers: HashMap<NodeId, ScrollerWalk>,`、`looks: HashMap<NodeId, Look>,` を足す。`Walk { .. }` を作る所（`paint_list`、`subtree_paint_list`）で三つとも空で初期化する。
     - `struct LayerPlan` に `moves: bool,`、`origin: Point,`、`perspective: Option<Matrix4>,` を足し、`plan` の中の `Some(LayerPlan { ... })` に `moves, origin, perspective: parent_perspective,` を足す（`moves` は今ある変数）。
     - `Walk::node`：
       - `let Some(n) = host.node(id) else { return; };` の直後に次を置く。
         ```rust
         if host.tree().scrollers.contains(&id) {
             self.scrollers.insert(
                 id,
                 ScrollerWalk {
                     origin,
                     alpha: parent_alpha,
                     clip,
                     perspective: parent_perspective,
                 },
             );
         }
         ```
       - `let look = Look::now(host, id, n);` の直後に `if n.states().is_some() || host.motion.get(id).is_some() { self.looks.insert(id, look); }` を置く。
       - `let visible = match &plan { ... };` を次にする。
         ```rust
         // A moving layer is listed whole: its matrix changes every frame
         // without a new list, so it is not culled.
         let moving = plan.as_ref().is_some_and(|p| p.moves);
         let visible = moving
             || match &plan {
                 Some(p) if p.affine.is_none() => projected_overlaps(&p.matrix, p.cull, clip),
                 _ => overlaps(placed(local.as_ref(), own, origin), clip),
             };
         ```
       - `self.place(local, clip, origin, plan)` を `self.place(id, local, clip, origin, plan)` にする。
     - `fn place` に最初の引数 `id: NodeId` を足し、plan の腕を次にする。
       ```rust
       if let Some(plan) = plan {
           // A layer's subtree is culled in its space: through the inverse of
           // an affine map, else within its bounds; a moving layer's within
           // its bounds (its map changes every frame).
           let clip = match plan.affine.and_then(|t| t.invert()).filter(|_| !plan.moves) {
               Some(back) => map_rect(Some(&back), clip),
               None => plan.cull,
           };
           self.layers.push(LayerSpec {
               matrix: plan.matrix,
               bounds: plan.bounds,
               plane: plan.plane,
               backface: plan.backface,
               grid: plan.grid,
           });
           let index = (self.layers.len() - 1) as u32;
           if plan.moves {
               self.moving.push(MovingLayer {
                   id,
                   index,
                   origin: plan.origin,
                   perspective: plan.perspective,
               });
           }
           return Some((None, clip, Point::default(), Some(index)));
       }
       ```
     - `paint_list` を次にし、元の `if rebuilt { ... }` の中身を `fn build_list(&mut self, list: &mut PaintList, key: ListState)` に移す。`build_list` の最後で `list.moving`、`list.scrollers`、`list.looks` に walk の値を入れ、`list.layer_origins` に walk の始めに一度だけ呼んだ `self.layers()` の値を入れる（`for (layer, origin) in self.layers()` は `let layers = self.layers(); for &(layer, origin) in &layers` にして、最後に `list.layer_origins = layers;`）。
       ```rust
       pub(crate) fn paint_list(&mut self) -> bool {
           let mut list = std::mem::take(&mut self.paint.list);
           let key = ListState::Built {
               size: self.window(),
               scale: self.paint.scale,
           };
           let rebuilt = list.state != key
               || (!list.rewalk.is_empty() && !self.rewalk_scrolled(&mut list));
           if rebuilt {
               self.build_list(&mut list, key);
           }
           list.rewalk.clear();
           self.paint.list = list;
           rebuilt
       }
       ```
     - 次を足す。
       ```rust
       /// Walk the subtrees of the scrolled containers again in place of
       /// their old steps; `false` when one cannot be (it is not listed with a
       /// close, or a layer's window origin moved): the caller builds the list.
       /// The old steps' transforms and layers are dropped (`compact`); the
       /// nodes that left the list lose their walk records and kept drawings.
       fn rewalk_scrolled(&self, list: &mut PaintList) -> bool {
           if self.layers() != list.layer_origins {
               return false;
           }
           let tree = self.tree();
           // Outermost first: a container inside another is walked with it.
           let ids: Vec<NodeId> = list
               .rewalk
               .iter()
               .copied()
               .filter(|id| !list.rewalk.iter().any(|o| o != id && tree.is_in_subtree(*id, *o)))
               .collect();
           let margin = MARGIN.max(1.0 / self.paint.scale.max(f64::MIN_POSITIVE));
           for id in ids {
               let Some(at) = list.scrollers.get(&id).copied() else {
                   return false;
               };
               let Some(open) = list
                   .entries
                   .iter()
                   .position(|e| matches!(e, Entry::Open(s) if s.id == id))
               else {
                   return false;
               };
               let Some(close) = list.entries[open..]
                   .iter()
                   .position(|e| matches!(e, Entry::Close(s) if s.id == id))
                   .map(|k| open + k)
               else {
                   return false;
               };
               let transforms = std::mem::take(&mut list.transforms);
               let layers = std::mem::take(&mut list.layers);
               let mut walk = Walk {
                   host: self,
                   margin,
                   indices: &mut list.indices,
                   facts: &list.facts,
                   generation: list.ink_gen,
                   used: HashSet::new(),
                   out: Vec::new(),
                   transforms,
                   layers,
                   moving: Vec::new(),
                   scrollers: HashMap::new(),
                   looks: HashMap::new(),
               };
               walk.node(id, at.origin, at.alpha, at.clip, at.perspective);
               let Walk {
                   out,
                   transforms,
                   layers,
                   moving,
                   scrollers,
                   looks,
                   ..
               } = walk;
               list.transforms = transforms;
               list.layers = layers;
               let left = left_ids(&list.entries[open..=close], &out);
               list.moving.retain(|m| !tree.is_in_subtree(m.id, id));
               list.moving.extend(moving);
               list.scrollers.retain(|node, _| !left.contains(node));
               list.scrollers.extend(scrollers);
               list.looks.retain(|node, _| !left.contains(node));
               list.looks.extend(looks);
               list.entries.splice(open..=close, out);
               self.forget_left_draws(&left);
           }
           compact(list);
           true
       }

       /// Drop the kept drawings of the nodes `left` (they left the list).
       fn forget_left_draws(&self, left: &HashSet<NodeId>) {
           if left.is_empty() {
               return;
           }
           self.paint.draw_owners.borrow_mut().retain(|(id, slot)| {
               let keep = !left.contains(id);
               if !keep {
                   self.slots.draws[*slot].set(None);
               }
               keep
           });
       }

       /// Set the matrices of the moving layers for this frame.
       pub(crate) fn refresh_moving_layers(&mut self) {
           let mut list = std::mem::take(&mut self.paint.list);
           for m in &list.moving {
               let Some(n) = self.node(m.id) else {
                   continue;
               };
               let size = self.layout_at(n.index).unwrap_or_default().size;
               let own = self.local_matrix(m.id, size).unwrap_or(Matrix4::IDENTITY);
               let matrix = own
                   .then(Matrix4::translate(m.origin.x, m.origin.y, 0.0))
                   .then(m.perspective.unwrap_or(Matrix4::IDENTITY));
               if let Some(spec) = list.layers.get_mut(m.index as usize) {
                   spec.matrix = matrix;
               }
           }
           self.paint.list = list;
       }
       ```
       （`draw_owners` と `Slots::draws` は E12 のもの。`rewalk_scrolled` が 80 行を超えるときは、`for` の中の `walk.node(..)` から `list.entries.splice(..)` までを `fn splice_rewalk(&self, list: &mut PaintList, id: NodeId, at: ScrollerWalk, open: usize, close: usize, margin: f64)` に移す。）
     - 次の自由関数を `src/paint/list/walk.rs` の最後に足す。
       ```rust
       /// The node of a list entry.
       fn entry_id(entry: &Entry) -> NodeId {
           match entry {
               Entry::Open(step) | Entry::Close(step) => step.id,
           }
       }

       /// The nodes the old steps `old` list and the new steps `new` do not.
       fn left_ids(old: &[Entry], new: &[Entry]) -> HashSet<NodeId> {
           let listed: HashSet<NodeId> = new.iter().map(entry_id).collect();
           old.iter()
               .map(entry_id)
               .filter(|id| !listed.contains(id))
               .collect()
       }

       /// Drop the transforms and layers no listed step refers to (a walk
       /// again of a scroll container leaves its old ones behind), renumbering
       /// them in the order the steps first refer to them: the order a walk
       /// pushes them, so the list equals one built anew. A moving layer no
       /// step refers to any more is dropped too.
       fn compact(list: &mut PaintList) {
           let PaintList {
               entries,
               transforms,
               layers,
               moving,
               ..
           } = list;
           let mut transform_at = vec![None; transforms.len()];
           let mut layer_at = vec![None; layers.len()];
           let mut kept_transforms = Vec::new();
           let mut kept_layers = Vec::new();
           for entry in entries.iter_mut() {
               let (Entry::Open(step) | Entry::Close(step)) = entry;
               if let Some(t) = &mut step.transform {
                   let old = *t as usize;
                   *t = *transform_at[old].get_or_insert_with(|| {
                       kept_transforms.push(transforms[old]);
                       (kept_transforms.len() - 1) as u32
                   });
               }
               if let Some(l) = &mut step.layer {
                   let old = *l as usize;
                   *l = *layer_at[old].get_or_insert_with(|| {
                       kept_layers.push(layers[old]);
                       (kept_layers.len() - 1) as u32
                   });
               }
           }
           moving.retain_mut(|m| match layer_at.get(m.index as usize).copied().flatten() {
               Some(index) => {
                   m.index = index;
                   true
               }
               None => false,
           });
           *transforms = kept_transforms;
           *layers = kept_layers;
       }
       ```
       （walk は step を list に載せるときにだけ transform と layer を積む（`place` の後の `listed`）ので、積まれた要素はどれも `Open` の step に指される。歩き直しの後の list は、作り直した list と entries・transforms・layers が等しい：`paint/list/self_check.rs` の比較もそのまま通る。）
  3. `src/paint/frame_scene.rs` の `composition`：`if self.paint_list() { ... }` に `else { self.refresh_moving_layers(); }` を足す。
  4. `src/model/frame/ticks.rs` の `tick`：`if self.motion.is_active() { self.changes.restack = true; ... }` の塊全体を消す。
  5. `src/model/state_motion/state.rs`：`impl StateAnimation` に次を足し、`StateMotion::moves_layer` の本体を `self.0.get(&id).is_some_and(StateAnimation::moves_layer)` にする。
     ```rust
     /// Whether it moves its node's opacity or transform now (a running
     /// track of either): the node is a compositor layer while it does.
     pub(crate) fn moves_layer(&self) -> bool {
         self.tracks.iter().any(|t| {
             matches!(t.phase, TrackPhase::Running { .. })
                 && matches!(t.prop, TransitionProp::Opacity | TransitionProp::Transform)
         })
     }
     ```
  6. `src/model/state_motion/steps.rs` の `step_states`：`retain` の前に `let mut switched = Vec::new();` を置く。閉包の中の `a.step(now, reduced);` を `let was = a.moves_layer(); a.step(now, reduced); if was != a.moves_layer() { switched.push(*id); }` にし、transform の塊の中の `self.changes.moved();` の行を消す（a11y と layout の印は残す）。`self.motion.0 = animations;` の後に `for id in switched { self.motion_changed(id); }` を置く（layer になる・やめる時だけ list を作り直す）。
  7. `src/model/layout_motion/steps.rs` の `step_layout_motion`：最後の `self.changes.restack = true; self.changes.paint = true;` の 2 行を消し、`for id in before { if !self.layout_motion.runs.contains_key(&id) { ... } }` の内側の塊に `self.changes.restack = true;` を足す（run が終わった時だけ）。
  8. `src/model/changes.rs` の `scrolled`：`self.moved();` の行を消す（scroll は list を作り直さず歩き直す）。
  9. `src/model/interact/sync.rs`：
     - 次を足す。
       ```rust
       /// Whether `id`'s look changed what its list steps are made of since
       /// the list was built; a node the list did not record changes it.
       fn relooked(&self, id: NodeId) -> bool {
           let Some(n) = self.node(id) else {
               return false;
           };
           let Some(listed) = self.paint.listed_look(id) else {
               return true;
           };
           let now = crate::layer::Look::now(self, id, n);
           if self.motion.moves_layer(id) || self.layout_motion_transform(id).is_some() {
               // A moving layer applies its opacity itself: only the other facts count.
               let steady = |look: crate::layer::Look| crate::layer::Look { opacity: 1.0, ..look };
               steady(now).stacking() != steady(listed).stacking()
           } else {
               now.stacking() != listed.stacking()
           }
       }
       ```
     - `sync` の `let redraw = changes.paint || changes.fonts || moved || changes.any(Dirty::WRITTEN.union(Dirty::REDRAW));` を `... || changes.any(Dirty::WRITTEN.union(Dirty::REDRAW).union(Dirty::SCROLLED));` にする。
     - `for id in &restyled { ... }` の前に `let mut relook = false;` を置き、loop の中に `relook |= self.relooked(*id);` を足す。`let restack = changes.restack || moved || !restyled.is_empty();` を `let restack = changes.restack || moved || relook;` にする。
     - `self.paint.outdate_paint(...)` の行の後に次を足す。
       ```rust
       self.paint.rewalk_after(changes.marked(Dirty::SCROLLED));
       ```
       （`changes.marked` が借用で書けないときは `let scrolled: Vec<NodeId> = changes.marked(Dirty::SCROLLED).collect();` を先に作る。）
     - E04 の `if changes.geometry != Geometry::Kept { self.forget_origins(); }` を `if changes.geometry != Geometry::Kept || changes.any(Dirty::SCROLLED.union(Dirty::STUCK)) { self.forget_origins(); }` にする。
  10. `src/paint/list.rs` の module doc の「The list is rebuilt only when something may move, show, hide, restack or reclip nodes (typed setter invalidation for writes, `Model::sync` for layout passes, scrolls and state styles)」を「The list is rebuilt only when stacking facts change: a write's stacking, the children of a box, a layout pass, a state style or transition whose look's stacking facts differ from the listed look, a layer that starts or stops moving. A scroll walks the scrolled container's subtree again (`Model::rewalk_scrolled`); a moving layer's matrix is set every frame (`Model::refresh_moving_layers`) and it is never culled.」にする。
- **W5a で足したもの（設計者が受け入れた。commit `3694e0e1`）**：手順 4 で tick の塊を消すと、動いている node の事実を毎 tick 作り直す処理も消え、paint の self_check が不一致になった。次の 3 つで直す。
  - `interact/sync.rs` の `relook_moving(&Dispatched) -> bool`：`Dirty::REDRAW` の node のうち transition が動いているもの（state style の transition だけでなく prop の transition も。`restyled` には入らない）の事実を作り直し、`relooked` で restack を決める。`relooked` は `pub(crate)`。`state_motion/steps.rs` の `step_states` は、終わった transition の node に `motion_ended(id)`（事実を作り直し、`relooked` なら restack）を呼ぶ。
  - `paint/list/walk.rs` の `opacity`：動いている layer（`moving`）は不透明度 1.0 として grouping と alpha を決める（layer が自分の不透明度を合成で掛けるので、list は不透明度の値に依らない。`relooked` の `steady` と同じ考え）。
  - E12 が `frame/ticks.rs` の塊に足した文字色の分岐（動いている文字色の下の draw を忘れる）は、塊と一緒に消え、`step_states` の各 step の後に移る（`a.colors_text(|| true)` なら `forget_draws_below`。最後の step を含む）。
- **消すもの**：`tick` の motion の restack の塊、`step_states` の `changes.moved()`、`step_layout_motion` の毎 tick の restack と `paint = true`、`Changes::scrolled` の `moved()`、`!restyled.is_empty()` による restack。
- **状態の閉じ方**：新しい型は値だけ（`MovingLayer`、`ScrollerWalk`）。`PaintList` は field が増えるが `&mut self` の method を持たない（`Clone, Debug, Default` の値。`compact` は自由関数）。
- **試験**：本 unit では足さない（engine の試験の入口 `src/host/testing/test_host.rs` は W5a では鎖の E15 が持つ）。歩き直しの不変条件は E32（W6b）が確かめる：`draw_cache::scrolling_back_and_forth_keeps_the_paint_list_the_size_it_was`（EA-02）と `draw_cache::rows_scrolled_out_drop_their_kept_drawings`（EA-03）。`paint/list/self_check.rs` の比較（歩き直した list と作り直した list の entries・transforms・layers が等しい）は E2E の `layout_scroll`・`touch_scroll` の中で毎回働く。
- **確認（tier a）**：`ENGINE-A`、`E2E: compositor, motion, shared_layout, presence, presence_modes, layout_scroll, touch_scroll, scroll_timeline, transform, transform_3d, hover, style_paint, frame_sources, a11y_incremental, drawables, overlay, layout`、`E2E: interaction_tooltip, interaction_show_hide, ui_collections, ui_data_table`
- **完了条件**：`rg -n "restack = true" crates/kivact-engine/src/model/frame/ticks.rs crates/kivact-engine/src/model/layout_motion/steps.rs` が `update_env` の reduced motion の 1 行と、run が終わった時の 1 行だけを返す。確認がすべて通る。

## E14：layer ごとの `content_version`（engine 側）

- **wave / lane / 前提**：W6b / E2 / E13（W5a）、A09（W5b。`CompositeLayer::content_version` の field と `src/lower/content_versions.rs` の `fresh_content_version()`）
- **所有**：`src/lower/scene_out.rs`、`src/lower/content_versions.rs`、`src/paint/frame_scene.rs`、`src/paint/state.rs`、`src/paint/canvas.rs`、`src/paint/snapshot.rs`、`src/paint/list/walk.rs`（`paint_list` だけ）、`src/host/testing/test_host.rs`（method を一つ足すだけ）、`crates/kivact-engine/Cargo.toml`（`[[test]]` を一つ）、`crates/kivact-engine/tests/content_version.rs`（新設）
- **目的**：Arvane が layer の中身の変化を版で知る（PERF-7、AR-03、付属書 A 4.2 の契約）。A09 は engine に「毎回新しい版」（`fresh_content_version()`）を書いた。本 unit はそれを、window の frame の layer では「描き直した node（draw cache の外れ）・毎 frame 描く `Live` の node・list の作り直しと歩き直しで版を上げ、それ以外は前の frame の同じ `LayerKey` の版を保つ」に置き換える。検証のための描き直し（E12 の `VERIFY_DRAWS`）は内容の変化ではない：別の出力に描くので、照合して同じだった kept の drawing は持ち越しのまま（EA-01、設計書 12 章）。数えない layer（drawable の layer、export、snapshot）と DevTools の overlay は `fresh_content_version()` のまま（A 4.2：上げすぎは正しさを損なわない）。版の値はいつも `fresh_content_version()` から取る（process で一意に増えるので、同じ window の同じ `key` に一度使った値を別の内容に使わない）。
- **手順**：
  1. `src/lower/scene_out.rs`：
     - 次の型を足す。
       ```rust
       /// Whether a layer of the frame took a command not kept from the last frame.
       #[derive(Clone, Copy, Debug, PartialEq, Eq)]
       pub(crate) enum LayerChange {
           /// Every command it took was kept from the last frame.
           Kept,
           /// It took a new command.
           Touched,
           /// It is not tracked (a drawable's layers, composed elsewhere).
           Untracked,
       }
       ```
     - `struct Layer` に `touched: bool,` を足し、`Layer::new` で `touched: false` にする。`Layer` の印（W1 の後にあれば）の文の最後に `; touched says a command not kept from the last frame reached it (SceneOut::push)` を足す。無ければ struct の直前に `// xtask-allow(state): reopen and deepest track the chunk's open groups; touched says a command not kept from the last frame reached it (SceneOut::push); independent` を置く。
     - `struct SceneOut` に `keeping: bool,` を足し（`Default` で `false`）、印の文の最後に `; keeping marks the commands kept from the last frame (set_keeping)` を足す。`impl SceneOut` に次を足す。
       ```rust
       /// The next commands are (`true`) or are not kept from the last frame.
       pub(crate) fn set_keeping(&mut self, keeping: bool) {
           self.keeping = keeping;
       }
       ```
       `push` の `self.top().push(command);` の前に `if !self.keeping { self.top().touched = true; }` を、`open_layer` と `item` と `drawable` の `self.spoil();` の後にも `if !self.keeping { self.top().touched = true; }` を置く（`open_layer` は新しい layer を積む前の親に印す）。
     - `LayerProps::into_layer` の `CompositeLayer { .. }` の `content_version: super::content_versions::fresh_content_version(),`（A09 が書いた行）を次にする。
       ```rust
       // Set by the caller of `into_composition`: the window's frame stamps the
       // kept versions (`PaintState::stamp_versions`), every other composition
       // takes fresh ones (`content_versions::stamp_fresh`).
       content_version: 0,
       ```
     - `fn flatten(pieces, layers)` に引数 `changes: &mut Vec<LayerChange>` を足し、`Piece::Layer(child)` の腕で `let Layer { props, clips, pieces, touched, .. } = *child;` とし、`layers.push(...)` の直後に `changes.push(if touched { LayerChange::Touched } else { LayerChange::Kept });` を置く。`adopt` を呼ぶ `Piece::Drawable` の腕は、`adopt` の後で `changes.resize(layers.len(), LayerChange::Untracked);` を置く。
     - `into_composition(mut self, size: Size) -> Composition` を `-> (Composition, Vec<LayerChange>)` にし、root の分を先に `changes` に入れる：`let mut changes = vec![if root.touched { LayerChange::Touched } else { LayerChange::Kept }];` を `let mut layers = vec![...]` の後に置き、`flatten(root.pieces, &mut layers, &mut changes)` にし、`(Composition { layers, exports: Vec::new() }, changes)` を返す。root が無い場合は `(Composition { .. }, Vec::new())`。
  2. `src/lower/content_versions.rs`（A09 の file）に次を足す。`fresh_content_version` は変えない。
     ```rust
     /// Give every layer of `composition` a fresh version: a composition
     /// whose layers are not compared with a last frame (an export, a
     /// snapshot, a drawable's subtree).
     pub(crate) fn stamp_fresh(composition: &mut Composition) {
         for layer in &mut composition.layers {
             layer.content_version = fresh_content_version();
         }
     }
     ```
     （`Composition` の `use` は `src/lower/scene_out.rs` と同じ path。）
  3. `src/paint/frame_scene.rs`：
     - `compose_list` の返り値を `(Composition, Vec<LayerChange>)` にする（最後の行 `out.into_composition(pass.size)` の型に合わせる）。
     - `paint_open` の `if VERIFY_DRAWS && let Some(draw) = &kept { .. }` の次に `out.set_keeping(kept.as_ref().is_some_and(|d| matches!(d.recorded, Recorded::Commands { .. })));` を、関数の最後（`keep_draw` の後）に `out.set_keeping(false);` を置く。
     - `paint_close` の `let kept = self.take_draw(step, pass);` の次に `out.set_keeping(kept.as_ref().is_some_and(|d| matches!(d.recorded, Recorded::Commands { over: Some(_), .. })));` を、関数の最後に `out.set_keeping(false);` を置く。
     - 持ち越しとして数えるのは、再生した命令（`Recorded::Commands`）とその node の opening だけ。`Recorded::Live` の node は、kept の paint を使っても毎 frame 新しい命令として数える：画像や pattern の中身（animation の frame、外部の画像の更新）と scene の layer・item は命令に表れずに変わるので、版を保つと Arvane が古い layer を使う（付属書 A 4.2：上げすぎは正しさを損なわない）。照合（`VERIFY_DRAWS`）は E12 のとおり別の `SceneOut` に描くので `out` の印に触れず、照合した kept の drawing は照合の無い build と同じく持ち越しになる。
     - `composition` を次の形にする。
       ```rust
       let rebuilt = self.paint_list();
       if rebuilt {
           self.evict_draws();
           self.refresh_shader_clocks();
           self.refresh_playback();
       } else {
           self.refresh_moving_layers();
       }
       let pass = PaintPass { scale: self.paint.scale, size: self.window(), cache: true };
       self.prepare_canvas(engine);
       let (mut composition, changes) = self.compose_list(engine, &self.paint.list, pass);
       self.paint.stamp_versions(&mut composition, &changes, rebuilt);
       self.report_dropped_draws();
       self.append_exports(engine, &mut composition);
       composition
       ```
       （歩き直した frame も layer の中身が変わるので版を上げる。手順 5 で `paint_list` がその事実を `PaintState::rewalked` に書き、`stamp_versions` が読んで `false` に戻す。`append_exports` が足す export の layer は手順 6 で版を持つ。）
  4. `src/paint/state.rs`：
     ```rust
     /// The content version each layer of the window's last frame had.
     #[derive(Debug, Default)]
     pub(crate) struct LayerVersions(HashMap<LayerKey, u64>);
     ```
     `PaintState` に `pub(super) versions: LayerVersions,`（`Default`）を足し、`impl PaintState` に次を足す。
     ```rust
     /// Give each layer of the frame its content version: the last frame's
     /// when nothing in it changed and the list stayed, a fresh one otherwise
     /// (an untracked layer always).
     pub(crate) fn stamp_versions(
         &mut self,
         composition: &mut Composition,
         changes: &[LayerChange],
         rebuilt: bool,
     ) {
         let list_changed = rebuilt || std::mem::take(&mut self.rewalked);
         let mut seen = HashMap::with_capacity(composition.layers.len());
         for (layer, change) in composition.layers.iter_mut().zip(changes) {
             let kept = self.versions.0.get(&layer.key).copied();
             let version = match (change, kept) {
                 (LayerChange::Kept, Some(version)) if !list_changed => version,
                 _ => crate::lower::content_versions::fresh_content_version(),
             };
             layer.content_version = version;
             if *change != LayerChange::Untracked {
                 seen.insert(layer.key, version);
             }
         }
         self.versions.0 = seen;
     }

     /// The content versions of the last frame's tracked layers, by key (the test driver reads them).
     #[cfg(feature = "testing")]
     pub(crate) fn layer_versions(&self) -> Vec<(LayerKey, u64)> {
         self.versions.0.iter().map(|(key, version)| (*key, *version)).collect()
     }
     ```
  5. `src/paint/state.rs`：`PaintState` に `pub(super) rewalked: bool,` を足し（`PaintState::new` で `false`）、`PaintState` の印の文の最後に `; rewalked says the list was walked again in part since the last frame (paint_list), taken by stamp_versions` を足す。`src/paint/list/walk.rs` の `paint_list` の `let rebuilt = ...;` を次にする。
     ```rust
     let rewalk = list.state == key && !list.rewalk.is_empty();
     let rewalked = rewalk && self.rewalk_scrolled(&mut list);
     let rebuilt = list.state != key || (rewalk && !rewalked);
     self.paint.rewalked |= rewalked;
     ```
  6. window の frame 以外の composition は新しい版にする：`rg -n "compose_list\(|into_composition\(" crates/kivact-engine/src` の結果のうち、`composition()` の 1 か所を除くすべて（`src/paint/canvas.rs` の `subtree_layers`、`src/paint/snapshot.rs`、ほか見つかったもの）で、結果を `let (mut composition, _) = ...;` で受けて `crate::lower::content_versions::stamp_fresh(&mut composition);` を呼んでから使う。`src/host/devtools.rs` の overlay の `fresh_content_version()` は変えない（DevTools の overlay は毎 frame 新しい値のまま。INC-04）。
  7. `src/host/testing/test_host.rs`：`frame_stats` と同じ書き方で window の model を取り、次の method を足す（`#[doc(hidden)]` は同じファイルの他の試験用 method に合わせる）。
     ```rust
     /// The content versions of window `window`'s layers in its last frame, by key.
     pub fn layer_versions(&self, window: WindowId) -> Vec<(LayerKey, u64)> {
         // the model of `window` as `frame_stats` reaches it, then:
         // model.paint.layer_versions()
     }
     ```
     （本体：`frame_stats` が `Launch::Running(host)` から host を取る形を写し、`host.app.borrow().model(window).map(|model| model.paint.layer_versions()).unwrap_or_default()` を返す。）
  8. `crates/kivact-engine/Cargo.toml` に `tests/host_locale.rs` と同じ形の `[[test]]` を足す：`name = "content_version"`、`required-features = ["testing"]`。`crates/kivact-engine/tests/content_version.rs` を作る（`LayerKey` は engine の `src/lower/scene_out.rs` が使う path で import する）。
     ```rust
     //! Role: Tests that an unchanged layer keeps its content version and a changed one bumps it.

     use kivact_engine::api::Client;
     use kivact_engine::contract::{BoxData, NodeKind, TurnOutput};
     use kivact_engine::testing::{ArvaneTestHost, MirrorMode, TestHostOptions};
     use kivact_types::paint::Fill;
     use kivact_types::render::LayerKey;
     use kivact_types::{Dimension, LayoutProps, NodeId, PaintProps, Rgba, WillChange, WindowId};

     const ROOT: NodeId = NodeId(1);

     fn painted(hex: u32) -> PaintProps {
         PaintProps {
             background: Some(Fill::Solid(Rgba::from_hex(hex, 1.0))),
             ..PaintProps::default()
         }
     }

     fn root_version(host: &ArvaneTestHost) -> u64 {
         host.layer_versions(WindowId::MAIN)
             .into_iter()
             .find(|(key, _)| *key == LayerKey::Root)
             .map(|(_, version)| version)
             .expect("the root layer is drawn")
     }

     #[test]
     fn an_unchanged_layer_keeps_its_content_version_and_a_changed_one_bumps_it() {
         let mut host = ArvaneTestHost::new(
             Client::new(
                 |_, _, writer| {
                     writer.create(WindowId::MAIN, ROOT, NodeKind::Root);
                     writer.set_paint(ROOT, painted(0x2563eb));
                     writer.commit();
                 },
                 |_, _, _| TurnOutput::default(),
             ),
             TestHostOptions {
                 mirror: MirrorMode::Off,
                 ..TestHostOptions::default()
             },
         )
         .expect("the test host starts");
         host.redraw();
         let first = root_version(&host);

         // The same paint is no write: the next frame keeps the version.
         host.writer().set_paint(ROOT, painted(0x2563eb));
         host.writer().commit();
         host.redraw();
         assert_eq!(root_version(&host), first, "nothing in the layer changed");

         // A new background is drawn again: the version moves on.
         host.writer().set_paint(ROOT, painted(0xdc2626));
         host.writer().commit();
         host.redraw();
         assert!(root_version(&host) > first, "the root's background changed");
     }

     const A: NodeId = NodeId(2);
     const B: NodeId = NodeId(3);

     /// The version of the layer `key` in the last frame.
     fn version_of(host: &ArvaneTestHost, key: LayerKey) -> u64 {
         host.layer_versions(WindowId::MAIN)
             .into_iter()
             .find(|(k, _)| *k == key)
             .map(|(_, version)| version)
             .expect("the layer is drawn")
     }

     /// A 100 × 100 box painted `hex` that is a compositor layer of its own.
     fn layer_painted(hex: u32) -> PaintProps {
         PaintProps {
             will_change: WillChange::Transform,
             ..painted(hex)
         }
     }

     #[test]
     fn a_changed_layer_bumps_its_version_and_a_sibling_layer_keeps_its_own() {
         let mut host = ArvaneTestHost::new(
             Client::new(
                 |_, _, writer| {
                     writer.create(WindowId::MAIN, ROOT, NodeKind::Root);
                     for (id, hex) in [(A, 0x2563eb), (B, 0x16a34a)] {
                         writer.create(WindowId::MAIN, id, NodeKind::Box(BoxData::default()));
                         writer.insert(ROOT, id, None);
                         writer.set_layout(
                             id,
                             LayoutProps {
                                 width: Dimension::Px(100.0),
                                 height: Dimension::Px(100.0),
                                 ..LayoutProps::default()
                             },
                         );
                         writer.set_paint(id, layer_painted(hex));
                     }
                     writer.commit();
                 },
                 |_, _, _| TurnOutput::default(),
             ),
             TestHostOptions {
                 mirror: MirrorMode::Off,
                 ..TestHostOptions::default()
             },
         )
         .expect("the test host starts");
         host.redraw();
         let (a, b) = (version_of(&host, LayerKey::Node(A)), version_of(&host, LayerKey::Node(B)));

         // A's new background draws A again; B is replayed (and, in debug
         // builds with `testing`, checked) without a new command.
         host.writer().set_paint(A, layer_painted(0xdc2626));
         host.writer().commit();
         host.redraw();
         assert!(version_of(&host, LayerKey::Node(A)) > a, "A's background changed");
         assert_eq!(version_of(&host, LayerKey::Node(B)), b, "nothing in B changed");
     }
     ```
     （`Client::new` の二つの閉包の引数の形は `tests/host_locale.rs` と同じ。`Fill`・`LayerKey`・`LayoutProps` などの path と名前は、W2・W6a の後の名前にする（1.2 節：`rg -n "pub enum Fill|pub enum LayerKey|pub struct LayoutProps" crates`）。二つ目の試験の B は `will_change` で自分の layer を持ち、その drawing は A の書込みで描き直しの印を受けない。）
- **消すもの**：`into_layer` の `fresh_content_version()` の呼出し（window の frame は stamp、他は `stamp_fresh`）。
- **状態の閉じ方**：`LayerVersions` は map 一つの値で対象外。`Layer`・`SceneOut`・`PaintState` は手順 1・5 の印の文を足す。
- **試験**：`crates/kivact-engine/tests/content_version.rs` の `an_unchanged_layer_keeps_its_content_version_and_a_changed_one_bumps_it` と `a_changed_layer_bumps_its_version_and_a_sibling_layer_keeps_its_own` を足す（付属書 A 4.2 が求める試験。`kivact::test` から composition を見られないので engine の試験にした）。二つは debug build と `testing`（照合が働く）と release（照合が無い）の両方で走らせる：照合の有無で版が変わらないこと（EA-01）。付属書 A の debug の hash の照合が、E2E の中で版の上げ忘れを検出する。
- **確認（tier a）**：`ENGINE-A`（`content_version` の試験を含む）、`dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test --release -p kivact-engine --features testing --test content_version`、`E2E: compositor, frame_sources, motion, layout_scroll, canvas, drawables, images_animated, effects_filters, offscreen, viewport`
- **完了条件**：どのコマンドも 0 件でない（1.4 節）。`rg -n "content_version" crates/kivact-engine/src` が、`src/paint/state.rs`（`stamp_versions`）、`src/lower/content_versions.rs`（`stamp_fresh` と A09 の関数）、`src/lower/scene_out.rs` の `content_version: 0`（置き場）、`src/host/devtools.rs` の overlay の `fresh_content_version()` だけを返す。新しい試験を含め確認がすべて通る。

## E32：draw cache と歩き直しの不変条件の試験（EA-02・EA-03・EA-14）

- **wave / lane / 前提**：W6b / E2 / E14（E14 → E32 → E24。E12・E13 は W5a で済んでいる）
- **所有**：`src/paint/state.rs`（thread-local 一つと試験用の method 一つ）、`src/paint/list/invalidation.rs`（`outdate_paint` の loop に 3 行）、`src/host/testing/test_host.rs`（method を二つ足すだけ）、`src/host/testing.rs`（再公開の 1 行）、`crates/kivact-engine/Cargo.toml`（`[[test]]` を一つ）、`crates/kivact-engine/tests/draw_cache.rs`（新設）
- **目的**：E12・E13 の不変条件を engine の試験で確かめる（設計書 12 章、6.4 節の TV-09）。(1) 描き直しの印の漏れ（試験の driver が入れる誤り）を、E12 の照合が今の paint から必ず見つける（EA-14）。(2) scroll の歩き直しを何度繰り返しても、list の entries・transforms・layers と draw cache は最初の frame と同じ大きさ（EA-02）。(3) list から外れた行は kept の drawing を持たない（EA-03）。W5a では engine の試験の入口（`test_host.rs`）を鎖の E15 が持つので、E12・E13 と同じ wave に置けない。
- **手順**：
  1. `src/paint/state.rs` に次を足す。
     ```rust
     #[cfg(feature = "testing")]
     thread_local! {
         /// A node whose kept drawing a redraw mark does not forget: the test
         /// driver's injected invalidation fault (`ArvaneTestHost::keep_stale_draw`),
         /// which the debug check of kept drawings must catch.
         pub(crate) static KEEP_STALE_DRAW: std::cell::Cell<Option<NodeId>> = const { std::cell::Cell::new(None) };
     }

     /// The sizes of a window's paint list and draw cache (the test driver reads them).
     #[cfg(feature = "testing")]
     #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
     pub struct PaintListSizes {
         /// The list's entries (opens and closes).
         pub entries: usize,
         /// The transforms the steps refer to.
         pub transforms: usize,
         /// The compositor layers the steps refer to.
         pub layers: usize,
         /// The nodes that keep a drawing.
         pub kept_draws: usize,
     }
     ```
     `impl PaintState` に次を足す。
     ```rust
     /// The sizes of the list and the draw cache (the test driver reads them).
     #[cfg(feature = "testing")]
     pub(crate) fn list_sizes(&self) -> PaintListSizes {
         PaintListSizes {
             entries: self.list.entries.len(),
             transforms: self.list.transforms.len(),
             layers: self.list.layers.len(),
             kept_draws: self.draw_owners.borrow().len(),
         }
     }
     ```
  2. `src/paint/list/invalidation.rs`：E12 が `outdate_paint` に足した `for id in changes.redrawn() {` の loop の最初に次を置く。
     ```rust
     #[cfg(feature = "testing")]
     if crate::paint::state::KEEP_STALE_DRAW.with(std::cell::Cell::get) == Some(id) {
         continue;
     }
     ```
  3. `src/host/testing.rs` の再公開の並びに `pub use crate::paint::state::PaintListSizes;` を足す。`src/host/testing/test_host.rs` の `impl ArvaneTestHost` に次を足す（`#[doc(hidden)]` は同じファイルの他の試験用 method に合わせる。window の model は E14 の `layer_versions` と同じ書き方で取る）。
     ```rust
     /// The sizes of window `window`'s paint list and draw cache after its last frame.
     pub fn paint_list_sizes(&self, window: WindowId) -> PaintListSizes {
         // the model of `window` as `layer_versions` reaches it, then:
         // model.paint.list_sizes(), or `PaintListSizes::default()` without it
     }

     /// Leave `node`'s kept drawing in place when a change marks it for a
     /// redraw (`None`: leave none): an injected fault the debug check of
     /// kept drawings must catch.
     pub fn keep_stale_draw(&self, node: Option<NodeId>) {
         crate::paint::state::KEEP_STALE_DRAW.with(|kept| kept.set(node));
     }
     ```
     （`paint_list_sizes` の本体：`layer_versions` が `Launch::Running(host)` から model を取る形を写し、`.map(|model| model.paint.list_sizes()).unwrap_or_default()` を返す。）
  4. `crates/kivact-engine/Cargo.toml` に `tests/host_locale.rs` と同じ形の `[[test]]` を足す：`name = "draw_cache"`、`required-features = ["testing"]`。`crates/kivact-engine/tests/draw_cache.rs` を作る（型の path と名前は W2・W6a の後のもの。1.2 節）。
     ```rust
     //! Role: Tests that the draw cache and the paint list keep their invariants:
     //! a stale kept drawing is caught, and scrolling keeps the list bounded.

     use kivact_engine::api::Client;
     use kivact_engine::contract::{BoxData, NodeKind, TurnOutput};
     use kivact_engine::testing::{ArvaneTestHost, MirrorMode, TestHostOptions};
     use kivact_types::paint::Fill;
     use kivact_types::{
         Dimension, ElementTransform, LayoutProps, NodeId, Overflow, PaintProps, Point, Rgba,
         WillChange, WindowId,
     };

     const ROOT: NodeId = NodeId(1);
     const LIST: NodeId = NodeId(2);
     /// How many rows the list scrolls: 20 px each in a list 200 px tall.
     const ROWS: u64 = 200;

     /// Row `i` of the list.
     fn row(i: u64) -> NodeId {
         NodeId(100 + i)
     }

     fn painted(hex: u32) -> PaintProps {
         PaintProps {
             background: Some(Fill::Solid(Rgba::from_hex(hex, 1.0))),
             ..PaintProps::default()
         }
     }

     /// A row's paint: even rows are compositor layers of their own, odd rows
     /// are drawn through a transform (the list keeps both kinds).
     fn row_paint(i: u64, hex: u32) -> PaintProps {
         if i % 2 == 0 {
             PaintProps {
                 will_change: WillChange::Transform,
                 ..painted(hex)
             }
         } else {
             PaintProps {
                 transform: ElementTransform {
                     scale: [0.9, 1.0, 1.0],
                     ..ElementTransform::default()
                 },
                 ..painted(hex)
             }
         }
     }

     fn sized(width: f64, height: f64) -> LayoutProps {
         LayoutProps {
             width: Dimension::Px(width),
             height: Dimension::Px(height),
             ..LayoutProps::default()
         }
     }

     /// A 200 × 200 list scrolling `ROWS` rows.
     fn start() -> ArvaneTestHost {
         ArvaneTestHost::new(
             Client::new(
                 |_, _, writer| {
                     writer.create(WindowId::MAIN, ROOT, NodeKind::Root);
                     writer.create(WindowId::MAIN, LIST, NodeKind::Box(BoxData::default()));
                     writer.insert(ROOT, LIST, None);
                     writer.set_layout(
                         LIST,
                         LayoutProps {
                             overflow_y: Overflow::Scroll,
                             ..sized(200.0, 200.0)
                         },
                     );
                     for i in 0..ROWS {
                         writer.create(WindowId::MAIN, row(i), NodeKind::Box(BoxData::default()));
                         writer.insert(LIST, row(i), None);
                         writer.set_layout(
                             row(i),
                             LayoutProps {
                                 shrink: 0.0,
                                 ..sized(180.0, 20.0)
                             },
                         );
                         writer.set_paint(row(i), row_paint(i, 0x2563eb));
                     }
                     writer.commit();
                 },
                 |_, _, _| TurnOutput::default(),
             ),
             TestHostOptions {
                 mirror: MirrorMode::Off,
                 ..TestHostOptions::default()
             },
         )
         .expect("the test host starts")
     }

     /// Scroll the list to `y` and draw the frame.
     fn scroll_to(host: &mut ArvaneTestHost, y: f64) {
         host.writer().scroll_to(LIST, Point { x: 0.0, y });
         host.writer().commit();
         host.redraw();
     }

     /// A paint change whose redraw mark is left out (an injected fault) is
     /// caught by the check of the kept drawing, which draws from the node's
     /// paint now, not the kept one (the check runs in debug builds with
     /// `testing` only).
     #[cfg(debug_assertions)]
     #[test]
     #[should_panic(expected = "is stale")]
     fn a_paint_change_that_does_not_forget_the_drawing_is_caught() {
         let mut host = start();
         host.redraw();
         host.keep_stale_draw(Some(row(0)));
         host.writer().set_paint(row(0), row_paint(0, 0xdc2626));
         host.writer().commit();
         host.redraw();
     }

     /// Scrolling down and back 10 000 times walks the list's subtree again
     /// each time; at the top the list is the size it was in the first frame.
     #[test]
     fn scrolling_back_and_forth_keeps_the_paint_list_the_size_it_was() {
         let mut host = start();
         host.redraw();
         let top = host.paint_list_sizes(WindowId::MAIN);
         assert!(top.transforms > 0 && top.layers > 0, "the rows hold transforms and layers: {top:?}");
         for round in 1..=10_000 {
             scroll_to(&mut host, 400.0);
             scroll_to(&mut host, 0.0);
             if round % 1_000 == 0 {
                 assert_eq!(host.paint_list_sizes(WindowId::MAIN), top, "after {round} round trips");
             }
         }
     }

     /// Rows scrolled out of the list drop their kept drawings: scrolling
     /// through every row keeps no more drawings than the list has entries.
     #[test]
     fn rows_scrolled_out_drop_their_kept_drawings() {
         let mut host = start();
         host.redraw();
         for step in 1..=38_u32 {
             scroll_to(&mut host, f64::from(step) * 100.0);
             let sizes = host.paint_list_sizes(WindowId::MAIN);
             assert!(sizes.kept_draws <= sizes.entries, "at {} px: {sizes:?}", step * 100);
         }
     }
     ```
     （一つ目の試験は、E32 の誤りの注入が無ければ印が kept の drawing を消し、照合は起きない。注入すると row 0 の kept の drawing が残り、E12 の `verify_draw` の paint の比較が panic する。三つ目の試験は、E13 の `forget_left_draws` が無いと、通り過ぎた行の drawing が残って `kept_draws` が `entries` を超える。）
- **消すもの**：無い。
- **状態の閉じ方**：`PaintListSizes` は値だけ（`&mut self` の method が無い）で対象外。`KEEP_STALE_DRAW` は `testing` の build だけの thread-local で、struct の field ではない。
- **試験**：手順 4 の 3 本を足す。
- **確認（tier a）**：
  ```
  ENGINE-A
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-engine --features testing --test draw_cache
  ```
- **完了条件**：二つ目のコマンドが 3 本の試験を実行して通る（0 件・2 件は失敗。1.4 節）。確認がすべて通る。

## E15：window の表を一つにする、close の信号を phase に写さない

- **wave / lane / 前提**：W5a / 鎖 / 付属書 R・C の W5a の鎖の unit のすべて（鎖の順は R05〜R31 → C07〜C18 → E15 → E17 → E18 → E20。E17 が E15 の後の `src/host/events.rs` と `src/host/testing/self_check.rs` の 2 行ずつを直すので、E15 は E17 の直前に鎖で走る。E15 の file は鎖の R・C の unit の file と重ならない）
- **所有**：`src/host/window.rs`（`src/model/window.rs` へ移す）、`src/host.rs`、`src/model.rs`、`src/model/app_model.rs`、`src/host/pump.rs`、`src/host/events.rs`、`src/host/window_changes.rs`、`src/host/foreign.rs`、`src/host/launch.rs`、`src/host/devtools.rs`、`src/host/write/reader.rs`、`src/host/write/writer.rs`、`src/host/testing/test_host.rs`、`src/host/testing/self_check.rs`
- **目的**：`Host.windows` と `AppModel.windows` の二つの表を、`AppModel.windows: BTreeMap<WindowId, Window { model, host }>` の一つにする。`HostPhase::Closed` と `observe_close` を消し、close は `CloseSignal` を直接読む（EN-15）。`HostWindow` は host が使う受け身の状態として `model::window` に置く（model は host を使えないので、型は model の葉に置き、host がそれを使う）。
- **手順**：
  1. `git mv crates/kivact-engine/src/host/window.rs crates/kivact-engine/src/model/window.rs` を実行する。`perl -pi -e 's/pub\(super\)/pub(crate)/g' crates/kivact-engine/src/model/window.rs` を実行する。先頭の `//! Role:` の文を `//! Role: The host's state of each platform window (surface, presentation, IME projections), kept with the window's model.` にする。
  2. `src/host.rs` の `mod window;` の行を消す。`src/model.rs` の子 module の並びに `pub(crate) mod window;` を足す。host の中の path を直す：`rg -l "super::window::|super::super::window::|crate::host::window::" crates/kivact-engine/src | xargs perl -pi -e 's/\b(super::super::window|super::window|crate::host::window)::/crate::model::window::/g'`。
  3. `src/model/app_model.rs`：
     - 次を足す。
       ```rust
       /// One open window: its model and the host's state of its platform window.
       pub(crate) struct Window {
           pub(crate) model: Model,
           pub(crate) host: super::window::HostWindow,
       }
       ```
     - `pub(crate) windows: BTreeMap<WindowId, Model>,` を `pub(crate) windows: BTreeMap<WindowId, Window>,` にする（doc：`/// Every window: its model from the write that opens it, and its host state (not drawable until the platform window opens).`）。
     - `new`：`windows.insert(window, main);` を `windows.insert(window, Window { model: main, host: super::window::HostWindow::new(kivact_types::WindowMode::Normal, None) });` にする。
     - `open_window`：`let model = Model::new(request.id, viewport, &shared); self.windows.insert(request.id, model);` を次にする。
       ```rust
       let model = Model::new(request.id, viewport, &shared);
       let host = super::window::HostWindow::new(request.mode, request.parent);
       self.windows.insert(request.id, Window { model, host });
       ```
     - `model`・`model_mut`・`window_of`・`with_node` を `Window` の `model` を返す形にし（`self.windows.get(&id).map(|w| &w.model)` など）、次を足す。
       ```rust
       /// The host state of window `id`.
       pub(crate) fn host(&self, id: WindowId) -> Option<&super::window::HostWindow> {
           self.windows.get(&id).map(|w| &w.host)
       }

       /// The host state of window `id`, mutably.
       pub(crate) fn host_mut(&mut self, id: WindowId) -> Option<&mut super::window::HostWindow> {
           self.windows.get_mut(&id).map(|w| &mut w.host)
       }
       ```
  4. `src/host/pump.rs`：
     - `struct Host` から `windows` の field と doc を消す。`HostPhase` から `Closed` を消し、`next_wake`・`set_next_wake` の `Closed` の腕を消す。
     - `observe_close` を次に置き換える。
       ```rust
       /// Whether the embedder closed the host (`CloseSignal`): no new turn starts.
       pub(super) fn closed(&self) -> bool {
           self.close.is_set()
       }
       ```
     - `drawable` を `!self.closed() && self.presence == AppPresence::Foreground && self.app.borrow().host(id).is_some_and(|w| matches!(w.surface, Surface::Drawable { .. }))` にする。
  5. host の全ファイルで次の置換をする：`self.observe_close()` → `self.closed()`、`self.phase == HostPhase::Closed` → `self.closed()`、`self.phase != HostPhase::Closed` → `!self.closed()`、`matches!(self.phase, HostPhase::Starting { .. } | HostPhase::Closed)` → `matches!(self.phase, HostPhase::Starting { .. }) || self.closed()`。コマンド：
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-engine/src/host" || exit 1
     rg -l "observe_close|HostPhase::Closed" . | xargs perl -pi -e 's/self\.observe_close\(\)/self.closed()/g; s/self\.phase == HostPhase::Closed/self.closed()/g; s/self\.phase != HostPhase::Closed/!self.closed()/g; s/matches!\(self\.phase, HostPhase::Starting \{ \.\. \} \| HostPhase::Closed\)/(matches!(self.phase, HostPhase::Starting { .. }) || self.closed())/g'
     cd "$ROOT" || exit 1
     ```
     残る `HostPhase::Closed` の match の腕（`control` など）は、その腕の処理を `if self.closed() { ... }` として match の前に移す。
  6. `self.windows`（`Host` の field）を使う所を、`self.app` の `Window` で書き直す。規則：(a) host の状態だけを読む所は `self.app.borrow().host(id)`、書く所は `self.app.borrow_mut().host_mut(id)`。(b) model と host の状態を両方使う所は `let mut app = self.app.borrow_mut(); let Some(window) = app.windows.get_mut(&id) else { ... };` で一度だけ借り、`window.model`・`window.host` を使う。(c) `self.windows.keys()` は `self.window_ids()` にする（`AppModel::windows()` の写し）。(d) `self.app` の借用を、`self.client.*`、`self.turn`、`self.effects`、`self.present`、`self.drawable`、その他 `self.app` を借りる `Host` の method の呼出しをまたいで持たない（借用は block で閉じる）。
  7. `src/host/window_changes.rs` の `apply_window_changes`：`self.windows.insert(id, HostWindow::new(request.mode, request.parent));` を消す（host の状態は `open_window` で作られている）。失敗の時の `self.windows.remove(&id);` を消し、`self.app.borrow_mut().remove_window(id);` は残す。`Close` の腕の `if self.windows.remove(&id).is_some() { windows.close(id); self.app.borrow_mut().remove_window(id); }` を `if self.app.borrow().host(id).is_some() { windows.close(id); self.app.borrow_mut().remove_window(id); }` にする。
  8. `src/host/launch.rs`：`host_windows` の map とその `insert` を消し、`Host { ... }` の literal から `windows: host_windows,` を消す。
  9. `src/host/write/writer.rs`・`reader.rs`・`src/host/testing/*.rs` の `app.windows.values()`・`values_mut()`・`get(..)` で `Model` を期待している所は `.map(|w| &w.model)`・`.map(|w| &mut w.model)` を付ける。`test_host.rs` の `Launch::Running(host) => host.windows.get(&window).and_then(|w| w.last_stats),` を `Launch::Running(host) => host.app.borrow().host(window).and_then(|w| w.last_stats),` にする。
  10. `rg -n "observe_close|HostPhase::Closed|self\.windows\b" crates/kivact-engine/src/host` が空であることを確かめる。
- **消すもの**：`Host::windows`、`HostPhase::Closed`、`Host::observe_close`、`src/host/window.rs`（移動）。
- **状態の閉じ方**：`Window` は二つの field の組で対象外。`HostWindow` の印（W1 の後にあれば）はそのまま移る。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: windows, close_request, app_handle, foreign_window, launch_window, launch_platform, launch_fonts, environment, lifecycle, launch_real_window`、`dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- embed::`、`dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --features embed --test foreign_window_run`
- **完了条件**：手順 10 の `rg` が空。確認がすべて通る。

## E16：a11y の Metadata で順序を一度だけ数える

- **wave / lane / 前提**：W5a / E1 / なし（E15 は鎖へ移った。E16 の file は E15 の file と重ならない）
- **所有**：`src/a11y/build/dirty.rs`、`src/a11y/build/identity.rs`、`src/a11y/build/node.rs`
- **目的**：一回の Metadata の構築で `semantic_order()` を一度だけ呼び、`clipping_of_all` は全 slot ではなくその順序（付いている node だけ）を回る（EN-10 の S の部分。設計書 6.2 の PERF-1 の Metadata の例外は残す）。
- **手順**：
  1. `src/a11y/build/identity.rs`：`prepare_region_ids(&mut self)` を `prepare_region_ids(&mut self, order: &[NodeId])` にし、中の `self.semantic_order().into_iter()` を `order.iter().copied()` にする。
  2. `src/a11y/build/node.rs`：`clipping_of_all(&self) -> Clipping` を次にする（名前も変える）。
     ```rust
     /// The clipping of every shown clipping list-like container among
     /// `order` (the attached nodes in semantic order).
     pub(super) fn clipping_of(&self, order: &[NodeId]) -> Clipping {
         let mut clipping = Clipping::default();
         for &list in order {
             if let Some((_, clipped)) = self.list_rows(list) {
                 clipping.set_list(list, clipped);
             }
         }
         clipping
     }
     ```
  3. `src/a11y/build/dirty.rs`：
     - `build_identity_metadata` の先頭を次にする（`let order = self.semantic_order();` を最初に一度だけ）。
       ```rust
       let order = self.semantic_order();
       self.prepare_region_ids(&order);
       let clipping = self.clipping_of(&order);
       ```
       関数の中の二つ目の `let order = self.semantic_order();` を消す。
     - `build_semantics_tail` に引数 `order: &[types::NodeId]` を足し（`metadata` の次）、中の `let order = self.semantic_order();` を消し、`&order` を使っていた所を `order` にする。`build_semantics_for` の呼出しに `&order` を渡す。
  4. `rg -n "semantic_order\(\)" crates/kivact-engine/src/a11y` が、定義と `build_identity_metadata` の 1 か所と、Metadata 以外の経路（あれば）だけを返すことを確かめる。
- **消すもの**：`clipping_of_all`（`clipping_of` に替わる）、二つの余分な `semantic_order()`。
- **状態の閉じ方**：新しい状態は無い。
- **試験**：足さない。`a11y_incremental` の `a11y_enumerated` の数の期待値は変えない（数える所は一度のまま）。
- **確認（tier a）**：`ENGINE-A`、`E2E: a11y_incremental, landmark_roles, relations, metrics_and_description, image_semantics, layout_scroll, components`、`E2E: apg_listbox, apg_grid, ui_collections`
- **完了条件**：手順 4 のとおり。確認がすべて通る。

## E17：editing の公開 API（`EditOutcome` から `changed` を消す、`EditRequest::Paste(String)`、寿命の無い `EditContext`、goal column）

- **wave / lane / 前提**：W5a / 鎖 / 付属書 R・C の W5a の鎖の unit のすべてと E15（鎖の順は R05〜R31 → C07〜C18 → E15 → E17 → E18 → E20。E15 が `src/host/events.rs` と `src/host/testing/self_check.rs` を書き終えた後に、その 2 行・2 行を直す）
- **所有**：`crates/kivact-editing/src/behavior.rs`、`crates/kivact-editing/src/session.rs`、`crates/kivact-editing/src/edit_state.rs`、`crates/kivact-editing/src/editor.rs`、`crates/kivact-editing/src/journal.rs`、`crates/kivact-editing/src/host.rs`、`crates/kivact-editing/src/lib.rs`、`src/model/keys.rs`、`src/model/apply.rs`（`unmark` の 1 行）、`src/model/edit/inputs.rs`、`src/model/edit/ime.rs`、`src/model/pointer/machine.rs`（`select_text`）、`src/measure/editor.rs`、`src/measure/input.rs`（1 行）、`src/text_input/shaping.rs`、`src/text_input.rs`（1 行）、`src/host/events.rs`（2 行）、`src/host/testing/self_check.rs`（2 行）、`crates/kivact-core/src/custom/text_editing.rs`、`crates/kivact-core/src/view/editing_data.rs`、`crates/kivact-core/src/view/mount/editing.rs`、`crates/kivact-core/src/custom.rs`（再公開の行）、`crates/kivact-interaction/src/text_edit/policy.rs`、`crates/kivact-ui/src/code_editing.rs`、`crates/kivact-ui-soft/src/rich_blocks.rs`、`crates/kivact-editing/tests/geometry.rs`（新設）、`dev/tests/kivact-public-api/tests/e2e_core/text_editing.rs`、`dev/tests/kivact-public-api/tests/e2e_core/custom_text_field.rs`
- **目的**：
  - 編集の behavior が「変えたか」を申告せず、host が text の revision の変化で `TextChanged` を決める（EN-14）。上限（`max_length`）と読み取り専用は journal の間だけ持つ（`Limits`）。終わりの手順を固定の関数 `close_edit` にする。
  - `EditRequest::Paste(String)` と寿命の無い `EditContext` にし、利用者の behavior の型に寿命を出さない（監査 4 章、設計書 5 章の「Rust 特有の作法を出さない」）。`EditContext` は editor の行の cache を `Rc` で共有し、文書の全部の行に lazy に答える：request ごとに行を写さず、caret の近くの窓に限らない（EA-11・EA-16。設計書 12 章）。caret の停止位置は既存の `TextGeometry` の合法な境界（grapheme と shaping cluster の共通部分）で、第二の実装を作らない（EA-12）。文字数は `max_length` があるときだけ数える（EA-11）。
  - 上下の caret 移動に goal column を持つ（EN-31）。`EditState::select(range)` を足す。
- **測定（手順 1 の前に一度）**：大きな文書の key の速さを、本 unit の変更の前に測って残す（EA-11。設計書 12 章「大きな文書の p95 が悪くならない」）。
  1. `dev/tests/kivact-public-api/tests/e2e_core/text_editing.rs` の最後に次を足す（公開 API だけなので、本 unit の前の code でも後の code でも compile する）。
     ```rust
     /// The 95th percentile of the time `press` takes, over `n` presses.
     fn p95(n: usize, mut press: impl FnMut(usize)) -> std::time::Duration {
         let mut times: Vec<std::time::Duration> = (0..n)
             .map(|i| {
                 let start = std::time::Instant::now();
                 press(i);
                 start.elapsed()
             })
             .collect();
         times.sort_unstable();
         times[n * 95 / 100]
     }

     /// Key latency in a 1 MB document of many lines and in a 1 MB single
     /// line: a key that moves the caret, a typed character and (many lines)
     /// a move by a line. A measurement run by hand in release: E17 compares
     /// it before and after; it asserts nothing.
     #[test]
     #[ignore = "a measurement, run by hand in release"]
     fn large_document_key_latency() {
         let line = "0123456789abcdef".repeat(4);
         let lines = vec![line.as_str(); 16_384].join("\n");
         let one_line = "0123456789abcdef".repeat(65_536);
         for (name, text, area) in [("lines", lines.as_str(), true), ("one line", one_line.as_str(), false)] {
             let (screen, _value, _log) = field_screen(text, area);
             let field = screen.get_by_role(Role::TextBox, if area { "Notes" } else { "Name" });
             field.press_key(Key::End);
             let left = p95(200, |_| {
                 field.press_key(Key::ArrowLeft);
             });
             eprintln!("latency {name} ArrowLeft p95 {left:?}");
             let typed = p95(200, |_| {
                 field.type_text("x");
             });
             eprintln!("latency {name} type p95 {typed:?}");
             if area {
                 let vertical = p95(200, |i| {
                     field.press_key(if i % 2 == 0 { Key::ArrowUp } else { Key::ArrowDown });
                 });
                 eprintln!("latency {name} ArrowUp/Down p95 {vertical:?}");
             }
         }
     }
     ```
  2. 次を実行し、出力を残す（`latency` で始まる 5 行が出る）。
     ```
     dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test --release -p kivact-public-api-tests --test e2e_core -- text_editing::large_document_key_latency --ignored --nocapture > target/e17-latency-before.txt 2>&1
     grep -c "^latency" target/e17-latency-before.txt
     ```
     二つ目のコマンドが `5` でなければ STOP する。
- **手順**：
  1. `crates/kivact-editing/src/behavior.rs`：
     - `EditOutcome` を次にする。
       ```rust
       /// The editing behavior's result, interpreted by the native host. A
       /// change the behavior made to the text is reported as `TextChanged`
       /// whatever the result says: the host reads it from the state.
       #[derive(Clone, Debug, PartialEq)]
       pub enum EditOutcome {
           /// Leave this request for normal event routing.
           NotEditing,
           /// Consumed.
           Done,
           /// Copy or cut of a non-empty selection: the host writes the text to
           /// the clipboard (a cut also removed it from the state).
           Copy(String),
           /// Ask the host to read the clipboard and route a Paste request.
           Paste,
       }
       ```
     - `EditRequest<'a>` の寿命を消し、`Paste(&'a str)` を `Paste(String)` にする。derive に `Clone, Debug, PartialEq` が無ければ足す。
     - `EditContext<'a>`・`EditLayout<'a>`・`trait Lines` を次に置き換える（`EditContext` の五つの公開 method の名前・引数・意味は今のまま。答える行は caret の近くに限らず文書の全部）。`EditContext` と各 field の上の注釈（W1 の印の行）はそのまま残し、`layout` の field の型だけを変える。`use kivact_types::{Key, Modifiers, Rect};` から `Rect` を消す。
       ```rust
       /// What an editing behavior is given beside the state and the request:
       /// the editor's native constraints and its laid-out lines.
       ///
       /// The layout queries take the state the behavior was given and answer
       /// for its text as the request found it, for every line of the text
       /// (`None` once the behavior changed the text, or when the editor is not
       /// laid out or shows its placeholder). The host shapes only the lines a
       /// query asks about. Lines are hard lines (a single-line editor has
       /// one); `x` is in logical pixels from the start of the editor's text
       /// box, before the editor's horizontal scroll. A context serves one
       /// request: it is neither cloned nor kept.
       pub struct EditContext {
           /// Whether committed text may contain line breaks.
           pub multiline: bool,
           /// Whether user input must preserve the value.
           pub read_only: bool,
           /// Whether clipboard copy and cut must hide the value.
           pub password: bool,
           layout: Option<EditLayout>,
       }

       impl EditContext {
           /// The constraints and the lines of one editing request (the host
           /// builds it; `layout` answers the line queries).
           #[doc(hidden)]
           pub fn new(multiline: bool, read_only: bool, password: bool, layout: Option<EditLayout>) -> Self {
               Self {
                   multiline,
                   read_only,
                   password,
                   layout,
               }
           }

           /// The lines, when they are of `state`'s text.
           fn layout(&self, state: &EditState) -> Option<&EditLayout> {
               self.layout.as_ref().filter(|l| l.revision == state.revision())
           }

           /// The number of lines of `state`'s text.
           pub fn line_count(&self, state: &EditState) -> Option<usize> {
               self.layout(state)?.lines.count()
           }

           /// The line holding byte offset `at` of `state`'s text (a line break
           /// belongs to the line it ends).
           pub fn line_of(&self, state: &EditState, at: usize) -> Option<usize> {
               self.layout(state)?.lines.line_of(at)
           }

           /// The byte range of `line`'s text, without its line break.
           pub fn line_range(&self, state: &EditState, line: usize) -> Option<Range<usize>> {
               self.layout(state)?.lines.content(line)
           }

           /// Where the caret at byte offset `at` of `state`'s text is drawn: its
           /// `x`.
           pub fn x_of(&self, state: &EditState, at: usize) -> Option<f64> {
               let l = self.layout(state)?;
               l.lines.caret_x(state.value(), at, l.width)
           }

           /// The editing stop of `line` (a grapheme and shaping-cluster
           /// boundary) whose caret is nearest `x` (the line's end past its last
           /// character), as a byte offset of `state`'s text.
           pub fn offset_at(&self, state: &EditState, line: usize, x: f64) -> Option<usize> {
               let l = self.layout(state)?;
               l.lines.nearest(state.value(), line, x, l.width)
           }
       }

       /// The host's laid-out lines of one revision of the text, as an
       /// [`EditContext`] reads them. Offsets and ranges are UTF-8 byte offsets
       /// into that text; content ranges exclude line breaks. A break belongs to
       /// the line it ends, and the byte right after it to the next line. Every
       /// query answers `None` when the host cannot answer it now: the lines are
       /// no longer of that revision, they show the placeholder, or the host is
       /// using them.
       #[doc(hidden)]
       pub trait Lines {
           /// How many lines there are.
           fn count(&self) -> Option<usize>;
           /// The line holding byte offset `at`, including its terminating break.
           fn line_of(&self, at: usize) -> Option<usize>;
           /// Line `i`'s text, without its break.
           fn content(&self, i: usize) -> Option<Range<usize>>;
           /// The `x` of the caret at byte offset `at` of `text`, in a box
           /// `width` wide.
           fn caret_x(&self, text: &str, at: usize, width: f64) -> Option<f64>;
           /// The editing stop of line `i` whose caret is nearest `x`: the
           /// stops of [`crate::TextGeometry`] (grapheme and shaping-cluster
           /// boundaries); `None` past the last line.
           fn nearest(&self, text: &str, i: usize, x: f64, width: f64) -> Option<usize>;
       }

       /// The lines an [`EditContext`] reads, the revision of the text they are
       /// of and the width of the box they are aligned in.
       #[doc(hidden)]
       pub struct EditLayout {
           /// The host's lines, shared with its line cache for the request.
           pub lines: Box<dyn Lines>,
           /// The revision of the text the lines are of.
           pub revision: u64,
           /// The width of the box the lines are aligned in.
           pub width: f64,
       }
       ```
       （`EditContext` は `Clone` を持たず、behavior には `&EditContext` で渡るので、request の後に残らない。`Lines` の実装（engine の `SharedLines`、手順 6）は文書を写さず、問われた行だけを shape する。）
     - `type Behave` と `EditBehavior::new`・`invoke` の署名から寿命を消す：`dyn Fn(&mut EditState, EditRequest, &EditContext) -> EditOutcome`。
  2. `crates/kivact-editing/src/edit_state.rs`：
     - `max_length` の field と `set_max_length` を消す。`replace` の「`text` を `self.max_length` まで切る」処理、`compose` の `max_length` の退避と戻し、`unmark` の「composition を `max_length` まで切る」処理を消す（上限は手順 5 の `close_edit` が journal の終わりに一度だけ掛ける）。
     - 次の method を足す。
       ```rust
       /// Select `range`: the anchor at its start, the caret at its end.
       pub fn select(&mut self, range: Range<usize>) {
           self.move_to(range.start, false);
           self.move_to(range.end, true);
       }

       /// Move the caret to `to` by a line (Up or Down), remembering the
       /// column `goal_x` the next vertical move aims for (`extend`: select
       /// from the anchor).
       pub fn move_vertically(&mut self, to: usize, extend: bool, goal_x: f64) {
           self.move_to(to, extend);
           self.session.goal_x = Some(goal_x);
       }

       /// The column the next vertical move aims for, if the caret got here by one.
       pub fn goal_x(&self) -> Option<f64> {
           self.session.goal_x
       }
       ```
     - `move_to` の本体の最後と、text を変える `splice` の本体の最後に `self.session.goal_x = None;` を足す（縦の移動以外のすべての移動と編集が goal を消す）。
  3. `crates/kivact-editing/src/session.rs`：`TextSession` に `pub(crate) goal_x: Option<f64>,`（doc：`/// The column a vertical caret move aims for (set by \`EditState::move_vertically\`, cleared by every other move or edit). It is not part of the session's identity: equality ignores it.`）を足す。derive から `PartialEq, Eq` を外し、次を書く。`caret_at` と `select` の literal に `goal_x: None` を足す。
     ```rust
     impl PartialEq for TextSession {
         fn eq(&self, other: &Self) -> bool {
             self.anchor == other.anchor && self.caret == other.caret && self.marked == other.marked
         }
     }

     impl Eq for TextSession {}
     ```
     `edit_state.rs` の `TextSession { .. }` の literal（`set_session` の中）にも `goal_x` を書く（`set_session` は渡された session の値をそのまま使う）。
  4. `crates/kivact-editing/src/journal.rs`：
     - 次を足し、`host.rs` の再公開に `Limits` を足す。
       ```rust
       /// What an edit may not do, held for the journal's lifetime.
       #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
       pub struct Limits {
           /// The most Unicode scalars the text may hold after the edit.
           pub max_length: Option<usize>,
           /// The text may not change (selection and copy still work).
           pub read_only: bool,
       }
       ```
     - `Journal` に `pub(crate) limits: Limits,` を足し、`pub fn limits(&self) -> Limits { self.limits }` を足す。
     - `editor.rs` の `pub fn journal(&mut self, max_length: Option<usize>) -> Journal<'_>` を `pub fn journal(&mut self, limits: Limits) -> Journal<'_>` にし、`self.state.set_max_length(max_length);` を消して `Journal { ..., limits }` を作る。`pub fn incoming(&self, mut state: EditState, max_length: Option<usize>) -> Incoming` は `state.set_max_length(max_length);` の行だけを消す（切り詰めの判定はその引数で今のまま行う）。
  5. `src/model/keys.rs`：
     - 次の自由関数を足す（`edit_editor_with` の前）。
       ```rust
       /// The end of every edit, in the host's fixed order: a composition is
       /// never cut; the text is cut to `limit` scalars (the editor's limit,
       /// never below what the text held when the edit began; `None` without a
       /// limit); a read-only editor takes every change of the text back; a
       /// value the core set during a composition is adopted. Whether the text
       /// changed is the journal's answer afterwards.
       fn close_edit(journal: &mut Journal<'_>, controlled: &mut ControlledValue, limit: Option<usize>) {
           if !journal.state().is_composing()
               && let Some(max) = limit
               && let Some((end, _)) = journal.state().value().char_indices().nth(max)
           {
               journal.truncate(end);
           }
           if journal.limits().read_only && (journal.text_changed() || journal.state().is_composing()) {
               journal.revert();
           }
           EditorNode::adopt_pending_controlled(controlled, journal);
       }
       ```
       （`ControlledValue` と `EditorNode` の path は `src/measure/editor.rs` のもの。text の scalar を数えるのは `limit` が `Some` のとき、つまり editor に `max_length` があるときだけ。上限が後から下がっても、今ある text は切らない。）
     - `edit_editor_with` を次にする。
       ```rust
       pub(super) fn edit_editor_with(
           &mut self,
           id: NodeId,
           f: impl FnOnce(&mut Journal<'_>, &Rc<RefCell<Option<EditLines>>>),
       ) -> Option<bool> {
           let n = self.tree.get_mut(id)?;
           if n.disabled() {
               return None;
           }
           let limits = Limits {
               max_length: n.max_length(),
               read_only: n.read_only(),
           };
           let report = n.reports_text();
           let EditorNode {
               editor,
               lines,
               controlled,
               ..
           } = self.slots.editors[n.index.0].as_deref_mut()?;
           let mut journal = editor.journal(limits);
           // Counted only under a limit: the edit never cuts below the scalars
           // the text holds now (a composition is never cut).
           let limit = limits.max_length.map(|max| {
               if journal.state().is_composing() {
                   max
               } else {
                   max.max(journal.state().value().chars().count())
               }
           });
           f(&mut journal, lines);
           close_edit(&mut journal, controlled, limit);
           // A read-only editor reports nothing (an adopted core value included).
           let changed = journal.text_changed() && !limits.read_only;
           let changes = journal.finish();
           let event = changed.then(|| HostEvent::TextChanged {
               node: id,
               changes,
               session: editor.state().session().clone(),
               text: report.then(|| editor.state().value().to_string()),
           });
           self.repaint_text(id);
           if let Some(e) = event {
               self.changes.mark(id, Dirty::MEASURE);
               self.changes
                   .layout
                   .insert(crate::model::changes::LayoutReasons::PASS);
               self.push(e);
           }
           Some(changed)
       }
       ```
       （`self.slots` を借りる行は、`n` の借用と重ならないよう E03 の後の形に合わせる。`lines` は `&mut Rc<..>` で分解され、`f` には `&Rc<..>` として渡る。）
     - `edit_input(&mut self, id, f: impl FnOnce(&mut EditState) -> bool)` を `f: impl FnOnce(&mut EditState)` にし、本体を `self.edit_editor_with(id, |journal, _| f(journal.state_mut()))` にする。`edit_input_with` を次にする（shaper は行を揃える間だけ借り、behavior の間は借りない）。
       ```rust
       /// Run `f` (a behavior's request) on the text input `id`, its lines
       /// brought up to its text first and shared with `f`'s context: the
       /// lines a query asks about are shaped then.
       pub(crate) fn edit_input_with(
           &mut self,
           id: NodeId,
           engine: &Rc<RefCell<arvane::Engine>>,
           f: impl FnOnce(&mut EditState, Option<EditLayout>),
       ) -> Option<bool> {
           let width = self.lay_out_lines(id, &mut engine.borrow_mut());
           self.edit_editor_with(id, |journal, lines| {
               let revision = lines.borrow().as_ref().map(|l| l.revision());
               let layout = revision.map(|revision| EditLayout {
                   lines: Box::new(SharedLines::new(Rc::clone(lines), Rc::clone(engine), revision)),
                   revision,
                   width,
               });
               f(journal.state_mut(), layout);
           })
       }
       ```
     - 閉包が `bool` を返していた呼出しをすべて直す：`rg -n "edit_input\(|edit_input_with\(|edit_editor_with\(" crates/kivact-engine/src` の各所で、閉包の最後の `true` / `false` の行を消す（`insert_text`、`compose_text`、`place_caret`、`set_input_value`、`ime_select`、`src/model/pointer/machine.rs` の `select_text`、`src/model/edit/inputs.rs` の `replace_edit_state`（閉包を `|journal, _| journal.take(incoming)` にし、`reported` の計算を消す）と `replace_edit_session`）。`src/model/apply.rs` の `self.edit_input(id, kivact_editing::EditState::unmark);` を `self.edit_input(id, |e| { e.unmark(); });` にする。
     - `commit_composition`：`let mut journal = editor.journal(n.max_length());` から `EditorNode::adopt_pending_controlled(controlled, &mut journal);` までを次にする（確定した composition は、composition の前の text の長さを下限として上限まで切る。これは前の `unmark` の切り詰めと同じ結果）。
       ```rust
       let limits = Limits {
           max_length: n.max_length(),
           read_only: n.read_only(),
       };
       let mut journal = editor.journal(limits);
       // Counted only under a limit: never below the text the composition was typed into.
       let limit = limits.max_length.map(|max| {
           let state = journal.state();
           let marked = state.marked().map_or(0, |r| state.value()[r].chars().count());
           max.max(state.value().chars().count() - marked)
       });
       if !journal.state_mut().unmark() {
           return;
       }
       close_edit(&mut journal, controlled, limit);
       ```
     - `handle_key` の behavior の呼出しを次にする。
       ```rust
       let mut outcome = EditOutcome::NotEditing;
       if let Some(behavior) = behavior.filter(|_| !spin_boundary) {
           self.edit_input_with(i, engine, |state, layout| {
               let (multiline, read_only, password) = flags;
               let context = EditContext::new(multiline, read_only, password, layout);
               outcome = behavior.invoke(state, EditRequest::Key { key, modifiers }, &context);
           });
       }
       match outcome {
           EditOutcome::NotEditing => {}
           EditOutcome::Done => return KeyResult::Consumed,
           EditOutcome::Copy(text) => return KeyResult::Copy(text),
           EditOutcome::Paste => return KeyResult::Paste(i),
       }
       ```
     - `paste_input` の閉包を次にする。
       ```rust
       self.edit_input_with(id, engine, |state, layout| {
           let (multiline, read_only, password) = flags;
           let context = EditContext::new(multiline, read_only, password, layout);
           behavior.invoke(state, EditRequest::Paste(text.to_string()), &context);
       });
       ```
     - 自由関数 `fn context`（`flags` と `Shaping` から `EditContext` を作る）を消す。
     - keys.rs の module doc の「(a behavior that returns `changed: false`)」を含む文を「A change of the text is reported when the text changed (`close_edit`), whatever the behavior answered.」にする。`use` 行を `use kivact_editing::host::{EditLayout, Journal, Limits};`、`use kivact_editing::{EditContext, EditOutcome, EditRequest, EditState};`、`use crate::text_input::{EditLines, SharedLines};` にする。
  6. 行の cache の共有（EA-11・EA-12・EA-16）：
     - `src/measure/editor.rs`：`EditorNode` の `pub(crate) lines: RefCell<Option<crate::text_input::EditLines>>,` を `pub(crate) lines: Rc<RefCell<Option<crate::text_input::EditLines>>>,` にし、doc の最後に `/// Shared, for one request, with an editing behavior's context (\`text_input::SharedLines\`).` の 1 行を足す（上の印の行はそのまま）。`EditorNode::new` の `lines: RefCell::default(),` を `lines: Rc::default(),` にする（`Rc` は同じファイルが `use` している）。
     - `src/measure/input.rs`：`refresh_lines` の `let cache = if measurement { &node.measure_lines } else { &node.lines };` の `&node.lines` を `&*node.lines` にする（二つの腕の型を `&RefCell<..>` に揃える）。
     - `src/host/testing/self_check.rs`：次を実行する（`Rc` の中の `RefCell` は `get_mut` できない）。
       ```
       perl -pi -e 's/std::mem::replace\(node\.lines\.get_mut\(\), independent\)/std::mem::replace(&mut *node.lines.borrow_mut(), independent)/; s/\.get_mut\(\) = original;/.borrow_mut() = original;/' crates/kivact-engine/src/host/testing/self_check.rs
       ```
     - `src/model/interact/sync.rs` の `editor.lines = Default::default();` は型が変わっても同じ意味（新しい空の cache）なので触らない。
     - `src/text_input/shaping.rs`：`struct Shaping<'a>`（doc を含む）、`impl<'a> Shaping<'a>`、`impl Lines for Shaping<'_>` を消し、`use kivact_editing::{LineColors, LineState, Lines, TextGeometry, caret_at};` から `Lines` を消す。ファイルの最後に次を足す（`use std::cell::RefCell;` と `use std::rc::Rc;` を足す）。
       ```rust
       /// An input's lines as an editing behavior reads them for one request:
       /// the input's own line cache and the shaper, shared, shaping only the
       /// lines a query asks about. It answers for the revision it was made for
       /// while nobody else holds the cache or the shaper.
       pub(crate) struct SharedLines {
           lines: Rc<RefCell<Option<EditLines>>>,
           engine: Rc<RefCell<arvane::Engine>>,
           revision: u64,
       }

       impl SharedLines {
           /// The lines of the input whose line cache is `lines`, of `revision`.
           pub(crate) fn new(
               lines: Rc<RefCell<Option<EditLines>>>,
               engine: Rc<RefCell<arvane::Engine>>,
               revision: u64,
           ) -> Self {
               SharedLines {
                   lines,
                   engine,
                   revision,
               }
           }

           /// `f` on the lines, when they answer.
           fn read<T>(&self, f: impl FnOnce(&EditLines) -> T) -> Option<T> {
               let lines = self.lines.try_borrow().ok()?;
               lines.as_ref().filter(|l| self.answers(l)).map(f)
           }

           /// `f` on the lines and the shaper (it may shape a line), when the
           /// lines answer.
           fn shape<T>(&self, f: impl FnOnce(&mut EditLines, &mut arvane::Engine) -> T) -> Option<T> {
               let mut lines = self.lines.try_borrow_mut().ok()?;
               let lines = lines.as_mut().filter(|l| self.answers(l))?;
               let mut engine = self.engine.try_borrow_mut().ok()?;
               Some(f(lines, &mut engine))
           }

           /// The lines are of the revision and show the value.
           fn answers(&self, lines: &EditLines) -> bool {
               lines.revision() == self.revision && lines.shows_value()
           }
       }

       impl kivact_editing::host::Lines for SharedLines {
           fn count(&self) -> Option<usize> {
               self.read(EditLines::len)
           }
           fn line_of(&self, at: usize) -> Option<usize> {
               self.read(|l| l.line_of(at))
           }
           fn content(&self, i: usize) -> Option<Range<usize>> {
               self.read(|l| l.content(i)).flatten()
           }
           fn caret_x(&self, text: &str, at: usize, width: f64) -> Option<f64> {
               self.shape(|l, engine| l.caret(text, at, width, engine).x)
           }
           fn nearest(&self, text: &str, i: usize, x: f64, width: f64) -> Option<usize> {
               self.shape(|l, engine| {
                   l.content(i)
                       .map(|_| l.nearest(text, i, x, width, engine))
               })
               .flatten()
           }
       }
       ```
       （`caret` と `nearest` は今の `EditLines` の method で、`nearest` は `TextGeometry::hit_scalar_in_line` を通る。caret の停止位置は grapheme と shaping cluster の共通の境界だけで、行の外の別の実装を作らない。）
     - `src/text_input.rs`：`pub(crate) use shaping::Shaping;` を `pub(crate) use shaping::SharedLines;` にする。
     - `crates/kivact-editing/src/host.rs`（E09 が作った）に `pub use crate::behavior::{EditLayout, Lines};` の行を足し、`lib.rs` の `pub use behavior::{EditBehavior, EditContext, EditLayout, EditOutcome, EditRequest, Lines};` を `pub use behavior::{EditBehavior, EditContext, EditOutcome, EditRequest};` にする。
     - shaper を behavior の間に借りっぱなしにしない：host は shaper を `Rc` のまま model に渡し、model は自分が使う間だけ借りる。
       ```
       perl -pi -e 's/model\.key\(input, physical, &mut self\.engine\.borrow_mut\(\)\)/model.key(input, physical, &self.engine)/; s/model\.paste\(node, &text, &mut self\.engine\.borrow_mut\(\)\)/model.paste(node, &text, &self.engine)/' crates/kivact-engine/src/host/events.rs
       ```
       `src/model/keys.rs` の `key`、`paste`、`handle_key`、`paste_input`、`edit_input_with` の引数 `engine: &mut arvane::Engine` を `engine: &Rc<RefCell<arvane::Engine>>` にする（`use std::cell::RefCell;` と `use std::rc::Rc;` を足す）。`key` と `paste` の中の `engine` の受け渡しはそのまま（型が変わるだけ）。`lay_out_lines` は `&mut arvane::Engine` のまま。
     - `rg -n "borrow_mut\(\)" crates/kivact-engine/src/host/events.rs | rg "model\.(key|paste)"` が空、`rg -n "model\.(key|paste)\(" crates/kivact-engine/src` が `&self.engine` を渡す 2 行だけを返すことを確かめる。
  7. kivact-core：
     - `src/custom/text_editing.rs` の `behavior<F>` の `where` を `F: Fn(&mut EditState, EditRequest, &EditContext) -> EditOutcome + 'static,` にする。
     - `src/view/editing_data.rs` の `pub(crate) type Behavior = dyn Fn(&mut EditState, EditRequest<'_>, &EditContext<'_>) -> EditOutcome;` を `dyn Fn(&mut EditState, EditRequest, &EditContext) -> EditOutcome;` にする。
     - `src/view/mount/editing.rs` の閉包はそのまま（型が変わるだけ）。`cargo check -p kivact-core` の error があれば寿命の `<'_>` を消す。
  8. kivact-interaction `src/text_edit/policy.rs`：
     - `text_edit_policy` の署名を `(state: &mut EditState, request: EditRequest, context: &EditContext) -> EditOutcome` にする。`return EditOutcome::Done { changed: true };` を `return EditOutcome::Done;` に、最後の `EditOutcome::Done { changed }` を `EditOutcome::Done` にする。
     - `vertical_key` と `vertical_target` を次にする。
       ```rust
       fn vertical_key(state: &mut EditState, up: bool, shift: bool, context: &EditContext) -> EditOutcome {
           let goal = state.goal_x().or_else(|| context.x_of(state, state.caret()));
           match vertical_target(state, up, context, goal) {
               Some(to) if to != state.caret() => {
                   match goal {
                       Some(goal) => state.move_vertically(to, shift, goal),
                       None => state.move_to(to, shift),
                   }
                   EditOutcome::Done
               }
               _ => EditOutcome::NotEditing,
           }
       }

       /// Where Up (`up`) or Down moves the caret of a multiline editor: the
       /// offset under the goal column `goal` on the laid-out line above or
       /// below; the start of the text from the first line and its end from the
       /// last. `None` without layout.
       fn vertical_target(state: &EditState, up: bool, context: &EditContext, goal: Option<f64>) -> Option<usize> {
           let line = context.line_of(state, state.caret())?;
           if up && line == 0 {
               return Some(0);
           }
           if !up && line + 1 >= context.line_count(state)? {
               return Some(state.value().len());
           }
           let next = if up { line - 1 } else { line + 1 };
           context.offset_at(state, next, goal?)
       }
       ```
     - `command_key`：`EditOutcome::Done { changed: false }` を `EditOutcome::Done` に、`EditOutcome::Copy { text: ..., changed: false }` を `EditOutcome::Copy(state.selected_text().to_string())` に、cut の `EditOutcome::Copy { text, changed: true }` を `EditOutcome::Copy(text)` にする。`'a'` の `state.move_to(0, false); state.move_to(state.value().len(), true);` を `state.select(0..state.value().len());` にする。
  9. kivact-ui `src/code_editing.rs` の `code_keys`：署名の寿命を消す（`request: EditRequest`、`use_context: &EditContext`）。`Key::ArrowUp | Key::ArrowDown => { ... use_context.offset_at(state, next, use_context.x_of(state, caret)?) }` を `use_context.offset_at(state, next, state.goal_x().or_else(|| use_context.x_of(state, caret))?)` にし、`match to { Some(to) => { state.move_to(to, modifiers.shift); EditOutcome::Done { changed: false } } ... }` を次にする。
     ```rust
     match to {
         Some(to) => {
             let vertical = matches!(key, Key::ArrowUp | Key::ArrowDown);
             match state.goal_x().or_else(|| use_context.x_of(state, caret)).filter(|_| vertical) {
                 Some(goal) => state.move_vertically(to, modifiers.shift, goal),
                 None => state.move_to(to, modifiers.shift),
             }
             EditOutcome::Done
         }
         None => text_edit_policy(state, request, use_context),
     }
     ```
  10. kivact-ui-soft `src/rich_blocks.rs`：`key_command` の `request: &EditRequest<'_>, context: &EditContext<'_>` を `request: &EditRequest, context: &EditContext` にする。他の寿命の error を同じ規則で直す。
  11. `dev/tests/kivact-public-api/tests/e2e_core/text_editing.rs` に次の試験を足す（`up_and_down_move_the_caret_between_lines_of_a_text_area` の後）。
      ```rust
      /// Down through a short line and on to a long one comes back to the
      /// column it started from (the goal column), and so does Up.
      #[test]
      fn up_and_down_keep_their_column_across_a_short_line() {
          let (screen, value, _log) = field_screen(
              "abcdef
      ab
      abcdef",
              true,
          );
          let notes = screen.get_by_role(Role::TextBox, "Notes");
          notes.press_key(Key::End);
          notes.press_key(Key::ArrowUp);
          notes.press_key(Key::ArrowUp);
          screen.type_text("X");
          assert_eq!(
              value.get(),
              "abcdefX
      ab
      abcdef",
              "Up through the short line keeps the column of the end"
          );
          notes.press_key(Key::ArrowDown);
          notes.press_key(Key::ArrowDown);
          screen.type_text("Y");
          assert_eq!(
              value.get(),
              "abcdefX
      ab
      abcdefY",
              "Down through the short line comes back to the column"
          );
      }
      ```
       （`field_screen` は同じファイルの helper。一つ目の `type_text` の後、caret は 1 行目の 7 文字目にあり、goal は消える。Down は 7 文字目の x を goal に、2 行目の端を経て 3 行目の端（6 文字目の後）へ行く。）
  12. `dev/tests/kivact-public-api/tests/e2e_core/custom_text_field.rs`：先頭の `use kivact::custom::{NodeRefExt, TextEditing, TextSession, inline_view};` を `use kivact::custom::{EditContext, EditOutcome, EditRequest, EditState, NodeRefExt, TextEditing, TextSession, inline_view};` にし、ファイルの最後に次を足す（`mount` は同じファイルの他の試験と同じ import のもの）。
      ```rust
      /// The text of the page test: 60 lines `line 00` to `line 59`, 8 bytes a
      /// line with its break.
      fn page_text() -> String {
          (0..60).map(|i| format!("line {i:02}")).collect::<Vec<_>>().join("\n")
      }

      /// PageUp and PageDown move the caret 40 lines to the same `x`, through
      /// the public line queries alone; every other key is the standard policy.
      fn page_keys(state: &mut EditState, request: EditRequest, context: &EditContext) -> EditOutcome {
          let (key, modifiers) = match request {
              EditRequest::Key {
                  key: key @ (Key::PageUp | Key::PageDown),
                  modifiers,
              } => (key, modifiers),
              request => return text_edit_policy(state, request, context),
          };
          let caret = state.caret();
          let to = context.line_of(state, caret).and_then(|line| {
              let last = context.line_count(state)?.checked_sub(1)?;
              let next = if key == Key::PageUp {
                  line.saturating_sub(40)
              } else {
                  (line + 40).min(last)
              };
              context.offset_at(state, next, context.x_of(state, caret)?)
          });
          match to {
              Some(to) => {
                  state.move_to(to, modifiers.shift);
                  EditOutcome::Done
              }
              None => EditOutcome::NotEditing,
          }
      }

      /// A behavior built from the public line queries reaches lines far from
      /// the caret: a page (40 lines) up and down in a 60-line document.
      #[test]
      fn a_behavior_moves_a_page_with_the_line_queries() {
          let (screen, pages) = mount(|| {
              let pages = node_ref();
              let view = node()
                  .node_ref(pages)
                  .role(Role::TextBox)
                  .name("Pages")
                  .focusable(true)
                  .style(style().size(420, 180).font_size(16).font_family("monospace"))
                  .text_editing(TextEditing::new(page_text()).multiline().behavior(page_keys));
              (pages, view)
          });
          let field = screen.get_by_role(Role::TextBox, "Pages");
          let caret = || pages.edit_state().expect("mounted native editor").caret();
          field.press_key(Key::End);
          assert_eq!(caret(), 59 * 8 + 7, "End goes to the end of `line 59`");
          field.press_key(Key::PageUp);
          assert_eq!(caret(), 19 * 8 + 7, "40 lines up: the end of `line 19`");
          field.press_key(Key::PageUp);
          assert_eq!(caret(), 7, "the first page stops at the first line");
          field.press_key(Key::PageDown);
          assert_eq!(caret(), 40 * 8 + 7, "40 lines down: the end of `line 40`");
          assert!(screen.errors().is_empty(), "{:?}", screen.errors());
      }
      ```
      （前の 3 行の窓の `EditContext` では、19 行目は caret の近くに無いので `offset_at` が `None` を返し、この試験は落ちる：EA-16 の試験。）
  13. `crates/kivact-editing/tests/geometry.rs` を作る（`ShapedText` は `src/geometry.rs` の `use` 行と同じ path。1.2 節）。
      ```rust
      //! Role: Tests that a caret stop is never inside a shaping cluster.

      use kivact_editing::TextGeometry;
      use kivact_types::render::ShapedText;
      use kivact_types::{Point, Rect, TextCaret};

      /// `ffix` shaped with `ffi` as one cluster (a ligature): carets 10 px
      /// apart at every scalar, stops at scalars 0, 3 and 4.
      fn ligature() -> ShapedText {
          ShapedText {
              carets: (0..=4u32)
                  .map(|offset| TextCaret {
                      offset,
                      rect: Rect {
                          x: 10.0 * f64::from(offset),
                          y: 0.0,
                          width: 1.0,
                          height: 20.0,
                      },
                  })
                  .collect(),
              cluster_boundaries: vec![0, 3, 4],
              ..ShapedText::default()
          }
      }

      #[test]
      fn a_hit_or_a_move_never_lands_inside_a_shaping_cluster() {
          let shaped = ligature();
          let geometry = TextGeometry::new(&shaped, "ffix");
          let bounds = Rect {
              x: 0.0,
              y: 0.0,
              width: 40.0,
              height: 20.0,
          };
          let hit = |x: f64| geometry.hit_scalar_in_line(Point { x, y: 0.0 }, 0..4, bounds);
          assert_eq!(hit(9.0), 0, "the caret at 10 px is inside the cluster: the start is the nearest stop");
          assert_eq!(hit(21.0), 3, "the caret at 20 px is inside the cluster: its end is the nearest stop");
          assert_eq!(hit(38.0), 4);
          assert_eq!(geometry.next_boundary(0), 3, "a move steps over the whole cluster");
          assert_eq!(geometry.prev_boundary(4), 3);
      }
      ```
      （`EditContext::offset_at` は engine の `SharedLines::nearest` から `EditLines::nearest`、この `hit_scalar_in_line` を通る：host の答えは同じ規則。）
  14. `rg -n "changed: (true|false)|EditRequest<'_>|EditContext<'_>|EditLayout<'|LineSnapshot|SnapshotLine|max_length" crates/kivact-editing/src crates/kivact-interaction/src crates/kivact-ui/src/code_editing.rs crates/kivact-ui-soft/src/rich_blocks.rs crates/kivact-core/src` が、`max_length` の node の prop（`EditingData::max_length`、`Writer::set_max_length`、core の `max_length` の setter）だけを返すことを確かめる。`rg -n "\bShaping\b|fn stops\b|fn snapshot\b" crates/kivact-engine/src` が空、`rg -n "chars\(\)\.count\(\)" crates/kivact-engine/src/model/keys.rs` が `limits.max_length.map(` の中の 2 か所（`edit_editor_with` と `commit_composition`）だけを返すことを確かめる。
- **消すもの**：`EditOutcome::Done { changed }`・`Copy { text, changed }` の形、`EditRequest` の寿命、`EditContext` の寿命と公開の constructor（`#[doc(hidden)]` になる）、`EditLayout<'a>`（寿命の無い `kivact_editing::host::EditLayout` になる）、crate root の `EditLayout`・`Lines` の再公開、`Lines::{shows_value, len, is_empty, caret}`（`count`・`caret_x` と `None` を返す形になる）、`EditState::{max_length, set_max_length}`、`Editor::journal` の `max_length` 引数、`edit_editor_with` の「申告と違えば戻す」処理と、上限の無い editor での text の数え、`Shaping` と `impl Lines for Shaping`、`fn context`、host の `model.key`・`model.paste` に渡す shaper の借用。
- **状態の閉じ方**：`EditContext` の bool は三つとも constructor だけで書く値（`&mut self` の method が無い）で対象外。`EditLayout` と `SharedLines` は constructor だけで書く値（`&mut self` の method が無い）で対象外。`EditorNode::lines` は型だけが変わり、印の文はそのまま。`Limits` は値。`TextSession` は `goal_x: Option<f64>` と `marked: Option<Range>` の二つの Option を持ち、`EditState` の method が書く：`TextSession` の直前に `// xtask-allow(state): marked is the IME composition; goal_x is the column of the last vertical move (move_vertically sets it, every other move or edit clears it); independent` を置く。
- **試験**：手順 11 の `text_editing::up_and_down_keep_their_column_across_a_short_line`、手順 12 の `custom_text_field::a_behavior_moves_a_page_with_the_line_queries`（EA-16）、手順 13 の `geometry::a_hit_or_a_move_never_lands_inside_a_shaping_cluster`（EA-12）、測定の `text_editing::large_document_key_latency`（EA-11。`#[ignore]`、手で実行する）を足す。
- **確認（tier a）**：
  ```
  EDITING-A
  ENGINE-A
  cargo check -p kivact-core -p kivact-interaction -p kivact-ui -p kivact-ui-soft --all-targets
  cargo clippy -p kivact-core -p kivact-interaction -p kivact-ui -p kivact-ui-soft --all-targets -- -D warnings
  cargo xtask check --no-write --crate kivact-interaction
  cargo xtask check --no-write --crate kivact-ui
  cargo test -p kivact-editing --test geometry -- a_hit_or_a_move_never_lands_inside_a_shaping_cluster
  E2E: text_editing, custom_text_field, input_platform, keyboard, components, launch_real_window
  E2E: interaction_edit, ui_chat_code, ui_field, ui_controls
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-ui-soft
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test --release -p kivact-public-api-tests --test e2e_core -- text_editing::large_document_key_latency --ignored --nocapture > target/e17-latency-after.txt 2>&1
  ```
- **完了条件**：手順 14 の `rg` のとおり。新しい試験を含め確認がすべて通る（どのコマンドも 0 件でない。1.4 節）。`grep latency target/e17-latency-before.txt target/e17-latency-after.txt` の 5 行（`lines` の三つと `one line` の二つ）それぞれで、after の p95 が before の p95 の 1.05 倍以下。超えた行があれば STOP し、両方の file を設計者に渡す。

## E18：`NodeInput` の 8 個の `Option<bool>` を bit にする

- **wave / lane / 前提**：W5a / 鎖 / E17
- **所有**：`src/contract/node.rs`、`src/contract.rs`、`src/host/write/typed.rs`、`src/tree/node.rs`、`src/model/apply.rs`、`src/model/pointer/shell.rs`、`crates/kivact-core/src/view/tree.rs`、`crates/kivact-test/src/facts.rs`
- **目的**：`NodeInput` の 8 個の三値（未指定・偽・真）を 2 bit ずつの `InputFlags(u16)` にし、`NodeProperties` を 8 byte 縮める（EN-23）。kivact-core と kivact-test が読む field が変わるので W5a の鎖に置く。
- **手順**：
  1. `src/contract/node.rs`：`NodeInput` を次にし、型を足す。`src/contract.rs` の再公開の並びに `InputFlag, InputFlags` を足す。
     ```rust
     /// Optional input flags; absent values leave the host defaults in force.
     #[derive(Clone, Debug, PartialEq, Default)]
     pub struct NodeInput {
         /// The yes/no/unset input flags ([`InputFlag`]).
         pub flags: InputFlags,
         /// Group descendants into one stop in the native Tab order.
         pub roving_focus: Option<RovingFocus>,
         /// Where a press moves the focus instead of this node; `None`: the host default.
         pub focus_on_press: Option<FocusTarget>,
     }

     /// One input flag of a node (see [`InputFlags`]).
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum InputFlag {
         /// Disable interactive behavior when true.
         Disabled,
         /// Exclude the node and subtree from display and interaction when true.
         Hidden,
         /// Allow the node to receive keyboard focus.
         Focusable,
         /// Keep an otherwise focusable node in focus order while disabled.
         FocusableWhenDisabled,
         /// Request routed pointer events.
         ListenPointer,
         /// Request wheel events for the node.
         ListenWheel,
         /// Request native file drag-and-drop events.
         ListenFile,
         /// `false`: the pointer passes through the node and its subtree; unset is `true`.
         HitTestable,
     }

     /// Each [`InputFlag`] unset, false or true, in two bits.
     #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
     pub struct InputFlags(u16);

     impl InputFlags {
         /// The value of `flag` (`None`: unset).
         pub fn get(self, flag: InputFlag) -> Option<bool> {
             match (self.0 >> (2 * flag as u16)) & 0b11 {
                 0 => None,
                 1 => Some(false),
                 _ => Some(true),
             }
         }

         /// Set `flag` to `value`.
         pub fn set(&mut self, flag: InputFlag, value: Option<bool>) {
             let shift = 2 * flag as u16;
             let bits = match value {
                 None => 0,
                 Some(false) => 1,
                 Some(true) => 2,
             };
             self.0 = (self.0 & !(0b11 << shift)) | (bits << shift);
         }
     }
     ```
     `const _: () = assert!(std::mem::size_of::<NodeProperties>() <= 240, ...)` を `<= 232` にする（根拠：8 個の `Option<bool>`（8 byte）が `u16`（2 byte）になり、`NodeInput` は整列 8 で 8 byte 縮む。`FocusTarget` を含むので `NodeInput` の整列は 8）。
  2. `src/model/apply.rs`：今の `set_prop` の本体を、値の比較と書込みを閉包で受ける `store_prop` に移す。
     - 新しい関数の署名（本体は今の `set_prop` の本体）：
       ```rust
       /// Store one typed property of `id` when `differs` says the value
       /// differs (`None`: the node's kind has no such property) and record
       /// what it outdates; `store` writes the value, once.
       fn store_prop<V>(
           &mut self,
           id: NodeId,
           value: V,
           differs: impl Fn(&mut NodeProperties, &V) -> Option<bool>,
           store: impl FnOnce(&mut NodeProperties, V),
           mut mutation: Mutation,
       ) -> bool
       ```
     - 本体の中の `let Some(old) = field(&mut node.properties) else { return false; }; if *old == value { return false; }` を `if differs(&mut node.properties, &value) != Some(true) { return false; }` にする。
     - 本体の四つの枝の書込み（`*field(&mut node.properties).expect(..) = value;` の三か所と `*old = value;`）を、それぞれ `store(&mut node.properties, value);` にする（`node` は各枝で今のとおり取り直す。四つの枝のどれか一つだけが走るので `store` は一度だけ呼ばれる）。
     - `set_prop` を次にする。
       ```rust
       pub(crate) fn set_prop<T: PartialEq>(
           &mut self,
           id: NodeId,
           value: Option<T>,
           field: impl Fn(&mut NodeProperties) -> Option<&mut Option<T>>,
           mutation: Mutation,
       ) -> bool {
           self.store_prop(
               id,
               value,
               |p, value| field(p).map(|old| *old != *value),
               |p, value| {
                   if let Some(old) = field(p) {
                       *old = value;
                   }
               },
               mutation,
           )
       }
       ```
     - 次を足す（`use crate::contract::InputFlag;`）。
       ```rust
       /// Store the input flag `flag` of `id` as `value` when it differs and
       /// record what it outdates (as `set_prop`).
       pub(crate) fn set_input_flag(
           &mut self,
           id: NodeId,
           flag: InputFlag,
           value: bool,
           mutation: Mutation,
       ) -> bool {
           self.store_prop(
               id,
               value,
               |p, value| Some(p.input.flags.get(flag) != Some(*value)),
               |p, value| p.input.flags.set(flag, Some(value)),
               mutation,
           )
       }
       ```
  3. `src/host/write/typed.rs`：`props!` の macro に store の腕を足す。
     ```rust
     (@store $w:ident, $id:ident, $v:ident, $m:ident, flag($flag:expr)) => {
         $w.record($id, |model| model.set_input_flag($id, $flag, $v, $m))
     };
     ```
     8 個の行の `=> field(Some(v), |p| Some(&mut p.input.X)),` を `=> flag(crate::contract::InputFlag::Y),` にする（`disabled`→`Disabled`、`hidden`→`Hidden`、`hit_testable`→`HitTestable`、`focusable`→`Focusable`、`focusable_when_disabled`→`FocusableWhenDisabled`、`listen_pointer`→`ListenPointer`、`listen_wheel`→`ListenWheel`、`listen_file`→`ListenFile`）。`roving_focus` と `focus_on_press` はそのまま。
  4. `src/tree/node.rs`：`self.properties.input.X.unwrap_or(d)` を `self.properties.input.flags.get(InputFlag::Y).unwrap_or(d)` に、`self.properties.input.focusable_when_disabled == Some(true)` を `self.properties.input.flags.get(InputFlag::FocusableWhenDisabled) == Some(true)` にする。`src/model/pointer/shell.rs` の `n.properties.input.listen_file.unwrap_or(false)` を `n.properties.input.flags.get(InputFlag::ListenFile).unwrap_or(false)` にする。
  5. `crates/kivact-core/src/view/tree.rs` と `crates/kivact-test/src/facts.rs`：`node.input.hidden` を `node.input.flags.get(InputFlag::Hidden)`、`node.input.disabled` を `node.input.flags.get(InputFlag::Disabled)`、`node.input.focusable` を `node.input.flags.get(InputFlag::Focusable)`、`node.input.focusable_when_disabled` を `node.input.flags.get(InputFlag::FocusableWhenDisabled)` にする（`== Some(true)` の比較はそのまま残る）。`use kivact_engine::contract::InputFlag;` を足す。
  6. `rg -n "input\.(disabled|hidden|focusable|focusable_when_disabled|listen_pointer|listen_wheel|listen_file|hit_testable)\b" crates` が空であることを確かめる。
- **消すもの**：`NodeInput` の 8 個の `Option<bool>` の field。
- **状態の閉じ方**：`InputFlags` は `&mut self` の method `set` を持つが、bool・Option・数の field を持たない（一つの `u16`）ので対象外。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`cargo check -p kivact-core -p kivact-test --all-targets`、`cargo clippy -p kivact-core -p kivact-test --all-targets -- -D warnings`、`E2E: focus, hover, pointer, components, keyboard, view, flows, drawables, lifecycle`、`E2E: apg_toolbar, apg_radio_group, interaction_list_nav`
- **完了条件**：手順 6 の `rg` が空。`NodeProperties` の assert が `<= 232` で build が通る。確認がすべて通る。

## E19：host の要求を App の一つの列に、frame の記録を一つに

- **wave / lane / 前提**：W6b / E1 / なし（E15 は W5a で済んでいる）
- **所有**：`src/model/aggregate.rs`、`src/model.rs`、`src/model/app_model.rs`、`src/host/write/writer.rs`、`src/host/pump.rs`、`src/host/launch.rs`、`src/embed.rs`、`src/frame_log.rs`、`src/lib.rs`
- **目的**：`DeferredRequest`（`HostRequest` の写し）と window ごとの `Model.effects` を消し、App の一つの FIFO `AppModel.effects: Vec<(RequestId, HostRequest)>` にする（EN-16）。frame の記録は一つの `FrameRecord` を作り、file と DevTools の ring の二つの受け手に渡す。renderer の変化の記録を一つの関数にする（EN-19）。
- **手順**：
  1. `src/model/aggregate.rs`：`enum DeferredRequest` と、`struct Model` の `effects` の field（doc を含む）と `Model::new` の `effects: vec![],` を消す。`src/model.rs` の `pub(crate) use aggregate::{DeferredRequest, Model};` を `pub(crate) use aggregate::Model;` にする。
  2. `src/model/app_model.rs`：`struct AppModel` に次を足し、`new` で `effects: Vec::new(),` にする。
     ```rust
     /// The platform and App-wide requests the host carries out after the
     /// turn, in write order (`Host::effects`).
     pub(crate) effects: Vec<(RequestId, crate::contract::HostRequest)>,
     ```
  3. `src/host/write/writer.rs`：`Writer::effect` の `match` を次にし、`fn queue_main` を消して `fn queue_effect` を足す。
     ```rust
     pub fn effect(&self, request: RequestId, effect: HostRequest) {
         let mut app = self.app.borrow_mut();
         match effect {
             // (the `HostRequest::Snapshot(node)` arm, unchanged)
             HostRequest::RegisterFont(font) => self.register_font(&mut app, request, font),
             drawable @ (HostRequest::DrawableExport { .. } | HostRequest::SetDrawableGeometry { .. }) => {
                 apply_drawable_effect(&mut app, drawable)
             }
             effect => queue_effect(&mut app, request, effect),
         }
     }

     /// Queue `effect` for the host after the turn; a window's request only
     /// while the window is open. A request that answers (all but a GPU
     /// demand and an unhandled key) is a command of the write.
     fn queue_effect(app: &mut AppModel, request: RequestId, effect: HostRequest) {
         let window = match &effect {
             HostRequest::Platform { window, .. }
             | HostRequest::Inject { window, .. }
             | HostRequest::DemandGpu { window }
             | HostRequest::UnhandledKey { window, .. } => Some(*window),
             _ => None,
         };
         if window.is_some_and(|w| !app.windows.contains_key(&w)) {
             return;
         }
         if !matches!(effect, HostRequest::DemandGpu { .. } | HostRequest::UnhandledKey { .. }) {
             app.writes.command(None);
         }
         // The shell events of the window come from this injection until it is delivered (E30).
         if let HostRequest::Inject { window, .. } = &effect {
             app.injecting.entry(*window).or_default().push_back(request);
         }
         app.effects.push((request, effect));
     }
     ```
  4. `src/host/pump.rs`：
     - `effects` を次の形にする。各腕の中身は、元の `DeferredRequest::X` の腕の中身を写し、元の `id`（window）は腕で取り出した `window` を使う。
       ```rust
       pub(super) fn effects(&mut self, windows: &mut Windows<'_>) {
           if matches!(self.phase, HostPhase::Starting { .. }) || self.closed() {
               return;
           }
           let effects = std::mem::take(&mut self.app.borrow_mut().effects);
           for (request, effect) in effects {
               let result = match effect {
                   HostRequest::Platform { window, request: PlatformRequest::RequestWindowSize(size) } => {
                       // (the old RequestWindowSize arm, with `id` = `window`)
                   }
                   HostRequest::Platform { window, request: platform } => {
                       // (the old Platform arm, with `id` = `window`)
                   }
                   HostRequest::ReducedMotion(override_value) => {
                       // (the old ReducedMotion arm)
                   }
                   HostRequest::Inject { window, inputs } => {
                       // (the old Inject arm, with `id` = `window`; it ends with `continue`)
                   }
                   HostRequest::Emulate(emulation) => {
                       // (the old Emulate arm)
                   }
                   HostRequest::TimeScale(scale) => {
                       // (the old TimeScale arm as E01 left it)
                   }
                   HostRequest::DemandGpu { window } => {
                       windows.demand_gpu(window);
                       continue;
                   }
                   HostRequest::UnhandledKey { window, code } => {
                       if let Some(mut w) = windows.get(window) {
                           w.unhandled_key(code);
                       }
                       continue;
                   }
                   // Answered in the write (`Writer::effect`): never queued.
                   HostRequest::Snapshot(_)
                   | HostRequest::RegisterFont(_)
                   | HostRequest::DrawableExport { .. }
                   | HostRequest::SetDrawableGeometry { .. } => continue,
               };
               if self.closed() {
                   return;
               }
               self.app
                   .borrow()
                   .push_event(HostEvent::EffectResult { request, result });
           }
       }
       ```
     - `control` の `let effects_pending = app.windows.values().any(|model| !model.effects.is_empty());` を `let effects_pending = !app.effects.is_empty();` にする。`/// Apply the emulation to every window after \`DeferredRequest::Emulate\`.` の doc を `HostRequest::Emulate` にする。`use` から `DeferredRequest` を消し、`HostRequest` を足す。
  5. `src/frame_log.rs`：
     - `struct FrameTimes` を消す。`FrameLog::frame(&mut self, times: FrameTimes, stats: &FrameStats)` を `FrameLog::frame(&mut self, record: &crate::contract::FrameRecord)` にし、`F` の行は `record.work_ns`、`record.scene_ns`、`record.raster_ns`、`record.present_ns` を、`S` の行は `record.stats` の同じ field を書く（行の形は変えない。`end` は今のとおり書く時の UNIX の ns）。
     - 次を足す。
       ```rust
       /// The observer of Arvane's renderer changes that notes each in `log`
       /// (benchmarks parse the `renderer=` notes; the first is the renderer
       /// the window starts with).
       pub(crate) fn renderer_logger(
           log: Rc<RefCell<FrameLog>>,
       ) -> impl FnMut(kivact_types::WindowId, &arvane::RendererChange) + 'static {
           move |_window, change| {
               let mut log = log.borrow_mut();
               match change {
                   arvane::RendererChange::Start { .. } => log.note(&format!("host=arvane {change}")),
                   _ => log.note(&change.to_string()),
               }
           }
       }
       ```
       （`on_renderer_change` の閉包の第一引数の型が `kivact_types::WindowId` でなければ、`arvane::Shell::on_renderer_change` の署名の型にする。）
  6. `src/lib.rs`：`pub(crate) use frame_log::{FrameLog, FrameTimes};` を `pub(crate) use frame_log::FrameLog;` にする。
  7. `src/host/launch.rs`：`let log = frame_log.clone();` と `shell.on_renderer_change(move |_window, change| { ... })` を `shell.on_renderer_change(crate::frame_log::renderer_logger(frame_log.clone()))` にする。`src/embed.rs` の `let log = frame_log.clone();` と `Embedder::new(host).on_renderer_change(move |_window, change| { ... })` を `Embedder::new(host).on_renderer_change(crate::frame_log::renderer_logger(log))` にし、`let log = frame_log.clone();` は `host::new(..., frame_log)` の前に残す。
  8. `src/host/pump.rs` の `present` の末尾の `if logging { let times = FrameTimes { ... }; self.record_frame(id, end_ns, times, stats, *renderer); }` と `fn record_frame` を次にする。
     ```rust
     if logging {
         let (revision, frame_period_ns) = {
             let app = self.app.borrow();
             let period = app.model(id).map_or(0, |model| model.frame.period_ns());
             (app.events().applied(), period)
         };
         self.record_frame(FrameRecord {
             window: id,
             revision,
             end_ns,
             work_ns: (Instant::now() - started).as_nanos() as u64,
             scene_ns: (t_raster - t_scene).as_nanos() as u64,
             raster_ns: raster.as_nanos() as u64,
             present_ns: present.as_nanos() as u64,
             stats: *stats,
             gpu: matches!(renderer, arvane::Renderer::Gpu),
             frame_period_ns,
         });
     }
     ```
     ```rust
     /// Give one shown frame's record to the frame log and the DevTools' ring.
     fn record_frame(&self, record: FrameRecord) {
         {
             let mut frame_log = self.frame_log.borrow_mut();
             if frame_log.enabled() {
                 frame_log.frame(&record);
             }
         }
         let log = self.app.borrow().log.clone();
         if log.frames_recording() {
             log.frame(|| record);
         }
     }
     ```
  9. `rg -n "DeferredRequest|FrameTimes|queue_main|\.effects\b" crates/kivact-engine/src` が `AppModel.effects` とその読み書き（`app_model.rs`、`writer.rs`、`pump.rs`）だけを返すことを確かめる。
- **消すもの**：`DeferredRequest`、`Model::effects`、`queue_main`、`FrameTimes`、launch と embed の閉包の重複。
- **状態の閉じ方**：新しい状態の型は無い（`AppModel` に `Vec` の field が一つ増える）。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: launch_platform, launch_window, environment, platform_paths, keys_unhandled, gpu_on_demand, windows, close_request`、`dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- devtools::`
- **完了条件**：手順 9 の `rg` のとおり。確認がすべて通る。`KIVACT_FRAME_LOG` の file の行の形（`F ...`、`S ...`、`# ...`）は変わらない。

## E20：二度・三度クリックの drag を単語・行で伸ばす、単語と行を editing へ

- **wave / lane / 前提**：W5a / 鎖 / E18（鎖の最後。手順が使う `EditState::select` は E17）
- **所有**：`crates/kivact-editing/src/edit_state.rs`、`src/model/pointer/machine.rs`、`src/model/pointer/state.rs`、`src/model/pointer/touch.rs`、`src/model/pointer/shell.rs`、`src/model/edit/ime.rs`、`src/text_input.rs`、`src/text_input/word.rs`（削除）、`dev/tests/kivact-public-api/tests/e2e_core/text_editing.rs`
- **目的**：二度・三度クリックで選んだ単語・行の範囲を drag の間も保ち、単語・行の境界まで伸ばす。touch と pen も同じ gesture で選ぶ（EN-01）。単語は UAX #29 の単語境界で、`EditState::word_at`・`line_at` として kivact-editing に置き、engine の `text_input/word.rs` を消す（EN-21）。
- **手順**：
  1. `crates/kivact-editing/src/edit_state.rs` に次を足す（`floor_char_boundary` は `kivact_types::text_utf::floor_char_boundary`）。
     ```rust
     /// The word at byte offset `at`: the Unicode word segment (UAX #29)
     /// holding the character at `at`, or the last one at the end of the
     /// text. A run of white space is a segment of its own.
     pub fn word_at(&self, at: usize) -> Range<usize> {
         let at = floor_char_boundary(&self.text, at.min(self.text.len()));
         let mut last = 0..0;
         for (start, segment) in self.text.split_word_bound_indices() {
             let range = start..start + segment.len();
             if range.contains(&at) {
                 return range;
             }
             last = range;
         }
         last
     }

     /// The line at byte offset `at`, without its line feed: between the
     /// line feeds around it when `multiline`, else the whole text.
     pub fn line_at(&self, at: usize, multiline: bool) -> Range<usize> {
         let text = &self.text;
         if !multiline {
             return 0..text.len();
         }
         let at = floor_char_boundary(text, at.min(text.len()));
         let start = text[..at].rfind('\n').map_or(0, |i| i + 1);
         let end = text[at..].find('\n').map_or(text.len(), |i| at + i);
         start..end
     }
     ```
  2. `src/model/pointer/state.rs`：
     - 次を足す。
       ```rust
       /// What a drag that selects text extends by.
       #[derive(Clone, Debug, PartialEq)]
       pub(crate) enum SelectUnit {
           /// Characters: from the press to the pointer.
           Char,
           /// Words: the double-clicked word and the words up to the pointer.
           Word(std::ops::Range<usize>),
           /// Lines: the triple-clicked line and the lines up to the pointer.
           Line(std::ops::Range<usize>),
       }
       ```
     - `MouseGesture::SelectText { editor: NodeId, route: Route }` に `unit: SelectUnit` を足す。`TouchGesture` に `SelectText { editor: NodeId, route: Route, unit: SelectUnit }` を足す。`PenTrack` の `route: Route` を `gesture: PenGesture` にし、次を足す。
       ```rust
       /// What a pen contact is doing.
       #[derive(Clone, Debug, PartialEq)]
       pub(crate) enum PenGesture {
           Route(Route),
           SelectText {
               editor: NodeId,
               route: Route,
               unit: SelectUnit,
           },
       }

       impl Default for PenGesture {
           fn default() -> Self {
               Self::Route(Route::default())
           }
       }
       ```
     - `impl Track` の `route`・`routed_mut`・`route_mut`・`reset_gesture` などで `Pen(t) => &t.route` と書いている所を、`PenGesture::Route(r) | PenGesture::SelectText { route: r, .. }` を読む形にし、`TouchGesture::SelectText { route: r, .. }` も `Route` と同じ扱いに足す（`cargo check` の error が場所を示す。規則：`SelectText` は route を持つ普通の経路として扱う）。`reset_gesture` は pen も `PenGesture::default()` に戻す。
     - `Track` に次を足す。
       ```rust
       /// The editor and unit a drag of this pointer selects text in, if it does.
       pub(crate) fn selecting(&self) -> Option<(NodeId, &SelectUnit)> {
           match self {
               Track::Mouse(t) => match &t.gesture {
                   MouseGesture::SelectText { editor, unit, .. } => Some((*editor, unit)),
                   _ => None,
               },
               Track::Touch(t) => match &t.gesture {
                   TouchGesture::SelectText { editor, unit, .. } => Some((*editor, unit)),
                   _ => None,
               },
               Track::Pen(t) => match &t.gesture {
                   PenGesture::SelectText { editor, unit, .. } => Some((*editor, unit)),
                   PenGesture::Route(_) => None,
               },
           }
       }

       /// Select text in `editor` by `unit` while this pointer is pressed (its route kept).
       pub(crate) fn start_selecting(&mut self, editor: NodeId, unit: SelectUnit) {
           match self {
               Track::Mouse(t) => {
                   if let MouseGesture::Route(route) = &mut t.gesture {
                       t.gesture = MouseGesture::SelectText { editor, route: std::mem::take(route), unit };
                   }
               }
               Track::Touch(t) => {
                   if let TouchGesture::Route(route) = &mut t.gesture {
                       t.gesture = TouchGesture::SelectText { editor, route: std::mem::take(route), unit };
                   }
               }
               Track::Pen(t) => {
                   if let PenGesture::Route(route) = &mut t.gesture {
                       t.gesture = PenGesture::SelectText { editor, route: std::mem::take(route), unit };
                   }
               }
           }
       }

       /// The press ended: a text selection goes back to plain routing.
       pub(crate) fn stop_selecting(&mut self) {
           match self {
               Track::Mouse(t) => {
                   if let MouseGesture::SelectText { route, .. } = &mut t.gesture {
                       t.gesture = MouseGesture::Route(std::mem::take(route));
                   }
               }
               Track::Touch(t) => {
                   if let TouchGesture::SelectText { route, .. } = &mut t.gesture {
                       t.gesture = TouchGesture::Route(std::mem::take(route));
                   }
               }
               Track::Pen(t) => {
                   if let PenGesture::SelectText { route, .. } = &mut t.gesture {
                       t.gesture = PenGesture::Route(std::mem::take(route));
                   }
               }
           }
       }
       ```
  3. `src/model/pointer/machine.rs`：
     - `use crate::text_input::{line_range, word_range};` を消し、`SelectUnit` の `use` を足す。
     - `press_text_input` と `select_text` を次にする。
       ```rust
       /// A primary press over the text input `input` at byte offset `at`
       /// (`clicks`: its click count): a click places the caret (Shift
       /// extends), a double click selects the word there, a triple click the
       /// line. Returns what a drag from it extends by.
       fn press_text_input(&mut self, input: NodeId, at: usize, clicks: u8, extend: bool) -> SelectUnit {
           let multiline = self.tree.get(input).is_some_and(|n| n.multiline());
           let mut unit = SelectUnit::Char;
           match clicks {
               0 | 1 => self.place_caret(input, at, extend),
               _ => {
                   self.edit_input(input, |e| {
                       unit = if clicks == 2 {
                           SelectUnit::Word(e.word_at(at))
                       } else {
                           SelectUnit::Line(e.line_at(at, multiline))
                       };
                       if let SelectUnit::Word(range) | SelectUnit::Line(range) = &unit {
                           e.select(range.clone());
                       }
                   });
               }
           }
           unit
       }

       /// Extend the selection of `input` to the pointer at byte offset `at`
       /// by `unit`: a word or line drag keeps the clicked unit selected and
       /// reaches the end of the unit under the pointer.
       fn extend_selection(&mut self, input: NodeId, at: usize, unit: &SelectUnit) {
           let multiline = self.tree.get(input).is_some_and(|n| n.multiline());
           let origin = match unit {
               SelectUnit::Char => return self.place_caret(input, at, true),
               SelectUnit::Word(origin) | SelectUnit::Line(origin) => origin.clone(),
           };
           let words = matches!(unit, SelectUnit::Word(_));
           self.edit_input(input, |e| {
               let here = if words { e.word_at(at) } else { e.line_at(at, multiline) };
               if here.start < origin.start {
                   e.select(origin.end..here.start);
               } else {
                   e.select(origin.start..here.end.max(origin.end));
               }
           });
       }
       ```
     - `primary_press` の末尾：`self.press_text_input(i, at, button_press.clicks, modifiers.shift);` を `unit = Some(self.press_text_input(i, at, button_press.clicks, modifiers.shift));` にし（その前に `let mut unit = None;`）、`if let Some(editor) = selecting && let Track::Mouse(mouse) = ... && let MouseGesture::Route(route) = &mut mouse.gesture { mouse.gesture = MouseGesture::SelectText { ... }; }` を次にする。
       ```rust
       if let Some(editor) = selecting {
           let unit = unit.unwrap_or(SelectUnit::Char);
           self.pointer.state.track_mut(pointer).start_selecting(editor, unit);
       }
       ```
     - `pointer_up` の `if let Track::Mouse(mouse) = track && let MouseGesture::SelectText { route, .. } = &mut mouse.gesture { mouse.gesture = MouseGesture::Route(std::mem::take(route)); }` を `track.stop_selecting();` にする。
     - `pointer_move` の `let selecting = match self.pointer.state.track(pointer) { ... };` と、それに続く `if let Some(i) = selecting && let Some(at) = space.text_offset(self, i) { self.place_caret(i, at, true); }` を次にする（touch と pen の、pressed の鎖から導く枝を消す）。
       ```rust
       // A primary drag that started in the focused text input selects.
       let selecting = self
           .pointer
           .state
           .track(pointer)
           .selecting()
           .filter(|(editor, _)| {
               held.contains(MouseButton::Primary)
                   && self
                       .focused()
                       .is_some_and(|f| f.node == *editor && f.region.is_none())
           })
           .map(|(editor, unit)| (editor, unit.clone()));
       if let Some((editor, unit)) = selecting
           && let Some(at) = space.text_offset(self, editor)
       {
           self.extend_selection(editor, at, &unit);
       }
       ```
     - `select_text` の関数を消す。
  4. `src/model/pointer/touch.rs` と `src/model/pointer/shell.rs`：`TouchGesture` と `PenTrack` の `match` で `cargo check` が出す error を、手順 2 の規則で直す（`SelectText` は `Route` と同じく経路を持つ。touch の pan の判定は `Route` と `SelectText` のどちらからも始まらない：`Pending` に入るのは `Route` からだけのまま）。
  5. `src/model/edit/ime.rs` の `ime_select`：E05 が書いた `e.move_to(start.min(end), false); e.move_to(start.max(end), true);` を `e.select(start.min(end)..start.max(end));` にする。
  6. `src/text_input.rs` の `mod word;` と `pub(crate) use word::{line_range, word_range};` の行を消し、`git rm crates/kivact-engine/src/text_input/word.rs` を実行する（中の単体試験も消える。同じことは手順 7 の E2E が確かめる）。
  7. `dev/tests/kivact-public-api/tests/e2e_core/text_editing.rs`：`words_screen` を、`o` の中点に加えて `foo` の `f` と `o` の間の中点も返す形にし（`stops` を 14 まで集め、`(stops[13].x + stops[14].x) / 2.0` を二つ目の x にする）、既存の二つの試験はその一つ目の点を使う。次の試験を足す。
     ```rust
     /// A drag after a double click extends the selection word by word: from
     /// the clicked word to the end of the word under the pointer.
     #[test]
     fn a_drag_after_a_double_click_extends_by_words() {
         let (screen, (x, y), (far, _)) = words_screen();
         screen.click_at(x, y);
         screen.mouse_move(x, y);
         screen.mouse_down(MouseButton::Primary);
         screen.mouse_move(far, y);
         screen.mouse_up(MouseButton::Primary);
         assert_eq!(
             screen.ime_text().expect("focused").selection,
             6..15,
             "\"world foo\", whole words"
         );
     }
     ```
     （`words_screen` の返り値を `(Screen, (f64, f64), (f64, f64))` にし、二つ目は `foo` の中の点。既存の二つの試験の `let (screen, (x, y)) = words_screen();` は `let (screen, (x, y), _) = words_screen();` にする。）
  8. `rg -n "word_range|line_range|text_input::word|select_text" crates/kivact-engine/src` が空であることを確かめる。
- **消すもの**：`src/text_input/word.rs`（`word_range`、`line_range` とその単体試験）、`Model::select_text`、`pointer_move` の touch・pen の導出の枝。
- **状態の閉じ方**：`SelectUnit`・`PenGesture` は enum。gesture の enum に variant が増えるだけ。
- **試験**：手順 7 の `text_editing::a_drag_after_a_double_click_extends_by_words` を足す。`text_input/word.rs` の単体試験 `a_double_click_selects_the_ascii_word_under_the_pointer`、`a_word_of_japanese_text_is_its_run_of_multi_byte_characters`、`a_triple_click_selects_the_line_under_the_pointer`、`a_single_line_value_is_one_line_even_with_a_newline_in_it` ほかファイルの全試験はファイルとともに消える（単語の意味が UAX #29 に変わるため。二度・三度クリックの契約は E2E の `a_double_click_selects_the_word_under_the_pointer`・`a_triple_click_selects_the_whole_line` が確かめる）。
- **確認（tier a）**：`EDITING-A`、`ENGINE-A`、`E2E: text_editing, input_platform, custom_text_field, pointer, touch_scroll`
- **完了条件**：手順 8 の `rg` が空。新しい試験を含め確認がすべて通る。

## E21：shell の入力の写し（`PointerSource`）を消す

- **wave / lane / 前提**：W6b / E1 / E19（E20 は W5a の鎖で済んでいる）
- **所有**：`src/model/pointer/state.rs`、`src/model/pointer/machine.rs`、`src/model/pointer/shell.rs`、`src/model/pointer/touch.rs`、`src/model/pointer/hover.rs`、`src/model/pointer.rs`
- **目的**：track は engine が決めたこと（over、hover、gesture）と pointer の id だけを持ち、shell の sample（button、圧力、傾き、時刻）は写さずに各関数へ `&PointerSample` で渡して転送する（EN-26 を設計書 7.3 のとおり変更して採用：`Admission` は engine に残す）。
- **手順**：
  1. `src/model/pointer/state.rs`：
     - `struct PointerSource` と `impl PointerSource`（`MOUSE`）を消す。
     - `MouseTrack` の `source: PointerSource` を消す。`TouchTrack` と `PenTrack` の `source: PointerSource` を `id: PointerId` にする。
     - `Track::mouse()` から source を消す。`Track::contact(source, primary)` を `Track::contact(id: PointerId, kind: PointerKind, primary: bool)` にし、`kind` で Touch か Pen かを決める（今の `source.kind` の判定と同じ）。
     - `Track::source` と `Track::source_mut` を消し、次を足す。
       ```rust
       /// The pointer's id (the mouse is `PointerId::MOUSE`).
       pub(crate) fn id(&self) -> PointerId {
           match self {
               Track::Mouse(_) => PointerId::MOUSE,
               Track::Touch(t) => t.id,
               Track::Pen(t) => t.id,
           }
       }

       /// Mouse, touch or pen.
       pub(crate) fn kind(&self) -> PointerKind {
           match self {
               Track::Mouse(_) => PointerKind::Mouse,
               Track::Touch(_) => PointerKind::Touch,
               Track::Pen(_) => PointerKind::Pen,
           }
       }
       ```
  2. `src/model/pointer/machine.rs`：
     - `pointer_sample(&self, pointer, node, phase, modifiers, at)` を `pointer_sample(&self, input: &PointerSample, node: NodeId, phase: PointerPhase, at: Point)` にし、`input` の値を写して `phase`・`position: at`・`buttons`（`Cancel` は `PointerButtons::NONE`）・`button`・`press`・`is_primary`（track の）を置き換える。
       ```rust
       input: PointerSample {
           phase,
           position: at,
           buttons: if phase == PointerPhase::Cancel {
               PointerButtons::NONE
           } else {
               input.buttons
           },
           button: phase.press().map(|press| press.button),
           press: phase.press(),
           is_primary,
           ..*input
       },
       ```
     - `send_pointer(&mut self, pointer, node, phase, modifiers, at)` を `send_pointer(&mut self, input: &PointerSample, node: NodeId, phase: PointerPhase, at: Point)` にする。
     - `pointer_down`・`pointer_up`・`pointer_move` の `pointer: PointerId` と `modifiers: Modifiers`（と `press`）の引数を `input: &PointerSample` に替え、本体では `let pointer = input.id; let modifiers = input.modifiers;`（`press` は `input.phase` の `Down(press)`/`Up(press)` から取る）。`pointer_up` の `let released = self.pointer.state.track(pointer).source().buttons.is_empty();` を `let released = input.buttons.is_empty();` に、`pointer_move` の `let held = self.pointer.state.track(pointer).source().buttons;` を `let held = input.buttons;` にする。
     - これらから呼ぶ内部の関数（`primary_press`、`track_hover`、`set_chain` の後の Enter/Leave の送出など）で `send_pointer` を呼ぶものは、`modifiers` の引数を `input: &PointerSample` に替える。
     - `begin_pointer(&mut self, source: PointerSource, primary: bool, at: Point)` を `begin_pointer(&mut self, input: &PointerSample, at: Point)` にし、`Track::contact(input.id, input.kind, input.is_primary)` を作る。
     - `cancel_pointer_at` の `self.pointer.state.track_mut(pointer).source_mut().buttons = PointerButtons::NONE;` を消し、引数に `input: &PointerSample` を足して `send_pointer(input, t, PointerPhase::Cancel, at)` にする。
     - 次を足す（sample の無い取消し：window が focus を失った時）。
       ```rust
       /// The sample a cancel without a shell sample carries: no button, no
       /// pressure or tilt, at the host time of the event being handled.
       pub(super) fn cancel_sample(&self, pointer: PointerId, at: Point) -> PointerSample {
           let track = self.pointer.state.track(pointer);
           PointerSample {
               id: pointer,
               kind: track.kind(),
               phase: PointerPhase::Cancel,
               position: at,
               buttons: PointerButtons::NONE,
               button: None,
               press: None,
               pressure: None,
               tilt: None,
               time_ns: self.frame.host_now(),
               modifiers: Modifiers::default(),
               is_primary: track.is_primary(),
           }
       }
       ```
  3. `src/model/pointer/shell.rs`：
     - `pointer_input` の `let track = self.pointer.state.track_mut(PointerId::MOUSE); track.source_mut().time_ns = ...; ... track.source_mut().buttons = input.buttons;` の 5 行を消し、以降の `mouse_moved`・`mouse_left`・`mouse_button` と `Cancel` の腕に `&input` を渡す（各関数の引数の `modifiers: Modifiers` を `input: &PointerSample` にする）。
     - `mouse_left` の `self.pointer.state.track(PointerId::MOUSE).source().buttons.is_empty()` を `input.buttons.is_empty()` にする。
     - `contact` の `self.begin_pointer(PointerSource { ... }, input.is_primary, at);` を `self.begin_pointer(&input, at);` にする。`contact_at(&mut self, id, at, input: PointerSample)` の 4 行の `source_mut()` の書込みを消す。
     - `cancel_pointer` は track ごとに `let sample = self.cancel_sample(id, at);` を作って `cancel_pointer_at(id, at, &sample)` に渡す。
  4. `src/model/pointer/touch.rs`：`track.source.time_ns` と `self.pointer.state.track(pointer).source().time_ns` を、その関数に渡す `input.time_ns`（関数の引数に `input: &PointerSample` を足す）にする。`release_touch_scroll(&mut self, pointer: PointerId)` を `release_touch_scroll(&mut self, pointer: PointerId, time_ns: u64)` にし、`pan.samples.pixels_per_ms(track.source.time_ns)` を `pan.samples.pixels_per_ms(time_ns)` にする（呼出し元は up の sample の `time_ns` を渡す）。`track.source().kind == PointerKind::Touch` は `track.kind() == PointerKind::Touch` に、`.source().id` は `.id()` にする。
  5. 残りの `source()` の読み（`machine.rs` の `.map(|t| t.source().id)`、`shell.rs` の `(t.source().id, t.at())`）は `.id()` にする。
  6. `rg -n "PointerSource|source\(\)|source_mut\(\)|\.source\b" crates/kivact-engine/src/model/pointer` が空であることを確かめる。
- **消すもの**：`PointerSource`、`Track::{source, source_mut}`、track の `source` の field。
- **状態の閉じ方**：track の型から Option の field（`pressure`、`tilt`）が消える。新しい状態は無い。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: pointer, hover, touch_scroll, interaction_drag_edit, interaction_slider, interaction_splitter, interaction_zoom_pan, drawables, windows, interaction_edit, text_editing`
- **完了条件**：手順 6 の `rg` が空。確認がすべて通る（listener が受ける sample の `buttons`・`pressure`・`tilt`・`time_ns` は前と同じ値）。

## E22：書込みの統計を数だけにする、a11y の数をいつも数える、`Mutation` を bit に

- **wave / lane / 前提**：W7a / E1 / なし（E21 は W6b で済んでいる）
- **所有**：`src/mutation.rs`、`src/host/write.rs`、`src/host/write/writer.rs`、`src/host/write/reader.rs`、`src/host/write/typed.rs`、`src/a11y/state.rs`、`src/a11y/publish.rs`、`src/a11y/build/dirty.rs`、`src/a11y/build/payload.rs`、`src/model/apply.rs`、`src/model/states.rs`、`src/model/edit/inputs.rs`、`src/model/presence/transitions.rs`、`src/model/changes.rs`、`src/model.rs`、`src/lib.rs`
- **目的**：書込みごとの HashSet（`updated`、`touched`）をやめて数だけを持ち、`touched_last` は commit の時に各 window の `Changes::touched()` を数える。a11y の三つの数を `testing` の外でも数え、`stats()` の実装を一つにする（EN-18）。`Mutation` の 10 個の bool を `u16` の bit にする（EN-29）。
- **手順**：
  1. `src/mutation.rs`：
     - `Writes` を次にする（`created` は「この revision で作った」の事実で、制御された値の初期化が読む：`write_input_value`）。
       ```rust
       #[derive(Default)]
       pub(crate) struct Writes {
           pub(crate) stats: WriteStats,
           /// The nodes created since the last commit (`Model::controlled_value` reads it).
           created: HashSet<NodeId>,
           /// The writes since the last commit.
           pending: u32,
       }

       impl Writes {
           pub(crate) fn created(&self, id: NodeId) -> bool {
               self.created.contains(&id)
           }
           pub(crate) fn create(&mut self, id: NodeId) {
               self.stats.creates += 1;
               self.created.insert(id);
               self.pending += 1;
           }
           pub(crate) fn field(&mut self) {
               self.stats.set_props += 1;
               self.pending += 1;
           }
           pub(crate) fn scene(&mut self) {
               self.stats.set_props += 1;
               self.pending += 1;
           }
           pub(crate) fn structural(&mut self) {
               self.stats.structural += 1;
               self.pending += 1;
           }
           pub(crate) fn command(&mut self) {
               self.pending += 1;
           }
           pub(crate) fn pending(&self) -> usize {
               self.pending as usize
           }
           /// Close the revision `revision`: `touched` nodes were written in it.
           pub(crate) fn commit(&mut self, revision: Revision, touched: usize) {
               let ops = self.pending();
               self.stats.batches += 1;
               self.stats.ops += ops as u64;
               self.stats.revision = revision.0;
               self.stats.last_ops = ops;
               self.stats.touched_last = touched;
               self.created.clear();
               self.pending = 0;
           }
       }
       ```
       `Writes` の直前に `// xtask-allow(state): pending counts the writes of the open revision, created lists its new nodes; commit clears both` を置く。
     - `Mutation` を次にする（10 個の bool と印、`NONE` などの struct literal の const を置き換える。`and` と `when` の意味は今のまま）。
       ```rust
       /// What a typed write outdates, one bit each.
       #[derive(Clone, Copy, PartialEq, Eq)]
       pub(crate) struct Mutation(u16);

       impl Mutation {
           /// Only the node itself is redrawn and exposed again.
           pub(crate) const NONE: Self = Self(0);
           /// The layout style is built again (and a layout pass runs).
           pub(crate) const STYLE: Self = Self(1);
           /// The node is measured again.
           pub(crate) const MEASURE: Self = Self(1 << 1);
           /// An inherited text style: the texts, text inputs and images below are measured or drawn again.
           pub(crate) const TEXT: Self = Self(1 << 2);
           /// Its transform or z-index may have moved it without a layout.
           pub(crate) const PLACED: Self = Self(1 << 3);
           /// The paint list is built again.
           pub(crate) const RESTACK: Self = Self(1 << 4);
           /// Its base paint: a state transition is retargeted and its place read.
           pub(crate) const PAINT: Self = Self(1 << 5);
           /// Its state styles: like `PAINT`.
           pub(crate) const STATES: Self = Self(1 << 6);
           /// A composition is committed under the old read-only policy first.
           pub(crate) const READ_ONLY: Self = Self(1 << 7);
           /// Inherited semantic state changes throughout the subtree.
           pub(crate) const SUBTREE: Self = Self(1 << 8);
           /// Only the node's own accessibility payload can change.
           pub(crate) const PAYLOAD_ONLY: Self = Self(1 << 9);

           /// Whether any bit of `bits` is set.
           pub(crate) const fn has(self, bits: Self) -> bool {
               self.0 & bits.0 != 0
           }
           /// These bits and `other`'s.
           pub(crate) const fn and(self, other: Self) -> Self {
               Self(self.0 | other.0)
           }
           /// These bits when `on`, else none.
           pub(crate) const fn when(self, on: bool) -> Self {
               if on { self } else { Self::NONE }
           }
           /// Add `bits`.
           pub(crate) fn insert(&mut self, bits: Self) {
               self.0 |= bits.0;
           }
       }
       ```
       （今の `when` の本体が「`on` なら self、でなければ `NONE`」でなければ、今の意味のとおりに書く。）
  2. `Mutation` の field の読みを直す：
     ```
     ROOT=$(git rev-parse --show-toplevel) && cd "$ROOT/crates/kivact-engine/src" || exit 1
     rg -l 'mutation\.(style|measure|text|placed|restack|paint|states|read_only|subtree|payload_only)\b' . | xargs perl -pi -e 's/\bmutation\.(style|measure|text|placed|restack|paint|states|read_only|subtree|payload_only)\b(?!\s*\|=)/"mutation.has(Mutation::".uc($1).")"/ge'
     cd "$ROOT" || exit 1
     ```
     手で直す所：`src/model/apply.rs` の `mutation.placed |= <式>;` は `if <式> { mutation.insert(Mutation::PLACED); }`、`mutation.restack |= <式>;` は `if <式> { mutation.insert(Mutation::RESTACK); }` にする。`let Mutation { style, measure, ... } = mutation;`（`changed` の中）は、分解していた名前ごとに `let style = mutation.has(Mutation::STYLE);` の行に置き換える。
  3. `src/a11y/state.rs`：`constructed`・`published`・`enumerated` の field と `Default` の初期化の `#[cfg(feature = "testing")]` を消す。`src/a11y/publish.rs`・`src/a11y/build/dirty.rs`（2 か所）・`src/a11y/build/payload.rs` の `#[cfg(feature = "testing")] { self.a11y.X += ...; }` を、cfg と block を外した一行にする。
  4. `src/host/write.rs` に次を足す。
     ```rust
     /// The client's write totals and the last committed revision, with the
     /// accessibility counts of every window.
     pub(super) fn stats_of(app: &AppModel) -> WriteStats {
         let models = || app.windows.values().map(|w| &w.model);
         WriteStats {
             a11y_constructed: models().map(|m| m.a11y.constructed).sum(),
             a11y_published: models().map(|m| m.a11y.published).sum(),
             a11y_enumerated: models().map(|m| m.a11y.enumerated).sum(),
             ..app.writes.stats
         }
     }
     ```
     `Reader::stats` と `Writer::stats` の本体を `super::stats_of(&self.app.borrow())` にする。
  5. `src/host/write/writer.rs`：
     - `writes.field(id)`・`writes.scene(id)`・`writes.structural(id)`・`writes.command(None)`・`writes.command(Some(..))`・`app.writes.command(None)` の呼出しを引数なしに直す：`perl -pi -e 's/\bwrites\.(field|scene|structural)\([a-z_]+\)/writes.$1()/g; s/\bwrites\.command\((None|Some\([^)]*\))\)/writes.command()/g' crates/kivact-engine/src/host/write/writer.rs crates/kivact-engine/src/host/write/typed.rs`。`rg -n "writes\.(field|scene|structural|command)\([^)]" crates/kivact-engine/src` が空であることを確かめる。
     - `commit` の `app.writes_mut().commit(revision);` を次にする（`prepare_commit` の loop の後、`model.sync()` の前）。
       ```rust
       let touched = app
           .windows
           .values()
           .map(|w| w.model.changes.touched().count())
           .sum();
       app.writes_mut().commit(revision, touched);
       ```
  6. `rg -n "cfg\(feature = \"testing\"\)" crates/kivact-engine/src/a11y crates/kivact-engine/src/host/write/reader.rs crates/kivact-engine/src/host/write/writer.rs` が、a11y の数と `stats` に関わる行を返さないことを確かめる。`rg -n "updated|sparse_touched_revisions|touched\.insert" crates/kivact-engine/src/mutation.rs` が空であることを確かめる。
- **消すもの**：`Writes::{updated, touched, sparse_touched_revisions, commands}`、`Writer::stats` と `Reader::stats` の重複した本体、a11y の数の `#[cfg(feature = "testing")]`、`Mutation` の 10 個の bool の field と印。
- **状態の閉じ方**：`Writes` は手順 1 の印。`Mutation` は bit の値で対象外。`A11yState` の数は W1 の印のまま（cfg が外れるだけ）。
- **試験**：足さない・直さない（`a11y_incremental` は `testing` の build で同じ数を読む）。
- **確認（tier a）**：`ENGINE-A`、`E2E: a11y_incremental, diagnostics, text_editing, reactive_props, binding_update, flows`、`dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native -- devtools::`
- **完了条件**：手順 6 の `rg` のとおり。確認がすべて通る。

## E23：試験だけの code を release から外す（devtools 以外）

- **wave / lane / 前提**：W7a / E1 / E22
- **所有**：`src/model/window.rs`、`src/host/foreign.rs`、`crates/kivact-layout/Cargo.toml`、`crates/kivact-layout/src/lib.rs`、`crates/kivact-layout/src/layout.rs`、`crates/kivact-layout/src/compare.rs`、`crates/kivact-layout/src/columns.rs`、`crates/kivact-layout/src/pass_state.rs`、`crates/kivact-engine/Cargo.toml`、`src/text_input/refresh.rs`
- **目的**：使われない `Content::Foreign::child` を消す。kivact-layout の比較の API を feature `reference`（engine の `testing` が有効にする）の中に入れる。`RefreshMode::Verify` を debug build だけにする（EN-28）。
- **手順**：
  1. `src/model/window.rs`：`Content::Foreign { child, reported }` から `child` の field と、その `#[allow(dead_code, ...)]` を消す。`src/host/foreign.rs` の `host_window.content = Content::Foreign { child, reported };` を `Content::Foreign { reported }` にし、使わなくなった `child` の束縛を消す（client へ送る `ForeignChange::Ready { child, size }` はそのまま）。
  2. `crates/kivact-layout/Cargo.toml` に次を足す。
     ```toml
     [features]
     # The comparison of retained layout with a fresh one (the engine's test driver).
     reference = []
     ```
     `crates/kivact-engine/Cargo.toml` の `testing = [...]` の並びに `"kivact-layout/reference"` を足す。
  3. `crates/kivact-layout/src/compare.rs` の `pub(crate) fn remeasures_same` を `crates/kivact-layout/src/columns.rs` の `impl NodeLayoutColumns` に移す（通常の経路で使う）。compare.rs が使う `use` は compare.rs に残す。
  4. `crates/kivact-layout/src/lib.rs`：`mod compare;` を `#[cfg(any(test, feature = "reference"))] mod compare;` に、`pub use compare::{Geometry, LayoutDifference};` を `#[cfg(any(test, feature = "reference"))] pub use compare::{Geometry, LayoutDifference};` にする。
  5. `crates/kivact-layout/src/layout.rs`：`rebuild_all`、`compare_measurements`、`compare` の三つの `pub fn` に `#[cfg(any(test, feature = "reference"))]` を付け、`use crate::LayoutDifference` の import も同じ cfg にする。`#[derive(Clone, Default)] pub struct Layout` を `#[derive(Default)] #[cfg_attr(any(test, feature = "reference"), derive(Clone))] pub struct Layout` にする。
  6. `Layout::clone` のためだけの `Clone` の derive を同じ `cfg_attr` にする：`NodeLayoutColumns`、`NodeLayoutValues`、`NodeLayoutState`、`ChildCursor`、`Record`（`columns.rs`）、`LayoutState`、`Pending`（`pass_state.rs`）。`cargo check -p kivact-layout`（feature なし）で `Clone` が要ると言われた型は derive を戻す。
  7. `src/text_input/refresh.rs`：`RefreshMode::Verify` の `#[cfg_attr(not(any(debug_assertions, feature = "testing")), expect(dead_code, ...))]` を `#[cfg(debug_assertions)]` にする（`Verify` を使う所はすべて debug の cfg の中にある）。
- **消すもの**：`Content::Foreign::child`、release の build の `compare`・`compare_measurements`・`rebuild_all`・`Geometry`・`LayoutDifference` と比較のための `Clone`、release の `RefreshMode::Verify`。
- **状態の閉じ方**：新しい状態は無い。
- **試験**：足さない。kivact-layout の `layout/tests.rs` は `cfg(test)` で比較の API を使える。
- **確認（tier a）**：
  ```
  LAYOUT-A
  cargo check -p kivact-layout --features reference --all-targets
  cargo build -p kivact-engine --release
  ENGINE-A
  E2E: foreign_window, layout, layout_check, text_editing
  ```
- **完了条件**：`cargo build -p kivact-engine --release` が通り、`rg -n "dead_code" crates/kivact-engine/src/model/window.rs crates/kivact-engine/src/text_input/refresh.rs` が空。確認がすべて通る。

## E24：hit test を paint へ移し、子の索引と重ね順を使う

- **wave / lane / 前提**：W6b / E2 / E32（E14 → E32 → E24）
- **所有**：`src/model/interact/hit.rs`（`src/paint/list/hit.rs` へ移す）、`src/model/interact.rs`、`src/paint/list/hit_reach.rs`（削除）、`src/paint/list/facts.rs`（新設）、`src/paint/list.rs`、`src/paint.rs`、`src/paint/state.rs`、`src/paint/list/walk.rs`、`src/layer.rs`、`src/model/interact/geometry.rs`、`src/model/drawables/hit.rs`、`src/model/drawables.rs`、`src/lower/shape.rs`（doc だけ）、`src/host/testing/test_host.rs`（method を一つ足すだけ）、`crates/kivact-engine/Cargo.toml`（`[[test]]` を一つ）、`crates/kivact-engine/tests/hit_reference.rs`（新設）
- **目的**：hit test と paint list は同じ単位なので、実装一つの trait `HitInk` と `LayerView` を消し、hit test を `paint::list` に置く（EN-29）。hit test は、list が持つ子の索引（32 個以上の子）で点の近くの子だけを調べ、重ね順は walk が作った順を使う（EN-09）。debug の self_check の参照の walk（`HitWalk::Complete`）は索引も walk の順も使わず、すべての子を model の重ね順で調べる：最適化の誤りを参照が同じ誤りで隠さない（EA-13、設計書 12 章）。
- **手順**：
  1. `git mv crates/kivact-engine/src/model/interact/hit.rs crates/kivact-engine/src/paint/list/hit.rs` を実行する。`src/model/interact.rs` の `pub(crate) mod hit;` を消し、`src/paint/list.rs` の module の並びに `mod hit;` と `mod facts;` を足し、`mod hit_reach;` を消して `git rm crates/kivact-engine/src/paint/list/hit_reach.rs` を実行する。
  2. `HitWalk` の定義を `src/paint/list/hit.rs` から `src/model/drawables/hit.rs` に移し（`pub(crate) enum HitWalk`、derive と variant の cfg はそのまま）、`src/paint/list/hit.rs` は `use crate::model::drawables::hit::HitWalk;` で使う。`src/model/drawables/hit.rs` の `use crate::model::interact::hit::HitWalk;` を消す。`src/model/drawables.rs` で `hit` が `pub(crate) mod hit;` でなければそうする。他の `crate::model::interact::hit::HitWalk` は `crate::model::drawables::hit::HitWalk` にする（`rg -n "interact::hit" crates/kivact-engine/src`）。`src/lower/shape.rs` の doc の `model::interact::hit` を `paint::list::hit` にする。
  3. `src/paint/list/hit.rs`：`trait HitInk`、`enum HitReach` を消す。`may_hit` の `let ink = match self.hit_ink(c, cn, &layout) { ... };` を次にする。
     ```rust
     let ink = match super::ink::drawn(self, c, cn, &layout, self.paint.list.ink_gen) {
         None => return false, // not drawn, so not shown
         Some(d) if d.beyond => return true,
         Some(d) => d.rect,
     };
     ```
  4. `src/paint/state.rs` の `PaintList` に `pub(super) stack_orders: HashMap<NodeId, Rc<[NodeId]>>,` を足す（doc：`/// The children of each listed container that sorts them, in stacked order (paint and hit test share it).`）。`src/paint/list/walk.rs`：`struct Walk` に `stack_orders: HashMap<NodeId, Rc<[NodeId]>>` を足し（作る所で空）、`Walk::children` の最初に次を置き、索引を使わない枝の `for child in host.stacked(n)` を、`order` があれば `order.iter().copied()`、無ければ `host.tree().flat_children(id)` を回す形にする。
     ```rust
     let order: Option<Rc<[NodeId]>> = host.sorts_children(n).then(|| host.stacked(n).collect());
     if let Some(order) = &order {
         self.stack_orders.insert(id, Rc::clone(order));
     }
     ```
     `build_list` で `list.stack_orders = stack_orders;` にする。E13 の `rewalk_scrolled` では、`Walk { .. } = walk` の分解に `stack_orders` を足し、`list.looks.extend(looks);` の次に `list.stack_orders.retain(|node, _| !left.contains(node));` と `list.stack_orders.extend(stack_orders);` を置く（list から外れた container の順は残さない）。
  5. `src/paint/list/hit.rs`：`hit_box` の `for c in self.stacked(n).rev() {` を次にする（loop の中身はそのまま。参照の walk は索引と walk の順を使わない）。
     ```rust
     let candidates: Vec<NodeId> = match walk {
         HitWalk::Pruned => self.hit_candidates(id, n, p, scroll),
         // The reference looks at every child in the model's stacked order:
         // neither the list's child index nor its stacked orders.
         #[cfg(all(debug_assertions, feature = "testing"))]
         HitWalk::Complete => self.stacked(n).rev().collect(),
     };
     for c in candidates {
     ```
     次の二つの method を足す（`listed_candidates` が候補を作り、`hit_candidates` は試験の driver が入れる誤りを足す）。
     ```rust
     /// The children of `n` (the box `id`) the point `p` may hit, topmost
     /// first (`listed_candidates`); the test driver may leave one out
     /// (`ArvaneTestHost::drop_hit_candidate`), a fault the reference walk
     /// must catch.
     fn hit_candidates(&self, id: NodeId, n: &HostNode, p: Point, scroll: Point) -> Vec<NodeId> {
         let found = self.listed_candidates(id, n, p, scroll);
         #[cfg(feature = "testing")]
         let found = {
             let mut found = found;
             if let Some(dropped) = crate::paint::state::DROP_HIT_CANDIDATE.with(std::cell::Cell::get) {
                 found.retain(|c| *c != dropped);
             }
             found
         };
         found
     }
     ```
     ```rust
     /// The children of `n` (the box `id`) the point `p` (in its box) may
     /// hit, topmost first: the listed stacked order of a container that sorts
     /// its children; the child index of a container with many, when the list
     /// keeps a fresh one; else every child in stacked order.
     fn listed_candidates(&self, id: NodeId, n: &HostNode, p: Point, scroll: Point) -> Vec<NodeId> {
         let list = &self.paint.list;
         if list.state != ListState::Stale {
             if let Some(order) = list.stack_orders.get(&id) {
                 return order.iter().rev().copied().collect();
             }
             if let Some(index) = list.indices.get(&id).filter(|index| index.touched.is_empty()) {
                 // The index's reaches are in the container's unscrolled content space.
                 let probe = Rect {
                     x: p.x + scroll.x - HIT_SLACK,
                     y: p.y + scroll.y - HIT_SLACK,
                     width: 2.0 * HIT_SLACK,
                     height: 2.0 * HIT_SLACK,
                 };
                 let mut found: Vec<(usize, NodeId)> = index
                     .reaching(probe)
                     .chain(index.sticky())
                     .map(|e| (e.order, e.id))
                     .collect();
                 found.sort_unstable_by(|a, b| b.0.cmp(&a.0));
                 found.dedup_by_key(|e| e.0);
                 return found.into_iter().map(|(_, c)| c).collect();
             }
         }
         self.stacked(n).rev().collect()
     }
     ```
     （`ListState`・`Rect`・`Point` の `use` を足す。`index.reaching` と `index.sticky` は `paint::list::index` の `pub(super)` で、hit.rs は `paint::list` の子なので読める。）
  6. `LayerView` を消す：
     - `src/layer.rs` から `trait LayerView`、`fn layer_facts`、`fn subtree_facts`、`fn node_facts`、`fn own_layer`、`fn promoted` を `src/paint/list/facts.rs` に移す（`//! Role: Derive the layer facts of the subtrees from the looks the walk uses.`）。移した関数の `view: &impl LayerView` の引数を `host: &Model` にし、`view.drawn_look(..)`・`view.is_layer_node(..)`・`view.scene_facts(..)`・`view.moves_layer(..)`・`view.layer_roots()` を `host.` の同名の method に、`view.flat_children(id)` を `host.tree().flat_children(id).collect::<Vec<_>>()` にする。layer.rs の doc の「This module does not name the model: the tree is read through [`LayerView`] ...」の項目を消す。
     - `src/model/interact/geometry.rs` の `impl layer::LayerView for Model { ... }` を `impl Model { ... }` にし、`flat_children` の method を消し、残る五つ（`drawn_look`、`is_layer_node`、`scene_facts`、`moves_layer`、`layer_roots`）を `pub(crate) fn` にする。同じファイルの `Model::layer_facts` と `Model::subtree_facts` を `src/paint/list/facts.rs` の `impl Model` に移し、中で同じファイルの自由関数を呼ぶ。
     - `src/paint/list/walk.rs` の `use crate::layer::LayerView;` を消し、`crate::layer::node_facts(self, ...)` を `super::facts::node_facts(self, ...)` にする。`src/model.rs` の `pub(crate) use crate::layer::{EVERYWHERE, Look, SubtreeFacts};` はそのまま。
  7. `rg -n "HitInk|HitReach|LayerView|hit_reach|interact::hit" crates/kivact-engine/src` が空であることを確かめる。
  8. 参照の walk が候補の誤りを見つけることの試験（EA-13）：
     - `src/paint/state.rs` に次を足す（E32 の `KEEP_STALE_DRAW` の隣）。
       ```rust
       #[cfg(feature = "testing")]
       thread_local! {
           /// A child the hit test's candidates leave out: the test driver's
           /// injected fault (`ArvaneTestHost::drop_hit_candidate`), which the
           /// debug self-check's complete walk must catch.
           pub(crate) static DROP_HIT_CANDIDATE: std::cell::Cell<Option<NodeId>> = const { std::cell::Cell::new(None) };
       }
       ```
     - `src/host/testing/test_host.rs` の `impl ArvaneTestHost` に次を足す（`#[doc(hidden)]` は同じファイルの他の試験用 method に合わせる）。
       ```rust
       /// Leave `node` out of the hit test's candidates (`None`: leave none
       /// out): an injected fault the self-check's reference walk must catch.
       pub fn drop_hit_candidate(&self, node: Option<NodeId>) {
           crate::paint::state::DROP_HIT_CANDIDATE.with(|dropped| dropped.set(node));
       }
       ```
     - `crates/kivact-engine/Cargo.toml` に `tests/host_locale.rs` と同じ形の `[[test]]` を足す：`name = "hit_reference"`、`required-features = ["testing"]`。`crates/kivact-engine/tests/hit_reference.rs` を作る。
       ```rust
       //! Role: Tests that the hit self-check's reference walk does not share the hit test's candidates.

       use kivact_engine::api::Client;
       use kivact_engine::contract::{BoxData, NodeKind, TurnOutput};
       use kivact_engine::testing::{ArvaneTestHost, MirrorMode, TestHostOptions};
       use kivact_types::{Dimension, LayoutProps, NodeId, WindowId};

       const ROOT: NodeId = NodeId(1);
       const CHILD: NodeId = NodeId(2);

       /// A root and one child box covering the whole window.
       fn start() -> ArvaneTestHost {
           ArvaneTestHost::new(
               Client::new(
                   |_, _, writer| {
                       writer.create(WindowId::MAIN, ROOT, NodeKind::Root);
                       writer.create(WindowId::MAIN, CHILD, NodeKind::Box(BoxData::default()));
                       writer.insert(ROOT, CHILD, None);
                       writer.set_layout(
                           CHILD,
                           LayoutProps {
                               width: Dimension::Percent(100.0),
                               height: Dimension::Percent(100.0),
                               ..LayoutProps::default()
                           },
                       );
                       writer.commit();
                   },
                   |_, _, _| TurnOutput::default(),
               ),
               TestHostOptions {
                   mirror: MirrorMode::Off,
                   ..TestHostOptions::default()
               },
           )
           .expect("the test host starts")
       }

       /// The self-check's hit errors.
       fn hit_errors(host: &mut ArvaneTestHost) -> Vec<String> {
           host.self_check()
               .into_iter()
               .filter(|error| error.starts_with("hit at"))
               .collect()
       }

       /// The hit test's candidates leave out the child under the window's
       /// center (an injected fault): the complete reference walk still finds
       /// it, and the self-check reports the difference.
       #[cfg(debug_assertions)]
       #[test]
       fn the_reference_hit_walk_finds_a_child_the_candidates_left_out() {
           let mut host = start();
           host.redraw();
           assert!(hit_errors(&mut host).is_empty(), "without a fault the walks agree");
           host.drop_hit_candidate(Some(CHILD));
           let found = hit_errors(&mut host);
           host.drop_hit_candidate(None);
           assert!(!found.is_empty(), "the reference walk must not share the candidates");
       }
       ```
       （`self_check` の参照の walk は debug build と `testing` にだけあるので、試験も `debug_assertions` の時だけ compile する。）
- **消すもの**：`HitInk`、`HitReach`、`src/paint/list/hit_reach.rs`、`LayerView`、`src/model/interact/hit.rs`（移動）。
- **状態の閉じ方**：新しい状態は無い（`PaintList` の値の field が一つ増える）。
- **試験**：`crates/kivact-engine/tests/hit_reference.rs` の `the_reference_hit_walk_finds_a_child_the_candidates_left_out` を足す（手順 8。EA-13）。既存の試験は直さない。
- **確認（tier a）**：`ENGINE-A`（`cargo xtask check` の module の向きの検査と `hit_reference` の試験を含む）、`dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-engine --features testing --test hit_reference -- the_reference_hit_walk_finds_a_child_the_candidates_left_out`、`E2E: hover, pointer, transform, transform_3d, compositor, drawables, a11y_incremental, layout_scroll, overlay, canvas`、`E2E: ui_collections, ui_data_table, interaction_list_nav, interaction_table`
- **完了条件**：手順 7 の `rg` が空。新しい試験を含め確認がすべて通る（どのコマンドも 0 件でない。1.4 節）。

## E25：何も無いときの sync を省く、消えた node の記録をその場で消す

- **wave / lane / 前提**：W7a / E1 / E23
- **所有**：`src/model/interact/sync.rs`、`src/model/changes.rs`、`src/tree/node_tree.rs`、`src/model/keys.rs`、`src/model/edit/ime.rs`、`src/model/apply.rs`、`src/paint/state.rs`、`src/lower/assets.rs`
- **目的**：`Changes` が空で、子の順の詰め直しも無いとき `sync` は何もしない。sync ごとの report の集合の全走査（`forget_missing`、`prune_reports`）をやめ、`remove_now` が消えた id で一度だけ消す。layout の後の画像の報告は測り直した画像だけにする。文字入力の入口は sync せず、続く pump の settle に任せる（EN-27）。
- **手順**：
  1. `src/model/changes.rs` の `impl Changes` に次を足す。
     ```rust
     /// Whether nothing is pending: no node is marked, no layout, paint,
     /// restack or font work waits and no box moved.
     pub(crate) fn is_clean(&self) -> bool {
         self.dirty.is_empty()
             && self.geometry == Geometry::Kept
             && self.layout.bits() == 0
             && !self.paint
             && !self.restack
             && !self.fonts
     }
     ```
     （`Changes` の field の形が W1 の後に変わっていれば、同じ意味の判定にする。）
  2. `src/tree/node_tree.rs` に `pub(crate) fn child_orders_pending(&self) -> bool { !self.pending_child_order.is_empty() }` を足す。
  3. `src/model/interact/sync.rs` の `sync` の最初の行に次を置く。
     ```rust
     // Nothing to dispatch: a sync after another, or an input that changed nothing.
     if self.changes.is_clean() && !self.tree.child_orders_pending() {
         return;
     }
     ```
     同じ関数の `let tree = &self.tree; self.paint.forget_missing(|id| tree.contains(id)); self.assets.borrow_mut().prune_reports(|id| tree.contains(id));` の 3 行を消す。
  4. `src/model/apply.rs` の `remove_now`：`for gone in self.remove_slots(id) { ... }` を `let gone = self.remove_slots(id); for gone in gone.iter().copied() { ... }`（中身はそのまま）にし、loop の後に次を置く（App で共有する asset の記録からも、この window で消えた node の分だけを消す）。
     ```rust
     let gone: std::collections::HashSet<NodeId> = gone.into_iter().collect();
     self.paint.forget_missing(|x| !gone.contains(&x));
     self.assets.borrow_mut().prune_reports(|x| !gone.contains(&x));
     ```
  5. `src/model/interact/sync.rs` の `settle`：`#[cfg(debug_assertions)] let remeasured: Vec<NodeId> = self.changes.marked(Dirty::MEASURE).collect();` の cfg を外し（いつも作る）、layout の後の `let images: Vec<NodeId> = self.tree.images.iter().copied().collect(); self.report_images(&images);` を次にする。
     ```rust
     // Images measured in this pass may have finished decoding during it.
     let images: Vec<NodeId> = remeasured
         .iter()
         .copied()
         .filter(|id| self.tree.images.contains(id))
         .collect();
     self.report_images(&images);
     ```
     debug の lint の `self.lint_layout(engine, ..., &remeasured)` はそのまま `remeasured` を使う。
  6. `poll_assets`：`let images: Vec<NodeId> = self.tree.images.iter().copied().collect();` は残し、`self.report_images(&images);` を次にする（失敗は node ごとに調べる。それ以外は大きさが変わった画像だけ）。
     ```rust
     if failed {
         self.report_images(&images);
     } else {
         self.report_images(&resized);
     }
     ```
     `resized` は、`if changed { for id in &images { ... if self.assets.borrow_mut().natural_changed(image, used) { ... } } }` の `if` の中で `resized.push(*id);` で集める（loop の前に `let mut resized = Vec::new();`）。
  7. `src/model/keys.rs` の `key` と `paste`、`src/model/edit/ime.rs` の `preedit`、`commit`、`ime_select` の最後の `self.sync();` の行を消す（host はこの後に必ず pump し、settle が sync する）。
  8. `rg -n "forget_missing\(|prune_reports\(" crates/kivact-engine/src/model/interact/sync.rs` が空であることを確かめる。
- **消すもの**：`sync` の中の report の全走査、文字入力の入口の `sync()`。
- **状態の閉じ方**：新しい状態は無い。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: assets, image_semantics, images_animated, resource_reclaim, disposal, text_editing, custom_text_field, input_platform, keyboard, launch_real_window, a11y_incremental, diagnostics`
- **完了条件**：手順 8 の `rg` が空。確認がすべて通る。

## E26：`HostNode` の `scroll` と `shift` を疎な表へ

- **wave / lane / 前提**：W7a / E1 / E25
- **所有**：`src/tree/node.rs`、`src/tree/node_tree.rs`、`src/host/write/writer.rs`、`src/a11y/build/dirty.rs`、`src/a11y/build/node.rs`、`src/a11y/build/payload.rs`、`src/model/apply.rs`、`src/model/interact/geometry.rs`、`src/paint/list/walk.rs`、`src/model/frame/ticks.rs`、`src/model/transform.rs`、`src/model/frame/scroll_snap.rs`、`src/model/layout_motion/steps.rs`、`src/model/interact/sticky.rs`、`src/paint/list/ink.rs`、`src/model/pointer/shell.rs`、`src/paint/list/hit.rs`、`src/model/scrollbar.rs`、`src/model/presence/transitions.rs`
- **目的**：ほとんどの node が持たない scroll の offset と sticky の移動（各 16 byte）を、`NodeTree` の疎な表に移す（EN-23、MEM-1）。MEM-6 の上限を締める。
- **手順**：
  1. `src/tree/node_tree.rs`：`struct NodeTree` に次の二つを足し（`scrollers` の次）、`impl NodeTree` に次の四つを足す。`remove_with` の中で `self.scrollers.remove(&x);` などを書いている所に `self.scroll.remove(&x); self.shift.remove(&x);` を足す。
     ```rust
     /// The offset of each scroll container that is scrolled (absent: at the origin).
     scroll: kivact_types::IdMap<Point>,
     /// How far each stuck sticky box is moved from its laid out position (absent: not moved).
     shift: kivact_types::IdMap<Point>,
     ```
     ```rust
     /// The scroll offset of `id`.
     pub(crate) fn scroll_of(&self, id: NodeId) -> Point {
         self.scroll.get(&id).copied().unwrap_or_default()
     }

     /// Set the scroll offset of `id`; whether it changed.
     pub(crate) fn set_scroll_of(&mut self, id: NodeId, offset: Point) -> bool {
         if self.scroll_of(id) == offset {
             return false;
         }
         if offset == Point::default() {
             self.scroll.remove(&id);
         } else {
             self.scroll.insert(id, offset);
         }
         true
     }

     /// How far the sticky box `id` is moved by its scrollport.
     pub(crate) fn shift_of(&self, id: NodeId) -> Point {
         self.shift.get(&id).copied().unwrap_or_default()
     }

     /// Set how far the sticky box `id` is moved; whether it changed.
     pub(crate) fn set_shift_of(&mut self, id: NodeId, shift: Point) -> bool {
         if self.shift_of(id) == shift {
             return false;
         }
         if shift == Point::default() {
             self.shift.remove(&id);
         } else {
             self.shift.insert(id, shift);
         }
         true
     }
     ```
  2. `src/tree/node.rs`：`HostNode` の `scroll: Point` と `shift: Point` の field と doc、`HostNode::new` の `scroll: Point::default(),` と `shift: Point::default(),`、`pub(crate) fn scroll(&self) -> Point` を消す。assert を次にする（`Point` は f64 二つで 16 byte・整列 8 なので、二つを消すと 32 byte ちょうど減る）。
     ```rust
     const _: () = assert!(
         std::mem::size_of::<HostNode>() <= 280,
         "HostNode grew: box the new rare field"
     );
     ```
  3. 読み書きを置き換える。規則：node の id が変数にあればそれを、無ければ `self.tree.nodes.id_at(n.index)`（model の中）・`host.tree().nodes.id_at(n.index)`（paint の中）を使う。
     - 読み：`n.scroll()`・`n.scroll`・`node.scroll()`・`self.tree.get(p)?.scroll`・`self.node(x).map(|n| n.scroll())` → `self.tree.scroll_of(<id>)`（paint・a11y の中は `host.tree().scroll_of(<id>)`・`self.tree().scroll_of(<id>)`）。
     - 書き：`n.scroll = o;`（`set_scroll` の中、`src/model/interact/geometry.rs`）と、`let moved = n.scroll != offset; n.scroll = offset;`（scroll の prop の書込み、`src/model/apply.rs`）→ `self.tree.set_scroll_of(id, o)` の返り値を「変わった」に使う。
     - `n.shift`（読み）→ `self.tree.shift_of(<id>)`。`n.shift = shift;`（`src/model/interact/sticky.rs`）→ `self.tree.set_shift_of(id, shift)`。`n.shift = Point::default();`（`src/model/apply.rs` の `changed`）→ `self.tree.set_shift_of(id, Point::default());`。
     - `crate::tree::node::ScrollReport(n.scroll, ...)`（`report_geometry`）→ `ScrollReport(self.tree.scroll_of(id), ...)`（借用が重なるので、`self.tree.get_mut(id)` の前に `let scroll = self.tree.scroll_of(id);` を取る）。
     - `src/a11y/hash.rs` の `n.scroll` は `SemanticsNode` の field なので変えない。
     置き換える場所の一覧は `rg -n "\.scroll\b[^_(]|\.scroll\(\)|\.shift\b" crates/kivact-engine/src --type rust` で出し、`a11y/hash.rs` と `text_input/chunks.rs`（`line.shift(..)` は別の method）と `modifiers.shift` を除いたすべてを直す。
  4. `rg -n "\bn\.scroll\b|node\.scroll\(\)|\.scroll\(\)|\bn\.shift\b|c\.shift\b|node\.shift\b" crates/kivact-engine/src` が `a11y/hash.rs` だけを返すことを確かめる。
- **消すもの**：`HostNode::{scroll, shift}` の field、`HostNode::scroll()`。
- **状態の閉じ方**：`NodeTree` に map の field が二つ増える（数・bool・Option ではない）。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: layout_scroll, touch_scroll, scroll_timeline, a11y_incremental, overlay, hover, transform, compositor, presence, shared_layout`、`E2E: ui_collections, ui_data_table, interaction_list_nav`
- **完了条件**：手順 4 の `rg` のとおり。`HostNode` の assert が `<= 280` で build が通る。確認がすべて通る。

## E27：動いた所の下だけを幾何の報告にする

- **wave / lane / 前提**：W7b / E1 / なし（E26 は W7a で済んでいる）
- **所有**：`src/model/changes.rs`、`src/model/interact/geometry.rs`、`src/model/interact/sync.rs`、`src/model/states.rs`、`src/model/apply.rs`、`src/model/focus/navigation.rs`、`src/model/state_motion/steps.rs`、`src/model/pointer/machine.rs`、`src/model/drawables/request.rs`、`src/check/layout_reference.rs`
- **目的**：layout pass ごとに全部の observed と scroll container を報告するのをやめ、幾何が変わった node・測り直した node・scroll した container・動かした node（`Dirty::REPORT`）の下にあるものだけを報告する。tooltip の測り直しも、その tooltip か anchor がその下にあるときだけにする（EN-08）。
- **手順**：
  1. `src/model/changes.rs`：
     - `impl Dirty` に `pub(crate) const REPORT: Self = Self(1 << 9);` を足す（doc：`/// The geometry reports at or below it are due (\`Model::report_geometry\`). Kept by \`dispatch\` until taken.`）。E10 が書いた `dispatch` の `let kept_bits = Dirty::MEASURE.union(Dirty::CARET);` を `Dirty::MEASURE.union(Dirty::CARET).union(Dirty::REPORT)` にする。
     - 次を足す。
       ```rust
       /// `id` moved without a layout computation (a transform, a placement,
       /// a scroll offset): positions are redrawn and its reports are due.
       pub(crate) fn moved_node(&mut self, id: NodeId) {
           self.moved();
           self.mark(id, Dirty::REPORT);
       }
       ```
     - `scrolled` の `self.mark(id, Dirty::SCROLLED);` を `self.mark(id, Dirty::SCROLLED.union(Dirty::REPORT));` に、`stuck` の `self.mark(id, Dirty::STUCK);` を `self.mark(id, Dirty::STUCK.union(Dirty::REPORT));` にする。
  2. `self.changes.moved()` を id 付きにする：`src/model/apply.rs` の scroll の prop の書込み（`self.changes.moved(); self.changes.mark(id, Dirty::SCROLLED);`）を `self.changes.moved_node(id); self.changes.mark(id, Dirty::SCROLLED);` に、`changed` の `if placed { ... self.changes.moved(); }` を `self.changes.moved_node(id);` に、`src/model/states.rs` の `restyled` の `self.changes.moved();` を `self.changes.moved_node(id);` に、`src/model/focus/navigation.rs` の `change_focus` の `self.changes.moved();` を `self.changes.moved_node(n);` に、`src/model/pointer/machine.rs` の state の transform の所（`// A state transform or z-index moves or reorders it`）の `self.changes.moved();` を `self.changes.moved_node(id);` に、`src/model/drawables/request.rs` の `m.changes.moved();` を `m.changes.moved_node(root);` にする。`src/model/state_motion/steps.rs` の `step_states` の transform の塊（E13 が `moved()` を消した所）に `self.changes.mark(*id, crate::model::changes::Dirty::REPORT);` を足す。`src/model/apply.rs` の observed の書込み `let changed = self.tree.set_observed(id, on);` の次に `if changed && on { self.changes.mark(id, Dirty::REPORT); }` を足す。
  3. `rg -n "changes\.moved\(\)" crates/kivact-engine/src` が `src/model/changes.rs` の中だけを返すことを確かめる。
  4. `src/model/interact/sync.rs` の `set_viewport` の `if self.viewport != window {` の中に `if let Some(root) = self.tree.root() { self.changes.mark(root, Dirty::REPORT); }` を足す。
  5. `src/model/interact/geometry.rs`：
     - 次を足す。
       ```rust
       /// Whether `id` or a node above it is in `marks`.
       fn report_due(&self, id: NodeId, marks: &std::collections::HashSet<NodeId>) -> bool {
           !marks.is_empty()
               && self.tree.nodes.index_of(id).is_some_and(|start| {
                   self.tree
                       .ancestors(start)
                       .any(|(i, _)| marks.contains(&self.tree.nodes.id_at(i)))
               })
       }
       ```
     - `layout`：`self.changes.begin_layout()` で受けた測り直しの id の列に、`Dirty::REPORT` を印す（`for id in &<その列> { self.changes.mark(*id, Dirty::REPORT); }`）。`for change in &geometry { ... }` の loop の中に `self.changes.mark(change.id, Dirty::REPORT);` を足す。`let fitted = self.refit_tooltips(engine, !computed);` の前に `let due: std::collections::HashSet<NodeId> = self.changes.marked(Dirty::REPORT).collect();` を置き、`self.refit_tooltips(engine, !computed, &due)` にする。
     - `refit_tooltips(&mut self, engine, reset: bool)` に引数 `due: &std::collections::HashSet<NodeId>` を足し、loop の先頭に次を置く（`reset` の時だけ、その tooltip か anchor が動いた所の下にあるものに絞る）。
       ```rust
       let anchor = match props.at {
           kivact_types::OverlayAt::Anchor { node, .. } => node,
           _ => None,
       };
       let moved = self.report_due(id, due) || anchor.is_some_and(|a| self.report_due(a, due));
       if reset && !moved {
           continue;
       }
       ```
     - `report_geometry` の最初に `let marks: std::collections::HashSet<NodeId> = self.changes.take(Dirty::REPORT).into_iter().collect();` を置き、`let scrollers: Vec<NodeId> = self.tree.scrollers.union(&self.tree.observed).copied().collect();` に `.filter(|id| self.report_due(*id, &marks))` を、`let observed: Vec<NodeId> = self.tree.observed.iter().copied().collect();` にも同じ `.filter(..)` を足す。
  6. `src/check/layout_reference.rs`：`self.refit_tooltips(engine, false);` を `self.refit_tooltips(engine, false, &std::collections::HashSet::new());` にする。
- **消すもの**：id の無い `changes.moved()` の呼出し、`report_geometry` の全件の報告、scroll だけの pass の全 tooltip の測り直し。
- **状態の閉じ方**：`Dirty` の bit が一つ増えるだけ。
- **試験**：足さない・直さない。
- **確認（tier a）**：`ENGINE-A`、`E2E: layout, layout_scroll, layout_units, overlay, node_ref, transform, transform_3d, shared_layout, presence, touch_scroll, viewport, a11y_incremental`、`E2E: interaction_tooltip, apg_tooltip, ui_collections, ui_carousel, apg_carousel, interaction_zoom_pan`
- **完了条件**：手順 3 の `rg` のとおり。確認がすべて通る。

## E28：taffy の cache を使う slot だけに持つ

- **wave / lane / 前提**：W7b / E1 / E27
- **所有**：`crates/kivact-layout/src/columns.rs`、`crates/kivact-layout/src/tree.rs`、`crates/kivact-layout/src/compare.rs`
- **目的**：layout に加わらない slot（Contents の node、隠れた subtree、切り離された node）が taffy の `Cache`（約 520 byte）を持たないよう、最初に保存する時に確保する（EN-23）。`unrounded` と `final_layout` の縮小は 要設計判断 D-4。
- **手順**：
  1. `crates/kivact-layout/src/columns.rs`：`NodeLayoutValues` の `caches: Vec<taffy::Cache>` を `caches: Vec<Option<Box<taffy::Cache>>>` にし、doc を `/// The taffy cache of each slot, made when a pass first stores into it.` にする。`Default` の `caches: vec![taffy::Cache::new()]` を `caches: vec![None]` に、`push_slot` の `values.caches.push(taffy::Cache::new());` を `values.caches.push(None);` に、`forget` の `values.caches[index] = taffy::Cache::new();` を `values.caches[index] = None;` にする。
  2. `crates/kivact-layout/src/tree.rs`：
     - 読み `self.caches[i].get(inputs)` を `self.caches[i].as_ref().and_then(|cache| cache.get(inputs))` にする。
     - 保存 `self.caches[i].store(inputs, output)` を `self.caches[i].get_or_insert_with(|| Box::new(taffy::Cache::new())).store(inputs, output)` にする。
     - 消去 `X.clear()`（`ClearState` を返す）を `X.as_mut().map_or(ClearState::AlreadyEmpty, |cache| cache.clear())` にし、返り値を使わない所は `if let Some(cache) = X.as_mut() { cache.clear(); }` にする。`rg -n "caches\[" crates/kivact-layout/src` の全行を直す。
  3. `crates/kivact-layout/src/compare.rs` の `before.caches[at].clone().get(&input)` を `before.caches[at].as_ref().and_then(|cache| cache.get(&input))` にする。
- **消すもの**：なし。
- **状態の閉じ方**：新しい状態は無い。
- **試験**：足さない・直さない。
- **確認（tier a）**：`LAYOUT-A`、`cargo check -p kivact-layout --features reference --all-targets`、`ENGINE-A`、`E2E: layout, layout_units, layout_scroll, grid, flows, presence`
- **完了条件**：`rg -n "Vec<taffy::Cache>" crates/kivact-layout/src` が空。確認がすべて通る。

## E29：engine の feature `devtools`

- **wave / lane / 前提**：W8 / E1 / なし
- **所有**：`crates/kivact-engine/Cargo.toml`、`crates/kivact-core/Cargo.toml`（`[features]` の `devtools` の行だけ）、`src/lib.rs`、`src/check.rs`、`src/host/devtools.rs`、`src/host/pump.rs`、`src/host/events.rs`、`src/host/window_changes.rs`、`src/host/write/reader.rs`、`src/host/write/writer.rs`
- **目的**：DevTools だけが使う layout の検査（`check` の findings を `Reader` に出す口）、overlay と pick を、engine の feature `devtools`（kivact-core の `devtools` が有効にする）の中に入れる（EN-28）。検査の module は debug build と `testing` でも要るので、その三つのどれかで compile する。
- **手順**：
  1. `crates/kivact-engine/Cargo.toml` の `[features]` に `devtools = []`（上に `# The DevTools' layout findings, overlay and pick (kivact-core's devtools).`）を足す。`crates/kivact-core/Cargo.toml` の `devtools = ["dep:kivact-devtools-agent", "dep:kivact-devtools-protocol"]` に `"kivact-engine/devtools"` を足す。
  2. `src/lib.rs` の `mod check;` を `#[cfg(any(debug_assertions, feature = "testing", feature = "devtools"))] mod check;` にする。
  3. `#[cfg(feature = "devtools")]` を付けるもの：`Reader::layout_findings`、`Writer::set_devtools_overlay`、`Writer::set_pick`、`Writer::set_devtools_keys`、`src/host/devtools.rs` の `append_overlay_layer`・`overlay_scene`・`push_box`・`push_label`、`src/host/pump.rs` の `intercept_devtools`・`start_key`・`pick_node` と `present` の overlay の塊（`let overlay = self.devtools.overlays.get(&id).cloned();` からその `append_overlay_layer(...)` の呼出しの終わりまで）、`src/host/events.rs` の `if self.intercept_devtools(id, &event) { ... }` の塊。
  4. `cargo clippy -p kivact-engine --all-targets -- -D warnings`（feature なし）を実行し、使われないと言われた item（`DevtoolsHost` の field、`apply_devtools_changes` の腕、`src/host/window_changes.rs` の overlay と pick の後始末、import）に同じ `#[cfg(feature = "devtools")]` を付ける。`contract` の `DevtoolsChange`・`HostEvent::DevtoolsKey`・`HostEvent::DevtoolsPick` の variant には付けない（他の crate が match する。CONTRIBUTING の feature の規則）。
  5. `cargo check -p kivact-engine --all-targets --features devtools` と feature なしの両方が通ることを確かめる。
- **消すもの**：なし（release で `devtools` の無い build から外れるだけ）。
- **状態の閉じ方**：新しい状態は無い。
- **試験**：足さない・直さない。
- **確認（tier a）**：
  ```
  ENGINE-A
  cargo check -p kivact-engine --all-targets --features devtools
  cargo clippy -p kivact-engine --all-targets -- -D warnings
  cargo clippy -p kivact-engine --all-targets --features devtools -- -D warnings
  cargo check -p kivact-core --all-targets --features devtools
  cargo build -p kivact-engine --release
  dev/tools/check-features.sh
  dev/tools/window-lock.sh --timeout 900 -- env KIVACT_TEST_MIRROR=off cargo test -p kivact-public-api-tests --test e2e_native --features devtools -- devtools::
  E2E: layout_check
  ```
- **完了条件**：上の確認がすべて通る。

## 4. 他の付属書との接点

### 4.1 kivact-types に置くもの（付属書 S と共有する crate）

| 名前 | 定義の場所 | unit・wave | 説明 |
|---|---|---|---|
| `kivact_types::clock::Clock` | `crates/kivact-types/src/clock.rs`（新設） | E01・W3b | App の motion の時計。`new`、`phase`、`running_ns`、`motion_ns`、`rate`、`set_scale`、`stop`、`resume`、`motion_clock`（`MotionClock` の snapshot） |
| `kivact_types::clock::ClockPhase` | 同上 | E01・W3b | `Running { resumed_ns }`、`Stopped { since_ns, resumed_ns }` |
| `crates/kivact-types/src/lib.rs` の `pub mod clock;` | `pub mod check;` の次の行 | E01・W3b | root の `pub use` は足さない |

E01 は W3b で、W3a の S10 が変えた後の `crates/kivact-types/src/lib.rs` に 1 行足す（INC-24。前の版の要調整 A-1 は解決済み）。`MotionClock` と `TimeScale`（`kivact_types::motion`）は変えない。client（runtime）が受ける `FrameTick.clock` の型は今のまま `MotionClock`。

### 4.2 付属書 C へ（E-REQ-1〜6 の engine の API）

| 付属書 C の要求 | unit・wave | 名前・署名（`kivact_engine`） |
|---|---|---|
| E-REQ-5 | E08・W4 | `contract::TreeChanges { pub upserted: Vec<(WindowId, NodeId)>, pub removed: Vec<(WindowId, NodeId)> }`、`api::Reader::record_tree_changes(&self, on: bool)`、`api::Reader::take_tree_changes(&self) -> TreeChanges`（受け手は `Reader`。C06 の手順の `Writer::record_tree_changes` は `Reader::` と読む。INC-02） |
| E-REQ-6 | E30・W4 | `contract::InputOrigin { Platform, Injected(RequestId) }`（`Default` は `Platform`）、`contract::Routed<T>` の `pub origin: InputOrigin`。`HostRequest::Inject` で注入した入力の event は `Injected(その request)` |
| E-REQ-1 | E31・W4b | `api::Writer::set(id, PropValue::InputValue(..))`（と `set_input_value`）は、その node の `TextChanged` を client が受け取ってまだ dispatch していない間（turn に渡し、`Writer::deliver(&self, event: &HostEvent)` が呼ばれる前）は書かず `false` を返す。core の turn の event の loop は各 event の dispatch の前に `Writer::deliver(event)` を呼ぶ（E31 が `launch/runtime.rs` に 1 行足す。C07 は `edits.deliver(event);` を消すときこの行を残し、R06 はこの行も turn の関数と一緒に移す） |
| E-REQ-2 | E31・W4b | `contract::LayoutReport { pub rect: Rect, pub layout_rect: Option<Rect>, pub text_truncated: bool }`、`api::Reader::layout_report(&self, id: NodeId) -> Option<LayoutReport>` |
| E-REQ-3 | E31・W4b | `contract::NodeFlags { pub visible: bool, pub in_live_region: bool }`（`Clone, Copy, PartialEq, Eq, Debug, Default`）、`api::Writer::watch_flags(&self, id: NodeId, watch: bool)`、`api::Reader::node_flags(&self, id: NodeId) -> NodeFlags`、`HostEvent::NodeFlagsChanged { node: NodeId, flags: NodeFlags }`（`Writer::commit` が見張っている node を比べて出す）。E31 は core の `client/dispatch/delivery.rs` に何もしない腕 `HostEvent::NodeFlagsChanged { .. } => {}` を足す。C07 はこの腕を自分の腕に置き換える |
| E-REQ-4 | E31・W4b | `api::Reader::font_generation(&self) -> u64`（`Writer` の font の登録ごとに増える） |

- tree の変更記録で記録する変化：client の書込み（node の作成、型付きの prop、scene、子の挿入・移動・削除の親）の node と、消えた node（削除・exit の終わり）。書込みの後に消えた node は `removed` だけに入る。記録しない：入力（hover、focus、scroll、編集の text）。記録していないときの費用は `EngineLog` の enum の判定一つだけ（PERF-9）。
- C06（W4b）の前提は E08・E30、C07（W5a）の前提は E31（INC-01）。

### 4.3 付属書 A へ（`content_version`、AR-03・PERF-7）

- **field**：付属書 A の A09（W5b の鎖）が `arvane_protocol::CompositeLayer` に `pub content_version: u64` を足し、engine の `src/lower/content_versions.rs` に `fresh_content_version()`（process で一意に増える値）を置き、`into_layer` と `append_overlay_layer` にそれを書く。契約は付属書 A 4.2 節（同じ window の連続する 2 frame で同じ `key` に同じ版を渡すのは items が等しい約束。上げすぎは正しさを損なわない）。
- **engine が上げる規則（E14、W6b。A09 の後）**：
  - window の frame の layer：list を作り直した frame と、scroll で一部を歩き直した frame は、すべての layer が新しい版。それ以外の frame では、layer に「前の frame から持ち越した命令でない命令」が一つでも入れば新しい版、入らなければ前の frame の同じ `LayerKey` の版。持ち越しとは、draw cache が当たった node の再生（`Recorded::Commands`）と、その node の opening。当たった node でも `Recorded::Live`（画像・pattern・scene の layer と item を描く node）は毎 frame 新しい命令と数える：画像の中身（animation の frame、外部の画像の更新）は命令に表れずに変わる（EA-01）。debug と `testing` の照合（E12 の `VERIFY_DRAWS`）は別の出力に描くので、照合した kept の drawing も持ち越し：照合の有無で版は変わらない（EA-01）。
  - 新しい版の値はいつも `fresh_content_version()` から取る。
  - 数えない layer（drawable の layer、export、snapshot など window の frame 以外の composition）は毎回 `fresh_content_version()`（`content_versions::stamp_fresh`）。DevTools の overlay は `append_overlay_layer` の `fresh_content_version()` のまま（INC-04）。
  - 動く layer の matrix（`LayerSpec::matrix` → `CompositeLayer::transform`）の変化は中身の変化ではないので版を上げない。
  - 版の上げ忘れは、付属書 A の debug build の hash の照合（PERF-7）が検出する。E14 は A 4.2 節が求める試験 `an_unchanged_layer_keeps_its_content_version_and_a_changed_one_bumps_it` と `a_changed_layer_bumps_its_version_and_a_sibling_layer_keeps_its_own` を engine の試験（`crates/kivact-engine/tests/content_version.rs`）として足し、debug（照合あり）と release（照合なし）の両方で走らせる。
- **順序**：A の鎖（A05〜A10）は W5b、E14 は W6b、A16（hash と inline）の前提に E14 が入る。
- **AR-06（engine が arvane-text・arvane-compose・arvane-asset・arvane-shell を直接使う）の engine 側の `use` の書換え**：A06 が W5b の鎖で `kivact-engine/` 全体を持って行う（作業指示書 4 章。前の版の要調整 A-2 は解決済み）。

### 4.4 公開 API の変化（`kivact::custom`、E17・E20、W5a の鎖）

| 変更前 | 変更後 |
|---|---|
| `EditOutcome::Done { changed: bool }` | `EditOutcome::Done`（text の変化は host が revision で知る） |
| `EditOutcome::Copy { text, changed }` | `EditOutcome::Copy(String)` |
| `EditRequest<'a>`、`EditRequest::Paste(&'a str)` | `EditRequest`、`EditRequest::Paste(String)` |
| `EditContext<'a>`（`EditLayout<'a>` で行を借りる） | `EditContext`（寿命が無い。`Clone` を持たず、behavior へは `&EditContext` で渡る。editor の行の cache を `Rc` で共有し、文書の全部の行に問われた時に答える（写さない）。`line_count`、`line_of`、`line_range`、`x_of`、`offset_at` は名前も引数も同じ。`offset_at` の答えは `TextGeometry` の停止位置（grapheme と shaping cluster の共通の境界）） |
| `EditContext::new(.., Option<EditLayout>)`（公開） | `#[doc(hidden)] EditContext::new(.., Option<kivact_editing::host::EditLayout>)`（host だけが作る） |
| `kivact_editing::{EditLayout, Lines}` | `kivact_editing::host::{EditLayout, Lines}`（`#[doc(hidden)]`。寿命が無く、`Lines` の query は `Option` を返す。engine の `SharedLines` だけが実装する） |
| — | `EditState::select(range)`、`EditState::move_vertically(to, extend, goal_x)`、`EditState::goal_x()`（E17）、`EditState::word_at(at)`、`EditState::line_at(at, multiline)`（E20） |
| behavior の型 `Fn(&mut EditState, EditRequest<'_>, &EditContext<'_>) -> EditOutcome` | `Fn(&mut EditState, EditRequest, &EditContext) -> EditOutcome` |
| `kivact_editing::{Editor, Journal, Incoming, Taken, Damage}` | `kivact_editing::host::{...}`（`#[doc(hidden)]`、engine だけが使う。E09） |

付属書 G（docs）と付属書 C（`custom` の doc）は、`EditContext` が文書の全部の行に答え（問われた行だけを shape する）、request の間だけ使えることを書く。付属書 U（W7b の U36 の interaction の作り直し）は、E17・E20 の名前（`goal_x()`、`move_vertically(to, extend, goal_x)`、`word_at(at)`、`line_at(at, multiline)`。`set_goal_x` は無い。目標の列は `move_vertically` だけが持ち、他の移動と編集で消える）を使い、`text_edit_policy` の goal column（E17 の形）を残す（INC-05）。

### 4.5 engine の crate 間の内部 API の変化

| 変化 | unit・wave | 呼出し元 |
|---|---|---|
| `contract::NodeInput` の 8 個の `Option<bool>` → `flags: InputFlags`、`contract::{InputFlag, InputFlags}` を新設 | E18・W5a の鎖 | kivact-core `view/tree.rs`、kivact-test `facts.rs`（E18 が直す） |
| `kivact_layout::Layout::{push, reset}` → `forget`、列は `LayoutTree::slot_count` まで自動で伸びる。`LayoutTree::slot_count` を新設 | E03・W3b | engine だけ |
| `kivact_layout` の `compare`・`compare_measurements`・`rebuild_all`・`Geometry`・`LayoutDifference` を feature `reference` に | E23・W7a | engine の `testing` だけ |
| engine の feature `devtools`（kivact-core の `devtools` が有効にする） | E29・W8 | kivact-core の Cargo.toml（E29 が直す） |
| `Reader::record_tree_changes`、`Reader::take_tree_changes`、`contract::TreeChanges`（window 付き） | E08・W4 | 付属書 C（C06） |
| `contract::InputOrigin`、`Routed::origin` | E30・W4 | 付属書 C（C06）、kivact-test `screen/host_sync_tests.rs`（E30 が直す） |
| `Writer::{deliver, watch_flags}`、`Reader::{layout_report, node_flags, font_generation}`、`contract::{LayoutReport, NodeFlags}`、`HostEvent::NodeFlagsChanged` | E31・W4b | 付属書 C（C07）、core の `launch/runtime.rs`・`client/dispatch/delivery.rs`（E31 が 1 行・1 腕） |

### 4.6 architecture.md に書くこと（付属書 G へ）

- 時刻：host の ns と App の motion の ns の二つ。motion の ns は App に一つの `kivact_types::clock::Clock`（どの window も描けない間は止まる、DevTools の速さ）だけが作る。motion の記録はすべて motion の ns。frame の要求と期限は host の ns。開始時刻を後からずらさない。
- slot の列：node の slot ごとの列は `model::aggregate::Slots` が持ち、push と reset は `Model::create_slot` と `Model::remove_slots` の一か所。layout の列は slot の数まで自分で伸びる。
- PERF-6 の実装：draw cache は `Slots::draws`（描画中の node だけ、list の作り直しで外れた node の分を捨てる）。描き直しの印（`WRITTEN|REDRAW|SCROLLED`）、文字色を変える state の変化の subtree、font、host の全描き直し（`invalidate_paint`）、reduced motion の切替えで捨てる。debug の `testing` では再生の前に、node の今の paint と箱から別の出力に描き直して照合する（frame・cache・layer の版には混ぜない）。restack は重なりの事実の変化だけ。scroll は container の subtree の歩き直し：置き換えた範囲の古い transform・layer を消して番号を詰め（作り直した list と等しい）、list から外れた node の歩きの記録と draw cache を消す。動く layer は culling せず matrix を毎 frame 置き直す。
- hit test は `paint::list::hit`（paint list の ink・子の索引・重ね順を使う）。debug の self_check の参照の walk は索引も list の順も使わず、すべての子を model の重ね順で調べる。`HitInk` と `LayerView` は無い。layer の facts は `paint::list::facts`。
- editing の behavior の `EditContext` は editor の行の cache を `Rc` で共有し、問われた行だけを shape する（文書を写さない）。caret の停止位置は `TextGeometry` の一つの実装（grapheme と shaping cluster の共通の境界）。host は shaper を behavior の間に借りない。
- 入力の配送の受理（`Admission`：modal が開く前に受理した drag と adoption）は engine が持つ（設計書 7.3 の EN-26）。pointer の track は engine が決めたこと（over、hover、gesture）だけを持ち、sample は渡して転送する。
- PERF-1 の例外の文：Tab 押下の一覧（`focus_order`）は利用者の操作の速さで全体を数える（4.7 の D-2 で決定済み）。a11y の Metadata の経路は順序を一度だけ数える。

### 4.7 設計判断の結果（親が決定済み。設計書 7.3 節。すべて推奨どおり採用）

- **D-1（決定済み・採用）（EN-30）**：`Launch::Starting` は `Handler::start` の中で `LaunchArgs` を所有のまま取り出すための置き場で、外から観測されない（`host()`・`host_mut()` は `Running` でない時に panic する）。消すには `Client` と `arvane::Engine` を既定値で作れる必要があるが、`Client` に既定値は無く、作ると S-7（未完成の object）に反する。**推奨：EN-30 は実装しない**（`HostPhase::Starting` と同じ事実の重複ではない）。採るなら別の unit が要る。
- **D-2（決定済み・採用）（EN-11 の Tab 押下）**：`move_focus`（Tab・Shift+Tab）と `roving_targets` は roving group を一つの stop にまとめる規則（`grouped_tab_stops`）を全体の順序の上で行う。前後どちらにも一歩ずつ進む走査に作り直すと APG の試験の範囲の挙動が変わる危険がある。**推奨：E06 のとおり、毎 frame・毎書込みの経路（再検証、削除、scene の書込み）だけを一歩ずつの走査にし、利用者の操作の速さの Tab 押下は全体の順序のまま残す**（`focus_order` の doc に理由を書く。E06 手順 2）。
- **D-3（決定済み・採用）（EN-12 の折返しの splice）**：折返しの editor の行は hard line を幅で折った row で、`chunks.rs` の `splice` は hard line で切るだけで row を作り直せない。触れた hard line を折り直す splice は `rows()` を使う新しい処理で、本書の手順に書ける確度が無い。**推奨：後続（設計書 13 章）に送る**。E09 は damage の記録の一本化と、幅を測る cache（`measure_lines`）への damage の受渡しだけを行う（幅を測る時は折返しの幅が無く splice が効く）。
- **D-4（決定済み・採用）（EN-23 の kivact-layout の `unrounded` の廃止）**：差分の layout の丸め（`tree.rs` の `unrounded_top`、`round_changed`）は、その pass で触れない祖先の `unrounded` を読むので、pass の間だけの作業域にはできない。**推奨：`unrounded` と `final_layout` は今のまま残し、taffy の cache の遅い確保（E28）だけを行う**。

## 5. 自己点検

### 5.1 所見の割当て

EN-01〜EN-29、EN-31、EN-32 はすべて 1.1 節の unit に割り当てた（EN-10 は S の部分、EN-26 は `PointerSource` の写しだけ。設計書 7.3 のとおり）。EN-30 は D-1 で parent の判断を待つ。EN-11 の Tab 押下（D-2）、EN-12 の折返しの splice（D-3）、EN-23 の `unrounded`（D-4）は、所見のそれ以外の部分を E06・E09・E28 で実装し、残りを要設計判断にした。付属書 C 向けは E08・E30（W4）と E31（W4b）、付属書 A 向け（E14）は W6b（A09 の後）。

外部の review（`20261118_1237_CODEX_REVIEW.md`、設計書 12 章の決定）の本書の分：EA-01（E12 の照合を別の出力に、E14 の持ち越しの規則と試験）、EA-02・EA-03（E13 の `compact`・`left_ids`・`forget_left_draws`、E32 の試験）、EA-11・EA-12・EA-16（E17 の `SharedLines`・共有する行の cache・`max_length` のときだけの数え、E17 の三つの試験と測定）、EA-13（E24 の `HitWalk::Complete` の全子の列挙と試験）、EA-14（E12 の `verify_draw`・`verify_over`、E32 の試験）。PLAN-01・PLAN-04・PLAN-07 は 1.4 節と各 unit の確認、TV-09 は E14・E24・E32 の不変条件の試験。

### 5.2 実装者に残した判断が無いこと

手順の中の「compiler の error が示す所を、書いた規則で直す」という指示は次だけで、どれも規則（何を何に替えるか）を書き、場所の一覧を出すコマンドを付けた：E03 手順 7（分解の書換え）、E11 手順 3（field の読み → 同名の method）、E15 手順 6（`Host::windows` の書換えの規則 (a)〜(d)）、E20 手順 2・4（`SelectText` を経路として扱う）、E21 手順 2（`modifiers` の引数を `input` に）、E23 手順 6（`Clone` の derive を戻す）、E29 手順 4（未使用と言われた item に同じ cfg）。`RenderCmd` の全 variant を書く `holds_pixels`（E12 手順 3）は、規則（`Paint` を持つ variant は `Pattern` か、それ以外は `false`、`_` の腕を書かない）を決めてある。

### 5.3 同じ wave の lane がファイルを共有しないこと

配置は作業指示書 4 章（SCHEDULE）。同じ wave の他の付属書の lane との重なりは作業指示書の表が確かめた（INC-24・INC-27・INC-28 で W3b・W5a/W5b・W6a/W6b・W7a に分けた）。本書の lane の間は次のとおり。

| wave | E1（または鎖・E）のファイル | E2 のファイル | 共有 |
|---|---|---|---|
| W3b | E01・E02：`kivact-types/src/{clock,lib}.rs`（S10 の後）、`model/frame/*`、`model/frame.rs`、`model/app_model.rs`、`model.rs`、`model/state_motion*`、`model/presence/{state,size}.rs`、`model/layout_motion/*`、`model/playback/{tick,state}.rs`、`model/motion_run.rs`、`host/{pump,events,window_changes}.rs` | E03・E04：`model/{aggregate,lifecycle}.rs`、`editors`・`measure`・`paint.ink/look` を読むファイル（`model/keys.rs`、`model/edit/*`、`model/interact/*`、`model/focus/navigation.rs`、`model/pointer/{machine,hover}.rs`、`model/semantics.rs`、`model/text.rs`、`model/apply.rs`、`measure/*`、`paint/*`、`a11y/*`、`check/*`、`host/write/reader.rs`、`host/testing/*`）、`tree/*`、`kivact-layout/*` | 無し（E03 の置換の一覧に `host/{pump,events,window_changes}.rs`、`model/frame/*`、motion の directory は入らない） |
| W4 | E05〜E09・E30：`model/edit/*`、`model/focus/navigation.rs`、`model/keys.rs`、`host/events.rs`、`host/write/{reader,writer,typed}.rs`、`measure/{state,editor,input}.rs`、`model/apply.rs`、`paint/state.rs`、`tree/node_tree.rs`、`contract/{devtools,routed}.rs`、`contract.rs`、`model/events.rs`、`model/lifecycle.rs`、`model/frame/state.rs`、`model/app_model.rs`、`model/pointer/{machine,shell}.rs`、`kivact-editing/*`、`text_input/*`、`text_input.rs`、`kivact-test/src/screen/host_sync_tests.rs` | E10・E11：`model/changes.rs`、`model/interact/{sync,text_geometry,caret,geometry,hit}.rs`、`model/aggregate.rs`、`host/pump.rs`、`layer.rs`、`paint/list/{look,walk}.rs` | 無し |
| W4b | E31（lane E）：`host/write/{reader,writer,typed}.rs`、`contract.rs`、`contract/events.rs`、`model/app_model.rs`、`model/semantics.rs`、`host/pump.rs`、core `launch/runtime.rs`（1 行）、core `client/dispatch/delivery.rs`（1 腕） | — | 付属書 C の C06（core の `devtools/`、`view/devtools_hooks.rs`、`view/mount/style.rs`、`launch/inspect.rs`、`kivact-devtools-agent`、`kivact-test/src/screen/state.rs`）と重ならない |
| W5a | E16（lane E1）：`a11y/build/{dirty,identity,node}.rs` | E12・E13：`paint/{state,frame_scene,canvas,snapshot}.rs`、`paint/list/{invalidation,walk}.rs`、`lower/scene_out.rs`、`model/{aggregate,states,scrollbar,transform,changes}.rs`、`model/interact/{sync,visual,geometry,hit}.rs`、`model/pointer/hover.rs`、`model/frame/ticks.rs`、`model/state_motion/*`、`model/layout_motion/steps.rs` | 無し。鎖の E15・E17・E18・E20 とも共有しない：E15 は `host/*`（`write/{reader,writer}.rs`、`testing/*` を含み `write/typed.rs` を除く。手順 2 の path の置換が当たる file も `host/*` だけ）、`host.rs`、`model/window.rs`、`model.rs`、`model/app_model.rs`。E17・E18・E20 は `kivact-editing/*`、`model/{keys,apply}.rs`、`model/edit/*`、`model/pointer/{machine,state,touch,shell}.rs`、`measure/{editor,input}.rs`、`text_input.rs`、`text_input/{shaping,word}.rs`、`host/events.rs` と `host/testing/self_check.rs`（E17 の 2 行ずつ。鎖の中で E15 の後）、`contract/{node.rs,contract.rs}`、`host/write/typed.rs`、`tree/node.rs`。`src/model/interact/sync.rs`（E2）は E17 が触らない。`src/embed.rs` は W5a の鎖の C11・C13 だけが持つ（E-REQ-7） |
| W6b | E19・E21：`model/{aggregate,app_model}.rs`、`model.rs`、`host/write/writer.rs`、`host/{pump,launch}.rs`、`embed.rs`、`frame_log.rs`、`lib.rs`、`model/pointer/*`、`model/pointer.rs` | E14・E32・E24：`lower/{scene_out,content_versions,shape}.rs`、`paint/*`、`paint/list/*`、`paint.rs`、`layer.rs`、`model/interact.rs`、`model/interact/{hit,geometry}.rs`、`model/drawables/hit.rs`、`model/drawables.rs`、`host/testing.rs`（E32 の 1 行）、`host/testing/test_host.rs`（E14 の method 一つ、E32 の二つ、E24 の一つ）、`crates/kivact-engine/{Cargo.toml,tests/content_version.rs,tests/draw_cache.rs,tests/hit_reference.rs}` | 無し |
| W7a | E22・E23・E25・E26（lane は E1 だけ） | — | — |
| W7b | E27・E28（lane は E1 だけ） | — | — |
| W8 | E29（lane は E1 だけ） | — | — |

外部の review の修正（設計書 12 章）で作業指示書 4 章の表が変わる所：W5a の E15 を E1 から鎖へ移した（鎖の順は … C18 → E15 → E17 → E18 → E20、E1 は E16 だけ）。W6b の E2 に E32 を足した（E14 → E32 → E24）。E15 の file は鎖の R・C の unit の file と重ならず、E32 の file は W6b の E1（E19・E21）の file と重ならない。

W6a の S の鎖（S13・S14）は engine・layout・editing を持つ。本書の W6b 以降の unit はその後の名前で書く（1.2 節）。前の版の要調整 A-3 は W5a/W5b の分割で解決した（INC-27）。
