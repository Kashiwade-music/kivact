# 未解決・差戻し（Orchestrator が更新）

## 基点の失敗
（W0 の基点の gate で失敗した試験。改修の失敗ではない）

- `kivact-ui-soft` の `--test consumer`：`chat_keyboard::composer_inserts_shift_enter_and_recalls_the_last_submission`（`crates/kivact-ui-soft/tests/consumer/chat_keyboard.rs:58`）。left=`Some("Another message")`、right=`Some("First line\nSecond line")`。基点 `2b8df5ac` で既に失敗。直さない。
- 1 回目の `cargo test --workspace` はこの binary で止まった（cargo の既定の fail-fast）。`--no-fail-fast` で全件を再実行した結果：`--all-features` も `--no-default-features` も失敗は上の `chat_keyboard` の 1 件だけ（81・79 binary を実行）。
- GPU E2E（`kivact-public-api-tests --test e2e`、KIVACT_TEST_GPU=1、751 件中 4 件失敗、基点で既に失敗）：
  - `app_handle::close_during_setup_exits_after_the_first_turn`：`crates/kivact-test/src/screen/driver.rs:50`、「GPU device did not start within five seconds」（GPU の起動の遅れ。環境の差の疑い）
  - `compositor::idle_window_redraws_whole_after_release`（`dev/tests/kivact-public-api/tests/e2e/compositor.rs:561`）：first frame after release rasters 4000 px
  - `frame_sources::incremental_frames_match_a_fresh_render`（`.../e2e/frame_sources.rs:264`）：20 partial frames が fresh frame と一致しない
  - `layout_scroll::a_long_list_draws_what_reaches_its_viewport_from_rows_outside_it`（`.../e2e/layout_scroll.rs:187`）：layout pass 後の pixel が fresh frame と一致しない
  - 直さない。改修後の同じ試験の結果と比べる基準にする。

## 差戻し
（設計者へ返す STOP。unit、場所、STOP の文、付属書の節）

（状態：ユーザーの回答（1A・2A・3A）で解消。T05・T07・T09・T10 は 180 字の上限を外して L・R を切らずに残し、T09 の R は戻した（`e13a418c`）。T13 は完了条件の rg を crate 名 `kivact-devtools-client|kivact_devtools_client` にして 0 件。T17 は unsafe_code を allow に足して commit（`6aee2d2b`）。未決は T14 の README の段落の置き場所のみ）

- **T05**（W1b、lane T1、commit `6c4aa02e`）：`crates/kivact-ui/src/data_table/types.rs` の `DataTable` の legacy の印の行が 423 文字。STOP の文：「理由文を切ると field 列（L）が残らず、field 列だけでも 180 文字を超えるので、180 文字の規則を守れない」。付属書 M（M2 の印の書き方）の行長の規則と、L・R の切り方。Orchestrator の処置：L・R を切らずに残した。設計者の判断を待つ。
- **T07**（W1b、lane T2、commit `a79c28b9`）：legacy の印の行が 180 文字を超える 2 件。`crates/kivact-core/src/view/devtools_state.rs:57`（`SessionData`、237 文字。field 列 209 文字）、`crates/kivact-core/src/view/element_data.rs:516`（`Common`、306 文字。field 列 263 文字）。STOP の文：T05 と同じ。付属書 M（M2）の行長の規則。Orchestrator の処置：切らずに残した。設計者の判断を待つ。
- **T09**（W1b、lane T3、commit `8cd6ea4c`）：legacy の印の行で、L（field 列）だけで 180 文字を超えるものが 3 件あり、subagent が理由文（R）を落とした。対象は `crates/kivact-style/src/text.rs:258`（`TextExtras`、行は 346 文字）、`crates/kivact-style/src/sheet/fields.rs:228`（`TransformFields`、199 文字）、`crates/kivact-runtime/src/reactive/data.rs:57`（`Runtime`）。STOP の文：「L だけで 180 文字を超え、R を入れる余地が無い」。付属書 M（M2）の行長の規則と、R を落としてよいかの判断。元の文は `b4aebab8` の該当 file に残っている。Orchestrator の処置：T09 の commit は R を落としたままにした。設計者の判断を待つ。
  - 同じ commit で `crates/kivact-runtime/src/task/action.rs` の `ActionCell` の印の W の文が、規則の「最初の `;` `,` `.` まで」で途中で切れた（規則どおり。文は読みにくい）。設計者の判断を待つ。
