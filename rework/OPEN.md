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
