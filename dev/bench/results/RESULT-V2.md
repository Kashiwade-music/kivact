# V2 の結果（W8、2026-11-11）

- 基点の B0：`B0-here2`（commit `79bbd409`、この機械）。
- 判定の B1：`B1-here`（commit `1f000b93` の統合、この機械。CP2 11.3）。`B1-cp1`（`443ed1c3`）は参考。
- 必達：DESIGN 2.3 の上限（元の機械の B0 から）。K6 の既定は B1-start の値。
- 倍率：B0-here2 ÷ B1-here（小さいほど良い）。

## KPI（V2 設計書 2.3）

| KPI | 指標 | B0-here2 | B1-here | 倍率 | 必達 | 判定 |
|---|---|---|---|---|---|---|
| K2 | counter idle WS（default）KiB | 97,688 | 25,716 | 3.80 | ≤ 27,214 | 満たす |
| K2 | counter idle WS（cpu）KiB | 36,580 | 25,720 | 1.42 | ≤ 26,694 | 満たす |
| K2 | 3000 行 idle WS（default）KiB | 569,580 | 99,164 | 5.74 | ≤ 129,989 | 満たす |
| K2 | 3000 行 idle WS（cpu）KiB | 475,568 | 98,536 | 4.83 | ≤ 158,479 | 満たす |
| K2 | item box KiB/item | 28.64 | 3.60 | 7.96 | ≤ 6.50 | 満たす |
| K2 | item text KiB/item | 10.58 | 2.90 | 3.65 | ≤ 3.64 | 満たす（余裕小） |
| K2 | item button KiB/item | 108.84 | 18.58 | 5.86 | ≤ 24.75 | 満たす |
| K2 | item row KiB/item | 146.06 | 24.21 | 6.03 | ≤ 34.45 | 満たす |
| K3 | 3000 行・毎 frame 更新の CPU（default）% | 100.30 | 12.12 | 8.28 | ≤ 19.87 | 満たす |
| K4 | 3000 行 work p95 ms | 22.2231 | 1.7832 | 12.46 | ≤ 3.67 | 満たす |
| K4 | 3000 行 work p99 ms | 23.588 | 2.0806 | 11.34 | ≤ 4.94 | 満たす |
| K4 | 3000 行 描画間隔 p99 ms | 25.4367 | 17.6238 | 1.44 | ≤ 19.17 | 満たす |
| K5 | unrelated text 更新 n3000（mean）ms | 73.213 | 1.237 | 59.2 | ≤ 9.46 | 満たす |
| K5 | row 更新 n3000（mean）ms | 98.415 | 7.408 | 13.3 | ≤ 16.96 | 満たす |
| K5 | pointer p95 n3000 ms | 103.148 | 4.537 | 22.7 | ≤ 10.60 | 満たす |
| K5 | wheel n3000（mean）ms | 22.802 | 0.137 | 166 | ≤ 16.29 | 満たす |
| K5 | mount n3000 ms | 1,557.247 | 499.897 | 3.11 | ≤ 813.8 | 満たす |
| K5 | counter click（mean）ms | 1.971 | 0.878 | 2.24 | ≤ 1.907 | 満たす |
| K6 | counter GPU 抜き（MS-5 (1)）B | 24,594,432 | 14,304,768 | 1.72 | ≤ 14,467,313 | 満たす |
| K6 | counter 既定（gpu 込み、MS-5 (2)）B | 24,594,432 | 18,900,480 | 1.30 | ≤ 18,978,816（B1-start） | 満たす |
| K6 | kv_startup no-default B | 24,590,848 | 11,250,176 | 2.19 | ≤ 11,177,658 | **未達**（+72,518 B、+0.65 %）。ユーザーが許容（下の AC-2） |
| K6 | basic catalog GPU 抜き（MS-5 (1)）B | 54,251,520 | 29,166,080 | 1.86 | ≤ 31,912,659 | 満たす |
| K6 | basic catalog 既定（gpu 込み、MS-5 (2)）B | 54,251,520 | 33,744,384 | 1.61 | ≤ 43,090,432（B1-start） | 満たす |
| K1 | main → first present（default）ms | 484.8 | 42.3 | 11.5 | 悪化させない | 満たす |

- K6 の未達は、サイズが決定的なので測り直していない（作業指示書 7.3。`B1-cp1` も同じ 11,246,592 B）。
- B0-here2 の K3・K4 の一部は、計測中に OS が window の描画を絞った（WARNING：46.8 fps）。数値は参考。

