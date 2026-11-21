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
- CP1（commit `d615568c`、W4b の後。基準は B0 `c9518946`）。同じ条件で 3 回測り（`rework/kpi/CP1.md`・`CP1-r2.md`・`CP1-r3.md`、比は `*-ratio.md`）、行ごとに 3 回中 2 回で判定した。後退と判定した行（必達の上限を超えた）：K1 kv_startup main_to_first_present（1.16〜1.24、≤1.05）、K1 basic catalog startup（1.08、≤1.05）、K2 counter idle WS（1.03、≤1.02）、K2 3000 rows idle WS（1.59〜1.60、≤1.02）、K2 per_item box（1.62〜1.64）・text（1.68〜1.70）・button（1.71〜1.73）・row（1.76〜1.81）（≤1.02）、K4 3000 rows work p95（1.14〜1.42、≤1.05）と p99（1.11〜1.36、≤1.05）、K5 wheel p95 n3000（1 回目 0.99、2 回目 1.60、3 回目 1.14。判定は後退、≤1.05）、K5 counter click p95（1.24〜1.49、≤1.05）。Orchestrator は直さない。設計者に返す。
## KPI の未達
- 改善の必達の未達（3 回中 2 回以上で未達）：K3 3000 rows continuous CPU（0.92・1.02 は改善未達、3 回目 1.07 は後退。改善の必達 ≤0.85）、K5 pointer p95 n3000（3 回とも 0.86〜0.93、改善の必達 ≤0.85）、K6 basic-catalog no-gpu（3 回とも 1.011、改善の必達 ≤0.97）。設計者に返す。
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

## 差戻し（W2b・C01、2026-11-18）
- **C01**（W2b、lane C、commit `f4192d74`）：STOP。付属書 C01 の手順 4 の `StyleHook`（`Off` と `On`）は、devtools 無しの build で `On` が一度も作られないため、`variant On is never constructed` の警告になり、`cargo clippy -p kivact-core --all-targets -- -D warnings` が失敗する。手順の「`devtools_hooks.rs` に cfg を一つも置かない」と「`#[allow]` で黙らせない」を両立できない。修正案：(a) `On` variant だけに `#[cfg(feature = "devtools")]` を付ける（`view` に cfg が入る）、(b) `StyleHook` を `Option` の field を持つ struct にして `view` から構築する（enum の形を変える）。Orchestrator の処置：止めた。C01 の他の確認（check・devtools の clippy・xtask・e2e の 1 件）は成功。設計者が (a) か (b) を決める。

## 設計者の回答（W2b・C01、2026-11-19）

- **C01**：(b) の形にする（cfg も allow も置かない）。`StyleHook` を enum から `pub(crate) struct StyleHook(Option<Rc<dyn NodeStyleHook>>)` にし、node ごとの trait `NodeStyleHook`（`apply`・`forced`）を `view/devtools_hooks.rs` に足す。`DevtoolsHooks::style_hook` は `Rc<dyn NodeStyleHook>` を返し、`StyleHook::new` は `StyleHook(hooks(rt).map(|h| h.style_hook(rt, owner, id, at)))`。DevTools 側は `devtools/hooks.rs` の private の `NodeHook`（`id`・`sites`・`overrides`・`forced`）が `NodeStyleHook` を実装する。値は view が作り、view で作らない variant が無いので、devtools 無しの build で dead code にならない。付属書 C の C01 の手順 4・5・状態の閉じ方を直した。設計者が `f4192d74` にこの変更を当てて、`cargo clippy -p kivact-core --all-targets -- -D warnings`（devtools 無しと `--features devtools` の両方）と `cargo xtask check --no-write --crate kivact-core` が通るのを確かめた。
  - 同じ型の問題が W4b の C06 にもあった（`StyleHook::Node` を DevTools 側で作る）。C06 の手順 8 を、`NodeHook` の field と `apply` を置き換える形に直した（`view/devtools_hooks.rs` は変えない）。
  - Orchestrator：lane C の fix として `wt/rw-W2b-C` の `view/devtools_hooks.rs` と `devtools/hooks.rs` を付属書 C01 の新しい手順 4・5 のコード片どおりにし、C01 の確認を全部通してから lane C を続ける。

## 差戻し（W2b・D03、2026-11-19）
- **D03**（W2b、lane D2、commit `fda3cd6e`）：STOP。`rfd 0.18.0` が `objc2-app-kit 0.3.2` の feature を有効にし、その先で `objc2-uniform-type-identifiers 0.3.2` が新しく入る（HEAD の lock は 0.2.2 だけだった）。付属書 D の 1.3 節の DUP-AFTER の 1 つ目の comm が空でない。付属書の 4.2 節と 1.1 節は pollster と rfd の組だけを見ていて、この二重は予期されていなかった。二重の 2 crate（0.2.2 と 0.3.2）は、`arboard` と `objc2-app-kit` の経路。選択肢：(a) 二重を許容する（D03 はこの commit のまま完了）、(b) `rfd 0.18` を見送り、D03 を組ごと取り下げる（`pollster 0.4` と `rfd 0.17.2` のまま）。Orchestrator の処置：D03 の commit は lane D2 に残し、統合 branch へ merge していない。D04 は設計者の判断後。設計者が (a) か (b) を決める。

## 設計者の回答（W2b・D03、2026-11-19）

- **D03**：(a) 許容する（`fda3cd6e` はそのまま受け入れる）。`objc2-uniform-type-identifiers` 0.2.2 を lock に持つのは iOS の `objc2-ui-kit` 0.2.2（winit 0.30 の objc2 0.5 系の経路）で、0.3.2 は macOS の rfd 0.18.0 と `objc2-app-kit` 0.3.2 が引く。2 つの版は別の target でしか build されず、一つの binary に両方が入ることは無いので、K6（binary の大きさ）は後退しない（objc2 0.5 と 0.6 の系は既に同じ理由で二重）。rfd 0.18 を見送ると pollster 1.0 の再入の deadlock の修正も失う。付属書 D の D03 の目的と完了条件（DUP-AFTER の 1 つ目の `comm` は `"objc2-uniform-type-identifiers" 2` の 1 行だけ）と 5 章の 1 文を直した。D04 の DUP-BEFORE は D03 の後に取るので、D04 の完了条件（両方の `comm` が空）は変えない。
  - Orchestrator：D03 を完了として lane D2 を D04 へ進める（D03 のやり直しは不要）。

## 差戻し（W3a・R07、2026-11-19）
- **R07**（W3a、lane R、commit `e999ec1c`）：付属書 R07 の完了条件の rg（`LoopState|Registry<|\.live\(\)|running: Value<bool>|current\.borrow\(\)\.clone\(\)`）に 2 件残る：`crates/kivact-runtime/src/motion/animated.rs:161`（`target()` の Idle 分岐の `self.current.borrow().clone()`）と `:201`（`make_run` の `self.current.borrow().clone()`）。どちらも手順のコードに含まれず、per-step の clone ではない。Orchestrator の処置：手順を外れて書き換えず、そのまま残した。設計者が完了条件を直すか、2 件を直すかを決める。
- **R07**（逸脱）：`LoopCell::set_run` は付属書のコードでは tick ごとに rate signal を書く。印のコメントは「rate が変わったときだけ書く」と書いてあるため、before と比べて変わったときだけ `self.rate.set` する形にした（`Signal::set` は同じ値なら通知しないので、挙動は同じ）。Orchestrator の処置：そのまま受け入れる。

## 確認事項（W3a・S11、2026-11-19）
- **S11**（W3a、lane S、commit `583c3db4`）：文字の gradient を `Units::ObjectBox`（単位正方形）で書く付属書の設計では、px で書かれた gradient の stop 位置や半径が正しく出ない（長さの基準が 1 になる）。%・比率の stop は正しい。また `coordinate_paint`（canvas の座標 gradient）を文字に使うと、座標が user space のまま objectBoundingBox と解釈される。試験は gradientUnits の文字列だけを見ている。Orchestrator の処置：付属書どおりに入れた。設計者が見直すか決める。
- **S11**（付属書との差）：付属書の `TextFormat::tabular_nums()` は無く、draw の `TextFormat` の pub field `tabular_nums: bool` を使った。`paint_attributes`・`linear_def`・`radial_def` に加え、`layer_paint`・`gradient_paint`・`coordinate_paint` にも `Units` を通した（コンパイルのため）。Orchestrator の処置：そのまま受け入れる。

## 差戻し（W3a・C03、2026-11-19）
- **C03**（W3a、lane C、commit `efc5a622`）：完了条件の一部が未達（STOP 3 件と差戻し 1 件）。
  1. STOP：`crates/kivact-core/src/view/state.rs:current_window`。付属書は「全 owner は WindowScope の下にある」として `expect` にするが、virtual_list の行 owner を dispose するとき、cleanup の `custom/node.rs:focused_node` → `current_window` で WindowScope が無く expect が落ちる（`e2e_core` の `a_list_navigation_in_a_virtual_list_reaches_the_last_row_and_keeps_the_focus`）。expect 版で E2E の失敗を確認し、元の MAIN の fallback に戻した（RC-36 は未完）。設計者が、owner を dispose の経路で持たせる方法か、fallback を残す方法かを決める。
  2. STOP：`crates/kivact-core/src/view/mount/resolve.rs:prepare_draw`。付属書の `report_session` は `selection.is_some() || draw_with_layout.is_some()` だが、`create.rs` の FirstWrite に `draw_with_layout` が無く、`observe_layout`（bool、3 layer の or）しか無い。未変更（RC-41 は未完）。設計者が、FirstWrite に `draw_with_layout` を足すか、付属書の式を `observe_layout` に直すかを決める。
  3. STOP：`crates/kivact-core/src/view/elements/builders.rs`（mount_host の後に `n.at = None` する関数）。付属書は「`n.at = Some` の直後に `n.at = None`」と書くが、実際は `create::write`（`mount/create.rs`）の Some と builders の None の二段。`mount.rs` と `mount/create.rs` は C04 の所有。未変更（RC-41 の `at` は未完）。設計者が、`at` を `mount_host` の引数で渡す形にするか、Some と None を同じ関数にまとめるかを決める。
  4. 差戻し：`crates/kivact-core/src/platform/app_handle.rs:AppLink`。xtask の state 規則（`state` と `requested` の 2 field）で error。付属書は「AtomicBool 一つなので印は要らない」と書くが、xtask は interior-mutable な field が 2 つで、印か enum を要求する。設計者が、印を付ける（付属書の文面を直す）か、`requested` を state に含める形にするかを決める。
  Orchestrator の処置：C03 の commit（`efc5a622`）は lane C に残し、統合 branch へ merge していない。C04 は上の 4 件の判断後。

## 設計者の回答（W3a・C03、2026-11-19）

付属書 C の C03（手順 9・10・11、所有、目的）と C04（所有、`bind_states` の呼出し元）、作業指示書 4 章の W3a の lane C の行を直した。設計者が `efc5a622` に下の 4 件を当てて、`cargo clippy -p kivact-core --all-targets`（feature 無しと `gpu-interop,devtools,embed`）、`cargo clippy -p kivact-runtime --all-targets --all-features`、`cargo xtask check --no-write` の kivact-core と kivact-runtime、`cargo test -p kivact-runtime`、`cargo test -p kivact-core`、`e2e_core` の `flows:: focus app_handle windows` が通るのを確かめた。

1. **current_window**：fallback は残さない（付属書どおり `expect`）。原因は runtime：`dispose_owner` が cleanup の前の `detach_owner(root)` で root の `parent` を `None` にするので、dispose される部分木の cleanup から祖先の context が見えない。`detach_owner` は親の `children` から外すだけにし、`parent` の link は owner が消えるまで残す（`crates/kivact-runtime/src/reactive/owner.rs`、C03 の所有に足した。W3a で runtime の `reactive/` を持つ lane は他に無い）。
2. **report_session**：`FirstWrite` に `report_session: bool` を足し、`finish_resolve` が `handlers.selection.is_some() || draw.with_layout.is_some()` で決める。`create::write` は自分で計算しない。偽 handler は消す。
3. **n.at**：`create::write` の 1 か所で `n.at = site` と一度だけ書く。`mount_host` の最後の引数を `site: Option<&'static Location<'static>>` にし、本体は private の `mount_node`。`mount_host_with` は署名を変えない（`virtual_list.rs` は lane R の R10 が持つ）。builders は `mount_host(cx, kind, common, at)` だけになり、`n.at = None` の書き直しが消える。`overlays.rs` は `Some(at)` を渡す。`mount.rs`・`mount/create.rs`・`overlays.rs`（1 行）を C03 の所有に足した（同じ lane の次の C04 も `mount.rs` の `bind_states` の 1 行を持つ。順番なので衝突しない）。
4. **AppLink**：`requested` を足す小項目は取り下げる（印も enum も足さない）。`requested` は `CloseRequested` を写した 2 つ目の真実で、state 規則が防ぐ形そのもの。競合の無い lock は turn ごとに数十 ns。`app_handle.rs` は C03 の前（`efc5a622~1`）の内容に戻す。

- Orchestrator：lane C の fix として、`wt/rw-W3a-C` に付属書 C03 の新しい手順 9（runtime の `detach_owner` と `current_window` の `expect`）・10（`report_session` と `n.at`）・11（`app_handle.rs` を戻す）を当て、C03 の確認を全部通してから C04 へ進む。確認の e2e には `flows::` を足して走らせる（`e2e_core -- flows:: focus app_handle windows`）。

## 差戻し（W3a・R10 と S12、2026-11-19）
- **R10**（W3a、lane R、commit `062ba24b`）：付属書 R10 の「状態の閉じ方」の印（`// xtask-allow(state): seek and held are the list's two phases; ...`、`VState` の直前）は、xtask が「未使用の印」の error を出した（`VState` に counted の状態が無い）。Orchestrator の処置：`VState` の印を外した。`Seeker` の直前の既存の `legacy:` 印は残した。設計者が、付属書の印を外すことを認めるか、規則を直すかを決める（R03・K02・S04 と同じ型）。
- **S12**（W3a、lane S、commit `baa76a50`）：付属書 S12 の「Draw の印」（`xtask-allow(state)`）は xtask の規則と合わない。印の変更後、`Draw` の counted field は `unsupported: u32` の 1 つだけになり、state の対象外（未使用の印は error）。Orchestrator の処置：印は足していない（xtask は ok）。設計者が、印を外すか、付属書の印を直すかを決める。
- **S12**（残し）：`measure_text`・`text_path`・`layout_paragraph` の doc に「engine が無いとき 0・空」の文が残る（engine は常に在るので今は当たらない）。S23 か docs の後始末で直す候補。

## 差戻し（W3a・C04、2026-11-19）
- **C04**（W3a、lane C）：STOP。`crates/kivact-core/src/view/root.rs:install_app_contexts` で、固定の theme を `owner.constant` にすると、Paint の why が Theme の signal を読まなくなり、`e2e_core` の `diagnostics::why_follows_the_prop_to_the_write_through_a_memo`（`dev/tests/kivact-public-api/tests/e2e_core/diagnostics.rs:114`）が失敗する（Paint の reads が memo `Merged` だけになる）。同じ試験を `diagnostics` の filter で実行し、1 件だけ失敗（残りは成功）を確認した。試験の期待値は変えていない。選択肢：(a) 固定 theme も signal にする（付属書の手順 2 の取り消し）、(b) 試験の期待値を変える（「button の style は theme の signal を読む」を別の形に直す）、(c) 試験の対象を別の node にする。作業の差分は worktree に未 commit で残っている（commit していない）。
- **C04**（付属の所有外の変更）：`crates/kivact-core/src/view/mount/semantics.rs` の `semantic_origins` の書込み（bind の 1 行）を `set_semantic_origins` の呼出しに置き換えた（`CoreRare.semantic_origins` を消した結果の同一 crate の機械的な追従）。設計者が許容しないなら戻す。

## 設計者の回答（W3a・C04、2026-11-19）

- **C04**：(b) 試験の期待値を直す（固定の theme は constant のまま。付属書の手順 2 は取り消さない）。`diagnostics::why_follows_the_prop_to_the_write_through_a_memo` の最後の assert が見ていた「button の Paint が theme の signal を読む」は、C04 の目的（固定の theme は constant）が消す実装の細部で、利用者に見える振る舞いではない。`dev/tests/kivact-public-api/tests/e2e_core/diagnostics.rs` の該当の 7 行を次にする（`dev/tests/kivact-public-api/tests/e2e_core/diagnostics.rs` を C04 の所有に足す）：
  ```rust
      // A button's style is a render effect reading the memo of its merged
      // style layers; the App's fixed theme is a constant, not a read.
      let other = screen.get_by_test_id("other").why();
      let paint = other.props.iter().find(|p| p.prop == "Paint");
      assert!(
          paint.is_some_and(|p| p.reads.iter().any(|d| d.kind == "memo")
              && !p.reads.iter().any(|d| d.ty.ends_with("Theme"))),
          "{other}"
      );
  ```
  設計者が lane C の未 commit の C04 の差分（`b8d666b0` の上）にこの変更を当てて、`e2e_core -- diagnostics::` が全件通るのを確かめた。
- **C04**（所有外の `view/mount/semantics.rs` の 1 行）：受け入れる（同じ crate の機械的な追従。作業指示書 0.2 の 11）。
- Orchestrator：lane C で上の試験の変更を当て、C04 の確認を全部通して commit する。

## 作業指示書 0.5 節の表に足した行で決着するもの（2026-11-19）

作業指示書 0.5 節の判断表に 8 行を足した（状態の印を xtask が「未使用」と言う、完了条件の rg が既存の行や検出用の文字列に当たる、Role・doc の文の長さ、`module_rules` の 1 参照、付属書の script の機械的な欠陥、名前だけ違う method・field、目的が消す実装の細部を見る試験、別 target だけの lock の二重）。表に当たるものは差戻しではなく「Orchestrator の判断」節に 1 行で書き、作業を続ける。これまでの次の差戻しは、Orchestrator の処置のまま受け入れ、決着とする（付属書の文は直さない。付属書の印の文は無効とみなす）：

- 状態の印：R03（`EffectNode`）、K02（`MobileContext`）、S04（`StateStyles`）、R10（`VState`）、S12（`Draw`）。
- 完了条件の rg：T22、R07（`animated.rs` の 2 件）。R07 の `set_run` の逸脱も受け入れる。
- Role の文の長さ（W2a の types）と W2b の layout。`module_rules` の `scene` → `shaped`。
- S06 の `rename_protocol.pl` の欠陥と手順 2 の別名の衝突（subagent の直し方を受け入れる）。
- S11 の `tabular_nums` の名前の差。

設計者の判断がまだ要るもの（lane は止めない）：R03 のリスク（`pass_runs` が pass の途中の panic で戻らない。workspace は panic を unwind にしている）、K03（`NodeMode::Connecting` の handler の場所）、S11 の確認事項（文字の gradient の px の長さ）。

## Orchestrator の判断
- W5b・A10（lane X、commit `f1d343db`）：確認（fmt・check・clippy workspace・xtask の全 crate・報告の範囲の E2E・5 件の後退）は成功（kivact-ui-soft の既知の 1 件のみ）。完了条件の rg：`arvane-compose/src/cpu/blend.rs` の local の `unpremultiply`（blend 計算の内部。手順にも消すものにも無い）は既存の行として受け入れる（表の行）。`chunks_exact_to_as_chunks` は clippy の提案どおり。
- 設計者に確認してほしい（A10 の報告。作業は続けた）：(a) `arvane/tests/e2e/support.rs` の `color_alpha` は straight の `Rgba` を返すので、doc は「A premultiplied color」ではなく「A straight-alpha color」のまま残した（4.7 節の表の doc と食い違う）。(b) `kivact-devtools-agent/src/image.rs`：4.7 節は `kivact_types::color::unpremultiply` を呼ぶと書くが、この crate は kivact-types に依存しない。private の `to_straight` を置いた。依存を足すか、この形のままかを決める。(c) gpu-share の wgpu・vulkan・d3d12 の doc は straight の文が無いので変更なし。(d) A06 の残り（root `Cargo.toml` の profile・`templates/basic/README.md`）は A10 の手順・4.7 節に無いので未了のまま。(e) 付属書 G の範囲（4.7 節の外）：user 向け docs が `Pixels` を straight-alpha と書いたまま（`docs/for-kivact-user/en/how-to-guide/{canvas.md:473,567,668,drawables.md:111,137,gpu-texture-sharing.md:287,295}`、`jp/how-to-guide/canvas.md:561`、`docs/for-kivact-ui-user/{en,jp}/creative/how-to-guide/show-app-rendered-images.md`）。G の unit で直す。(f) この記録は Orchestrator の判断として `rework/OPEN.md` に入れた。

