# PR の説明の下書き（Kivact 抜本改修、`refactor/kashiwade/fundamental-rework`）

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

他の処理が動いていない別マシン（i9-14900K、Windows 11）で、基点 `c9518946`（測定用の `cargo xtask kpi-run` を足した `e99e222d`）と統合 `dfd4c326` を交互に 3 回ずつ測った（`cargo xtask kpi-compare`。判定は 3 回中 2 回）。比は「改修後 ÷ 基点」。必達は後退の上限、改善は改善の必達。

| 行 | 比 r1 | 比 r2 | 比 r3 | 必達 | 改善 | 判定（3 回中 2 回） |
|---|---|---|---|---|---|---|
| K1 kv_startup main_to_first_present | 0.925 | 1.046 | 0.960 | ≤ 1.05 | - | 合格 |
| K1 basic catalog startup | 1.016 | 0.981 | 1.018 | ≤ 1.05 | - | 合格 |
| K2 counter idle WS | 0.986 | 0.985 | 0.986 | ≤ 1.02 | - | 合格 |
| K2 3000 rows idle WS | 0.956 | 0.960 | 0.960 | ≤ 1.02 | - | 合格 |
| K2 per_item box | 0.983 | 0.952 | 0.966 | ≤ 1.02 | - | 合格 |
| K2 per_item text | 0.993 | 1.029 | 1.000 | ≤ 1.02 | ≤ 0.95 | 改善未達 |
| K2 per_item button | 0.960 | 0.955 | 0.961 | ≤ 1.02 | - | 合格 |
| K2 per_item row | 0.943 | 0.944 | 0.951 | ≤ 1.02 | ≤ 0.95 | 合格 |
| K3 counter idle CPU | floor | floor | floor | ≤ 1.05 | - | 合格 |
| K3 3000 rows continuous CPU | 0.000 | floor | 0.000 | ≤ 1.05 | ≤ 0.85 | 合格 |
| K4 3000 rows work p95 | 0.814 | 0.921 | 0.411 | ≤ 1.05 | ≤ 0.80 | 改善未達 |
| K4 3000 rows work p99 | 0.722 | 0.805 | 0.529 | ≤ 1.05 | - | 合格 |
| K4 3000 rows interval p99 | 0.969 | 0.933 | 0.967 | ≤ 1.05 | - | 合格 |
| K5 pointer p95 n3000 | 0.565 | 0.535 | 0.466 | ≤ 1.05 | ≤ 0.85 | 合格 |
| K5 wheel p95 n3000 | 1.032 | 1.062 | 0.886 | ≤ 1.05 | - | 合格 |
| K5 row update p95 n3000 | 0.284 | 0.316 | 0.352 | ≤ 1.05 | - | 合格 |
| K5 unrelated text update p95 n3000 | 0.660 | 0.689 | 0.780 | ≤ 1.05 | - | 合格 |
| K5 mount n3000 | 0.934 | 0.954 | 0.946 | ≤ 1.05 | - | 合格 |
| K5 counter click p95 | 0.849 | 0.874 | 0.869 | ≤ 1.05 | - | 合格 |
| K6 kv_startup no-gpu | 0.994 | 0.994 | 0.994 | ≤ 1.02 | - | 合格 |
| K6 counter default | 0.998 | 0.998 | 0.998 | ≤ 1.02 | - | 合格 |
| K6 basic-catalog default | 1.039 | 1.039 | 1.039 | ≤ 1.02 | - | 後退（部品が増えた分。後退としない） |
| K6 basic-catalog no-gpu | 1.043 | 1.043 | 1.043 | ≤ 1.02 | ≤ 0.97 | 後退（部品が増えた分。後退としない） |

後退は K6 の basic catalog の大きさだけで、catalog に載る部品が増えた分のため後退としない（kivact 自身の大きさの `kv_startup` は 0.994〜0.998）。改善の必達に届かない行は K2 per_item text（生きた heap は 2971 → 2691 B/item に減ったが、working set は code page の分で横ばい）と K4 3000 rows work p95 の 2 行、および K6 basic-catalog no-gpu の改善（≤ 0.97）で、どれも後退ではない。3000 行の静止画面の private bytes は 86 → 81 MB、row の heap は 23453 → 21496 B/item。

## 実施した検証

最終 `dfd4c326`（部品の境界の node を外した後）：`cargo fmt --all -- --check`、`cargo xtask check --no-write`、clippy（workspace、all-targets、all-features、`-D warnings`）、`cargo check -p kivact-docs --all-targets --all-features`、test（workspace、`gpu-interop-producer`・`kivact-docs` を除く）：既定の feature で 1722 passed、all-features で 1763 passed、どちらも 0 failed。

それより前の統合 `1ad8547c` で行った検証（以後の変更は style の cache、DevTools の source、xtask の `kpi-run`、部品の境界）：

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
- GPU の e2e、`--no-default-features` の test、kivact-docs の doctest は `1ad8547c` で実施し、最終 `dfd4c326` では再実施していない（以後の変更は既定の feature と all-features の test で確かめた）。
- GPU の e2e の 4 件は基点でも失敗する環境の問題（上）。

## 残る制限（設計書 13 章。本改修では行わない）

- a11y の Metadata の経路の増分化（EN-10 の M の部分）。
- 折り返す editor の行の splice（EN-12 の一部）。
- layout の長さの `em` が祖先の font size を継ぐこと（今回は要素の style か `font::SIZE` に対して解決する。engine の計算済みの text style を layout に渡す改修は後続。ガイドに制限を書く）。
- `lower`・`text_input` の crate 化（狭い API ができたら）。
- 色の f32 化（ST-24）。
- drop target、combobox の trigger 文字、audio control の kit の統一（作者の判断）。
- Windows の CI、`architecture/` の図の公開（CI の artifact や Pages）。
