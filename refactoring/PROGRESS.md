# Foundation v3 progress

Integration: `refactor/kashiwade/foundation-v3`
Date: 2026-11-04

## Baseline

Baseline gates completed (serial E2E as documented below). Logs: `target/foundation-v3/baseline/` (not committed).

| Gate | Result |
|---|---|
| fmt | exit 0 |
| initial xtask / E2E | Interrupted after disk-full errors during compilation; no E2E result. |
| no-default-features E2E + doctest | exit 0; 1325 passed, 51 ignored; cumulative elapsed 830.5 seconds (includes preceding all-features attempt). |
| individual features | exit 0; cumulative elapsed 956.9 seconds (preceding suite attempts included). |
| xtask | exit 0; 43 workspace crates, 145 graphs; 1203.1 seconds including first build and Node shim shutdown wait. |
| clippy all targets/features | exit 0; 31.54 seconds after xtask. |
| all-features serial E2E + doctest | exit 0; 1338 passed, 52 ignored; 960.0 seconds including lock wait. |
| resource settings | Completed with CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, CARGO_BUILD_JOBS=2. |
| all-features E2E | wrapper exit 127, 390.5 seconds; arvane 33 + native 6 passed; GPU read_back passed 5/6 before STATUS_HEAP_CORRUPTION (0xc0000374). Serial rerun passed; retained as the existing GPU concurrency issue. |

## Units