- W5b・A09（lane X、commit `4c6db854`）：確認（fmt・xtask arvane-compose・workspace clippy・報告の範囲の E2E・5 件の後退）は成功（kivact-ui-soft の既知の 1 件のみ）。(a) `chunk()` の early path を helper `keep_chunk()` に分けた（関数の上限。0.4 の扱い）。(b) `#[cfg(debug_assertions)]` を `if cfg!(debug_assertions)` にした（release で未使用の引数の警告を避ける）。(c) 手順 5 のコード片を、現在の chunk の形（previous・redraw の分岐）に合わせた。同じ条件・同じ動作。以上は受け入れる。

- W5b・A08（lane X、commit `c53801b2`）：確認（fmt・xtask arvane-shell、報告の範囲の E2E・5 件の後退）は成功。(a) 付属書の `kivact-engine/src/host/window.rs` は `src/model/window.rs`（E15 で移動済み）。行番号のずれとして受け入れる（0.5 節）。(b) win32 の `State` の印の文（`surface` を挙げたまま）は、付属書が文面を指定していないので変えない（CP の台帳と付属書 L で閉じる）。(c) `Dropped(_) | NotDrawable(_)` の or-pattern、Shown の GPU 経路の `LastPresent::Settled`（AR-09）は付属書どおり。受け入れる。
- 設計者への注意（A06 の所有の残り）：A06 の所有には root `Cargo.toml` の profile 1 行と `templates/basic/README.md` の profile 1 行があるが、A06 の付属書の手順には無い（A06 は直さず報告した）。`templates/basic/README.md` の profile の文は G12（W11b）の手順 3 で直す。root の `[profile.dev.package.*]`（124〜132 行付近）は、どの手順にも無いので、設計者が A の手順に足すか、G 側で扱うかを決める。

- W5b・A07（lane X、commit `12cd5129`）：(a) `MakeSurface` の署名を `&dyn Fn(&GpuContext, &mut dyn ShellWindow)` にした。付属書の make closure は window（`&mut dyn ShellWindow`）を捕まえるので `Fn` にできず（コンパイル error）、window を引数で渡す。surface の作成は UI thread で `window.surface().with_gpu_target` を通す。付属書の意図どおり。機械的に壊れる式の行で受け入れる（設計者は形を確かめる）。(b) `frame_period_ns`・`reduced_motion` の impl は inherent にせず削除した（inherent だと dead code で clippy -D warnings が落ちる）。呼出し元は `environment()` と `Presenter::frame_period_ns` を読む。受け入れる。(c) `set_ime(None)` は仮想 window の `ime_text` を消さない（付属書の None の枝どおり。観測上の差は無い）。受け入れる。
- 作業の残り（A06・A07 の残り）：root `Cargo.toml` の `[profile.dev.package.*]`（124〜132 行付近）と `templates/basic/README.md` の profile の文は、どの unit の手順にも入っていない。W5b の所有には入っているので、付属書の手順の中（A10 か、その付近）で直すかを確かめる。

- W5a・R29（lane X、commit `d878f338`）：(a) `async_view` の `default_fallback` は既存の `i18n::message("async_view.retry", "Retry")` のまま（doc と i18n を保つ。同じ意味）。(b) stepper の `Prop<String>` を `text(..)` に渡す箇所は closure（`Prop` の get を使う。`Prop<String>` に IntoText が無いのは R24 のとおり）。(c) `button_group`・`control_size` の `size.get().0` は `size.0`（同じ挙動。context の get は R20 で無くなった）。以上は同じ意味の一つだけの method を使う行で受け入れる。機械的な掃引（r_sweep・r_handler_sig・r_handler_body、F0〜F2、clippy --fix）も受け入れる。

## 差戻し（W5a・R29、2026-11-20）
（設計者へ返す STOP。unit、場所、STOP の文、付属書の節）

- **R29**（W5a、lane X、commit `d878f338`）：`crates/kivact-ui/src/data_table/tree.rs` の `Plan`（`pub(super) struct Plan<K>`、field `show_all`・`expanded`）が xtask の state の error。付属書の「状態の閉じ方」の印の文（`// xtask-allow(state): independent: expanded is the caller's binding of the open rows, written by the tree toggles; show_all is the tree column's setting, fixed when the plan is made`）を `Plan` の直上に入れる Edit が、権限の自動判定で拒否された。subagent は別の手段で書かなかった。Orchestrator も、拒否された編集を別の方法で入れない（E31 の印と同じ扱い）。xtask が通らないので、この差戻しの間、lane X（鎖）は止める。設計者（またはユーザー）が、この 1 行を入れるか、編集の許可を出すかを決める。

- W5a・R28（lane X）：設計者の回答（`9b9e001c`）に従い、付属書 R28 の「状態の閉じ方」の印を、そのままの文で lane X に入れた（`range.rs` の effect-writes 3 件、`DisclosureApi`・`Target`・`Core` の state の印を doc と属性の塊の上に）。設計者の回答では印が `49707eeb` に入っているとあったが、その commit はコードを変えていなかったので、lane X で入れた。確認（fmt・check・clippy --lib・xtask）は kivact-interaction で成功。

- W5a・R27（lane X、commit `66965e52`）：確認（fmt・clippy --lib・xtask kivact-core）は成功。付属書どおりの policy の except 削除、`__private` と `frame_loop` の削除を受け入れる。(a) `HostEvent` の `StatesChanged`・`NodeFlagsChanged` の腕を cause_of に足した（E の variant の規則どおり）。(b) `launch/turn.rs` の `mounts_only` の helper は too_many_lines への対応（表の「関数の上限」）。(c) `prelude.rs` に `Trigger`・`trigger` は足していない（付属書 R27 の手順は constant_value の削除だけ）。設計書 5.11 との違いの確認は設計者に任せる。
- 設計者への注意（R27、STOP ではない）：`kivact-interaction`（`scalar/range.rs`）が `state::__private::Prop` を使っている。R28 の所有で直す。R28 までは interaction が compile できない（予期された途中の状態）。

- W5a・R26（lane X、commit `348aacee`）：確認（fmt・check -p kivact-macros）は成功。残りの rg の 2 件は受け入れる：(a) `crates/kivact/src/lib.rs:5` の doc の語 `Binding`（R26 の手順に無く、この file は R27 の所有。既存の行に当たる、表の行）。(b) `error_boundary.rs:49` の `"Retry"` の文字列（許される例外）。kivact-core 自身の check は、kivact-style（R27 の範囲）が先に失敗するので、R27 の後に確かめる。

- W5a・R25（lane X、commit `013e0cc4` と後続の commit）：`Diag` は付属書に印の文が無く、xtask が state error にした。0.5 節の新しい行に従い、Orchestrator が `independent:` の印を書いた（`reactive/data.rs`、`Diag` の doc の直前）。三つの条件を確かめた：(1) 書く関数を挙げられる（`cause` は `cause()`・`record_write`・`record_run` が読み、turn の境界で消す。`open_stage` は `stage()` が書き、`end_turn` が取る。`next_seq` は `record_error`・`record_diagnostic`）。(2) どの field も他の field の値で読み書きされない。(3) 同じ事実を二つの field が持っていない（`cause` は現在の turn の cause の列への index、`open_stage` は未確定の stage）。印は doc comment の上に置く（xtask は doc と derive を項目の始まりとして数える）。

- W5a・R24（lane X、commit `9176ce98`）：完了条件の rg の残り 4 件は、すべて `//!` の英語の説明文（「Reactive runtime」など）で、API の名前ではない。手順のコードに無い既存の行で、0.5 節の「既存の行に当たる」の行で受け入れる。

- W5a・R22（lane X、commit `629fb246`）：完了条件の rg 0 件、共通の形（fmt・check・clippy・xtask）は成功。(a) `ActionCell::enqueue` の pending の書込みを `publish`（lifecycle の guard）経由にした（同じ効果。dispose の後に publish しない ST-07 を守る）。(b) `on_complete` の `let (_, cell)`（同じ挙動）。(c) `xtask-allow(state)` の印が未使用と言われた `AsyncCellState` の印は外した（表の「未使用の印」の行）。(d) `state.rs` の `TaskState` の印の文は `draining`（削除済みの field）を挙げたままにした。文は R26 以降の掃引か L の閉じ方で直す。以上は受け入れる。
- 設計者への注意（R22、STOP ではない）：(i) `kivact-core`（`task.rs`・`error_boundary.rs`・`launch/turn.rs:105` の `check_draining`）と `kivact-ui` の `async_view.rs`、devtools が壊れる。付属書どおり R26 以降で直す。(ii) 破棄済みの owner に対する action の run を、旧「閉じた action の run は無視」の検査を消したあとどう扱うか、`Owned::get_at` が破棄済み owner を既に弾くかを確かめていない。設計者が確かめる。

- W5a・R21（lane X、commit `26b7c350`）：完了条件（rg の 0 件、確認の共通の形、xtask の api_shape の 6 件）は満たす。`dev/xtask/src/api_shape.rs` のテスト文字列を新しい形（`IntoHandler`）に直したのは、この unit の一部として受け入れる（期待する error の数は変えていない）。

- W5a・R20（lane X、commit `ba0406f7`）：(a) `reactive/keyed.rs` を、所有の一覧に無いが同じ crate 内の署名の変更に伴う機械的な追従として直した（`Row::new` の `item.key` を let-else に）。0.2 の 11 の範囲で受け入れる。(b) `binding` の closure の型注釈、`Row::new` の let-else は、付属書のコード片を実コードに合わせた形（同じ意味）。受け入れる。(c) 確認は R20〜R25 の共通の形（fmt・check --lib・clippy --lib・xtask）。完了条件の rg は 0 件、`update` は両方とも戻り値なし。

- W5a・R09（lane X、commit `c6e8979e`）：(a) `RNode` の `xtask-allow(state)` の印は、xtask が「未使用の印」の error にしたので外した（表の「未使用の印」の行）。(b) rg の残り `NodeInfo.written` は、付属書が「`NodeInfo` は変えない」と書く公開の field なので受け入れる（表の「既存の行に当たる」の行）。(c) `EffectNode` の R03 の印の置換は、基点に R03 の印が無いので何もしなかった（xtask は指摘しない）。
- 設計者への注意（R09、STOP ではない）：`frame_path`（cycle の path）が `node_meta` を読むようになり、devtools の feature が無い build、または記録中でない build では、Cycle の `path` が空になる。付属書の R09 に書かれておらず、試験は path を見ていない（`node_ref.rs` は kind だけ）。エラーの内容が変わるので、設計者が影響を見て決める。

- W5a・R08（lane X、commit `686e8409`）：(a) `Diag` に印の文面の 1 文を足す手順は、`Diag` に `xtask-allow(state)` の行が無いので、足さなかった（xtask は指摘しない。表の「印の文の編集」の行に準じて、無いものは足さない）。(b) `set_recorder`・`is_recording`・`RecordBits` は付属書 4.1 の名前どおり pub（`#[doc(hidden)]` は付けない）。受け入れる。
- 設計者への注意（R08、STOP ではない）：`crates/kivact-core/src/devtools/turn.rs:41` の `turn.cause_with(CauseTag::Devtools, format!(..))` は付属書の一覧に無いので触っていない。記録は `record_cause_with` のゲートで残らないが、`format!` は毎回走る。設計者が一覧に足すかを決める。

- W5a・R06（lane X、commit `13942301`）：(a) `count_round` を `pub(crate)` にした（private のままだと `reactive/host.rs` の `run_deferred` から呼べない。可視性だけの変更）。(b) `launch/turn.rs` の `ViewState::of(&rt).initial_render.set(false);` を書かなかった。`start_mount` がすでに false にしていて、どの turn より前なので効果が無い。表の「付属書のコード片が効果を持たない」の判断（同じ意味で既に済んでいる）。(c) `use` は現行の import に合わせた。以上は表の範囲で受け入れる。
- 設計者への注意（R06、STOP ではない）：`commit_rounds` の `if !mounts && !rt.has_work() break` は、予算切れで残った deferred があると `has_work` が真のままで、100 回回ってから「user effects did not settle within 100 rounds」の Cycle を余分に一つ報告する（メッセージも不正確）。付属書の形のまま書いた。直すかどうかは設計者が決める。試験は全エラーが Cycle であることしか見ていないので通る。

- W5a・C04 の memo patch（設計者の patch `C04-memo.diff`、lane X の fix commit）：`view/mount/{node,style}.rs` と `e2e_core/diagnostics.rs` に当てた。`diagnostics.rs` の試験の期待値は、memo の読みを「無い」に変えた（`why` の trace の内部の細部。0.5 節の「`why`・trace の読みの期待値を unit の後の事実に直す」の行）。確認（tier a）は fmt・check・clippy（devtools）・xtask・e2e_core の `diagnostics flows setup_functions i18n style_paint` で、すべて成功（38 件）。
- W5a・E1・E2 の merge（設計者の回答どおり）：E13 の確認（tier a の全部。ENGINE-A と e2e_core 17 module・e2e_ui 4 module）を実行し、全部成功（e2e_core 153 件、e2e_ui 33 件）。その後 `a5944729`（E1）と `1d6538f6`（E2）を merge した。

- W5a・E13（lane E2、commit `3694e0e1`）：完了条件の rg（`restack = true`）の残り行 `model/layout_motion/steps.rs:141`（run 開始）は、付属書 E13 の「目的」にも「消すもの」にも挙がっていない既存の行なので受け入れる（表の「完了条件の rg が既存の行に当たる」の行）。

- W5a・E12（lane E2、commit `c5e568ee`）：(a) `SceneOut` の `xtask-allow(state)` の印を外した（xtask が「未使用の印」の error にした。表の「印を外す」の行）。(b) `outdate_paint` の引数が clippy の too_many_arguments に当たったので、draw の無効化を `PaintState::outdate_draws`（別関数）に分けた（表の「引数を struct にまとめる・helper に分ける」。STOP にしない）。(c) `shape::clip_path` は無いので既存の `s.in_rect(bounds)` を使った、`Paint` は `arvane_protocol::RasterPaint`、`Look::now` は `LayerTraits::now`（同じ意味のものが一つだけある。表の行）。(d) 付属書に無い追加：`model/frame/ticks.rs` で、tick の先頭で動いている text color の node の子の draw を忘れる（`VERIFY_DRAWS` の stale 検出を直した。E2E の `motion` が一回だけ失敗し、この修正で全 32 module が通った）。E13 が `state_motion` の steps に触るとき、この分岐との関係を確かめる。

- W5a・R05（lane X、commit `683b4d4d`）：(a) `ViewState::new` の引数に `hooks`（`devtools.hooks()`）を入れ、devtools を先に作る順序は変えない。付属書 R の R05 の build closure の `ViewState::new(root, info, writer.clone(), at)` は、C の後の形（`hooks` を足した形）と違う。表の「同じ意味のものが一つだけある → それを使う」の行。(b) `AppState.gpu` は付属書の eager の `gpu_interop::State::new` にせず、base の `StateSlot(Deferred<State>)`（遅延）を使う。C-core の RC-41 の文（launch で常に作るのをやめる）と base の形に合う。表の「同じ意味のものが一つだけある」の行。

- CP1（2.4 節）の `cargo test -p kivact-test --test gpu` は、その binary が無いので実行できなかった。GPU の試験は `kivact-test` の `tests/screen/main.rs` の `gpu` module にある。`cargo test -p kivact-test --test screen -- gpu::` に代えて実行し、3 件とも成功した（`KIVACT_TEST_GPU=1 KIVACT_TEST_MIRROR=required`、`--test-threads=1`）。作業指示書の該当行は直していない。

（作業指示書 0.5 節の表で決めたものを 1 行ずつ書く。unit、場所、表のどの行か、commit。）

## 設計者の回答（R03 のリスク・K03・S11、2026-11-19）

- **R03 のリスク**（`pass_runs` が pass の途中の panic で戻らない）：受け入れる。直さない。panic を捕まえるのは arvane-native の 2 か所（`port/win32/proc.rs`、`port/x11/thread.rs`）だけで、捕まえた後は App のコードを二度と走らせない（`keep_panic` で子を止め、Kivact は再開しない）。standalone の run は panic で終わる。次の `stabilize` の pass は来ないので、数が残っても害が無い。
- **K03**（`NodeMode::Connecting` の handler の場所）：その 1 項目は取り下げる（付属書 K03 の手順 4 の文を直した）。接続を確定するのは `editors/node/edit.rs` の `invoke` で、`connect` → `write_links` の `valid_link` が消えた出力からの接続を拒み、`self.mode.cancel()` が mode を終える。設計者が試験 `a_keyboard_connection_from_a_removed_output_does_not_connect`（出力の node を消してから入力を Activate しても link ができない）を書き、修正なしで通ることを確かめた。
  - Orchestrator：K の lane の fix として、この試験を `crates/kivact-ui-creative/tests/consumer/node_editor.rs` の `a_keyboard_connection_ends_when_the_editor_loses_focus` の前に足す（W3a・W3b に `kivact-ui-creative` を持つ lane は無い）。本体：`a_keyboard_connection_ends_when_the_editor_loses_focus` と同じ 2 つの node で、`render_with_handle` は `((data, links), app_node_editor(data, links, ui::NodeState::new()).style(style().height(300)).test_id("nodes"))` を返す。`nodes` に focus → `"Image output Image"` を `A11yAction::Activate` → `data.update(|nodes| nodes.retain(|node| node.id != 1))` → `"Mix input Image"` を Activate → `links.get_untracked().is_empty()`（`"the pending connection ended with its output"`）と `screen.errors().is_empty()` を assert。確認は `cargo test -p kivact-ui-creative`（window-lock を通す）。
- **S11**（文字の gradient の px の長さ）：設計を直す。`objectBoundingBox` の単位正方形は px の stop・半径と canvas の座標の gradient を壊す。window の描画（engine の `paint_run`）は文字の gradient を run の原点と shaping した大きさの箱で塗るので、SVG も同じ箱を user space で使う。`Resources::baseline` を `Resources::metrics(&self, run) -> TextMetrics`（大きさと baseline）にし、offscreen の実装は `self.measurer.measure(run)`。`text.rs` は `Rect { run.origin, metrics.size }` を `paint_attributes` に渡す。`fill.rs` の `Units` と `text.rs` の `UNIT_BOX` は消す（gradient は今までどおり `userSpaceOnUse`）。付属書 S の S11 の所有・手順 3・状態の閉じ方・試験を直した。設計者が `baa76a50`（lane S の S12 の後）にこの変更を当てて、`cargo clippy -p kivact-svg`・`-p kivact-offscreen`（`--all-targets -- -D warnings`）、`cargo xtask check --no-write` の 2 crate、`cargo test -p kivact-svg`、`e2e_core -- offscreen_svg` が通るのを確かめた。
  - Orchestrator：lane S の fix として、`wt/rw-W3a-S` に付属書 S11 の新しい手順 3 を当てる（`fill.rs` の `Units` の enum・引数・`units.name()` を消して `gradientUnits="userSpaceOnUse"` の直書きに戻す、`emit.rs` の `Units::UserSpace` の引数を消す、`resources.rs`・offscreen・`text.rs`・`document.rs` の試験の `Resources` と assert を直す）。S11 の確認を通してから lane S を閉じる。
- **U04**（W3a、lane U、commit `ffbb3f40`）：付属書 U04 の手順どおりに入れたが、table の `State` の `active` field を消した（機械的な逸脱）。消した後は誰も読まず、dead_code の警告が `-D warnings` で落ちるため。その役割は `active_row`・`active_cell`（`selector(..)`）が持つ。Orchestrator の処置：そのまま受け入れる（設計の判断ではない）。
- **U07**（W3a、lane U、commit `9e70dcc2`）：完了条件の rg（`Cell<Binding` を含む）が `combobox/handle.rs:129` の `OptionsCell<Binding<String>>` に誤検出で当たる。Orchestrator の処置：完了条件の rg は crate 名と意味で見る（誤検出として扱う）。list の `State` の印（`legacy: item_states, container_state, typed`）は `typed` が消えたため xtask が未使用の印として error を出したので、印を外した（0.5 の表の決着どおり）。`item_states`・`container_state` の legacy は CP1 の台帳で扱う。use_select の関数を `retained_focus()`・`trigger_and_list()` に分けたのは clippy の too_many_lines 対策（動作は変えていない）。
- **W3a の tier b の fix（C）**（`d73cc130`）：xtask の non-leaf と循環を、`view/mount.rs` の本体を `view/mount/node.rs` へ移すことと、`ViewState.gpu_interop`（gpu_interop の型）を `view::state` から外し、`gpu_interop::import::registry` に `StateSlot`（`Deferred<State>` を持つ拡張）を置くことで解いた（policy は足していない。付属の `[[style.exceptions]]` の 2 件は、対象の path が無くなったので消した）。Orchestrator の処置：そのまま受け入れる（構造の直しで、振る舞いは同じ）。

