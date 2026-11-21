# Rework 作業の進捗（Orchestrator が更新）

## 環境（W0 手順 3）
- 統合 branch：`refactor/kashiwade/fundamental-rework`（基点 `2b8df5ac` = main の code。以後の差は docs のみ）
- target：空きが 200 GB 以上の drive は V:（2026-11-18 時点 1.7 TB 空き）。よって lane ごとに `V:/kv-target/rw-<L>`。統合・基点の測定は `V:/kv-target/rw-main`。
- lane の worktree：`W:/w/kv-wt/rw-W<W>-<L>`（W: の空き 136 GB）。
- 計算機：24 論理 core、RTX 3060 Ti（driver 32.0.16.1088）、WSL2 の Ubuntu-24.04 あり（Linux check 用）。

## 進捗
- W0 完了（基点の測定と gate）
  - 基点の試験の一覧：`rework/baseline-tests.txt`（1644 件。生成物で commit しない）
  - B0：`rework/kpi/B0.md`（INVALID・WARNING なし。commit `c9518946` の code は基点 `2b8df5ac` と同じ）
  - 基点の gate（T 4.1 の「W0 の基点の gate」）：fmt・xtask・clippy・gpu-interop-producer・kivact-docs・check-features は成功。
    - workspace（all-features）と workspace（no-default-features）：`chat_keyboard` の 1 件が失敗（`--no-fail-fast` で確認。他に失敗なし）。
    - GPU E2E（`kivact-public-api-tests --test e2e`、751 件）：4 件失敗（`rework/OPEN.md` の「基点の失敗」）。
    - 失敗はすべて基点で既に起きていたもので、改修の失敗ではない。直していない。
  - gate のログ：`rework/baseline-gate/`（生成物で commit しない）

## W1a 完了（merge 済み・tier b 成功）
- 設計者の回答（2026-11-18、`rework/OPEN.md` の「設計者の回答」）を受けて解消：S01（構成式の expect）、T01（rg の完了条件）、T02（`kivact-devtools-client` は T13 まで残す）、T04（docs-pairs の 2 件は T17、`dev/README.md:41` は T21）。
- lane T0（`wt/rw-W1a-T0`）：T01 `5beb3a38`、T02 `e2879bee`、T03 `0a0cc403`、T04 `a75087ab`、T23 `50333d86`、T02 の行末修正 `58206caa`（`workspace.toml` を基点と同じ CRLF に戻した）。merge `39c3c0cb`。
- lane S（`wt/rw-W1a-S`）：S01 `b0e84c65`、expect の追加 `ceb72b64`（`Deferred::new`・`SharedDeferred::new`）、行末の統一 `aea207ea`（`cell.rs` を LF に。もとは行ごとに CRLF と LF が混在）。merge `7e7e85d3`。
- tier b（統合 branch、`V:/kv-target/rw-main`、試験は `--no-fail-fast`）：fmt・xtask（`--no-write`）・clippy（workspace all-targets all-features）・`kivact-docs` の check は成功。test は失敗 1 件（`chat_keyboard`、基点の失敗と同じ。`rework/OPEN.md`）。それ以外は成功。GPU E2E は KIVACT_TEST_GPU を付けない tier b の形で実行（基点の 4 件は tier c で見る）。
- worktree の作り方の注意：submodule `vendor/xtask-core` の初期化は、worktree では `git -c protocol.file.allow=always submodule update --init vendor/xtask-core` が要る（既定の file transport の禁止）。
- 行末の注意：この環境は `core.autocrlf=true`。CRLF の blob を commit するときは `git -c core.autocrlf=false add` と `commit` を使う（でないと LF に戻る）。

## W1b（途中：lane T1〜T5 を merge 済み、lane T6 は差戻しで停止中）
- lane T1（T05・T06）：`6c4aa02e`・`8c20407d`。lane T2（T07・T08）：`a79c28b9`・`e9797b54`。lane T3（T09・T10・T24）：`8cd6ea4c`・`1139ee17`・`cd7cd657`。lane T4（T11・T12）：`4ed97960`・`a85f9542`。lane T5（T13・T14・T15）：`0a373ea5`・`43086f3d`・`c4e54a5a`。
- 統合 branch へ merge 済み（`69b8cd9a`・`66041131`・`f905e6b6`・`cb0162bf`・`b14bf84f`）。merge 後、kivact-docs を除く workspace の `cargo check --workspace --all-targets` は成功（error 0、warning 0）。
- lane T6（T16 完了、`1bb20616`）：T17 は設計者の差戻し（unsafe_code の lint。`rework/OPEN.md`）で停止。T17 の変更は worktree の stash（`T17 WIP`）。T18 は T17 に依存して停止。T19〜T21 は未着手。
- W1b の tier b は lane T6 が終わってから 1 回だけ行う（作業指示書 1.3）。W1b の push はまだ行っていない。
- 設計者の判断待ち（`rework/OPEN.md` の「差戻し」）：T05・T07・T09・T10 の 180 文字の印の行、T09 の R の扱い、T13 の完了条件の rg、T14 の README の段落、T17 の unsafe_code、T18 の依存。