- **T13**（W1b、lane T5、commit `0a373ea5`）：完了条件の `rg -n "devtools-client|devtools_client" crates dev Cargo.toml` が 2 件残る。いずれも試験の関数名（`dev/tests/kivact-public-api/tests/e2e/devtools.rs:25` の `a_devtools_client_inspects_drives_and_reviews_a_test_app`、`crates/kivact-devtools-agent/tests/agent.rs:270` の `open_devtools_notifies_a_connected_devtools_client`）。STOP の文：「完了条件の rg が、付属書 T13 の『試験の名前は変えない』と食い違う」。付属書 T13（C-core.md・E-engine.md も同じ名前を参照）。Orchestrator の処置：試験の名前は変えず残した。設計者が完了条件の rg の扱いを決める（T01 と同じ型）。
- **T17**（W1b、lane T6、commit なし）：`dev/tests/docs/src/checked.rs` の生成元の `checked_body`（付属書 T17 の手順 1〜3）で、`gpu-texture-sharing.md` の no_run block 4 の `unsafe`（`link.provide`・`target.submit`）が、workspace の lint `unsafe_code = "deny"`（Cargo.toml:96）で error になる（`checked.rs:2652`・`2663`）。STOP の文：「checked module の `#[allow]` の一覧（手順 3）に `unsafe_code` が無く、no_run block の unsafe が deny の lint で error になる」。付属書 T17 の手順 3。Orchestrator の処置：`#[allow]` を足さずに止めた。T17 の変更（build.rs・checked.rs・lib.rs・docs-pairs の jp 4 file ほか）は、lane T6 の worktree の stash（`T17 WIP`）に保存した。設計者が決める：(a) checked module の allow に `unsafe_code` を足す（付属書の文面を直す）、(b) block 4 の書き方を変える。判断が出れば、stash を戻して T17 を続ける。
  - lane T6 の T18 は止まった（subagent の STOP）。T18 の本文は T17 に依存しないが、確認の `cargo xtask check --no-write --only docs-pairs --strict` と `cargo test -p kivact-docs --doc -- readme ...devtools` が、T17 の所有の 4 file（jp の kit tutorial 2 件、`canvas.md`、`windows.md`）と、T17 が置き換える `build.rs` に依存する。よって lane T6 の T19〜T21 も T17 の解消まで止める。
- **T10**（W1b、lane T3、commit `1139ee17`）：legacy の印の行が 180 文字を超える。`crates/arvane-virtual/src/window.rs:142`（`VirtualWindow`、308 文字）。L（field 列）だけで 180 文字を超えるので、R を切っても 180 文字の規則を守れない。STOP の文：「L 単独で 180 文字を超え、付属書 M2 の『R を 180 文字で切る』が適用できない」。付属書 M（M2）の行長の規則と、field 列の表し方（長い field 列を許すか、書き方を決めるか）。Orchestrator の処置：切らずに残した。設計者の判断を待つ。
- **T14**（W1b、lane T5、commit `43086f3d`）：STOP ではない。報告のみ。kit の README（basic・soft・creative）で「Migrating from the previous release」と「Filled (standard) recipes」を消したところ、その後ろの段落（「carousel … Lyra recipes」など）が、前の見出しの下に残った（basic では「Catalog and tests」の下）。設計者が段落の置き場所を見るか決める。
- **T18**（W1b、lane T6、commit `e47a5da2`）：STOP。確認の試験 filter `readme for_kivact_user_en::how_to_guide::devtools` が実行 0 件（devtools の guide に rust の block が無く、`dev/tests/docs/build.rs` が module を生成しない。`--list` に `devtools` の doctest が無い。3 件は ignored）。STOP の文：「付属書 T18 の試験 filter が実行する試験を 1 件も選ばない」。付属書 T18 の確認の行。Orchestrator の処置：filter は変えず、期待値も緩めず、作業は続けた（docs-pairs と完了条件の rg は成功）。設計者が filter を直すか、この試験を省くかを決める。

## 設計者の回答（2026-11-18、W1a の 4 件。差戻しから外した）