## 確認事項（W3b・E02、2026-11-19）
- **E02**（W3b、lane E1、commit `7317a565`）：振る舞いに関する注記（E2E は通っている）。(1) `SizeMotion`（entrance・exit の size）は、これまで spring を start していなかった。`MotionRun::start` は `spring.start` を呼ぶので、spring の transition の size motion は初期化されるようになった（tween の size motion は変わらない）。(2) `MotionRun::step` は reduced かつ decorative のとき `current` を `to` に写す。元の size・layout の step は写さなかった。Orchestrator の処置：受け入れる。設計者が (1)(2) の意図を確かめるならここを見る。`layout_motion/state.rs` の Role 行は 87 字だが、xtask は通った。
- **E04**（W3b、lane E2、commit `913556e1`）：付記（判断は設計者に任せる。修正はしていない）。木の構造の編集（insert・detach）は layout の前に `forget_origins` を通らない。次の `layout()` の先頭で消えるので、編集と layout の間に `rect` を呼ぶと、付属書どおりの cache が編集前の origin を返し得る。付属書の指定（layout の先頭と末尾、scroll、sticky、viewport、geometry の変化）どおりに実装した。設計者が、木の編集の時点で `forget_origins` を足すかを決める。

## 差戻し（W4・U09、2026-11-19）
- **U09**（W4、lane U、commit `08e26d3a`）：STOP（E2E の失敗）。付属書どおり `crates/kivact-ui/src/carousel.rs` の `follow_scroll` を `effect_on` にすると、scroll の反映が約 150ms 遅れて callback が走り、`ui_carousel::carousel_scroll_mode_snaps_to_a_slide`（`ui_carousel.rs:41`）が index 0 のまま失敗する。HEAD に戻すと同じ試験は通る（変更が原因）。debug の出力では、at=1 の初回が now=150ms で、Settle の 120ms が試験の advance の 200ms に入らない。Orchestrator の処置：この commit は lane U に残し、統合へは入れない。lane U は U10 以降を止めた。設計者が決める：(a) `follow_scroll` は `effect_on` のまま、試験の時間の進め方（advance）を直す（期待値は変えない）、(b) `follow_scroll` を effect にせず、同期的な反映に変える（付属書の手順を変える）。
- **U09**：設計者の回答（(a)）：`follow_scroll` は `effect_on` のまま、`ui_carousel::carousel_scroll_mode_snaps_to_a_slide` の試験の時間の進め方（advance）を、反映が届く長さまで増やす（期待値は変えない）。Orchestrator の処置：lane U で fix する。
- **E06**（W4、lane E1、commit `a9d35ba8`）：逸脱なし。`next_tab_stop_outside` は `scan_from` で root から走査する（呼出し元は tree から外す前に呼ぶので挙動は同じ）。受け入れる。
- **A03**（W4、lane A1、commit `4a9d3753`）：`xtask-allow(state)` の印は「未使用の印」の error になったので外した（決着済みの処置）。`*_pl` の field は実コードに無かったので新設した。受け入れる。
- **C05**（W4、lane C、commit `17fff897`）：`open_window_record` は rt を持たない 7 引数。`view.rs` の `mod window_view` は `pub(crate)`。`EnvFields.keyboard_inset` は `Insets`（付属書は f64）。`frame_period_ns` の memo と `with_window_mut` は使われないので消した。`WindowRecord`・`WindowSlot`・`ForeignHooks` の印は未使用で外した。`EnvState` の印は付属書に無いので、実装者が閉じ文を書いた。`foreign_window.rs` の `request_size` は `state.decl.min_size` を読む。受け入れる（機械的な対応）。
- **U09 の E2E（再）**：設計者の回答（(a)：試験の advance を直す）では通らなかった。advance を 200・400・1000ms にしても `ui_carousel::carousel_scroll_mode_snaps_to_a_slide`（`ui_carousel.rs:41`）は index 0 のまま。失敗の記録では、スクロールが `screen.get_by_text("One").scroll(180, 0)` で Text「One」（node 6）に届き、スライド列（Slide 2 / 3）は x=256 から動いていない。HEAD（U09 の前）では同じ試験が通るので、時間ではなく、スクロールの経路（テキストに送ったスクロールがカルーセルのスクロール領域に届かない、または follow_scroll の配線）が原因と見ている。Orchestrator の処置：lane U は止めた（commit なし。worktree は U09 の HEAD）。設計者に次を決めてもらう（下の質問）。
- **E08**（W4、lane E1、commit `6c5d7c63`）：付属書 E の確認行 `e2e_native -- devtools::` は、この試験が `#[cfg(feature = "devtools")]` で gate されているため `running 0 tests` になる（0 件は失敗）。実装者は `--features devtools` を足して 1 件を通した。試験の期待値は変えていない。Orchestrator の処置：受け入れる。設計者が付属書の確認行に feature を書くかを決める。
- **E09**（W4、lane E1、commit `551186f1`）：逸脱（報告のみ、Orchestrator の処置は受け入れ）。(1) `edit_lines.rs` の状態の印を外した（数える状態が Option の `coloring` 1 つだけになり、「未使用の印」の error。0.5 の表どおり）。(2) `edit_state.rs` の印の文から `damage` の語を削った（`damage` が Option でなくなったため）。(3) 手順 1 の `self.damage.clear()` は入れていない（書く箇所はすべて `damaged` を通るため記録は欠けない）。(4)(5) doc の語の直しのみ。E2E（text_editing・custom_text_field・input_platform・interaction_edit・ui_chat_code・ui_field）は通っている。

## 設計者の回答（W4・U09、2026-11-19）

- **U09**：質問の 1（carousel の経路を直す）。試験の入力・期待値・advance（200ms）は変えない。上の「U09：設計者の回答（(a)）」（advance を延ばす）は取り消す。
  - 原因：`NodeRef::rect()` は呼ばれたときに host へ layout の報告を頼む。元の `effect` は mount 直後の初回の実行で viewport と slide の `rect()` を呼んでいたので、scroll の前に layout が届いていた。`effect_on` は最初の値では走らないので、最初の scroll の callback の中で初めて `rect()` を呼び、その時点では `None` で何もせずに戻る。scroll offset はその後変わらないので、callback は二度と走らず、index は 0 のまま。
  - 直し方：`follow_scroll` は viewport と slide の rect の `Value` を setup で取り、callback ではそれを読む（`slide_rects` を足し、`nearest_slide` は rect の `Value` の slice を受ける）。付属書 U の U09 の手順 1 を直した。設計者が `08e26d3a` に当てて、`cargo clippy -p kivact-ui --all-targets -- -D warnings`、`cargo xtask check --no-write --crate kivact-ui`、`e2e_ui -- ui_carousel::`（2 件、advance 200ms のまま）が通るのを確かめた。`follow_scroll` だけを元の `effect` に戻すと通り、U09 のままだと失敗することも確かめた。
  - 作業指示書 0.3 節に 1 行足した：`effect` を `effect_on` に変えるときは、`NodeRef` の観察を頼む getter を callback の中で初めて呼ばない（setup で `Value` を取る）。
  - Orchestrator：lane U の fix として、`wt/rw-W4-U` の `carousel.rs` を付属書 U09 の新しい手順 1 にし（試験の advance を変えていたら 200ms に戻す）、U09 の確認を全部通して commit し、lane U を再開する。

## 差戻し（W4 の tier b、2026-11-19）
- **W4 の tier b（統合 branch、`b26716f3`）**：(1) xtask に 667 件の error。内訳：non-leaf 違反 627 件（kivact-ui の部品の `X.rs` が `mod look;` と本体を両方持つ）、同名の module と値の曖昧さ 48 件（`crate::avatar` と `crate::avatar::initials` など）、循環 数件（`alert` と `alert::look` など）、Role 行 5 件（`arvane-asset/src/svg/fonts.rs` の tests の Role 不足、`kivact-editing/src/host.rs` の Role が 80 字超、`kivact-ui/src/recipes.rs` の Role 不足）。(2) test：`resource_reclaim::closing_three_thousand_rows_returns_the_live_counts_to_the_baseline` が失敗（`UiError(Cycle …)`：`kivact-ui/src/button.rs:124` の effect が per-pass の上限を超え、stabilisation が収束しない）。W3b の tier b では通っていたので、W4 で入った。Orchestrator の処置：(1) の kivact-ui の構造は設計の判断が要るので、設計者に確認する。(2) は原因を切り分けてから直す。Role 行の 5 件は fix lane で直す。
- **resource_reclaim の失敗（W4）**：二分探索の結果、最初に失敗する commit は `17fff897`（C05、lane C）。その親 `a0994fe1`（W3b）は通る。button.rs は C05 で変わっていないので、button の `node()` の effect は巻き添え。原因は kivact-core の env・window の変更（`open_window_record` と `WindowDecl`、env の lazy memo、メイン root の作り方）と見られる。Orchestrator の処置：C05 の lane（`wt/rw-W4-C-fix`）で、C05 の意図を残したまま循環を解く fix をする。button.rs・試験は変えない。
- **resource_reclaim の原因（W4、C05 の fix の結果）**：循環ではない。`kivact-runtime/src/reactive/turn.rs` の `Runtime::stabilize` は、1 回の pass の上限を `CYCLE_LIMIT * (graph.len() + 1)` として pass の開始時に 1 度だけ計算する（CYCLE_LIMIT = 100）。C05 の lazy memo で開始時の node 数が減り、上限が 3000 になった。3000 行を 1 回の pass で mount すると 3001 の effect が走り（graph は 72031 まで伸びる）、上限を超える。doc の「App の大きさに応じて増える」は、開始時の大きさを使うので、実際の大きさとずれている。C05 の fix は STOP（コミットなし。worktree は b66d2549）。設計者の判断待ち（下の質問）。

## 設計者の回答（W4 の resource_reclaim と kivact-ui の構造、2026-11-19）

- **resource_reclaim（質問 1）**：1 を採る。`Runtime::stabilize` の上限を、loop の中でその時点の graph から数え直す（C05 はそのまま）。kivact-runtime に試験 2 本（`a_pass_that_mounts_three_thousand_effects_is_not_a_cycle`、`an_effect_that_reruns_itself_is_a_cycle`）を足す。書き込む effect は `EffectKind::Internal`。設計者の確認：開く pass は graph が 28 node（上限 2900 回）で始まり、pass の中で 72031 node まで増える。どの effect も 1 回しか走っていないのに、2901 回目で切られていた。循環ではなく、上限を pass の開始時に 1 度だけ決めているのが誤り。
  - 処置：lane `wt/rw-W4-R-fix`（worktree `W:/w/kv-wt/rw-W4-R-fix`、基点 `902fc181`）。確認は tier a の全部と、e2e_core の `resource_reclaim::`。1 本目は修正の前に失敗し、修正の後に通ることを確かめる。
- **kivact-ui の構造（質問 2）**：1 を採る。X.rs は mod と pub use だけにし、コードは葉（X/view.rs など）へ移す。look が使う型は look の葉に置く。依存は view → look → look_common・tokens の一方向。recipes.rs は look だけを読む。これで、あいまい 48 件と循環 6 件も同時に消える。付属書 U の U11〜U14 の節は、root にコードを残したまま mod look を足させる指示で、設計の誤り（この節の指示は使わない）。
  - 確認は `cargo xtask check --no-write`（workspace 全体）で kivact-ui の error が 0 件になること。`--crate` は style の規則を検査しないので使わない。
  - 処置：lane `wt/rw-W4-U-fix`（worktree `W:/w/kv-wt/rw-W4-U-fix`、基点 `902fc181`）。61 部品を 7 batch に分け、同じ worktree で順番に直す（batch ごとに commit）。
- **W4 の Role 行の修正**：lane `wt/rw-W4-fix`（`9eb3db5a`）を `902fc181` で統合 branch に merge した。

## 設計者の回答（W4 の統合後の xtask 54 行、2026-11-19）

- **質問 1（あいまい 25 箇所）と質問 3（kbd の循環）**：1 を採る。ただし原因は scope ではない。xtask-core の `resolve_path`（`crates/xtask-modules/src/symbols.rs`）が、後ろに segment が続く途中の segment でも値の名前空間の項目を残していた。`lib.rs` は `pub use avatar::{.., avatar}` のように、部品の module と同じ名前の関数を crate root に再公開している。そのため `crate::avatar::..` を通る path が、関数の module（`crate::avatar::view`）も指していた。`crate::kbd::labels::KeyNames::new` が `kbd::view` も指したのも、`labels` から `crate::kbd::look::..` 経由の偽の辺が出たのも、同じ原因。
  - 設計者が直した：xtask-core の branch `fix/kashiwade/value-path-prefix` の `afbfd83`（途中の segment は型の名前空間だけを残す。回帰試験 `a_function_named_like_a_module_is_not_a_path_prefix` は修正前に失敗し、修正後に通る。`cargo test -p xtask-modules` と clippy が通る）。統合 branch の `vendor/xtask-core` をこの commit に上げた。
  - xtask-core の push は利用者の OK 待ち。それまで kivact の統合 branch を push しない（submodule の commit が origin に無いため）。
  - 追記（2026-11-19）：同じ修正を xtask-core の最新 main（`2d230a7`）に当て直し、`94cf072` として origin の main に push した（利用者の OK あり）。
    - kivact は `94cf072` に上げられない。`2d230a7` で xtask-core の library API が変わり、kivact の `dev/xtask` が compile できない（15 件、`Command` の field、`TargetGraph`、`Policy::analysis` など）。
    - 統合 branch は、`1aeabaf` の上の `afbfd83` を指したまま。`afbfd83` は xtask-core の origin の branch `fix/kashiwade/value-path-prefix-1aeabaf` に push した（利用者の OK あり）。kivact の push の止めは解除。
    - 新しい xtask-core への移行は rework とは別の作業にする。
- **質問 2（大きい SCC）**：1 も 2 も採らない。`UiTokens` の形と公開の名前は変えない。部品の token の型を、look の葉から新しい葉 `X/tokens.rs` へ移す。また、look_common と radio_group にある 2 つの循環を直す。
  - token の型：23 個の部品の token の型と、その field の型を `X/tokens.rs` に移す。field の型は `ToastRowMotion`、`ButtonBusyMotion`、`DataTableRowStatusPaint`、`DataTableStickyPaint`、`DialogCenter`、`DisclosureMotion`、`ProgressMotion`、`SparklineFill`、`ToggleLabelReservation`。
    - `tokens.rs` は `X::tokens` の葉だけを読む。look は今までどおり theme から `UiTokens` を読む。
    - 依存は view → look → `crate::tokens` → `X::tokens` の一方向になる。`X::tokens` は kivact_core の型だけを読む。
  - look_common：`metric_height` は progress だけが使う（Role の「2 つ以上の部品」に当たらない）。progress の look の private な fn に移す。`ProgressState` は動かさない。付属書 U の U14 の表の誤り。
  - radio_group：循環 `layer`・`radio_group::look`・`radio_group::view`・`recipes` が 3 つ目にある。`radio_group/look.rs` が view にある `RadioGroupVariant` を読んでいた。`RadioGroupVariant`（と `impl Variant`）を look に移す。root は `pub use look::RadioGroupVariant;`。
  - 設計者の確認：`9929d520` に xtask-core の修正と上の 3 つを当てて、次を確かめた。
    - `cargo xtask check --no-write`（workspace 全体）が error 0 件で通る。
    - `cargo clippy -p kivact-ui --all-targets -- -D warnings` と `cargo test -p kivact-ui` が通る。
    - `cargo doc -p kivact-ui` の公開名の一覧が、`9929d520` と同じ。
  - patch は `W:/w/kivact/target/rework-designer/W4-U-fix2.diff`（`9929d520` に `git apply` できる。80 file）。
- **処置**：
  - lane U の fix（`wt/rw-W4-U-fix`）を統合 branch の今の HEAD に進めて、patch を当てる。
  - `cargo xtask check --no-write` と kivact-ui の clippy・test を通して commit する。
  - その後、予定どおり R・U の fix lane を統合して tier b を 1 回回す。push は xtask-core の push の後。

## 設計者の回答（W4b・C06 の質問 4〜6、E31 の質問 7、2026-11-19）

- **質問 4（`AgentSlot::Ready`）**：1。`Ready(Box<Agent>)` のままにする。agent は起動したときに一度だけ box にするので、費用は無い。付属書 C の C06 の手順 2・4 を `Box` の形に直した。`AgentStart` を `state.rs`、`promote` を `start.rs` の `impl Inspector` に置いた配置も受け入れ、付属書に書いた。
- **質問 5（`apply::node_overrides`・`same_location`）**：1。残す。`layers()` は client が node を選んだときにだけ呼ばれ、DevTools の層を client が送った文字列のまま見せる。解析済みの `Style` からは元の文字列を作れない。付属書の「消す」は誤りで、手順 8 と「消すもの」を直した。RC-10 の目的（style hook が node ごとに一覧を走査・解析しない）は満たしている。
- **質問 6（`WindowsChanged`）**：2。以前どおり `WindowInfo` 全体を比べる。protocol の doc は「A window opened, closed or changed its title」で、DevTools の app（`kivact-devtools` の `on_windows`）は通知で window の一覧を置き換えるので、id だけの比較では title・大きさ・focus が古いまま残る。付属書の誤り。
  - 直し方：`sent_windows` を `RefCell<SentWindows>`（`open: Vec<WindowId>`、`listed: Option<Vec<p::WindowInfo>>`）にする。閉じた window の id は購読が無くても毎 turn 比べ（剪定に使う）、購読があるときだけ `list_windows` の一覧を前回送ったものと比べて送る。付属書 C の C06 の手順 9 に型と形を書いた。field の名前は変わらないので、`SessionData` の印は変えない。
  - 設計者の確認：`fd8413d8` に当てて、`cargo clippy -p kivact-core -p kivact-devtools-agent -p kivact-test --all-targets --features kivact-core/devtools -- -D warnings` と `cargo xtask check --no-write --crate kivact-core` が通るのを確かめた。patch は `W:/w/kivact/target/rework-designer/C06-windows.diff`（`fd8413d8` に `git apply` できる）。
  - Orchestrator：lane C の fix として `wt/rw-W4b-C` に patch を当て、C06 の確認（tier a。e2e の `devtools` を含む）を通して commit し、merge する。
- **質問 7（E31 の印の文）**：1。印の文は変えない。`AppModel` の印は W1 の `legacy:` の印で、CP1 の状態の台帳に載り、付属書 L が閉じ方を書く。台帳の `AppModel` の行に `text_ahead`・`font_generation` も挙げること。付属書 E の E31 の手順 3 と「状態の閉じ方」を直した。E31 はこのまま merge してよい。
- 作業指示書 0.5 節の表に 4 行足した（clippy の構造の lint、module の循環を断つ配置、消すはずのものに別の経路の呼出しが残る、印の文の編集が権限の判定で拒否される）。同じ型の STOP は今後 Orchestrator が判断し、「Orchestrator の判断」節に書く。

## 差戻し（W5a・R05、2026-11-19）
（設計者へ返す STOP。unit、場所、STOP の文、付属書の節）

- **R05**（W5a、lane X、commit `683b4d4d`）：`crates/kivact-core/src/app_state.rs` の `AppState` の field `foreign: ForeignWindows`。`ForeignWindows` は C05（W4）で消えているので、付属書 R の R05 の AppState の field 一覧（`foreign: crate::view::foreign::ForeignWindows::default()`）は base に合わない。付属書の R05 の節は、消えた型の扱いを書いていない（表の「状態の unit の対象が、前の unit で消えている」の行に当たり、付属書に扱いが無いので差戻し）。lane の commit では field を作っていない（ext の読みは 8 か所で全部置換済み）。設計者は、この field を消す（付属書の一覧から外す）か、別の扱いにするかを決める。この差戻しの間、lane X（W5a の鎖）は止める。他の lane（E1・E2）は続ける。

## 差戻し（W5a・E13、2026-11-19）
（設計者へ返す STOP。unit、場所、STOP の文、付属書の節）

- **E13**（W5a、lane E2、commit `3694e0e1`）：E13 の最初の E2E で motion の 2 件が debug の self_check（保持した list と作り直した list の比較）で失敗し、subagent はそれを直した。0.4 節は self_check の不一致を STOP にし、直さないと定めている。直した内容は付属書に無い：(1) `model/interact/sync.rs` に `relook_moving` を足し、`model/state_motion/steps.rs` の transition の終了時に `motion_ended` を入れた（prop の transition は restyled に入らない）。(2) 動いている不透明度の layer の grouping を、不透明度 1.0 として数えるように `paint/list/walk.rs` の opacity 関数を変えた。(3) E12 が tick の先頭に足した text color の分岐（`model/frame/ticks.rs`）を消し、`model/state_motion/steps.rs` の `step_states` の step の後へ移した（付属書の E13 にはこの移動の記述が無い）。commit `3694e0e1` は lane E2 にある。設計者は、この 3 点を受け入れるかを決める。受け入れないなら、E13 を STOP のとおり直し直す。この差戻しの間、lane E2 は止める（merge しない）。