## W1b 完了（merge 済み・tier b 成功・push 済み）
- lane T1〜T6 を統合 branch に merge（`69b8cd9a`・`66041131`・`f905e6b6`・`cb0162bf`・`b14bf84f`・`c5f9dd45`）。T09 の R 復元（`e13a418c`）、T13 の import 修正（`19974dbd`）。
- tier b（統合 branch、`V:/kv-target/rw-main`、試験は `--no-fail-fast`）：fmt・xtask（`--no-write`）・clippy（workspace all-targets all-features）・`kivact-docs` の check は成功。test は 1555 件成功、失敗 1 件（`chat_keyboard`、基点の失敗）、無視 36 件。
- xtask の legacy 印は 200 件（CP1 の台帳で閉じる対象）。
- W1b の push：`19974dbd`（origin の `refactor/kashiwade/fundamental-rework`）。
- 設計者の回答（`rework/OPEN.md`）で解消：T05・T07・T09・T10（180 字の上限は外した。L と R は切らない）、T13（完了条件の rg は crate 名）、T17（checked の allow に unsafe_code）。未決：T14（README の段落の置き場所）、T18（確認の試験 filter が実行 0 件）。

## W1c 完了（merge 済み・tier b 成功）
- lane T0（T22、`7c2b0460`）を merge（`3746d96d`）。`[rework.pending]` の state と docs-pairs を外し、`redundant_imports` を workspace の lint にした（未使用 use を消した）。
- tier b：fmt・xtask（`--no-write`）・clippy（workspace）・kivact-docs の check は成功。test は 1555 件成功、失敗 1 件（`chat_keyboard`、基点の失敗）、無視 36 件。
- T22 の完了条件 3 つ目の rg は、xtask 自身の旧い印の検出文字列に当たる。T01 と同じ扱いで受け入れた（`rework/OPEN.md` の「差戻しの判断」）。

## W2a 完了（merge 済み・tier b 成功・push 対象）
- lane A（S02〜S06）、lane S（S07〜S09）、lane R（R01〜R04）、lane U（U01〜U03）、lane K（K01〜K05）を統合 branch に merge（`4dfb8fe0`・`f21b4c95`・`505b7cc5`、A は W2a-A の merge、S・R・U・K の順）。
- K01 の fix（`ab399741`、K01 の付属書の新しい手順）、K03 の 1 項目（`NodeMode::Connecting` の handler の場所）は設計者の判断待ち（`rework/OPEN.md`）。
- tier b の過程で出た xtask の error を fix lane で直した：types の Role 行（`0f52ef4a`）、types の scene→shaped 規則（同）、i18n の module 循環（`2757bc1f`）、ui-creative の module 循環（`e3e54ab7`）、i18n の shared の Role 行（`cda5080d`）。すべて merge 済み。
- tier b（統合 branch）：fmt・xtask（`--no-write`）・clippy（workspace）・kivact-docs の check は成功。test は 1563 件成功、失敗 1 件（`chat_keyboard`、基点の失敗）、無視 36 件。
- 状態の印の逸脱（R03・K02・S04・S06 など）と、付属書の script の欠陥（S06）は `rework/OPEN.md` の差戻しに記録。