| ID | Unit | Status | Merge commit | Gates | Time | Issues |
|---|---|---|---|---|---|---|
| P-0 | 準備 | merged | 19e720f (+ fixes below) | CP0 pass; 28 tool tests | see gate logs | none |
| P-1 | 状態の全数調査 | merged | 6758cb4 | 34 new / 203 total; unprocessed 0 | reports 173s; clippy 153s | none |
| P-2 | 比較用の trace | merged | ab50865 (+ 35375f9, 468deac) | CP0 trace pass | full trace 503.1s | documented GPU empty polls |
| V-1a | 名前の衝突を先に解く | merged | b53d30c | targeted check pass; CP1 pending | see logs | none |
| V-1b | `kivact-wire` の解体 | merged | 0dbd396 | workspace check pass before V2–V6 | 16.82s | none |
| V-2 | semantics を一つに | merged | 8576b8a + W1 grouped merges | CP1 in progress | see logs | engine follow-up check |
| V-3 | 環境・設定・option・request | merged | 69ec6f4 + W1 grouped merges | targeted Arvane check pass; CP1 pending | see logs | font generation owner tracked in E2 |
| V-4 | 共通の部品と定数 | merged | 6b75ec9, c78783c + grouped merges | types/layout check pass; public f32 diagnostics 115→0 | see logs | CP1 pending |
| V-5 | 入力 event を一つに | merged | 52e7934 + W1 grouped merges | targeted Arvane check pass; CP1 pending | see logs | engine follow-up check |
| V-6 | 描画の記録・render・画素・text style | merged | bcb6f69 + W1 grouped merges | render/text/asset check pass; CP1 pending | see logs | engine follow-up check |
| A-0 | `Backend` と注入 | todo | — | — | — | — |
| A-1a | `arvane-virtual` と試験の経路 | todo | — | — | — | — |
| A-1b | 試験の経路の名前と残り | todo | — | — | — | — |
| A-2 | サービスと capability | todo | — | — | — | — |
| A-3 | native の runner と shell | todo | — | — | — | — |
| A-4 | embed の lifecycle | todo | — | — | — | — |
| A-5 | GPU の状態 | todo | — | — | — | — |
| A-6 | arvane-native の port と pipeline | todo | — | — | — | — |
| E-1 | text の統合と lower の公開面 | todo | — | — | — | — |
| X-1 | feature の閉じ込め | todo | — | — | — | — |
| E-2 | host の状態機械 | todo | — | — | — | — |
| E-3 | pointer と key | todo | — | — | — | — |
| E-4 | IME と編集 | todo | — | — | — | — |
| E-5 | layout と presence | todo | — | — | — | — |
| E-6 | 支援技術 | todo | — | — | — | — |
| E-7 | paint | todo | — | — | — | — |
| E-8 | engine の fragment | todo | — | — | — | — |
| E-9 | 出現と退場の大きさ | todo | — | — | — | — |
| R-1 | `kivact-runtime` を作る | todo | — | — | — | — |
| R-2 | host との境界と turn | todo | — | — | — | — |
| R-3 | reactive の内部 | todo | — | — | — | — |
| R-4 | 時間の仕組み | todo | — | — | — | — |
| R-5 | 非同期の状態 | todo | — | — | — | — |
| R-6 | `Machine` と `Trigger` | todo | — | — | — | — |
| R-7 | 内部の handler の型 | todo | — | — | — | — |
| C-1 | 起動の経路と `kivact-test` | todo | — | — | — | — |
| C-2 | i18n と style の純粋化 | todo | — | — | — | — |
| C-3 | view の状態 | todo | — | — | — | — |
| C-4 | part の入力と事実の持ち主 | todo | — | — | — | — |
| C-5 | core の木を side table に | todo | — | — | — | — |
| N-1 | module と正準パス | todo | — | — | — | — |
| N-2 | handler | todo | — | — | — | — |
| N-3 | modifier の語彙 | todo | — | — | — | — |
| N-4 | prop の形 | todo | — | — | — | — |
| N-5 | 子・slot・選択・文脈・回復 | todo | — | — | — | — |
| N-6 | `EditPhase` と `on_edit` | todo | — | — | — | — |
| N-7 | interaction の名前と試験の API | todo | — | — | — | — |
| N-9 | kit・例・文書・試験の追従 | todo | — | — | — | — |
| N-8 | 公開 API の検査 | todo | — | — | — | — |
| I-0 | option と policy | todo | — | — | — | — |
| I-1 | 開閉の family | todo | — | — | — | — |
| I-2 | menu と選択の family | todo | — | — | — | — |
| I-3 | 編集・drag・文字編集・toggle | todo | — | — | — | — |
| I-4 | 一覧の family | todo | — | — | — | — |
| I-5 | 値の family | todo | — | — | — | — |
| I-6 | 通知と追従の family | todo | — | — | — | — |
| U-1 | ui の gesture | todo | — | — | — | — |
| U-2 | ui の開閉と編集 | todo | — | — | — | — |
| I-7 | 出現と退場の一本化 | todo | — | — | — | — |
| I-8 | 振る舞いを APG に合わせる | todo | — | — | — | — |
| T-1 | 編集する node の制約の解除 | todo | — | — | — | — |
| T-2 | 公開 API だけの text field | todo | — | — | — | — |
| G-1 | Path と snapshot | todo | — | — | — | — |
| G-2 | 明示の pointer の capture | todo | — | — | — | — |
| G-3 | モバイルの境界 | todo | — | — | — | — |
| G-4 | soft kit が求めた中核の不足 | todo | — | — | — | — |
| G-5 | 非同期の標準部品 | todo | — | — | — | — |
| L-1 | ICU4X による地域書式 | todo | — | — | — | — |
| K-0 | kit のための中核の部品 | todo | — | — | — | — |
| K-1 | 日付・carousel・dropzone | todo | — | — | — | — |
| K-2 | 開閉と hover と選択の入力 | todo | — | — | — | — |
| K-3 | navigation と focus の部品 | todo | — | — | — | — |
| K-4 | collection と creative と game | todo | — | — | — | — |
| K-5 | soft に残る状態と hook | todo | — | — | — | — |
| K-6 | 見た目の部品 | todo | — | — | — | — |
| K-8 | `viewport` を中核へ | todo | — | — | — | — |
| K-9 | Tier 3 の部品を中核へ | todo | — | — | — | — |
| K-7 | kit の規約の残り | todo | — | — | — | — |
| F-1 | 文書 | todo | — | — | — | — |
| F-2 | 検査の仕上げ | todo | — | — | — | — |
| F-3 | 最後の確認と PR の説明の下書き | todo | — | — | — | — |

## CP0

Initial integrated checks: fmt exit 0; xtask exit 1 (three client-to-diagnostics module edges); clippy exit 101 (eight new tool style diagnostics). Logs: `target/foundation-v3/cp0/`.

- `fix-cp0-xtask` c47fceb: separate policy helpers from check dispatch to preserve the module DAG; 28 tool tests pass.
- `fix-cp0-core` 35375f9: move private tracing to client; core all-targets/all-features check passes.
- Tool Clippy syntax fixes 09952b2 are merged; the phased lint configuration 5a24aee is merged and its crate-deny override was checked with a scratch compiler fixture.
- CP0 all-features E2E/doctest: exit 0, 634.8 seconds. Remaining checks restarted after the lint configuration merge; interrupted logs are retained.
- Checkpoint resources: jobs 4, incremental/debug symbols off, bundled XTASK_NODE; RUST_TEST_THREADS=1 for GPU concurrency; RUSTDOCFLAGS=--test-args=--test-threads=4 for doctest concurrency.
- P-1 raw report logs archived under `target/foundation-v3/p1/`. State inventory: 34 new / 203 total; 521 checker lines and 43 unique Clippy locations classified, unprocessed 0.