## 設計者の回答（W5a の R05・E13、CP1 の KPI、2026-11-19）

- **R05 の `AppState.foreign`**：(a) 外す。`ForeignWindows` は C05 で消え、window の記録に入った。付属書 R の R05 の `AppState`・手順 9・build closure を直した（`foreign` を消す。build closure は lane の形：DevTools を先に作り `ViewState::new(.., hooks, at)`、`gpu: StateSlot::default()`（遅延））。Orchestrator の判断（`hooks` を渡す、`gpu` の遅延）は受け入れ、付属書に書いた。lane X（鎖）を再開する。
- **E13 の付属書外の 3 点**：3 点とも受け入れる。付属書の手順 4 が tick の塊を消すと、動いている node の事実を毎 tick 作り直す処理も消えた。3 点はその穴を埋めるもので、E13 の目的（重なりの事実が変わったときだけ restack）のまま。付属書 E の E13 に「W5a で足したもの」として書いた。ただし subagent の報告だけでは merge しない：Orchestrator が lane E2 で E13 の確認（tier a の E2E の全部）を自分で実行し、通ってから merge する。今後、subagent が self_check の不一致を直していたら、同じように STOP として差戻しに書いてよい（今回の扱いは正しい）。
- **CP1 の KPI**：
  - **測定の環境**：設計者が基点 `c9518946` と CP1 `d615568c` の kv_heap を、いま同じ環境で測った。**どちらのコードでも** a11y の木（`AcceptedTree::new`、`NativeAccessibility::hand_over`）が作られた。B0（11-18）の report には無い。AccessKit は、UI Automation の client が window に問い合わせたときだけ木を作る。CP1 の測定中にはその client が動いていて、B0 のときには動いていなかった。a11y だけで row は約 11 KB/item、box は約 1.4 KB/item になる。つまり CP1 の K2 の 1.6〜1.8 倍の大半は環境の差で、K1・K4・K5 の時間の行にも同じ差が入っている可能性がある。CP1 の比は、B0 との比較としては使わない。
  - **同じ環境で比べた K2**（MiB、3000 items）：

    | mode | 基点 | CP1 | 修正後 |
    |---|---|---|---|
    | box | 16.2 | 18.5 | 16.0 |
    | text | 14.3 | 14.1 | 14.1 |
    | button | 77.3 | 94.2 | 77.0 |
    | row | 103.2 | 131.4 | 102.5 |

    残った後退は C04 の style の memo（全 node に merge 済みの `Style` と memo を持つ）だった。設計者が直した：patch `W:/w/kivact/target/rework-designer/C04-memo.diff`（3 file。`18460fae` と lane X の `683b4d4d` に `git apply` できる）。memo は disabled・selected が変わりうる node と DevTools の hook のある App だけに置く。設計者の確認：`cargo clippy -p kivact-core --all-targets --features devtools -- -D warnings`、`cargo xtask check --no-write --crate kivact-core`、`e2e_core` の全件（436 件。`KIVACT_TEST_MIRROR=off`）が通る。付属書 C の C04 に追記した。
  - **処置**：(1) patch を lane X（鎖）に R05 の次の fix の commit として当て、C04 の確認（tier a）を通す。(2) KPI の測り方を直した（作業指示書 5.1 節の 2・3、5.2 の 4、設計書 2 章）：checkpoint では、基点 `c9518946` のコードを同じ環境で続けて測り直し（`B0-CP<n>`）、それと比べる。両方の report の kv_heap の site に `arvane_shell::a11y::state::AcceptedTree` があるかを見て、片方だけなら測り直す。(3) CP1 は測り直す：W5a の merge（E1・E2・鎖の R05 と上の fix）の後で、`B0-CP1` と `CP1b` を続けて測り、`kpi-compare rework/kpi/B0-CP1.md rework/kpi/CP1b.md` の結果で「KPI の後退」を書き直す。それでも後退する行だけを設計者に返す（時間の行 K1・K4・K5 は、この測り直しの結果を見て決める）。
  - **改善の必達の未達**（K3 の continuous CPU、K5 pointer p95、K6 no-gpu）：CP1 では返さない。手段の unit（E12 の draw cache、E13 の restack、hit test の索引、recipe 表の分割、plot の crate 化）の多くは CP1 の後にあり、判定は最終 checkpoint の AC-2。作業指示書 5.1 節の 3 に「CP1・CP2 では記録だけ」と書いた。
- **Orchestrator の判断**（`ViewState::new` の `hooks`、`gpu` の遅延、`SceneOut` の印の除去、E13 の完了条件の残り行、E12 の `outdate_draws` の分割と代用）：すべて受け入れる。CP1 の `--test gpu` は `--test screen -- gpu::` が正しい。作業指示書 2 章の行を直した。
- **付属書 L**（`legacy:` の印 194 件）：設計者が書く。W8L の前に用意するので、待たずに W5a 以降を進めてよい（作業指示書の W8L の行のとおり、W8 の merge の時点で無ければ止める）。

## 設計者の回答（付属書 L、2026-11-19）

- 付属書 L（`rework/L-ledger.md`）を書いた。台帳の 194 件の内訳：
  - 書く 176 件
  - 変える 5 件（`TimelineCell` の時刻の二重の保存、`EditorNode` の `measure_lines` を font の登録で捨てない不具合、型の別名に隠れた setting 3 件）
  - 他 unit 8 件（A14・E17・C07・R22・S17・U54 が型を直す、または消す）
  - 外す 5 件（xtask 自身の `legacy:` の文字列）
- 配置：W8L は L1〜L6（crate ごと）。kit とその example の印は、W9〜W10 の K の unit が型を書き直すので、W10 の各 kit の lane の最後（L7〜L11）に置く。xtask の `legacy:` の仕組みは、最後に W11a の L12 が消す。作業指示書の W8L・W10・W11a の表と unit の一覧を直した。
- 台帳の後に型が移った、改名された、field が足された・消えた場合の扱いは、付属書 L の 2 章の共通手順 L に書いた（印の field の並びで台帳の行に対応させる。後の unit が足した節は残す。無い field の名前は消す。新しい状態の field の説明が無ければ差戻し）。
- 「台帳」節の項目（付属書 L を書くこと）は済んだので消した。

## 差戻し（W5a・R21、2026-11-20）
（設計者へ返す STOP。unit、場所、STOP の文、付属書の節）

- **R21**（W5a、lane X、commit `26b7c350`）：`crates/kivact-runtime/src/reactive/handler.rs` の `mod tests` を追加していない。付属書 R21 の試験のコードに `assert_eq!(rt.create_owner(None, at), rt.root);` があり、`create_owner`（`reactive/owner.rs:66`）は新しい owner の key を挿入するので、root の key と等しくならず、そのまま走ると失敗する。試験の型と関数（`Runtime::register`・`Owner::with_setup`・`shutdown`・`TaskState::new`・`ThreadWaker::new`・`Host::wake`）は存在する。R21 の完了条件は満たしている。設計者は、この assert を消すか、別の検査にするかを決める。この差戻しの間、lane X（鎖）は止める。

## 設計者の回答（W5a・R21 の差戻し、2026-11-20）

- **R21（`handler.rs` の試験）**：assert は消さず、付属書のとおりに入れる。差戻しの前提が誤り。`rt.root` は arena の最初の slot の key（`runtime.rs` で slot 0・generation 0 と決めてある）で、空の arena で最初に呼ぶ `create_owner(None, ..)` はその slot を取るので、返る key は `rt.root` と等しい（`host.rs` の `Runtime::new` が同じ不変条件を assert している。`reactive/selector.rs`・`reactive/turn.rs` の既存の試験も同じ行で通っている）。
  - 設計者の確認：`26b7c350` に付属書の `mod tests` をそのまま足し、`cargo test -p kivact-runtime --lib handler::tests` が通る（1 passed）こと、`cargo clippy -p kivact-runtime --lib --tests -- -D warnings` が通ることを確かめた。lane X の鎖の途中では `kivact-core` が compile できず、dev-dependency の `kivact` が build できないので、確かめるときは試しの worktree で `crates/kivact-runtime/Cargo.toml` の dev-dependency の `kivact` を一時的に外した（commit しない）。lane X の確認は R20〜R25 の共通の形（`check --lib`・`clippy --lib`）のままでよく、この試験が走るのは R31 の `cargo test -p kivact-runtime`。
  - 付属書 R21 の試験の assert の前に、理由の注記を 2 行足した。
  - Orchestrator：lane X で `handler.rs` に試験を足して commit し、R22 から再開する。
- 作業指示書 0.5 節の表に 1 行足した（付属書の試験の assert が失敗しそうに見えても、予想では止めず、入れて走らせる。失敗したら出力を付けて差戻し）。

## 差戻し（W5a・R23、2026-11-20）
（設計者へ返す STOP。unit、場所、STOP の文、付属書の節）

- **R23**（W5a、lane X、commit `7a2f4022`）：`crates/kivact-runtime/src/task/remote.rs` の `RemoteInbox`（field `mail`（`Mutex`）と `signal`、`mail` を lock で書く）が、xtask の state 検査で error になった（`RemoteInbox holds state fields mail, signal and is mutated`）。付属書 R23 は `RemoteInbox` の型を定義しているが（`R-runtime.md` の `struct RemoteInbox<T>`）、「状態の閉じ方」にこの型の印の文が無い。0.2 の 4 により、文面の無い `xtask-allow` は書けない。他の検査は通る（fmt・check・clippy・xtask の残りは ok）。設計者は、この型の印の文（閉じ方）を決める。この差戻しの間、lane X（鎖）は止める。

## 設計者の回答（W5a・R23 の差戻し、2026-11-20）

- **R23（`RemoteInbox`）**：印を 1 行足して進める。構造は変えない。`mail`（`Mutex<Mailbox<T>>`）は他の thread が `Remote::set` で書き、owner の cleanup が `Closed` にする mailbox。`signal` は UI の値で、`deliver` だけが turn ごとに最新の mail で書く。二つは別の事実で、どの組合せも有効（`Closed` の後に signal が残っても、signal は同じ owner と一緒に消える）。`struct RemoteInbox` の直前に次を置く：
  `// xtask-allow(state): independent: mail is the mailbox other threads fill through Remote::set and the owner's cleanup closes; signal is the UI value that only deliver writes, once per turn with the latest mail`
  付属書 R23 の手順のコード片と「状態の閉じ方」に書いた。設計者の確認：`7a2f4022` にこの 1 行を足し、`cargo xtask check --no-write --crate kivact-runtime` が ok になる（error 0）ことを確かめた。Orchestrator：lane X で印を足し、R23 の確認を通して commit し、R24 から再開する。
- **同じ理由で止まる箇所の見直し**：付属書の全コード片の struct を、xtask の state 検査と同じ規則（`bool`・`Option`・counter の名前・`Cell`/`RefCell`/`Mutex`/`RwLock`/`Atomic*`・`Signal`/`Var`。`Rc`/`Arc`/`Box` は剥がす）で数え、2 つ以上のものを未着手の unit について調べた。
  - 止まるもの：E17（W5a の鎖）の `SharedLines`。付属書は「`&mut self` の method が無いので対象外」と書いていたが、`Rc<RefCell<..>>` の field を二つ持ち、xtask は内部可変の field を持つ型を「変更される型」と数える。付属書 E の E17 の「状態の閉じ方」と手順のコード片に、次の印を足した：`// xtask-allow(state): independent: lines is the input's own line cache and engine the shared shaper, both owned elsewhere and borrowed one request at a time; neither field is reassigned, only borrowed`
  - 止まらないもの：A11 の `GpuDevice`（付属書に印の文がある）。A07 の `ImeRequest`、C11 の `LaunchPolicy`、E17 の `EditContext`・`Limits`、E18 の `NodeInput`、R25 の `TraceWrite`・`TraceRun`、K11 の `SidebarState`・`SidebarItemState`、K26 の `ThemeOptions`、U25 の `Authored` は、`&mut self` の method も内部可変の field も Signal を書く method も無い record（xtask の対象外）。
  - 付属書の試験の関数（R31・U32・U42 の、状態の変数を 2 つ以上持つ試験）は、xtask が試験の file を見ないので止まらない（基点の `kivact-test/tests/screen/disposal.rs` に、印の無い同じ形がある）。
- 作業指示書 0.5 節の表に 1 行足した：xtask の `state` 検査が、付属書が作る型か field を足す型を報告し、印の文が無いときは、Orchestrator が 3 つの条件を確かめて `independent:` の印を書いて進める。(1) field ごとの書き手を挙げられる。(2) 他の field を見てから読み書きされない。(3) 同じ事実の重複が無い。印の文と書き手は「Orchestrator の判断」に書く。条件を満たさないときだけ差戻し。付属書の文の上で field を足す型（散文で「`X` に field を足す」とあるもの）は機械的に洗えないので、この行で受ける。

## 差戻し（W5a・R28、2026-11-20）
（設計者へ返す STOP。unit、場所、STOP の文、付属書の節）

- **R28**（W5a、lane X、commit `49707eeb`）：`crates/kivact-interaction` の呼出し元は直した（fmt・check --lib・clippy --lib は成功）。xtask が次の 4 件で失敗する。(1) `crates/kivact-interaction/src/scalar/range.rs`（`follow_range`・`follow_ranges`、59・70・73 行）：effect の closure の中で `Signal::set` を書き、`xtask effect-writes` が error（付属書に扱いが無い。旧 `Binding` の set は検査されず、`Signal` への置換で数えられるようになった）。(2) `disclosure.rs` の `DisclosureApi`：state field `open`・`trigger_state`・`content_state`・`host_disabled` が `xtask state` の error。(3) `scalar/track_data.rs` の `Target`：state field `binding`・`bounds`。(4) `toggle.rs` の `Core`：state field `checked`・`observed`。(2)〜(4) は付属書 R28 に型の印の文が無い。Orchestrator の新しい行（独立の印）の三つの条件を確かめたが、`DisclosureApi` では `open`（Signal）と `shown`（Value、「requested state」）が同じ事実を持つかどうかが判断できない。`Target`・`Core` も書き手と読み手の全部を見ないと確かめられない。設計者は、(1) の扱いと (2)〜(4) の印（または型の作り直し）を決める。この差戻しの間、lane X（鎖）は止める。

## 設計者の回答（W5a・R28 の差戻し、2026-11-20）

4 件とも原因は同じ：R20 が `Binding` を `Signal` に統合したので、caller の binding だった field と、その書込みが、xtask の `state`・`effect-writes` に初めて見えるようになった。どれも型は作り直さず、印で閉じる。

- **(1) `scalar/range.rs` の effect の書込み**：effect の外へは移さない。range の prop が変わったときに caller の値を新しい範囲に入れ直すのは、U08 が消した range の machine と同じ振る舞い。直すのは caller の値で、範囲から外れた値だけを書くので、次の実行では何も書かない。3 つの書込み（`binding.set(value)`・`low.set(from)`・`high.set(to)`）の各行の直前に、同じ字下げで次を置く：
  `// xtask-allow(effect-writes): a range prop change normalizes the caller's bound value into the new range (the base behavior of the range machine U08 removed); it writes only a value the range leaves out, so the next run writes nothing`
- **(2) `DisclosureApi`**：`open` と `shown` は同じ事実だが、重複ではない。`shown` は `use_disclosure` で `open.into_value()` から作る、同じ cell を読む handle で、値の写しを持たない（xtask も `Value` を数えない。報告された field は `open`・`trigger_state`・`content_state`・`host_disabled`）。`shown` は残してよい。
- **(2)〜(4) の印**（付属書 R28 の「状態の閉じ方」に書いた）：
  - `DisclosureApi`：`// xtask-allow(state): independent: open is the caller's requested state, written by hide and the trigger's actions; trigger_state and content_state hold the mounted parts' states (attach_part_state sets them on mount and clears them on cleanup); host_disabled registers each mounted trigger's disabled value for its mount's life`
  - `Target`：`// xtask-allow(state): independent: binding is the thumb's value, written by set and set_position; bounds is the neighbor thumbs' values of a range slider, fixed when the target is made`
  - `Core`：`// xtask-allow(state): independent: checked is the caller's binding, written only by the control's flip through edit; observed holds the mounted control's state (attach_part_state sets it on mount and clears it on cleanup)`
  - 置く場所：印は型の宣言の直前ではなく、その型の doc comment と属性（`///`・`#[..]`）の塊の上に置く（xtask は item の先頭＝doc の 1 行目の直前の行を見る。`struct` の行の直前に置くと「未使用の印」と「印が無い」の二つの error になる）。
- 設計者の確認：`49707eeb` に上の 3 つの印と 3 つの `effect-writes` の印を足し、`cargo xtask check --no-write --crate kivact-interaction` が ok（error 0）になることを確かめた。
- **R29・R30 で同じ原因で止まる箇所（先回りして閉じた）**：同じ worktree で kivact-ui・5 つの kit・kivact-devtools・examples の `Binding<` を機械的に `Signal<` にして xtask を走らせ、基点からの差を見た。新しい error は `state` の 12 型（kivact-ui 2、basic 2、creative 7、soft 1）だけで、`effect-writes`・`ui-machine` の新しい error は無い。12 型の印の文を付属書 R29（`Plan`・`TimeInput`）と R30（残りの 10 型）の「状態の閉じ方」に書いた。12 の印を入れると、どの crate も基点と同じ error の数に戻ることを確かめた。
- 作業指示書 0.5 節の、印の文が無い型の行に判定の補足を足した。(a) caller の binding の field は、書き手を「caller と、その型の `set`/`update` の呼出し」と書けば条件 1 を満たす。(b) 同じ `Signal` から作った `Value` は重複に当たらない。(c) builder（`&mut self`）で報告された型は `record:` の印にする。(d) 印は doc comment の上に置く。今回の (2)〜(4) は、この補足があれば Orchestrator が判断できた種類。
- Orchestrator：lane X で R28 に 6 つの印を足し、R28 の確認を通して commit し、R29 から再開する。

## W5a の記録（設計者、2026-11-20）

ユーザーの指示で、設計者が W5a の鎖の R29〜E20 を実施し（実装は Haiku 5.5 の subagent）、統合と tier b まで行った。差戻しは残っていない。

- **付属書の誤りを直したもの**：R21 の注記、R27 の `on_exit_complete`（包み直した Callback が branch と一緒に破棄される）、C07 の日本語の印、C07〜C17 の W5a の照合の注記（R20〜R30 の後のコードとの名前・path の違い）、C12 の SetupOnly の試験（panic が既存の契約）、C14 の launch_fonts の試験の基準（Inter だけの theme）、E17 の `SharedLines` の印。
- **lane E2（E12・E13）の後退**：W5a の lane E2 は tier b の前に鎖と合わさったため、次の 4 件は鎖の途中で見つかった。すべて直した（`rework/PROGRESS.md` の W5a の節の commit）。(1) 文字色の transition の開始と目標の変え直しで下の描画を捨てていなかった。(2) 読み込み中の画像を含む canvas の scene の描画を保存していた。(3) 描画の key に own と大きさが無く、own でない step の空の描画を再利用していた。(4) 動いている layer を含む layer の bounds と、動く menu に anchor した submenu の位置が古いまま残った。付属書 E の E12・E13 の文は直していない（W5a で済んだ unit なので、理由はコードの comment に書いた）。
- **E17 の性能（記録、W6b で調べる）**：`text_editing::large_document_key_latency`（release、手で実行）で、16,384 行の文書の ArrowUp/Down の p95 が 0.51ms から 2.58ms（約 5 倍）、65,536 文字の 1 行の ArrowLeft が 1.075 倍。上下の移動が goal column のために今の行の x を求めるようになった分だけでは説明できない。1 frame（16ms）より十分小さいので W5a は止めない。W6b の engine の lane で profile を取って原因を調べる（Orchestrator は W6b の E の unit の始めに、この行を設計者に返すこと）。
- **判断の記録**：E17 の上限は、host の切り詰めで文字が元に戻ったら編集を丸ごと取り消し、何も動かない `Done` の key は先へ送る（満杯の text area の Enter が親に届く）。E20 で、focus のある text input での touch は scroll の候補にならない（付属書どおり。touch_scroll の E2E は通る）。C18 の prelude は 5.11 節の一覧から、後の unit（S・U）が足す 6 個を除いた形。component の例は印を置かず、閉じられる card を部品にした。
- **Linux**：C13 の Linux の target の check で、C13 が触っていない `arvane-native` の `winit/foreign/x11.rs` と `x11/test_parent/queries.rs` に未使用の import の warning が 2 件ある。tier b（Windows）の範囲外。Linux の CI か CP2 の前に直す。
- **後片付け**：`W:/temp_check.log`（C08 の subagent が誤って書いた確認のログ）は、subagent の削除が安全確認で拒否されたので残っている。中身はビルドのログだけ。
- **確認の範囲の教訓**：unit の確認（付属書の filter の E2E）だけでは、R の unit・lane E2 の後退を見落とした。W5a の後半は各 unit の後に e2e_core・e2e_ui・kivact-test・e2e_kits・kivact-ui-soft の全件と workspace の clippy（`--all-features`）を走らせた（`target/rework-designer/W5a-unit-brief.md`）。W5b 以降の Orchestrator にも同じ形を勧める。doc の例（doctest）は unit では走らないので、wave の tier b で初めて見つかる。