## W2b 完了（merge 済み・tier b 成功）
- lane C（C01・C02）、lane D1（D01）、lane D2（D02・D03・D04）を統合 branch に merge（`0a49fd0d`・`8d4acff9`・`59413203`）。
- C01 の StyleHook は設計者の (b) の形（`1e24383f`）で直した（`71bfe580`、`5e4c3463`）。D03 は設計者の (a)（二重の許容。`d23b397f`）。
- tier b の過程で出た xtask の error を fix lane で直した：facade `draw.rs` の glob を explicit な一覧に（`d30c894b`）、core の view の循環を leaf の trait module で（同）、layout の tree を leaf 群に分けて循環を解消（`9559f8aa`）、layout と core の Role 行を 80 字以内に（`2b4cb2db`・`cda5080d` の系）。すべて merge 済み。
- tier b（統合 branch）：fmt・xtask（`--no-write`）・clippy（workspace）・kivact-docs の check は成功。test は 1565 件成功、失敗 1 件（`chat_keyboard`、基点の失敗）、無視 36 件。
- 注記（`rework/OPEN.md`）：Linux の x11rb の未使用 import 2 件（D02 の所見）、kivact-svg の lib.rs の doc が `Offscreen` の所在（`kivact-core`）を古く書いている（C02 の所見、S 系の後で直す）。

## W3a 完了（merge 済み・tier b 成功）
- lane S（S10〜S12）、lane R（R07・R10）、lane C（C03・C04）、lane U（U04〜U08）、lane A1（A01）、lane A2（A02）を統合 branch に merge（`f0b8b993`・`0b491547`・`74398af0`・`c8de2982`・`cf6581f5`・`03338778`）。
- C03 の 4 件は設計者の回答（`506abdb6`）どおりに直した（`b8d666b0`）。C04 の試験の期待値は設計者の回答（b）どおりに直した（`0164769c`）。
- tier b の過程で出た xtask の error を fix lane で直した：virtual_list の leaf 化（`4f738708`）、mount.rs の leaf 化と gpu_interop の循環（`d73cc130`）、arvane-text のテスト module の Role と kivact-types の Role 行（`ad7267b4`）。すべて merge 済み。
- tier b（統合 branch）：fmt・xtask（`--no-write`）・clippy（workspace）・kivact-docs の check は成功。test は 1580 件成功、失敗 1 件（`chat_keyboard`、基点の失敗）、無視 36 件。外部 GPU の試験 `external::frames_match_in_four_places_on_dx12` は、1 回目の失敗（GPU 初期化の揺れ）の後、単独と 2 回目の tier b で通った。
- 注記（`rework/OPEN.md`）：R10・S12・C04 などの状態の印の逸脱、R07 の完了条件の rg の 2 件、S11 の文字の gradient の設計上の注意、C04 の試験の期待値（設計者が (b) で直した）。

## W3b 完了（merge 済み・tier b 成功）
- lane E1（E01・E02）、lane E2（E03・E04）を統合 branch に merge（`455b78de`・`5f06d47a`）。E01 は subagent が E2E の途中で終わったので、Orchestrator が E2E 13 件（すべて rc=0）を確かめて commit した（`f69f7a13`）。
- E04 の paint walk の失敗（visibility_hidden の試験、`display: contents` の経路）は E2 の fix で直した（`0e93e878`、merge `324e2e8c`）。
- tier b（統合 branch）：fmt・xtask（`--no-write`）・clippy（workspace）・kivact-docs の check は成功。test は 1580 件成功、失敗 1 件（`chat_keyboard`、基点の失敗）、無視 36 件。
- 注記（`rework/OPEN.md`）：E02 の SizeMotion の spring と reduced 時の step（振る舞いの注記）、E04 の木の編集と origin cache（設計者の判断待ち）。

## W4 完了（merge 済み・tier b 成功）
- lane C（C05）、E1、E2、A1、A2、U を統合 branch に merge（最後の lane 統合 `8d4a156d`、U は `b26716f3`）。
- tier b の過程で出た error を、次の順に直した。
  - xtask の Role 行 5 件：lane `wt/rw-W4-fix`（`9eb3db5a`、merge `902fc181`）。
  - resource_reclaim（stabilize の上限を pass の開始時に 1 度だけ決めていた）：lane `wt/rw-W4-R-fix`（`0937a649`、merge `2eff3e50`）。設計者の回答 1 のとおり、上限を loop の中で数え直す。kivact-runtime に試験 2 本を足した。
  - kivact-ui の module 構造（非葉 627 件）：lane `wt/rw-W4-U-fix` の batch 1/7〜7/7（`8f28886f`〜`9429591e`、merge `9929d520`）。X.rs は mod と pub use だけにし、コードは X/view.rs などの葉へ移した。
  - 設計者の patch（xtask-core の解決、tokens の型を X/tokens.rs へ、look_common と progress、radio_group の循環）：lane の commit `eda141df`（merge `f9f03d65`）。xtask-core は `afbfd83` を使う。