- Final CP0 fmt/xtask/Clippy: all exit 0 (257.3 seconds to xtask, 276.0 cumulative to Clippy); 43 crates / 145 graphs. Final E2E/feature/trace results are recorded below.

- Trace-only performance fix 468deac: buffered writes and ephemeral O(n) snapshot lookup, preserving DFS/removal order. Tool checks and core Clippy pass; final fmt/xtask/full Clippy rechecked with exit 0 (154.1 seconds cumulative).
- Unbuffered trace run interrupted at the large Creative catalog; partial reference files retained at target/foundation-v3/cp0/trace-before-buffering/. One restart hit an executable lock from the orphaned catalog test; the verified old test process was stopped and the final trace run restarted.

### CP0 result: PASS

- Final fmt, xtask, full Clippy, feature checks: exit 0.
- All features: 1362 passed / 52 ignored (634.8s). No default features: 1349 passed / 51 ignored (405.6s). Trace all features: 1362 passed / 52 ignored (503.1s).
- 915 trace files, 87,069,573 bytes, saved at target/foundation-v3/traces/cp0/ with SHA-256 manifest beside it.
- Buffered trace content: 781/782 completed reference files byte-identical; one DX12 async poll difference classified and normalized as documented in DECISIONS/work order. Creative catalog all 3 tests passed in 132.13s with tracing.
- W0 complete. Next: W1 V-1a → V-1b definition/follow-up → V-2–V-6 definition/follow-up → CP1.

## CP1 (in progress; not passed)

- Definitions and grouped consumers merged. G1 engine/lower/text 9494f22; layout 620bad4; G2 Arvane/native 11bf078; G2 renderer/text/assets b278d5f + 1a59dbc; G3 core/style/runtime 2427f59 + 8e88be0; G4 UI cedcbfa; G5 kits 23276c3; G6 examples/docs/tests a6dbd7c.
- Types check passes with f64 scalar math. Public-source reachability now includes signature and field types without creating extra export paths; 32 tooling tests pass. Improved numeric report: 115 diagnostics before → 0 after (7 existing ownership findings remain for N8).
- Targeted all-targets/all-features checks pass for Arvane/shell/native, renderer/text/assets/GPU sharing, and layout. Style/motion/task/editing library checks pass after scalar adapter/path fixes.
- Full workspace check logs: `target/foundation-v3/w1/workspace-check*.log`. First integration error was an AssetError-to-Failure detail conversion, corrected in 25b5b3a. The next check found five engine reference/import errors and one unused parameter; `fix-w1-g1` is validating those fixes before further workspace checking.
- Full fmt/xtask/Clippy, E2E suites, feature checks, and CP0 trace comparison are still pending. Do not start W2 or push CP1 until all pass.
- W1 reconciliation decisions are in DECISIONS.md. Font generation publication is explicitly assigned to E2 after the font owner moves in E1; the existing registration event order is preserved in W1.
- Integration compiler follow-ups 744ec0d (engine), afd6a99 (core), and dc585ec (kits) are merged. Full workspace all-targets/all-features check passes with zero warnings (`workspace-check-6.log`).
- VirtualShell completion 0c39900 stores one Environment and owns injected mouse/modifier facts; core's added duplicate input caches are removed. Raw complete samples retain their supplied information. Targeted Arvane/core check passes.
- All six lint groups completed. Final full workspace all-targets/all-features Clippy with warnings denied passes in 15.99s (`cp1/clippy-final.log`); full fmt check passes. Unit logs are archived under `target/foundation-v3/unit-logs/`, merged worktrees removed.
- Initial full xtask generated 42-crate/143-graph architecture, then rejected an overlong Role description and a missing embed-to-frame_log policy edge. Both corrected in 34b4677; final write-mode xtask is in progress. E2E/feature/trace gates have not run yet.
- Final xtask passes (`cp1/xtask-final.log`): 42 crates / 143 target/configuration graphs. Architecture regenerated from the integrated source.
- All-feature E2E/doctest gate is running with serial window/GPU tests and four doctest threads. Initial sandboxed Git Bash launch failed before tests with signal-pipe Win32 error 5; the identical authorized command is running with native execution permissions. Current log: `cp1/all-features.log`. No-default/individual-feature/trace gates remain pending.
- First all-feature run stopped after 103.68s: Arvane/native/GPU preceding suites passed; core had 38 passed / 1 failed / 1 ignored. The existing tiny-positive resize fixture exposed loss of f32-underflow rejection. Corrected the shared window predicate in 110eebe with an explicit one-logical-pixel minimum; fixture retained. Initial log preserved as `cp1/all-features-initial.log`. Targeted existing fixture is being rerun before a full `--no-fail-fast` retry.