## KPI の再測定（同じ環境の B0-CP1 と CP1b、2026-11-20）

設計者の回答（W5a の後）の手順で、基点 `c9518946` を同じ環境で測り直した（`rework/kpi/B0-CP1.md`）と、現在の統合 `CP1b`（`rework/kpi/CP1b.md`）を比べた（`rework/kpi/CP1b-ratio.md`）。両方の `kv_heap` に `AcceptedTree` の site がある（B0-CP1 は 4 行、CP1b は 3 行）ので、測り直しは不要。

- **前回の後退（K2 のメモリ、K1 の startup など）は、同じ環境では消えた**（比は 0.99〜1.02 の範囲か、目標に入る）。CP1 の後退は環境の差だった（設計者の回答どおり）。
- **後退（差し戻し）**：K6 `basic-catalog default` の比 1.074（34042880 → 36572672 B）、K6 `basic-catalog no-gpu` の比 1.086（29470208 → 31990784 B）。どちらも改善の必達の上限を超える。原因の候補は、W5b までの間に増えた catalog・ui の binary の中身。設計者が決める。Orchestrator は直さない。
- **改善未達（記録だけ）**：K2 `per_item text`（比 1.011、改善の必達 ≤0.95）、K2 `per_item row`（0.989、≤0.95）、K5 `pointer p95 n3000`（0.953、≤0.85）。
- exit 1（後退あり）。判定は上の 2 行の K6 だけ。


## 設計者の回答（W5b の確認事項と CP1b の K6、2026-11-20）

- **K6 basic-catalog の後退（差し戻しの 2 行）**：原因は W5a。設計者が catalog の release を commit ごとに作り、linker の map で関数ごとの大きさを比べた：`d615568c` 34378240 → W5a の merge `500f069e` 36562944（W5b はほぼ 0）。増えた code の 8 割は `kivact_core::view::part::part<S>` の写しだった。W5a で kivact-interaction の popup・table・drag・disclosure の API が generic な関数の中で `part(..)` を呼ぶようになり、`part::<S, F>` の実体が約 4 → 数百に増えた。`part` の中の観測の closure（`input_observers` の key・key_up・pointer）・binding の effect・`Signal<PartInput>::update` が実体ごとに写され、3052 個（約 1.6 MB）になっていた。
  - **修正（設計者、この commit）**：`crates/kivact-core/src/view/part.rs`。`S` に依らない部分（入力の signal・binding の effect・観測）を generic でない `attach_part()` に移し、`part<S>` は `memo(move || state(facts()))` だけにした。memo の数は 1 つのまま（K2 は変わらない）。写しは 3052 → 73。
  - **修正後**（同じ方法・新しい target dir で測った）：`basic-catalog default` 34792448（B0-CP1 との比 1.022）、`basic-catalog no-gpu` 30210560（1.025）。
  - **判断**：残り（必達の上限 1.02 を 0.2〜0.5 point 超える）は CP1 では受け入れる。残りの増分は広く薄い：E12・E13 の paint の保持の list（engine の約 80 KB。K3・K4・K5 の改善の手段）、R20 で `Binding` を `Signal` にまとめたことによる型ごとの `Signal<T>` の写し、`UiRecipes` の初期化と複製。手段の unit（recipe 表の分割、plot の crate 化。W9 以降）の後、CP3 の AC-2 で判定する。CP2 は catalog を測らない。CP3 で 1.02 を超えていたら、また設計者に返す。
  - **教訓**：generic な公開関数（`part`・`for_each` など、generic な部品から呼ばれるもの）の本体は、型に依らない部分を generic でない関数に分ける。
  - **測り方の注意**：基点と checkpoint の checkout で `CARGO_TARGET_DIR` を共有しない。cargo は checkout の場所を区別しないので、別の checkout で作った古い artifact をそのまま使うことがある（設計者の測定で起きた）。`run.sh` は既定で各 checkout の `target/` を使う。作業指示書 5.1 節の 2 に書いた。K6 は毎回同じ値なので、測り直さない（5.1 節の 3）。
- **(b) `kivact-devtools-agent` の依存**：足す。`kivact-types` は依存の無い値の crate で `visible_to` の制限も無く、agent を使う App は既に link している。丸めの規則を 1 か所に置くため、private の `to_straight` を消して `kivact_types::color::unpremultiply` を copy に呼んだ（設計者、この commit。clippy・試験・xtask は通る）。
- **(a) `color_alpha` の doc**：受け入れる（関数は straight の `Rgba` を返すので、doc はコードに合わせる）。
- **(d) A06 の残り（root `Cargo.toml` の profile）**：変えるものは無い。A06 の所有の「profile 1 行」は、`arvane` crate が消える前提の計画の名残り。`arvane` は window の driver と presenter として残り、毎 frame の経路にあるので、`[profile.dev.package.arvane]` はそのままでよい。root の `[profile.*.package.*]` に並ぶ package はすべて `Cargo.lock` にある（設計者が確かめた）。`templates/basic/README.md` の profile の文は G12（W11b）の手順 3 のとおり。
- **(e) user 向け docs の `Pixels` の straight-alpha の文**：Orchestrator の判断どおり G の unit で直す。
- **release の警告**（記録）：`--no-default-features` の release の build で `kivact-core` の `launch/policy.rs` の `TestRun::name` が「never read」の警告を出す。W5b の tier b の範囲（`--all-features`）には出ない。G かその前の後片付けで扱う。


## Orchestrator の判断（W6a・鎖、2026-11-20）

W6a の鎖（S13・S14）の判断。表で判断したもので、差戻しではない。

- **S13（commit `0b22666e`、0.5：完了条件の rg が既存の行に当たる）**：`git grep -w "Dimension"` が `taffy::Dimension` に当たる（kivact-layout の columns・style/node・style/convert・tree/compute・tree/invalidation・tree/partial）。taffy の型で、Kivact の `Dimension` ではないので受け入れる。
- **S13（commit `0b22666e`、0.5：付属書の置換の式が機械的に壊れる）**：`Dimension`→`Length` の全体の置換が taffy の `Dimension` にも当たったので、kivact-layout の taffy の行を元に戻した。`convert.rs` は taffy の `GridAutoFlow` を `TFlow` と別名にした。`Insets` の script が `impl Edges {` を作ったのは `impl Edges<f64>` に直した。
- **S13（commit `0b22666e`）**：ST-19 の `LayoutProps.hidden` の消去で、付属書の対象（types・style・layout・core の view/tree・devtools/node）に加えて engine の `tree/node.rs` の `hidden()` を消した。engine は W6a の所有の範囲で、同じ field の機械的な追従（0.2 の 11）。`dev/tests/kivact-public-api/tests/e2e_ui/ui_theming.rs` の `.into()` を削った（clippy の `useless_conversion`）。
- **S14（commit `1a27a7f1`、0.5：付属書の置換の式が機械的に壊れる）**：型名と同じ名前の enum variant（`WillChange::Transform`、`TransitionProp::Transform`、`FillImage::Gradient`、recording の `Shape`、`Open::Transform`）が巻き込まれたので、元の variant 名に戻した。kurbo の `Shape as _` の import を `Outline` に変えたものも戻した（`arvane-render` の `convert.rs`・`painter.rs`）。kurbo の `Affine` は残し、kivact の型は `kivact_types::Affine` と書いた（付属書 4 章「手で直すもの」）。
- **S14（commit `1a27a7f1`、0.5：書き換え漏れ）**：`kivact_types` の prefix の pass が `kivact_types::geometry::Transform`・`kivact_types::effect::FilterGraph` のようなネストした module の path を漏らしたので、2 回目の pass（`kivact_types::(?:geometry|effect|paint)`）で直した。`lib.rs` の `Kind` の再公開は付属書の手順 2 の一覧に無かったが、付属書の意図（`TransitionKind`）どおりに改名した。
- **S14（commit `1a27a7f1`、付属書の確認コマンドの誤り）**：arvane の E2E は `crates/arvane/Cargo.toml` の `required-features = ["test"]` のため、付属書の確認コマンドは `--features test` が無いと build できない。`--features test` を足して実行した（試験の期待値は変えていない）。設計者は付属書 S14 の確認欄を直してよい。
- **後退の確認の範囲（W6a）**：unit の後の確認は、`e2e_core`・`e2e_ui`・`e2e_native --all-features`・`kivact-test` の全件と、workspace clippy（XK を除外）。`e2e_kits` と `kivact-ui-soft` は、この wave で compile の対象外（作業指示書 2.1 と 4 章）なので走らせていない。
- **window-lock の終了コード（注記）**：`dev/tools/window-lock.sh` は cargo の終了コードをそのまま返す（`run "$@" && exit 0 || exit $?`）。S14 の subagent が exit 0 を見たのは、確認のコマンドの pipe によるものと思われる。以後、Orchestrator は pipe を使わず、試験の結果は log の `test result` 行で確かめる。

## W6b の開始と設計者へ返す行（E17 の性能、2026-11-20）

- W6b は 5 lane（鎖 X：S15→S23 の 10 unit、E1：E19→E21、E2：E14→E32→E24、A1：A11、A2：A12）で始める。
- **E17 の性能（W5a の記録から、設計者へ返す）**：`text_editing::large_document_key_latency`（release、手で実行）で、16,384 行の文書の ArrowUp/Down の p95 が 0.51 ms から 2.58 ms（約 5 倍）、65,536 文字の 1 行の ArrowLeft が 1.075 倍。上下の移動が goal column のために今の行の x を求めるようになった分だけでは説明できない。1 frame（16 ms）より十分小さいので W5a は止めなかった。W6b の engine の lane で profile を取って原因を調べる（作業指示書の W5a の記録の指示）。設計者の判断は今は要らない。原因が分かったら返す。

## Orchestrator の判断（W6b・A11、2026-11-20）

A11（lane A1、commit `03c4d66a`・`db068636`）の subagent が、付属書から外れて自分で決めた点。Orchestrator の判断は次のとおり。どれも振る舞いを変えず、付属書の意図どおりに合わせたもの。

- (a) `GpuDevice` の field の順：`external` を先頭にした（drop の順で `external` を `device` より先に落とす現行の不変条件を守る）。採る（0.5：付属書の意図どおり）。
- (b) `SurfaceConfigurer` に `failure`（`Arc<DeviceLost>`）を持たせた：現行の `configure` が最後に device の失敗を返すため。採る（同じ）。
- (c) `SurfaceConfigurer::configure` の戻り値は `()`、時間計測は presenter の worker（付属書の「今の configure に合わせる」）。採る。
- (d) `GpuDevice::release_caches` は追加しない：現行の呼出しが無く dead code になるため。採る。
- (e) `surface`（`GpuDevice` へ移動）：field が移るための移動。採る。
- (f) `Pipelines::present` を `RefCell`：`present_for` が `&self` を要するため。採る。
- (g) 非 gpu の stub（`gpu_unbuilt.rs`）：`GpuContext` と `SurfaceConfigurer` を、uninhabited な enum から private field の struct（本体は `unreachable!`）に変えた。uninhabited のままだと非 gpu の build で到達しない型が 10 件の warning を出すため。型の形の変更なので、設計者が見直せるよう記録する。採る。
- 0.5 の行（clippy の `result_large_err`）：`ConfigureFailure::Failed` の note を縮めた。`Box` にせず、`#[allow]` は足さない。
- 試験：`cargo test -p arvane-compose --all-features` は 0 件（この crate に test が無い）。付属書の行は到達できない。arvane の E2E（36 件）と GPU 系の E2E（`gpu_on_demand`・`drawable_export_gpu`・`gpu_interop`・`shaders`）で代える。採る。
- 未実施：`KIVACT_TEST_GPU=1` と `KIVACT_TEST_MIRROR=required` の GPU の形の試験は、A11 の unit では走らせていない。CP2 の tier c で確かめる。

## 差戻し（W6b・A12、2026-11-20）

- **A12**（lane A2、commit `4783cc17`）：付属書 A12 の試験 `injected_input_reaches_an_embedded_app`（`dev/tests/kivact-public-api/tests/e2e_native/embed/contract.rs` に足す）を書けなかった。embed の App へ注入する helper が無い。`kivact::test` の Screen は engine の test host、DevTools の `inject_input` は独立の test app だけを駆動し、`TestParent::click` は実 window の OS 入力。どれも `ShellLoop::inject` の embed の経路を通らない。試験は足していない。
- 設計者に決めてほしいこと：helper を新設するか、この試験の場所（または経路）を変えるか。
- 他は完了：確認（tier a）は全件 exit 0、後退の確認 4 件（e2e_core 470・e2e_ui 252・e2e_native 45・kivact-test 24、失敗 0）。この差戻しが残っている間は、最終 checkpoint で完了を宣言しない。

## Orchestrator の判断（W6b・A12、2026-11-20）

- `FRAME_PERIOD_NS`：付属書の `crate::FRAME_PERIOD_NS` は `kivact_types::defaults::FRAME_PERIOD_NS`（同じ意味の一つだけ）を使った。採る（0.5）。
- `canonical`（a11y の validation）：付属書の `canonical(tree) -> SemanticsTree`（Result なし）の形は採らず、`Result<SemanticsTree, AccessibilityError>` のまま残した。並べ替えの traversal は MissingNode・InvalidTree・UnreachableNode を返しうるので、release で消すと不正な木で panic するか誤った順になる。検査（重複・値・text・relations）だけを `#[cfg(debug_assertions)]` の validate に移し、正規化は release でも行う（付属書の意図どおり）。採る。形は付属書と違うので、設計者は見直せる。
- Linux の warning 2 件（`crates/arvane-native/src/port/winit/foreign/x11.rs:13`、`crates/arvane-native/src/port/x11/test_parent/queries.rs:5`）：A12 が触っていない file の未使用 import で、W5a の記録（C13）と同じ前からのもの。Linux の CI か CP2 の前に直す（記録のみ）。

## Orchestrator の判断（W6b・E19、2026-11-20）

E19（lane E1、commit `c0dbc924`）の subagent が付属書から外れて決めた点。Orchestrator の判断は次のとおり。

- 手順 4 の `HostRequest::Platform { window, request: PlatformRequest::RequestWindowSize(size) }` は実在しない。`HostRequest::RequestWindowSize { window, size }` が独立の variant で、`DeferredRequest::RequestSize` に対応する。これを使った。採る（0.5：付属書のコード片の名前が無く、同じ意味のものが一つだけある）。
- 手順 7 の `shell.on_renderer_change(...)` は実在しない。実コードは `arvane::observer(closure)` で、observer を `DriverOptions` に渡している。`renderer_logger` は `arvane::observer` の引数の型（`FnMut(WindowId, &RendererChange)`）に合わせて返し、`launch.rs`・`embed.rs` は observer を使う形にした。採る（同じ行）。
- `queue_effect` に `RequestWindowSize` の window の存在確認を足した。旧コードが確認していたため、これが無いと閉じた window への要求が queue に残る。採る（振る舞いを保つ）。
- `pump` の effects は、付属書の一つの関数のままだと 80 code 行を超えるので、腕の処理を `carry_out`（`Option<PlatformResult>`）に分けた。採る（0.4 の「意味のある helper に分ける」）。
- `DeferredRequest` の削除後に未使用になった `EngineEmulation` と `PlatformRequest` の use を消した。採る（機械的な後片付け）。

## 差戻し（W6b・S15、2026-11-20）

- **S15**（lane X、commit `b8470efa`）：`crates/kivact-style/src/paint/border_image.rs` の `BorderImageSource::Gradient(Gradient)` を `Gradient(Box<Gradient>)` にした（clippy の `large_enum_variant` を消すため）。公開の variant の型が変わるので、0.5 の表の「公開の API を変えないものを採る」に当たらない。設計者が決める：Box の形を採るか（API の変更として記録するか）、別の形にするか。この決まりが出るまで、lane X は S16 に進まない。
- **S15**：`Draw` の text_run の spacing は、theme が無いときに既定の theme に落ち、その unsupported の数を上へ渡していない（subagent の報告）。振る舞いの決まりなので、設計者が見る：この落とし方でよいか。

## Orchestrator の判断（W6b・S15、2026-11-20）

- **S15 の手順 8 の残り**：subagent の報告で、`to_len()` を resolved に通していない呼出しが残った（`effect/shape.rs`・`geometry.rs`・`paint/fill_layer.rs`・`path/commands.rs`・`resolve/paint.rs`・`transform.rs`）。付属書 S15 の手順 8 が `rg` の全件を対象にしているので、設計の判断ではない。同じ lane の fresh の subagent で残りを仕上げる（commit は新しく作る。`Box` の件には触らない）。
- `kivact-engine/src/model/interact/sticky.rs` の `Calc {percent, px}` の pattern を `..` に直した：所有の外の 1 file。W6b の他の lane は、この file を持っていない。採る（0.5 の「所有の外の 1〜3 ファイル」の行）。
- `theme_var.rs` の doc の例を `text` の block にした：例が `kivact::` の path を使い、kivact-types の doctest で失敗するため。採る。
- `VarValue<FontFamily>` の `From<&str>` / `From<String>` は無い（orphan rule）。呼出し側は `FontFamily::from("...")` と書く。採る。
- `misplaced_content` の文面を変えた（`N length(s) the host cannot take here: ...`）。E2E の期待は文に `length` を含むことなので、満たす。採る。
- `view/overlays.rs` の `theme_overlay_margin`、`chat_thread/view.rs` の `follow_slack`：関数の長さの上限を守るための helper。採る（0.5）。
- `apply.rs`（DevTools）：token の表を ThemeVar の参照に置いた。DevTools の token の名前と protocol は変えていない。採る。

## 差戻し（W6b・E21、2026-11-20）

- **E21**（lane E1、commit `f8788b46`）：付属書に書かれていない経路の値が変わった。sample の無い取消し（`cancel_pointer_input`）は、`cancel_sample`（host の時刻、pressure・tilt は `None`、modifiers は default）を使うようになった。このため、取消しの event の `time_ns`・pressure・tilt は、前の event の値ではなくなった。付属書 E21 の完了条件は「time_ns は前と同じ値」なので、この経路だけ食い違う。試験は通っている。設計者が決める：前の event の値を保つ（track に値を残す）か、host の時刻を採るか。この決まりが出るまで、最終 checkpoint で完了を宣言しない。

## Orchestrator の判断（W6b・E21、2026-11-20）

- shell の mouse の Cancel：Cancel の event と後続の Move・Leave は、modifiers と buttons を default・NONE にした sample（ended）で送る（以前の挙動に合わせた）。採る。
- `cancel_contact` の helper を新設し、`cancel_pointer_input` と `end_contact` を一つにまとめた。`end_contact` は sample を受ける（Up の input、取消しの `cancel_sample`）。付属書の手順にはないが、同じ経路の二重を消す。採る（0.4 の「意味のある helper に分ける」）。
- `contact_at` の input 引数は使わなくなるので消した。`touch_move`・`touch_candidate`・`release_touch_scroll` は付属書の文どおり引数を足した。採る（crate の中の機械的な追従）。
- `pointer_down`・`pointer_up` は press を引数から外し、`input.phase` から取る（付属書どおり）。`primary_press` には modifiers の値をそのまま渡す（`send_pointer` を呼ばないため）。採る。

## 差戻し（W6b・S15 の続き、commit `67fdd8eb`、2026-11-20）