- tier b（統合 branch `f9f03d65`、`V:/kv-target/rw-main`）：fmt・xtask（`--no-write`、error 0 件）・clippy（workspace all-targets all-features）・kivact-docs の check は成功。test は 1585 件成功、失敗 1 件（`chat_keyboard`、基点の失敗）、無視 36 件。rg が無かったので、同じ確認を grep で行い、`xtask-allow(state-fields)` は 0 件。
- 注記（`rework/OPEN.md`）：xtask の legacy 印は 189 件（CP1 の台帳で閉じる）。e2e に名前の一致する試験が無い部品（number_input・tag_input・typing_indicator・spinner・avatar_group・badge・date_parts）がある。

## W4b 完了（merge 済み・tier b 成功）
- C06：lane `wt/rw-W4b-C` の commit `fd8413d8`、設計者の patch（`0cf2e9d6`：窓の通知を `WindowInfo` 全体の変化で送る。`SentWindows` に分ける）、merge `e132dc52`。tier a（fmt・check・clippy・xtask（`kivact-core`・`kivact-devtools-agent`）・e2e_native の `devtools`）は成功。
- E31：lane `wt/rw-W4b-E` の commit `9bb8d22d`、merge `fe684dcb`。tier a（engine の fmt・check・clippy・xtask・試験、core の check・clippy、e2e_core の 8 モジュール、e2e_ui の 2 モジュール）は成功。
- 設計者の回答（C06 の質問 4〜6、E31 の質問 7）：`rework/OPEN.md` の「設計者の回答（W4b・C06 の質問 4〜6、E31 の質問 7、2026-11-19）」節。
- tier b（統合 branch `fe684dcb`、`V:/kv-target/rw-main`）：fmt・xtask（`--no-write`、error 0 件）・clippy（workspace all-targets all-features）・kivact-docs の check・`xtask-allow(state-fields)` の grep（0 件）は成功。test は 1585 件成功、失敗 1 件（`chat_keyboard`、基点の失敗。`rework/OPEN.md` の「基点の失敗」節）。

## CP1 完了（tier c 成功・KPI 測定済み・後退は設計者へ）
- tier c（統合 branch `d615568c`、`V:/kv-target/rw-main`）：fmt・xtask（`--no-write`、error 0 件）・clippy（workspace all-targets all-features）・gpu-interop-producer の test・kivact-docs の check と doctest・`--no-default-features` の test・`check-features.sh`・`kivact`・`kivact-offscreen`・`arvane-render`・`arvane-compose`・`kivact-style` の `--no-default-features` check・wgpu/naga の除外は成功。
- test（all-features と no-default-features）：失敗は基点の `chat_keyboard` 1 件だけ（`rework/OPEN.md` の「基点の失敗」）。
- GPU（`KIVACT_TEST_GPU=1`）：e2e_core は 434 件成功、4 件失敗（すべて基点の失敗の 4 件）。e2e_native は全件成功。kivact-test の GPU は `--test screen -- gpu::` で 3 件成功（作業指示書の `--test gpu` は binary が無いので代えた。`rework/OPEN.md` の「Orchestrator の判断」）。
- KPI：`rework/kpi/CP1.md`・`CP1-r2.md`・`CP1-r3.md`（比は `*-ratio.md`）。3 回中 2 回で判定。後退と改善未達は `rework/OPEN.md` の「KPI の後退」「KPI の未達」に書き、設計者に返す。Orchestrator は直さない。
- 状態の台帳：`rework/state-ledger.md`（`legacy:` の印 194 件。AppModel の行に `text_ahead`・`font_generation` を含む）。付属書 L（閉じ方）は設計者が書く。

## W4b 完了（merge 済み・tier b 成功）
- C06：lane `wt/rw-W4b-C` の commit `fd8413d8`、設計者の patch（`0cf2e9d6`：窓の通知を `WindowInfo` 全体の変化で送る。`SentWindows` に分ける）、merge `e132dc52`。tier a（fmt・check・clippy・xtask（`kivact-core`・`kivact-devtools-agent`）・e2e_native の `devtools`）は成功。
- E31：lane `wt/rw-W4b-E` の commit `9bb8d22d`、merge `fe684dcb`。tier a（engine の fmt・check・clippy・xtask・試験、core の check・clippy、e2e_core の 8 モジュール、e2e_ui の 2 モジュール）は成功。
- 設計者の回答（C06 の質問 4〜6、E31 の質問 7）：`rework/OPEN.md` の「設計者の回答（W4b・C06 の質問 4〜6、E31 の質問 7、2026-11-19）」節。
- tier b（統合 branch `fe684dcb`、`V:/kv-target/rw-main`）：fmt・xtask（`--no-write`、error 0 件）・clippy（workspace all-targets all-features）・kivact-docs の check・`xtask-allow(state-fields)` の grep（0 件）は成功。test は 1585 件成功、失敗 1 件（`chat_keyboard`、基点の失敗。`rework/OPEN.md` の「基点の失敗」節）。