- **S01**：Clippy の `disallowed_types` は構成式の path（`OnceCell::new()`・`OnceLock::new()`）も報告する。付属書 S の S01 のコード片を直した：`Deferred::new` と `SharedDeferred::new` にも `#[expect(clippy::disallowed_types, reason = "...")]` を付ける（field と `new` の 2 か所ずつ。設計 6.2 の DEC-13 の文も直した）。inline の `mod` で囲む形は `cell.rs` が style の non-leaf の規則に当たるので採らない。設計者が `b0e84c65` にこの変更を当てて、`cargo clippy -p kivact-types --all-targets -- -D warnings`、`cargo test -p kivact-types cell`（2 件）、`cargo xtask check --no-write --crate kivact-types`（`ok`）が通るのを確かめた。
  - Orchestrator：S lane の fix として `wt/rw-W1a-S` で付属書 S01 の新しいコード片どおりに 2 つの `new` に expect を足し（文面は付属書のとおり）、確認を通してから W1a を merge する。
- **T01**：受け入れる（コードは変えない）。付属書 T01 の完了条件を「`rg` が出すのは旧い印を検出する仕様の文字列だけ（`state/check.rs` の `OBSOLETE` と検査の文面、`state/tests.rs` の試験）」に直した。
- **T02**：受け入れる（現状のまま、`kivact-devtools-client` を `kivact-devtools-protocol` の `visible_to` に残す）。付属書 T02 の表と「消すもの」を直し、名前を消すのは T13 の手順 9 にした（`[visible_to]` の行と `[rework.pending]` の `state` の列の 2 か所。T13 の所有に足した。同じ T5 の lane で後の T15 と順に持つ）。merge の前に `workspace.toml` の行末（LF・CRLF）を基点と同じにする。
- **T04**：受け入れる。`dev/README.md:41` は T21 が file ごと置き換える（T21 の rg に `devtools-codegen` が既にある）。kit の tutorial の jp の 2 件（`02-build-the-settings-form.md`、`03-dark-mode-and-feedback.md`）は T17 の所有と手順 5 に足した。原因は、日本語の `field("メンション", ...)` が複数行の形で `hide_label(true),` の後に末尾の comma を持ち、英語より token が 1 つ多いこと。T04 の完了条件も 4 件と `dev/README.md` の 1 行を前提に直した。

## 設計者の回答（W1b の 3 件。ユーザーは「No preference」と答えたので、推奨を採った）

設計者が回答を確かめ、付属書 T の文面を直した（T17 の手順 3 の `#[allow]` に `unsafe_code`、共通手順 M2 の 180 字の規則を削除、T13 の完了条件の rg を crate の名前 `kivact-devtools-client|kivact_devtools_client` に）。

- **T17（unsafe_code）**：採用：checked module の `#[allow]` の一覧に `unsafe_code` を足す。対象は doc test の no_run block 4 の unsafe だけ。付属書 T17 の手順 3 の文面は設計者が直す。
- **180 字の行（T05・T07・T09・T10）**：採用：行長の上限（180 字）を外し、L（field 列）と R（理由文）は切らずに残す。xtask は行長を検査していない。付属書 M2 の「180 字以内」の文は設計者が直す。T09 の 3 件は、落とした R を元に戻す（fix lane `T3-fix`）。
- **T13（rg）**：採用：試験の名前は変えず、完了条件の rg は試験の関数名 2 件を除いたものとする。付属書 T13 の文面は設計者が直す。

## 設計者の回答（W2a・K01、2026-11-18）

- **K01**：private theme は保つ（kivact-devtools は basic に依存しない）。原因は tab の recipe だけ：標準の tab は左右の padding 12 px と tab の間の gap 4 px で、basic（左右 6 px、gap 0）より 1 tab あたり約 14 px 広く、11 個の panel の tab が 800 px を超える。付属書 K の K01 の手順 2 の `look.rs` に、basic と同じ幅の tab の recipe（`tab_list`・`tab`、`UiRecipes::default().tabs(..)`）を足した。設計者が `6761d39d` にこの変更を当てて、`cargo clippy -p kivact-devtools --all-targets -- -D warnings`、`cargo xtask check --no-write --crate kivact-devtools`（`ok`）、`cargo test -p kivact-devtools`（`tests/app` の 39 件を含む全件）が通るのを確かめた。試験の期待値は変えていない。
  - W7a の U20（`UiRecipes` の削除）が、この `look.rs` を `TabsTheme` の形に直す（付属書 U の U20 の所有・手順 11・確認・完了条件に足した）。
  - Orchestrator：lane K の fix として `wt/rw-W2a-K` の `look.rs` を付属書 K01 の新しい手順 2 のコード片どおりにし（doc の 2 行、`use`、`.with(..)` の 1 行、`tab_list`・`tab`）、確認を通してから lane K を続ける。