- **`path/commands.rs:Path::rounded_rect`**（公開のメソッド、theme を取らない）の `r.x.to_len()`・`r.y.to_len()`（213〜214 行付近）：resolved を通すには theme が要る。公開の API を変える（設計書 5 章）か、解決を後にする（`Length` を未解決のまま持つ）か。設計者が決める。
- **`transform.rs:Transform::translate_3d`**（公開。`moved()` が px と pct を即時に持つ）の `x.into().to_len()`（285 行付近）：build の時点で var と em が落ちる。解決を遅らせる保存の形が要る（設計の判断）。
- **workspace clippy の失敗（権限の判定で止まっている）**：`kivact-core/src/devtools/apply.rs` の `COLOR_TOKENS`・`LENGTH_TOKENS`・`WEIGHT_TOKENS`（424・445・466 行付近）の `&'static ThemeVar<..>` が `redundant_static_lifetimes` になる（`b8470efa` から）。clippy の提案どおり `'static` を消すだけの機械的な修正。subagent の編集（sed）は権限の自動判定で拒否された。Orchestrator はこれを回避しない。ユーザーの許可を待つ。許可が出たら、その 3 行だけを直して clippy を通す。
- **workspace clippy の残り**（`BorderImageSource` の `large_enum_variant` を含む）は、上の clippy が通るまで確認できていない。
- **設計者が見る subagent の判断**（Orchestrator は採っていない）：(1) draw 時の resolve は theme の base size の em を使う（drawing は style を持たないため）。(2) draw 時の不正な sum は `Draw::unsupported` に足す。(3) `fill_for` と span の背景では、不正な sum を捨てる（`spacing_px` の先例に合わせた）。(3) は値を黙って落とすので、設計者の確認が要る。

## 設計者の回答（W6b の差戻し：E14・A12・S15・E21、2026-11-20）

権限の自動判定で止まった編集（印と `'static`）は、設計者が lane の branch に直接 commit した。Orchestrator はそれを取り込んで続ける（lane の worktree を reset しないこと）。

### E14（lane E2）：`SceneOut` の印

- 付属書 E14 の手順どおり `keeping` を残し、印を足した（W5a の E12 で `SceneOut` の印は外れていたので、新しい印にした）：`// xtask-allow(state): keeping marks the commands kept from the last frame (set_keeping); independent`。commit `f12b505b`（`wt/rw-W6b-E2`）。`cargo xtask check --no-write --only local` は成功（設計者が確認）。
- 未実施のまま残っているもの（commit `f54a8875` の message のとおり）：`tests/content_version.rs` の再実行（debug の全体と release）と後退の確認 4 件。Orchestrator が走らせてから E32 へ進む。

### A12（lane A2）：embed への注入の試験

- 原因：embed の App の DevTools は注入を断っていた（`agent_slot` の `can_inject` が `!policy.is_embedded()`、`inject_input` が `UNSUPPORTED` を返す）。embed の loop が注入の入力を捨てていた（AR-05）ための封じで、A12 が届けるようにしたので封じは要らない。helper を新設せず、DevTools の client の経路で試験する。
- 設計者が実施（commit `d9abce37`、`wt/rw-W6b-A2`）：
  - `crates/kivact-core/src/devtools/{state,start,input}.rs`・`launch/runtime.rs`：`can_inject` を消した（`AgentStart` は agent の receiver の別名、`inject_input` の `UNSUPPORTED` の分岐を削除）。`Inspector` の印は、xtask が「未使用の印」にしたので外した（0.5 の表の行。legacy の印は 182 個に減った）。
  - `dev/tests/kivact-public-api/tests/e2e_native/embed/contract.rs`：`injected_input_reaches_an_embedded_app`（feature `devtools`）。embed の App の agent を `list_targets()`（pid と App の名前）で見つけ、別 thread の client が `list_windows`・`get_by_role`・`inject_input`（Click）を呼び、押した回数が 1 になるのを確かめる。
  - 確認：`e2e_native --all-features -- embed::` 13 件成功（新しい試験を含む）、`-- devtools` 成功、`XK` 付きの workspace clippy 成功、`cargo fmt --all --check` 成功、`cargo xtask check --no-write --only local` 成功。
- 付属書 A12 の試験の行はこれで満たす。A12 の差戻しは閉じる。

### S15（lane X）

1. **`BorderImageSource::Gradient(Box<Gradient>)`**：採る（公開の変更として記録する）。呼出し側は `BorderImage::new(linear_gradient(..))`（`From<Gradient>`）で作り、`Box` を書かない。変種を名指す呼出しは repo に無い。G の移行の文書に 1 行足す（G へ）。
2. **workspace clippy**：`apply.rs` の 3 行の `'static` を消した。commit `0b554b35`（`wt/rw-W6b-X`）。W6 の clippy は作業指示書 2 章の `XK` の除外を付ける（kit は DEC-14 で compile の対象外。付けないと kit が `ColorToken` などで落ちるが、それは正しい状態）。`XK` 付きの `cargo clippy --workspace --all-targets --all-features --exclude kivact-docs` は成功（設計者が確認。`large_enum_variant` を含め 0 件）。
3. **`Transform::translate_3d`（と `translate`・`translate_x`・`translate_y`）**：`Transform` は幾何の値で、theme を持たない（`Path::transformed` や `Layer::transform` にも使う）。解決を遅らせる保存は作らない。決まり：
   - x・y は px と box の百分率（`pct(-50)`、`pct(50) - 4`）。theme の変数・`em`・不正な和を含む長さは、その長さ 1 つにつき `self.unsupported += 1` し、px と百分率の部分だけを使う（`to_len()` のまま）。数は今の仕組み（`Transform::unsupported`、style では `transform: N function(s) the form cannot keep`）で報告される。
   - 判定は kivact-types の `Length` に足す `pub fn needs_theme(self) -> bool`（`Length::Var(..)`・`Length::Em(..)`・`Length::Invalid`・`Calc` の `em != 0.0` か `var.is_some()` で `true`。実際の variant 名は `length.rs` に合わせる）。
   - `translate` の doc に 1 文：「A theme variable or `em` here is unsupported (a transform holds no theme): set the style's `translate_x`/`translate_y`, which resolve them.」。
   - style の `translate_x`/`translate_y`（`transform_of` の `cx.len(..)`）は今のまま解決する。
   - 試験（`transform.rs` の `mod tests`、無ければ作る）：`fn a_theme_length_in_a_transform_is_unsupported()`（`Transform::new().translate(space::MD, em(1)).unsupported() == 2`、`Transform::new().translate(pct(-50), 4).unsupported() == 0`）。
4. **`Path::rounded_rect`**：`Path` の他のメソッドはすべて px の数（`Point`・`Rect`・`f64`）を取る。`rounded_rect` だけが `Length` を取っているので、px に揃える（公開の変更。repo の中に呼出しは無い）：
   - `pub fn rounded_rect(self, rect: Rect, radii: [(f64, f64); 4]) -> Self`。doc：「Clockwise corners from the top left, each `(x, y)` in px; radii that do not fit shrink together as CSS's do.」。本体は `let mut r = radii.map(|(x, y)| (x.max(0.0), y.max(0.0)));` で始め、以降（`ratio` と `scale`）は今のまま。`CornerRadius` の use が不要になれば消す。
   - style の radius と同じ形が欲しい呼出し側は、百分率を `rect.width * p / 100.0` で、変数を `Draw::theme().get(&radius::MD_VAR)` で px にしてから渡す。
5. **`Draw` の text_run の spacing（theme が無いときの既定の theme）**：採らない。測定と描画が別の theme で長さを解けば、測った幅と描いた幅が食い違う。text_run は常に使う場所の theme で解く：
   - `crates/kivact-style/src/draw/text_run.rs`：`pub fn text_run(text, origin, style, scale, color, theme: &Theme, unsupported: &mut u32)`。`fill`・`stroke` の `theme.map(..)` を外し、`fill_for(f, theme, unsupported)` にする。`spacing_px(length, size, theme, unsupported)`（`unwrap_or_default` を消す）。`fill_for(paint, theme, dropped: &mut u32)` は `Resolve::of_theme(theme, dropped)` を使う（「not counted」の注記を消す）。
   - `Draw` の中の呼出し（`text_run.rs` の `text`・`text_path` など、`paragraph.rs`）：描く呼出しは `&self.theme, &mut self.unsupported`。測るだけの呼出し（`Draw::layout_paragraph`・`layout_inline_paragraph`）は `&self.theme` と捨てる数（`&mut 0`。同じ format を描く呼出しが数える）。
   - `crates/kivact-core/src/custom/text_shaping.rs`：`paragraph_measurer` が読んでいる theme を返し（`-> (impl TextShaper, Theme)`）、`text_run(text, style, &theme)` に渡す（数は捨てる）。
   - `crates/kivact-core/src/custom/measurement.rs` と `view/measured_leaf.rs`：`Constraints` に private な field `theme: &'a Theme` を足し、`constraints(request, text, theme)` にする。`bind_measure` は `custom/text_editing.rs` の `bind_highlight` と同じ形にする：hook の content を `(T, Theme)` にし、theme は `bind_highlight` と同じ取り方（`kivact_style::internal::theme_of(owner)`、無ければ `Theme::default()`）で render effect の中で読む。theme が変わると measure の関数を書き直す。`src.fixed()` の近道は消す（theme が変わりうるので、常に effect を通す）。`measure_text`・`measure_paragraph`・`measure_inline_paragraph` は `self.theme` を渡す（数は捨てる）。
   - 試験（`kivact-public-api` の e2e_core の canvas か primitive の module。付属書 S15 の E2E の置き場所に合わせる）：`fn measured_and_drawn_spacing_agree_in_a_theme()`：`Theme::light().set(&space::MD_VAR, 20)` の下の primitive で、`letter_spacing: space::MD` の `TextFormat` の `Constraints::measure_text` の幅が、同じ text を `Draw::measure_text` で測った幅と等しく、spacing 0 の幅より大きい。
6. **subagent の draw 時の判断**：(1) draw の `em` は theme の基準の大きさ（`font::SIZE`）：採る。(2) draw の不正な和は `Draw::unsupported`：採る。(3) 黙って捨てる所：`fill_for` は上の 5 で数える。span の背景は `compile_span_style` が既に数えている（`internal.rs`）。残る例外は二つだけとし、doc に書く：測定（同じ format の描画が数える）と、editor の highlighter の装飾（`compile_text_decoration`。layout の中で走り、報告の先が無い。`StyledRange` の `decoration` の doc に「An invalid length here is ignored: a highlighter runs inside layout, where nothing is reported.」）。
- 3〜6 は lane X の fresh の subagent が `67fdd8eb`・`0b554b35` の上で実施し、付属書 S15 の確認（tier a）と後退の確認、`XK` 付きの workspace clippy を通してから S16 へ進む。

### E21（lane E1）：sample の無い取消しの値

- 付属書 E21 の手順自体が `cancel_sample` の `time_ns: self.frame.host_now()` を指定している。shell は event に自分の時刻（`clicks.rs` の `input.time_ns = now`）を付けるので、host の時刻は前の event と同じ時計で、それより前にならない。取消しはその時に起きるので host の時刻が正しい（Web の `pointercancel` の時刻も発送の時刻）。pressure・tilt は `None`、modifiers は default：押しているものが無い sample として正しい。
- 実装はこのまま採る。食い違っていたのは完了条件の文なので、付属書 E21 の完了条件を「前と同じ値（sample の無い取消しは `cancel_sample` の値：host の時刻、pressure・tilt は `None`）」に直した。E21 の差戻しは閉じる。

## Orchestrator の判断（W6b・S15 の残り、commit `bb13f33b`、2026-11-20）

- 設計者の回答 3〜6 を subagent が実施（commit `bb13f33b`）。確認（tier a）・後退の確認 4 件（e2e_core 475・e2e_ui 252・e2e_native 45・kivact-test 24＋2）・XK 付きの workspace clippy は成功。採る。
- 決まり 4 の「repo の中に呼出しは無い」は事実と違った。呼出しは 2 箇所：`crates/kivact-ui/src/toggle/look.rs`（kivact-ui の本体。決まりの追従として 0.5 の行で許可）と、`dev/tests/kivact-public-api/tests/e2e_core/canvas.rs`（試験。期待値は変えず、同じ px の値に書き換えた）。採る。
- 決まり 5 の「測るだけの呼出し」に `Draw::measure_text` を含めた（数は捨てる）。決まりの本文に明示は無かったが、測るだけの呼出しなので採る。
- 決まり 5 の `text_path` を `&mut self` に変えた（公開の API の変更。決まりどおり。呼出しは e2e の試験だけ）。採る。
- **kit に残る呼出し（DEC-14、compile の対象外）**：`crates/kivact-ui-basic/src/recipes.rs` の 250・259 行の `Path::new().rounded_rect(.., corners(..))`。kit の移行（W9）で、px の `[(f64, f64); 4]` の形に直す。K の lane（W9）の仕事として記録する。
- **G へ（設計者の決まり 1）**：`BorderImageSource::Gradient` の `Box` の 1 行を G の移行の文書に足す（W11b）。

## Orchestrator の判断（W6b・E32、commit `19fa977e`、2026-11-20）

- 注入の位置：付属書の `outdate_paint` の loop ではなく、`PaintState::outdate_draws`（`paint/list/invalidation.rs`）の `for id in changes.redrawn()` に置いた。`outdate_paint` の loop は `forget_ink` を呼ぶもので意味が違う。採る（0.5：付属書のコード片の名前が無く、同じ意味のものが一つだけある）。
- 試験の driver の順：付属書の `scroll_to` と注入の試験は `commit` の後に `redraw` だけを呼んでいた。`wake` が無いと sync（`outdate_draws`）も scroll も起きず、試験は何もしない。`content_version` と同じ `commit → wake → redraw` の順に直した。期待値は変えていない。採る。**設計者へ**：付属書 E32 の driver の順が誤っていたので、付属書の本文も `wake` を含めて直す。
- clippy の `manual_is_multiple_of`（`i % 2 == 0`）は、clippy の提案どおり `i.is_multiple_of(2)` にした。`#[allow]` は無い。採る。

## Orchestrator の判断（W6b・S16、commit `1307ae90`、2026-11-20）

- `Owner::ambient` は現コードに無い。`Owner::try_ambient() -> Option<Owner>`（misuse を記録しない版、`owner_scope` の成功の経路を使う）を `kivact-runtime`（内部の crate）に足した。採る（0.5：付属書のコード片の名前が無く、同じ意味のものが一つだけある）。
- `rg` の完了条件が `flows.rs:28` の関数名 `show_sets_up_plain_components_in_the_branch` に当たった（`components_in` の一部）。別の識別子なので受け入れる（0.5：完了条件の rg が既存の行に当たる）。
- devtools の `hsl_of`・`hsl_hex` は literal の色だけを扱うので `&Theme::default()`、`look.rs` の literal の `mix` は `&Theme::light()` を渡した。出力は theme に依らない。採る。
- `kivact-style/src/resolve.rs:6` の doc に `Color::resolve` の名前が残っている（intra-doc の link ではないので警告は出ない）。doc の名前だけなので、後片付けの unit（W11c）で直す。記録のみ。

## 差戻し（W6b・S16、2026-11-20）

- **S16 の試験**（`crates/kivact-style/src/theme/value.rs` の `components_and_mix_resolve_in_the_given_theme`）：付属書の `assert_eq!` が、`mix(.., 0.0, ..)` の Oklab の往復の誤差（約 1e-16）で失敗した。subagent は、期待値は「同じ色」のまま、`1e-9` の許容差の比較に変えた。試験を緩めたくない場合は設計者が見る（0.2 の 3：許容差を広げることは禁止されている。ただし、この試験は新しく書いたもの）。設計者が決める：許容差を残すか、付属書の期待値を直すか。残す間は、最終 checkpoint で完了を宣言しない。

## Orchestrator の判断（W6b・E24、commit `8103101c`、2026-11-20）

- 付属書の `Dimension::Percent` は、W6a の改名で `Length::Percent` になっていたので `Length` を使った（`tests/hit_reference.rs`）。採る（0.5：付属書のコード片の名前が無く、同じ意味のものが一つだけある）。
- 試験の driver の順は `commit → wake → redraw`（E32 の決まり）。採る。
- `LayerView` の移動に付随する `merge_below` と `own_facts` を `facts.rs` に移した（`layer.rs` の唯一の利用元）。採る（移動の付随）。
- `layer_facts`・`subtree_facts` の free 関数の引数を `&Model` にした。`flat_children` は Vec 化せず iterator のまま使った（結果は同じ）。採る。
- `children()` の small 枝の本体は `small_child` に切り出した（引数 7 個の上限を守るため）。採る（0.4 の「意味のある helper に分ける」）。
- 経過：E24 の subagent は 2 回、確認の途中で止まった（detached の regression 実行を残した）。regression の結果は logs（`reg1`〜`reg4`）で確かめ、clippy・xtask・fmt・commit は別の subagent が仕上げた。結果：e2e_core 470 passed・e2e_ui 252 passed・e2e_native 49 passed・kivact-test 26 passed、すべて failed 0。workspace clippy 成功、`xtask --only local` 成功。

## 差戻し（W6b・S17、commit `20a06b61`、2026-11-20）

lane X を S17 の所で止める。次の 2 件は設計者が決める。決まるまで S18 以降に進まない。

- **(a) `describe.rs` の `unsupported_in_states`**：付属書 S17 は `!matches!(d, Decl::State(..))` で nested state を報告から除く。subagent は除かずに直した。理由：既存の E2E `style_paint::a_property_its_state_cannot_change_is_reported` が `hover.hover` の報告を期待するため。0.5 の表の「画面・a11y・focus・event・API の返す値の期待値は差戻し」に当たる（報告は API の返す値）。設計者が決める：(1) 付属書どおり nested state を除き、この E2E の期待値を直す、または (2) 報告を残し、付属書の表を直す。
- **(b) border の幅の分類**：付属書の表は `BorderTopWidth` 等の幅を `Rest` とする。だが `paint_delta` は幅を描くので、hover の border 幅は描かれるのに「Unsupported」として報告される。付属書の表と `border()` の doc（「state で幅は描かれる」）が食い違う。subagent は S19（ST-05）で解消される見込みと書いたが、S19 までこの報告は残る。設計者が決める：表と doc のどちらを正とするか、S17 で直すか S19 まで待つか。

## Orchestrator の判断（W6b・S17、commit `20a06b61`、2026-11-20）

- 0.5 の表どおり、`grid.rs`・`text.rs`・`sheet/fields.rs` の legacy の印 9 件と、`Groups` の印を外した（mutate する状態が 2 つ未満で、xtask が未使用の印として error にするため）。`StateFields` の印は残した。採る。
- `describe_state_styles` が `StyleState::ALL` の `name()` を使うため、`Active` の名前が `pressed` ではなく `active` になる。`kivact-core/src/devtools/styles.rs` の `state_layers` は `"pressed"` を探すので、DevTools で pressed の layer が消える。試験は通っていた（試験で見えていない）。C19 がこの file を持つので、C19 の brief に「`active` を `pressed` に読み替える」を明記して直す。採る（付属書の意図は C19 で閉じる）。
- `CompiledStyle.duplicates` が層をまたぐ重複も数えるので、debug の診断が層の上書きも報告しうる。付属書 S18 で core が層ごとに数えるように直す。採る（interim。S18 で閉じる）。
- clippy の `redundant_guards`（`Some((var, a)) if a == 1.0`）と `clone_on_copy`（`LineHeight`）は、clippy の提案どおり直した。採る。

## 設計者の回答（W6b の差戻し：S16・S17・E32、2026-11-21）

- **S16 の許容差**：`1e-9` の許容差を残す。`mix(.., 0.0, ..)` は Oklab を往復するので、bit の一致は契約にしない（`t == 0.0` の特別扱いを入れる理由がない）。この試験は新しく書いたものなので 0.2 の 3 には当たらない。付属書 S16 の試験の文を「各成分の差が `1e-9` 以下」に直した。S16 は閉じる。最終 checkpoint の妨げにならない。
- **S17 (a) nested state**：(2) を採る。報告を残し（subagent の実装のまま）、付属書 S17 の `unsupported_in_states` の `filter` から `&& !matches!(d, Decl::State(..))` を消した。状態の style の中の状態は効かないので、黙って捨てずに報告する。
  - S19 で報告そのものを消すと同じ E2E が落ちるので、付属書 S19 を直した：`unsupported_in_states` を `nested_in_states`（状態の中の `Decl::State` と `Decl::Group` だけ）に置き換え、core の `StyleReports.states` は残して文を変える。E2E は `a_state_inside_a_state_is_reported` に改名し、`hover.hover` を含み `hover.width` を含まないことを確かめる。S19 の所有に `style_paint.rs` の 1 試験を足した。
- **S17 (b) border の幅**：S19 まで待つ。表（幅は `Rest`）が正で、`border()` の doc は S19 後の姿。S17〜S18 の間に幅を hover に置くと「Unsupported」と報告されるのは、lane の中だけの途中の状態として受け入れる（S19 が `sets_border` を色と line style だけにし、幅は base に merge する）。S17 で直さない。
- **E32 の driver の順**：付属書 E32 の `scroll_to` と注入の試験に `host.wake();` を足した（`commit → wake → redraw`）。
- lane X は S18 へ進んでよい。