## W5a 完了（merge 済み・tier b 成功。R29 以降は設計者が実施）
- lane E1（E16）・E2（E12・E13）は Orchestrator が merge（`a5944729`・`1d6538f6`）。lane X（鎖）は R05〜R28 を Orchestrator、R29〜E20 をユーザーの指示で設計者が Haiku の subagent で実施し、merge `500f069e`。
- 設計者が鎖の途中で入れた fix（lane X の commit）：E13（文字色の transition の開始・目標の変え直しで下の描画を捨てる `2db1d53f`、動いている layer を含む layer の bounds を毎 frame 計算し直す・layer の root の原点が動いたら list を作り直す `990c6eef`）、E12（読み込み中の画像を含む scene の描画を保存しない `138ce9c3`、描画の key に大きさと own `c39764be`）、R07（thread の終わりの登録表の読み取りを try_with `c39764be`）、R25（tokio の import `955c301d`）、R27（on_exit_complete `af8b14a4`）、C09（#[component] の属性の二重と where の bound `486731f8`）、C11（消し忘れの試験 `c27fae56`）、C14（font の無い試験の host の fallback、arvane-text `a02a10f2`）、E17（api-shape と満杯の Enter `7b8a6cde`）、日本語の印を英語に（`f91271df`）。
- tier b の過程の fix（統合 branch）：component の例を部品に分ける（`2d92ff1c`）、doc の例の import（`65926f75`）。
- tier b（統合 branch `65926f75`、`V:/kv-target/rw-main`）：fmt・xtask（`--only local`、error 0 件）・clippy（workspace all-targets all-features、`--exclude kivact-docs`）・`xtask-allow(state-fields)` の rg（0 件）は成功。test（`--exclude gpu-interop-producer --exclude kivact-docs --no-fail-fast`）は 1614 件成功、失敗 1 件（`chat_keyboard`、基点の失敗）、無視 40 件。
- 注記：`rework/OPEN.md` の「W5a の記録（設計者、2026-11-20）」節。


## W5b 完了（merge 済み・tier b 成功）
- Arvane の鎖 A05〜A10 を lane `wt/rw-W5b-X` で実施（commit `da98c03a`・`cef68c88`・`12cd5129`・`c53801b2`・`4c6db854`・`f1d343db`）。Haiku の subagent が unit ごとに実施し、Orchestrator が commit と確認を見た。merge `8602f3d2`。
- 各 unit の後の確認：報告の範囲の E2E（ARV・EMBED・GPU-DEMAND・LIFE・TEXT・ASSETS・OFFSCREEN・DRAW・GPU-SHARE・DEVTOOLS）と、後退の確認 5 件（e2e_core・e2e_ui・kivact-test・e2e_kits・kivact-ui-soft）。すべて成功（kivact-ui-soft の既知の 1 件を除く）。
- 判断と注記：`rework/OPEN.md` の「Orchestrator の判断」（W5b の A05〜A10）と「設計者に確かめる点」。未了：root `Cargo.toml` の profile 行と `templates/basic/README.md`（A06 の所有の残り。G12 の範囲）、`kivact-devtools-agent` の kivact-types 依存（設計者の判断待ち）。
- tier b（統合 branch `8602f3d2`、`V:/kv-target/rw-main`）：fmt・xtask（`--only local`、error 0 件）・clippy（workspace all-targets all-features、`--exclude kivact-docs`）は成功。test（`--workspace --exclude gpu-interop-producer --exclude kivact-docs --no-fail-fast`）は 1613 件成功、失敗 1 件（`chat_keyboard`、基点の失敗）。`xtask-allow(state-fields)` の grep は 0 件。