## 設計者の回答（T14・T18・T22、2026-11-18）

- **T14**：受け入れる（`43086f3d` はそのまま）。残った段落の置き場所は、kit の README を所有する付属書 G の G12（W11b）の手順 9 に足した：basic は「## Catalog and tests」の下の `kivact::ui::carousel(..)` の段落の前に見出し `## Standard components in this look` を置く。soft は `Pagination presentation helpers:` の段落を「## Standard components」の節の最後へ移す。creative・pop・game は該当なし。
- **T18**：受け入れる（`e47a5da2` はそのまま）。`README.md` の Rust の block は `no_run` の 1 つだけで、T17 の後は doctest ではなく checked module の型検査になる。そのため `--doc -- readme` の filter は何も実行しない。付属書 T の T17 と T18 の確認から、この doctest の行を消した（T18 は `cargo check -p kivact-docs --all-targets` にした）。T21 が書いた `dev/README.md` の同じ例と説明の文も、設計者が直した（README と英語の guide の `no_run` は `cargo check -p kivact-docs --all-targets` が型検査する）。
- **T22**：受け入れる（`7c2b0460` はそのまま）。T22 の確認の rg と、作業指示書 4.1・W1c の `rg -n "xtask-allow\(state-fields\)"` に `--glob '!dev/xtask/src/state/**'` を足した（旧い印を検出する文字列の directory。T01 と同じ）。

## KPI の後退
## KPI の未達
## 台帳

## 差戻しの判断（W1c・T22、2026-11-18）
- **T22**（W1c、lane T0、commit `7c2b0460`）：STOP。完了条件 3 つ目の `rg -n "xtask-allow\(state-fields\)|#!\[warn\(redundant_imports\)\]" crates dev examples templates` が、xtask 自身の旧い印の検出文字列 3 行（`dev/xtask/src/state/check.rs:12`・`check.rs:129`・`state/tests.rs:101`）に当たる。T01 の判断（検出仕様の文字列は除く）と同じ型。Orchestrator の処置：検査の文字列は消さず、T01 と同じ扱いで受け入れた（rg が出すのは検出仕様の文字列だけ）。付属書 T22 の完了条件は設計者が直す。

## 差戻し（W2a・K01、2026-11-18）
- **K01**（W2a、lane K、commit `6761d39d`）：STOP。付属書どおり、kivact-devtools の shell の theme を basic の recipe を持たない private theme（`crates/kivact-devtools/src/look.rs`）にすると、DevTools の tab 行が 800x600 の窓の外（x=843）に出て、`tests/app` の `review::` の 3 試験（`adding_a_comment_uses_the_window_and_refetches`、`open_folder_exports_the_review_and_reveals_the_json`、`a_removed_node_is_marked_and_cannot_be_highlighted`）が失敗する。basic の theme に戻すと 39 件とも成功する（試験の期待値は変えていない）。原因（tab の recipe か、ほかの token か）は未確認。STOP の文：「private theme で tab 行が窓の外に出て、review:: の 3 試験が失敗する」。Orchestrator の処置：lane K は止めた。K01 の commit は branch `wt/rw-W2a-K` に残し、統合 branch へは merge していない。設計者（ユーザー）が方針を決める。