## 差戻し（W6b・C19、2026-11-21）

C19 の commit `79358493`（`wt/rw-W6b-X`）は、確認（tier a）をすべて通した（check・clippy・xtask・e2e_native の devtools の 1 件）。ただし次の 2 件が、Orchestrator の判断表（0.5 節）に当たらず、設計の判断が要る。lane X はここで止める（S18 以降は未着手。commit は reset せず残す）。

- **(1) shorthand 名の override（`background`・`line_height`・`radius`）**：付属書 S17 の `shown_only!`（`crates/kivact-style/src/sheet/edit_text.rs`、付属書の 3735 行付近）が `Background`・`LineHeight`・`CornerRadius` を「表示のみ」（`from_edit_text` が常に Err）にしている。旧 `override_paint`・`override_text` はこれらを text から読んでいた。C19 の後、DevTools の `set_style_override` でこの 3 名は `... is not editable as text` の error になる。試験（`padding` だけ）は落ちない。設計者が決める：(a) S17 の表で、この型に text の parse を足す（DevTools の override を保つ）、(b) この 3 名の override を DevTools から外す（動きの変更として G の移行に記録する）。
- **(2) state layer の値（`crates/kivact-core/src/devtools/styles.rs` の `state_layers`）**：`describe_state_styles` は `describe_style` の Debug の文字列を返す（付属書 S17 の内部の橋。3531 行と 4453 行）。state ごとに表の `describe` を呼ぶ入口が無いので、C19 は state layer の値を Debug の文字列のまま出した。Orchestrator の判断 (d)（値は表の `describe` から読み、Debug の文字列は読まない）を満たせない。設計者が決める：(a) S17 に state ごとの表の入口（表の `describe` を使う `pub(crate)` の関数）を足し、C19 がそれを使う、(b) state layer の値を一覧に出さず名前だけにする、(c) Debug の文字列のまま残す（その旨を C19 の完了に書く）。

C19 のほかの点は、Orchestrator の判断どおり：shorthand の展開は apply.rs の `SHORTHANDS` の一覧（`padding`・`margin`・`gap`・`border_width`・`border_color`・`radius`）、`state_layers` の `active` と wire の `pressed` の対応は一か所（comment 付き）、`number`・`color`・`line_height` の旧 parser は `apply_token` が使うので残した。E2E の期待値（`padding`）は変えていない。

## Orchestrator の判断（W6b・E14 の検証、2026-11-21）

- E14 の `content_version` の再実行と後退の確認は、E2 の worktree で実施済み（`V:/kv-target/logs-W6b-E2/verify-e14-summary.txt`：xtask・content_version の debug と release・e2e_core・e2e_ui・e2e_native・kivact-test がすべて rc=0）。E24 の後の後退の確認（E24 の節）も成功。採る。
- 同じ確認を、Orchestrator が `cargo test -p kivact-engine --test content_version`（feature `testing` が要る）で再実行しようとしたが、feature の付け忘れで走らなかった（ログは `V:/kv-target/logs-W6b-E2/e14-content-version-*.log`、exit 101）。確認の結果ではない。統合の tier b（`--all-features`）で改めて通る。

## 設計者の回答（W6b の差戻し：C19、2026-11-21）

- **(1) shorthand 名の override**：(a) を採る。DevTools の override は 2b8df5ac で `background`（色）・`line_height`（数と `px`）・`radius`（数）を読めたので、これを保つ。付属書 S に追補 **S17b**（`## S18` の直前）を足した：`Background`（色だけを text から読む）・`LineHeight`（`normal`・数・`Npx`）・`CornerRadius`（1 つか 2 つの長さ）の `EditText` を書き、`shown_only!` から外す。`radius` は C19 の `SHORTHANDS` で 4 隅に展開されるので、各隅の `CornerRadius` が読めればよい。
- **(2) state layer の値**：(a) の変形を採る。新しい入口は足さず、`describe_style` の値を `Debug` から表の text の形（`Decl::edit_text`、`EditText::to_edit_text`）に変える（S17b の 2・3）。これで `describe_state_styles` も同じ形になり、base の層と state の層の値が揃う。Orchestrator の判断 (d) を満たす。値の文字列を期待する既存の試験は text の形に直してよい。
- 進め方：lane X で `79358493` の上に S17b を実施し、C19 の確認（tier a）をやり直してから S18 へ進む。

## Orchestrator の判断（W6b・C19・S17b・S18、2026-11-21）

- C19 の再確認（check・clippy・xtask・e2e_native の devtools）は、S17b の commit `f213e6d9` の上で Orchestrator が実行し、すべて通した（ログは `V:/kv-target/logs-W6b-X/c19-recheck-*.log`）。採る。
- S17b（`f213e6d9`）：付属書の試験の場所 `decl.rs` の `style_properties_round_trip` は、実際には `sheet/edit_text.rs` にあった。名前で探して、そこに足した（0.5：付属書の場所が無く、名前で同じものが一つ）。採る。e2e_native の devtools は付属書の command で 0 件（feature `devtools` の module）だったので、2.2 節の雛形どおり `--all-features` で再実行し、1 件成功。採る。
- **S18（`2214e8aa`）の試験の期待値（設計者に確認）**：付属書の `radius(Length::Auto)` の試験の期待値は 4 だが、実装は 8 にした。`corner()` が隅ごとに x と y の 2 つの長さを数えるため（ST-31 の「数えて報告する」は長さの数）。新しく書いた試験で、期待値を緩めたのではなく数え方の定義の確定なので採る。ただし 0.2 の 3 に近いので、設計者が 4 の定義（隅ごと）を選ぶなら S18 の試験と実装の両方を直す。
- S18：`misplaced` の数えは `Resolve` の既存の counter で、`Resolve::paint_len` を足した（件数は同じ）。採る。
- S18：clippy の `too_many_lines`（`bind_style` が 82 行）は `write_props` に分けた（0.4 の「意味のある helper に分ける」）。採る。
- S18：`ForcedStates` の再公開を消し、`kivact_types::StateSet` にした（ST-09 の style 側）。`pressed` の wire 名は変えていない。採る。

## Orchestrator の判断（W6b・S19、2026-11-21）

- **S19 の STOP（`crates/kivact-engine/src/model/focus/navigation.rs:change_focus`）**：許可した。`report_states(n)` が `self.focus.focused = new;` より前に呼ばれ、フォーカスの state が core に届かなかった（E2E の `state_styles::focus_applies_a_text_property` と `focus_draws_its_paint_callback` が落ちた）。0.5 節の「所有ファイルの外の 1〜3 ファイル」の行に当たる：この file は W6b のどの lane も所有しない（W4・W5a・W7b の E の所有）。報告の loop を代入の後へ移す 1 file の順序修正だけ。採る。
- **S19 の border の doc（`sheet/shorthands.rs`、「a width lays out again while the state applies, as CSS does」）**：採る。付属書 S19 の module doc の文（「border widths … are compiled into the node's style while the host reports the state, so they lay out again like CSS」）と一致する。設計者の 2026-11-21 の S17 (b) の答えの「border() の doc は S19 後の姿」は、この文を指すと読んだ。
- S19 の module doc と hover の doc の `pressed` は、S20 で `active` に直す（付属書 S20 の所有）。採る。

## 設計者の回答（W6b・S18 と S19 の確認、2026-11-21）

- **S18 の `misplaced_lengths`**：8 を採る。報告の数は長さごとに数える（S15 の `Transform` の unsupported と同じ規則）。隅は x と y の 2 つの長さを持つので、4 隅で 8。付属書 S18 の試験の文を 8 に直した。実装と試験はそのままでよい。
- **S19 の `navigation.rs` の順序**：採る。`report_states` を `focused` の書込みの後に呼ぶのは S19 の目的に要る直しで、W6b のどの lane も所有しない file なので衝突もない。

## Orchestrator の判断（W6b・S20、2026-11-21）

S20（commit `c0904ec4`、subagent の報告の J1〜J5）。

- **J1（`Composite` の型を `kivact::draw` に残した）**：採る（中間状態）。設計書 3 章の `Mix::Composite(Composite)`（`REWORK_DESIGN.md` の 336 行）は型を Mix の payload として残す。`e2e_core/compositor.rs` と `offscreen_svg.rs` がこの path を import しているので、S20 では残し、S21（`Mix` の移動）で Mix と一緒に扱う。S21 の付属書の手順で決める。
- **J2（`Offscreen`）**：採る。付属書の手順 7 は core の `draw.rs` に再公開を書くが、現状の core の `draw.rs` には無い。`kivact::draw` の `Offscreen` は facade に従来どおり置いた。
- **J3（facade の `crates/kivact/src/draw.rs`）**：採る（0.5 の「所有の外の 1〜3 ファイル」。W6b のどの lane も所有しない）。
- **J4（行末）**：採る。変更した 189 file を CRLF にそろえた（前の unit の編集で LF になった file があった）。`git diff --stat` と `--ignore-cr-at-eol` が一致する。
- **J5（DevTools の override 名の改名：`grow`→`flex_grow`、`shrink`→`flex_shrink`、`weight`→`font_weight`、`radius`→`border_radius` と四隅）**：設計者に確認する。採るが、DevTools の wire に見える名前なので、統一するかは設計者が決める。表示用の label（`kivact-core/src/devtools/node.rs` の "ring"・"radius"・"weight"）は変えていない。
- S20 の試験の名前：S17 の試験が `radius_top_left` を探していたので `border_top_left_radius` に直した（付属書の試験の文の名前の追従。期待値は変えていない）。採る。

## Orchestrator の判断（W6b・S21・S22、2026-11-21）

- **S21（`27549ceb`）**：採る。`Layer::blend`・`Layer::composite` を消し、`Layer::mix(Mix)` にした。付属書の `crates/kivact-test/src/screen/gpu_tests.rs` は無く、実際の call site は `crates/kivact-test/tests/screen/gpu.rs`（同じ意味の一つだけ）。`Composite` の path は S20 の時点で既に `kivact::draw` から公開されていた。採る。
- **S22（`b1838af2`）**：食い違い 1〜5 は採る。
  - 1（`engine/src/paint/text.rs` の 1 行）：付属書の所有に無いが、同じ crate の機械的な追従（0.2 の 11）。E2 の lane は完了しているので、統合時の衝突は W6b の merge で見る。
  - 2（`compile_span`）：付属書の `resolve::text_of`・`fill_of` が無いので、`compile_style` の結果を使った。`resolve/` は触っていない。採る。
  - 3（tnum）：`span_features()` と `TextStyle::span_decoration()` で、同じ規則を engine と core の編集で共有する。採る。
  - 4（ui の path）：`code_block/editing.rs` のみ（付属書の path は無い）。採る。
  - 5（`line_background`）：「背景を先に置き、range の style を後から merge」。range 自身の background が勝つ（元と同じ結果）。採る。
- **S22 の食い違い 6（確認事項・設計者へ）**：span の変換で捨てる Style の text 系 property（`line_height`・`text_align`・`text_shadow`・`overflow`・`transform`・caret・selection・placeholder の色など）は、`Unsupported` に数えていない。付属書どおりだが、黙って捨てる。数えるかは設計者が決める（ST-31 の「数えて報告する」との関係）。
- **S22 の食い違い 8（kit の旧い field）**：kivact-ui-basic（`code_block/highlight.rs`）と kivact-ui-soft（`rich_blocks.rs`、`code_syntax.rs`）は旧い `StyledRange` の field を使う。DEC-14 の compile の対象外なので直していない。W9 の kit の unit で直す。記録する。
- **S22 の食い違い 9（既存の別件・確認事項）**：`resolve_extras` は paragraph の features に `tabular_nums` を足しておらず、engine に `tabular_nums` の参照も無いので、paragraph の `tabular_nums()` が効いていないように見える。S22 の範囲外。未確認。設計者が見る（後で W11 の G の確認か、別の unit）。

## 設計者の回答（W7a・A13 の差戻し、2026-11-21）

- **(1) 通知の数**：付属書の手順 4 が誤っていた。`RendererChange` は「その window の frame を何で描くか」の通知なので、window ごとに出す（Orchestrator の直しの形を採る）。device が Driver に一つであることは、通知の数ではなく、各 window の `Window::gpu_adapter()` の `generation` が同じことで確かめる。付属書 A13 の手順 4 と、試験 `two_windows_of_one_driver_share_one_gpu_device`（`Gpu` は window ごとに 1 回・計 2 回、generation が同じ）、`two_drivers_have_a_device_each_and_closing_one_leaves_the_other_drawing`（generation が違う）を直した。
- **(2) 開き直した window の device**：DEC-08 が正で、試験の期待を直す。`each_window_opened_again_starts_its_own_device_on_demand` を `each_window_opened_again_draws_on_the_drivers_one_device` に改名し、generation が round ごとに同じことを期待する。main の window が CPU のままである期待は残す。そのため、付属書に手順 4b を足した：`WindowGpu` に `Waiting` を足し（4 variant）、`OnDemand` では adapter を読んだ window だけが GPU で描く（device ができても、読まない window は CPU のまま）。
- lane A1 は `8fed6148` の上で、直した付属書 A13 の 4・4b と試験に合わせ、A13 の確認をやり直してから進む。

## Orchestrator の判断（W7a・U20・U21・U25・A14・E22、2026-11-21）

- **U20（`8a96e844`）**：採る。`clippy -p kivact-ui` は arvane-native の既存の unused import（Windows の `RECT`・`GetWindowRect`）で落ちる。クリッピーは path の依存も lint するため。tier b（workspace）では通っている。`--no-deps` で対象 crate だけ見る。
- **U21（`6abda8f2`）の STOP（spinner の `reduced_period`）**：採る。4.3 の fallback 1000 にすると、reduced motion で装飾の動きが止まる挙動（None）が変わる。`reduced_period(Option<f64>) = None` の setting にして、挙動を保つ。設計者に確認：付属書 4.3 の spinner の行。
- **U21 の食い違い（設計者に確認）**：(a) combobox の `FOCUS_RING` は 4.3 の fallback 2 に従った。旧は theme の `focus_ring()` を読んでいた。テーマで focus ring を変えた利用者は、combobox の見た目が変わる。(b) tree の `RADIUS` は SM のまま（見た目を変えない。4.3 は MD）。(c) tree の試験は `INDENT` でなく `PADDING` を確かめる（付属書の INDENT は 16 固定）。(d) spinner に `GAP`（6）を足した（`indicators.gap` も読むため）。採る。
- **U25（`e2e0b710`）**：採る。`#[caller]` の binding は、書かれた型（`Caller`）を使う（試験の `use` に合わせる）。`Children` の試験は付属書の形（`shelf((depth_text(),))`）。`absorb` の disabled の OR は設計書 5.1 の規則 2 どおり。挙動が変わる（本体と呼出し側の両方が disabled を書く部品）ので、E2E は component_macro だけを走らせた。他の試験は tier b で確かめる。
- **A14（`f5382490`）**：採る（付属書との食い違い 1〜8）。`Lent::drop` は常に `give_back`、`forget` は `Session::dispose` の panic 分岐。`State::stop` は `needs_cleanup()` を使う（`accepts_callbacks()` では片付かない）。`stopped_outside_callback` は win32 と x11 で呼ぶ。`Child::stop_idle` の `Calling` の扱いは付属書の表に無い（記録）。Linux の check は成功（x11 の E2E は未実行。WSLg で確認が要る）。
- **E22（`750ad059`）**：採る。`Writes` の印は xtask が未使用と判定したので外した（0.5 の行）。`kivact-engine --features testing` の DLL 初期化の失敗（0xc0000142）は、再実行では出なかった（環境の一時的な失敗として記録）。`e2e_native --all-features -- devtools::` は 1 件成功。

## Orchestrator の判断（W7a・A13・U22・E23、2026-11-21）

- **A13（`3a719498`、設計者の答えに従う続き）**：採る。`Start{gpu}` は window を開いたとき `device_started()` で決める（OnDemand で device が既にある時に、開いた window の試験の round 2 が GpuStarting を見られるように）。shader fallback と slow frame の開始も、その window を Waiting にする（起動した device を使えるように）。他の window の device 喪失の `Cpu` は、event の終わりに `sync_gpu` で気づいた時に汎用の理由で出す。`Gpu{..}` は window の context が ready になるたびに出す（restart の後も）。`ReadyDevice` に `init` を足し、`WindowGpu::advance` は `Option<&ReadyDevice>` を受ける（付属書の sketch では configure の wake と init が取れないため）。すべて付属書の意図どおり。採る。ARV・GPU-DEMAND・GPU-SHARE・EMBED・LIFE は、CPU の形と GPU の形の両方が通った（報告）。
- **U22（`485acfee`）**：採る。入力部品の constructor は名前を取らず、`label(..)` が builder のメソッドになった（DEC-05・API-17・UI-13 (a)）。`TagInput::on_add`・`on_remove` は U22 で消した（dead_code のため）。`NumberInput`・`TagInput` の名前は root node に持つ（xtask の state 規則のため）。`date_range_input`・`date_time_input` の `label()` の既定は既存のメッセージ（U22 の判断。設計者に確認：既定の文言）。`Look3::Plain` は component_macro.rs から消した（HEAD で既に warning）。clippy は `--no-deps`（arvane-native の既存の warning を避けるため）。
- **E23（`6b66158b`）**：採る。付属書の手順 6 の Clone の cfg_attr に加え、`ChildCursor::forget`・`LayoutState::clear_relayout` に cfg を付けた（dead_code を避けるため。どちらも `rebuild_all` からだけ使う）。`Calcs` の Clone は手順に無いので残した。engine の testing（全件）・E2E（foreign_window 11・layout 8・layout_check 5・text_editing 19）は通った。
- **E22 の完了後の確認**：`rg dead_code` の 2 file は、E23 の確認で空。

## 差戻し（W7a・U23、2026-11-21）

U23（lane U1、commit `571ec07a`）は未完。次の 3 件は設計の判断が要る（0.5 の表に当たらない）。U1 lane はここで止める（U24 は U23 の後）。

- **(1) `ColumnDef::with_width` の改名**（`crates/kivact-interaction/src/table/model.rs:132`）：付属書 U23 の手順 4 は `with_width` を `width` に改名すると書くが、同じ型に既存の getter `pub fn width(&self) -> Option<f64>`（model.rs:187）がある。衝突する。呼出し元は `crates/kivact-ui/src/data_table/column.rs:168,177`。設計者が決める：getter の扱い（改名の名前を変える、または getter を別の名前にする）。
- **(2) Stepper の state 印（`crates/kivact-ui/src/stepper/view.rs:112`）**：付属書 U23 の手順 5 で `stepper(items, current: Signal<usize>)` にしたため、`Stepper` の field（`current`・`min_step_width`）が mutate される型として xtask の state 検査が error を出す。`cargo xtask check --no-write --crate kivact-ui` が 1 error で落ちる。0.5 の表の行（印の文を付ける条件 3 つ）は満たすと見えるが、印を入れる編集が権限の自動判定で拒否された。回避はしていない。Orchestrator が印を入れる許可か、差戻しのままにするかを設計者が決めてください。
- **(3) `NavigationMenuLink::on_press`**（`crates/kivact-ui/src/navigation_menu/link.rs`）：付属書 U23 の手順 5 は固有の method を消すと書く。だが `into_view` が同じ node の `on_press` で閉じる処理（`context.close`）を足し、`Node::on_press` は既存の handler を置き換える（`node_modifiers.rs` の doc）。利用者の押下と閉じる処理を両方載せられない。固有の method を残した。設計者が決める：閉じる処理を別の経路（`Node` の追加の handler、または `Callback` の合成）へ移すか、付属書の形を変えるか。

U23 の報告にある食い違い 1〜15（ComboboxApi::mode が無い、`Stepper::disabled` が無い、TextEditApi の `on_draft_commit` は改名の対象外、`lib.rs` の表は一部だけ更新、Avatar の press、ChatMessage・Card・Badge・ItemRow の押下が root に載る、Stepper の項目は常に押下可能、icon_button の Ghost、button の group の角、input_group、試験の改名と追加、`#[track_caller]` の 53 file、未使用の印の削除）は、Orchestrator が読んだ。判断は U23 の後の確認で見る。ただし、食い違い 6（押下が body でなく root に載る）と 7（Stepper の項目が常に押下可能）は、利用者の挙動が変わるので、設計者の確認が要る。

## 設計者の回答（W7a・U23 の差戻し、2026-11-21）

