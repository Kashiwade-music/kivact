# Foundation v3 progress

Integration: `refactor/kashiwade/foundation-v3`。記録の書き方は作業手順書 0.5 節（表の一行ずつ。checkpoint に添える文は 3 行まで）。

## Checkpoints

| CP | commit | fmt | clippy | xtask | features | E2E all-features | E2E no-default | 時間 | 備考 |
|---|---|---|---|---|---|---|---|---|---|
| CP0 | b6e7ff2 | 0 | 0 | 0 | 0 | 1362 passed / 52 ignored | 1349 passed / 51 ignored | 634.8s / 405.6s | 通過 |
| CP1 | 97689c4 | 0 | 0 | 0 | 0 | 1365 passed / 51 ignored | 1352 passed / 50 ignored | 519.1s / 355.0s | 通過（2026-11-05 の改訂の基準。trace の比較は廃止） |
| CP2 | 01768d3 | 0 | 0 | 0 | 0（17 件） | 101→失敗3 target個別0、GPU 6 passed | 0、GPU 2 passed | Clippy 25.36s / WSL 68s | 通過。WSL Clippy 0、失敗全体の再実行なし |
| CP3 | b0aad13 | 0 | 0 | — | — | 101→失敗4 target個別0、GPU6 passed | — | Clippy 8.69s / 固定public3.62s | 通過、固定 gate の公開308/Game3と対象 docs 0 |
| CP4 | 19dfddc | 0 | 0 | 0（ab3322a） | 0（18件） | 101→失敗19 target個別0 | 101→失敗19 target個別0 | Clippy16.83s / WSL75s | 通過。固定a9e6377 public30/doc0、Creative追加2はc97d2c6で0 |
| CP5 | b3b2cf0 | 0 | 0 | — | — | 101→失敗5 target個別0、GPU6 passed | — | Clippy19.52s | 通過、Enter/disabled既定の意図的APG移行とSoft初期enter修正 |

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
| KL-creative | doing | 10a222e | (a)43 passed 統合、(b)43対象通過/source merge 待ち |
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
| N-F1-props | merged | 2ef7cce | fmt 0、安定 root の reactive props/identity equality |
| fix-G3-7 | merged | 0db0e29 | Notifier/setter/path 移行、残りは別 thread unit |
| fix-G3-8 | merged | 01a2798 | canonical Send settings/固定 embed theme、所有 error 解消 |
| fix-G3-9 | merged | ac70830 | core/test/types all-target/all-feature check 0、warning 0 |
| fix-G4-1 | merged | b759f70 | interaction check 0、37 files、所有 warning 0 |
| fix-G4-2 | merged | bb9f1b0 | UI/facade all-target/all-feature check 0、warning 0 |
| fix-G5-2 | merged | ad12634 | basic/pop/game check 0、所有 warning 0 |
| fix-G5-3 | merged | c24b36c | creative/source consumers check 0、warning 0 |
| fix-G5-4 | merged | 08d13c1 | Soft all-target/all-feature check 0、所有 warning 0 |
| fix-G4-4 | merged | 4102d9b | 共通 control size を Value で保持、UI/facade check 0 |
| fix-G3-10 | merged | d647a55 | Align/FlexDirection/Justify の定数入力、check 0 |
| fix-G4-3 | merged | 69c5df6 | reactive scalar key_step、interaction/creative check 0 |
| N-F2 | merged | 7efa8aa | fmt 0、62 files、Part/Selection/shared draft/state reads |
| N-F3 | merged | aba3e97 | fmt 0、Recipe/TreeRows/drafting、追加の旧 caller なし |
| N-F4 | merged | 39220fd | fmt 0、139 files、reactive Calendar/headless menubar |
| N-F5 | merged | 50dd85c | fmt 0、117 files、typed builder/slot/prop 移行 |
| N-F6 | merged | 60b6026 | fmt 0、139 files、runtime prop 残件なし/型確認へ |
| N-G1 | merged | 979d9d1 | examples/catalog/templates check 0 |
| N-G2 | merged | 7a59eff | docs/public check 0、既知 docs 失敗は fix-G4-5 対象48 passed |
| N-G3 | merged | 0fafc29 | native EditPhase/on_edit、対象4 passed |
| N-G4 | merged | d248ec3 | semantics 単一 publisher、対象8 passed |
| N-G5 | merged | a37cfd9 | machine/binding/theme/reactive prop E2E 11 passed |
| N-8 | merged | a634983 | API/経路/構造検査 0、xtask36 passed、workspace check 0 |
| fix-G4-5 | merged | 16be39b | composite activation/cleanup、docs48/public10 passed |
| fix-G3-11 | merged | 7478530 | core Clippy 18 件修正、check 0 |
| fix-G4-6 | merged | c76cf0c | interaction/UI Clippy 修正、check 0 |
| fix-G5-5 | merged | 19dfddc | basic/pop/game/creative Clippy 修正、check 0 |
| fix-G5-6 | merged | 2e25c69 | Soft Clippy 修正、check 0 |
| fix-G6-8 | merged | 3d97dc1 | canonical launch/app policy parse 0 |
| fix-G3-12 | merged | 384cad8 | launch 一元化、統合 core check 0 |
| fix-G3-13 | merged | 2e0171a | view passive contracts/queue 所有、core check 0、docs 待ち |
| fix-G4-7 | merged | c8acd89 | scalar/calendar cycle 解消、check 0 |
| fix-G6-9 | merged | 2dddc69 | policy canonical paths/kit 責務分割、check 0 |
| I-0 | merged | c9cf306 | public17/Soft20 passed、check/state-fields 0 |
| I-1 | merged | d94dd0a | public20/Soft9/dialog6 passed、check/state-fields 0 |
| I-2a | merged | ac88c15 | public15/Soft6 passed、shared binding と caller 統合 check 0 |
| I-2b | merged | d92cae4 | APG/menu10・Soft3 passed、check/state-fields0 |
| I-3 | merged | c0691c5 | public19 passed、check/state-fields 0 |
| I-4a | merged | a64fe5a | public26/Soft4 passed、check/state-fields0 |
| I-4b | merged | afc0a71 | APG grid8・Soft7 passed、check/state-fields0 |
| I-5a | merged | 1156889 | public19/slider4/Soft8 passed、check/state-fields0 |
| I-5b | merged | 5e85bb7 | public10/state-fields5scope passed、最終leafは親整合check待ち |
| I-6 | merged | cca5c70 | public7/Soft2、固定follow2 passed、check/state-fields0 |
| U-1 | merged | 0c87c70 | public12/Soft6 passed、check/state-fields0 |
| U-2 | merged | 257f8d5 | public56/APG2/Soft5 passed、check/state-fields0。details残件はI7 |
| I-7 | merged | c41ae9b | public24+motion/swipe12/details2/Soft7 passed、check0 |
| T-1 | merged | 6771046 | text_editing10 passed、check/fmt 0 |
| T-2 | merged | 9130a41 | 公開API editor証明7 passed、check0 |
| G-1a | merged | 1685f3e | canvas8 passed、workspace check/fmt 0 |
| G-1b | merged | b36cb40 | engine/core check0、固定canvas10 passed |
| G-2 | merged | bec641a | pointer6 passed、engine/core check/fmt0 |
| G-3 | merged | 72f4a97 | workspacecheck0/public21 passed |
| G-4a | merged | 586a75c | style_paint6 passed、check/fmt 0 |
| G-4b | merged | 0703861 | motion11 passed、check/fmt0 |
| G-4c | merged | 814ca08 | 固定統合environment/scroll17 passed、workspacecheck/fmt0 |
| G-5 | merged | a3d8db9 | async6/doc538+1 passed、scopedcheck/state-fields0 |
| K-0 | merged | 7385fe7 | core3/conformance20/5kit tests/docs0、workspacecheck0 |
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
| H-G1 | merged | b7ff6ff | groupclippy/static/tests0、親fullstatic0/WSL0 |
| H-G2 | merged | 7385fe7 | groupclippy/static/tests0、GPU6、親fullstatic0/WSL48.97s |
| H-G3 | todo | | |
| H-G4 | todo | | |
| H-G5 | todo | | |
| H-G6 | todo | | |
| F-1 | todo | | |
| F-2 | todo | | |
| F-3 | todo | | |
| fix-G4-8 | merged | 143d513 | public2/Soft7/catalog15/Game1 passed |
| fix-G5-7 | merged | f52935c | Creative42+固定advanced1/catalog3 passed |
| fix-G5-8 | merged | c40d5cb | Basic navigation1 passed |
| fix-G6-10 | merged | 1558df8 | 6 crate doctests 対象0 |
| fix-G3-14 | merged | 3852a9f | 診断期待を新契約へ、source修正はG3-15 |
| fix-G3-15 | merged | 640fea0 | core check0、diagnostics4 passed |
| fix-G4-9 | merged | 0b78efb | passive policy/token 型分離、check0 |
| fix-G4-10 | merged | f1b0340 | private module namespace 分離、check0 |
| fix-G4-11 | merged | a2946b2 | press-time factory、check0/E2E1 passed |
| fix-G4-12 | merged | 291e2de | Generation/active payload、check0/E2E1 passed |
| fix-G4-13 | merged | 507ea44 | 元のgrouped-table試験1/list11 passed、check/state-fields0 |
| fix-G5-9 | merged | 2ed4bfb | Creative9親を葉へ分離、check0/state-fields所有0 |
| fix-G4-14 | merged | 8382d72 | no-default flow1/allfeature20 passed、check/state-fields0 |
| fix-G5-10 | merged | 03ad2c5 | Basic2/publictooltip6/Soft4 passed、fmt/diff0 |
| fix-G1-5 | merged | 663d17b | native all/default/no-default check0、警告0 |
| fix-G3-16 | merged | 4c405bf | runtime循環分離、check/state-fields0 |
| fix-G4-15 | merged | dc3892f | interaction lint10修正、check0 |
| fix-G5-11 | merged | c97d2c6 | queued capture/release mailbox、pointer6/catalog2 passed |
| fix-G6-11 | merged | 89d8916 | E2E lint4、check/fmt0 |
| fix-G3-17 | merged | f005440 | passive観測値を分離、check0/親fullstatic0 |
| fix-G4-17 | merged | 75c7185 | APG Enter移行、元のfailed3件各0 |
| fix-G5-12 | merged | 99b03e7 | PopのSkip fixture、元のfailed1件0 |
| fix-G5-13 | merged | ee2ea2d | 初期展開は自然高、既存consumer2件0 |
