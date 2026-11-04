# Foundation v3 progress

Integration: `refactor/kashiwade/foundation-v3`。記録の書き方は作業手順書 0.5 節（表の一行ずつ。checkpoint に添える文は 3 行まで）。

## Checkpoints

| CP | commit | fmt | clippy | xtask | features | E2E all-features | E2E no-default | 時間 | 備考 |
|---|---|---|---|---|---|---|---|---|---|
| CP0 | b6e7ff2 | 0 | 0 | 0 | 0 | 1362 passed / 52 ignored | 1349 passed / 51 ignored | 634.8s / 405.6s | 通過 |
| CP1 | 97689c4 | 0 | 0 | 0 | 0 | 1365 passed / 51 ignored | 1352 passed / 50 ignored | 519.1s / 355.0s | 通過（2026-11-05 の改訂の基準。trace の比較は廃止） |
| CP2 | 01768d3 | 0 | 0 | 0 | 0（17 件） | 101→失敗3 target個別0、GPU 6 passed | 0、GPU 2 passed | Clippy 25.36s / WSL 68s | 通過。WSL Clippy 0、失敗全体の再実行なし |
| CP3 | b0aad13 | 0 | 0 | — | — | 101→失敗4 target個別0、GPU6 passed | — | Clippy 8.69s / 固定public3.62s | 通過、固定 gate の公開308/Game3と対象 docs 0 |

- 2026-11-05: 作業手順書と設計書（19 章）を改訂。W2 から再開する。`target/foundation-v3/` は不要（消してよい）。

## Units