## AC の判定

### AC-1 正しさ：満たす（Linux を除く）

CP2 11.1 の結果（統合 HEAD `1f000b93` の上、フルの xtask は `XTASK_NODE` で Node を指定）：
- `cargo fmt --all -- --check`：成功。
- `cargo xtask check`（フル、architecture を生成）：成功、error 0（commit `3d4d2420`）。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：成功。
- feature の組み合わせ：`cargo check -p kivact --no-default-features`、`--features gpu`、`--features gpu-interop` は成功。GPU の依存の漏れなし。`kivact-public-api-tests --no-default-features` の試験は成功。
- workspace の全試験（`--no-fail-fast`）：失敗 0。gpu-interop-producer、kivact-docs、GPU の `gpu_on_demand`：成功。
- Linux：未確認。WSL（Ubuntu-24.04）は起動できなかった（`ERROR_INVALID_DATA`）。作業指示書の規則に従い、「Linux 未確認」と記録する。

### AC-2 性能：未達 1 行（ユーザーが許容）

作業指示書 12.1 は「全ての必達行を満たす」を求める。上の表で満たさないのは K6 `kv_startup` no-default の 1 行だけ（+72,518 B）。ユーザーがこの未達を許容した（2026-11-11）。したがって、AC-2 は厳密には未達として記録し、許容の扱いを人間が決める。

### AC-3 状態：機械の検査は満たす、人間の確認待ち

- (a) 機械の検査：`dev/xtask/state-audit.md` の OPEN は 0 件（V2 の前は 27 件）。
- (b) 人間の確認：`dev/bench/results/AC3_REVIEW.md` を作成した（確認の欄は空のまま、DEC-16）。**人間の確認が済むまで AC-3 は「人間の確認待ち」。**

### AC-4 公開 API・差戻し：残件あり

- `OPEN.md` の「V2 差戻し」は、W1 の差戻し（F5・M7・T8・M6・S4・M2・F4・G1）を D9〜D15 と W2〜W7 で解消済み。
- 残る差戻し：(1) K6 `kv_startup` no-default の未達（許容済みだが、AC-2 の判定の扱いが未決）、(2) `kivact-types` の `cargo doc` の警告 21 件（`geometry3d.rs`、W5 より前からあり、作業指示書の所有の外。担当が未決）。
- 公開 API の変更は G5〜G9 と FINAL-1 で入った。版 1.1 の改名（付録 B）、ARG-1〜6、`#[component]`、`Binding::update`、`GpuStart`、`complex-scripts`、`kivact::launch`、`while_open`、show/hide/toggle の規約。
- U23（M-R5）は V2 の M7・M10 で、U56（GPU OnDemand の E2E）は G1〜G4 で解消（OPEN.md に記録）。

## 判定と止めた理由

**V2 は完了を宣言しない。** 作業指示書 12.1 の 3 により、次の 2 つが残るため、12.2（書き戻し・後片付け）と 12.3（PR の下書き）は行わない。
1. AC-3 の人間の確認（`AC3_REVIEW.md`）が未了。
2. AC-2 の K6 `kv_startup` no-default が必達を超えたまま（許容済みだが、AC-2 の判定は人間が決める）。

## 人間に確認をお願いする項目

1. `AC3_REVIEW.md` の確認欄に結果を書く（AC-3 (b)）。
2. K6 `kv_startup` no-default（+72,518 B）の扱い：許容として AC-2 を「満たす（例外あり）」とするか、必達を見直すか。
3. `kivact-types` の `cargo doc` の警告 21 件（`geometry3d.rs`）の担当。
4. Linux の確認（WSL が起動できないため未確認）。
5. S45 の fontdb の版（lock は 0.23.0、付属書は 0.24.0）と、T4 の `xtask-allow` marker の削除（付属書の文言と差がある）の確認。
6. 残りの後片付け：`W:/w/kv-b0`（B0 の計測用の worktree、W8 の後に消す）と、`wt/CP1-A-fix`（不採用の案 `52810781` を含む branch）。

## 次（人間の確認の後）

確認が承認されて、K6 と kivact-types の警告の扱いが決まったら、作業指示書 12.1 をやり直して 12.2（書き戻し・後片付け）と 12.3（PR の下書き）に進む。
