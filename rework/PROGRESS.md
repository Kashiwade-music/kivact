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