| ID | 状態 | merge | 残件 |
|---|---|---|---|
| P-0 | merged | 19e720f | |
| P-1 | merged | 6758cb4 | |
| P-2 | merged | ab50865 | |
| V-1a | merged | b53d30c | |
| V-1b | merged | 0dbd396 | |
| V-2 | merged | 8576b8a | |
| V-3 | merged | 69ec6f4 | |
| V-4 | merged | c78783c | |
| V-5 | merged | 52e7934 | |
| V-6 | merged | bcb6f69 | |
| A-0 | merged | 1d54c60 | 指定 check 0、統合 check 0 |
| A-1a | merged | 17d4592 | Arvane 指定 check 0 |
| A-1b | merged | 7319467 | 指定 check 0、公開 API E2E 285 passed、native smoke 0 |
| A-2 | merged | 26d62bb | 指定 check 0、platform E2E 7 passed、統合 workspace check 0 |
| A-3 | merged | f4f17dc | 指定 5 crate/統合 workspace check 0、状態点検 1〜10 完了 |
| A-4 | merged | 19f1a69 | 新 trait 前提の確認は A-1a/1b 合流で |
| A-5 | merged | 9d59292 | 対象 4 package check 0、全体は A-1b 合流で |
| A-6a | merged | 49a6fc3 | 統合 check 0、input_platform 8/embed 8 passed |
| A-6b | merged | 48cb043 | 統合 check 0、input_platform 8/embed 8 passed |
| E-1 | merged | 3f3b930 | 初回 check 0、xtask 0、統合 check 0 |
| E-2 | merged | 20439fd | check 0、lifecycle 3/environment 6/embed 8 passed |
| E-3 | merged | 7fb10d4 | keyboard 5/pointer 16/hover 7 passed、統合 check 0 |
| E-4 | merged | 27be7b0 | check 0、最終 text_editing 7 passed、統合 check 0 |
| E-5a | merged | 22f1d5e | check 0、layout E2E 15 passed、既存 state-fields は HG |
| E-5b | merged | 6fa0834 | check 0、motion E2E 10 passed、対象 state-fields 0 |
| E-6 | merged | 40a0c11 | check/統合 workspace 0、関連 E2E 5 passed |
| E-7 | merged | 82be6c5 | check/統合 workspace 0、drawable 判定統合済 |
| E-8 | merged | 5f46fbb | check 0、layout E2E 16 passed、Contents の透明化 |
| E-9a | merged | d1ea035 | check 0、presence E2E 2 passed、親 auto 箱の縮小も確認 |
| E-9b | merged | e94c53c | check 0、presence E2E 1 passed、破棄時の誤通知を除去 |
| X-1 | merged | 025199b | all/no-default check 0、feature 17 件 0 |
| fix-G3-2 | merged | 6e0e516 | mod.rs 移動と runtime Clippy 記法、check 0 |
| fix-G2-1 | merged | 7a5ef0d | smoke skip 診断と virtual 条件式、check 0 |
| fix-G2-2 | merged | 59aee2a | GPU readback 条件式、check 0 |
| fix-G1-1 | merged | 5c3b17b | Launch payload/engine 記法、check 0、全 Clippy 0 |
| fix-G2-3 | merged | c02ab11 | Arvane E2E 33 passed、WSL Clippy 0 |
| fix-G1-2 | merged | d914da0 | worker/空 root/復帰の失敗試験すべて 0 |
| fix-G6-2 | merged | 231f760 | 新 Presence 通知の診断 turn、対象 E2E 0 |
| fix-G2-4 | merged | b9f70e2 | no-GPU bound/Role/葉const/shell policy、check 0 |
| fix-G1-3 | merged | e86d141 | KeyDownTargets の所属で engine cycle 解消、check 0 |
| fix-G3-3 | merged | 01768d3 | Host/turn 値の所属で runtime cycle 解消、全 xtask 0 |
| R-1 | merged | 0341b35 | 初回 check 0、統合 check 0 |
| R-2 | merged | b4b03bd | check 0、関連 E2E 32 passed |
| R-3 | merged | 0165094 | 関連 E2E 34 passed、合流 runtime check 0 |
| R-4 | merged | 5340dd7 | runtime check 0、E2E は A-1b 合流で |
| R-5 | merged | 85c6f7f | 統合 runtime check 0、E2E は A-1b 合流で |
| R-6 | merged | e4a39b3 | check/doc test 0 |
| C-1 | merged | cc3b9d4 | check 0、公開12/private24 E2E 0、cost1削除 |
| C-2 | merged | ba2b938 | 指定 check 0、i18n/style の内部境界 |
| L-1 | merged | c13faed | check 0、i18n E2E8 passed、ICU4X2.4.0 |
| C-3a | merged | a9fa75e | 担当 check 0、kit2 caller は fix-G5-1 |
| C-3b | merged | a7512f8 | check 0、focus/lifecycle/interop 単一所有 |
| C-4 | merged | 589adb5 | check 0、parts/canvas/hover/pointer E2E15 passed |
| C-5 | merged | 4785daa | check/state-fields 0、関連 E2E22 passed |
| KL-basic | merged | 8056fb4 | check 0、E2E32/doctest40 passed |
| KL-game | merged | 6332eb4 | check/state-fields 0、E2E12/doctest3 passed |
| KL-creative | todo | | unit の明示依存 I-3 後へ（W5） |
| KL-soft | merged | c2a8a74 | check 0、全 soft E2E85 passed、最終 rich_blocks2 passed |
| fix-G3-4 | merged | 5189902 | Machine の正準 state 経路を公開 |
| fix-G6-3 | merged | 65f313f | Game/Pop catalog の C1 caller 移行 |
| fix-G5-1 | merged | a0063b2 | kit HandlerResult caller 移行、check 0 |
| fix-G6-4 | merged | 279b508 | 残る catalog/lookbook caller 移行、check 0 |
| fix-G3-5 | merged | 71e7223 | i18n 条件式の Clippy 修正、check 0 |
| fix-G6-5 | merged | b0aad13 | RenderOptions の Clippy 修正、check 0 |
| fix-G3-6 | merged | 0161c6c | core docs25/default・no-default embed 対象 0 |
| fix-G6-7 | merged | bbceec7 | ガイド対象 doctest28 passed |
| fix-G1-4 | merged | 08f0f71 | engine/layout check 0、motion9 passed、固定 gate で再検証 |
| fix-G6-6 | merged | 9b2cd29 | query check 0、子要素回帰解消、layout 修正と固定 gate で再検証 |
| N-D1 | merged | c938d63 | fmt 0、build 不可の定義段 |
| N-D2 | merged | b0beeae | fmt 0、build 不可の定義段 |
| N-F1 | merged | 909b7df | fmt 0、ambient/ThemeRef/Outcome/owner/test API |
| N-F2 | doing | | interaction API の追従 |
| N-F3 | merged | aba3e97 | fmt 0、Recipe/TreeRows/drafting、hover caller の追補待ち |
| N-F4 | doing | | basic/pop/game API の追従 |
| N-F5 | merged | 50dd85c | fmt 0、117 files、typed builder/slot/prop 移行 |
| N-F6 | doing | | soft API の追従 |
| N-G1 | todo | | |
| N-G2 | todo | | |
| N-G3 | todo | | |
| N-G4 | todo | | |
| N-G5 | todo | | |
| N-8 | todo | | |
| I-0 | todo | | |
| I-1 | todo | | |
| I-2a | todo | | |
| I-2b | todo | | |
| I-3 | todo | | |
| I-4a | todo | | |
| I-4b | todo | | |
| I-5a | todo | | |
| I-5b | todo | | |
| I-6 | todo | | |
| U-1 | todo | | |
| U-2 | todo | | |
| I-7 | todo | | |
| T-1 | todo | | |
| T-2 | todo | | |
| G-1a | todo | | |
| G-1b | todo | | |
| G-2 | todo | | |
| G-3 | todo | | |
| G-4a | todo | | |
| G-4b | todo | | |
| G-4c | todo | | |
| G-5 | todo | | |
| K-0 | todo | | |
| K-1a | todo | | |
| K-1b | todo | | |
| K-2a | todo | | |
| K-2b | todo | | |
| K-3a | todo | | |
| K-3b | todo | | |
| K-4 | todo | | |
| K-5 | todo | | |
| K-6a | todo | | |
| K-6b | todo | | |
| K-8 | todo | | |
| K-9a | todo | | |
| K-9b | todo | | |
| K-7a | todo | | |
| K-7b | todo | | |
| K-7c | todo | | |
| K-7d | todo | | |
| K-7e | todo | | |
| H-G1 | todo | | |
| H-G2 | todo | | |
| H-G3 | todo | | |
| H-G4 | todo | | |
| H-G5 | todo | | |
| H-G6 | todo | | |
| F-1 | todo | | |
| F-2 | todo | | |
| F-3 | todo | | |
