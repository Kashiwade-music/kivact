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