## W6a 完了（merge 済み・tier b 成功）
- 鎖（lane X）：S13 の commit `0b22666e`、S14 の commit `1a27a7f1`、merge `632b85c1`。各 unit の後に tier a と後退の確認（e2e_core・e2e_ui・e2e_native・kivact-test の全件）。S14 の後の数：e2e_core 470 passed、e2e_ui 252 passed、e2e_native 45 passed、kivact-test 24 passed。失敗は無し。`e2e_kits` と `kivact-ui-soft` は、この wave で compile の対象外（作業指示書 2.1 と 4 章）なので走らせていない。
- tier b（統合 branch `632b85c1`、`V:/kv-target/rw-main`）：fmt・xtask（`--only local`、ok）・clippy（workspace、XK と kivact-docs を除外、-D warnings）・test（workspace、`--exclude gpu-interop-producer --exclude kivact-docs` と XK を除外、`--no-fail-fast`）は成功。test は 1261 件成功、失敗 0、無視 39。XK を除外したので、基点の失敗 `chat_keyboard`（kivact-ui-soft）は走っていない。`xtask-allow(state-fields)` の grep は 0 件。
- 判断：`rework/OPEN.md` の「Orchestrator の判断（W6a・鎖、2026-11-20）」節。
- W6a merged 632b85c1: S13・S14、tier b 成功。
- 次の wave：W6b。W6b の E の unit（E19〜E21・E14 など）の始めに、E17 の性能の行（16,384 行の文書の上下移動の p95 0.51 → 2.58 ms、`rework/PROGRESS.md` の W5a の記録）を設計者へ返す（W5a の記録の指示）。


## W6b（途中：lane A1・A2・E1・E2 は完了、lane X は差戻しで停止中、2026-11-21）
- lane A1（A11：`db068636`）、A2（A12：`d9abce37`、設計者の commit を含む）、E1（E19 `c0dbc924`・E21 `f8788b46`）、E2（E14 `f54a8875`・`f12b505b`、E32 `19fa977e`、E24 `8103101c`）は完了。merge は wave の終わりに行う（まだ統合 branch に入れていない）。
- E14 の `content_version` の再実行と後退の確認は実施済み（`V:/kv-target/logs-W6b-E2/verify-e14-summary.txt`、rc=0 が全件）。
- lane X：S15・S16・S17 まで完了（`0b554b35`・`67fdd8eb`・`b8470efa`・`bb13f33b`・`1307ae90`・`20a06b61`）。C19 は `79358493` まで進んだが、2 件の設計の判断が要るため停止（`rework/OPEN.md` の「差戻し（W6b・C19）」）。S18〜S23 は未着手。
- 次：設計者の答えを待つ。W6b の merge（X → E1 → E2 → A1 → A2）と tier b は lane X が終わってから。

## W6b 完了（merge 済み・tier b 成功）
- lane 鎖（X）：S15・S16・S17・C19・S17b・S18・S19・S20・S21・S22・S23 を merge `1e4ba32e`（lane の HEAD `73c2b19a`）。
- lane E1（E19・E21）：merge `9019f3b8`。lane E2（E14・E32・E24）：merge `10a3d738`。lane A1（A11）：merge `9e79aa16`。lane A2（A12）：merge `25949e78`。競合は無し。
- 設計者の回答と Orchestrator の判断：`rework/OPEN.md` の W6b の節（C19 の差戻し 2 件、S17b、S18 の期待値 8、S19 の navigation.rs、S20 の J1〜J5、S21・S22 の食い違い、S22 の確認事項）。
- tier b（統合、`V:/kv-target/rw-main`、W6a〜W9 の形：xtask は `--only local`、clippy と test から XK と XD を除外）：fmt・xtask（`--only local`）・clippy（`-D warnings`）は成功。test は 1303 件成功、失敗 0、無視 39。`xtask-allow(state-fields)` は 0 件（grep で確認。rg は無い）。ログは `V:/kv-target/logs-W6b-tierb/`。
- 未了：lane worktree（`W:/w/kv-wt/rw-W6b-{X,E1,E2,A1,A2}`）と branch（`wt/rw-W6b-*`）の削除、wave の終わりの push。権限の判定で拒否されたため、保留（設計者または利用者の判断を待つ）。
- 次：W7a（U20〜U25、E22・E23・E25・E26、A13、A14）。

## W6b の後片付け（設計者、2026-11-21）
- lane の worktree（`W:/w/kv-wt/rw-W6b-{X,E1,E2,A1,A2}`）と branch（`wt/rw-W6b-*`）は、設計者が利用者の許可を得て削除した（clean と merge 済みを確かめてから）。W6b の target（`V:/kv-target/rw-W6b-*`）は残してある。
- W6b の push は設計者が利用者の許可を得て済んだ（統合 `b1b2c3c2`）。

