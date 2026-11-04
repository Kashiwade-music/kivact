# Foundation v3 progress

Integration: `refactor/kashiwade/foundation-v3`
Date: 2026-11-04

## Baseline

Baseline gates are running. Logs: `target/foundation-v3/baseline/` (not committed).

| Gate | Result |
|---|---|
| fmt | exit 0 |
| initial xtask / E2E | Interrupted after disk-full errors during compilation; no E2E result. |
| no-default-features E2E + doctest | exit 0; 1325 passed, 51 ignored; cumulative elapsed 830.5 seconds (includes preceding all-features attempt). |
| individual features | exit 0; cumulative elapsed 956.9 seconds (preceding suite attempts included). |
| xtask | exit 0; 43 workspace crates, 145 graphs; 1203.1 seconds including first build and Node shim shutdown wait. |
| clippy all targets/features | exit 0; 31.54 seconds after xtask. |
| retry | Running with CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, CARGO_BUILD_JOBS=2. |
| all-features E2E | wrapper exit 127, 390.5 seconds; arvane 33 + native 6 passed; GPU read_back passed 5/6 before STATUS_HEAP_CORRUPTION (0xc0000374). Serial rerun queued to isolate concurrency. |

## Units

| ID | Unit | Status | Merge commit | Gates | Time | Issues |
|---|---|---|---|---|---|---|
| P-0 | 準備 | doing | — | — | — | — |
| P-1 | 状態の全数調査 | doing | — | — | — | — |
| P-2 | 比較用の trace | doing | — | — | — | — |
| V-1a | 名前の衝突を先に解く | todo | — | — | — | — |
| V-1b | `kivact-wire` の解体 | todo | — | — | — | — |
| V-2 | semantics を一つに | todo | — | — | — | — |
| V-3 | 環境・設定・option・request | todo | — | — | — | — |
| V-4 | 共通の部品と定数 | todo | — | — | — | — |
| V-5 | 入力 event を一つに | todo | — | — | — | — |
| V-6 | 描画の記録・render・画素・text style | todo | — | — | — | — |
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