- **(1) `ColumnDef::with_width`**：付属書どおり builder を `width` にし、getter を `preferred_width` に改名する（使うのは `data_table/column.rs` の 1 箇所）。付属書 U23 の手順 4 に足した。
- **(2) Stepper の state 印**：印の行は 0.5 の表どおりでよい。ただし Orchestrator の権限の判定で拒否された編集なので、設計者が代わりに入れることはしない。利用者に判断を求めた（利用者の許可が出たら設計者が入れる）。それまで U23 の xtask の 1 件は未解決のまま残し、ほかの直しを先に進める。
- **(3) `NavigationMenuLink::on_press`**：固有の method は消す（API-13 のとおり。固有の method を残すと、`Element::on_press` で付けた handler が `into_view` で置き換えられて消える）。core に `#[doc(hidden)] Node::take_press` を足し、link は呼出し側の handler を取り出して自分の押下の中で呼んでから閉じる。付属書 U23 の手順 5 に書き、試験を 1 つ足した。
- **食い違い 6（押下が root に載る）**：採る。`Element` の modifier は root に付く（`element_data.rs` の doc の決まり）。Card・ChatMessage・Badge・ItemRow の全体が押せる面になり、`use_button()` の part も root に付く。G の移行の文書に 1 行足す（W11b）。
- **食い違い 7（Stepper の項目が常に押せる）**：採る。値は `current` の signal が持ち、利用者の操作はそれを書く。飛ばせない手順の項目は `StepperItem::disabled(true)` にする。`stepper` の doc に 1 文足す（付属書の手順 1）。G の移行の文書に 1 行足す（W11b）。

## 差戻し（W7a・E25、2026-11-21）

E25（lane E1、作業ツリーの未 commit の 6 file。HEAD は `6b66158b`）は、確認で 1 件が落ちた。lane E1 はここで止める（E26 は E25 の後）。

- **落ちた試験**：`dev/tests/kivact-public-api/tests/e2e_core/custom_text_field.rs:285`（`custom_text_field_accessibility_publishes_text_and_selection`）。`field.reconvert(1..3)` の直後の `assert_eq!(field.text_selection(), Some((1, 3)))` が、`Some((14, 14))` になる。a11y の選択範囲が、操作の直後に publish されない。
- **原因（見立て）**：付属書 E25 の EN-27（「文字入力の入口は sync せず、続く pump の settle に任せる」）に従い、`model/keys.rs`（`key`・`paste`）と `model/edit/ime.rs`（`preedit`・`commit`・`ime_select`）から `sync()` を外した。`reconvert` の経路がその `sync()` に依っていた。試験は pump を挟まず a11y を読む。
- **設計者が決める**（0.5 の表に当たらない。a11y の期待値は差戻し対象）：(a) `ime_select`（または `reconvert` の経路）の `sync()` を残す（E25 の手順 3 の一部を取り消す。試験の期待値は変えない）、(b) 試験の前に pump を挟む（試験の期待値を変える。設計者の承認が要る）、(c) EN-27 の文を直す。
- 作業ツリーの変更は reset せず残してある（lane E1 の `W:/w/kv-wt/rw-W7a-E1`）。E25 の他の確認（engine の試験 13 件、E2E の他の 11 module、fmt、clippy、xtask）は通った（`V:/kv-target/logs-W7a-verify/e1-*.log`）。

## 設計者の回答（W7a・E25 の差戻しと、確認の残り、2026-11-21）

- **E25**：(a)・(b) は採らない。EN-27 と E25 の早い return（`is_clean`）は、「書込み・入力・tick・resize・asset は、すべて一つの `Changes` に記録される」（`sync.rs` の module doc）を前提にしている。試験が落ちたのは、reconvert の経路のどこかが `Changes` に記録せずに状態を変え、前は無条件の `sync` がそれを拾っていたためと見る（隠れた通り道。root で直す）。次の順で調べて直す（subagent に任せてよい）：
  1. 手順 3 の早い return だけを一時的に外して試験を走らせる。通るなら、早い return が仕事を飛ばしている。reconvert（`ime_select`・`session_select`・`session_event` と、それを受けた App の commit）の後に、`sync` のどの段が a11y の選択範囲を出しているか、その段が読む状態を誰が書いているかを突き止め、その書き手が `Changes` に印を付ける（その段が既に使う `Dirty::` の bit か `Changes` の field。新しい field は作らない）。早い return は戻す。
  2. 早い return を外しても落ちるなら、手順 7 の `sync()` の削除が原因（試験の host の経路で、入力の後に settle が走っていない）。その場合は、落ちる入口の関数だけ `self.sync();` を残し、付属書 E25 の手順 7 と EN-27 の文をその入口を除く形に Orchestrator が直す（設計者の決まりとして記録する）。
  どちらでも試験の期待値は変えない。E25 の確認を全部やり直してから commit し、E26 へ進む。
- **J5（DevTools の override 名）**：採る。DevTools の名前は style の setter の名前（S20 の CSS の名前）に揃える。wire は rework の中で互換を約束しない。表示用の label（`node.rs` の "ring"・"radius"・"weight"）も同じ名前にする：W11c の後片付けの unit に入れる（設計者が付属書に書く）。
- **S22 の食い違い 6**：数える（ST-31 の「数えて報告する」が正）。span の変換で捨てる text 系の property を `Unsupported` に数える unit を、設計者が W9 の前に付属書 S に足す（S22b）。それまで今の形でよい。
- **S22 の食い違い 9（paragraph の `tabular_nums`）**：設計者が調べ、要るなら同じ時に unit を足す（S22c）。
- **U21 (a)〜(d)**：採る。(a) は 4.3 の fallback の規則どおり。G の移行の文書に「combobox の focus ring は theme の `focus_ring()` でなく token の `FOCUS_RING`」を 1 行足す（W11b）。
- **U22 の `date_range_input`・`date_time_input` の既定の label**：既存のメッセージのままでよい。
- **push**：W6b までの統合 branch は、利用者の許可を得て設計者が push した（`b1b2c3c2`）。Orchestrator は push しない。
- **W6b の worktree と branch の後片付け**：権限の判定で拒否された操作なので、利用者の判断を待つ。それまで残す（作業の妨げにならない）。

## Orchestrator の判断（W7a・E25 の直し、W7b・A15、2026-11-21）

- **E25（`f0ed653b`）**：採る（設計者の決め、手順 2）。`ime_select` の末尾の `self.sync();` を戻した。試験の host の経路（`ime_event` → `ime_select`）で settle が走っていなかった。`preedit`・`commit`・`key`・`paste` の sync は外したまま。早い return は手順 1 の結果で戻した（外しても試験は落ちた）。付属書 E25 の手順 7 と EN-27 の文に追補を書いた（`E-engine.md`、本 commit）。確認（engine の試験 216 秒、E2E 12 module、fmt、clippy、xtask）は全部通った。
- **A15（W7b、`08c1371d`、lane `wt/rw-W7b-A1`、W7a の A13 `3a719498` の上）**：採る。`CpuCompositor` の target の pool（64 MiB）、`filters::apply` の借用（input と output が同じ時だけ clone）、GPU への切替えでの `invalidate`。付属書との食い違い 1〜6（`run` は `size` を取らず `output.size()`、`give_back` は Temp だけ、`render()` は local の pool、試験の pool の非空の assert、E2E は comment 変更の前に走らせた、CRLF の復元後は check・clippy・xtask・compose の試験を再実行）は、配置と同じ形の選択で設計の変更は無い。E2E（ARV 38、DRAW 83、OFFSCREEN 4+8、GPU-DEMAND 6）は通った。
- **W6b の後片付け**：設計者が利用者の許可を得て実施（worktree と branch の削除）。W6b の target（`V:/kv-target/rw-W6b-*`）は残してある。
- **U23（`e23f39a8`）**：設計者が Stepper の record の印を入れた（利用者の許可）。U23 の完了 commit は U1 lane で行う。

## Orchestrator の判断（W7a・E26、U23 の完了、2026-11-21）

- **E26（`935ea765`、lane `wt/rw-W7a-E1`）**：採る。`HostNode` の `scroll`・`shift` を `NodeTree` の疎な表へ移した（`size_of <= 280` の assert は通る）。付属書との食い違い 1〜4（`report_geometry` の読む順は `ScrollReport` の値を変えないため、`children_frame` の未使用の引数を消した、CRLF の復元、`scroll_of` は既定値を返す。`?` の所は `self.tree.get(p)?;` で Option の意味を保った）は、配置と同じ形の選択で、振る舞いは変えない。確認（engine の試験 13 件、E2E の e2e_core 10 module と e2e_ui 3 module、fmt、clippy、xtask、check）は通った。
- **U23（`8270c564`、lane `wt/rw-W7a-U1`）**：採る。設計者の印の commit `e23f39a8` が `stepper/view.rs` を CRLF の blob で入れて、親との diff が全行になっていた。`git add --renormalize` で index を LF に戻した（内容は `e23f39a8^` との 1 行の追加のまま）。clippy は `--no-deps`（arvane-native の既存の warning のため）。xtask（`kivact-ui`・`kivact-interaction`）は error 0。e2e_ui 154・16 件、e2e_core view 11 件は通った。
- **U23 の完了条件の rg の残り（`TextEditApi::on_draft_commit`、`crates/kivact-interaction/src/text_edit/handle.rs:218`）**：受け入れる（0.5 の「目的・消すものに挙がらない既存の行」）。付属書の手順 1 は ui の `TextInput` の builder だけを改名すると書いており、interaction の method は所有の外。設計者が後で付属書に書くかを決める。

## 差戻し（W7a・U24、2026-11-21）

U24（lane U1、commit `c691dd01`）で、付属書の `Element::variant` に移せない builder が 5 つある。U1 lane はここで止める（W7b の U 側は U24 の後）。

- **対象**：`crates/kivact-ui/src/{dialog/view.rs:Dialog, menu/view.rs:Menu, toast/view.rs:ToastRegion, hover_card/view.rs:HoverCard, command_palette/view.rs:CommandPalette}`。これらには `impl Element for` が無く、node を持たない（付属書 U24 の手順 3 は `Element::variant` に移すと書く）。
- **いまの形**：inherent の `pub fn variant` と `variants` の field を残し、型だけ core の `Variants` にした（`std::mem::take(..).with(v)`）。
- **設計者が決める**：(a) この 5 つに node を持たせる（`impl Element` を足す）、(b) W7b の U30a・U30b・U30c で作り直すまで inherent のまま残す、(c) 別の形。
- 同じ報告の食い違い（判断済み）：1 の `Variants::merge` を `#[doc(hidden)] pub` にした（継承に必要。`Node::authored` と同じ前例）。3 の `Prop` derive の追加（`IntoValue<V>` に要る）。4 の `Look` の derive を `Clone` だけに。5 の tooltip の variants は trigger の node から読む（U47 の注記どおり）。6 の `cargo fix` で unused import を消した。7 の完了条件の `rg "variants: Variants"` は、内部 struct の field 21 件を数える（builder ではないので、0.5 の「目的に挙がらない既存の行」として受け入れる案）。
- xtask（報告だけ）：`--strict --only element-shadow,hand-element` は exit 1、`hand-element` 130 件（W8 で消す）、`element-shadow` 0 件。結果は PROGRESS に書いた。

## 設計者の回答（W7a・U24 の差戻しと E27 の性能、2026-11-21）

- **U24 の 5 つ（Dialog・Menu・ToastRegion・HoverCard・CommandPalette）**：(b) を採るが、待つ先は W7b の U30a〜c ではなく W8 の U47・U48（この 5 つを `#[component]` にして root の node を持たせる unit）。U30 は開閉の語彙を直す unit で、node を持たせない。それまで固有の `variant` と `variants`（core の `Variants`）を残す今の形でよい。付属書 U の「U44〜U54」の節に、U47・U48 がこの 5 つの固有の `variant` を消す手順を足した。U24 の完了条件の `rg` に当たるこの 5 つの field は、0.5 の「目的に挙がらない既存の行」として受け入れる。U24 を完了してよい。
- **U24 の食い違い 1・3〜7**：採る（Orchestrator の判断どおり）。
- **E27 の性能の注意**：確かめる試験を E27 に足す。engine の試験（`crates/kivact-engine/tests/` の E27 が足す試験の file）に `fn a_drawable_geometry_write_leaves_no_report_pending()`：drawable の geometry を書いて（`SetDrawableGeometry`）commit・wake・redraw した後、もう一度 wake したとき sync が何もしない（`Changes` が空。試験の hook が無ければ、二度目の wake の後の `paint_list_sizes` と frame の数が変わらないことで確かめる）。落ちたら、`settle` で `Dirty::REPORT` の印が残っているときは、layout が走らなくても `report_geometry` を呼んで印を取る形に直す（印を残したまま次の sync へ持ち越さない）。E27 の 1 件の試験の落ち（`scroll_state_follows_the_scroll_position`）は、Orchestrator の見立てどおり印の付け漏れを lane で直してよい（期待値は変えない）。

## 差戻し（W7a・tier b、2026-11-21）

W7a の 4 lane を統合 branch に merge した（HEAD `fd0acf24`）。tier b（W6〜W9 の形）：fmt・clippy（workspace、XK と XD を除外、`-D warnings`）・test（1324 件成功、失敗 0、無視 39）は成功。`xtask-allow(state-fields)` は 0 件。**xtask `--only local` が 5 error で失敗**：

- `dev/xtask/src/name_collision.rs` の規則：kit の公開名が、core・std の prelude の名前と衝突しない。予約名には `kivact::prelude::*` と `kivact::ui::*` の最後の名前が入る。
- W7a の U21・U22 で `kivact::ui` に `theme` の module（`pub mod theme`、`theme.rs`）が出来たので、`theme` が予約名になった。kit（`kivact-ui-basic`・`-creative`・`-game`・`-pop`・`-soft`）は `pub use theme::theme;`（`lib.rs` の 159・105・69・73・227 行）で `theme` を公開しており、衝突する。
- kit は W9 の K 系の unit で移行する（コンパイル対象外。DEC-14）。ただし xtask の規則は kit も見る。
- 設計者が決める：(a) kit の `theme` の公開を、W9 の K unit で改名する（tier b の xtask は W9 まで kit を除外する形にする）、(b) `kivact::ui::theme` の module の名前を変える（U21 の付属書の変更。公開名）、(c) この規則の予約名に `kivact::ui::*` の module 名を入れない（xtask の規則の変更）。
- W7b は kit に依らないので、この答えを待たずに進める。W7b の lane は統合の HEAD から作る。

E27（W7b・E1）：`198ec9e5`（REPORT の印を settle で取る。試験 `a_drawable_geometry_write_leaves_no_report_pending` は通る）。計測で、修正前は 19 回の settle のうち 9 回で印が残り、修正後は 0 回。採る（設計者の決め）。

## 設計者の回答（W7a・tier b の name-collision、2026-11-21）

- (c) を採る。ただし「`kivact::ui::*` を予約名から外す」のではなく、**core の module の名前を予約名にしない**形にする。検査の目的は「kit を `kivact::prelude` と並べて glob で import しても名前が曖昧にならない」ことで、module は型の名前空間、kit の `pub fn theme` は値の名前空間にあるので、Rust では曖昧にならない。付属書 K は W9 の後も kit に `pub use theme::theme;` を残すので、(a) では直らない。(b) の `kivact::ui::theme` の改名は要らない。
- 直し（Orchestrator が統合 branch で、Haiku high の subagent に任せてよい。所有：`dev/xtask/src/public_surface.rs`、`dev/xtask/src/name_collision.rs`）：
  1. `public_surface.rs` の `pub(crate) struct Surface` に `pub(crate) modules: BTreeSet<String>`（doc：「The public paths that name a module.」）を足し、`build_for` の `if modules.contains(&target) {` の中で `surface.modules.insert(path.clone());` を足す。
  2. `name_collision.rs` の `reserved` を作る loop で、`surface.modules.contains(path)` の path は飛ばす。module の doc に 1 文足す：「A core module's name is not reserved: a module is in the type namespace and does not make a kit's function or value of the same name ambiguous.」
  3. `mod tests` に `fn a_core_module_name_is_not_reserved()`：kit の source `pub fn theme() {} pub struct Button;` と、surface の `exports` に `kivact::ui::theme`（`modules` にも入れる）と `kivact::ui::Button` を入れたとき、error が 1 件（`Button` だけ）。
  4. 確認：xtask の package の `cargo test`（`name_collision` と `public_surface`）、`cargo xtask check --no-write --only local` が通る。それから W7a の tier b の xtask を走らせ直す。
- W7a の tier b の他（fmt・clippy・test 1324 件）は成功として採る。xtask が通れば W7a を完了してよい。

## 差戻し（W7b・U30a、2026-11-21）

U30a（lane U1、commit `fbad507e`、`wt/rw-W7b-U1`）は、`kivact-ui` が compile しない状態で commit されている。U1 lane はここで止める（U30c・U30b・U31・U33 は U30a の後）。

- **原因**：付属書 U30a は `DialogState.kind` が `DialogKind` だと書くが、実物は `DialogMode` のまま（U23 の残り）。U30a は `DialogMode::Popover` を消したので、popover の状態を `DialogState` で表せない。`crates/kivact-ui/src/dialog/view.rs:137`（`Shape::state` の Popover の腕）に印（コンパイルエラー）を置いて止めた。
- **直すのに要る所**：`crates/kivact-ui/src/dialog/look.rs`（`DialogState.kind` の型）と、`crates/kivact-ui/src/dialog/drawer_swipe.rs:125` の 1 行。後者は U2 lane（U32、commit `459b6715`）の所有で、0.5 の「他の lane の所有の外 1〜3 file」の行には当たらない（同じ wave の他の lane の所有）。
- **設計者が決める**：(a) `drawer_swipe.rs:125` を U1 の lane で 1 行直す（U2 の file。merge の衝突に注意）、(b) U2 の lane で先に直す（U32 の追補）、(c) `DialogMode::Popover` を残す（U30a の付属書の変更）、(d) 別の形。
- U32 の報告の食い違い（判断済み）：U32 は付属書の `toast.rs`・`empty_state.rs` を実際の `toast/view.rs`・`empty_state/view.rs` に当てた（同じ module の中）。試験の import は付属書と実物が違い、`kivact::motion::{Easing, Transition}` を足した。

U34・U35（W7b の U3）は完了、U32（U2）は完了、U36（U4）は ui の試験を確認中、E27・E28（W7b の E1）と A15（W7b の A1）は完了。

## 設計者の回答（W7b・U30a の差戻し、2026-11-21）

- (d) を採る。U23 の残り（`DialogState.kind: DialogKind`）を U30a の中で済ませ、U2 の 1 行は wave の merge で直す。
  1. lane U1（U30a の続き）：`crates/kivact-ui/src/dialog/look.rs` の `DialogState.kind` を `crate::DialogKind`（ui の型）にし、`use kivact_interaction::DialogMode;` を消す。look.rs の中の `DialogMode::Drawer(..)` などは `DialogKind::Drawer(..)` にする。`dialog/view.rs` の `Shape::state` の変換の `match`（STOP の印の所）は消し、`kind` をそのまま入れる。ui の中で `DialogMode` を使うのは、render が `DialogKind` を interaction の `DialogMode`（`Popover` の無いもの）に変える所と popup の分岐だけになる（付属書 U30a の前提の文のとおり）。U1 の lane の中の他の `DialogMode` の `DialogState` 用の書込みも同じく直す。check と U30a の確認をやり直して commit する。
  2. lane U2 の `dialog/drawer_swipe.rs` の `kind: kivact_interaction::DialogMode::Drawer(edge),`（U32 で入った行）は U2 の lane では直さない（U2 の lane では look.rs がまだ `DialogMode` なので、直すと U2 が compile しない）。W7b の merge で U2 を入れた直後に、統合 branch で `kind: crate::DialogKind::Drawer(edge),` の 1 行に直し、`cargo check -p kivact-ui --all-targets` を通してから次の lane を merge する（merge の直しとして PROGRESS に記録する）。
- U32 の食い違い（file の置き場、試験の import）は採る。

## 設計者の追加（W8 の lane S：S22b・S22d、S22c の確認、2026-11-21）

- 付属書 S に **S22b**（span が運べない text の宣言を `compile_span` の数に足す、ST-31）と **S22d**（DevTools の computed の名前を style の名前に揃える、J5）を足した（`## S23` の後）。W8 に lane S を一つ足し、S22b → S22d の順で行う。所有は他の W8 の lane と重ならない（`kivact-style/src/internal.rs`、`kivact-core/src/devtools/node.rs`）。W8 の merge の順では lane S を最後にする。
- **S22c**：paragraph の `tabular_nums` は効いている（`resolve_extras` → computed の継承 → `arvane-text` の `tnum`）。S22 の食い違い 9 は閉じる。unit は無い。
- W11c の「表示用の label」の項（J5）は S22d に移した。