## W7a の進み（2026-11-21）
- 完了（lane の commit、統合へは未 merge）：U20 `8a96e844`、U21 `6abda8f2`、U25 `e2e0b710`、U22 `485acfee`、U23（WIP `a4e8303b`、設計者の印 `e23f39a8`。完了 commit は未）、E22 `750ad059`、E23 `6b66158b`、E25 `f0ed653b`（ime_select の sync を残す）、A13 `3a719498`、A14 `f5382490`。
- 残り：U23 の完了、U24（U1 lane）、E26（E1 lane）。W7a の lane の merge（U1 → E1 → A1 → A2）と tier b は、その後。
- W7b：A15 `08c1371d`（lane `wt/rw-W7b-A1`、W7a の A13 の上）。W7b の他の lane は W7a の完了を待つ。

## U24：Element::variant（2026-11-21、lane U1）
- 報告の xtask：`cargo xtask check --no-write --strict --only element-shadow,hand-element --crate kivact-ui` は exit 1。`hand-element` 130 件（W8 まで手書きの builder が残るため予定どおり）、`element-shadow` 0 件。通常の `cargo xtask check --no-write --crate kivact-ui` は exit 0（pending は従来どおり）。
- 残した行（U24 の完了条件に対する例外）：`dialog`・`menu`・`toast`・`hover_card`・`command_palette` の 5 つの builder は `impl Element` が無い（node を持たない）ため `Element::variant` を使えない。`pub fn variant` と `variants: Variants` field を残し、型だけ core の `Variants` にした（STOP を参照）。
- 内部の struct の `variants: Variants` field は builder の写しではないので残した（`rg "variants: Variants"` は 26 件。RowRender・GroupRender・Level・PanelPresentation・PageOptions など）。
- `inherited: Variants` field（date_input・time_input・disclosure）は親の group・親の input の look を子へ渡すための field（`Variants::merge` を core で `#[doc(hidden)] pub` にした）。

## W7a 完了（統合 merge 済み・tier b 成功、2026-11-21）
- 統合の merge：lane U1（U20・U21・U25・U22・U23・U24）、E1（E22・E23・E25・E26）、A1（A13）、A2（A14）。競合は無し。
- lane の commit：U1 `8270c564`・`c691dd01`、E1 `935ea765`・`f0ed653b`・`6b66158b`、A1 `3a719498`、A2 `f5382490`。
- tier b（統合、`V:/kv-target/rw-main`、W6〜W9 の形）：fmt・clippy（workspace、XK と XD を除外、`-D warnings`）・test（1324 件成功、失敗 0、無視 39）は成功。xtask `--only local` は、name-collision の規則を直した後（`67f7f48a`、設計者の決め (c)）に成功。`xtask-allow(state-fields)` は 0 件（grep）。
- 残りの判断（OPEN）：U24 の 5 builder の variant は W8 の U47・U48 で直す（設計者の決め）。E25 の ime_select の sync（設計者の決め）。
- 次：W7b（E27・E28・A15 完了。U32・U34・U35 完了。U36 は確認中。U30a は compile の残りで差戻し）。

## W7b の進み（2026-11-21）
- 完了（lane の commit、統合へは未 merge）：E27 `198ec9e5`、E28 `e908b61c`（lane `wt/rw-W7b-E1`）、A15 `08c1371d`（lane `wt/rw-W7b-A1`）、U32 `459b6715`（lane `wt/rw-W7b-U2`）、U34 `ca79381b` と U35 `7869702e`（lane `wt/rw-W7b-U3`）、U36 `288c6f6e`（lane `wt/rw-W7b-U4`）。
- 停止：U30a（lane `wt/rw-W7b-U1`、commit `fbad507e`、compile が通らない）。設計者の答えを待つ（OPEN「差戻し（W7b・U30a）」）。U30c・U30b・U31・U33 は U30a の後。
- W7b の merge と tier b は、U1 lane が終わってから。