## 差戻し（W2a・R03、2026-11-18）
- **R03**（W2a、lane R、commit `3990b637`）：付属書 R03 は `EffectNode` に `// xtask-allow(state): ...` の印を置くと書くが、xtask は「未使用の印」として拒否する（`EffectNode` は `&mut self` の method も内部可変の field も持たず、設計書 6.1 の状態の struct ではない）。Orchestrator の処置：印を外した（印は足さず、`#[allow]` も足さない）。付属書 R03 の「状態の閉じ方」は、現在の xtask の規則と食い違うので、設計者が直す。
- **R03**（リスク）：`pass_runs` は `stabilize` の最後にだけ戻る。ユーザー code が pass の途中で panic すると、次の pass で effect が早く打ち切られる可能性がある。付属書どおりの設計なので、この unit では変えず、報告に記録する。設計者が許容するか、panic 時の戻しを入れるかを決める。
- **K02**（W2a、lane K、commit `0e711247`）：付属書 K02 は `MobileContext` の直前に `// xtask-allow(state): ...` を置くと書くが、state 検査は `Binding<bool>` と `MachineHandle` を数えないため、counted field が `auto_toggle` の 1 個だけになり、印が「未使用の印」の error になる。Orchestrator の処置：印を外した（旧コードも同じ構成で印を持っていなかった）。付属書 K02 の「状態の閉じ方」の前提（`open` が counted される）は、検査の規則と食い違うので、設計者が直す。
- **S04**（W2a、lane A、commit `a94fb198`）：付属書 S04 は `StateStyles` の直前に `// xtask-allow(state): passive record ...` を置くと書くが、新しい `StateStyles` は counted field が `group`（bool）の 1 つだけで、state 検査の規則（2 つ以上の bool・Option・counter）に当たらない。xtask が「未使用の印」の error を出したので、印を外した。付属書 S04 の「状態の閉じ方」（「StateStyles は手順 3 の印の 1 行」）と食い違う。R03・K02 と同じ型の逸脱（付属書の印と xtask の規則の食い違い）。設計者が、手順 3 と「状態の閉じ方」を直すか、規則を決める。
- **K03**（W2a、lane K、commit `76e91102`）：STOP（1 項目）。付属書 K03 の手順 4 の末尾「`NodeMode::Connecting` を扱う key の handler に `if super::view::stale(state) { state.cancel(); return Handled::Yes; }` を足す」の対象が見つからない。`rg -n Connecting` は `editors/node/input.rs` に 0 件で、`key` は `Connecting` を見ない。`Connecting` を扱うのは `editors/node/edit.rs` の `invoke`（所有外）。STOP の文：「付属書 K03 手順 4 の handler の場所（`NodeMode::Connecting` を扱う key の handler）が存在しない」。Orchestrator の処置：その 1 項目は足さず、残りの K03 は commit した。設計者が場所（`edit.rs` の `invoke` か、別の handler か）を決める。K03 の他の確認はすべて成功。
- **S06**（W2a、lane A、commit `419107dd`）：付属書 S06 の手順 1 の `rename_protocol.pl` に欠陥がある。enum の variant 名まで書き換えて（`Gradient(Gradient)` → `RasterGradient(RasterGradient)`）、他 crate の使用側と食い違って compile が落ちた。一方、`use crate::paint::Paint;` は `(?<!::)` のため書き換わらなかった。subagent は variant の宣言と `use` を残す版（`rename_types.pl`）に直して進めた。他に、手順 2 の前に `arvane-render/src/gradient.rs` で vello の `Gradient as RasterGradient` の別名が衝突し、別名を外して `peniko::Gradient` と完全修飾に直した。`crates/arvane/tests/e2e/text_style.rs` の 3 か所の `TextSpan {` は heuristic で残り、手で `ResolvedSpan {` に直した。設計者が S06 の script と手順 2 の文を直す。

## 差戻しの判断（W2a の tier b、2026-11-18）
- **types の Role 行**：付属書 S（S02・S05・S06 などの節）が指定する `//! Role:` の文が、xtask の「Role は 80 字以内」の規則を超える（例：`shaped.rs` は 105 字、`scene/resolve.rs` は 102 字）。Orchestrator の処置：`wt/rw-W2a-A-fix`（`0f52ef4a`）で、意味を変えずに 80 字以内に縮めた。設計者が付属書の文を直すか、規則を決める。
- **scene が shaped を参照する規則**：付属書 S には `module_rules` の `scene` に `shaped` を足す指定が無く、`crate::scene::recording` が `crate::shaped` を参照して xtask の error になった。Orchestrator の処置：`dev/xtask/policy/workspace.toml` の `scene` に `"shaped"` を足した（`shaped` は `color`・`geometry`・`semantics_text`・`text` にだけ依存し、循環は無い）。設計者が付属書に反映する。

## 確認事項（W2b・D02、2026-11-18）
- **D02**（W2b、lane D2、commit `4f9dacfb`）：Linux 向けの `cargo check --target x86_64-unknown-linux-gnu` で、未使用 import の警告が 2 件出る（`crates/arvane-native/src/port/winit/foreign/x11.rs:13` の x11rb の `ConnectionExt`、`crates/arvane-native/src/port/x11/test_parent/queries.rs:5` の x11rb の `Connection`）。D02 の変更は x11rb に触れていないので、D02 由来ではないと見ている。どの commit で入ったかは未確認。Windows の clippy では見えない。Orchestrator の処置：所有外なので直していない。Linux の check を wave の締めで行い、出たら fix lane で直す。
