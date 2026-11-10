# Slim-Fast 受入れ判定（W7-1）

- B0: `dev/bench/results/B0.md`（有効、commit 79bbd409 の計測）
- B1: `dev/bench/results/B1.md`（有効、commit 0699af99 の計測）
- 倍率は B0 ÷ B1（大きいほど良い）。K3 のアイドルだけは B1 の値そのもの。
- 計測は同じ機械（Windows 11 / RTX 3060 Ti / 24 logical CPUs）で、`dev/tools/window-lock.sh` を通して実行した。テストの checkpoint はディスクと PDB の制約から `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` で実行した（debug assertions は有効、期待値は変えていない）。

## KPI

| KPI | 指標 | B0 | B1 | 倍率 | 必達 | 判定 |
|---|---|---|---|---|---|---|
| K1 | `kv_startup` main→初回 present（default） | 461.8 ms | 41.1 ms | 11.24× | ≥ 5× | ✓ |
| K1 | basic catalog 起動→初回 present | 654.0 ms | 189.6 ms | 3.45× | ≥ 2.5× | ✓ |
| K2 | counter idle WS（default） | 136068 KiB | 34912 KiB | 3.90× | ≥ 5× | ✗ |
| K2 | counter idle WS（CPU） | 37372 KiB | 34968 KiB | 1.07× | ≥ 1.4× | ✗ |
| K2 | counter idle WS（GPU 強制） | 136444 KiB | 94396 KiB | 1.45× | 悪化なし（≤ 1.05×B0） | ✓ |
| K2 | 3000 行 idle WS（default） | 649944 KiB | 292776 KiB | 2.22× | ≥ 5× | ✗ |
| K2 | item 空 box | 32.52 KiB | 12.39 KiB | 2.62× | ≥ 5× | ✗ |
| K2 | item text | 14.57 KiB | 10.48 KiB | 1.39× | ≥ 4× | ✗ |
| K2 | item button | 123.75 KiB | 66.59 KiB | 1.86× | ≥ 5× | ✗ |
| K2 | item 行 | 172.24 KiB | 85.44 KiB | 2.02× | ≥ 5× | ✗ |
| K2 | 3000 行 idle WS（CPU） | 554676 KiB | 292328 KiB | 1.90× | ≥ 3.5× | ✗ |
| K3 | アイドル（counter） | 0 frame、0.000 ms/s | 0 frame、0.000 ms/s | — | 0 frame、≤ 0.5 ms/s | ✓ |
| K3 | 3000 行・毎 frame 更新の CPU | 99.37% | 99.08% | 1.00× | ≥ 5× | ✗ |
| K4 | 3000 行 work p95 | 21.99 ms | 19.28 ms | 1.14× | ≥ 6× | ✗ |
| K4 | 3000 行 work p99 | 24.71 ms | 21.51 ms | 1.15× | ≥ 5× | ✗ |
| K4 | 0 行 work p95 | 3.54 ms | 1.81 ms | 1.96× | 悪化なし（≤ 1.05×B0） | ✓ |
| K4 | 描画間隔 p99（3000 行） | 27.13 ms | 22.89 ms | — | ≤ 19.17 ms | ✗ |
| K4 | 最初の frame の raster（CPU、800×600） | 29.7 ms | 3.3 ms | 9.0× | ≥ 6× | ✓ |
| K5 | 無関係な 1 text の更新 n=3000 | 75.68 ms | 57.53 ms | 1.32× | ≥ 8× | ✗ |
| K5 | 1 行の更新 n=3000 | 101.74 ms | 78.98 ms | 1.29× | ≥ 6× | ✗ |
| K5 | pointer 移動 p95 n=3000 | 105.99 ms | 89.05 ms | 1.19× | ≥ 10× | ✗ |
| K5 | wheel n=3000 | 24.44 ms | 22.02 ms | 1.11× | ≥ 1.5× | ✗ |
| K5 | mount n=3000 | 1627.5 ms | 1507.4 ms | 1.08× | ≥ 2× | ✗ |
| K5 | counter の click | 2.288 ms | 1.954 ms | 1.17× | ≥ 1.2× | ✗（CP2 は 1.28×） |
| K6 | counter（default） | 24594432 B | 18978816 B | 1.30× | ≥ 1.7× | ✗ |
| K6 | `kv_startup`（no-default-features） | 24590848 B | 15922688 B | 1.54× | ≥ 2.2× | ✗ |
| K6 | basic catalog（default） | 54251520 B | 43090432 B | 1.26× | ≥ 1.7× | ✗ |

未達の行は多いが、K1・K4 の起動と全画面 raster、K3 のアイドル、K5 の counter click（CP2）は改善した。K2・K4 の 3000 行・K5・K6 は設計の見込みに届いていない。W7 の規則により、未達の行の再計測（B1-r2、B1-r3）は行わない（未達の原因の調査と設計の見直しを設計者に差し戻す段階であり、実装者が目標や測定条件を変えない）。

## AC-1〜AC-4

| ID | 結果 | 根拠 |
|---|---|---|
| AC-1 正しさ | ✓ 満たす | CP3 の gate がすべて成功：`cargo fmt --all -- --check`、`cargo xtask check`（architecture 生成）、`cargo clippy --workspace --all-targets --all-features -- -D warnings`、workspace の全 test（producer・docs を除く）、`gpu-interop-producer` の test、`kivact-docs` の test、`cargo check -p kivact --no-default-features`（と `--features gpu`、`--features gpu-interop`）、`cargo tree` の GPU 非混入、`kivact-public-api-tests --no-default-features`。既存の E2E の期待値は緩めず、新しい E2E（`#[component]`、`Binding::update`、資源の回収、SVG の埋込み文字）も通した。 |
| AC-2 KPI | ✗ 満たさない | 上の表のとおり、K2・K3（3000 行）・K4（3000 行）・K5 の多く・K6 が必達未達。 |
| AC-3 状態 | ✗ 満たさない | `dev/xtask/state-audit.md` に 27 件の `OPEN`（閉じ方の無い field）が残った。`dev/bench/results/OPEN.md` にも U23 の M-R5（button ごとの意味論 patch の保持）の状態項目が残っている。 |
| AC-4 未実施 | ✗ 満たさない | U23 の STOP（M-R5）は K2 に影響する。GPU OnDemand の E2E（`gpu_on_demand.rs`）は、公開 API で `GpuStart::OnDemand` を要求できず未実施（設計書 7 章）。撤退規則の適用は無し（U13・U21 は修正で通した）。 |

## 結論

AC-2・AC-3・AC-4 を満たさないため、設計書 1.6 節の受入れは満たされない。W7-1 の規則によりここで停止する：

- `dev/bench/results/RESULT.md` と `dev/bench/results/OPEN.md` を commit・push する。
- W7-2（architecture.md・decisions.md への書き戻し、設計書・作業指示書の削除）と W7-3（PR の下書き）は行わない。
- `SLIM_FAST_DESIGN.md`、`SLIM_FAST_WORK_ORDER.md`、`20261109_2107_CODEX_REVIEW.md` は残す。
