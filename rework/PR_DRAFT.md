# PR の説明の下書き（Kivact 抜本改修、`refactor/kashiwade/fundamental-rework`）

> 下書き。PR は作っていない（作るのは設計者）。CP3 は未完了：AC-2（KPI の必達）が未達で、判断を待つ行がある（下の「未達」）。

## 目的

最近の規律のない機能追加で増えた複雑さを整理し、設計書（`REWORK_DESIGN.md` 1 章）の 6 つの基準で Kivact を作り直す。公開 API は壊す（利用者は移行ガイドで追従する）。性能は KPI（設計書 2 章）で測り、後退させない。

## 変更の要約（領域ごと）

- 中核（kivact-core・kivact-runtime・kivact-engine・kivact-layout）：状態・effect・task の土台を整理し、公開 API を一つのことに一つの書き方にそろえた。DevTools の hook と engine の性能（描画・時計・祖先の扱い）を直した。配置の移動と layout の整理を行った。
- 見た目（kivact-style・kivact-svg・kivact-types・arvane-*）：style の型を統合して改名し、theme の循環（sheet と theme、`StyledRange` の所在）を解いた。色は token と theme 変数に寄せた。Arvane の描画と文字の経路を整えた。
- 部品と kit（kivact-ui・basic・pop・game・creative・soft・各 catalog）：部品を `#[component]` の関数にし、kit を新しい API に移した。interaction の破壊的な改修（drawer の `on_open_change` の削除、`on_draft_commit` → `on_commit`、ZoomPan への改名）を行った。
- 開発の道具と文書（xtask・dev/tests・docs・CONTRIBUTING）：xtask の state・module 規則を追加し、報告だけの検査（pending）を消して全検査を error にした。e2e を binary ごとに分けた。利用者向けの文書を新しい API に書き直し、移行ガイドを足した。

## 公開 API の破壊的変更の一覧（移行ガイド）

移行の表：`docs/for-kivact-user/en/how-to-guide/migrating-to-rework.md`（日本語は `jp/`）。主な変更：

- `kivact_style::text::StyledRange` の path を削除。正の path は `kivact_style::StyledRange`（facade の `kivact::style::StyledRange` は変わらない）。
- `kivact_types::ViewTransform` を `ZoomPan` に改名（`kivact::interaction::ZoomPan`）。旧名と別名は残さない。
- `#[component]` の素の整数の prop は、書いた型のまま受ける（`u32` などは literal がそのまま通る。`progress_ring(3, 10)` が通る）。浮動小数は今のまま `impl Into<T>`。
- `TextEditApi::on_draft_commit` を `on_commit` に改名（UI-13）。
- soft の drawer の `on_open_change` と `on_change` の field を削除。変化は `.open(signal)` の signal を読む。
- 他の旧名（`Binding`・`Reactive`・`UiRecipes`・`DialogMode::Popover` など）は、移行の表のとおり新しい名前に移る。

## KPI の比（`rework/kpi/`）

基点 `c9518946`（`B0-CP3*`）と統合 `1ad8547c`（`CP3*`）を続けて 3 回ずつ測った（`cargo xtask kpi-compare`。判定は 3 回中 2 回。K6 の binary の大きさは 1 回で判定）。比は「checkpoint ÷ 基点」。必達は後退の上限、改善は改善の必達、目標は記録のみ。

| 行 | 比 r1 | 比 r2 | 比 r3 | 必達 | 改善 | 判定（3 回中 2 回） |
|---|---|---|---|---|---|---|

K2 の診断（kv_heap の site）の `arvane_shell::a11y::state::AcceptedTree` は、1 回目の組は両方に 4 件（一致）、2 回目と 3 回目の組は両方に 0 件（一致）。組の中で一致しているので測り直しは不要と判断した。

## 実施した検証（統合 `1ad8547c`、`V:/kv-target/logs-CP3/`）

- `cargo fmt --all -- --check`、`cargo xtask check --no-write`（ok）、clippy（workspace、all-targets、all-features、`-D warnings`）：すべて成功。
- test（workspace、`gpu-interop-producer`・`kivact-docs` を除く、all-features）：1762 passed、0 failed。`gpu-interop-producer`（`--test-threads=1`）：成功。
- `cargo check -p kivact-docs --all-targets --all-features`、kivact-docs の doctest（`--all-features`）：62 passed、0 failed（全体の確認は 71 passed）。
- test（workspace、`--no-default-features`）：1721 passed、0 failed。
- GPU の e2e（`e2e_core`・`e2e_native`、`KIVACT_TEST_GPU=1`）：504 passed、4 failed、1 ignored。失敗の 4 件（`app_handle::close_during_setup_exits_after_the_first_turn`・`compositor::idle_window_redraws_whole_after_release`・`frame_sources::incremental_frames_match_a_fresh_render`・`layout_scroll::a_long_list_draws_what_reaches_its_viewport_from_rows_outside_it`）は、基点でも失敗する（`rework/OPEN.md` の「基点の失敗」）。
- GPU の screen 試験、`dev/tools/check-features.sh`、`--no-default-features --lib` の check（kivact・kivact-offscreen・arvane-render・arvane-compose・kivact-style）、wgpu・naga が kivact の no-default-features の依存に無いこと：すべて成功。
- AC-3：`legacy:` の印が 0 件、`xtask-allow(state-fields)` が 0 件、xtask の state 検査が全 workspace で 0 件。
- AC-4（作業指示書 2.5 節の検査、`V:/kv-target/logs-CP3/ac4-final.out`）：231 行、すべて (i)〜(vii) の分類に当たる。分類の全件は `rework/PROGRESS.md` の「W11c・CP3 の結果」の節。

## 未実施の検証とその理由

- Linux の画面の試験（WSLg）は、この CP3 の検証には含めていない。
- AC-2（KPI）：後退 4 行（K2 の 3000 rows idle WS・per_item box・per_item button・per_item row）と、K6 の basic-catalog の 2 行（binary の大きさ）が後退。改善の必達の未達は、K2 per_item text・K3 3000 rows continuous CPU・K4 3000 rows work p95（3 行）。設計者の判断を待つ。
- GPU の e2e の 4 件は基点でも失敗する環境の問題（上）。

## 残る制限（設計書 13 章。本改修では行わない）

- a11y の Metadata の経路の増分化（EN-10 の M の部分）。
- 折り返す editor の行の splice（EN-12 の一部）。
- layout の長さの `em` が祖先の font size を継ぐこと（今回は要素の style か `font::SIZE` に対して解決する。engine の計算済みの text style を layout に渡す改修は後続。ガイドに制限を書く）。
- `lower`・`text_input` の crate 化（狭い API ができたら）。
- 色の f32 化（ST-24）。
- drop target、combobox の trigger 文字、audio control の kit の統一（作者の判断）。
- Windows の CI、`architecture/` の図の公開（CI の artifact や Pages）。