## W7b 完了（統合 merge 済み・tier b 成功、2026-11-21）
- 統合の merge：U1（U30a・U30c・U30b・U31・U33）、U2（U32）、U3（U34・U35）、U4（U36）、E1（E27・E28）、A1（A15）。
- 統合の直し：`drawer_swipe.rs:125` を `crate::DialogKind::Drawer(edge)` に（設計者の (d)）。`interaction_dialog.rs` の `IntoValue` の import。`interaction_toggle.rs` の無効の trigger の試験を `is_disabled()` の assert（harness は無効の node への click を拒むため）。
- U30b の開閉の一方向（設計者の決め）：注記の event（`Prepare`・`Closing`・`Typed`）で値を先に渡し、開閉は `open.set` だけが決める。注記の残りは、同じ turn の hide で既定に戻す（`52468fd0`、設計者の (d)）。
- tier b（統合、`V:/kv-target/rw-main`、W6〜W9 の形）：fmt・clippy（workspace、XK と XD を除外、`-D warnings`）・test（1345 件成功、失敗 0、無視 39）・xtask `--only local` は成功。`xtask-allow(state-fields)` は 0 件（grep）。ログは `V:/kv-target/logs-W7b-tierb/`。
- 次：W8（U 4 lane、E29、A16、lane S は S22b → S22d）。

## 再開の手順（2026-11-21、利用者の PC 再起動による中断）

- **統合の HEAD**：`f3a67c1a`（ローカル。push していない。worktree は clean）。W6b・W7a・W7b は統合に merge 済み。W8 は全 lane を統合へ merge 済み（順 U2 → U1 → U3 → U4 → E1 → A2 → S）。W8 の a11y 修正 `2e90ced7`、carousel 修正 `7d2870cc`、U45 の残り `b20e2ef3`（merge）、menu の WIP `f3a67c1a`。
- **各 lane の状態**（branch `wt/rw-W8-*`、いずれも統合に merge 済み）：
  - U1：`a2dd779a`（U45 完了）。U40・U42・U44・U45・U46 は完了。
  - U2：`42874813`（U41・U47・U48 完了）。
  - U3：`46b10e0a`（U43 の checkpoint・U49・U50 `6c4b20cb`・U51 完了。U51 の carousel の初回 scroll は設計者の (a) で統合の `7d2870cc` に入った）。worktree の 10 file は内容の差が 0 行の行末だけ（`git status` は M と出すが、`--ignore-cr-at-eol` で 0 行。作業ツリーは CRLF、index は LF）。
  - U4：`130d16bb`（U53 の code_block・chat_thread、U54 完了）。
  - E1：`7b7b87df`（E29 完了）。A2：`508a49fe`（A16 完了）。S：`104c172e`（S22b・S22d 完了）。
  - W7a・W7b の lane も統合へ merge 済み。W6b の lane は設計者が後片付け済み。
- **途中の作業（統合の上）**：
  1. W8 の tier b（`V:/kv-target/logs-W8-tierb/`）：fmt・xtask・clippy は通過。test は 2 件が落ちていた。(a) a11y（`a_text_update_publishes_only_the_text`）は `2e90ced7` で直した（Bump 8 → 6、上限 6 は変えず）。直した後の試験の結果は未確認（W8 の tier b を再実行する）。(b) menu（`ui_components::menu_renders_nested_levels_and_custom_trigger_keeps_its_style`）は、custom trigger に label を書かない形（`f3a67c1a`、WIP）に直したが、試験の driver（`kivact-test` の `name_of`）が surface の名前を labelled_by から読めず、Menu "" で落ちる。**設計者が決める**：(a) driver（所有の外）か engine が labelled_by を名前として解決する、(b) surface の名前を trigger の内容（Text）から読む。
  2. carousel（設計者の (a)）：`7d2870cc` で済み。試験 `a_carousel_opens_at_its_initial_slide` は直す前に落ち、直した後に通る。
  3. U54 の確認：`cargo xtask check --no-write --strict --only interaction-view,ui-machine,unit-machine,hand-element,element-shadow --crate kivact-ui` は ui-machine の 1 件（`combobox/labels.rs:54`、Signal/Var write inside an effect closure）が残る。U54 の所有の file の error は 0。この 1 件の所有 unit は未決定（設計者に確認する）。
- **次にすること**：(1) 設計者の menu の答え（(a) か (b)）を実施し、menu の試験を通す。(2) W8 の tier b（W6 の形、`V:/kv-target/logs-W8-tierb/run.sh`）を再実行し、test が全部通ることを確かめる。(3) U54 の残りの ui-machine（`combobox/labels.rs:54`）の所有を決める。(4) W8 を完了として PROGRESS に記録する。(5) W8L（L1〜L6）→ CP2（KPI、`--strict`）→ W9（K 5 lane と K6）→ W10（K と L7〜L11）→ W11a → W11b → W11c → CP3。
- **注意**：stash は使わない（worktree 間で共有されていて取り違えが起きた）。kit（`kivact-ui-*`）は compile しない（W9 の所有）。
